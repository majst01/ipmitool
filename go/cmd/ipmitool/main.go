// Command ipmitool is a drop-in compatible IPMI utility. It is the Go port of
// the Rust workspace under rust/ (itself a rewrite of the C ipmitool).
package main

import (
	"fmt"
	"os"

	"codeberg.org/ipmitool/ipmitool/cmd/ipmitool/commands/chassis"
	"codeberg.org/ipmitool/ipmitool/cmd/ipmitool/commands/fru"
	"codeberg.org/ipmitool/ipmitool/cmd/ipmitool/commands/mc"
	"codeberg.org/ipmitool/ipmitool/cmd/ipmitool/commands/raw"
	"codeberg.org/ipmitool/ipmitool/cmd/ipmitool/commands/sol"
	"codeberg.org/ipmitool/ipmitool/cmd/ipmitool/commands/user"
	"codeberg.org/ipmitool/ipmitool/internal/args"
	"codeberg.org/ipmitool/ipmitool/internal/globals"
	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
	"codeberg.org/ipmitool/ipmitool/internal/lanplus"
)

// VERSION is the version string reported by -V, matching the C build's VERSION.
const VERSION = "1.8.19"

func main() {
	os.Exit(run(os.Args[1:]))
}

func run(argv []string) int {
	parsed, err := args.Parse(argv)
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		return 1
	}
	switch parsed.Kind {
	case args.ParsedHelp:
		printUsage()
		return 0
	case args.ParsedVersion:
		fmt.Printf("ipmitool version %s\n", VERSION)
		return 0
	}
	opts := parsed.Options

	// Mirror the C globals `verbose` and `csv_output` before any command runs.
	globals.SetVerbose(opts.Verbose)
	globals.SetCSV(opts.CSV)

	intfName := "lanplus"
	if opts.Intf != nil {
		intfName = *opts.Intf
	}
	if intfName != "lanplus" {
		fmt.Fprintf(os.Stderr, "Interface %s not supported\n", intfName)
		return 1
	}

	params := ipmicore.DefaultSessionParams()
	params.Hostname = opts.Hostname
	params.Port = opts.Port
	if opts.Username != nil {
		params.SetUsername(*opts.Username)
	}
	if opts.Password != nil {
		if len(*opts.Password) > 20 {
			fmt.Fprintln(os.Stderr, "lanplus: password is longer than 20 bytes.")
			return 1
		}
		params.SetPassword(*opts.Password)
	}
	params.Privlvl = opts.Privlvl
	params.Lookupbit = opts.Lookupbit
	params.AuthtypeSet = opts.Authtype
	params.Timeout = opts.Timeout
	params.Retry = opts.Retry
	params.TargetLun = opts.TargetLun
	params.TargetAddr = opts.TargetAddr
	params.TargetChannel = opts.TargetChannel
	params.TransitAddr = opts.TransitAddr
	params.TransitChannel = opts.TransitChannel
	params.MyAddr = uint32(opts.MyAddr)
	params.CipherSuite = opts.Cipher
	params.SolEscapeChar = opts.EscapeChar
	if opts.Kgkey != nil {
		kg := []byte(*opts.Kgkey)
		n := min(len(kg), len(params.Kg))
		copy(params.Kg[:n], kg[:n])
	}

	if params.Hostname == nil {
		fmt.Fprintln(os.Stderr, "No hostname specified!")
		return 1
	}

	intf := lanplus.New(params)
	if err := intf.Open(); err != nil {
		fmt.Fprintf(os.Stderr, "Error: Unable to establish IPMI v2 / RMCP+ session: %v\n", err)
		return 1
	}

	code := dispatch(intf, parsed.Command, parsed.Args)

	intf.Close()
	return code
}

func dispatch(intf ipmicore.Interface, command string, cargs []string) int {
	switch command {
	case "raw":
		return raw.Main(intf, cargs)
	case "chassis":
		return chassis.Main(intf, cargs)
	case "power":
		return chassis.PowerShortcut(intf, cargs)
	case "mc", "bmc":
		return mc.Main(intf, cargs)
	case "sol":
		return sol.Main(intf, cargs)
	case "fru":
		return fru.Main(intf, cargs)
	case "user":
		return user.Main(intf, cargs)
	default:
		fmt.Fprintf(os.Stderr, "Invalid command: %s\n", command)
		return -1
	}
}

func printUsage() {
	fmt.Printf("ipmitool version %s\n", VERSION)
	fmt.Println("usage: ipmitool [options...] <command>")
	fmt.Println("       -h             This help")
	fmt.Println("       -V             Show version information")
	fmt.Println("       -v             Verbose (can use multiple times)")
	fmt.Println("       -c             Display output in comma separated format")
	fmt.Println("       -I intf        Interface to use (only lanplus is supported)")
	fmt.Println("       -H hostname    Remote host name for LAN interface")
	fmt.Println("       -p port        Remote RMCP port [default=623]")
	fmt.Println("       -U username    Remote session username")
	fmt.Println("       -P password    Remote session password")
	fmt.Println("       -C ciphersuite Cipher suite to be used by lanplus interface")
	fmt.Println("       -L level       Remote session privilege level [default=ADMINISTRATOR]")
	fmt.Println("       -m address     Set local IPMB address")
	fmt.Println("       -b channel     Set destination channel for bridged request")
	fmt.Println("       -l lun         Set destination lun for raw commands")
	fmt.Println("       -N seconds     Specify timeout for lanplus interface [default=1]")
	fmt.Println("       -R retry       Set the number of retries [default=4]")
	fmt.Println()
	fmt.Println("Commands:")
	fmt.Println("  raw         Send a RAW IPMI request and print response")
	fmt.Println("  chassis     Get chassis status and set power state")
	fmt.Println("  power       Shortcut to chassis power commands")
	fmt.Println("  mc          Management Controller status and global enables")
	fmt.Println("  sol         Configure and connect to the Serial-over-LAN interface")
	fmt.Println("  fru         Print Field Replaceable Unit (FRU) inventory information")
	fmt.Println("  user        Manage IPMI user accounts and channel access")
}
