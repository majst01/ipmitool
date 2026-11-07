package ipmicore

// Interface is a transport capable of carrying IPMI requests to a BMC. It is
// the Go counterpart to the Rust `ipmi_core::interface::Interface` trait.
type Interface interface {
	// Name is the short interface name (e.g. "lanplus").
	Name() string
	// Open opens the transport and, where applicable, establishes a session.
	Open() error
	// Close closes the transport, terminating any active session.
	Close()
	// IsOpen reports whether the transport is currently open.
	IsOpen() bool
	// SendRecv sends a request and waits for the corresponding response.
	SendRecv(req *Request) (*Response, error)
	// SessionParams provides access to the configured session parameters.
	SessionParams() *SessionParams
	// TargetLun is the destination LUN for raw commands.
	TargetLun() uint8
	// TargetChannel is the destination channel for raw commands.
	TargetChannel() uint8
	// SolActivate activates an SOL payload.
	SolActivate(req *SolActivateRequest) (*SolActivateResponse, error)
	// SolDeactivate deactivates an SOL payload.
	SolDeactivate(instance uint8) error
	// SolSend sends an SOL packet and returns the acknowledgement.
	SolSend(pkt *SolPacket) (*SolPacket, error)
	// SolRecv receives an SOL packet.
	SolRecv() (*SolPacket, error)
	// RawFd returns the raw file descriptor for the underlying transport, used
	// to poll for inbound data while in an interactive SOL session. The second
	// result is false when unsupported.
	RawFd() (int, bool)
}

// UnsupportedSolError is returned by transports that do not implement SOL.
func UnsupportedSolError() error {
	return NewUnsupported("Serial-over-LAN is not supported by this interface")
}
