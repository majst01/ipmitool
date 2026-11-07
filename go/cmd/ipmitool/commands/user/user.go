// Package user implements the `user` command: manage IPMI user accounts and
// channel access.
//
// It mirrors the Rust `commands/user.rs` (itself a port of `lib/ipmi_user.c`):
// `summary`, `list`, `set name`, `set password`, `enable`, `disable`, `priv`
// and `test`.
package user

import (
	"bufio"
	"bytes"
	"fmt"
	"os"
	"strings"

	"codeberg.org/ipmitool/ipmitool/internal/args"
	"codeberg.org/ipmitool/ipmitool/internal/globals"
	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
)

const netfnApp = ipmicore.NetfnApp

const (
	cmdSetUserAccess   uint8 = 0x43
	cmdGetUserAccess   uint8 = 0x44
	cmdSetUserName     uint8 = 0x45
	cmdGetUserName     uint8 = 0x46
	cmdSetUserPassword uint8 = 0x47
)

const (
	passwordDisableUser  uint8 = 0x00
	passwordEnableUser   uint8 = 0x01
	passwordSetPassword  uint8 = 0x02
	passwordTestPassword uint8 = 0x03
)

// The user id is a 6-bit field in bits [5:0].
const (
	uidMask uint8 = 0x3F
	uidMin  uint8 = 1
	uidMax  uint8 = 63
)

// channelCurrent (0x0E) tells the BMC to use the channel the request arrived on.
const channelCurrent uint8 = 0x0E

const (
	userPwIPMI15Len = 16
	userPwIPMI20Len = 20
	userPwMaxLen    = userPwIPMI20Len
)

// uid is `IPMI_UID(id)`.
func uid(id uint8) uint8 { return id & uidMask }

// userAccess holds the access rights returned by `Get User Access`.
type userAccess struct {
	callinCallback bool
	channel        uint8
	enabledUserIDs uint8
	enableStatus   uint8
	fixedUserIDs   uint8
	ipmiMessaging  bool
	linkAuth       bool
	maxUserIDs     uint8
	privilegeLimit uint8
	sessionLimit   uint8
	userID         uint8
}

// userName is a user name as fetched by `Get User Name`.
type userName struct {
	userID   uint8
	userName string
}

// Main is the entry point for the `user` command (`ipmi_user_main`).
func Main(intf ipmicore.Interface, argv []string) int {
	if len(argv) == 0 {
		fmt.Fprintln(os.Stderr, "Not enough parameters given.")
		printUsage()
		return -1
	}

	switch argv[0] {
	case "help":
		printUsage()
		return 0
	case "summary":
		return summary(intf, argv)
	case "list":
		return list(intf, argv)
	case "test":
		return test(intf, argv)
	case "set":
		if len(argv) >= 3 && argv[1] == "password" {
			return password(intf, argv)
		} else if len(argv) >= 2 && argv[1] == "name" {
			return name(intf, argv)
		}
		printUsage()
		return -1
	case "priv":
		return privCmd(intf, argv)
	case "disable", "enable":
		return modUser(intf, argv)
	default:
		fmt.Fprintf(os.Stderr, "Invalid user command: '%s'\n", argv[0])
		printUsage()
		return -1
	}
}

func printUsage() {
	fmt.Fprintln(os.Stderr, "User Commands:")
	fmt.Fprintln(os.Stderr, "               summary      [<channel number>]")
	fmt.Fprintln(os.Stderr, "               list         [<channel number>]")
	fmt.Fprintln(os.Stderr, "               set name     <user id> <username>")
	fmt.Fprintln(os.Stderr, "               set password <user id> [<password> [<16|20>]]")
	fmt.Fprintln(os.Stderr, "               disable      <user id>")
	fmt.Fprintln(os.Stderr, "               enable       <user id>")
	fmt.Fprintln(os.Stderr, "               priv         <user id> <privilege level> [<channel number>]")
	fmt.Fprintln(os.Stderr, "                     Privilege levels:")
	fmt.Fprintln(os.Stderr, "                      * 0x1 - Callback")
	fmt.Fprintln(os.Stderr, "                      * 0x2 - User")
	fmt.Fprintln(os.Stderr, "                      * 0x3 - Operator")
	fmt.Fprintln(os.Stderr, "                      * 0x4 - Administrator")
	fmt.Fprintln(os.Stderr, "                      * 0x5 - OEM Proprietary")
	fmt.Fprintln(os.Stderr, "                      * 0xF - No Access")
	fmt.Fprintln(os.Stderr)
	fmt.Fprintln(os.Stderr, "               test         <user id> <16|20> [<password>]")
	fmt.Fprintln(os.Stderr)
}

