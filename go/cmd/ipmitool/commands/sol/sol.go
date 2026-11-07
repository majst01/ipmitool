// Package sol implements the Serial-over-LAN commands. It mirrors the Rust
// `commands/sol.rs`, itself a port of `lib/ipmi_sol.c` and the SOL parts of
// `src/plugins/lanplus/lanplus.c`.
package sol

import (
	"fmt"
	"os"
	"strings"
	"time"

	"codeberg.org/ipmitool/ipmitool/internal/args"
	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
)

// defaultChannel is the default IPMI LAN channel.
const defaultChannel uint8 = 0x0e

// keepaliveTimeout is the number of seconds between SOL keepalive probes.
const keepaliveTimeout uint64 = 15

// maxKeepaliveRetry is the number of consecutive keepalive failures tolerated
// before exiting.
const maxKeepaliveRetry uint32 = 6

// Main is the entry point for the `sol` command.
func Main(intf ipmicore.Interface, argv []string) int {
	if len(argv) == 0 {
		printUsage()
		return -1
	}
	cmd := argv[0]
	rest := argv[1:]
	switch cmd {
	case "help":
		printUsage()
		return 0
	case "info":
		return info(intf, rest)
	case "payload":
		return payload(intf, rest)
	case "set":
		return set(intf, rest)
	case "activate":
		return activate(intf, rest)
	case "deactivate":
		return deactivate(intf, rest)
	case "looptest":
		fmt.Fprintln(os.Stderr, "Error: sol looptest is not implemented")
		return -1
	default:
		fmt.Fprintf(os.Stderr, "Invalid sol command: %s\n", cmd)
		printUsage()
		return -1
	}
}

func printUsage() {
	fmt.Fprintln(os.Stderr, "SOL Commands: info [<channel number>]")
	fmt.Fprintln(os.Stderr, "              set <parameter> <value> [channel]")
	fmt.Fprintln(os.Stderr, "              payload <enable|disable|status> [channel] [userid]")
	fmt.Fprintln(os.Stderr, "              activate [<usesolkeepalive|nokeepalive>] [instance=<number>]")
	fmt.Fprintln(os.Stderr, "              deactivate [instance=<number>]")
}

// parseChannel parses the optional channel argument.
func parseChannel(argv []string, index int) (uint8, int) {
	if index >= len(argv) {
		return defaultChannel, 0
	}
	s := argv[index]
	if c, ok := args.ParseU8(s); ok {
		return c, 0
	}
	fmt.Fprintf(os.Stderr, "Invalid channel number: %s\n", s)
	return 0, -1
}

// ---------------------------------------------------------------------------
// sol info
// ---------------------------------------------------------------------------

// getParam reads a single SOL configuration parameter. The bool result is
// false when the BMC reports it as unsupported (completion code 0x80); a
// non-zero code result signals an error.
func getParam(intf ipmicore.Interface, channel, selector uint8) ([]byte, bool, int) {
	req := ipmicore.NewRequest(ipmicore.NetfnTransport, ipmicore.IpmiGetSolConfigParameters)
	req.Data = []byte{channel, selector, 0x00, 0x00}
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: No response requesting SOL parameter '%s'\n", ipmicore.ParameterName(selector))
		return nil, false, -1
	}
	switch rsp.Ccode {
	case 0x00:
		return rsp.Data, true, 0
	case 0x80:
		fmt.Fprintf(os.Stderr, "Info: SOL parameter '%s' not supported\n", ipmicore.ParameterName(selector))
		return nil, false, 0
	default:
		fmt.Fprintf(os.Stderr, "Error requesting SOL parameter '%s': %s\n",
			ipmicore.ParameterName(selector), ipmicore.CompletionCodeStr(rsp.Ccode))
		return nil, false, -1
	}
}

