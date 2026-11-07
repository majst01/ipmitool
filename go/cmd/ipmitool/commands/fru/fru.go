// Package fru implements the `fru` command: read and print Field Replaceable
// Unit inventory.
//
// It mirrors the read paths of `lib/ipmi_fru.c` (`fru print`, `fru list`,
// `fru read`) and the Rust `commands/fru.rs`. The destructive `write`/`edit`
// commands and the PICMG OEM extension record decoder are not implemented.
package fru

import (
	"fmt"
	"os"
	"strings"
	"time"

	"codeberg.org/ipmitool/ipmitool/cmd/ipmitool/commands/mc"
	"codeberg.org/ipmitool/ipmitool/internal/args"
	"codeberg.org/ipmitool/ipmitool/internal/globals"
	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
)

const (
	netfnStorage = ipmicore.NetfnStorage
	getFruInfo   = 0x10
	getFruData   = 0x11
)

const (
	fruEndOfFields        = 0xc1
	fruBlockSz            = 8
	fruAreaMaximumBlockSz = 32
)

// lanplusMaxResponseSize is `IPMI_LAN_MAX_RESPONSE_SIZE` from
// `src/plugins/lanplus/lanplus.c`. The FRU reader derives its per-request read
// size from this (see readFruArea).
const lanplusMaxResponseSize = 34

const (
	fruRecordTypePowerSupplyInformation = 0x00
	fruRecordTypeDCOutput               = 0x01
	fruRecordTypeDCLoad                 = 0x02
	fruRecordTypeManagementAccess       = 0x03
	fruRecordTypeOemExtension           = 0xc0
	fruRecordFormatEolMask              = 0x80
)

var combinedVoltageDesc = [4]string{"12 V", "-12 V", "5 V", "3.3 V"}

var chassisTypeDesc = [30]string{
	"Unspecified",
	"Other",
	"Unknown",
	"Desktop",
	"Low Profile Desktop",
	"Pizza Box",
	"Mini Tower",
	"Tower",
	"Portable",
	"LapTop",
	"Notebook",
	"Hand Held",
	"Docking Station",
	"All in One",
	"Sub Notebook",
	"Space-saving",
	"Lunch Box",
	"Main Server Chassis",
	"Expansion Chassis",
	"SubChassis",
	"Bus Expansion Chassis",
	"Peripheral Chassis",
	"RAID Chassis",
	"Rack Mount Chassis",
	"Sealed-case PC",
	"Multi-system Chassis",
	"CompactPCI",
	"AdvancedTCA",
	"Blade",
	"Blade Enclosure",
}

// FruInfo is the information returned by `Get FRU Inventory Area Info`.
type FruInfo struct {
	size        uint32
	access      bool
	maxReadSize uint32
}

func newFruInfo() *FruInfo { return &FruInfo{} }

// ---------------------------------------------------------------------------
// Command dispatch (`ipmi_fru_main`).
// ---------------------------------------------------------------------------

// Main implements `ipmitool fru <subcommand>`.
func Main(intf ipmicore.Interface, args []string) int {
	if len(args) == 0 {
		return printAll(intf)
	}
	cmd := args[0]

	switch cmd {
	case "help":
		printHelp()
		return 0
	case "print", "list":
		if len(args) > 1 {
			if args[1] == "help" {
				fmt.Fprintln(os.Stderr, "fru print [fru id] - print information about FRU(s)")
				return 0
			}
			if id, ok := isFruID(args[1]); ok {
				return printDevice(intf, id)
			}
			return -1
		}
		return printAll(intf)
	case "read":
		return read(intf, args[1:])
	case "write", "edit", "get", "internaluse", "upgEkey":
		fmt.Fprintf(os.Stderr, "fru %s: not implemented in this build\n", cmd)
		return -1
	default:
		fmt.Fprintf(os.Stderr, "Invalid FRU command: %s\n", cmd)
		printHelp()
		return -1
	}
}

// isFruID parses a FRU id, printing the C error message on failure.
func isFruID(s string) (uint8, bool) {
	if id, ok := args.ParseU8(s); ok {
		return id, true
	}
	fmt.Fprintf(os.Stderr, "FRU ID '%s' is either invalid or out of range.\n", s)
	return 0, false
}

