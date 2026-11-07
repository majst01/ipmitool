package lanplus

import (
	"bytes"
	"testing"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
)

func TestRequestPayloadChecksumsAreValid(t *testing.T) {
	req := ipmicore.NewRequest(0x06, 0x01).WithData([]byte{0x01, 0x02})
	payload := RequestPayload(req, 0x20, 3)
	// First checksum covers rsAddr + netfn byte (and the checksum itself).
	if ipmicore.IpmiCsum(payload[:3]) != 0 {
		t.Fatal("first checksum invalid")
	}
	// Second checksum covers rqAddr..data plus the trailing checksum.
	if ipmicore.IpmiCsum(payload[3:]) != 0 {
		t.Fatal("second checksum invalid")
	}
	// netfn/cmd are placed correctly.
	if payload[1] != 0x06<<2 {
		t.Fatalf("netfn byte = %#x", payload[1])
	}
	if payload[5] != 0x01 {
		t.Fatalf("cmd = %#x", payload[5])
	}
	if payload[3] != ipmicore.RemoteSWID {
		t.Fatalf("rqAddr = %#x", payload[3])
	}
	if payload[4] != 3<<2 {
		t.Fatalf("seq = %#x", payload[4])
	}
}

func TestParseIpmiResponseExtractsFields(t *testing.T) {
	// rsAddr, netfn, csum, rqAddr, seq, cmd, ccode, data, csum
	payload := []byte{
		0x20,
		0x06 << 2,
		0x00,
		0x81,
		0x02 << 2,
		0x01,
		0x00,
		0xaa,
		0xbb,
		0x00,
	}
	rsp, ok := ParseIpmiResponse(payload)
	if !ok {
		t.Fatal("parse failed")
	}
	if rsp.Netfn != 0x06 || rsp.RqSeq != 2 || rsp.Cmd != 0x01 || rsp.Ccode != 0x00 {
		t.Fatalf("fields: %+v", rsp)
	}
	if !bytes.Equal(rsp.Data, []byte{0xaa, 0xbb}) {
		t.Fatalf("data = %x", rsp.Data)
	}
}

func TestConfidentialityRoundTrip(t *testing.T) {
	k2 := bytes.Repeat([]byte{0x33}, 20)
	plain := []byte("hello ipmi world")
	enc := EncryptPayload(ipmicore.CryptAesCbc128, k2, plain)
	dec, err := DecryptPayload(ipmicore.CryptAesCbc128, k2, enc)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(dec, plain) {
		t.Fatalf("decrypted = %q", dec)
	}
}

func TestConfidentialityNoneIsPassthrough(t *testing.T) {
	data := []byte("unencrypted")
	enc := EncryptPayload(ipmicore.CryptNone, nil, data)
	if !bytes.Equal(enc, data) {
		t.Fatalf("enc = %q", enc)
	}
}

func TestOpenSessionRequestLayout(t *testing.T) {
	suite, ok := ipmicore.CipherSuiteFromID(3)
	if !ok {
		t.Fatal("suite 3 not found")
	}
	m := OpenSessionRequest(ipmicore.SessionPrivAdmin, 0xA0A2A3A4, suite)
	if len(m) != 32 {
		t.Fatalf("len = %d", len(m))
	}
	if !bytes.Equal(m[4:8], le32(0xA0A2A3A4)) {
		t.Fatalf("session id = %x", m[4:8])
	}
	if m[12] != suite.AuthAlg || m[20] != suite.IntegrityAlg || m[28] != suite.CryptAlg {
		t.Fatalf("algs = %#x %#x %#x", m[12], m[20], m[28])
	}
}

func TestRakp1MessageTruncatesUsername(t *testing.T) {
	var rand [16]byte
	for i := range rand {
		rand[i] = 0x11
	}
	msg := Rakp1Message(0x01020304, &rand, 0x14, []byte("admin"))
	if len(msg) != 28+5 {
		t.Fatalf("len = %d", len(msg))
	}
	if msg[27] != 5 {
		t.Fatalf("ulen = %d", msg[27])
	}
	if !bytes.Equal(msg[28:33], []byte("admin")) {
		t.Fatalf("username = %q", msg[28:33])
	}
}

func TestBuildV2xAdvancesSequenceAndMarksPayload(t *testing.T) {
	session := NewSession(1, [ipmicore.IPMIAuthcodeBufferSize]byte{})
	session.V2.State = SessionStatePresession
	m := BuildV2x(session, ipmicore.PayloadTypeRmcpOpenRequest, []byte{1, 2, 3, 4})
	if !bytes.Equal(m[:4], []byte{0x06, 0x00, 0xff, 0x07}) {
		t.Fatalf("rmcp = %x", m[:4])
	}
	if m[offAuthtype] != ipmicore.SessionAuthtypeRmcpPlus {
		t.Fatalf("authtype = %#x", m[offAuthtype])
	}
	if m[offPayloadType]&0x3f != ipmicore.PayloadTypeRmcpOpenRequest {
		t.Fatalf("payload type = %#x", m[offPayloadType]&0x3f)
	}
	plen := uint16(m[offPayloadSize]) | uint16(m[offPayloadSize+1])<<8
	if int(plen) != 4 {
		t.Fatalf("plen = %d", plen)
	}
}

func TestBuildV2xActiveIntegrityCoversHeaderAndTrailer(t *testing.T) {
	// Regression: the integrity HMAC must cover everything from the auth-type
	// byte to the end of the trailer, and the slice must not run past the
	// buffer.
	session := NewSession(1, [ipmicore.IPMIAuthcodeBufferSize]byte{})
	session.V2.State = SessionStateActive
	session.V2.IntegrityAlg = ipmicore.IntegrityHmacSha196
	session.V2.CryptAlg = ipmicore.CryptNone
	session.V2.K1 = bytes.Repeat([]byte{0x11}, 20)

	m := BuildV2x(session, ipmicore.PayloadTypeIpmi, []byte{1, 2, 3, 4, 5, 6, 7, 8})

	authLen := 12 // HMAC-SHA1-96
	trailerEnd := len(m) - authLen
	// Next-header byte is the last byte before the authcode.
	if m[trailerEnd-1] != 0x07 {
		t.Fatalf("next header = %#x", m[trailerEnd-1])
	}
	expected := hmacIntegrity(ipmicore.IntegrityHmacSha196, session.V2.K1, m[offAuthtype:trailerEnd])
	if !bytes.Equal(m[trailerEnd:], expected[:authLen]) {
		t.Fatal("authcode mismatch")
	}
}