// getSolInfo fetches the full SOL configuration, mirroring
// `ipmi_get_sol_info`.
func getSolInfo(intf ipmicore.Interface, channel uint8) (ipmicore.SolConfigParams, int) {
	var p ipmicore.SolConfigParams

	if d, ok, code := getParam(intf, channel, ipmicore.ParamSetInProgress); code != 0 {
		return p, code
	} else if ok && len(d) >= 2 {
		p.SetInProgress = d[1]
	}
	if d, ok, code := getParam(intf, channel, ipmicore.ParamSolEnable); code != 0 {
		return p, code
	} else if ok && len(d) >= 2 {
		p.Enabled = d[1]&0x01 != 0
	}
	if d, ok, code := getParam(intf, channel, ipmicore.ParamSolAuthentication); code != 0 {
		return p, code
	} else if ok && len(d) >= 2 {
		p.ForceEncryption = d[1]&0x80 != 0
		p.ForceAuthentication = d[1]&0x40 != 0
		p.PrivilegeLevel = d[1] & 0x0f
	}
	if d, ok, code := getParam(intf, channel, ipmicore.ParamCharacterInterval); code != 0 {
		return p, code
	} else if ok && len(d) >= 3 {
		p.CharacterAccumulateLevel = d[1]
		p.CharacterSendThreshold = d[2]
	}
	if d, ok, code := getParam(intf, channel, ipmicore.ParamSolRetry); code != 0 {
		return p, code
	} else if ok && len(d) >= 3 {
		p.RetryCount = d[1]
		p.RetryInterval = d[2]
	}
	if d, ok, code := getParam(intf, channel, ipmicore.ParamNonVolatileBitRate); code != 0 {
		return p, code
	} else if ok && len(d) >= 2 {
		p.NonVolatileBitRate = d[1] & 0x0f
	}
	if d, ok, code := getParam(intf, channel, ipmicore.ParamVolatileBitRate); code != 0 {
		return p, code
	} else if ok && len(d) >= 2 {
		p.VolatileBitRate = d[1] & 0x0f
	}
	if d, ok, code := getParam(intf, channel, ipmicore.ParamPayloadChannel); code != 0 {
		return p, code
	} else if ok && len(d) >= 2 {
		p.PayloadChannel = d[1]
	} else {
		p.PayloadChannel = channel
	}
	if d, ok, code := getParam(intf, channel, ipmicore.ParamPayloadPort); code != 0 {
		return p, code
	} else if ok && len(d) >= 3 {
		p.PayloadPort = uint16(d[1]) | uint16(d[2])<<8
	} else {
		p.PayloadPort = intf.SessionParams().Port
	}
	return p, 0
}

func info(intf ipmicore.Interface, argv []string) int {
	if len(argv) > 1 {
		printUsage()
		return -1
	}
	channel, code := parseChannel(argv, 0)
	if code != 0 {
		return code
	}
	params, code := getSolInfo(intf, channel)
	if code != 0 {
		return code
	}
	fmt.Printf("Set in progress                 : %s\n", ipmicore.SetInProgressStr(params.SetInProgress))
	fmt.Printf("Enabled                         : %s\n", boolStr(params.Enabled))
	fmt.Printf("Force Encryption                : %s\n", boolStr(params.ForceEncryption))
	fmt.Printf("Force Authentication            : %s\n", boolStr(params.ForceAuthentication))
	fmt.Printf("Privilege Level                 : %s\n", ipmicore.PrivlvlStr(params.PrivilegeLevel))
	fmt.Printf("Character Accumulate Level (ms) : %d\n", uint16(params.CharacterAccumulateLevel)*5)
	fmt.Printf("Character Send Threshold        : %d\n", params.CharacterSendThreshold)
	fmt.Printf("Retry Count                     : %d\n", params.RetryCount)
	fmt.Printf("Retry Interval (ms)             : %d\n", uint16(params.RetryInterval)*10)
	fmt.Printf("Volatile Bit Rate (kbps)        : %s\n", ipmicore.BitRateStr(params.VolatileBitRate))
	fmt.Printf("Non-Volatile Bit Rate (kbps)    : %s\n", ipmicore.BitRateStr(params.NonVolatileBitRate))
	fmt.Printf("Payload Channel                 : %d (0x%02x)\n", params.PayloadChannel, params.PayloadChannel)
	fmt.Printf("Payload Port                    : %d\n", params.PayloadPort)
	return 0
}

