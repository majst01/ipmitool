package sol

import (
	"strings"
	"testing"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
	"codeberg.org/ipmitool/ipmitool/internal/mock"
)

// solParamHandler returns the scripted parameter data for each SOL parameter
// selector carried in request byte 1.
func solParamHandler() mock.Handler {
	return func(req *ipmicore.Request) (*ipmicore.Response, error) {
		if req.Netfn != ipmicore.NetfnTransport || req.Cmd != ipmicore.IpmiGetSolConfigParameters {
			return &ipmicore.Response{Ccode: ipmicore.CC_INV_CMD}, nil
		}
		selector := req.Data[1]
		switch selector {
		case ipmicore.ParamSetInProgress:
			return mock.OK(selector, 0x00), nil
		case ipmicore.ParamSolEnable:
			return mock.OK(selector, 0x01), nil
		case ipmicore.ParamSolAuthentication:
			return mock.OK(selector, 0x80|0x40|0x04), nil
		case ipmicore.ParamCharacterInterval:
			return mock.OK(selector, 0x0a, 0x05), nil
		case ipmicore.ParamSolRetry:
			return mock.OK(selector, 0x03, 0x02), nil
		case ipmicore.ParamNonVolatileBitRate:
			return mock.OK(selector, 0x0a), nil
		case ipmicore.ParamVolatileBitRate:
			return mock.OK(selector, 0x0a), nil
		case ipmicore.ParamPayloadChannel:
			return mock.OK(selector, 0x01), nil
		case ipmicore.ParamPayloadPort:
			return mock.OK(selector, 0x6f, 0x02), nil
		}
		return mock.OK(selector), nil
	}
}

func TestSolInfo(t *testing.T) {
	m := mock.New(solParamHandler())
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"info"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	for _, want := range []string{
		"Set in progress                 : set-complete",
		"Enabled                         : true",
		"Force Encryption                : true",
		"Force Authentication            : true",
		"Privilege Level                 : ADMINISTRATOR",
		"Character Accumulate Level (ms) : 50",
		"Character Send Threshold        : 5",
		"Retry Count                     : 3",
		"Retry Interval (ms)             : 20",
		"Volatile Bit Rate (kbps)        : 115.2",
		"Non-Volatile Bit Rate (kbps)    : 115.2",
		"Payload Channel                 : 1 (0x01)",
		"Payload Port                    : 623",
	} {
		if !strings.Contains(stdout, want) {
			t.Errorf("stdout missing %q\n%s", want, stdout)
		}
	}
	// Nine parameters, each a Get SOL Configuration Parameters request.
	if len(m.Requests) != 9 {
		t.Fatalf("request count = %d, want 9", len(m.Requests))
	}
	if m.Requests[0].Data[0] != defaultChannel {
		t.Fatalf("channel = %#x, want %#x", m.Requests[0].Data[0], defaultChannel)
	}
}

func TestSolInfoUnsupportedParameter(t *testing.T) {
	m := mock.New(func(req *ipmicore.Request) (*ipmicore.Response, error) {
		if req.Data[1] == ipmicore.ParamSolEnable {
			return mock.CCode(0x80), nil
		}
		return mock.OK(req.Data[1]), nil
	})
	var code int
	_, stderr := mock.Capture(func() { code = Main(m, []string{"info"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if !strings.Contains(stderr, "Info: SOL parameter 'Enable (1)' not supported") {
		t.Fatalf("stderr = %q", stderr)
	}
}

func TestSolPayloadStatus(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, ipmicore.IpmiGetUserPayloadAccess, mock.OK(0x02, 0, 0, 0))
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"payload", "status", "1", "2"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	req := m.Requests[0]
	if req.Cmd != ipmicore.IpmiGetUserPayloadAccess {
		t.Fatalf("cmd = %#x", req.Cmd)
	}
	if req.Data[0] != 1 || req.Data[1] != 2 {
		t.Fatalf("data = %v, want [1 2]", req.Data)
	}
	if !strings.Contains(stdout, "User 2 on channel 1 is enabled") {
		t.Fatalf("stdout = %q", stdout)
	}
}

func TestSolPayloadDisable(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, ipmicore.IpmiSetUserPayloadAccess, mock.OK())
	m := r.InterfaceWith()
	var code int
	_, _ = mock.Capture(func() { code = Main(m, []string{"payload", "disable", "1", "3"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	req := m.Requests[0]
	if req.Cmd != ipmicore.IpmiSetUserPayloadAccess {
		t.Fatalf("cmd = %#x", req.Cmd)
	}
	// channel 1, access = userid | 0x40 (disabled), operation 0x02.
	if req.Data[0] != 1 || req.Data[1] != 0x43 || req.Data[2] != 0x02 {
		t.Fatalf("data = %v, want [1 67 2 ...]", req.Data)
	}
}

func TestSolSetEnabledSendsGuardedSequence(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnTransport, ipmicore.IpmiSetSolConfigParameters, mock.OK())
	m := r.InterfaceWith()
	var code int
	_, stderr := mock.Capture(func() { code = Main(m, []string{"set", "enabled", "true"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0 (stderr=%q)", code, stderr)
	}
	// guard: set-in-progress=1, then enabled=1, then commit-write=2, then 0.
	want := [][]byte{
		{defaultChannel, ipmicore.ParamSetInProgress, 0x01},
		{defaultChannel, ipmicore.ParamSolEnable, 0x01},
		{defaultChannel, ipmicore.ParamSetInProgress, 0x02},
		{defaultChannel, ipmicore.ParamSetInProgress, 0x00},
	}
	if len(m.Requests) != len(want) {
		t.Fatalf("request count = %d, want %d", len(m.Requests), len(want))
	}
	for i, w := range want {
		if len(m.Requests[i].Data) != len(w) {
			t.Fatalf("request %d data = %v, want %v", i, m.Requests[i].Data, w)
		}
		for j := range w {
			if m.Requests[i].Data[j] != w[j] {
				t.Fatalf("request %d data = %v, want %v", i, m.Requests[i].Data, w)
			}
		}
	}
}

func TestSolDeactivateSuccess(t *testing.T) {
	m := mock.New(nil)
	var got uint8
	m.SolDeactivateFn = func(instance uint8) error {
		got = instance
		return nil
	}
	var code int
	_, _ = mock.Capture(func() { code = Main(m, []string{"deactivate", "instance=3"}) })
	if code != 0 || got != 3 {
		t.Fatalf("code=%d instance=%d, want 0/3", code, got)
	}
}

func TestSolDeactivateUnsupported(t *testing.T) {
	m := mock.New(nil)
	var code int
	_, stderr := mock.Capture(func() { code = Main(m, []string{"deactivate"}) })
	if code != -1 || !strings.Contains(stderr, "Error de-activating SOL payload") {
		t.Fatalf("code=%d stderr=%q", code, stderr)
	}
}

func TestSolInvalidChannel(t *testing.T) {
	m := mock.New(nil)
	var code int
	_, stderr := mock.Capture(func() { code = Main(m, []string{"info", "zzz"}) })
	if code != -1 || !strings.Contains(stderr, "Invalid channel number: zzz") {
		t.Fatalf("code=%d stderr=%q", code, stderr)
	}
}
