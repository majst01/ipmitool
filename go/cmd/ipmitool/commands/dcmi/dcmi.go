// Package dcmi implements the `dcmi` command: Data Center Management Interface
// operations. It mirrors the Rust `commands/dcmi.rs`, itself a port of the
// `dcmi` command in `lib/ipmi_dcmi.c`.
//
// Currently only `dcmi power reading [<sample_time>]` is implemented; the
// remaining subcommands are stubbed. The request/response layouts follow
// `struct power_reading` in `include/ipmitool/ipmi_dcmi.h`.
package dcmi

import (
	"fmt"
	"os"
	"strings"
	"time"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
)

// Group Extension Identification byte for DCMI requests.
const ipmiDCMI uint8 = 0xDC

// DCMI `Get Power Reading` command.
const ipmiDCMIGetRed uint8 = 0x02

// cmdEntry is a DCMI command-table row. The final row of every table is a
// terminator with an empty name; its value is what str2val2 returns for an
// unknown name, matching the C reference.
type cmdEntry struct {
	val  uint16
	name string
	desc string
}

// dcmiCmdVals is the main DCMI command table (`dcmi_cmd_vals`).
var dcmiCmdVals = []cmdEntry{
	{0x00, "discover", "Used to discover supported DCMI capabilities"},
	{0x01, "power", "Platform power limit command options"},
	{0x02, "sensors", "Prints the available DCMI sensors"},
	{0x03, "asset_tag", "Prints the platform's asset tag"},
	{0x04, "set_asset_tag", "Sets the platform's asset tag"},
	{0x05, "get_mc_id_string", "Get management controller ID string"},
	{0x06, "set_mc_id_string", "Set management controller ID string"},
	{0x07, "thermalpolicy", "Thermal policy get/set"},
	{0x08, "get_temp_reading", "Get Temperature Readings"},
	{0x09, "get_conf_param", "Get DCMI Config Parameters"},
	{0x0A, "set_conf_param", "Set DCMI Config Parameters"},
	{0x0B, "oob_discover", "Ping/Pong Message for DCMI Discovery"},
	{0xFF, "", ""},
}

// dcmiPwrmgmtVals is the power-management command table (`dcmi_pwrmgmt_vals`).
var dcmiPwrmgmtVals = []cmdEntry{
	{0x00, "reading", "Get power related readings from the system"},
	{0x01, "get_limit", "Get the configured power limits"},
	{0x02, "set_limit", "Set a power limit option"},
	{0x03, "activate", "Activate the set power limit"},
	{0x04, "deactivate", "Deactivate the set power limit"},
	{0xFF, "", ""},
}

// dcmiSamplingVals holds the power-reading sample-time selectors
// (`dcmi_sampling_vals`).
var dcmiSamplingVals = []cmdEntry{
	{0x05, "5_sec", ""},
	{0x0f, "15_sec", ""},
	{0x1e, "30_sec", ""},
	{0x41, "1_min", ""},
	{0x43, "3_min", ""},
	{0x47, "7_min", ""},
	{0x4f, "15_min", ""},
	{0x5e, "30_min", ""},
	{0x81, "1_hour", ""},
	{0x00, "", ""},
}

// dcmiCcodeVals holds DCMI-specific completion codes (`dcmi_ccode_vals`).
var dcmiCcodeVals = []cmdEntry{
	{0x80, "Parameter not supported", ""},
	{0x81, "Something else has already claimed these parameters", ""},
	{0x82, "Not supported or failed to write a read-only parameter", ""},
	{0x83, "Access mode is not supported", ""},
	{0x84, "Power/Thermal limit out of range", ""},
	{0x85, "Correction/Exception time out of range", ""},
	{0x89, "Sample/Statistics Reporting period out of range", ""},
	{0x8A, "Power limit already active", ""},
}