func boolStr(b bool) string {
	if b {
		return "true"
	}
	return "false"
}

// ---------------------------------------------------------------------------
// sol payload
// ---------------------------------------------------------------------------

func payload(intf ipmicore.Interface, argv []string) int {
	if len(argv) == 0 || len(argv) > 3 {
		printUsage()
		return -1
	}
	channel, code := parseChannel(argv, 1)
	if code != 0 {
		return code
	}
	userid := uint8(1)
	if len(argv) > 2 {
		s := argv[2]
		id, ok := args.ParseU8(s)
		if !ok {
			fmt.Fprintf(os.Stderr, "Invalid user id: %s\n", s)
			return -1
		}
		userid = id
	}
	userid &= 0x3f

	switch argv[0] {
	case "enable", "disable":
		enable := argv[0] == "enable"
		req := ipmicore.NewRequest(ipmicore.NetfnApp, ipmicore.IpmiSetUserPayloadAccess)
		var access = (userid & 0x3f)
		if !enable {
			access |= 0x40
		}
		req.Data = []byte{channel & 0x0f, access, 0x02, 0, 0, 0}
		rsp, err := intf.SendRecv(req)
		if err == nil && rsp.Ccode == 0 {
			return 0
		}
		verb := "en"
		if !enable {
			verb = "dis"
		}
		if err == nil {
			fmt.Fprintf(os.Stderr, "Error %sabling SOL payload for user %d on channel %d: %s\n",
				verb, userid, channel, ipmicore.CompletionCodeStr(rsp.Ccode))
		} else {
			fmt.Fprintf(os.Stderr, "Error %sabling SOL payload for user %d on channel %d\n",
				verb, userid, channel)
		}
		return -1
	case "status":
		req := ipmicore.NewRequest(ipmicore.NetfnApp, ipmicore.IpmiGetUserPayloadAccess)
		req.Data = []byte{channel & 0x0f, userid & 0x3f}
		rsp, err := intf.SendRecv(req)
		if err != nil {
			fmt.Fprintln(os.Stderr, "Error. No valid response received.")
			return -1
		}
		if rsp.Ccode != 0 {
			fmt.Fprintf(os.Stderr, "Error getting SOL payload status for user %d on channel %d: %s\n",
				userid, channel, ipmicore.CompletionCodeStr(rsp.Ccode))
			return -1
		}
		if len(rsp.Data) != 4 {
			fmt.Fprintf(os.Stderr, "Error parsing SOL payload status for user %d on channel %d\n", userid, channel)
			return -1
		}
		state := "dis"
		if rsp.Data[0]&0x02 != 0 {
			state = "en"
		}
		fmt.Printf("User %d on channel %d is %sabled\n", userid, channel, state)
		return 0
	default:
		fmt.Fprintf(os.Stderr, "Invalid payload action: %s\n", argv[0])
		printUsage()
		return -1
	}
}

// ---------------------------------------------------------------------------
// sol set
// ---------------------------------------------------------------------------

// sendSet issues a single `Set SOL Configuration Parameters` request,
// returning the completion code and a non-zero error code on transport
// failure.
func sendSet(intf ipmicore.Interface, channel, selector uint8, values []byte) (uint8, int) {
	req := ipmicore.NewRequest(ipmicore.NetfnTransport, ipmicore.IpmiSetSolConfigParameters)
	data := []byte{channel, selector}
	data = append(data, values...)
	req.Data = data
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error setting SOL parameter '%s'\n", ipmicore.ParameterName(selector))
		return 0, -1
	}
	return rsp.Ccode, 0
}

func setInProgress(intf ipmicore.Interface, channel, value uint8) (uint8, int) {
	return sendSet(intf, channel, ipmicore.ParamSetInProgress, []byte{value})
}

