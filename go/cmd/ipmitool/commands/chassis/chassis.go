// Package chassis implements the `chassis ...` commands and the top-level
// `power` shortcut. It mirrors `lib/ipmi_chassis.c` and the Rust
// `commands/chassis.rs`.
package chassis

import (
	"fmt"
	"os"

	cliargs "codeberg.org/ipmitool/ipmitool/internal/args"
	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
)

// Main implements `ipmitool chassis ...`.
func Main(intf ipmicore.Interface, args []string) int {
	if len(args) == 0 {
		printUsage()
		return -1
	}
	cmd := args[0]
	rest := args[1:]
	switch cmd {
	case "help":
		printUsage()
		return 0
	case "status":
		return status(intf)
	case "selftest":
		return selftest(intf)
	case "power":
		return power(intf, rest)
	case "identify":
		return identify(intf, rest)
	case "poh":
		return poh(intf)
	case "restart_cause":
		return restartCause(intf)
	case "policy":
		return policy(intf, rest)
	default:
		fmt.Fprintf(os.Stderr, "Invalid chassis command: %s\n", cmd)
		return -1
	}
}

// PowerShortcut is the top-level `power` shortcut, which forwards to
// `chassis power`.
func PowerShortcut(intf ipmicore.Interface, args []string) int {
	return power(intf, args)
}

func power(intf ipmicore.Interface, args []string) int {
	if len(args) == 0 {
		fmt.Fprintln(os.Stderr, "chassis power Commands: status, on, off, cycle, reset, diag, soft")
		return 0
	}
	cmd := args[0]
	if cmd == "help" {
		fmt.Fprintln(os.Stderr, "chassis power Commands: status, on, off, cycle, reset, diag, soft")
		return 0
	}
	if cmd == "status" {
		return printPowerStatus(intf)
	}
	var ctl uint8
	switch cmd {
	case "up", "on":
		ctl = ipmicore.ChassisCtlPowerUp
	case "down", "off":
		ctl = ipmicore.ChassisCtlPowerDown
	case "cycle":
		ctl = ipmicore.ChassisCtlPowerCycle
	case "reset":
		ctl = ipmicore.ChassisCtlHardReset
	case "diag":
		ctl = ipmicore.ChassisCtlPulseDiag
	case "acpi", "soft":
		ctl = ipmicore.ChassisCtlAcpiSoft
	default:
		fmt.Fprintf(os.Stderr, "Invalid chassis power command: %s\n", cmd)
		return -1
	}
	return powerControl(intf, ctl)
}

// powerStatus returns the low bit of the chassis power status and an exit
// code: 0 on success, -1 on error (matching the Rust Result<u8, i32>).
func powerStatus(intf ipmicore.Interface) (uint8, int) {
	req := ipmicore.NewRequest(ipmicore.NetfnChassis, 0x1)
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintln(os.Stderr, "Unable to get Chassis Power Status")
		return 0, -1
	}
	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Get Chassis Power Status failed: %s\n",
			ipmicore.CompletionCodeStr(rsp.Ccode))
		return 0, -1
	}
	var b byte
	if len(rsp.Data) > 0 {
		b = rsp.Data[0]
	}
	return b & 1, 0
}

func printPowerStatus(intf ipmicore.Interface) int {
	ps, code := powerStatus(intf)
	if code != 0 {
		return code
	}
	state := "off"
	if ps != 0 {
		state = "on"
	}
	fmt.Printf("Chassis Power is %s\n", state)
	return 0
}

func powerControl(intf ipmicore.Interface, ctl uint8) int {
	req := ipmicore.NewRequest(ipmicore.NetfnChassis, 0x2)
	req.Data = []byte{ctl}
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Unable to set Chassis Power Control to %s\n",
			ipmicore.PowerControlStr(ctl))
		return -1
	}
	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Set Chassis Power Control to %s failed: %s\n",
			ipmicore.PowerControlStr(ctl), ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}
	fmt.Printf("Chassis Power Control: %s\n", ipmicore.PowerControlStr(ctl))
	return 0
}

