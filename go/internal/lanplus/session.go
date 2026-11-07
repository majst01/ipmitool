package lanplus

// RMCP+ session state and key derivation (RAKP). Mirrors the RAKP logic in
// `src/plugins/lanplus/lanplus.c` and `lanplus_crypt.c`.

import "codeberg.org/ipmitool/ipmitool/internal/ipmicore"

// SessionState describes session negotiation states, mirroring
// `enum LANPLUS_SESSION_STATE`.
type SessionState int

const (
	SessionStatePresession SessionState = iota
	SessionStateOpenSessionSent
	SessionStateOpenSessionReceived
	SessionStateRakp1Sent
	SessionStateRakp2Received
	SessionStateRakp3Sent
	SessionStateActive
	SessionStateCloseSent
)

// V2Data is IPMI v2 / RMCP+ specific session data.
type V2Data struct {
	State                 SessionState
	RequestedAuthAlg      uint8
	RequestedIntegrityAlg uint8
	RequestedCryptAlg     uint8
	AuthAlg               uint8
	IntegrityAlg          uint8
	CryptAlg              uint8
	MaxPrivLevel          uint8
	ConsoleID             uint32
	BMCID                 uint32
	ConsoleRand           [16]byte
	BMCRand               [16]byte
	BMCGUID               [16]byte
	RequestedRole         uint8
	Rakp2ReturnCode       uint8
	SIK                   []byte
	K1                    []byte
	K2                    []byte
}

// DefaultV2Data returns the same field values as `V2Data::default()`.
func DefaultV2Data() V2Data {
	return V2Data{
		State:                 SessionStatePresession,
		RequestedAuthAlg:      ipmicore.AuthRakpNone,
		RequestedIntegrityAlg: ipmicore.IntegrityNone,
		RequestedCryptAlg:     ipmicore.CryptNone,
		AuthAlg:               ipmicore.AuthRakpNone,
		IntegrityAlg:          ipmicore.IntegrityNone,
		CryptAlg:              ipmicore.CryptNone,
		MaxPrivLevel:          0,
		ConsoleID:             0,
		BMCID:                 0,
		ConsoleRand:           [16]byte{},
		BMCRand:               [16]byte{},
		BMCGUID:               [16]byte{},
		RequestedRole:         0,
		Rakp2ReturnCode:       0,
		SIK:                   nil,
		K1:                    nil,
		K2:                    nil,
	}
}

// Session is a live RMCP+ session.
type Session struct {
	State   SessionState
	Timeout uint32
	// Authcode is the user authentication key (Kuid), i.e. the zero padded
	// password.
	Authcode [ipmicore.IPMIAuthcodeBufferSize]byte
	OutSeq   uint32
	V2       V2Data
}

// NewSession builds a fresh pre-session Session.
func NewSession(timeout uint32, authcode [ipmicore.IPMIAuthcodeBufferSize]byte) *Session {
	return &Session{
		State:    SessionStatePresession,
		Timeout:  timeout,
		Authcode: authcode,
		OutSeq:   1,
		V2:       DefaultV2Data(),
	}
}

// Active reports whether the session has reached the Active state.
func (s *Session) Active() bool {
	return s.V2.State == SessionStateActive
}

// AuthcodeLen returns the integrity authcode length for the negotiated
// algorithm.
func (s *Session) AuthcodeLen() int {
	switch s.V2.IntegrityAlg {
	case ipmicore.IntegrityHmacSha196:
		return 12
	case ipmicore.IntegrityHmacMd5128, ipmicore.IntegrityMd5128:
		return 16
	case ipmicore.IntegrityHmacSha256128:
		return 16
	default:
		return 0
	}
}

// Rakp2Matches verifies the key exchange auth code the BMC sent in RAKP 2.
func (s *Session) Rakp2Matches(bmcMac, username []byte) bool {
	if s.V2.AuthAlg == ipmicore.AuthRakpNone {
		return true
	}
	buf := make([]byte, 0, 58+len(username))
	buf = append(buf, le32(s.V2.ConsoleID)...) // SIDm
	buf = append(buf, le32(s.V2.BMCID)...)     // SIDc
	buf = append(buf, s.V2.ConsoleRand[:]...)  // Rm
	buf = append(buf, s.V2.BMCRand[:]...)      // Rc
	buf = append(buf, s.V2.BMCGUID[:]...)      // GUIDc
	buf = append(buf, s.V2.RequestedRole)      // ROLEm
	buf = append(buf, byte(len(username)))     // ULENGTHm
	buf = append(buf, username...)

	mac := hmacAuth(s.V2.AuthAlg, s.Authcode[:], buf)
	return len(bmcMac) >= len(mac) && byteSliceEqual(bmcMac[:len(mac)], mac)
}

