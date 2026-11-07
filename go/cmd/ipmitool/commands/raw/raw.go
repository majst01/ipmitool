// Package raw implements the `raw` command: send an arbitrary IPMI request and
// print the response. It mirrors `ipmi_raw_main` in `lib/ipmi_raw.c` and the
// Rust `commands/raw.rs`.
package raw

import (
	"fmt"
	"os"

	cliargs "codeberg.org/ipmitool/ipmitool/internal/args"
	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
)

// Main implements `ipmitool raw <netfn> <cmd> [data...]`.
func Main(intf ipmicore.Interface, args []string) int {
	if len(args) == 1 && args[0] == "help" {
		printHelp()
		return 0
	}
	if len(args) < 2 {
		fmt.Fprintln(os.Stderr, "Not enough parameters given.")
		printHelp()
		return -1
	}
	if len(args) > 256 {
		fmt.Fprintln(os.Stderr, "Raw command input limit (256 bytes) exceeded")
		return -1
	}

	lun := intf.TargetLun()
	netfn, ok := resolveNetfn(args[0])
	if !ok {
		fmt.Fprintf(os.Stderr, "Given netfn \"%s\" is invalid.\n", args[0])
		return -1
	}
	cmd, ok := cliargs.ParseU8(args[1])
	if !ok {
		fmt.Fprintf(os.Stderr, "Given command \"%s\" is invalid.\n", args[1])
		return -1
	}

	data := make([]byte, 0, len(args)-2)
	for _, a := range args[2:] {
		v, ok := cliargs.ParseU8(a)
		if !ok {
			fmt.Fprintf(os.Stderr, "Given data \"%s\" is invalid.\n", a)
			return -1
		}
		data = append(data, v)
	}

	channel := intf.TargetChannel() & 0x0f
	fmt.Fprintf(os.Stderr, "RAW REQ (channel=0x%x netfn=0x%x lun=0x%x cmd=0x%x data_len=%d)\n",
		channel, netfn, lun, cmd, len(data))

	req := ipmicore.NewRequest(netfn, cmd)
	req.Lun = lun
	req.Data = data

	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Unable to send RAW command (channel=0x%x netfn=0x%x lun=0x%x cmd=0x%x)\n",
			channel, netfn, lun, cmd)
		fmt.Fprintln(os.Stderr, err)
		return -1
	}

	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Unable to send RAW command (channel=0x%x netfn=0x%x lun=0x%x cmd=0x%x rsp=0x%x): %s\n",
			channel, netfn, lun, cmd, rsp.Ccode, ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}

	fmt.Fprintf(os.Stderr, "RAW RSP (%d bytes)\n", len(rsp.Data))
	for i, b := range rsp.Data {
		if i%16 == 0 && i != 0 {
			fmt.Println()
		}
		fmt.Printf(" %02x", b)
	}
	fmt.Println()

	return 0
}

func resolveNetfn(s string) (uint8, bool) {
	if n, ok := ipmicore.NetfnFromStr(s); ok {
		return n, true
	}
	return cliargs.ParseU8(s)
}

func printHelp() {
	fmt.Fprintln(os.Stderr, "RAW Commands:  raw <netfn> <cmd> [data]")
	fmt.Fprintln(os.Stderr, "Network Function Codes")
	for _, e := range []struct{ val, name string }{
		{"0", "Chassis"},
		{"2", "Bridge"},
		{"4", "SensorEvent"},
		{"6", "Application"},
		{"8", "Firmware"},
		{"a", "Storage"},
		{"c", "Transport"},
	} {
		fmt.Fprintf(os.Stderr, "\t%-16s  %s\n", e.val, e.name)
	}
	fmt.Fprintln(os.Stderr, "(can also use raw hex values)")
}