// parseRange parses a value in min..=max, printing the same diagnostics as C.
func parseRange(value, param string, min, max uint8) (uint8, int) {
	v, ok := args.ParseU8(value)
	if ok && v >= min && v <= max {
		return v, 0
	}
	fmt.Fprintf(os.Stderr, "Invalid value %s for parameter %s\n", value, param)
	fmt.Fprintf(os.Stderr, "Valid values are %d-%d\n", min, max)
	return 0, -1
}

func parseBitRate(value string) (uint8, int) {
	switch value {
	case "serial":
		return 0x00, 0
	case "9.6":
		return 0x06, 0
	case "19.2":
		return 0x07, 0
	case "38.4":
		return 0x08, 0
	case "57.6":
		return 0x09, 0
	case "115.2":
		return 0x0a, 0
	default:
		fmt.Fprintf(os.Stderr, "Invalid value \"%s\" for parameter \"bit-rate\"\n", value)
		fmt.Fprintln(os.Stderr, "Valid values are serial, 9.6 19.2, 38.4, 57.6 and 115.2")
		return 0, -1
	}
}

func set(intf ipmicore.Interface, argv []string) int {
	// Usage: set <param> <value> [channel] [noguard]
	if len(argv) < 2 || len(argv) > 4 {
		printSetUsage()
		return -1
	}
	param := argv[0]
	value := argv[1]

	guard := true
	channel := defaultChannel
	for _, extra := range argv[2:] {
		if extra == "noguard" {
			guard = false
		} else if c, ok := args.ParseU8(extra); ok {
			channel = c
		} else {
			fmt.Fprintf(os.Stderr, "Invalid channel number: %s\n", extra)
			return -1
		}
	}

	// Per-parameter preparation: selector plus the value bytes.
	var selector uint8
	var values []byte // values exclude the selector byte (C's data[2..])
	switch param {
	case "set-in-progress":
		guard = false
		var code uint8
		switch value {
		case "set-complete":
			code = 0x00
		case "set-in-progress":
			code = 0x01
		case "commit-write":
			code = 0x02
		default:
			fmt.Fprintf(os.Stderr, "Invalid value %s for parameter %s\n", value, param)
			fmt.Fprintln(os.Stderr, "Valid values are set-complete, set-in-progress and commit-write")
			return -1
		}
		selector = ipmicore.ParamSetInProgress
		values = []byte{code}
	case "enabled":
		var code uint8
		switch value {
		case "true":
			code = 0x01
		case "false":
			code = 0x00
		default:
			fmt.Fprintf(os.Stderr, "Invalid value %s for parameter %s\n", value, param)
			fmt.Fprintln(os.Stderr, "Valid values are true and false")
			return -1
		}
		selector = ipmicore.ParamSolEnable
		values = []byte{code}
	case "force-encryption", "force-authentication", "privilege-level":
		current, code := getSolInfo(intf, channel)
		if code != 0 {
			return code
		}
		byteVal := current.PrivilegeLevel
		if current.ForceEncryption {
			byteVal |= 0x80
		}
		if current.ForceAuthentication {
			byteVal |= 0x40
		}
		switch param {
		case "force-encryption":
			switch value {
			case "true":
				byteVal |= 0x80
			case "false":
				byteVal &^= 0x80
			default:
				return invalidBool(param, value)
			}
		case "force-authentication":
			switch value {
			case "true":
				byteVal |= 0x40
			case "false":
				byteVal &^= 0x40
			default:
				return invalidBool(param, value)
			}
		default:
			var lvl uint8
			switch value {
			case "user":
				lvl = 0x02
			case "operator":
				lvl = 0x03
			case "admin":
				lvl = 0x04
			case "oem":
				lvl = 0x05
			default:
				fmt.Fprintf(os.Stderr, "Invalid value %s for parameter %s\n", value, param)
				fmt.Fprintln(os.Stderr, "Valid values are user, operator, admin, and oem")
				return -1
			}
			byteVal = (byteVal &^ 0x0f) | lvl
		}
		selector = ipmicore.ParamSolAuthentication
		values = []byte{byteVal}
	case "character-accumulate-level":
		level, code := parseRange(value, param, 1, 255)
		if code != 0 {
			return code
		}
		current, code := getSolInfo(intf, channel)
		if code != 0 {
			return code
		}
		selector = ipmicore.ParamCharacterInterval
		values = []byte{level, current.CharacterSendThreshold}
	case "character-send-threshold":
		threshold, code := parseRange(value, param, 0, 255)
		if code != 0 {
			return code
		}
		current, code := getSolInfo(intf, channel)
		if code != 0 {
			return code
		}
		selector = ipmicore.ParamCharacterInterval
		values = []byte{current.CharacterAccumulateLevel, threshold}
	case "retry-count":
		count, code := parseRange(value, param, 0, 7)
		if code != 0 {
			return code
		}
		current, code := getSolInfo(intf, channel)
		if code != 0 {
			return code
		}
		selector = ipmicore.ParamSolRetry
		values = []byte{count, current.RetryInterval}
	case "retry-interval":
		interval, code := parseRange(value, param, 0, 255)
		if code != 0 {
			return code
		}
		current, code := getSolInfo(intf, channel)
		if code != 0 {
			return code
		}
		selector = ipmicore.ParamSolRetry
		values = []byte{current.RetryCount, interval}
	case "non-volatile-bit-rate":
		rate, code := parseBitRate(value)
		if code != 0 {
			return code
		}
		selector = ipmicore.ParamNonVolatileBitRate
		values = []byte{rate}
	case "volatile-bit-rate":
		rate, code := parseBitRate(value)
		if code != 0 {
			return code
		}
		selector = ipmicore.ParamVolatileBitRate
		values = []byte{rate}
	default:
		fmt.Fprintf(os.Stderr, "Error: invalid SOL parameter %s\n", param)
		return -1
	}

	// Guarded write sequence, matching ipmi_sol_set_param.
	if guard {
		if _, code := setInProgress(intf, channel, 0x01); code != 0 {
			return code
		}
	}

	ccode, code := sendSet(intf, channel, selector, values)
	if code != 0 {
		return code
	}

	isCommitWrite := param == "set-in-progress" && value == "commit-write"
	if !isCommitWrite && ccode != 0 {
		switch ccode {
		case 0x80:
			fmt.Fprintf(os.Stderr, "Error setting SOL parameter '%s': Parameter not supported\n", param)
		case 0x81:
			fmt.Fprintf(os.Stderr, "Error setting SOL parameter '%s': Attempt to set set-in-progress when not in set-complete state\n", param)
		case 0x82:
			fmt.Fprintf(os.Stderr, "Error setting SOL parameter '%s': Attempt to write read-only parameter\n", param)
		case 0x83:
			fmt.Fprintf(os.Stderr, "Error setting SOL parameter '%s': Attempt to write write-only parameter\n", param)
		default:
			fmt.Fprintf(os.Stderr, "Error setting SOL parameter '%s' to '%s': %s\n",
				param, value, ipmicore.CompletionCodeStr(ccode))
		}
		if guard {
			setInProgress(intf, channel, 0x00)
		}
		return -1
	}

	if guard {
		setInProgress(intf, channel, 0x02) // commit-write, may be unimplemented
		if _, code := setInProgress(intf, channel, 0x00); code != 0 {
			fmt.Fprintln(os.Stderr, "Error could not set \"set-in-progress\" to \"set-complete\"")
			return -1
		}
	}
	return 0
}

