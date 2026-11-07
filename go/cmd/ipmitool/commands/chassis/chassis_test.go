package chassis

import (
	"strings"
	"testing"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
	"codeberg.org/ipmitool/ipmitool/internal/mock"
)

func TestPowerStatusOn(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnChassis, 0x1, mock.OK(0x01))
	m := r.InterfaceWith()

	var code int
	stdout, _ := mock.Capture(func() { code = PowerShortcut(m, []string{"status"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if !strings.Contains(stdout, "Chassis Power is on") {
		t.Fatalf("stdout = %q", stdout)
	}
}

func TestPowerStatusOff(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnChassis, 0x1, mock.OK(0x00))
	m := r.InterfaceWith()

	var code int
	stdout, _ := mock.Capture(func() { code = PowerShortcut(m, []string{"status"}) })
	if code != 0 || !strings.Contains(stdout, "Chassis Power is off") {
		t.Fatalf("code=%d stdout=%q", code, stdout)
	}
}

func TestPowerControlSendsControlByte(t *testing.T) {
	cases := []struct {
		arg      string
		ctl      uint8
		expected string
	}{
		{"on", ipmicore.ChassisCtlPowerUp, "Chassis Power Control: Up/On"},
		{"off", ipmicore.ChassisCtlPowerDown, "Chassis Power Control: Down/Off"},
		{"cycle", ipmicore.ChassisCtlPowerCycle, "Chassis Power Control: Cycle"},
		{"reset", ipmicore.ChassisCtlHardReset, "Chassis Power Control: Reset"},
		{"diag", ipmicore.ChassisCtlPulseDiag, "Chassis Power Control: Diag"},
		{"soft", ipmicore.ChassisCtlAcpiSoft, "Chassis Power Control: Soft"},
	}
	for _, tc := range cases {
		t.Run(tc.arg, func(t *testing.T) {
			r := mock.NewRouter()
			r.Static(ipmicore.NetfnChassis, 0x2, mock.OK())
			m := r.InterfaceWith()
			var code int
			stdout, _ := mock.Capture(func() { code = PowerShortcut(m, []string{tc.arg}) })
			if code != 0 {
				t.Fatalf("exit code = %d, want 0", code)
			}
			if len(m.Requests) != 1 || m.Requests[0].Cmd != 0x2 {
				t.Fatalf("unexpected requests: %+v", m.Requests)
			}
			if len(m.Requests[0].Data) != 1 || m.Requests[0].Data[0] != tc.ctl {
				t.Fatalf("data = %v, want [%d]", m.Requests[0].Data, tc.ctl)
			}
			if !strings.Contains(stdout, tc.expected) {
				t.Fatalf("stdout = %q, want %q", stdout, tc.expected)
			}
		})
	}
}

func TestPowerStatusError(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnChassis, 0x1, mock.CCode(ipmicore.CC_INV_CMD))
	m := r.InterfaceWith()
	var code int
	_, stderr := mock.Capture(func() { code = PowerShortcut(m, []string{"status"}) })
	if code != -1 || !strings.Contains(stderr, "Get Chassis Power Status failed") {
		t.Fatalf("code=%d stderr=%q", code, stderr)
	}
}

func TestChassisStatusPrintsLabels(t *testing.T) {
	// d[0]: power on, overload+interlock set, restore policy = previous (0x1).
	// d[1]: ac-failed + command.
	// d[2]: intrusion + cooling fault.
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnChassis, 0x1, mock.OK(0x01|0x02|0x04|0x20, 0x01|0x10, 0x01|0x08))
	m := r.InterfaceWith()

	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"status"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	for _, want := range []string{
		"System Power         : on",
		"Power Overload       : true",
		"Power Interlock      : active",
		"Power Restore Policy : previous",
		"ac-failed ",
		"command",
		"Chassis Intrusion    : active",
		"Cooling/Fan Fault    : true",
	} {
		if !strings.Contains(stdout, want) {
			t.Errorf("stdout missing %q\n%s", want, stdout)
		}
	}
}

func TestChassisIdentify(t *testing.T) {
	cases := []struct {
		args  []string
		data  []byte
		print string
	}{
		{[]string{}, nil, "Chassis identify interval: default (15 seconds)"},
		{[]string{"10"}, []byte{10}, "Chassis identify interval: 10 seconds"},
		{[]string{"0"}, []byte{0}, "Chassis identify interval: off"},
		{[]string{"force"}, []byte{0, 1}, "Chassis identify interval: indefinite"},
	}
	for _, tc := range cases {
		r := mock.NewRouter()
		r.Static(ipmicore.NetfnChassis, 0x4, mock.OK())
		m := r.InterfaceWith()
		var code int
		args := append([]string{"identify"}, tc.args...)
		stdout, _ := mock.Capture(func() { code = Main(m, args) })
		if code != 0 {
			t.Fatalf("args %v: exit code = %d, want 0", tc.args, code)
		}
		want := tc.data
		var got []byte
		if len(m.Requests) == 1 {
			got = m.Requests[0].Data
		}
		if string(got) != string(want) {
			t.Fatalf("args %v: data = %v, want %v", tc.args, got, want)
		}
		if !strings.Contains(stdout, tc.print) {
			t.Fatalf("args %v: stdout = %q, want %q", tc.args, stdout, tc.print)
		}
	}
}

func TestChassisPohMinutes(t *testing.T) {
	// minsPerCount = 60, count = 1 -> 1 hour -> "0 days, 1 hours".
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnChassis, 0xf, mock.OK(60, 1, 0, 0, 0))
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"poh"}) })
	if code != 0 || !strings.Contains(stdout, "POH Counter  : 0 days, 1 hours") {
		t.Fatalf("code=%d stdout=%q", code, stdout)
	}
}

func TestChassisRestartCause(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnChassis, 0x7, mock.OK(0x04))
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"restart_cause"}) })
	if code != 0 || !strings.Contains(stdout, "System restart cause: watchdog expired") {
		t.Fatalf("code=%d stdout=%q", code, stdout)
	}
}

func TestChassisPolicyList(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnChassis, 0x6, mock.OK(0x07))
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"policy", "list"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if m.Requests[0].Data[0] != ipmicore.ChassisPolicyNoChange {
		t.Fatalf("policy select = %d, want no-change", m.Requests[0].Data[0])
	}
	for _, want := range []string{"always-off", "always-on", "previous"} {
		if !strings.Contains(stdout, want) {
			t.Errorf("stdout missing %q: %s", want, stdout)
		}
	}
}

func TestChassisPolicySet(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnChassis, 0x6, mock.OK())
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"policy", "always-on"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if m.Requests[0].Data[0] != ipmicore.ChassisPolicyAlwaysOn {
		t.Fatalf("policy select = %d", m.Requests[0].Data[0])
	}
	if !strings.Contains(stdout, "Set chassis power restore policy to always-on") {
		t.Fatalf("stdout = %q", stdout)
	}
}

func TestChassisSelftestPassed(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, ipmicore.IpmiGetSelfTestResults, mock.OK(0x55, 0x00))
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"selftest"}) })
	if code != 0 || !strings.Contains(stdout, "Self Test Results    : passed") {
		t.Fatalf("code=%d stdout=%q", code, stdout)
	}
}

func TestChassisSelftestDeviceError(t *testing.T) {
	// 0x57 device error, bit 0 -> firmware corrupted.
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, ipmicore.IpmiGetSelfTestResults, mock.OK(0x57, 0x01))
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"selftest"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if !strings.Contains(stdout, "device error") || !strings.Contains(stdout, "[firmware corrupted]") {
		t.Fatalf("stdout = %q", stdout)
	}
}
