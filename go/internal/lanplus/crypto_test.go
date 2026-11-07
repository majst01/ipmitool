package lanplus

import (
	"bytes"
	"testing"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
)

func TestAESCBCRoundTrip(t *testing.T) {
	key := bytes.Repeat([]byte{0x11}, 16)
	iv := bytes.Repeat([]byte{0x22}, 16)
	plain := make([]byte, 32)
	for i := range plain {
		plain[i] = byte(i)
	}
	ct := aesCBCEncrypt(key, iv, plain)
	if bytes.Equal(ct, plain) {
		t.Fatal("ciphertext equals plaintext")
	}
	if got := aesCBCDecrypt(key, iv, ct); !bytes.Equal(got, plain) {
		t.Fatalf("round trip mismatch: %x", got)
	}
}

func TestHMACSHA1HasExpectedLength(t *testing.T) {
	mac := hmacIntegrity(ipmicore.IntegrityHmacSha196, []byte("key"), []byte("data"))
	if len(mac) != 20 { // full digest; caller truncates to 12
		t.Fatalf("got %d", len(mac))
	}
}

func TestHMACSHA256HasExpectedLength(t *testing.T) {
	mac := hmacIntegrity(ipmicore.IntegrityHmacSha256128, []byte("key"), []byte("data"))
	if len(mac) != 32 {
		t.Fatalf("got %d", len(mac))
	}
}