// Main is the entry point for `ipmitool dcmi ...`.
func Main(intf ipmicore.Interface, args []string) int {
	if len(args) == 0 {
		printStrs(dcmiCmdVals, "Data Center Management Interface commands", false)
		return -1
	}

	switch str2val2(args[0], dcmiCmdVals) {
	case 0x01:
		// `power` with no subcommand prints the usage and returns early,
		// before the trailing newline `ipmi_dcmi_main` emits after a break.
		if len(args) < 2 {
			printStrs(dcmiPwrmgmtVals, "power <command>", false)
			return -1
		}
		rc := power(intf, args[1], args[2:])
		fmt.Fprintln(os.Stdout) // `ipmi_dcmi_main` prints "\n" after `power`.
		return rc
	case 0x00, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B:
		fmt.Fprintf(os.Stderr, "dcmi %s: not implemented in this build\n", args[0])
		return -1
	default:
		// Unknown command (and `help`): print the command list.
		printStrs(dcmiCmdVals, "Data Center Management Interface commands", false)
		return -1
	}
}

// power implements `dcmi power <sub> [args...]` (`ipmi_dcmi_parse_power`).
func power(intf ipmicore.Interface, sub string, rest []string) int {
	switch str2val2(sub, dcmiPwrmgmtVals) {
	case 0x00:
		var sampleTime uint8
		if len(rest) > 0 {
			v := str2val2(rest[0], dcmiSamplingVals)
			if v == 0 {
				printStrs(dcmiSamplingVals, "Invalid sample time. Valid times are: ", true)
				fmt.Fprintln(os.Stdout)
				return -1
			}
			sampleTime = uint8(v)
		}
		return pwrRd(intf, sampleTime)
	case 0x01, 0x02, 0x03, 0x04:
		fmt.Fprintf(os.Stderr, "dcmi power %s: not implemented in this build\n", sub)
		return -1
	default:
		// `ipmi_dcmi_parse_power` leaves rc at 0 for the default case.
		printStrs(dcmiPwrmgmtVals, "power <command>", false)
		return 0
	}
}

// pwrRd implements `ipmi_dcmi_pwr_rd`: fetch and print the DCMI power reading.
func pwrRd(intf ipmicore.Interface, sampleTime uint8) int {
	msgData := make([]byte, 4)
	msgData[0] = ipmiDCMI
	if sampleTime != 0 {
		msgData[1] = 0x02 // Enhanced Power Statistics
		msgData[2] = sampleTime
	} else {
		msgData[1] = 0x01 // Mode Power Status
		msgData[2] = 0x00 // reserved
	}
	msgData[3] = 0x00 // reserved

	req := ipmicore.NewRequest(ipmicore.NetfnDcgrp, ipmiDCMIGetRed)
	req.Data = msgData

	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintln(os.Stderr, "\n    Unable to get DCMI information")
		return -1
	}
	if chkRsp(rsp) {
		return -1
	}

	d := rsp.Data
	currPwr := u16le(d, 1)
	minSample := u16le(d, 3)
	maxSample := u16le(d, 5)
	avgPwr := u16le(d, 7)
	timeStamp := u32le(d, 9)
	sample := u32le(d, 13)
	state := byteAt(d, 17)

	fmt.Fprintln(os.Stdout)
	fmt.Fprintf(os.Stdout, "    Instantaneous power reading:              %8d Watts\n", currPwr)
	fmt.Fprintf(os.Stdout, "    Minimum during sampling period:           %8d Watts\n", minSample)
	fmt.Fprintf(os.Stdout, "    Maximum during sampling period:           %8d Watts\n", maxSample)
	fmt.Fprintf(os.Stdout, "    Average power reading over sample period: %8d Watts\n", avgPwr)
	fmt.Fprintf(os.Stdout, "    IPMI timestamp:                           %s", timestampNumeric(timeStamp))
	fmt.Fprint(os.Stdout, "    Sampling period:                          ")
	if sampleTime != 0 {
		fmt.Fprintf(os.Stdout, "%s \n", val2str2(uint16(sample&0xffff), dcmiSamplingVals))
	} else {
		fmt.Fprintf(os.Stdout, "%08d Seconds.\n", sample/1000)
	}
	fmt.Fprint(os.Stdout, "    Power reading state is:                   ")
	if state&0x40 == 0x40 {
		fmt.Fprint(os.Stdout, "activated")
	} else {
		fmt.Fprint(os.Stdout, "deactivated")
	}
	fmt.Fprintln(os.Stdout)
	fmt.Fprintln(os.Stdout)
	return 0
}