// Rakp4Matches verifies the integrity check value the BMC sent in RAKP 4.
func (s *Session) Rakp4Matches(bmcMac []byte) bool {
	if s.V2.AuthAlg == ipmicore.AuthRakpNone {
		return true
	}
	buf := make([]byte, 0, 36)
	buf = append(buf, s.V2.ConsoleRand[:]...) // Rm
	buf = append(buf, le32(s.V2.BMCID)...)    // SIDc
	buf = append(buf, s.V2.BMCGUID[:]...)     // GUIDc

	mac := hmacAuth(s.V2.AuthAlg, s.V2.SIK, buf)
	cmpLen := 12
	if s.V2.AuthAlg == ipmicore.AuthRakpHmacMd5 {
		cmpLen = 16
	}
	return len(bmcMac) >= cmpLen && len(mac) >= cmpLen &&
		byteSliceEqual(bmcMac[:cmpLen], mac[:cmpLen])
}

// GenerateRakp3Authcode generates the RAKP 3 key exchange auth code.
func (s *Session) GenerateRakp3Authcode(username []byte) []byte {
	if s.V2.AuthAlg == ipmicore.AuthRakpNone {
		return nil
	}
	buf := make([]byte, 0, 22+len(username))
	buf = append(buf, s.V2.BMCRand[:]...)      // Rc
	buf = append(buf, le32(s.V2.ConsoleID)...) // SIDm
	buf = append(buf, s.V2.RequestedRole)      // ROLEm
	buf = append(buf, byte(len(username)))     // ULENGTHm
	buf = append(buf, username...)
	return hmacAuth(s.V2.AuthAlg, s.Authcode[:], buf)
}

// GenerateSIK derives the session integrity key (SIK).
func (s *Session) GenerateSIK(kg *[ipmicore.IPMIKgBufferSize]byte, username []byte) {
	s.V2.SIK = nil
	if s.V2.AuthAlg == ipmicore.AuthRakpNone {
		return
	}
	buf := make([]byte, 0, 34+len(username))
	buf = append(buf, s.V2.ConsoleRand[:]...) // Rm
	buf = append(buf, s.V2.BMCRand[:]...)     // Rc
	buf = append(buf, s.V2.RequestedRole)     // ROLEm
	buf = append(buf, byte(len(username)))    // ULENGTHm
	buf = append(buf, username...)

	var key []byte
	if kg[0] != 0 {
		key = kg[:ipmicore.IPMIAuthcodeBufferSize]
	} else {
		key = s.Authcode[:]
	}
	s.V2.SIK = hmacAuth(s.V2.AuthAlg, key, buf)
}

// GenerateK1 derives K1 (message integrity key).
func (s *Session) GenerateK1() {
	if s.V2.AuthAlg == ipmicore.AuthRakpNone {
		s.V2.K1 = make([]byte, 20)
		for i := range s.V2.K1 {
			s.V2.K1[i] = 0x01
		}
		return
	}
	s.V2.K1 = hmacAuth(s.V2.AuthAlg, s.V2.SIK, repeatByte(0x01, 20))
}

// GenerateK2 derives K2 (confidentiality key).
func (s *Session) GenerateK2() {
	if s.V2.AuthAlg == ipmicore.AuthRakpNone {
		s.V2.K2 = make([]byte, 20)
		for i := range s.V2.K2 {
			s.V2.K2[i] = 0x02
		}
		return
	}
	s.V2.K2 = hmacAuth(s.V2.AuthAlg, s.V2.SIK, repeatByte(0x02, 20))
}

// NextOutSeq takes the next outgoing sequence number (never zero).
func (s *Session) NextOutSeq() uint32 {
	seq := s.OutSeq
	s.OutSeq++
	if s.OutSeq == 0 {
		s.OutSeq = 1
	}
	return seq
}

// le32 encodes a uint32 little-endian into a fresh slice.
func le32(v uint32) []byte {
	return []byte{byte(v), byte(v >> 8), byte(v >> 16), byte(v >> 24)}
}

// repeatByte returns n copies of b.
func repeatByte(b byte, n int) []byte {
	out := make([]byte, n)
	for i := range out {
		out[i] = b
	}
	return out
}

// byteSliceEqual reports whether a and b hold the same bytes.
func byteSliceEqual(a, b []byte) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}
