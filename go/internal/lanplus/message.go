package lanplus

// RMCP+ wire format: message construction, confidentiality, integrity and
// parsing. This mirrors `ipmi_lanplus_build_v2x_msg`, `getIpmiPayloadWireRep`
// and the `read_*` functions from the C implementation.

import (
	"errors"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
)

// Session header offsets (table 13-8).
const (
	offAuthtype    = 0x04
	offPayloadType = 0x05
	offSessionID   = 0x06
	offPayloadSize = 0x0e
	offPayload     = 0x10
)

// AESBlockSize is the AES block size.
const AESBlockSize = 0x10

// SessionHeader is a parsed RMCP+ session header.
type SessionHeader struct {
	Authtype      uint8
	PayloadType   uint8
	Encrypted     bool
	Authenticated bool
	SessionID     uint32
	Msglen        uint16
	// Offset is the offset of the payload within the datagram.
	Offset int
}

// RequestPayload builds the IPMI message wire representation (non-bridged).
func RequestPayload(req *ipmicore.Request, myAddr, rqSeq uint8) []byte {
	if myAddr == 0 {
		myAddr = ipmicore.BmcSlaveAddr
	}
	m := make([]byte, 0, len(req.Data)+7)
	// Only local requests (target == my address) are supported at present.
	_ = myAddr
	m = append(m, ipmicore.BmcSlaveAddr)      // rsAddr
	m = append(m, (req.Netfn<<2)|(req.Lun&3)) // netfn/lun
	c := ipmicore.IpmiCsum(m[:2])
	m = append(m, c)
	cs := len(m)
	m = append(m, ipmicore.RemoteSWID) // rqAddr
	m = append(m, rqSeq<<2)
	m = append(m, req.Cmd)
	m = append(m, req.Data...)
	c = ipmicore.IpmiCsum(m[cs:])
	m = append(m, c)
	return m
}

// EncryptPayload encrypts a payload with the confidentiality algorithm, adding
// the confidentiality header (IV) and trailer.
func EncryptPayload(cryptAlg uint8, k2, input []byte) []byte {
	if cryptAlg == ipmicore.CryptNone {
		return append([]byte(nil), input...)
	}
	if cryptAlg != ipmicore.CryptAesCbc128 {
		panic("EncryptPayload: unexpected confidentiality algorithm")
	}
	mod := (len(input) + 1) % AESBlockSize
	pad := 0
	if mod != 0 {
		pad = AESBlockSize - mod
	}
	padded := append([]byte(nil), input...)
	for i := 0; i < pad; i++ {
		padded = append(padded, byte(i+1))
	}
	padded = append(padded, byte(pad))

	iv := RandomBytes(AESBlockSize)
	ct := aesCBCEncrypt(k2[:16], iv, padded)
	out := append([]byte(nil), iv...)
	out = append(out, ct...)
	return out
}

// DecryptPayload decrypts a payload, returning the plaintext with the
// confidentiality padding removed. It returns an error on malformed padding.
func DecryptPayload(cryptAlg uint8, k2, input []byte) ([]byte, error) {
	if cryptAlg == ipmicore.CryptNone {
		return append([]byte(nil), input...), nil
	}
	if len(input) < AESBlockSize*2 {
		return nil, errors.New("encrypted payload too short")
	}
	iv := input[:AESBlockSize]
	ct := input[AESBlockSize:]
	dec := aesCBCDecrypt(k2[:16], iv, ct)
	confPad := int(dec[len(dec)-1])
	if confPad+1 > len(dec) {
		return nil, errors.New("malformed confidentiality padding")
	}
	payloadSize := len(dec) - confPad - 1
	for i := range confPad {
		if dec[payloadSize+i] != byte(i+1) {
			return nil, errors.New("malformed payload padding")
		}
	}
	return append([]byte(nil), dec[:payloadSize]...), nil
}