func invalidBool(param, value string) int {
	fmt.Fprintf(os.Stderr, "Invalid value %s for parameter %s\n", value, param)
	fmt.Fprintln(os.Stderr, "Valid values are true and false")
	return -1
}

func printSetUsage() {
	fmt.Fprintln(os.Stderr, "SOL set parameters and values:")
	fmt.Fprintln(os.Stderr, "  set-in-progress             set-complete | set-in-progress | commit-write")
	fmt.Fprintln(os.Stderr, "  enabled                     true | false")
	fmt.Fprintln(os.Stderr, "  force-encryption            true | false")
	fmt.Fprintln(os.Stderr, "  force-authentication        true | false")
	fmt.Fprintln(os.Stderr, "  privilege-level             user | operator | admin | oem")
	fmt.Fprintln(os.Stderr, "  character-accumulate-level  <in 5 ms increments>")
	fmt.Fprintln(os.Stderr, "  character-send-threshold    N")
	fmt.Fprintln(os.Stderr, "  retry-count                 N")
	fmt.Fprintln(os.Stderr, "  retry-interval              <in 10 ms increments>")
	fmt.Fprintln(os.Stderr, "  non-volatile-bit-rate       serial | 9.6 | 19.2 | 38.4 | 57.6 | 115.2")
	fmt.Fprintln(os.Stderr, "  volatile-bit-rate           serial | 9.6 | 19.2 | 38.4 | 57.6 | 115.2")
}

