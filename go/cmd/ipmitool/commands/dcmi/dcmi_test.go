package dcmi

import (
	"strings"
	"testing"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
	"codeberg.org/ipmitool/ipmitool/internal/mock"
)

// ---------------------------------------------------------------------------
// Helpers.
// ---------------------------------------------------------------------------

func TestStr2Val2IsCaseInsensitiveAndFallsBack(t *testing.T) {
	if v := str2val2("reading", dcmiPwrmgmtVals); v != 0x00 {
		t.Fatalf("str2val2(reading) = %#x, want 0", v)
	}
	if v := str2val2("READING", dcmiPwrmgmtVals); v != 0x00 {
		t.Fatalf("str2val2(READING) = %#x, want 0", v)
	}
	if v := str2val2("bogus", dcmiPwrmgmtVals); v != 0xFF {
		t.Fatalf("str2val2(bogus) = %#x, want 0xFF", v)
	}
	if v := str2val2("15_min", dcmiSamplingVals); v != 0x4f {
		t.Fatalf("str2val2(15_min) = %#x, want 0x4f", v)
	}
	if v := str2val2("bogus", dcmiSamplingVals); v != 0x00 {
		t.Fatalf("str2val2(bogus) = %#x, want 0", v)
	}
}

func TestVal2Str2RendersUnknown(t *testing.T) {
	if got := val2str2(0x05, dcmiSamplingVals); got != "5_sec" {
		t.Fatalf("val2str2(5) = %q", got)
	}
	if got := val2str2(0x1234, dcmiSamplingVals); got != "Unknown (0x1234)" {
		t.Fatalf("val2str2(0x1234) = %q", got)
	}
}

func TestDcmiCcodeStr(t *testing.T) {
	if got := dcmiCcodeStr(0x80); got != "Parameter not supported" {
		t.Fatalf("dcmiCcodeStr(0x80) = %q", got)
	}
	if got := dcmiCcodeStr(0x8A); got != "Power limit already active" {
		t.Fatalf("dcmiCcodeStr(0x8a) = %q", got)
	}
	if got := dcmiCcodeStr(0x88); got != "Unknown (0x88)" {
		t.Fatalf("dcmiCcodeStr(0x88) = %q", got)
	}
}

func TestTimestampNumeric(t *testing.T) {
	if got := timestampNumeric(3661); got != "S+ 01:01:01" {
		t.Fatalf("timestampNumeric(3661) = %q", got)
	}
	if got := timestampNumeric(0xFFFFFFFF); got != "Unspecified" {
		t.Fatalf("timestampNumeric(unset) = %q", got)
	}
}

// ---------------------------------------------------------------------------
// Mock-interface tests.
// ---------------------------------------------------------------------------

// powerReadingData builds an 18-byte DCMI Get Power Reading response.
func powerReadingData(curr, min, max, avg uint16, ts, sample uint32, state uint8) []byte {
	d := make([]byte, 18)
	d[0] = 0xDC
	d[1] = byte(curr)
	d[2] = byte(curr >> 8)
	d[3] = byte(min)
	d[4] = byte(min >> 8)
	d[5] = byte(max)
	d[6] = byte(max >> 8)
	d[7] = byte(avg)
	d[8] = byte(avg >> 8)
	d[9] = byte(ts)
	d[10] = byte(ts >> 8)
	d[11] = byte(ts >> 16)
	d[12] = byte(ts >> 24)
	d[13] = byte(sample)
	d[14] = byte(sample >> 8)
	d[15] = byte(sample >> 16)
	d[16] = byte(sample >> 24)
	d[17] = state
	return d
}