// BuildV2x builds a complete IPMI v2 / RMCP+ message. It updates
// session.OutSeq.
func BuildV2x(session *Session, payloadType uint8, payload []byte) []byte {
	active := session.V2.State == SessionStateActive

	m := make([]byte, 0, len(payload)+64)
	// RMCP header
	m = append(m, 0x06, 0x00, 0xff, 0x07)

	// IPMI session header
	m = append(m, ipmicore.SessionAuthtypeRmcpPlus)
	pt := payloadType
	if active {
		if session.V2.CryptAlg != ipmicore.CryptNone {
			pt |= 0x80
		}
		if session.V2.IntegrityAlg != ipmicore.IntegrityNone {
			pt |= 0x40
		}
	}
	m = append(m, pt)
	if active {
		m = append(m, le32(session.V2.BMCID)...)
		m = append(m, le32(session.OutSeq)...)
	} else {
		m = append(m, 0, 0, 0, 0)
		m = append(m, 0, 0, 0, 0)
	}
	m = append(m, 0, 0) // payload size placeholder

	// Payload (possibly encrypted)
	var body []byte
	if active {
		body = EncryptPayload(session.V2.CryptAlg, session.V2.K2, payload)
	} else {
		body = append([]byte(nil), payload...)
	}
	plen := uint16(len(body))
	m[offPayloadSize] = byte(plen)
	m[offPayloadSize+1] = byte(plen >> 8)
	m = append(m, body...)

	// Session trailer: integrity pad, pad length, next header, authcode.
	if active && session.V2.IntegrityAlg != ipmicore.IntegrityNone {
		lengthBeforeAuthcode := offPayload + len(body) + 2
		pad := 0
		if lengthBeforeAuthcode%4 != 0 {
			pad = 4 - (lengthBeforeAuthcode % 4)
		}
		for i := 0; i < pad; i++ {
			m = append(m, 0xff)
		}
		m = append(m, byte(pad))
		m = append(m, 0x07)
		// The integrity HMAC covers the session header from the auth-type
		// byte through the end of the trailer, i.e. everything built so far
		// after `offAuthtype`.
		authLen := session.AuthcodeLen()
		mac := hmacIntegrity(session.V2.IntegrityAlg, session.V2.K1, m[offAuthtype:])
		m = append(m, mac[:authLen]...)
	}

	session.NextOutSeq()
	return m
}

// BuildV15 builds a pre-session IPMI v1.5 message (used for the Get Channel
// Authentication Capabilities command).
func BuildV15(session *Session, req *ipmicore.Request) []byte {
	m := []byte{}
	m = append(m, 0x06, 0x00, 0xff, 0x07) // RMCP
	m = append(m, ipmicore.SessionAuthtypeNone)
	m = append(m, le32(session.OutSeq)...)
	m = append(m, 0, 0, 0, 0)            // session id
	m = append(m, byte(len(req.Data)+7)) // message length

	cs := len(m)
	m = append(m, ipmicore.BmcSlaveAddr)
	m = append(m, req.Netfn<<2)
	c := ipmicore.IpmiCsum(m[cs:])
	m = append(m, c)
	cs = len(m)
	m = append(m, ipmicore.RemoteSWID)
	m = append(m, 0) // rq seq / lun
	m = append(m, req.Cmd)
	m = append(m, req.Data...)
	c = ipmicore.IpmiCsum(m[cs:])
	m = append(m, c)

	// NB: ipmitool does not advance the session sequence number for v1.5
	// pre-session messages, so neither do we.
	return m
}

// ParseSessionHeader parses the RMCP+ session header from a received datagram.
func ParseSessionHeader(data []byte) (SessionHeader, bool) {
	if len(data) < offAuthtype+1 {
		return SessionHeader{}, false
	}
	authtype := data[offAuthtype]
	if authtype == ipmicore.SessionAuthtypeRmcpPlus {
		if len(data) < offPayload {
			return SessionHeader{}, false
		}
		return SessionHeader{
			Authtype:      authtype,
			PayloadType:   data[offPayloadType] & 0x3f,
			Encrypted:     data[offPayloadType]&0x80 != 0,
			Authenticated: data[offPayloadType]&0x40 != 0,
			SessionID:     le32dec(data[offSessionID : offSessionID+4]),
			Msglen:        uint16(data[offPayloadSize]) | uint16(data[offPayloadSize+1])<<8,
			Offset:        offPayload,
		}, true
	}
	// IPMI v1.5 style session header (13 bytes after RMCP, 1-byte length).
	if len(data) < 15 {
		return SessionHeader{}, false
	}
	return SessionHeader{
		Authtype:      authtype,
		PayloadType:   ipmicore.PayloadTypeIpmi,
		Encrypted:     false,
		Authenticated: false,
		SessionID:     le32dec(data[6:10]),
		Msglen:        uint16(data[13]),
		Offset:        14,
	}, true
}