func printHelp() {
	fmt.Fprintln(os.Stderr, "FRU Commands:  print read write upgEkey edit internaluse get")
}

// ---------------------------------------------------------------------------
// `fru read` (`ipmi_fru_read_to_bin`).
// ---------------------------------------------------------------------------

func read(intf ipmicore.Interface, a []string) int {
	if len(a) > 0 && a[0] == "help" {
		fmt.Fprintln(os.Stderr, "fru read <fru id> <fru file>")
		fmt.Fprintln(os.Stderr, "Note: FRU ID and file(incl. full path) must be specified.")
		fmt.Fprintln(os.Stderr, "Example: ipmitool fru read 0 /root/fru.bin")
		return 0
	}
	if len(a) < 2 {
		fmt.Fprintln(os.Stderr, "Not enough parameters given.")
		fmt.Fprintln(os.Stderr, "fru read <fru id> <fru file>")
		return -1
	}
	fruID, ok := isFruID(a[0])
	if !ok {
		return -1
	}
	file := a[1]
	if isValidFilename(file) != 0 {
		return -1
	}
	if globals.Verbose() > 0 {
		fmt.Printf("FRU ID           : %d\n", fruID)
		fmt.Printf("FRU File         : %s\n", file)
	}
	readToBin(intf, file, fruID)

	return 0
}

func isValidFilename(name string) int {
	if name == "" {
		fmt.Fprintln(os.Stderr, "File/path is invalid.")
		return -2
	}
	if len(name) >= 512 {
		fmt.Fprintln(os.Stderr, "File/path must be shorter than 512 bytes.")
		return -3
	}
	return 0
}

func readToBin(intf ipmicore.Interface, fileName string, fruID uint8) {
	fru, ferr := queryFruInfo(intf, fruID)
	if ferr != nil {
		if ferr.kind == fruInfoNoResponse {
			return
		}
		if ferr.ccode == ipmicore.CC_TIMEOUT {
			fmt.Println("  Timeout accessing FRU info. (Device not present?)")
		}
		return
	}

	if globals.Verbose() > 0 {
		fmt.Printf("Fru Size   = %d bytes\n", fru.size)
		access := 0
		if fru.access {
			access = 1
		}
		fmt.Printf("Fru Access = %xh\n", access)
	}

	fmt.Printf("Fru Size         : %d bytes\n", fru.size)
	size := fru.size
	data, _ := readFruArea(intf, fru, fruID, 0, size)
	if data == nil {
		data = []byte{}
	}

	f, err := os.Create(fileName)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error opening file %s\n", fileName)
		return
	}
	defer f.Close()
	if _, err := f.Write(data); err != nil {
		fmt.Fprintf(os.Stderr, "Error writing file %s: %v\n", fileName, err)
		return
	}
	fmt.Println("Done")
}

// ---------------------------------------------------------------------------
// `fru print` (`__ipmi_fru_print` and `ipmi_fru_print_all`).
// ---------------------------------------------------------------------------

func printDevice(intf ipmicore.Interface, id uint8) int {
	fru, ferr := queryFruInfo(intf, id)
	if ferr != nil {
		if ferr.kind == fruInfoNoResponse {
			fmt.Println(" Device not present (No Response)")
			return -1
		}
		fmt.Printf(" Device not present (%s)\n", ipmicore.CompletionCodeStr(ferr.ccode))
		return -1
	}

	if fru.size < 1 {
		fmt.Fprintf(os.Stderr, " Invalid FRU size %d\n", fru.size)
		return -1
	}

	// Retrieve the common header (8 bytes).
	header, ok := readFruArea(intf, fru, id, 0, 8)
	if !ok {
		return 1
	}
	if globals.Verbose() > 1 {
		printbuf(header, "FRU DATA")
	}
	// readFruArea returns 8 bytes; the caller-visible header starts at 1 in the
	// raw response, but here we read the FRU data directly.
	header = header[:8]

	if header[0] != 1 {
		fmt.Fprintf(os.Stderr, " Unknown FRU header version 0x%02x\n", header[0])
		return -1
	}

	offInternal := uint32(header[1]) * 8
	offChassis := uint32(header[2]) * 8
	offBoard := uint32(header[3]) * 8
	offProduct := uint32(header[4]) * 8
	offMulti := uint32(header[5]) * 8

	if globals.Verbose() > 0 {
		logDebug(fmt.Sprintf("fru.header.version:         0x%x", header[0]))
		logDebug(fmt.Sprintf("fru.header.offset.internal: 0x%x", offInternal))
		logDebug(fmt.Sprintf("fru.header.offset.chassis:  0x%x", offChassis))
		logDebug(fmt.Sprintf("fru.header.offset.board:    0x%x", offBoard))
		logDebug(fmt.Sprintf("fru.header.offset.product:  0x%x", offProduct))
		logDebug(fmt.Sprintf("fru.header.offset.multi:    0x%x", offMulti))
	}

	if offChassis >= 8 {
		printChassis(intf, fru, id, offChassis)
	}
	if offBoard >= 8 {
		printBoard(intf, fru, id, offBoard)
	}
	if offProduct >= 8 {
		printProduct(intf, fru, id, offProduct)
	}

	if globals.Verbose() == 0 {
		return 0
	}
	if offMulti >= 8 {
		printMultirec(intf, fru, id, offMulti)
	}
	return 0
}