// ---------------------------------------------------------------------------
// sol activate / deactivate
// ---------------------------------------------------------------------------

func deactivate(intf ipmicore.Interface, argv []string) int {
	instance := uint8(1)
	for _, arg := range argv {
		v, ok := strings.CutPrefix(arg, "instance=")
		if !ok {
			printUsage()
			return -1
		}
		i, ok := args.ParseU8(v)
		if !ok || i < 1 || i > 15 {
			fmt.Fprintf(os.Stderr, "Given instance '%s' is invalid.\n", v)
			printUsage()
			return -1
		}
		instance = i
	}
	if err := intf.SolDeactivate(instance); err != nil {
		fmt.Fprintf(os.Stderr, "Error de-activating SOL payload: %v\n", err)
		return -1
	}
	return 0
}

func activate(intf ipmicore.Interface, argv []string) int {
	instance := uint8(1)
	disableKeepalive := false
	useSolKeepalive := false
	for _, arg := range argv {
		switch arg {
		case "usesolkeepalive":
			useSolKeepalive = true
		case "nokeepalive":
			disableKeepalive = true
		default:
			v, ok := strings.CutPrefix(arg, "instance=")
			if !ok {
				printUsage()
				return -1
			}
			i, ok := args.ParseU8(v)
			if !ok || i < 1 || i > 15 {
				fmt.Fprintf(os.Stderr, "Given instance '%s' is invalid.\n", v)
				printUsage()
				return -1
			}
			instance = i
		}
	}

	req := &ipmicore.SolActivateRequest{
		Instance:       instance,
		Encryption:     true,
		Authentication: true,
		AssertCTS:      false,
	}
	if _, err := intf.SolActivate(req); err != nil {
		fmt.Fprintf(os.Stderr, "Error activating SOL payload: %v\n", err)
		return -1
	}

	escapeChar := intf.SessionParams().SolEscapeChar
	fmt.Printf("[SOL Session operational.  Use %c? for help]\n", escapeChar)

	code := interactive(intf, disableKeepalive, useSolKeepalive)

	// Best-effort cleanup, as in ipmi_sol_red_pill.
	intf.SolDeactivate(instance)
	return code
}

// ---------------------------------------------------------------------------
// Interactive session
// ---------------------------------------------------------------------------

// processInput processes user input, handling escape sequences. It returns a
// result code (1 terminates the session) and false when the send failed, which
// the caller treats as the BMC having closed the session.
func processInput(intf ipmicore.Interface, input []byte, escapePending, lastWasCR *bool, escapeChar uint8) (uint8, bool) {
	out := make([]byte, 0, len(input))
	result := uint8(0)

	for _, ch := range input {
		if *escapePending {
			*escapePending = false
			switch ch {
			case '.':
				fmt.Printf("%c. [terminated ipmitool]\n", escapeChar)
				result = 1
			case 0x1a, 0x18:
				// ^Z / ^X: suspend. Real suspension needs SIGTSTP, which we
				// do not attempt here; report and continue.
				fmt.Printf("%c^Z [suspend ipmitool]\n", escapeChar)
			case 'B':
				fmt.Printf("%cB [send break]\n", escapeChar)
				intf.SolSend(ipmicore.SolGenerateBreak())
			case '?':
				printEscapeSequences(escapeChar)
			default:
				if ch != escapeChar {
					out = append(out, escapeChar)
				}
				out = append(out, ch)
			}
		} else if *lastWasCR && ch == escapeChar {
			*escapePending = true
		} else {
			out = append(out, ch)
		}
		*lastWasCR = ch == '\r' || ch == '\n'
	}

	if len(out) != 0 {
		pkt := ipmicore.SolData(out)
		if _, err := intf.SolSend(pkt); err != nil {
			fmt.Fprintf(os.Stderr, "Error sending SOL data: %v\n", err)
			return 0, false
		}
	}
	return result, true
}

