package ipmicore

import (
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

// IANA PEN used for the "Unknown" manufacturer, as per `IPMI_OEM_UNKNOWN`.
const oemUnknown uint32 = 0

// IANA PEN used for the "Unspecified" manufacturer (`IPMI_OEM_RESERVED`).
const oemReserved uint32 = 0x000F_FFFF

// Sentinel enterprise number for the built-in debug entry (`IPMI_OEM_DEBUG`).
const oemDebug uint32 = 0x00FF_FFFE

// Debug manufacturer name, from `ipmi_oem_info_tail`.
const oemDebugName = "A Debug Assisting Company, Ltd."

// UnknownValStr formats an unmatched value the way C's `unknown_val_str()` does.
func UnknownValStr(val uint32) string {
	return fmt.Sprintf("Unknown (0x%02X)", val)
}

// OemRegistry is a runtime registry of IANA Private Enterprise Numbers mapped
// to names.
type OemRegistry struct {
	names map[uint32]string
}

// NewEmptyOemRegistry returns an empty registry, used when the PEN file cannot
// be read. Manufacturer lookups still resolve the built-in head and tail
// entries.
func NewEmptyOemRegistry() *OemRegistry {
	return &OemRegistry{names: make(map[uint32]string)}
}

// LoadOemRegistry loads the registry from the first readable IANA PEN file,
// mirroring `oem_info_list_load()`.
func LoadOemRegistry() *OemRegistry {
	names := make(map[uint32]string)
	for _, path := range oemSearchPaths() {
		if text, err := os.ReadFile(path); err == nil {
			parseRegistry(string(text), names)
			break
		}
	}
	return &OemRegistry{names: names}
}

// Len reports the number of dynamically loaded registry entries.
func (r *OemRegistry) Len() int { return len(r.names) }

// IsEmpty reports whether the registry has no dynamically loaded entries.
func (r *OemRegistry) IsEmpty() bool { return len(r.names) == 0 }

// ManufacturerName resolves a manufacturer id to a name, matching
// `OEM_MFG_STRING()`.
func (r *OemRegistry) ManufacturerName(id uint32) string {
	switch id {
	case oemUnknown:
		return "Unknown"
	case oemReserved:
		return "Unspecified"
	default:
		if name, ok := r.names[id]; ok {
			return name
		}
		if id == oemDebug {
			return oemDebugName
		}
		return UnknownValStr(id)
	}
}

// ProductName resolves a product name from the manufacturer and product id,
// matching `OEM_PROD_STRING()`. As in C, PICMG entries match every
// manufacturer.
func (r *OemRegistry) ProductName(oem uint32, product uint16) string {
	for _, entry := range OemProductInfo {
		if (entry.Oem == oem || entry.Oem == IPMIOemPicmg) && entry.Product == product {
			return entry.Name
		}
	}
	return UnknownValStr(uint32(product))
}

// oemSearchPaths returns the candidate PEN files, in precedence order.
func oemSearchPaths() []string {
	fileNames := []string{"enterprise-numbers", "enterprise-numbers.txt"}
	var dirs []string
	if home, ok := os.LookupEnv("HOME"); ok {
		dirs = append(dirs, filepath.Join(home, ".local/usr/share/misc"))
	}
	dirs = append(dirs, "/usr/share/misc")
	dirs = append(dirs, "/usr/local/share/misc")

	var paths []string
	for _, dir := range dirs {
		for _, name := range fileNames {
			paths = append(paths, filepath.Join(dir, name))
		}
	}
	return paths
}

// parseRegistry parses the PEN registry lines into names. Entries read later in
// the file win on duplicates, as they end up earlier in the C lookup array.
func parseRegistry(text string, names map[uint32]string) {
	lines := strings.Split(text, "\n")
	i := 0
	for i < len(lines) {
		line := lines[i]
		if line == "" || line[0] < '0' || line[0] > '9' {
			i++
			continue
		}

		digitEnd := len(line)
		for j := 0; j < len(line); j++ {
			if line[j] < '0' || line[j] > '9' {
				digitEnd = j
				break
			}
		}
		pen, penErr := strconv.ParseUint(line[:digitEnd], 10, 64)
		i++

		// The organization name is expected on the immediately following line
		// and that line is consumed whether or not it is valid.
		if i >= len(lines) {
			break
		}
		org := lines[i]
		i++

		if penErr == nil {
			if name, ok := orgName(org); ok && pen <= uint64(^uint32(0)) {
				names[uint32(pen)] = name
			}
		}
	}
}

// orgName extracts the organization name from an indented registry line. It
// requires exactly two leading spaces, as C's `count_bytes(line, ' ') == 2`
// check.
func orgName(line string) (string, bool) {
	bytes := []byte(line)
	if len(bytes) < 3 {
		return "", false
	}
	if bytes[0] != ' ' || bytes[1] != ' ' || bytes[2] == ' ' {
		return "", false
	}
	return line[2:], true
}
