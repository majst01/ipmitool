package raw

import (
	"strings"
	"testing"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
	"codeberg.org/ipmitool/ipmitool/internal/mock"
)

func TestRawSendsRequestAndPrintsResponse(t *testing.T) {
	r := mock.NewRouter()
	r.On(ipmicore.NetfnApp, 0x01, func(req *ipmicore.Request) (*ipmicore.Response, error) {
		return mock.OK(0xde, 0xad, 0xbe, 0xef), nil
	})
	m := r.InterfaceWith()
	m.Params.TargetLun = 0x02
	m.Params.TargetChannel = 0x07

	var code int
	stdout, stderr := mock.Capture(func() {
		code = Main(m, []string{"0x06", "0x01", "0xaa", "0xbb"})
	})

	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if len(m.Requests) != 1 {
		t.Fatalf("request count = %d, want 1", len(m.Requests))
	}
	req := m.Requests[0]
	if req.Netfn != ipmicore.NetfnApp || req.Cmd != 0x01 || req.Lun != 0x02 {
		t.Fatalf("unexpected request: %+v", req)
	}
	if len(req.Data) != 2 || req.Data[0] != 0xaa || req.Data[1] != 0xbb {
		t.Fatalf("unexpected request data: %v", req.Data)
	}
	if !strings.Contains(stderr, "RAW REQ (channel=0x7 netfn=0x6 lun=0x2 cmd=0x1 data_len=2)") {
		t.Fatalf("missing RAW REQ header, got: %q", stderr)
	}
	if !strings.Contains(stderr, "RAW RSP (4 bytes)") {
		t.Fatalf("missing RAW RSP header, got: %q", stderr)
	}
	if strings.TrimSpace(stdout) != "de ad be ef" {
		t.Fatalf("stdout = %q, want %q", stdout, "de ad be ef")
	}
}

func TestRawAcceptsSymbolicNetfn(t *testing.T) {
	r := mock.NewRouter()
	m := r.InterfaceWith()
	var code int
	_, _ = mock.Capture(func() {
		code = Main(m, []string{"Application", "0x01"})
	})
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if m.Requests[0].Netfn != ipmicore.NetfnApp {
		t.Fatalf("netfn = %#x, want App", m.Requests[0].Netfn)
	}
}

func TestRawInvalidNetfn(t *testing.T) {
	m := mock.New(nil)
	var code int
	var stderr string
	_, stderr = mock.Capture(func() {
		code = Main(m, []string{"bogus", "0x01"})
	})
	if code != -1 {
		t.Fatalf("exit code = %d, want -1", code)
	}
	if !strings.Contains(stderr, `Given netfn "bogus" is invalid.`) {
		t.Fatalf("stderr = %q", stderr)
	}
	if len(m.Requests) != 0 {
		t.Fatalf("no request should have been sent")
	}
}

func TestRawInvalidCommand(t *testing.T) {
	m := mock.New(nil)
	var code int
	_, stderr := mock.Capture(func() {
		code = Main(m, []string{"0x06", "zzz"})
	})
	if code != -1 || !strings.Contains(stderr, `Given command "zzz" is invalid.`) {
		t.Fatalf("code=%d stderr=%q", code, stderr)
	}
}

func TestRawNotEnoughParameters(t *testing.T) {
	m := mock.New(nil)
	var code int
	_, stderr := mock.Capture(func() {
		code = Main(m, []string{"0x06"})
	})
	if code != -1 || !strings.Contains(stderr, "Not enough parameters given.") {
		t.Fatalf("code=%d stderr=%q", code, stderr)
	}
}

func TestRawCompletionError(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, 0x01, mock.CCode(ipmicore.CC_INV_CMD))
	m := r.InterfaceWith()
	var code int
	_, stderr := mock.Capture(func() {
		code = Main(m, []string{"0x06", "0x01"})
	})
	if code != -1 {
		t.Fatalf("exit code = %d, want -1", code)
	}
	if !strings.Contains(stderr, "Invalid command") {
		t.Fatalf("stderr = %q", stderr)
	}
}

func TestRawHelp(t *testing.T) {
	m := mock.New(nil)
	var code int
	_, stderr := mock.Capture(func() {
		code = Main(m, []string{"help"})
	})
	if code != 0 || !strings.Contains(stderr, "RAW Commands:") {
		t.Fatalf("code=%d stderr=%q", code, stderr)
	}
}