func printAll(intf ipmicore.Interface) int {
	fmt.Println("FRU Device Description : Builtin FRU Device (ID 0)")

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
	// `adtl_device_support` (FRU Inventory Device bit) is byte 5.
	var adtl uint8
	if len(rsp.Data) > 5 {
		adtl = rsp.Data[5]
	}
	rc := 0
	if adtl&0x08 != 0 {
		rc = printDevice(intf, 0)
		fmt.Println()
	}
	return rc
}

// fruInfoErrorKind discriminates the two failure branches of
// `Get FRU Inventory Area Info`.
type fruInfoErrorKind int

const (
	// fruInfoNoResponse: the request got no response at all.
	fruInfoNoResponse fruInfoErrorKind = iota
	// fruInfoCcode: the BMC returned a non-zero completion code.
	fruInfoCcode
)

type fruInfoError struct {
	kind  fruInfoErrorKind
	ccode uint8
}

func (e *fruInfoError) Error() string {
	if e.kind == fruInfoNoResponse {
		return "no response"
	}
	return ipmicore.CompletionCodeStr(e.ccode)
}

// queryFruInfo sends `Get FRU Inventory Area Info` for FRU `id`.
func queryFruInfo(intf ipmicore.Interface, id uint8) (*FruInfo, *fruInfoError) {
	req := ipmicore.NewRequest(netfnStorage, getFruInfo).WithData([]byte{id})
	rsp, err := intf.SendRecv(req)
	if err != nil {
		return nil, &fruInfoError{kind: fruInfoNoResponse}
	}
	if !rsp.OK() {
		return nil, &fruInfoError{kind: fruInfoCcode, ccode: rsp.Ccode}
	}
	if len(rsp.Data) < 3 {
		return nil, &fruInfoError{kind: fruInfoNoResponse}
	}
	fru := newFruInfo()
	fru.size = uint32(rsp.Data[1])<<8 | uint32(rsp.Data[0])
	fru.access = rsp.Data[2]&0x1 != 0
	return fru, nil
}

// ---------------------------------------------------------------------------
// Section printers.
// ---------------------------------------------------------------------------

func printChassis(intf ipmicore.Interface, fru *FruInfo, id uint8, offset uint32) {
	tmp, ok := readFruArea(intf, fru, id, offset, 2)
	if !ok {
		return
	}
	fruLen := 8 * int(tmp[1])
	if fruLen == 0 {
		return
	}
	data, ok := readFruArea(intf, fru, id, offset, uint32(fruLen))
	if !ok {
		return
	}

	chassisType := int(data[2])
	if chassisType > len(chassisTypeDesc)-1 {
		chassisType = 2
	}
	fmt.Printf(" Chassis Type          : %s\n", chassisTypeDesc[chassisType])

	i := 3
	printField(data, &i, " Chassis Part Number   : ")
	printField(data, &i, " Chassis Serial        : ")

	for i < fruLen && data[i] != fruEndOfFields {
		j := i
		printField(data, &i, " Chassis Extra         : ")
		if i == j {
			break
		}
	}

	status := "INVALID"
	if checksumIsValid(data, fruLen) {
		status = "OK"
	}
	fmt.Printf(" Chassis Area Checksum : %s\n", status)
}