func summary(intf ipmicore.Interface, argv []string) int {
	var channel uint8
	switch len(argv) {
	case 1:
		channel = channelCurrent
	case 2:
		c, ok := isChannelNum(argv[1])
		if !ok {
			return -1
		}
		channel = c
	default:
		printUsage()
		return -1
	}
	return printSummary(intf, channel)
}

func list(intf ipmicore.Interface, argv []string) int {
	var channel uint8
	switch len(argv) {
	case 1:
		channel = channelCurrent
	case 2:
		c, ok := isChannelNum(argv[1])
		if !ok {
			return -1
		}
		channel = c
	default:
		printUsage()
		return -1
	}
	return printList(intf, channel)
}

func test(intf ipmicore.Interface, argv []string) int {
	// argv[0] == "test", argv[1] == user id, argv[2] == 16|20, argv[3] optional password
	if len(argv) != 3 && len(argv) != 4 {
		printUsage()
		return -1
	}
	userID, ok := isUserID(argv[1])
	if !ok {
		return -1
	}
	v, ok := args.ParseNum(argv[2])
	passwordLength := 0
	if ok && (v == userPwIPMI15Len || v == userPwIPMI20Len) {
		passwordLength = int(v)
	} else {
		fmt.Fprintf(os.Stderr, "Given password length '%s' is invalid.\n", argv[2])
		fmt.Fprintln(os.Stderr, "Expected value is either 16 or 20.")
		return -1
	}
	password := ""
	if len(argv) == 3 {
		password = askPassword(userID)
	} else {
		password = argv[3]
	}
	return testPassword(intf, userID, password, passwordLength == userPwIPMI20Len)
}

func password(intf ipmicore.Interface, argv []string) int {
	// argv[0] == "set", argv[1] == "password", argv[2] == user id,
	// argv[3] optional password, argv[4] optional 16|20
	userID, ok := isUserID(argv[2])
	if !ok {
		return -1
	}

	var pw string
	if len(argv) == 3 {
		first := askPassword(userID)
		second := askPassword(userID)
		compareLen := min(len(second), userPwMaxLen+1)
		if !strncmpEq(first, second, compareLen) {
			fmt.Fprintf(os.Stderr, "Passwords do not match or are longer than %d\n", userPwMaxLen)
			return -1
		}
		pw = first
	} else {
		pw = argv[3]
	}

	passwordLen := min(len(pw), userPwMaxLen+1)
	passwordType := userPwIPMI15Len
	if len(argv) > 4 {
		v, ok := args.ParseNum(argv[4])
		if ok && (v == userPwIPMI15Len || v == userPwIPMI20Len) {
			passwordType = int(v)
		} else {
			fmt.Fprintf(os.Stderr, "Invalid password length '%s'\n", argv[4])
			return -1
		}
	} else if passwordLen > userPwIPMI15Len {
		passwordType = userPwIPMI20Len
	}

	if passwordLen > passwordType {
		fmt.Fprintf(os.Stderr, "Password is too long (> %d bytes)\n", passwordType)
		return -1
	}

	ccode := setUserPassword(intf, userID, passwordSetPassword, &pw, passwordType > userPwIPMI15Len)
	if evalCcode(ccode) != 0 {
		fmt.Fprintf(os.Stderr, "Set User Password command failed (user %d)\n", userID)
		return -1
	}
	fmt.Printf("Set User Password command successful (user %d)\n", userID)
	return 0
}

func privCmd(intf ipmicore.Interface, argv []string) int {
	// argv[0] == "priv", argv[1] == user id, argv[2] == priv, argv[3] optional channel
	if len(argv) != 3 && len(argv) != 4 {
		printUsage()
		return -1
	}
	channel := channelCurrent
	if len(argv) == 4 {
		c, ok := isChannelNum(argv[3])
		if !ok {
			return -1
		}
		channel = c
	}
	privilegeLimit, ok := isPrivLimit(argv[2])
	if !ok {
		return -1
	}
	userID, ok := isUserID(argv[1])
	if !ok {
		return -1
	}
	ua := userAccess{
		channel:        channel,
		privilegeLimit: privilegeLimit,
		userID:         userID,
	}

	ccode := setUserAccess(intf, &ua, true)
	if evalCcode(ccode) != 0 {
		fmt.Fprintf(os.Stderr, "Set Privilege Level command failed (user %d)\n", userID)
		return -1
	}
	fmt.Printf("Set Privilege Level command successful (user %d)\n", userID)
	return 0
}

