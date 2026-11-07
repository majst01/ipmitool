package mc

import (
	"strings"
	"testing"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
	"codeberg.org/ipmitool/ipmitool/internal/mock"
)

// deviceIDData builds a 15-byte Get Device ID response.
func deviceIDData() []byte {
	d := make([]byte, 15)
	d[0] = 0x20       // device id
	d[1] = 0x81       // device revision, bit7 -> provides SDRs
	d[2] = 0x01       // fw rev 1
	d[3] = 0x23       // fw rev 2
	d[4] = 0x02       // ipmi version 2.0
	d[5] = 0x05       // additional support: sensor + SEL
	d[6] = 343 & 0xff // manufacturer (Intel) low
	d[7] = (343 >> 8) & 0xff
	d[8] = (343 >> 16) & 0xff
	d[9] = 0x0c  // product id low
	d[10] = 0x00 // product id high
	return d
}

func TestMcInfo(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, ipmicore.IpmiGetDeviceID, mock.OKData(deviceIDData()))
	m := r.InterfaceWith()

	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"info"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	for _, want := range []string{
		"Device ID                 : 32",
		"Firmware Revision         : 1.23",
		"IPMI Version              : 2.0",
		"Manufacturer ID           : 343",
		"Product ID                : 12 (0x000c)",
		"Product Name              : TSRLT2",
		"Device Available          : yes",
		"Provides Device SDRs      : yes",
		"    Sensor Device",
		"    SEL Device",
		"Aux Firmware Rev Info     : ",
	} {
		if !strings.Contains(stdout, want) {
			t.Errorf("stdout missing %q\n%s", want, stdout)
		}
	}
}

func TestMcInfoShortResponse(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, ipmicore.IpmiGetDeviceID, mock.OK(0x01))
	m := r.InterfaceWith()
	var code int
	_, stderr := mock.Capture(func() { code = Main(m, []string{"info"}) })
	if code != -1 || !strings.Contains(stderr, "short response") {
		t.Fatalf("code=%d stderr=%q", code, stderr)
	}
}

func TestMcResetCold(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, 0x02, mock.OK())
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"reset", "cold"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if m.Requests[0].Cmd != 0x02 {
		t.Fatalf("cmd = %#x, want 0x02", m.Requests[0].Cmd)
	}
	if !strings.Contains(stdout, "Sent cold reset command to MC") {
		t.Fatalf("stdout = %q", stdout)
	}
}

func TestMcResetColdToleratesNoResponse(t *testing.T) {
	// A cold reset commonly gets no answer; that must not be an error.
	m := mock.New(func(*ipmicore.Request) (*ipmicore.Response, error) {
		return nil, ipmicore.ErrTimeout
	})
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"reset", "cold"}) })
	if code != 0 || !strings.Contains(stdout, "Sent cold reset command to MC") {
		t.Fatalf("code=%d stdout=%q", code, stdout)
	}
}

func TestMcGetEnables(t *testing.T) {
	// event_msg (bit2) and system_event_log (bit3) enabled.
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, ipmicore.IpmiGetGlobalEnables, mock.OK(0x0c))
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"getenables"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if !strings.Contains(stdout, "Event Message Buffer                     : enabled") {
		t.Fatalf("stdout = %q", stdout)
	}
	if !strings.Contains(stdout, "System Event Logging                     : enabled") {
		t.Fatalf("stdout = %q", stdout)
	}
	if !strings.Contains(stdout, "Receive Message Queue Interrupt          : disabled") {
		t.Fatalf("stdout = %q", stdout)
	}
}

func TestMcSetEnables(t *testing.T) {
	r := mock.NewRouter()
	// First get: 0; then set: 0x08; then verify get: 0x08.
	call := 0
	r.On(ipmicore.NetfnApp, ipmicore.IpmiGetGlobalEnables, func(*ipmicore.Request) (*ipmicore.Response, error) {
		call++
		if call == 1 {
			return mock.OK(0x00), nil
		}
		return mock.OK(0x08), nil
	})
	r.On(ipmicore.NetfnApp, ipmicore.IpmiSetGlobalEnables, func(req *ipmicore.Request) (*ipmicore.Response, error) {
		if len(req.Data) != 1 || req.Data[0] != 0x08 {
			t.Errorf("set enables data = %v, want [8]", req.Data)
		}
		return mock.OK(), nil
	})
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"setenables", "system_event_log=on"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if !strings.Contains(stdout, "Enabling System Event Logging") {
		t.Fatalf("stdout = %q", stdout)
	}
	if !strings.Contains(stdout, "Verifying...") {
		t.Fatalf("stdout = %q", stdout)
	}
}

func TestMcSetEnablesNothingToChange(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, ipmicore.IpmiGetGlobalEnables, mock.OK(0x08))
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"setenables", "system_event_log=on"}) })
	if code != 0 || !strings.Contains(stdout, "Nothing to change...") {
		t.Fatalf("code=%d stdout=%q", code, stdout)
	}
}

// guidData returns a 16-byte IPMI-encoding GUID whose version nibble is 4.
func guidData() []byte {
	d := make([]byte, 16)
	// ipmi_guid_t: time_hi = le16(d[8:10]); version nibble in bits 12..15.
	// Set time_hi = 0x4000 -> version 4.
	d[8] = 0x00
	d[9] = 0x40
	return d
}

func TestMcGuidDump(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, 0x37, mock.OKData(guidData()))
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"guid", "ipmi"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if !strings.Contains(stdout, "System GUID   : 00000000-0000-4000-0000-000000000000") {
		t.Fatalf("stdout = %q", stdout)
	}
	if !strings.Contains(stdout, "GUID Version  : Random or pseudo-random") {
		t.Fatalf("stdout = %q", stdout)
	}
}

func TestMcSelftestPassed(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, ipmicore.IpmiGetSelfTestResults, mock.OK(0x55, 0x00))
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"selftest"}) })
	if code != 0 || !strings.Contains(stdout, "Selftest: passed") {
		t.Fatalf("code=%d stdout=%q", code, stdout)
	}
}

func TestMcSelftestFatal(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, ipmicore.IpmiGetSelfTestResults, mock.OK(0x58, 0x42))
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"selftest"}) })
	if code != -1 {
		t.Fatalf("exit code = %d, want -1", code)
	}
	if !strings.Contains(stdout, "Selftest     : fatal error") || !strings.Contains(stdout, "Failure code : 42") {
		t.Fatalf("stdout = %q", stdout)
	}
}