func printBoard(intf ipmicore.Interface, fru *FruInfo, id uint8, offset uint32) {
	tmp, ok := readFruArea(intf, fru, id, offset, 2)
	if !ok {
		return
	}
	fruLen := 8 * int(tmp[1])
	if fruLen == 0 {
		return
	}
	data, ok := readFruArea(intf, fru, id, offset, uint32(fruLen))
	if !ok {
		return
	}

	i := 3
	fmt.Printf(" Board Mfg Date        : %s\n", formatFruTimestamp(data, i))
	i += 3

	printField(data, &i, " Board Mfg             : ")
	printField(data, &i, " Board Product         : ")
	printField(data, &i, " Board Serial          : ")
	printField(data, &i, " Board Part Number     : ")
	printFieldVerbose(data, &i, " Board FRU ID          : ")

	for i < fruLen && data[i] != fruEndOfFields {
		j := i
		printField(data, &i, " Board Extra           : ")
		if i == j {
			break
		}
	}

	status := "INVALID"
	if checksumIsValid(data, fruLen) {
		status = "OK"
	}
	fmt.Printf(" Board Area Checksum   : %s\n", status)
}

func printProduct(intf ipmicore.Interface, fru *FruInfo, id uint8, offset uint32) {
	tmp, ok := readFruArea(intf, fru, id, offset, 2)
	if !ok {
		return
	}
	fruLen := 8 * int(tmp[1])
	if fruLen == 0 {
		return
	}
	data, ok := readFruArea(intf, fru, id, offset, uint32(fruLen))
	if !ok {
		return
	}

	i := 3
	printField(data, &i, " Product Manufacturer  : ")
	printField(data, &i, " Product Name          : ")
	printField(data, &i, " Product Part Number   : ")
	printField(data, &i, " Product Version       : ")
	printField(data, &i, " Product Serial        : ")
	printField(data, &i, " Product Asset Tag     : ")
	printFieldVerbose(data, &i, " Product FRU ID        : ")

	for i < fruLen && data[i] != fruEndOfFields {
		j := i
		printField(data, &i, " Product Extra         : ")
		if i == j {
			break
		}
	}

	status := "INVALID"
	if checksumIsValid(data, fruLen) {
		status = "OK"
	}
	fmt.Printf(" Product Area Checksum : %s\n", status)
}

// printField reads one FRU area string at *i, prints it if non-empty, and
// advances *i.
func printField(data []byte, i *int, label string) {
	if s, ok := getFruAreaStr(data, i); ok {
		if s != "" {
			fmt.Printf("%s%s\n", label, s)
		}
	}
}

// printFieldVerbose is like printField but only printed at verbosity >= 1
// (FRU ID fields).
func printFieldVerbose(data []byte, i *int, label string) {
	if s, ok := getFruAreaStr(data, i); ok {
		if s != "" && globals.Verbose() > 0 {
			fmt.Printf("%s%s\n", label, s)
		}
	}
}

// ---------------------------------------------------------------------------
// Multi-record area printer.
// ---------------------------------------------------------------------------

func printMultirec(intf ipmicore.Interface, fru *FruInfo, id uint8, offset uint32) {
	lastOff := offset
	var registry *ipmicore.OemRegistry

	for {
		hdr, ok := readFruArea(intf, fru, id, lastOff, 5)
		if !ok {
			break
		}
		rtype := hdr[0]
		format := hdr[1]
		rlen := uint32(hdr[2])

		var body []byte
		if rlen > 0 {
			b, ok := readFruArea(intf, fru, id, lastOff+5, rlen)
			if !ok {
				break
			}
			body = b
		} else {
			body = []byte{}
		}
		lastOff += rlen + 5

		switch rtype {
		case fruRecordTypePowerSupplyInformation:
			printPowerSupply(body)
		case fruRecordTypeDCOutput:
			printDCOutput(body)
		case fruRecordTypeDCLoad:
			printDCLoad(body)
		case fruRecordTypeManagementAccess:
			printManagementAccess(body, rlen)
		case fruRecordTypeOemExtension:
			iana := uint32(byteAt(body, 0)) |
				uint32(byteAt(body, 1))<<8 |
				uint32(byteAt(body, 2))<<16
			if iana == ipmicore.IPMIOemPicmg {
				fmt.Println("  PICMG Extension Record")
				fmt.Fprintln(os.Stderr, "  (PICMG extension record decoding is not implemented in this build)")
			} else {
				if registry == nil {
					registry = ipmicore.LoadOemRegistry()
				}
				fmt.Printf("  OEM (%s) Record\n", registry.ManufacturerName(iana))
			}
		}

		if format&fruRecordFormatEolMask != 0 {
			break
		}
	}

	logDebug(fmt.Sprintf("Multi-Record area ends at: %d (%xh)", lastOff, lastOff))
}