func modUser(intf ipmicore.Interface, argv []string) int {
	// argv[0] == "disable"|"enable", argv[1] == user id
	if len(argv) != 2 {
		printUsage()
		return -1
	}
	userID, ok := isUserID(argv[1])
	if !ok {
		return -1
	}
	operation := passwordDisableUser
	if argv[0] == "disable" {
		operation = passwordDisableUser
	} else {
		operation = passwordEnableUser
	}

	ccode := setUserPassword(intf, userID, operation, nil, false)
	if evalCcode(ccode) != 0 {
		fmt.Fprintf(os.Stderr, "Set User Password command failed (user %d)\n", userID)
		return -1
	}
	return 0
}

func name(intf ipmicore.Interface, argv []string) int {
	// argv[0] == "set", argv[1] == "name", argv[2] == user id, argv[3] == name
	if len(argv) != 4 {
		printUsage()
		return -1
	}
	userID, ok := isUserID(argv[2])
	if !ok {
		return -1
	}
	if len(argv[3]) > 16 {
		fmt.Fprintln(os.Stderr, "Username is too long (> 16 bytes)")
		return -1
	}
	return setUsername(intf, userID, argv[3])
}

// ---------------------------------------------------------------------------
// Protocol operations (mirror `_ipmi_get_user_access` and friends).
// ---------------------------------------------------------------------------

// getUserAccess returns 0 on success, a negative value for a transport/length
// error, or a positive completion code.
func getUserAccess(intf ipmicore.Interface, ua *userAccess) int {
	data := []byte{ua.channel & 0x0F, uid(ua.userID)}
	req := ipmicore.NewRequest(netfnApp, cmdGetUserAccess).WithData(data)
	rsp, err := intf.SendRecv(req)
	if err != nil {
		return -1
	}
	if rsp.Ccode != 0 {
		return int(rsp.Ccode)
	}
	if len(rsp.Data) != 4 {
		return -2
	}
	ua.maxUserIDs = uid(rsp.Data[0])
	ua.enableStatus = rsp.Data[1] & 0xC0
	ua.enabledUserIDs = uid(rsp.Data[1])
	ua.fixedUserIDs = uid(rsp.Data[2])
	ua.callinCallback = rsp.Data[3]&0x40 != 0
	ua.linkAuth = rsp.Data[3]&0x20 != 0
	ua.ipmiMessaging = rsp.Data[3]&0x10 != 0
	ua.privilegeLimit = rsp.Data[3] & 0x0F
	return 0
}

// getUserName fetches the user name for `un.userID` into `un.userName`.
func getUserName(intf ipmicore.Interface, un *userName) int {
	req := ipmicore.NewRequest(netfnApp, cmdGetUserName).WithData([]byte{uid(un.userID)})
	rsp, err := intf.SendRecv(req)
	if err != nil {
		return -1
	}
	if rsp.Ccode != 0 {
		return int(rsp.Ccode)
	}
	if len(rsp.Data) != 16 {
		return -2
	}
	end := bytes.IndexByte(rsp.Data, 0)
	if end < 0 {
		end = 16
	}
	un.userName = strings.ToValidUTF8(string(rsp.Data[:end]), "\uFFFD")
	return 0
}

func setUserAccess(intf ipmicore.Interface, ua *userAccess, changePrivLimitOnly bool) int {
	d0 := uint8(0x00)
	if !changePrivLimitOnly {
		d0 = 0x80
	}
	if ua.callinCallback {
		d0 |= 0x40
	}
	if ua.linkAuth {
		d0 |= 0x20
	}
	if ua.ipmiMessaging {
		d0 |= 0x10
	}
	d0 |= ua.channel & 0x0F
	data := []byte{
		d0,
		uid(ua.userID),
		ua.privilegeLimit & 0x0F,
		ua.sessionLimit & 0x0F,
	}
	req := ipmicore.NewRequest(netfnApp, cmdSetUserAccess).WithData(data)
	rsp, err := intf.SendRecv(req)
	if err != nil {
		return -1
	}
	return int(rsp.Ccode)
}

