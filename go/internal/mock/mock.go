// Package mock provides a scriptable, in-memory ipmicore.Interface plus output
// capture helpers, so command logic can be exercised end-to-end without a real
// BMC. It is used only by tests.
package mock

import (
	"os"
	"sync"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
)

// Handler produces the response (or transport error) for a request. Returning
// a nil error and nil response yields an empty successful response.
type Handler func(req *ipmicore.Request) (*ipmicore.Response, error)

// Interface is a scriptable ipmicore.Interface. It records every request passed
// to SendRecv and answers using the configured Handler. SOL operations are
// unsupported, matching a transport without serial-over-LAN.
type Interface struct {
	// Params is returned by SessionParams and drives TargetLun/TargetChannel.
	Params ipmicore.SessionParams

	// Requests records every request handed to SendRecv, in order.
	Requests []ipmicore.Request

	// OpenErr, when non-nil, is returned by Open.
	OpenErr error

	// SolActivateFn, when set, overrides the default unsupported result.
	SolActivateFn func(*ipmicore.SolActivateRequest) (*ipmicore.SolActivateResponse, error)
	// SolDeactivateFn, when set, overrides the default unsupported result.
	SolDeactivateFn func(uint8) error
	// SolSendFn, when set, overrides the default unsupported result.
	SolSendFn func(*ipmicore.SolPacket) (*ipmicore.SolPacket, error)
	// SolRecvFn, when set, overrides the default unsupported result.
	SolRecvFn func() (*ipmicore.SolPacket, error)

	handler Handler
	opened  bool
}

// New builds a mock whose requests are answered by handler. A nil handler
// answers every request with an empty success.
func New(handler Handler) *Interface {
	if handler == nil {
		handler = func(*ipmicore.Request) (*ipmicore.Response, error) { return &ipmicore.Response{}, nil }
	}
	return &Interface{
		Params:  ipmicore.DefaultSessionParams(),
		handler: handler,
	}
}

// Name implements ipmicore.Interface.
func (m *Interface) Name() string { return "mock" }

// Open implements ipmicore.Interface.
func (m *Interface) Open() error {
	if m.OpenErr != nil {
		return m.OpenErr
	}
	m.opened = true
	return nil
}

// Close implements ipmicore.Interface.
func (m *Interface) Close() { m.opened = false }

// IsOpen implements ipmicore.Interface.
func (m *Interface) IsOpen() bool { return m.opened }

// SendRecv records req and dispatches it to the handler.
func (m *Interface) SendRecv(req *ipmicore.Request) (*ipmicore.Response, error) {
	m.Requests = append(m.Requests, *req)
	rsp, err := m.handler(req)
	if err != nil {
		return nil, err
	}
	if rsp == nil {
		rsp = &ipmicore.Response{}
	}
	return rsp, nil
}

// SessionParams implements ipmicore.Interface.
func (m *Interface) SessionParams() *ipmicore.SessionParams { return &m.Params }

// TargetLun implements ipmicore.Interface.
func (m *Interface) TargetLun() uint8 { return m.Params.TargetLun }

// TargetChannel implements ipmicore.Interface.
func (m *Interface) TargetChannel() uint8 { return m.Params.TargetChannel }

// SolActivate implements ipmicore.Interface (unsupported unless overridden).
func (m *Interface) SolActivate(req *ipmicore.SolActivateRequest) (*ipmicore.SolActivateResponse, error) {
	if m.SolActivateFn != nil {
		return m.SolActivateFn(req)
	}
	return nil, ipmicore.UnsupportedSolError()
}

// SolDeactivate implements ipmicore.Interface (unsupported unless overridden).
func (m *Interface) SolDeactivate(instance uint8) error {
	if m.SolDeactivateFn != nil {
		return m.SolDeactivateFn(instance)
	}
	return ipmicore.UnsupportedSolError()
}

