// Package mc implements the `mc`/`bmc` command: Management Controller status
// and global enables. Mirrors `lib/ipmi_mc.c` and the Rust `commands/mc.rs`
// (info, reset, selftest, getenables, setenables, guid).
package mc

import (
	"fmt"
	"os"
	"strings"
	"time"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
)

// adtlDevSupport holds the Additional Device Support bits, indexed by bit
// number.
var adtlDevSupport = [8]string{
	"Sensor Device",
	"SDR Repository Device",
	"SEL Device",
	"FRU Inventory Device",
	"IPMB Event Receiver",
	"IPMB Event Generator",
	"Bridge",
	"Chassis Device",
}

type enablesBit struct {
	name string
	desc string
	mask uint8
}

var mcEnables = [7]enablesBit{
	{"recv_msg_intr", "Receive Message Queue Interrupt", 1 << 0},
	{"event_msg_intr", "Event Message Buffer Full Interrupt", 1 << 1},
	{"event_msg", "Event Message Buffer", 1 << 2},
	{"system_event_log", "System Event Logging", 1 << 3},
	{"oem0", "OEM 0", 1 << 5},
	{"oem1", "OEM 1", 1 << 6},
	{"oem2", "OEM 2", 1 << 7},
}

// Main implements `ipmitool mc <subcommand>`.
func Main(intf ipmicore.Interface, args []string) int {
	if len(args) == 0 {
		fmt.Fprintln(os.Stderr, "Not enough parameters given.")
		printUsage()
		return -1
	}
	cmd := args[0]
	rest := args[1:]
	switch cmd {
	case "help":
		printUsage()
		return 0
	case "reset":
		return reset(intf, rest)
	case "info":
		return info(intf)
	case "guid":
		return guid(intf, rest)
	case "getenables":
		return getEnables(intf)
	case "setenables":
		return setEnables(intf, rest)
	case "selftest":
		return selftest(intf)
	default:
		fmt.Fprintf(os.Stderr, "Invalid mc command: %s\n", cmd)
		return -1
	}
}

func info(intf ipmicore.Interface) int {
	req := ipmicore.NewRequest(ipmicore.NetfnApp, ipmicore.IpmiGetDeviceID)
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintln(os.Stderr, "Get Device ID command failed")
		return -1
	}
	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Get Device ID command failed: %s\n", ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}
	d := rsp.Data
	if len(d) < 11 {
		fmt.Fprintln(os.Stderr, "Get Device ID command failed: short response")
		return -1
	}

	deviceID := d[0]
	deviceRevision := d[1]
	fwRev1 := d[2]
	fwRev2 := d[3]
	ipmiVersion := d[4]
	adtl := d[5]
	manufacturer := uint32(d[6]) | uint32(d[7])<<8 | uint32(d[8])<<16
	productID := uint16(d[9]) | uint16(d[10])<<8

	fmt.Printf("Device ID                 : %d\n", deviceID)
	fmt.Printf("Device Revision           : %d\n", deviceRevision&0x0f)
	fmt.Printf("Firmware Revision         : %d.%02x\n", fwRev1&0x7f, fwRev2)
	fmt.Printf("IPMI Version              : %x.%x\n", ipmiVersion&0x0f, (ipmiVersion&0xf0)>>4)
	registry := ipmicore.LoadOemRegistry()

	fmt.Printf("Manufacturer ID           : %d\n", manufacturer)
	fmt.Printf("Manufacturer Name         : %s\n", registry.ManufacturerName(manufacturer))
	fmt.Printf("Product ID                : %d (0x%02x%02x)\n", productID, d[10], d[9])
	fmt.Printf("Product Name              : %s\n", registry.ProductName(manufacturer, productID))
	if fwRev1&0x80 != 0 {
		fmt.Println("Device Available          : no")
	} else {
		fmt.Println("Device Available          : yes")
	}
	if deviceRevision&0x80 != 0 {
		fmt.Println("Provides Device SDRs      : yes")
	} else {
		fmt.Println("Provides Device SDRs      : no")
	}
	fmt.Println("Additional Device Support :")
	for i, name := range adtlDevSupport {
		if adtl&(1<<uint(i)) != 0 {
			fmt.Printf("    %s\n", name)
		}
	}
	if len(d) == 15 {
		fmt.Println("Aux Firmware Rev Info     : ")
		fmt.Printf("    0x%02x\n    0x%02x\n    0x%02x\n    0x%02x\n", d[11], d[12], d[13], d[14])
	}
	return 0
}