func printPowerSupply(d []byte) {
	capacity := le16(d, 0) & 0x0fff
	peakVa := le16(d, 2)
	inrushCurrent := byteAt(d, 4)
	inrushInterval := byteAt(d, 5)
	lowendInput1 := le16(d, 6)
	highendInput1 := le16(d, 8)
	lowendInput2 := le16(d, 10)
	highendInput2 := le16(d, 12)
	lowendFreq := byteAt(d, 14)
	highendFreq := byteAt(d, 15)
	dropoutTolerance := byteAt(d, 16)
	flags := byteAt(d, 17)
	predictiveFail := flags&0x01 != 0
	pfc := flags&0x02 != 0
	autoswitch := flags&0x04 != 0
	hotswap := flags&0x08 != 0
	tach := flags&0x10 != 0
	peakCapHT := le16(d, 18)
	peakHoldUpTime := (peakCapHT & 0xf000) >> 12
	peakCapacity := peakCapHT & 0x0fff
	combinedByte := byteAt(d, 20)
	combinedVoltage1 := (combinedByte >> 4) & 0x0f
	combinedVoltage2 := combinedByte & 0x0f
	combinedCapacity := le16(d, 21)
	rpsThreshold := byteAt(d, 23)

	fmt.Println(" Power Supply Record")
	fmt.Printf("  Capacity                   : %d W\n", capacity)
	fmt.Printf("  Peak VA                    : %d VA\n", peakVa)
	fmt.Printf("  Inrush Current             : %d A\n", inrushCurrent)
	fmt.Printf("  Inrush Interval            : %d ms\n", inrushInterval)
	fmt.Printf("  Input Voltage Range 1      : %d-%d V\n", lowendInput1/100, highendInput1/100)
	fmt.Printf("  Input Voltage Range 2      : %d-%d V\n", lowendInput2/100, highendInput2/100)
	fmt.Printf("  Input Frequency Range      : %d-%d Hz\n", lowendFreq, highendFreq)
	fmt.Printf("  A/C Dropout Tolerance      : %d ms\n", dropoutTolerance)

	tail := ""
	if predictiveFail {
		if rpsThreshold != 0 {
			if tach {
				tail = "'Two pulses per rotation'"
			} else {
				tail = "'One pulse per rotation'"
			}
		} else if tach {
			tail = "'Failure on pin de-assertion'"
		} else {
			tail = "'Failure on pin assertion'"
		}
	}
	predStr := ""
	if predictiveFail {
		predStr = "'Predictive fail' "
	}
	pfcStr := ""
	if pfc {
		pfcStr = "'Power factor correction' "
	}
	autoStr := ""
	if autoswitch {
		autoStr = "'Autoswitch voltage' "
	}
	hotStr := ""
	if hotswap {
		hotStr = "'Hot swap' "
	}
	fmt.Printf("  Flags                      : %s%s%s%s%s\n", predStr, pfcStr, autoStr, hotStr, tail)
	fmt.Printf("  Peak capacity              : %d W\n", peakCapacity)
	fmt.Printf("  Peak capacity holdup       : %d s\n", peakHoldUpTime)
	if combinedCapacity == 0 {
		fmt.Println("  Combined capacity          : not specified")
	} else {
		fmt.Printf("  Combined capacity          : %d W (%s and %s)\n",
			combinedCapacity, voltageDesc(combinedVoltage1), voltageDesc(combinedVoltage2))
	}
	if predictiveFail {
		fmt.Printf("  Fan lower threshold        : %d RPS\n", rpsThreshold)
	}
}