// SolSend implements ipmicore.Interface (unsupported unless overridden).
func (m *Interface) SolSend(pkt *ipmicore.SolPacket) (*ipmicore.SolPacket, error) {
	if m.SolSendFn != nil {
		return m.SolSendFn(pkt)
	}
	return nil, ipmicore.UnsupportedSolError()
}

// SolRecv implements ipmicore.Interface (unsupported unless overridden).
func (m *Interface) SolRecv() (*ipmicore.SolPacket, error) {
	if m.SolRecvFn != nil {
		return m.SolRecvFn()
	}
	return nil, ipmicore.UnsupportedSolError()
}

// RawFd implements ipmicore.Interface (unsupported).
func (m *Interface) RawFd() (int, bool) { return 0, false }

// ---------------------------------------------------------------------------
// Response builders.
// ---------------------------------------------------------------------------

// OK builds a successful response carrying data.
func OK(data ...byte) *ipmicore.Response {
	return &ipmicore.Response{Ccode: 0, Data: data}
}

// CCode builds a response carrying a completion code and no data.
func CCode(code uint8) *ipmicore.Response { return &ipmicore.Response{Ccode: code} }

// OKData is OK with a slice rather than variadic bytes.
func OKData(data []byte) *ipmicore.Response { return &ipmicore.Response{Data: data} }

// ---------------------------------------------------------------------------
// Router: a convenient Handler that dispatches on (netfn, cmd).
// ---------------------------------------------------------------------------

// routeKey identifies a request by network function and command.
type routeKey struct{ netfn, cmd uint8 }

// Router dispatches requests to per-(netfn, cmd) handlers. Requests with no
// matching route get an empty successful response.
type Router struct {
	routes map[routeKey]Handler
}

// NewRouter returns an empty router.
func NewRouter() *Router { return &Router{routes: map[routeKey]Handler{}} }

// On registers a handler for (netfn, cmd).
func (r *Router) On(netfn, cmd uint8, h Handler) *Router {
	r.routes[routeKey{netfn, cmd}] = h
	return r
}

// Static registers a fixed response for (netfn, cmd).
func (r *Router) Static(netfn, cmd uint8, rsp *ipmicore.Response) *Router {
	return r.On(netfn, cmd, func(*ipmicore.Request) (*ipmicore.Response, error) {
		return rsp, nil
	})
}

// Handle answers a request, satisfying the Handler signature.
func (r *Router) Handle(req *ipmicore.Request) (*ipmicore.Response, error) {
	if h, ok := r.routes[routeKey{req.Netfn, req.Cmd}]; ok {
		return h(req)
	}
	return &ipmicore.Response{}, nil
}

// InterfaceWith builds a mock wired to this router and returns both.
func (r *Router) InterfaceWith() *Interface { return New(r.Handle) }

// ---------------------------------------------------------------------------
// Output capture.
// ---------------------------------------------------------------------------

var captureMu sync.Mutex

// Capture runs fn while redirecting os.Stdout and os.Stderr to temporary files,
// returning what each received. The command packages print through the global
// os.Stdout/os.Stderr, so this must be called from a single test at a time;
// the package-level mutex enforces that within a process.
func Capture(fn func()) (stdout, stderr string) {
	captureMu.Lock()
	defer captureMu.Unlock()

	outFile, err := os.CreateTemp("", "mock-stdout-*")
	if err != nil {
		panic(err)
	}
	errFile, err := os.CreateTemp("", "mock-stderr-*")
	if err != nil {
		panic(err)
	}
	outName, errName := outFile.Name(), errFile.Name()
	defer os.Remove(outName)
	defer os.Remove(errName)

	oldOut, oldErr := os.Stdout, os.Stderr
	os.Stdout, os.Stderr = outFile, errFile
	defer func() {
		os.Stdout, os.Stderr = oldOut, oldErr
	}()

	fn()

	_ = outFile.Sync()
	_ = errFile.Sync()
	out, _ := os.ReadFile(outName)
	errBytes, _ := os.ReadFile(errName)
	_ = outFile.Close()
	_ = errFile.Close()
	return string(out), string(errBytes)
}