func reset(intf ipmicore.Interface, args []string) int {
	if len(args) == 0 {
		fmt.Fprintln(os.Stderr, "Not enough parameters given.")
		fmt.Fprintln(os.Stderr, "usage: mc reset <warm|cold>")
		return -1
	}
	kind := args[0]
	if kind == "help" {
		fmt.Fprintln(os.Stderr, "usage: mc reset <warm|cold>")
		return 0
	}
	var cmd uint8
	var label string
	switch kind {
	case "cold":
		cmd = 0x02
		label = "cold"
	case "warm":
		cmd = 0x03
		label = "warm"
	default:
		fmt.Fprintf(os.Stderr, "Invalid mc reset command: %s\n", kind)
		fmt.Fprintln(os.Stderr, "usage: mc reset <warm|cold>")
		return -1
	}

	req := ipmicore.NewRequest(ipmicore.NetfnApp, cmd)
	rsp, err := intf.SendRecv(req)
	switch {
	// A cold reset commonly gets no answer; that is expected.
	case err != nil && label == "cold":
	case err != nil:
		fmt.Fprintln(os.Stderr, "MC reset command failed.")
		return -1
	case !rsp.OK():
		fmt.Fprintf(os.Stderr, "MC reset command failed: %s\n", ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}
	fmt.Printf("Sent %s reset command to MC\n", label)
	return 0
}

func getEnables(intf ipmicore.Interface) int {
	req := ipmicore.NewRequest(ipmicore.NetfnApp, ipmicore.IpmiGetGlobalEnables)
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintln(os.Stderr, "Get Global Enables command failed")
		return -1
	}
	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Get Global Enables command failed: %s\n", ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}
	var en uint8
	if len(rsp.Data) > 0 {
		en = rsp.Data[0]
	}
	for _, bf := range mcEnables {
		state := "dis"
		if en&bf.mask != 0 {
			state = "en"
		}
		fmt.Printf("%-40s : %sabled\n", bf.desc, state)
	}
	return 0
}

func setEnables(intf ipmicore.Interface, args []string) int {
	if len(args) == 0 {
		printUsage()
		return -1
	}
	if args[0] == "help" {
		printUsage()
		return 0
	}

	req := ipmicore.NewRequest(ipmicore.NetfnApp, ipmicore.IpmiGetGlobalEnables)
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintln(os.Stderr, "Get Global Enables command failed")
		return -1
	}
	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Get Global Enables command failed: %s\n", ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}
	var original uint8
	if len(rsp.Data) > 0 {
		original = rsp.Data[0]
	}
	en := original

	for _, arg := range args {
		for _, bf := range mcEnables {
			prefix := bf.name + "="
			setting, ok := strings.CutPrefix(arg, prefix)
			if !ok {
				continue
			}
			switch setting {
			case "off":
				fmt.Printf("Disabling %s\n", bf.desc)
				en &^= bf.mask
			case "on":
				fmt.Printf("Enabling %s\n", bf.desc)
				en |= bf.mask
			default:
				fmt.Fprintf(os.Stderr, "Unrecognized option: %s\n", arg)
			}
		}
	}

	if en == original {
		fmt.Println("\nNothing to change...")
		return getEnables(intf)
	}

	set := ipmicore.NewRequest(ipmicore.NetfnApp, ipmicore.IpmiSetGlobalEnables)
	set.Data = []byte{en}
	rsp, err = intf.SendRecv(set)
	if err != nil {
		fmt.Fprintln(os.Stderr, "Set Global Enables command failed")
		return -1
	}
	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Set Global Enables command failed: %s\n", ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}

	fmt.Println("\nVerifying...")
	return getEnables(intf)
}

func selftest(intf ipmicore.Interface) int {
	req := ipmicore.NewRequest(ipmicore.NetfnApp, ipmicore.IpmiGetSelfTestResults)
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintln(os.Stderr, "No response from devices")
		return -1
	}
	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Bad response: (%s)\n", ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}
	var code, test uint8
	if len(rsp.Data) > 0 {
		code = rsp.Data[0]
	}
	if len(rsp.Data) > 1 {
		test = rsp.Data[1]
	}
	switch code {
	case 0x55:
		fmt.Println("Selftest: passed")
		return 0
	case 0x56:
		fmt.Println("Selftest: not implemented")
		return -1
	case 0x57:
		fmt.Println("Selftest: device corrupted")
		checks := [8]struct {
			mask uint8
			msg  string
		}{
			{0x80, " -> SEL device not accessible"},
			{0x40, " -> SDR repository not accessible"},
			{0x20, "FRU device not accessible"},
			{0x10, "IPMB signal lines do not respond"},
			{0x08, "SDR repository empty"},
			{0x04, "Internal Use Area corrupted"},
			{0x02, "Controller update boot block corrupted"},
			{0x01, "controller operational firmware corrupted"},
		}
		for _, c := range checks {
			if test&c.mask != 0 {
				fmt.Println(c.msg)
			}
		}
		return -1
	case 0x58:
		fmt.Println("Selftest     : fatal error")
		fmt.Printf("Failure code : %02x\n", test)
		return -1
	case 0xff:
		fmt.Println("Selftest: reserved")
		return -1
	default:
		fmt.Println("Selftest: unknown status")
		return -1
	}
}