func voltageDesc(idx uint8) string {
	if int(idx) < len(combinedVoltageDesc) {
		return combinedVoltageDesc[idx]
	}
	return ""
}

func printDCOutput(d []byte) {
	b0 := byteAt(d, 0)
	outputNumber := b0 & 0x0f
	standby := b0&0x80 != 0
	nominalVoltage := int16(le16(d, 1))
	maxNegDev := int16(le16(d, 3))
	maxPosDev := int16(le16(d, 5))
	rippleAndNoise := le16(d, 7)
	minCurrent := le16(d, 9)
	maxCurrent := le16(d, 11)

	fmt.Println(" DC Output Record")
	fmt.Printf("  Output Number              : %d\n", outputNumber)
	if standby {
		fmt.Println("  Standby power              : Yes")
	} else {
		fmt.Println("  Standby power              : No")
	}
	fmt.Printf("  Nominal voltage            : %.2f V\n", float64(nominalVoltage)/100.0)
	fmt.Printf("  Max negative deviation     : %.2f V\n", float64(maxNegDev)/100.0)
	fmt.Printf("  Max positive deviation     : %.2f V\n", float64(maxPosDev)/100.0)
	fmt.Printf("  Ripple and noise pk-pk     : %d mV\n", rippleAndNoise)
	fmt.Printf("  Minimum current draw       : %.3f A\n", float64(minCurrent)/1000.0)
	fmt.Printf("  Maximum current draw       : %.3f A\n", float64(maxCurrent)/1000.0)
}

func printDCLoad(d []byte) {
	outputNumber := byteAt(d, 0) & 0x0f
	nominalVoltage := int16(le16(d, 1))
	minVoltage := int16(le16(d, 3))
	maxVoltage := int16(le16(d, 5))
	rippleAndNoise := le16(d, 7)
	minCurrent := le16(d, 9)
	maxCurrent := le16(d, 11)

	fmt.Println(" DC Load Record")
	fmt.Printf("  Output Number              : %d\n", outputNumber)
	fmt.Printf("  Nominal voltage            : %.2f V\n", float64(nominalVoltage)/100.0)
	fmt.Printf("  Min voltage allowed        : %.2f V\n", float64(minVoltage)/100.0)
	fmt.Printf("  Max voltage allowed        : %.2f V\n", float64(maxVoltage)/100.0)
	fmt.Printf("  Ripple and noise pk-pk     : %d mV\n", rippleAndNoise)
	fmt.Printf("  Minimum current load       : %.3f A\n", float64(minCurrent)/1000.0)
	fmt.Printf("  Maximum current load       : %.3f A\n", float64(maxCurrent)/1000.0)
}

func printManagementAccess(body []byte, rlen uint32) {
	subtype := byteAt(body, 0)
	var data []byte
	if len(body) > 1 {
		data = body[1:]
	} else {
		data = []byte{}
	}
	datalen := 0
	if rlen > 0 {
		datalen = int(rlen - 1)
	}

	var name string
	var minlen, maxlen int
	var isUUID bool
	switch subtype {
	case 1:
		name, minlen, maxlen, isUUID = "System Management URL", 16, 256, false
	case 2:
		name, minlen, maxlen, isUUID = "System Name", 8, 64, false
	case 3:
		name, minlen, maxlen, isUUID = "System Ping Address", 8, 64, false
	case 4:
		name, minlen, maxlen, isUUID = "Component Management URL", 16, 256, false
	case 5:
		name, minlen, maxlen, isUUID = "Component Name", 8, 64, false
	case 6:
		name, minlen, maxlen, isUUID = "Component Ping Address", 8, 64, false
	case 7:
		name, minlen, maxlen, isUUID = "System Unique ID", 16, 16, true
	default:
		fmt.Fprintf(os.Stderr, "Unsupported subtype 0x%02x found for multi-record area management record\n", subtype)
		return
	}

	if datalen < minlen || datalen > maxlen {
		fmt.Fprintf(os.Stderr, "Wrong data length %d, must be %d < X < %d\n", datalen, minlen, maxlen)
	}

	var value string
	if isUUID {
		value = mc.GuidAutoStr(data)
	} else {
		end := len(data)
		for i, b := range data {
			if b == 0 {
				end = i
				break
			}
		}
		if end > datalen {
			end = datalen
		}
		value = strings.ToValidUTF8(string(data[:end]), "\uFFFD")
	}
	fmt.Printf(" %-22s: %s\n", name, value)
}