func status(intf ipmicore.Interface) int {
	req := ipmicore.NewRequest(ipmicore.NetfnChassis, 0x1)
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintln(os.Stderr, "Error sending Chassis Status command")
		return -1
	}
	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Error sending Chassis Status command: %s\n",
			ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}
	d := rsp.Data
	if len(d) < 3 {
		fmt.Fprintln(os.Stderr, "Error sending Chassis Status command: short response")
		return -1
	}

	fmt.Printf("System Power         : %s\n", onoff(d[0]&0x1 != 0))
	fmt.Printf("Power Overload       : %s\n", boolstr(d[0]&0x2 != 0))
	fmt.Printf("Power Interlock      : %s\n", activeInactive(d[0]&0x4 != 0))
	fmt.Printf("Main Power Fault     : %s\n", boolstr(d[0]&0x8 != 0))
	fmt.Printf("Power Control Fault  : %s\n", boolstr(d[0]&0x10 != 0))
	fmt.Print("Power Restore Policy : ")
	switch (d[0] & 0x60) >> 5 {
	case 0x0:
		fmt.Println("always-off")
	case 0x1:
		fmt.Println("previous")
	case 0x2:
		fmt.Println("always-on")
	default:
		fmt.Println("unknown")
	}

	fmt.Print("Last Power Event     : ")
	if d[1]&0x1 != 0 {
		fmt.Print("ac-failed ")
	}
	if d[1]&0x2 != 0 {
		fmt.Print("overload ")
	}
	if d[1]&0x4 != 0 {
		fmt.Print("interlock ")
	}
	if d[1]&0x8 != 0 {
		fmt.Print("fault ")
	}
	if d[1]&0x10 != 0 {
		fmt.Print("command")
	}
	fmt.Println()

	fmt.Printf("Chassis Intrusion    : %s\n", activeInactive(d[2]&0x1 != 0))
	fmt.Printf("Front-Panel Lockout  : %s\n", activeInactive(d[2]&0x2 != 0))
	fmt.Printf("Drive Fault          : %s\n", boolstr(d[2]&0x4 != 0))
	fmt.Printf("Cooling/Fan Fault    : %s\n", boolstr(d[2]&0x8 != 0))

	if len(d) > 3 {
		if d[3] == 0 {
			fmt.Println("Front Panel Control  : none")
		} else {
			fmt.Printf("Sleep Button Disable : %s\n", allowedNotAllowed(d[3]&0x80 != 0))
			fmt.Printf("Diag Button Disable  : %s\n", allowedNotAllowed(d[3]&0x40 != 0))
			fmt.Printf("Reset Button Disable : %s\n", allowedNotAllowed(d[3]&0x20 != 0))
			fmt.Printf("Power Button Disable : %s\n", allowedNotAllowed(d[3]&0x10 != 0))
			fmt.Printf("Sleep Button Disabled: %s\n", boolstr(d[3]&0x08 != 0))
			fmt.Printf("Diag Button Disabled : %s\n", boolstr(d[3]&0x04 != 0))
			fmt.Printf("Reset Button Disabled: %s\n", boolstr(d[3]&0x02 != 0))
			fmt.Printf("Power Button Disabled: %s\n", boolstr(d[3]&0x01 != 0))
		}
	}
	return 0
}

func identify(intf ipmicore.Interface, args []string) int {
	var interval uint8
	forceOn := false
	length := 0

	if len(args) == 0 {
		// No argument.
	} else {
		switch args[0] {
		case "help":
			fmt.Fprintln(os.Stderr, "chassis identify <interval>")
			fmt.Fprintln(os.Stderr, "                 default is 15 seconds")
			fmt.Fprintln(os.Stderr, "                 0 to turn off")
			fmt.Fprintln(os.Stderr, "                 force to turn on indefinitely")
			return 0
		case "force":
			forceOn = true
			length = 2
		default:
			val, ok := cliargs.ParseU8(args[0])
			if !ok {
				fmt.Fprintln(os.Stderr, "Invalid interval given.")
				return -1
			}
			interval = val
			length = 1
		}
	}

	req := ipmicore.NewRequest(ipmicore.NetfnChassis, 0x4)
	switch length {
	case 2:
		req.Data = []byte{interval, 1}
	case 1:
		req.Data = []byte{interval}
	}

	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintln(os.Stderr, "Unable to set Chassis Identify")
		return -1
	}
	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Set Chassis Identify failed: %s\n",
			ipmicore.CompletionCodeStr(rsp.Ccode))
		if forceOn {
			fmt.Fprintln(os.Stderr, "Chassis may not support Force Identify On")
		}
		return -1
	}

	fmt.Print("Chassis identify interval: ")
	switch {
	case len(args) == 0:
		fmt.Println("default (15 seconds)")
	case forceOn:
		fmt.Println("indefinite")
	case interval == 0:
		fmt.Println("off")
	default:
		fmt.Printf("%d seconds\n", interval)
	}
	return 0
}

func poh(intf ipmicore.Interface) int {
	req := ipmicore.NewRequest(ipmicore.NetfnChassis, 0xf)
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintln(os.Stderr, "Unable to get Chassis Power-On-Hours")
		return -1
	}
	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Get Chassis Power-On-Hours failed: %s\n",
			ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}
	if len(rsp.Data) < 5 {
		fmt.Fprintln(os.Stderr, "Get Chassis Power-On-Hours failed: short response")
		return -1
	}
	minsPerCount := rsp.Data[0]
	count := uint32(rsp.Data[1]) | uint32(rsp.Data[2])<<8 |
		uint32(rsp.Data[3])<<16 | uint32(rsp.Data[4])<<24
	totalMinutes := float64(count) * float64(minsPerCount)
	days := uint32(totalMinutes / 1440.0)
	rem := totalMinutes - float64(days)*1440.0
	hours := uint32(rem / 60.0)
	minutes := rem - float64(hours)*60.0

	if minsPerCount < 60 {
		fmt.Printf("POH Counter  : %d days, %d hours, %d minutes\n", days, hours, uint32(minutes))
	} else {
		fmt.Printf("POH Counter  : %d days, %d hours\n", days, hours)
	}
	return 0
}