// ---------------------------------------------------------------------------
// GUID handling
// ---------------------------------------------------------------------------

type guidMode int

const (
	guidRfc4122 guidMode = iota
	guidIpmi
	guidSmbios
	guidAuto
	guidDump
)

func guid(intf ipmicore.Interface, args []string) int {
	var mode guidMode
	if len(args) == 0 {
		mode = guidAuto
	} else {
		switch args[0] {
		case "rfc", "rfc4122":
			mode = guidRfc4122
		case "smbios":
			mode = guidSmbios
		case "ipmi":
			mode = guidIpmi
		case "auto":
			mode = guidAuto
		case "dump":
			mode = guidDump
		case "help":
			mode = guidAuto
		default:
			fmt.Fprintf(os.Stderr, "Invalid GUID mode: %s\n", args[0])
			return -1
		}
	}

	req := ipmicore.NewRequest(ipmicore.NetfnApp, 0x37) // Get System GUID
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintln(os.Stderr, "Get GUID command failed")
		return -1
	}
	if !rsp.OK() {
		fmt.Fprintf(os.Stderr, "Get GUID command failed: %s\n", ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}
	if len(rsp.Data) < 16 {
		fmt.Fprintln(os.Stderr, "Get GUID command failed: short response")
		return -1
	}
	var data [16]byte
	copy(data[:], rsp.Data[:16])

	parsed := parseGuid(data, mode)
	fmt.Printf("System GUID   : %s\n", parsed)

	if mode == guidAuto {
		fmt.Printf("GUID Encoding : %s", parsed.modeName())
		if parsed.mode != guidIpmi {
			fmt.Print(" (WARNING: IPMI Specification violation!)")
		}
		fmt.Println()
	}
	if parsed.mode != guidDump {
		fmt.Printf("GUID Version  : %s", parsed.versionName())
		switch parsed.version {
		case 0:
			fmt.Printf(" (%d)\n", parsed.versionNibble())
		case 1:
			fmt.Println()
			fmt.Printf("Timestamp     : %s\n", formatUnix(parsed.time))
		default:
			fmt.Println()
		}
	}
	return 0
}

// GuidAutoStr renders a 16-byte GUID/UUID the way C's `ipmi_guid2str(...,
// GUID_AUTO)` does, auto-detecting the byte order. Used by the `fru`
// management-access record.
func GuidAutoStr(data []byte) string {
	var raw [16]byte
	n := min(len(data), 16)
	copy(raw[:n], data[:n])
	return parseGuid(raw, guidAuto).String()
}

type parsedGuid struct {
	mode    guidMode
	version uint8
	timeLow uint32
	timeMid uint16
	timeHi  uint16
	clock   uint16
	node    [6]byte
	time    int64
	raw     [16]byte
}

func (g parsedGuid) versionNibble() uint8 {
	return uint8(((uint16(g.raw[8]) | uint16(g.raw[9])<<8) >> 12) & 0xf)
}

func (g parsedGuid) versionName() string {
	switch g.version {
	case 0:
		return "Unknown/unsupported"
	case 1:
		return "Time-based"
	case 2:
		return "DCE Security with POSIX UIDs (not for IPMI)"
	case 3:
		return "Name-based using MD5"
	case 4:
		return "Random or pseudo-random"
	case 5:
		return "Name-based using SHA-1"
	default:
		return "Unknown/unsupported"
	}
}

func (g parsedGuid) modeName() string {
	switch g.mode {
	case guidIpmi:
		return "IPMI"
	case guidRfc4122:
		return "RFC4122"
	case guidSmbios:
		return "SMBIOS"
	case guidDump:
		return "Unknown (data dumped)"
	case guidAuto:
		return "Automatic (if you see this, report a bug)"
	default:
		return ""
	}
}

func (g parsedGuid) String() string {
	if g.mode == guidDump {
		var b strings.Builder
		for _, v := range g.raw {
			fmt.Fprintf(&b, "%02x", v)
		}
		return b.String()
	}
	return fmt.Sprintf("%08x-%04x-%04x-%04x-%02x%02x%02x%02x%02x%02x",
		g.timeLow, g.timeMid, g.timeHi, g.clock,
		g.node[0], g.node[1], g.node[2], g.node[3], g.node[4], g.node[5])
}

func be16(b []byte) uint16 { return uint16(b[0])<<8 | uint16(b[1]) }
func be32(b []byte) uint32 {
	return uint32(b[0])<<24 | uint32(b[1])<<16 | uint32(b[2])<<8 | uint32(b[3])
}
func le16(b []byte) uint16 { return uint16(b[0]) | uint16(b[1])<<8 }
func le32(b []byte) uint32 {
	return uint32(b[0]) | uint32(b[1])<<8 | uint32(b[2])<<16 | uint32(b[3])<<24
}

