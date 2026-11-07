package fru

import "testing"

func TestAsciiFruFieldIsDecoded(t *testing.T) {
	// Type 11b (8-bit ASCII), length 3, followed by "ABC".
	data := []byte{0xc3, 'A', 'B', 'C'}
	off := 0
	s, ok := getFruAreaStr(data, &off)
	if !ok || s != "ABC" {
		t.Fatalf("getFruAreaStr = (%q, %v), want (%q, true)", s, ok, "ABC")
	}
	if off != 4 {
		t.Fatalf("off = %d, want 4", off)
	}
}

func TestBcdPlusFruFieldIsDecoded(t *testing.T) {
	// Type 01b (BCD plus), length 2 -> four nibbles "1234".
	data := []byte{0x42, 0x12, 0x34}
	off := 0
	s, ok := getFruAreaStr(data, &off)
	if !ok || s != "1234" {
		t.Fatalf("getFruAreaStr = (%q, %v), want (%q, true)", s, ok, "1234")
	}
	if off != 3 {
		t.Fatalf("off = %d, want 3", off)
	}
}

func TestSixBitAsciiFruFieldIsDecoded(t *testing.T) {
	// Type 10b (6-bit ASCII), length 3. "ABC" encoded as 6-bit values
	// (0x21, 0x22, 0x23) packed little-endian into 3 bytes.
	packed := uint32(0x21) | (0x22 << 6) | (0x23 << 12)
	data := []byte{0x83, byte(packed), byte(packed >> 8), byte(packed >> 16)}
	off := 0
	s, ok := getFruAreaStr(data, &off)
	if !ok {
		t.Fatalf("getFruAreaStr returned false, want true")
	}
	if s != "ABC " {
		t.Fatalf("getFruAreaStr = %q, want %q", s, "ABC ")
	}
	if off != 4 {
		t.Fatalf("off = %d, want 4", off)
	}
}

func TestZeroLengthFieldReturnsNone(t *testing.T) {
	data := []byte{0x00}
	off := 0
	if _, ok := getFruAreaStr(data, &off); ok {
		t.Fatalf("getFruAreaStr returned true, want false")
	}
	if off != 1 {
		t.Fatalf("off = %d, want 1", off)
	}
}

func TestChecksumDetectsValidAndInvalidAreas(t *testing.T) {
	valid := []byte{0x01, 0x02, 0x03, 0xfa}
	if !checksumIsValid(valid, len(valid)) {
		t.Fatalf("checksumIsValid(valid) = false, want true")
	}
	invalid := []byte{0x01, 0x02, 0x03, 0x00}
	if checksumIsValid(invalid, len(invalid)) {
		t.Fatalf("checksumIsValid(invalid) = true, want false")
	}
}

func TestZeroFruTimestampIsUnspecified(t *testing.T) {
	if got := formatFruTimestamp([]byte{0, 0, 0}, 0); got != "Unspecified" {
		t.Fatalf("formatFruTimestamp = %q, want %q", got, "Unspecified")
	}
}

func TestVoltageDescriptionsMatchSpec(t *testing.T) {
	if got := voltageDesc(0); got != "12 V" {
		t.Fatalf("voltageDesc(0) = %q, want %q", got, "12 V")
	}
	if got := voltageDesc(3); got != "3.3 V" {
		t.Fatalf("voltageDesc(3) = %q, want %q", got, "3.3 V")
	}
	if got := voltageDesc(9); got != "" {
		t.Fatalf("voltageDesc(9) = %q, want %q", got, "")
	}
}