func printEscapeSequences(escapeChar uint8) {
	e := escapeChar
	fmt.Printf("%c?\n", e)
	fmt.Println("\tSupported escape sequences:")
	fmt.Printf("\t%c.  - terminate connection\n", e)
	fmt.Printf("\t%c^Z - suspend ipmitool\n", e)
	fmt.Printf("\t%c^X - suspend ipmitool, but don't restore tty on restart\n", e)
	fmt.Printf("\t%cB  - send break\n", e)
	fmt.Printf("\t%c?  - this message\n", e)
	fmt.Printf("\t%c%c  - send the escape character by typing it twice\n", e, e)
	fmt.Println("\t(Note that escapes are only recognized immediately after newline.)")
}

func interactive(intf ipmicore.Interface, disableKeepalive, useSolKeepalive bool) int {
	term, err := enterRawTerminal()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: Unable to enter raw terminal mode: %v\n", err)
		return -1
	}
	defer term.restore()

	escapeChar := intf.SessionParams().SolEscapeChar
	fd, ok := intf.RawFd()
	if !ok {
		fd = -1
	}
	escapePending := false
	lastWasCR := true
	shouldExit := false
	bmcClosed := false
	keepaliveStart := time.Now()
	keepaliveFailures := uint32(0)

	for !shouldExit {
		if !disableKeepalive && uint64(time.Since(keepaliveStart).Seconds()) > keepaliveTimeout {
			ok := false
			if useSolKeepalive {
				_, err := intf.SolSend(ipmicore.SolData(nil))
				ok = err == nil
			} else {
				req := ipmicore.NewRequest(ipmicore.NetfnApp, ipmicore.IpmiGetDeviceID)
				_, err := intf.SendRecv(req)
				ok = err == nil
			}
			if ok {
				keepaliveStart = time.Now()
				keepaliveFailures = 0
			} else {
				keepaliveFailures++
				if keepaliveFailures >= maxKeepaliveRetry {
					fmt.Fprintln(os.Stderr, "Error: No response to keepalive - Terminating session")
					shouldExit = true
					continue
				}
			}
		}

		fds := []pollFd{
			{fd: int32(os.Stdin.Fd()), events: pollIn},
			{fd: int32(fd), events: pollIn},
		}
		n, err := poll(fds, 500)
		if err != nil {
			fmt.Fprintln(os.Stderr, "Error: poll failed")
			shouldExit = true
			continue
		}
		if n == 0 {
			continue
		}

		if fds[0].revents&pollIn != 0 {
			buf := make([]byte, 256)
			got, err := os.Stdin.Read(buf)
			if err != nil || got <= 0 {
				shouldExit = true
			} else {
				result, ok := processInput(intf, buf[:got], &escapePending, &lastWasCR, escapeChar)
				if !ok {
					shouldExit = true
					bmcClosed = true
				} else if result == 1 {
					shouldExit = true
				}
			}
		} else if fds[1].revents&pollIn != 0 {
			pkt, err := intf.SolRecv()
			if err != nil {
				fmt.Fprintln(os.Stderr, "SOL session closed by BMC")
				shouldExit = true
				bmcClosed = true
			} else if len(pkt.Data) != 0 {
				os.Stdout.Write(pkt.Data)
			}
		}
	}

	if bmcClosed {
		return 1
	}
	return 0
}