// chkRsp mirrors `chk_rsp`: returns true when the response aborts the command.
func chkRsp(rsp *ipmicore.Response) bool {
	if rsp.Ccode >= 0x80 && rsp.Ccode <= 0x8F {
		fmt.Fprintf(os.Stderr, "\n    DCMI request failed because: %s (%x)\n",
			dcmiCcodeStr(rsp.Ccode), rsp.Ccode)
		return true
	} else if rsp.Ccode != 0 {
		fmt.Fprintf(os.Stderr, "\n    DCMI request failed because: %s (%x)\n",
			ipmicore.CompletionCodeStr(rsp.Ccode), rsp.Ccode)
		return true
	}
	if len(rsp.Data) == 0 || rsp.Data[0] != ipmiDCMI {
		// The C prints this to stdout without a trailing newline.
		fmt.Fprintf(os.Stdout, "\n    A valid DCMI command was not returned! (%x)", byteAt(rsp.Data, 0))
		return true
	}
	return false
}

// ---------------------------------------------------------------------------
// Helpers mirroring the C `str2val2` / `val2str2` / `print_strs`.
// ---------------------------------------------------------------------------

// str2val2 is a case-insensitive name lookup; on a miss it returns the
// terminator value, matching `str2val2`.
func str2val2(name string, table []cmdEntry) uint16 {
	for _, e := range table {
		if e.name == "" {
			return e.val // terminator
		}
		if strings.EqualFold(e.name, name) {
			return e.val
		}
	}
	return 0
}

// val2str2 looks up a value; unknown values render like `val2str2` (lowercase
// hex).
func val2str2(val uint16, table []cmdEntry) string {
	for _, e := range table {
		if e.name == "" {
			break
		}
		if e.val == val {
			return e.name
		}
	}
	return fmt.Sprintf("Unknown (0x%x)", val)
}

// dcmiCcodeStr renders a DCMI completion code (`val2str` over
// `dcmi_ccode_vals`).
func dcmiCcodeStr(code uint8) string {
	for _, e := range dcmiCcodeVals {
		if uint16(code) == e.val {
			return e.name
		}
	}
	return fmt.Sprintf("Unknown (0x%02X)", code)
}

// printStrs mirrors the C `print_strs` with a non-negative log level (stderr).
func printStrs(table []cmdEntry, title string, verthorz bool) {
	fmt.Fprintf(os.Stderr, "\n%s\n", title)
	last := len(table) - 1 // terminator index
	for i, e := range table {
		if i == last {
			break
		}
		fmt.Fprintf(os.Stderr, "    %s    %s\n", e.name, e.desc)
		if verthorz && i+1 < last {
			fmt.Fprint(os.Stdout, " | ")
		}
	}
	if !verthorz {
		fmt.Fprintln(os.Stderr)
	}
}

func byteAt(data []byte, i int) uint8 {
	if i < len(data) {
		return data[i]
	}
	return 0
}

func u16le(data []byte, i int) uint16 {
	return uint16(byteAt(data, i)) | uint16(byteAt(data, i+1))<<8
}

func u32le(data []byte, i int) uint32 {
	return uint32(byteAt(data, i)) |
		uint32(byteAt(data, i+1))<<8 |
		uint32(byteAt(data, i+2))<<16 |
		uint32(byteAt(data, i+3))<<24
}

// timestampNumeric formats an IPMI timestamp the way `ipmi_timestamp_numeric()`
// does: an absolute epoch timestamp uses `%x %X %Z` in local time; a "special"
// value (< 0x20000000) is seconds since BMC power-on, rendered in UTC with an
// `S+` prefix.
func timestampNumeric(ts uint32) string {
	if ts == 0xFFFFFFFF {
		return "Unspecified"
	}
	if ts < 0x20000000 {
		t := time.Unix(int64(ts), 0).UTC()
		if ts < 86400 {
			return t.Format("S+ 15:04:05")
		}
		return t.Format("S+ 06/002 15:04:05")
	}
	return time.Unix(int64(ts), 0).Format("01/02/06 15:04:05 MST")
}