func parseGuid(data [16]byte, requested guidMode) parsedGuid {
	// Fields for each interpretation.
	// ipmi_guid_t: node[0..6], clock le[6..8], time_hi le[8..10],
	//              time_mid le[10..12], time_low le[12..16]
	// rfc_guid_t:  time_low be[0..4], time_mid be[4..6], time_hi be[6..8],
	//              clock be[8..10], node[10..16]
	modes := [3]guidMode{guidRfc4122, guidIpmi, guidSmbios}

	var chosen guidMode
	haveChosen := false
	if requested != guidAuto && requested != guidDump {
		chosen = requested
		haveChosen = true
	}

	if requested == guidAuto {
		for _, m := range modes {
			tHi, _, _, _ := guidFields(data, m)
			ver := uint8((tHi >> 12) & 0xf)
			if ver >= 1 && ver <= 5 {
				chosen = m
				haveChosen = true
				if ver == 1 {
					seconds := guidTime(data, m)
					if timeValid(seconds) {
						break
					}
				}
			}
		}
	}

	if !haveChosen {
		m := requested
		if requested == guidAuto {
			m = guidDump
		}
		return parsedGuid{
			mode: m,
			raw:  data,
		}
	}

	timeHi, timeMid, timeLow, clock := guidFields(data, chosen)
	version := uint8((timeHi >> 12) & 0xf)
	if version > 5 {
		version = 0
	}
	var node [6]byte
	if chosen == guidIpmi {
		for i := range 6 {
			node[i] = data[5-i]
		}
	} else {
		copy(node[:], data[10:16])
	}
	var t int64
	if version == 1 {
		t = guidTime(data, chosen)
	}

	return parsedGuid{
		mode:    chosen,
		version: version,
		timeLow: timeLow,
		timeMid: timeMid,
		timeHi:  timeHi,
		clock:   clock,
		node:    node,
		time:    t,
		raw:     data,
	}
}

func guidFields(data [16]byte, mode guidMode) (timeHi, timeMid uint16, timeLow uint32, clock uint16) {
	switch mode {
	case guidIpmi:
		return le16(data[8:10]), le16(data[10:12]), le32(data[12:16]), le16(data[6:8])
	case guidRfc4122:
		return be16(data[6:8]), be16(data[4:6]), be32(data[0:4]), be16(data[8:10])
	case guidSmbios:
		return le16(data[6:8]), le16(data[4:6]), le32(data[0:4]), be16(data[8:10])
	default:
		return 0, 0, 0, 0
	}
}

func guidTime(data [16]byte, mode guidMode) int64 {
	timeHi, timeMid, timeLow, _ := guidFields(data, mode)
	hi := uint64(timeHi & 0x0fff) // strip version nibble
	gregorian := (hi << 48) | (uint64(timeMid) << 32) | uint64(timeLow)
	seconds := int64(gregorian / 10_000_000)
	return seconds - 12_219_292_800
}

func timeValid(t int64) bool {
	// Between the Unix epoch and now (with a little slack for clock skew).
	now := time.Now().Unix()
	return t >= 0 && t <= now+86400
}

// formatUnix formats a Unix timestamp as `YYYY-MM-DD HH:MM:SS UTC` without
// pulling in an external date package.
func formatUnix(t int64) string {
	if t < 0 {
		return "Unspecified"
	}
	days := t / 86400
	secs := t % 86400
	year, month, day := civilFromDays(days)
	hour := secs / 3600
	minute := (secs % 3600) / 60
	second := secs % 60
	return fmt.Sprintf("%04d-%02d-%02d %02d:%02d:%02d UTC", year, month, day, hour, minute, second)
}

// civilFromDays is Howard Hinnant's `civil_from_days` algorithm.
func civilFromDays(z int64) (int64, uint32, uint32) {
	z += 719_468
	var era int64
	if z >= 0 {
		era = z / 146_097
	} else {
		era = (z - 146_096) / 146_097
	}
	doe := uint64(z - era*146_097)
	yoe := (doe - doe/1460 + doe/36524 - doe/146_096) / 365
	y := int64(yoe) + era*400
	doy := doe - (365*yoe + yoe/4 - yoe/100)
	mp := (5*doy + 2) / 153
	d := uint32(doy - (153*mp+2)/5 + 1)
	var m uint32
	if mp < 10 {
		m = uint32(mp + 3)
	} else {
		m = uint32(mp - 9)
	}
	if m <= 2 {
		return y + 1, m, d
	}
	return y, m, d
}

func printUsage() {
	fmt.Fprintln(os.Stderr, "MC Commands:\n  info, reset, selftest, getenables, setenables, guid,\n  watchdog, getsysinfo, setsysinfo")
}