func setUserPassword(intf ipmicore.Interface, userID, operation uint8, pw *string, isTwentyByte bool) int {
	dataLen := 18
	if isTwentyByte {
		dataLen = 22
	}
	data := make([]byte, dataLen)
	if isTwentyByte {
		data[0] = 0x80 | uid(userID)
	} else {
		data[0] = 0x00 | uid(userID)
	}
	data[1] = 0x03 & operation
	if pw != nil {
		b := []byte(*pw)
		n := min(len(b), dataLen-2)
		copy(data[2:2+n], b[:n])
	}
	req := ipmicore.NewRequest(netfnApp, cmdSetUserPassword).WithData(data)
	rsp, err := intf.SendRecv(req)
	if err != nil {
		return -1
	}
	return int(rsp.Ccode)
}

func setUsername(intf ipmicore.Interface, userID uint8, name string) int {
	if len(name) >= 17 {
		return -1
	}
	data := make([]byte, 17)
	data[0] = uid(userID)
	b := []byte(name)
	copy(data[1:1+len(b)], b)
	req := ipmicore.NewRequest(netfnApp, cmdSetUserName).WithData(data)
	rsp, err := intf.SendRecv(req)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Set User Name command failed (user %d, name %s)\n", userID, name)
		return -1
	}
	if rsp.Ccode != 0 {
		fmt.Fprintf(os.Stderr, "Set User Name command failed (user %d, name %s): %s\n",
			userID, name, ipmicore.CompletionCodeStr(rsp.Ccode))
		return -1
	}
	return 0
}

func testPassword(intf ipmicore.Interface, userID uint8, pw string, isTwentyBytePassword bool) int {
	ret := setUserPassword(intf, userID, passwordTestPassword, &pw, isTwentyBytePassword)
	switch ret {
	case 0:
		fmt.Println("Success")
	case 0x80:
		fmt.Println("Failure: password incorrect")
	case 0x81:
		fmt.Println("Failure: wrong password size")
	default:
		fmt.Println("Unknown error")
	}
	if ret == 0 {
		return 0
	}
	return -1
}

// ---------------------------------------------------------------------------
// Output.
// ---------------------------------------------------------------------------

func printSummary(intf ipmicore.Interface, channel uint8) int {
	ua := userAccess{
		channel: channel,
		userID:  1,
	}
	if evalCcode(getUserAccess(intf, &ua)) != 0 {
		return -1
	}
	if globals.CSV() {
		fmt.Printf("%d,%d,%d\n", ua.maxUserIDs, ua.enabledUserIDs, ua.fixedUserIDs)
	} else {
		fmt.Printf("Maximum IDs\t    : %d\n", ua.maxUserIDs)
		fmt.Printf("Enabled User Count  : %d\n", ua.enabledUserIDs)
		fmt.Printf("Fixed Name Count    : %d\n", ua.fixedUserIDs)
	}
	return 0
}

func printList(intf ipmicore.Interface, channel uint8) int {
	currentUserID := uint8(1)
	printedHeader := false
	for {
		ua := userAccess{
			userID:  currentUserID,
			channel: channel,
		}
		if evalCcode(getUserAccess(intf, &ua)) != 0 {
			return -1
		}
		un := userName{userID: currentUserID}
		ccode := getUserName(intf, &un)
		if ccode == 0xCC {
			// Some BMCs return "Invalid data field in request" for user ids
			// that have no name; treat them as having an empty name.
			un.userName = ""
		} else if evalCcode(ccode) != 0 {
			return -1
		}

		if globals.CSV() {
			dumpUserAccessCSV(un.userName, &ua)
		} else {
			dumpUserAccess(un.userName, &ua, &printedHeader)
		}

		currentUserID++
		if !(currentUserID <= ua.maxUserIDs && currentUserID <= uidMax) {
			break
		}
	}
	return 0
}

