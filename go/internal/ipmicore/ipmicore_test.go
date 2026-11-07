package ipmicore

import "testing"

func TestChecksumMakesSumZero(t *testing.T) {
	data := []byte{0x20, 0x18, 0x04, 0x81}
	c := IpmiCsum(data)
	var sum uint8
	for _, b := range data {
		sum += b
	}
	if sum+c != 0 {
		t.Fatalf("checksum does not zero the sum: %d", sum+c)
	}
}

func TestEmptyChecksumIsZero(t *testing.T) {
	if IpmiCsum(nil) != 0 {
		t.Fatalf("empty checksum should be zero")
	}
}

func TestPowerControlStringsMatchC(t *testing.T) {
	cases := map[uint8]string{
		0x0: "Down/Off",
		0x1: "Up/On",
		0x2: "Cycle",
		0x3: "Reset",
		0x4: "Diag",
		0x5: "Soft",
	}
	for v, want := range cases {
		if got := PowerControlStr(v); got != want {
			t.Errorf("PowerControlStr(%#x) = %q, want %q", v, got, want)
		}
	}
}

func TestRestartCauseStringsMatchC(t *testing.T) {
	cases := map[uint8]string{
		0x1: "chassis power control command",
		0x3: "power-up via pushbutton",
		0x4: "watchdog expired",
		0x8: "reset via PEF",
		0x9: "power-cycle via PEF",
		0xa: "soft reset",
		0xb: "power-up via RTC wakeup",
	}
	for v, want := range cases {
		if got := ChassisRestartCauseStr(v); got != want {
			t.Errorf("ChassisRestartCauseStr(%#x) = %q, want %q", v, got, want)
		}
	}
}

func TestCipherSuite3UsesSha1AndAes(t *testing.T) {
	s, ok := CipherSuiteFromID(3)
	if !ok {
		t.Fatal("suite 3 should exist")
	}
	if s.AuthAlg != AuthRakpHmacSha1 || s.IntegrityAlg != IntegrityHmacSha196 || s.CryptAlg != CryptAesCbc128 {
		t.Fatalf("unexpected suite 3 algorithms: %+v", s)
	}
	if s.AuthcodeLen() != 12 {
		t.Fatalf("suite 3 authcode len = %d, want 12", s.AuthcodeLen())
	}
}

func TestCipherSuite17IsSha256(t *testing.T) {
	s, ok := CipherSuiteFromID(17)
	if !ok {
		t.Fatal("suite 17 should exist")
	}
	if s.AuthAlg != AuthRakpHmacSha256 || s.IntegrityAlg != IntegrityHmacSha256128 {
		t.Fatalf("unexpected suite 17 algorithms: %+v", s)
	}
	if s.AuthcodeLen() != 16 {
		t.Fatalf("suite 17 authcode len = %d, want 16", s.AuthcodeLen())
	}
}

func TestUnknownCipherSuiteIsNone(t *testing.T) {
	if _, ok := CipherSuiteFromID(0xff); ok {
		t.Fatal("0xff should be unknown")
	}
	// Mirrors the current Rust table, which omits suite 18 (see spec-drift.md).
	if _, ok := CipherSuiteFromID(18); ok {
		t.Fatal("18 should be unknown in the ported table")
	}
}

func TestPasswordIsNulPadded(t *testing.T) {
	p := DefaultSessionParams()
	p.SetPassword("abc")
	pb := p.PasswordBytes()
	if string(pb[:3]) != "abc" {
		t.Fatalf("password prefix = %q", pb[:3])
	}
	for i := 3; i < len(pb); i++ {
		if pb[i] != 0 {
			t.Fatalf("password byte %d = %d, want 0", i, pb[i])
		}
	}
}

func TestSolPacketRoundTrip(t *testing.T) {
	pkt := &SolPacket{Sequence: 3, Acked: 2, Accepted: 1, Flags: SolFlagGenerateBreak, Data: []byte("hi")}
	bytes := pkt.ToBytes()
	want := []byte{3, 2, 1, 0x10, 'h', 'i'}
	if len(bytes) != len(want) {
		t.Fatalf("len = %d, want %d", len(bytes), len(want))
	}
	for i := range want {
		if bytes[i] != want[i] {
			t.Fatalf("byte %d = %#x, want %#x", i, bytes[i], want[i])
		}
	}
	back, ok := SolPacketFromBytes(bytes)
	if !ok {
		t.Fatal("from_bytes failed")
	}
	if back.Sequence != 3 || back.Acked != 2 || back.Accepted != 1 || string(back.Data) != "hi" {
		t.Fatalf("round trip mismatch: %+v", back)
	}
}

