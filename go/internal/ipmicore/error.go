package ipmicore

import "fmt"

// ErrorKind classifies an Error.
type ErrorKind int

const (
	KindIo ErrorKind = iota
	KindResolution
	KindTimeout
	KindSession
	KindCompletion
	KindProtocol
	KindInvalid
	KindUnsupported
)

// Error is the Go counterpart to the Rust `ipmi_core::error::Error` enum.
type Error struct {
	Kind    ErrorKind
	Msg     string
	Code    uint8  // completion code (KindCompletion)
	Command string // command name (KindCompletion)
	Wrapped error  // underlying error (KindIo)
}

func (e *Error) Error() string {
	switch e.Kind {
	case KindIo:
		if e.Wrapped != nil {
			return e.Wrapped.Error()
		}
		return e.Msg
	case KindResolution:
		return fmt.Sprintf("could not resolve host %s", e.Msg)
	case KindTimeout:
		return "no response from BMC (timeout)"
	case KindSession:
		return "session error: " + e.Msg
	case KindCompletion:
		if e.Command != "" {
			return fmt.Sprintf("%s failed: %s", e.Command, CompletionCodeStr(e.Code))
		}
		return CompletionCodeStr(e.Code)
	case KindProtocol:
		return "protocol error: " + e.Msg
	case KindInvalid:
		return "invalid argument: " + e.Msg
	case KindUnsupported:
		return "unsupported: " + e.Msg
	default:
		return e.Msg
	}
}

// Unwrap exposes the wrapped IO error.
func (e *Error) Unwrap() error { return e.Wrapped }

// ErrTimeout is the canonical "no response" error.
var ErrTimeout = &Error{Kind: KindTimeout}

// NewResolution builds a hostname-resolution error.
func NewResolution(host string) *Error { return &Error{Kind: KindResolution, Msg: host} }

// NewSession builds a session-negotiation error.
func NewSession(msg string) *Error { return &Error{Kind: KindSession, Msg: msg} }

// NewCompletion builds a non-zero completion-code error.
func NewCompletion(code uint8, command string) *Error {
	return &Error{Kind: KindCompletion, Code: code, Command: command}
}

// NewProtocol builds a protocol error.
func NewProtocol(msg string) *Error { return &Error{Kind: KindProtocol, Msg: msg} }

// NewInvalid builds an invalid-argument error.
func NewInvalid(msg string) *Error { return &Error{Kind: KindInvalid, Msg: msg} }

// NewUnsupported builds an unsupported-feature error.
func NewUnsupported(msg string) *Error { return &Error{Kind: KindUnsupported, Msg: msg} }

// NewIo wraps an IO error.
func NewIo(err error) *Error { return &Error{Kind: KindIo, Wrapped: err} }
