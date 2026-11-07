// Package args is the hand-written command-line parser mirroring ipmitool's
// `getopt` handling in `lib/ipmi_main.c`. It is the Go counterpart to the Rust
// `ipmitool` crate's `args` module.
package args

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
)

// ParsedKind discriminates the result of Parse.
type ParsedKind int

// Parse results.
const (
	ParsedRun ParsedKind = iota
	ParsedHelp
	ParsedVersion
)

// Parsed is the result of Parse.
type Parsed struct {
	Kind    ParsedKind
	Options Options
	Command string
	Args    []string
}

// Options holds the command line options.
type Options struct {
	Intf           *string
	Hostname       *string
	Port           uint16
	Username       *string
	Password       *string
	Cipher         *uint8
	Privlvl        uint8
	Lookupbit      uint8
	Authtype       *uint8
	Timeout        uint32
	Retry          uint32
	TargetLun      uint8
	TargetAddr     uint8
	TargetChannel  uint8
	TransitAddr    uint8
	TransitChannel uint8
	MyAddr         uint8
	Kgkey          *string
	EscapeChar     uint8
	Verbose        uint32
	CSV            bool
}

// DefaultOptions mirrors `Options::default()`.
func DefaultOptions() Options {
	return Options{
		Privlvl:    ipmicore.SessionPrivAdmin,
		Lookupbit:  0x10,
		MyAddr:     ipmicore.BmcSlaveAddr,
		EscapeChar: '~',
	}
}

// ParseNum parses a number that may be decimal or hex (`0x..`) into a uint32.
func ParseNum(s string) (uint32, bool) {
	s = strings.TrimSpace(s)
	if hex, ok := strings.CutPrefix(s, "0x"); ok {
		v, err := strconv.ParseUint(hex, 16, 32)
		return uint32(v), err == nil
	}
	if hex, ok := strings.CutPrefix(s, "0X"); ok {
		v, err := strconv.ParseUint(hex, 16, 32)
		return uint32(v), err == nil
	}
	v, err := strconv.ParseUint(s, 10, 32)
	return uint32(v), err == nil
}

// ParseU8 parses a u8 the way ipmitool's `str2uchar` does.
func ParseU8(s string) (uint8, bool) {
	v, ok := ParseNum(s)
	if !ok || v > 0xff {
		return 0, false
	}
	return uint8(v), true
}

func takesValue(c byte) bool {
	switch c {
	case 'I', 'H', 'p', 'U', 'P', 'f', 'k', 'y', 'S', 'D', 'O', 'C', 'L',
		'A', 't', 'T', 'b', 'B', 'l', 'm', 'e', 'o', 'N', 'R', 'd', 'z':
		return true
	default:
		return false
	}
}

func req(value *string, flag string) (string, error) {
	if value == nil {
		return "", fmt.Errorf("option %s requires an argument", flag)
	}
	return *value, nil
}