func TestSolSequenceNumbersAreMasked(t *testing.T) {
	pkt := &SolPacket{Sequence: 0xff, Acked: 0xff}
	bytes := pkt.ToBytes()
	if bytes[0] != 0x0f || bytes[1] != 0x0f {
		t.Fatalf("sequence not masked: %#x %#x", bytes[0], bytes[1])
	}
}

func TestSolRecvFlags(t *testing.T) {
	pkt, ok := SolPacketFromBytes([]byte{1, 0, 4, 0x74, 0xaa})
	if !ok {
		t.Fatal("from_bytes failed")
	}
	if !pkt.IsNack() || !pkt.TransferUnavailable() || !pkt.SolInactive() || !pkt.BreakDetected() {
		t.Fatalf("missing expected flags: %+v", pkt)
	}
	if pkt.TransmitOverrun() {
		t.Fatal("transmit overrun should be clear")
	}
	if len(pkt.Data) != 1 || pkt.Data[0] != 0xaa {
		t.Fatalf("data = %v", pkt.Data)
	}
}

func TestSolAckMatchesSequence(t *testing.T) {
	ack := SolAck(5, 10)
	if !ack.Acks(5) || ack.Acks(4) || !ack.Acks(0x15) {
		t.Fatal("ack matching wrong")
	}
}

func TestActivateResponseParsesSizesAndPort(t *testing.T) {
	// NOTE: encodes the ported (off-by-two) offsets; see rust/spec-drift.md.
	data := make([]byte, 12)
	data[6] = 0x56
	data[7] = 0x02
	data[8] = 0x10
	data[9] = 0x00
	data[10] = 0x6f
	data[11] = 0x02
	var rsp SolActivateResponse
	if err := rsp.Parse(data); err != nil {
		t.Fatal(err)
	}
	if rsp.InboundPayloadSize != 0x0256 || rsp.OutboundPayloadSize != 0x0010 || rsp.Port != 0x026f {
		t.Fatalf("unexpected parse: %+v", rsp)
	}
}

func TestActivateResponseRejectsWrongLength(t *testing.T) {
	var rsp SolActivateResponse
	if err := rsp.Parse(make([]byte, 4)); err == nil {
		t.Fatal("short data should be rejected")
	}
}

func TestParsesRegistryEntries(t *testing.T) {
	text := "PRIVATE ENTERPRISE NUMBERS\n\n343\n  Intel Corporation\n    Adam\n1\n  Foo Bar\n    baz\n"
	names := map[uint32]string{}
	parseRegistry(text, names)
	if names[343] != "Intel Corporation" || names[1] != "Foo Bar" || len(names) != 2 {
		t.Fatalf("unexpected registry: %v", names)
	}
}

func TestIgnoresMalformedOrgLines(t *testing.T) {
	text := "10\n   Three spaces\n20\nnot indented\n30\n  Good Name\n"
	names := map[uint32]string{}
	parseRegistry(text, names)
	if _, ok := names[10]; ok {
		t.Fatal("three-space org should be rejected")
	}
	if _, ok := names[20]; ok {
		t.Fatal("unindented org should be rejected")
	}
	if names[30] != "Good Name" {
		t.Fatalf("good name missing: %v", names)
	}
}

func TestBuiltinManufacturerNames(t *testing.T) {
	reg := NewEmptyOemRegistry()
	if got := reg.ManufacturerName(0); got != "Unknown" {
		t.Errorf("0 -> %q", got)
	}
	if got := reg.ManufacturerName(oemReserved); got != "Unspecified" {
		t.Errorf("reserved -> %q", got)
	}
	if got := reg.ManufacturerName(oemDebug); got != oemDebugName {
		t.Errorf("debug -> %q", got)
	}
	if got := reg.ManufacturerName(999); got != "Unknown (0x3E7)" {
		t.Errorf("999 -> %q", got)
	}
}

func TestProductNameLookup(t *testing.T) {
	reg := NewEmptyOemRegistry()
	if got := reg.ProductName(343, 0x000C); got != "TSRLT2" {
		t.Errorf("Intel 0x000C -> %q", got)
	}
	if got := reg.ProductName(343, 0xFFFF); got != "Unknown (0xFFFF)" {
		t.Errorf("Intel 0xFFFF -> %q", got)
	}
}
