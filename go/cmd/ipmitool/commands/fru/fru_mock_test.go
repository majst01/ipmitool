package fru

import (
	"os"
	"strings"
	"testing"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
	"codeberg.org/ipmitool/ipmitool/internal/mock"
)

// buildBoardFru builds a minimal 40-byte FRU with only a board area.
func buildBoardFru() []byte {
	fru := make([]byte, 40)

	// Common header (offset 0, 8 bytes).
	fru[0] = 0x01 // format version
	fru[3] = 0x01 // board area offset = 1 * 8
	// Checksum for bytes [0..6].
	var hsum uint8
	for _, b := range fru[0:7] {
		hsum += b
	}
	fru[7] = -hsum

	// Board area at offset 8, 32 bytes (4 blocks of 8).
	board := fru[8:40]
	board[0] = 0x01 // board format version
	board[1] = 0x04 // length in 8-byte blocks
	board[2] = 0x00 // language
	// mfg date bytes [3..5] stay 0 -> "Unspecified"
	putField(board, 6, "Acme")
	putField(board, 11, "Board")
	putField(board, 17, "SN1")
	putField(board, 21, "P1")
	putField(board, 24, "F1") // FRU file ID
	board[27] = fruEndOfFields
	// Area checksum is the last byte.
	var bsum uint8
	for _, b := range board[0:31] {
		bsum += b
	}
	board[31] = -bsum
	return fru
}

// putField writes an 8-bit-ASCII FRU field (type/length byte + bytes) at off.
func putField(area []byte, off int, s string) {
	area[off] = 0xc0 | byte(len(s))
	copy(area[off+1:], s)
}

// fruRouter scripts Get FRU Info and Get FRU Data from a fixed FRU image.
func fruRouter(fru []byte) *mock.Router {
	r := mock.NewRouter()
	r.On(ipmicore.NetfnStorage, 0x10, func(req *ipmicore.Request) (*ipmicore.Response, error) {
		return mock.OK(byte(len(fru)), byte(len(fru)>>8), 0x00), nil
	})
	r.On(ipmicore.NetfnStorage, 0x11, func(req *ipmicore.Request) (*ipmicore.Response, error) {
		addr := int(req.Data[1]) | int(req.Data[2])<<8
		want := int(req.Data[3])
		if addr > len(fru) {
			addr = len(fru)
		}
		end := min(addr+want, len(fru))
		chunk := fru[addr:end]
		out := append([]byte{byte(len(chunk))}, chunk...)
		return mock.OKData(out), nil
	})
	return r
}

func TestFruMainNoArgsPrintsBuiltinHeader(t *testing.T) {
	// Get Device ID with the FRU Inventory Device bit (0x08) clear.
	devID := make([]byte, 11)
	devID[5] = 0x00
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnApp, ipmicore.IpmiGetDeviceID, mock.OKData(devID))
	m := r.InterfaceWith()

	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, nil) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if !strings.Contains(stdout, "FRU Device Description : Builtin FRU Device (ID 0)") {
		t.Fatalf("stdout = %q", stdout)
	}
	if len(m.Requests) != 1 || m.Requests[0].Cmd != ipmicore.IpmiGetDeviceID {
		t.Fatalf("unexpected requests: %+v", m.Requests)
	}
}

func TestFruPrintBoard(t *testing.T) {
	fru := buildBoardFru()
	m := fruRouter(fru).InterfaceWith()

	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"print", "1"}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	for _, want := range []string{
		" Board Mfg Date        : Unspecified",
		" Board Mfg             : Acme",
		" Board Product         : Board",
		" Board Serial          : SN1",
		" Board Part Number     : P1",
		" Board Area Checksum   : OK",
	} {
		if !strings.Contains(stdout, want) {
			t.Errorf("stdout missing %q\n%s", want, stdout)
		}
	}
	// First request is Get FRU Info for id 1.
	if m.Requests[0].Cmd != 0x10 || m.Requests[0].Data[0] != 1 {
		t.Fatalf("first request = %+v", m.Requests[0])
	}
}

func TestFruPrintDeviceNotPresent(t *testing.T) {
	r := mock.NewRouter()
	r.Static(ipmicore.NetfnStorage, 0x10, mock.CCode(0xc7)) // request data length invalid
	m := r.InterfaceWith()
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"print", "1"}) })
	if code != -1 {
		t.Fatalf("exit code = %d, want -1", code)
	}
	if !strings.Contains(stdout, "Device not present (Request data length invalid)") {
		t.Fatalf("stdout = %q", stdout)
	}
}

func TestFruReadToFile(t *testing.T) {
	fru := buildBoardFru()
	m := fruRouter(fru).InterfaceWith()

	dir := t.TempDir()
	path := dir + "/fru.bin"
	var code int
	stdout, _ := mock.Capture(func() { code = Main(m, []string{"read", "1", path}) })
	if code != 0 {
		t.Fatalf("exit code = %d, want 0", code)
	}
	if !strings.Contains(stdout, "Fru Size         : 40 bytes") || !strings.Contains(stdout, "Done") {
		t.Fatalf("stdout = %q", stdout)
	}
	got, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("read back: %v", err)
	}
	if len(got) != len(fru) {
		t.Fatalf("file length = %d, want %d", len(got), len(fru))
	}
	for i := range fru {
		if got[i] != fru[i] {
			t.Fatalf("byte %d = %#x, want %#x", i, got[i], fru[i])
		}
	}
}

func TestFruWriteNotImplemented(t *testing.T) {
	m := mock.New(nil)
	var code int
	_, stderr := mock.Capture(func() { code = Main(m, []string{"write"}) })
	if code != -1 || !strings.Contains(stderr, "fru write: not implemented in this build") {
		t.Fatalf("code=%d stderr=%q", code, stderr)
	}
}