func dumpUserAccess(name string, ua *userAccess, printedHeader *bool) {
	if !*printedHeader {
		fmt.Println("ID  Name\t     Callin  Link Auth\tIPMI Msg   Channel Priv Limit")
		*printedHeader = true
	}
	callin := "true "
	if ua.callinCallback {
		callin = "false"
	}
	linkAuth := "false"
	if ua.linkAuth {
		linkAuth = "true "
	}
	ipmiMsg := "false"
	if ua.ipmiMessaging {
		ipmiMsg = "true "
	}
	fmt.Printf("%-4d%-17s%-8s%-11s%-11s%s\n",
		ua.userID, name, callin, linkAuth, ipmiMsg, ipmicore.PrivlvlStr(ua.privilegeLimit))
}

func dumpUserAccessCSV(name string, ua *userAccess) {
	callin := "true"
	if ua.callinCallback {
		callin = "false"
	}
	linkAuth := "false"
	if ua.linkAuth {
		linkAuth = "true"
	}
	ipmiMsg := "false"
	if ua.ipmiMessaging {
		ipmiMsg = "true"
	}
	fmt.Printf("%d,%s,%s,%s,%s,%s\n",
		ua.userID, name, callin, linkAuth, ipmiMsg, ipmicore.PrivlvlStr(ua.privilegeLimit))
}

// ---------------------------------------------------------------------------
// Helpers.
// ---------------------------------------------------------------------------

// evalCcode mirrors `eval_ccode()`: print the C error text and collapse
// everything to a plain success/failure result.
func evalCcode(ccode int) int {
	if ccode == 0 {
		return 0
	}
	if ccode < 0 {
		switch ccode {
		case -1:
			fmt.Fprintln(os.Stderr, "IPMI response is NULL.")
		case -2:
			fmt.Fprintln(os.Stderr, "Unexpected data length received.")
		case -3:
			fmt.Fprintln(os.Stderr, "Invalid function parameter.")
		case -4:
			fmt.Fprintln(os.Stderr, "ipmitool: malloc failure.")
		}
		return -1
	}
	fmt.Fprintf(os.Stderr, "IPMI command failed: %s\n", ipmicore.CompletionCodeStr(uint8(ccode)))
	return -1
}

func isChannelNum(s string) (uint8, bool) {
	c, ok := args.ParseU8(s)
	if ok && (c <= 0xB || (c >= 0xE && c <= 0xF)) {
		return c, true
	}
	fmt.Fprintf(os.Stderr, "Given Channel number '%s' is either invalid or out of range.\n", s)
	fmt.Fprintln(os.Stderr, "Channel number must be from ranges: <0x0..0xB>, <0xE..0xF>")
	return 0, false
}

func isUserID(s string) (uint8, bool) {
	id, ok := args.ParseU8(s)
	if ok && id >= uidMin && id <= uidMax {
		return id, true
	}
	fmt.Fprintf(os.Stderr, "Given User ID '%s' is either invalid or out of range.\n", s)
	fmt.Fprintf(os.Stderr, "User ID is limited to range <%d..%d>.\n", uidMin, uidMax)
	return 0, false
}

func isPrivLimit(s string) (uint8, bool) {
	p, ok := args.ParseU8(s)
	if ok && ((p >= 0x01 && p <= 0x05) || p == 0x0F) {
		return p, true
	}
	fmt.Fprintf(os.Stderr, "Given Privilege Limit '%s' is invalid.\n", s)
	fmt.Fprintln(os.Stderr, "Privilege Limit is limited to <0x1..0x5> and <0xF>.")
	return 0, false
}

// userStdin is the shared buffered stdin, matching Rust's global stdin.
var userStdin = bufio.NewReader(os.Stdin)

// askPassword mirrors `ipmi_user_build_password_prompt()` + `ask_password()`.
// Echo is not disabled, matching the existing `-a` limitation.
func askPassword(userID uint8) string {
	fmt.Fprintf(os.Stderr, "Password for user %d: ", userID)
	line, _ := userStdin.ReadString('\n')
	return strings.TrimRight(line, "\r\n")
}

// strncmpEq is `strncmp(a, b, n) == 0`, treating past-the-end bytes as NUL.
func strncmpEq(a, b string, n int) bool {
	ab := []byte(a)
	bb := []byte(b)
	for i := range n {
		var x, y byte
		if i < len(ab) {
			x = ab[i]
		}
		if i < len(bb) {
			y = bb[i]
		}
		if x != y {
			return false
		}
	}
	return true
}