// Parse parses the command line, returning an error on malformed input.
func Parse(argv []string) (Parsed, error) {
	opts := DefaultOptions()
	var positionals []string

	i := 0
	for i < len(argv) {
		arg := argv[i]
		if arg == "--" {
			positionals = append(positionals, argv[i+1:]...)
			break
		}
		if len(arg) >= 2 && arg[0] == '-' && arg != "-" {
			chars := arg[1:]
			ci := 0
			for ci < len(chars) {
				c := chars[ci]
				var value *string
				if takesValue(c) {
					rest := string(chars[ci+1:])
					if rest != "" {
						v := rest
						value = &v
					} else {
						i++
						if i < len(argv) {
							v := argv[i]
							value = &v
						}
					}
				}

				switch c {
				case 'I':
					opts.Intf = value
				case 'H':
					opts.Hostname = value
				case 'p':
					s, err := req(value, "-p")
					if err != nil {
						return Parsed{}, err
					}
					n, ok := ParseNum(s)
					if !ok || n > 65535 {
						return Parsed{}, fmt.Errorf("invalid port")
					}
					opts.Port = uint16(n)
				case 'U':
					v, err := req(value, "-U")
					if err != nil {
						return Parsed{}, err
					}
					if len(v) > 16 {
						return Parsed{}, fmt.Errorf("Username is too long (> 16 bytes)")
					}
					opts.Username = &v
				case 'P':
					opts.Password = value
				case 'f':
					path, err := req(value, "-f")
					if err != nil {
						return Parsed{}, err
					}
					pw, err := readPasswordFile(path)
					if err != nil {
						return Parsed{}, err
					}
					opts.Password = &pw
				case 'a', 'Y':
					prompt := "Password: "
					if c == 'Y' {
						prompt = "Key: "
					}
					pw := promptPassword(prompt)
					opts.Password = &pw
				case 'E':
					if v, ok := os.LookupEnv("IPMITOOL_PASSWORD"); ok {
						opts.Password = &v
					} else if v, ok := os.LookupEnv("IPMI_PASSWORD"); ok {
						opts.Password = &v
					}
				case 'k', 'y':
					opts.Kgkey = value
				case 'K':
					if v, ok := os.LookupEnv("IPMI_KGKEY"); ok {
						opts.Kgkey = &v
					}
				case 'C':
					s, err := req(value, "-C")
					if err != nil {
						return Parsed{}, err
					}
					n, ok := ParseU8(s)
					if !ok {
						return Parsed{}, fmt.Errorf("invalid cipher suite")
					}
					opts.Cipher = &n
				case 'L':
					v, err := req(value, "-L")
					if err != nil {
						return Parsed{}, err
					}
					if strings.HasSuffix(v, "+") {
						opts.Lookupbit = 0
						v = v[:len(v)-1]
					}
					priv, ok := ipmicore.PrivlvlFromStr(v)
					if !ok {
						return Parsed{}, fmt.Errorf("Invalid privilege level %s", v)
					}
					opts.Privlvl = priv
				case 'A':
					s, err := req(value, "-A")
					if err != nil {
						return Parsed{}, err
					}
					at, ok := ipmicore.AuthtypeFromStr(s)
					if !ok {
						return Parsed{}, fmt.Errorf("invalid authentication type")
					}
					opts.Authtype = &at
				case 't':
					s, err := req(value, "-t")
					if err != nil {
						return Parsed{}, err
					}
					n, ok := ParseU8(s)
					if !ok {
						return Parsed{}, fmt.Errorf("invalid address")
					}
					opts.TargetAddr = n
				case 'T':
					s, err := req(value, "-T")
					if err != nil {
						return Parsed{}, err
					}
					n, ok := ParseU8(s)
					if !ok {
						return Parsed{}, fmt.Errorf("invalid address")
					}
					opts.TransitAddr = n
				case 'b':
					s, err := req(value, "-b")
					if err != nil {
						return Parsed{}, err
					}
					n, ok := ParseU8(s)
					if !ok {
						return Parsed{}, fmt.Errorf("invalid channel")
					}
					opts.TargetChannel = n
				case 'B':
					s, err := req(value, "-B")
					if err != nil {
						return Parsed{}, err
					}
					n, ok := ParseU8(s)
					if !ok {
						return Parsed{}, fmt.Errorf("invalid channel")
					}
					opts.TransitChannel = n
				case 'l':
					s, err := req(value, "-l")
					if err != nil {
						return Parsed{}, err
					}
					n, ok := ParseU8(s)
					if !ok {
						return Parsed{}, fmt.Errorf("invalid lun")
					}
					opts.TargetLun = n
				case 'm':
					s, err := req(value, "-m")
					if err != nil {
						return Parsed{}, err
					}
					n, ok := ParseU8(s)
					if !ok {
						return Parsed{}, fmt.Errorf("invalid address")
					}
					opts.MyAddr = n
				case 'e':
					v, err := req(value, "-e")
					if err != nil {
						return Parsed{}, err
					}
					if len(v) == 0 {
						return Parsed{}, fmt.Errorf("option -e requires a character")
					}
					opts.EscapeChar = v[0]
				case 'o', 'g', 's', 'S', 'D', 'O', 'd', 'z':
					// OEM / device / serial / caches: accepted but unused.
				case 'N':
					s, err := req(value, "-N")
					if err != nil {
						return Parsed{}, err
					}
					n, ok := ParseNum(s)
					if !ok {
						return Parsed{}, fmt.Errorf("invalid timeout")
					}
					opts.Timeout = n
				case 'R':
					s, err := req(value, "-R")
					if err != nil {
						return Parsed{}, err
					}
					n, ok := ParseNum(s)
					if !ok {
						return Parsed{}, fmt.Errorf("invalid retry count")
					}
					opts.Retry = n
				case 'v':
					opts.Verbose++
				case 'c':
					opts.CSV = true
				case '4', '6', 'Z':
				case 'h':
					return Parsed{Kind: ParsedHelp}, nil
				case 'V':
					return Parsed{Kind: ParsedVersion}, nil
				default:
					return Parsed{}, fmt.Errorf("invalid option -- '%c'", c)
				}
				ci++
			}
		} else {
			positionals = append(positionals, arg)
		}
		i++
	}

	if len(positionals) == 0 {
		return Parsed{}, fmt.Errorf("no command provided")
	}
	command := positionals[0]
	rest := positionals[1:]
	if command == "help" {
		return Parsed{Kind: ParsedHelp}, nil
	}
	return Parsed{Kind: ParsedRun, Options: opts, Command: command, Args: rest}, nil
}

func readPasswordFile(path string) (string, error) {
	contents, err := os.ReadFile(path)
	if err != nil {
		return "", fmt.Errorf("Unable to read password from file %s: %v", path, err)
	}
	line, _, _ := strings.Cut(string(contents), "\n")
	return strings.TrimRight(line, "\r\n\t"), nil
}

// promptPassword reads a password from the terminal without disabling echo.
func promptPassword(prompt string) string {
	fmt.Fprint(os.Stderr, prompt)
	reader := bufio.NewReader(os.Stdin)
	line, _ := reader.ReadString('\n')
	return strings.TrimRight(line, "\r\n")
}
