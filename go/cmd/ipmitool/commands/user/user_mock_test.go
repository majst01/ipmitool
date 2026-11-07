package user

import (
	"strings"
	"testing"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
	"codeberg.org/ipmitool/ipmitool/internal/mock"
)

func padName(s string) []byte {
	b := make([]byte, 16)
	copy(b, s)
	return b
}

func TestUserSummary(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, cmdGetUserAccess, mock.OK(10, 3, 1, 0x04))
	m := r.InterfaceWith()

	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"summary"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if !strings.Contains(stdout, "Maximum IDs\t    : 10") ||
		!strings.Contains(stdout, "Enabled User Count  : 3") ||
		!strings.Contains(stdout, "Fixed Name Count    : 1") {
		t.Fatalf("stdout = %q", stdout)
	}
	if len(m.Requests) != 1 || m.Requests[0].Cmd != cmdGetUserAccess {
		t.Fatalf("unexpected requests: %+v", m.Requests)
	}
	if m.Requests[0].Data[0] != channelCurrent {
		t.Fatalf("channel = %#x, want current", m.Requests[0].Data[0])
	}
}

func TestUserList(t *testing.T) {
	r := mock.NewRouter()
	r.On(ipmicore.NetfnApp, cmdGetUserAccess, func(req *ipmicore.Request) (*ipmicore.Response, error) {
		// data[1] is the user id.
		priv := uint8(0x04)
		switch req.Data[1] {
		case 1:
			return mock.OK(2, 0x03, 1, priv|0x10), nil
		case 2:
			return mock.OK(2, 0x02, 0, priv), nil
		}
		return mock.OK(2, 0x00, 0, 0), nil
	})
	r.On(ipmicore.NetfnApp, cmdGetUserName, func(req *ipmicore.Request) (*ipmicore.Response, error) {
		switch req.Data[0] {
		case 1:
			return mock.OKData(padName("root")), nil
		case 2:
			return mock.OKData(padName("admin")), nil
		}
		return mock.OKData(padName("")), nil
	})
	m := r.InterfaceWith()

	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"list"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if !strings.Contains(stdout, "ID  Name\t     Callin  Link Auth\tIPMI Msg   Channel Priv Limit") {
		t.Fatalf("missing header: %q", stdout)
	}
	if !strings.Contains(stdout, "1   root") {
		t.Fatalf("missing root row: %q", stdout)
	}
	if !strings.Contains(stdout, "2   admin") {
		t.Fatalf("missing admin row: %q", stdout)
	}
	if !strings.Contains(stdout, "ADMINISTRATOR") {
		t.Fatalf("missing privilege: %q", stdout)
	}
}

