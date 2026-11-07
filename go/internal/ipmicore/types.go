package ipmicore

// Request is an IPMI request message.
type Request struct {
	Netfn uint8
	Lun   uint8
	Cmd   uint8
	Data  []byte
}

// NewRequest builds a request with no data.
func NewRequest(netfn, cmd uint8) *Request {
	return &Request{Netfn: netfn, Cmd: cmd}
}

// WithData sets the request data.
func (r *Request) WithData(data []byte) *Request { r.Data = data; return r }

// WithLun sets the destination LUN.
func (r *Request) WithLun(lun uint8) *Request { r.Lun = lun; return r }

// Response is an IPMI response message.
type Response struct {
	Ccode uint8
	Data  []byte
	Netfn uint8
	Cmd   uint8
	RqSeq uint8
}

// OK reports whether the completion code indicates success.
func (r *Response) OK() bool { return r.Ccode == 0 }

// CipherSuite identifies a standard cipher suite's algorithm triple.
type CipherSuite struct {
	ID           uint8
	AuthAlg      uint8
	IntegrityAlg uint8
	CryptAlg     uint8
}

// CipherSuiteFromID looks up the algorithms for a standard cipher suite id.
// The second result is false for reserved / unsupported suites.
//
// NOTE: this mirrors the Rust `CipherSuite::from_id`, which (as recorded in
// rust/spec-drift.md) omits the spec's suites 18 and 19.
func CipherSuiteFromID(id uint8) (CipherSuite, bool) {
	var auth, integrity, crypt uint8
	switch id {
	case 0:
		auth, integrity, crypt = AuthRakpNone, IntegrityNone, CryptNone
	case 1:
		auth, integrity, crypt = AuthRakpHmacSha1, IntegrityNone, CryptNone
	case 2:
		auth, integrity, crypt = AuthRakpHmacSha1, IntegrityHmacSha196, CryptNone
	case 3:
		auth, integrity, crypt = AuthRakpHmacSha1, IntegrityHmacSha196, CryptAesCbc128
	case 4:
		auth, integrity, crypt = AuthRakpHmacSha1, IntegrityHmacSha196, CryptXrc4128
	case 5:
		auth, integrity, crypt = AuthRakpHmacSha1, IntegrityHmacSha196, CryptXrc440
	case 6:
		auth, integrity, crypt = AuthRakpHmacMd5, IntegrityNone, CryptNone
	case 7:
		auth, integrity, crypt = AuthRakpHmacMd5, IntegrityHmacMd5128, CryptNone
	case 8:
		auth, integrity, crypt = AuthRakpHmacMd5, IntegrityHmacMd5128, CryptAesCbc128
	case 9:
		auth, integrity, crypt = AuthRakpHmacMd5, IntegrityHmacMd5128, CryptXrc4128
	case 10:
		auth, integrity, crypt = AuthRakpHmacMd5, IntegrityHmacMd5128, CryptXrc440
	case 11:
		auth, integrity, crypt = AuthRakpHmacMd5, IntegrityMd5128, CryptNone
	case 12:
		auth, integrity, crypt = AuthRakpHmacMd5, IntegrityMd5128, CryptAesCbc128
	case 13:
		auth, integrity, crypt = AuthRakpHmacMd5, IntegrityMd5128, CryptXrc4128
	case 14:
		auth, integrity, crypt = AuthRakpHmacMd5, IntegrityMd5128, CryptXrc440
	case 15:
		auth, integrity, crypt = AuthRakpHmacSha256, IntegrityNone, CryptNone
	case 16:
		auth, integrity, crypt = AuthRakpHmacSha256, IntegrityHmacSha256128, CryptNone
	case 17:
		auth, integrity, crypt = AuthRakpHmacSha256, IntegrityHmacSha256128, CryptAesCbc128
	default:
		return CipherSuite{}, false
	}
	return CipherSuite{ID: id, AuthAlg: auth, IntegrityAlg: integrity, CryptAlg: crypt}, true
}

// AuthcodeLen returns the number of bytes emitted for an integrity authcode
// with this suite.
func (c CipherSuite) AuthcodeLen() int {
	switch c.IntegrityAlg {
	case IntegrityHmacSha196:
		return 12
	case IntegrityHmacMd5128, IntegrityMd5128:
		return 16
	case IntegrityHmacSha256128:
		return 16
	default:
		return 0
	}
}

// SessionParams holds session parameters configured on the command line,
// mirroring `struct ipmi_session_params`.
type SessionParams struct {
	Hostname       *string
	Username       [17]byte
	Password       [IPMIAuthcodeBufferSize + 1]byte
	AuthtypeSet    *uint8
	Privlvl        uint8
	CipherSuite    *uint8
	Port           uint16
	Retry          uint32
	Timeout        uint32
	Kg             [IPMIKgBufferSize]byte
	Lookupbit      uint8
	SolEscapeChar  byte
	TargetLun      uint8
	TargetAddr     uint8
	TargetChannel  uint8
	TransitAddr    uint8
	TransitChannel uint8
	MyAddr         uint32
}

// DefaultSessionParams mirrors `SessionParams::default()`.
func DefaultSessionParams() SessionParams {
	return SessionParams{
		Privlvl:       SessionPrivAdmin,
		Lookupbit:     0x10,
		SolEscapeChar: '~',
		MyAddr:        uint32(BmcSlaveAddr),
	}
}

// SetUsername stores a username, truncating to 16 bytes and NUL padding.
func (p *SessionParams) SetUsername(username string) {
	p.Username = [17]byte{}
	n := min(len(username), 16)
	copy(p.Username[:n], username[:n])
}

// SetPassword stores a password, truncating to 20 bytes and NUL padding.
func (p *SessionParams) SetPassword(password string) {
	p.Password = [IPMIAuthcodeBufferSize + 1]byte{}
	n := min(len(password), IPMIAuthcodeBufferSize)
	copy(p.Password[:n], password[:n])
}

// UsernameBytes returns the username up to the first NUL.
func (p *SessionParams) UsernameBytes() []byte {
	len := 17
	for i, b := range p.Username {
		if b == 0 {
			len = i
			break
		}
	}
	return p.Username[:len]
}

// PasswordBytes returns the fixed 20-byte authentication key buffer.
func (p *SessionParams) PasswordBytes() *[IPMIAuthcodeBufferSize]byte {
	return (*[IPMIAuthcodeBufferSize]byte)(p.Password[:IPMIAuthcodeBufferSize])
}