// ---------------------------------------------------------------------------
// FRU area reading (`read_fru_area`).
// ---------------------------------------------------------------------------

// readFruArea reads `length` bytes from `offset` in FRU device `id`. The
// returned buffer is always `length` bytes; bytes past the end of the device
// are zero. The second result is false on failure.
func readFruArea(intf ipmicore.Interface, fru *FruInfo, id uint8, offset, length uint32) ([]byte, bool) {
	if offset > fru.size {
		fmt.Fprintf(os.Stderr, "Read FRU Area offset incorrect: %d > %d\n", offset, fru.size)
		return nil, false
	}

	// The output buffer always covers the full requested length (C zero-fills
	// the region past the end of the device).
	buf := make([]byte, length)

	readLen := length
	finish := offset + length
	if finish > fru.size {
		finish = fru.size
		readLen = finish - offset
	}

	if fru.maxReadSize == 0 {
		// Mirror the C `read_fru_area` computation for the lanplus transport:
		// `ipmi_intf_get_max_response_data_size()` is `IPMI_LAN_MAX_RESPONSE_SIZE`
		// (34), minus one byte for the count and one more for the checksum.
		fru.maxReadSize = lanplusMaxResponseSize - 2
		if fru.access {
			fru.maxReadSize &^= 1
		}
	}

	off := offset
	sizeLeft := readLen

	for off < finish {
		addr := off
		if fru.access {
			addr = off >> 1
		}
		want := min(fru.maxReadSize, finish-off)
		if want > 255 {
			want = 255
		}
		req := ipmicore.NewRequest(netfnStorage, getFruData).WithData([]byte{
			id,
			byte(addr & 0xff),
			byte((addr >> 8) & 0xff),
			byte(want),
		})

		rsp, err := intf.SendRecv(req)
		if err != nil {
			fmt.Fprintln(os.Stderr, "FRU Read failed")
			return nil, false
		}

		if !rsp.OK() {
			if fruCCRq2Big(rsp.Ccode) && fru.maxReadSize > fruBlockSz {
				if fru.maxReadSize > fruAreaMaximumBlockSz {
					fru.maxReadSize -= fruBlockSz
				} else {
					fru.maxReadSize -= 1
				}
				logInfo(fmt.Sprintf("Retrying FRU read with request size %d", fru.maxReadSize))
				continue
			}
			fmt.Fprintf(os.Stderr, "FRU Read failed: %s\n", ipmicore.CompletionCodeStr(rsp.Ccode))
			return nil, false
		}

		var count uint32
		if len(rsp.Data) == 0 {
			count = 0
		} else if fru.access {
			count = uint32(rsp.Data[0]) << 1
		} else {
			count = uint32(rsp.Data[0])
		}

		if len(rsp.Data) == 0 || int(count) > len(rsp.Data)-1 || count > sizeLeft {
			fmt.Println(" Not enough buffer size")
			return nil, false
		}

		start := int(off - offset)
		copy(buf[start:start+int(count)], rsp.Data[1:1+int(count)])
		off += count
		sizeLeft -= count

		// A device may report a size larger than it can serve; stop early so
		// the caller can parse whatever was returned.
		if count == 0 && off < finish {
			return buf, true
		}
	}

	if off < finish {
		return nil, false
	}
	return buf, true
}

func fruCCRq2Big(code uint8) bool {
	return code == ipmicore.CC_REQ_DATA_INV_LENGTH ||
		code == ipmicore.CC_REQ_DATA_FIELD_EXCEED ||
		code == ipmicore.CC_CANT_RET_NUM_REQ_BYTES
}

// ---------------------------------------------------------------------------
// FRU string decoding (`get_fru_area_str`).
// ---------------------------------------------------------------------------

var bcdPlus = []byte("0123456789 -.:,_")

