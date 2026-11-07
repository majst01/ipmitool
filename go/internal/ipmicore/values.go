package ipmicore

import (
	"fmt"
	"os"
	"strings"
)

// ValStr is a name/description pair used when printing help and status.
type ValStr struct {
	Val  uint8
	Name string
}

var privlvlVals = []ValStr{
	{SessionPrivCallback, "CALLBACK"},
	{SessionPrivUser, "USER"},
	{SessionPrivOperator, "OPERATOR"},
	{SessionPrivAdmin, "ADMINISTRATOR"},
	{SessionPrivOem, "OEM"},
	{SessionPrivNoAccess, "NO ACCESS"},
}

var authTypeVals = []ValStr{
	{0x00, "NONE"},
	{0x01, "MD2"},
	{0x02, "MD5"},
	{0x04, "PASSWORD"},
	{0x05, "OEM"},
}

var netfnVals = []ValStr{
	{NetfnChassis, "Chassis"},
	{NetfnBridge, "Bridge"},
	{NetfnSE, "SensorEvent"},
	{NetfnApp, "Application"},
	{NetfnFirmware, "Firmware"},
	{NetfnStorage, "Storage"},
	{NetfnTransport, "Transport"},
	{NetfnPicmg, "PICMG"},
	{NetfnOem, "OEM"},
	{NetfnIsol, "ISOL"},
	{NetfnTSol, "TSOL"},
}

var powerControlVals = []ValStr{
	{0x0, "Down/Off"},
	{0x1, "Up/On"},
	{0x2, "Cycle"},
	{0x3, "Reset"},
	{0x4, "Diag"},
	{0x5, "Soft"},
}

var chassisRestartCauseVals = []ValStr{
	{0x0, "unknown"},
	{0x1, "chassis power control command"},
	{0x2, "reset via pushbutton"},
	{0x3, "power-up via pushbutton"},
	{0x4, "watchdog expired"},
	{0x5, "OEM"},
	{0x6, "power-up due to always-restore power policy"},
	{0x7, "power-up due to restore-previous power policy"},
	{0x8, "reset via PEF"},
	{0x9, "power-cycle via PEF"},
	{0xa, "soft reset"},
	{0xb, "power-up via RTC wakeup"},
}

func lookup(vals []ValStr, s string, caseSensitive bool) (uint8, bool) {
	for _, v := range vals {
		var match bool
		if caseSensitive {
			match = v.Name == s
		} else {
			match = strings.EqualFold(v.Name, s)
		}
		if match {
			return v.Val, true
		}
	}
	return 0, false
}

func nameOf(vals []ValStr, val uint8) string {
	for _, v := range vals {
		if v.Val == val {
			return v.Name
		}
	}
	return fmt.Sprintf("Unknown (0x%02X)", val)
}

// PrivlvlFromStr parses a privilege level name (case-insensitive).
func PrivlvlFromStr(s string) (uint8, bool) { return lookup(privlvlVals, s, false) }

// PrivlvlStr returns the printable name for a privilege level.
func PrivlvlStr(val uint8) string { return nameOf(privlvlVals, val) }

// AuthtypeFromStr parses an authentication type name (case-insensitive).
func AuthtypeFromStr(s string) (uint8, bool) { return lookup(authTypeVals, s, false) }

// NetfnFromStr parses a network function name (case-insensitive).
func NetfnFromStr(s string) (uint8, bool) { return lookup(netfnVals, s, false) }

// PowerControlStr returns the printable name for a power control action.
func PowerControlStr(val uint8) string { return nameOf(powerControlVals, val) }

// ChassisRestartCauseStr returns the printable name for a restart cause.
func ChassisRestartCauseStr(val uint8) string { return nameOf(chassisRestartCauseVals, val) }

// PrintValstr prints a value table to stderr, mirroring `print_valstr`.
func PrintValstr(vals []ValStr, title string) {
	if title != "" {
		fmt.Fprintln(os.Stderr, title)
	}
	for _, v := range vals {
		fmt.Fprintf(os.Stderr, "\t%-16d  %s\n", v.Val, v.Name)
	}
}