func TestDcmiPowerReading(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnDcgrp, ipmiDCMIGetRed, mock.OKData(
		powerReadingData(100, 90, 110, 95, 3661, 3000, 0x40)))
	m := r.InterfaceWith()

	var code int
	stdout, _ := mock.Capture(func() {
		code = Main(m, []string{"power", "reading"})
	})
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if len(m.Requests) != 1 {
		t.Fatalf("request count = %d, want 1", len(m.Requests))
	}
	req := m.Requests[0]
	if req.Netfn != ipmicore.NetfnDcgrp || req.Cmd != ipmiDCMIGetRed {
		t.Fatalf("unexpected request: %+v", req)
	}
	want := []byte{0xDC, 0x01, 0x00, 0x00}
	if len(req.Data) != len(want) || req.Data[0] != want[0] || req.Data[1] != want[1] ||
		req.Data[2] != want[2] || req.Data[3] != want[3] {
		t.Fatalf("request data = %v, want %v", req.Data, want)
	}
	for _, w := range []string{
		"    Instantaneous power reading:                   100 Watts",
		"    Minimum during sampling period:                 90 Watts",
		"    Maximum during sampling period:                110 Watts",
		"    Average power reading over sample period:       95 Watts",
		"    IPMI timestamp:                           S+ 01:01:01",
		"    Sampling period:                          00000003 Seconds.",
		"    Power reading state is:                   activated",
	} {
		if !strings.Contains(stdout, w) {
			t.Errorf("stdout missing %q\n%s", w, stdout)
		}
	}
}

func TestDcmiPowerReadingEnhanced(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnDcgrp, ipmiDCMIGetRed, mock.OKData(
		powerReadingData(0, 0, 0, 0, 0, 0x41, 0x00)))
	m := r.InterfaceWith()

	var code int
	stdout, _ := mock.Capture(func() {
		code = Main(m, []string{"power", "reading", "1_min"})
	})
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	want := []byte{0xDC, 0x02, 0x41, 0x00}
	if len(m.Requests[0].Data) != len(want) || string(m.Requests[0].Data) != string(want) {
		t.Fatalf("request data = %v, want %v", m.Requests[0].Data, want)
	}
	if !strings.Contains(stdout, "1_min ") {
		t.Fatalf("stdout = %q", stdout)
	}
	if !strings.Contains(stdout, "deactivated") {
		t.Fatalf("stdout = %q", stdout)
	}
}

func TestDcmiPowerReadingInvalidSampleTime(t *testing.T) {
	m := mock.New(nil)
	var code int
	stdout, stderr := mock.Capture(func() {
		code = Main(m, []string{"power", "reading", "bogus"})
	})
	if code != -1 {
		t.Fatalf("exit code = %d, want -1", code)
	}
	if !strings.Contains(stderr, "Invalid sample time. Valid times are: ") {
		t.Fatalf("stderr = %q", stderr)
	}
	if !strings.Contains(stdout, " | ") {
		t.Fatalf("stdout = %q", stdout)
	}
}

func TestDcmiPowerReadingDcmiCcode(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnDcgrp, ipmiDCMIGetRed, mock.CCode(0x84))
	m := r.InterfaceWith()
	var code int
	_, stderr := mock.Capture(func() {
		code = Main(m, []string{"power", "reading"})
	})
	if code != -1 {
		t.Fatalf("exit code = %d, want -1", code)
	}
	if !strings.Contains(stderr, "DCMI request failed because: Power/Thermal limit out of range (84)") {
		t.Fatalf("stderr = %q", stderr)
	}
}

func TestDcmiPowerReadingWrongGroupExtension(t *testing.T) {
	r := mock.NewRouter()
	d := powerReadingData(0, 0, 0, 0, 0, 0, 0)
	d[0] = 0x00
	r.Static(ipmicore.NetfnDcgrp, ipmiDCMIGetRed, mock.OKData(d))
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() {
		code = Main(m, []string{"power", "reading"})
	})
	if code != -1 {
		t.Fatalf("exit code = %d, want -1", code)
	}
	if !strings.Contains(stdout, "A valid DCMI command was not returned! (0)") {
		t.Fatalf("stdout = %q", stdout)
	}
}

func TestDcmiHelpListsCommands(t *testing.T) {
	m := mock.New(nil)
	var code int
	_, stderr := mock.Capture(func() {
		code = Main(m, nil)
	})
	if code != -1 {
		t.Fatalf("exit code = %d, want -1", code)
	}
	if !strings.Contains(stderr, "Data Center Management Interface commands") {
		t.Fatalf("stderr = %q", stderr)
	}
	if !strings.Contains(stderr, "    power    Platform power limit command options") {
		t.Fatalf("stderr = %q", stderr)
	}
}