// getFruAreaStr decodes one FRU area string field at *offset, advancing *offset
// past it. The second result is false when the field is empty (length 0).
func getFruAreaStr(data []byte, offset *int) (string, bool) {
	off := *offset
	if off >= len(data) {
		return "", false
	}
	first := data[off]
	off++

	typecode := (first & 0xc0) >> 6
	length := int(first & 0x3f)

	var size int
	switch typecode {
	case 0, 1:
		size = length * 2
	case 2:
		size = (length/3 + 1) * 4
	default:
		size = length
	}
	if size < 1 {
		*offset = off
		return "", false
	}

	avail := max(len(data)-off, 0)

	var s string
	switch typecode {
	case 0:
		// Binary/unspecified: hex dump, two characters per byte.
		n := min(avail, length)
		var b strings.Builder
		b.Grow(n * 2)
		for _, v := range data[off : off+n] {
			fmt.Fprintf(&b, "%02x", v)
		}
		s = b.String()
	case 1:
		// BCD plus.
		var b strings.Builder
		b.Grow(size)
		for k := 0; k < size; k++ {
			var nib uint8
			var present bool
			idx := off + k/2
			if k%2 == 1 {
				if idx < len(data) {
					nib = data[idx] & 0x0f
					present = true
				}
			} else {
				if idx < len(data) {
					nib = (data[idx] >> 4) & 0x0f
					present = true
				}
			}
			if !present {
				break
			}
			b.WriteByte(bcdPlus[nib])
		}
		s = b.String()
	case 2:
		// 6-bit ASCII, little-endian packing.
		var b strings.Builder
		b.Grow(size)
		i := 0
		for i < length {
			k := min(length-i, 3)
			var bits uint32
			for j := 0; j < k; j++ {
				if idx := off + i + j; idx < len(data) {
					bits |= uint32(data[idx]) << (8 * uint(j))
				}
			}
			for range 4 {
				b.WriteByte(byte((bits & 0x3f) + 0x20))
				bits >>= 6
			}
			i += 3
		}
		s = b.String()
	default:
		// 8-bit ASCII.
		n := min(avail, length)
		s = strings.ToValidUTF8(string(data[off:off+n]), "\uFFFD")
	}

	*offset = off + length
	return s, true
}

func checksumIsValid(area []byte, length int) bool {
	if length == 0 || length > len(area) {
		return false
	}
	var checksum uint8
	for _, b := range area[:length-1] {
		checksum += b
	}
	return area[length-1] == -checksum
}

// ---------------------------------------------------------------------------
// Small helpers.
// ---------------------------------------------------------------------------

func byteAt(d []byte, i int) uint8 {
	if i < len(d) {
		return d[i]
	}
	return 0
}

func le16(d []byte, i int) uint16 {
	return uint16(byteAt(d, i)) | uint16(byteAt(d, i+1))<<8
}

// formatFruTimestamp formats a board manufacturing date (3 little-endian bytes,
// minutes since 1996) the way `ipmi_timestamp_string()` does for a normal
// timestamp.
func formatFruTimestamp(data []byte, i int) string {
	fruTS := uint32(byteAt(data, i)) |
		uint32(byteAt(data, i+1))<<8 |
		uint32(byteAt(data, i+2))<<16
	if fruTS == 0 {
		return "Unspecified"
	}
	return formatUnixTimestamp(int64(fruTS)*60 + 820_454_400)
}

func formatUnixTimestamp(ts int64) string {
	return time.Unix(ts, 0).Format("Mon Jan _2 15:04:05 2006 MST")
}

func logInfo(msg string) {
	if globals.Verbose() >= 1 {
		fmt.Fprintln(os.Stderr, msg)
	}
}

func logDebug(msg string) {
	if globals.Verbose() >= 2 {
		fmt.Fprintln(os.Stderr, msg)
	}
}

// printbuf is a hex dump in `printbuf()` format (stderr, gated by verbosity).
func printbuf(data []byte, desc string) {
	if len(data) == 0 || globals.Verbose() < 1 {
		return
	}
	fmt.Fprintf(os.Stderr, "%s (%d bytes)\n", desc, len(data))
	for i, b := range data {
		if i%16 == 0 && i != 0 {
			fmt.Fprintln(os.Stderr)
		}
		fmt.Fprintf(os.Stderr, " %02x", b)
	}
	fmt.Fprintln(os.Stderr)
}