// ValidAuthcode validates the authcode of a received session message.
func ValidAuthcode(data []byte, header *SessionHeader, session *Session) bool {
	if session.V2.State != SessionStateActive ||
		header.Authtype != ipmicore.SessionAuthtypeRmcpPlus ||
		!header.Authenticated ||
		session.V2.IntegrityAlg == ipmicore.IntegrityNone {
		return true
	}
	authLen := session.AuthcodeLen()
	if len(data) < offAuthtype+authLen {
		return false
	}
	bmc := data[len(data)-authLen:]
	mac := hmacIntegrity(session.V2.IntegrityAlg, session.V2.K1,
		data[offAuthtype:len(data)-authLen])
	return len(mac) >= authLen && byteSliceEqual(bmc, mac[:authLen])
}

// ParseIpmiResponse parses an IPMI response payload (already decrypted) into a
// Response.
func ParseIpmiResponse(payload []byte) (*ipmicore.Response, bool) {
	if len(payload) < 7 {
		return nil, false
	}
	netfn := payload[1] >> 2
	rqSeq := payload[4] >> 2
	cmd := payload[5]
	ccode := payload[6]
	dataEnd := len(payload) - 1 // trailing checksum
	var data []byte
	if dataEnd > 7 {
		data = append([]byte(nil), payload[7:dataEnd]...)
	}
	return &ipmicore.Response{
		Ccode: ccode,
		Data:  data,
		Netfn: netfn,
		Cmd:   cmd,
		RqSeq: rqSeq,
	}, true
}

// OpenSessionRequest builds the 32-byte Open Session Request payload.
func OpenSessionRequest(privlvl uint8, consoleID uint32, suite ipmicore.CipherSuite) []byte {
	m := make([]byte, 32)
	m[0] = 0 // message tag
	if privlvl != ipmicore.SessionPrivAdmin {
		m[1] = privlvl
	} else {
		m[1] = 0
	}
	copy(m[4:8], le32(consoleID))
	// Authentication payload
	m[8] = 0
	m[11] = 8
	m[12] = suite.AuthAlg
	// Integrity payload
	m[16] = 1
	m[19] = 8
	m[20] = suite.IntegrityAlg
	// Confidentiality payload
	m[24] = 2
	m[27] = 8
	m[28] = suite.CryptAlg
	return m
}

// Rakp1Message builds the 44-byte RAKP 1 message (truncated for short
// usernames).
func Rakp1Message(bmcID uint32, consoleRand *[16]byte, role uint8, username []byte) []byte {
	m := make([]byte, 44)
	m[0] = 0 // message tag
	copy(m[4:8], le32(bmcID))
	copy(m[8:24], consoleRand[:])
	m[24] = role
	n := min(len(username), 16)
	m[27] = byte(n)
	copy(m[28:28+n], username[:n])
	length := 44 - (16 - n)
	m = m[:length]
	return m
}

// Rakp3Message builds the RAKP 3 message (8-byte header plus optional auth
// code).
func Rakp3Message(bmcID uint32, rakp2Rc uint8, authcode []byte) []byte {
	m := make([]byte, 0, 8+len(authcode))
	m = append(m, 0) // message tag
	m = append(m, rakp2Rc)
	m = append(m, 0, 0)
	m = append(m, le32(bmcID)...)
	m = append(m, authcode...)
	return m
}

// le32dec decodes a little-endian uint32.
func le32dec(b []byte) uint32 {
	return uint32(b[0]) | uint32(b[1])<<8 | uint32(b[2])<<16 | uint32(b[3])<<24
}