func TestUserSetName(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, cmdSetUserName, mock.OK())
	m := r.InterfaceWith()
	var code int
	_, stderr := mock.Capture(func() { code = Main(m, []string{"set", "name", "3", "bob"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0 (stderr=%q)", code, stderr)
	}
	req := m.Requests[0]
	if req.Cmd != cmdSetUserName {
		t.Fatalf("cmd = %#x", req.Cmd)
	}
	if req.Data[0] != 3 {
		t.Fatalf("user id byte = %#x, want 3", req.Data[0])
	}
	if string(req.Data[1:4]) != "bob" {
		t.Fatalf("name bytes = %v", req.Data[1:4])
	}
	if len(req.Data) != 17 {
		t.Fatalf("data length = %d, want 17", len(req.Data))
	}
}

func TestUserSetNameTooLong(t *testing.T) {
	m := mock.New(nil)
	var code int
	_, stderr := mock.Capture(func() { code = Main(m, []string{"set", "name", "3", "this-name-is-way-too-long"}) })
	if code != -1 || !strings.Contains(stderr, "Username is too long (> 16 bytes)") {
		t.Fatalf("code=%d stderr=%q", code, stderr)
	}
	if len(m.Requests) != 0 {
		t.Fatalf("no request should have been sent")
	}
}

func TestUserDisableEnable(t *testing.T) {
	for _, tc := range []struct {
		cmd string
		op  uint8
	}{
		{"disable", passwordDisableUser},
		{"enable", passwordEnableUser},
	} {
		r := mock.NewRouter()
		r.Static(ipmicore.NetfnApp, cmdSetUserPassword, mock.OK())
		m := r.InterfaceWith()
		var code int
		_, stderr := mock.Capture(func() { code = Main(m, []string{tc.cmd, "5"}) })
		if code != 0 {
			t.Fatalf("%s: exit code = %d (stderr=%q)", tc.cmd, code, stderr)
		}
		req := m.Requests[0]
		if req.Cmd != cmdSetUserPassword {
			t.Fatalf("%s: cmd = %#x", tc.cmd, req.Cmd)
		}
		if req.Data[0] != 5 {
			t.Fatalf("%s: user id = %#x", tc.cmd, req.Data[0])
		}
		if req.Data[1] != (0x03 & tc.op) {
			t.Fatalf("%s: operation = %#x, want %#x", tc.cmd, req.Data[1], tc.op)
		}
	}
}

func TestUserPriv(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, cmdSetUserAccess, mock.OK())
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"priv", "4", "0x4"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	req := m.Requests[0]
	if req.Cmd != cmdSetUserAccess {
		t.Fatalf("cmd = %#x", req.Cmd)
	}
	// changePrivLimitOnly -> d0 high bit clear; channel current in low nibble.
	if req.Data[0]&0x80 != 0 {
		t.Fatalf("d0 = %#x, changePrivLimitOnly should clear bit7", req.Data[0])
	}
	if req.Data[0]&0x0f != channelCurrent {
		t.Fatalf("channel = %#x, want %#x", req.Data[0]&0x0f, channelCurrent)
	}
	if req.Data[1] != 4 || req.Data[2] != 0x04 {
		t.Fatalf("uid/priv = %d/%#x", req.Data[1], req.Data[2])
	}
	if !strings.Contains(stdout, "Set Privilege Level command successful (user 4)") {
		t.Fatalf("stdout = %q", stdout)
	}
}

func TestUserTestPasswordSuccess(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, cmdSetUserPassword, mock.OK())
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"test", "2", "20", "secret"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	req := m.Requests[0]
	if req.Data[0] != (0x80 | 2) {
		t.Fatalf("user id byte = %#x, want 0x82 (20-byte)", req.Data[0])
	}
	if req.Data[1] != passwordTestPassword {
		t.Fatalf("operation = %#x, want test", req.Data[1])
	}
	if string(req.Data[2:8]) != "secret" {
		t.Fatalf("embedded password = %v", req.Data[2:8])
	}
	if !strings.Contains(stdout, "Success") {
		t.Fatalf("stdout = %q", stdout)
	}
}

func TestUserTestPasswordFailure(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, cmdSetUserPassword, mock.CCode(0x80))
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"test", "2", "16", "secret"}) })
	if code != -1 {
		t.Fatalf("exit code = %d, want -1", code)
	}
	if !strings.Contains(stdout, "Failure: password incorrect") {
		t.Fatalf("stdout = %q", stdout)
	}
}

func TestUserInvalidID(t *testing.T) {
	m := mock.New(nil)
	var code int
	_, stderr := mock.Capture(func() { code = Main(m, []string{"disable", "0"}) })
	if code != -1 || !strings.Contains(stderr, "out of range") {
		t.Fatalf("code=%d stderr=%q", code, stderr)
	}
}

func TestUserInvalidCommand(t *testing.T) {
	m := mock.New(nil)
	var code int
	_, stderr := mock.Capture(func() { code = Main(m, []string{"frobnicate"}) })
	if code != -1 || !strings.Contains(stderr, "Invalid user command: 'frobnicate'") {
		t.Fatalf("code=%d stderr=%q", code, stderr)
	}
}