func restartCause(intf ipmicore.Interface) int {
	req := ipmicore.NewRequest(ipmicore.NetfnChassis, 0x7)
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintln(os.Stderr, "Unable to get Chassis Restart Cause")
		return -1
	}
	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Get Chassis Restart Cause failed: %s\n",
			ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}
	var b byte
	if len(rsp.Data) > 0 {
		b = rsp.Data[0]
	}
	cause := b & 0xf
	fmt.Printf("System restart cause: %s\n", ipmicore.ChassisRestartCauseStr(cause))
	return 0
}

func policy(intf ipmicore.Interface, args []string) int {
	if len(args) == 0 {
		printPolicyHelp()
		return 0
	}
	cmd := args[0]
	if cmd == "help" {
		printPolicyHelp()
		return 0
	}
	var ctl uint8
	switch cmd {
	case "list":
		ctl = ipmicore.ChassisPolicyNoChange
	case "always-on":
		ctl = ipmicore.ChassisPolicyAlwaysOn
	case "previous":
		ctl = ipmicore.ChassisPolicyPrevious
	case "always-off":
		ctl = ipmicore.ChassisPolicyAlwaysOff
	default:
		fmt.Fprintf(os.Stderr, "Invalid chassis policy: %s\n", cmd)
		return -1
	}

	req := ipmicore.NewRequest(ipmicore.NetfnChassis, 0x6)
	req.Data = []byte{ctl}
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintln(os.Stderr, "Error in Power Restore Policy command")
		return -1
	}
	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Power Restore Policy command failed: %s\n",
			ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}

	if ctl == ipmicore.ChassisPolicyNoChange {
		var bits byte
		if len(rsp.Data) > 0 {
			bits = rsp.Data[0]
		}
		fmt.Print("Supported chassis power policy:  ")
		if bits&(1<<ipmicore.ChassisPolicyAlwaysOff) != 0 {
			fmt.Print("always-off ")
		}
		if bits&(1<<ipmicore.ChassisPolicyAlwaysOn) != 0 {
			fmt.Print("always-on ")
		}
		if bits&(1<<ipmicore.ChassisPolicyPrevious) != 0 {
			fmt.Print("previous")
		}
		fmt.Println()
	} else {
		fmt.Print("Set chassis power restore policy to ")
		switch ctl {
		case ipmicore.ChassisPolicyAlwaysOn:
			fmt.Println("always-on")
		case ipmicore.ChassisPolicyAlwaysOff:
			fmt.Println("always-off")
		case ipmicore.ChassisPolicyPrevious:
			fmt.Println("previous")
		default:
			fmt.Println("unknown")
		}
	}
	return 0
}

func selftest(intf ipmicore.Interface) int {
	req := ipmicore.NewRequest(ipmicore.NetfnApp, ipmicore.IpmiGetSelfTestResults)
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintln(os.Stderr, "Error sending Get Self Test command")
		return -1
	}
	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Error sending Get Self Test command: %s\n",
			ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}
	var code, test byte
	if len(rsp.Data) > 0 {
		code = rsp.Data[0]
	}
	if len(rsp.Data) > 1 {
		test = rsp.Data[1]
	}
	fmt.Print("Self Test Results    : ")
	switch code {
	case 0x55:
		fmt.Println("passed")
	case 0x56:
		fmt.Println("not implemented")
	case 0x57:
		fmt.Println("device error")
		broken := [8]string{
			"firmware corrupted",
			"boot block corrupted",
			"FRU Internal Use Area corrupted",
			"SDR Repository empty",
			"IPMB not responding",
			"cannot access BMC FRU",
			"cannot access SDR Repository",
			"cannot access SEL Device",
		}
		for i, name := range broken {
			if test&(1<<uint(i)) != 0 {
				fmt.Printf("                       [%s]\n", name)
			}
		}
	case 0x58:
		fmt.Printf("Fatal hardware error: %02xh\n", test)
	default:
		fmt.Printf("Device-specific failure %02xh:%02xh\n", code, test)
	}
	return 0
}

func onoff(b bool) string {
	if b {
		return "on"
	}
	return "off"
}

func boolstr(b bool) string {
	if b {
		return "true"
	}
	return "false"
}

func activeInactive(b bool) string {
	if b {
		return "active"
	}
	return "inactive"
}

func allowedNotAllowed(b bool) string {
	if b {
		return "allowed"
	}
	return "not allowed"
}

func printPolicyHelp() {
	fmt.Fprintln(os.Stderr, "chassis policy <state>")
	fmt.Fprintln(os.Stderr, "   list        : return supported policies")
	fmt.Fprintln(os.Stderr, "   always-on   : turn on when power is restored")
	fmt.Fprintln(os.Stderr, "   previous    : return to previous state when power is restored")
	fmt.Fprintln(os.Stderr, "   always-off  : stay off after power is restored")
}

func printUsage() {
	fmt.Fprintln(os.Stderr, "Chassis Commands:\n  status, power, policy, restart_cause\n  poh, identify, selftest,\n  bootdev, bootparam, bootmbox")
}
