package lanplus

// Cryptographic primitives used by RMCP+: HMAC dispatch, AES-128-CBC with the
// IPMI confidentiality padding, and random number generation.

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/hmac"
	"crypto/md5"
	"crypto/rand"
	"crypto/sha1"
	"crypto/sha256"

	"codeberg.org/ipmitool/ipmitool/internal/ipmicore"
)

// RandomBytes generates n cryptographically random bytes.
func RandomBytes(n int) []byte {
	buf := make([]byte, n)
	if _, err := rand.Read(buf); err != nil {
		panic(err)
	}
	return buf
}

// hmacAuth is HMAC using the authentication algorithm negotiated during RAKP.
func hmacAuth(alg uint8, key, data []byte) []byte {
	switch alg {
	case ipmicore.AuthRakpHmacSha1:
		m := hmac.New(sha1.New, key)
		m.Write(data)
		return m.Sum(nil)
	case ipmicore.AuthRakpHmacMd5:
		m := hmac.New(md5.New, key)
		m.Write(data)
		return m.Sum(nil)
	case ipmicore.AuthRakpHmacSha256:
		m := hmac.New(sha256.New, key)
		m.Write(data)
		return m.Sum(nil)
	default:
		return nil
	}
}

// hmacIntegrity is HMAC using the integrity algorithm negotiated in the
// open-session exchange.
func hmacIntegrity(alg uint8, key, data []byte) []byte {
	switch alg {
	case ipmicore.IntegrityHmacSha196:
		m := hmac.New(sha1.New, key)
		m.Write(data)
		return m.Sum(nil)
	case ipmicore.IntegrityHmacMd5128:
		m := hmac.New(md5.New, key)
		m.Write(data)
		return m.Sum(nil)
	case ipmicore.IntegrityMd5128:
		// Plain MD5 over the packet, no key (IPMI spec table 13-18).
		_ = key
		sum := md5.Sum(data)
		return sum[:]
	case ipmicore.IntegrityHmacSha256128:
		m := hmac.New(sha256.New, key)
		m.Write(data)
		return m.Sum(nil)
	default:
		return nil
	}
}

// aesCBCEncrypt performs AES-128-CBC encryption with no additional padding.
// data must be a multiple of 16 bytes; key/iv must be 16 bytes.
func aesCBCEncrypt(key, iv, data []byte) []byte {
	if len(data)%16 != 0 {
		panic("aesCBCEncrypt: input is not block aligned")
	}
	block, err := aes.NewCipher(key[:16])
	if err != nil {
		panic(err)
	}
	out := make([]byte, len(data))
	enc := cipher.NewCBCEncrypter(block, iv[:16])
	enc.CryptBlocks(out, data)
	return out
}

// aesCBCDecrypt performs AES-128-CBC decryption with no additional padding.
func aesCBCDecrypt(key, iv, data []byte) []byte {
	if len(data)%16 != 0 {
		panic("aesCBCDecrypt: input is not block aligned")
	}
	block, err := aes.NewCipher(key[:16])
	if err != nil {
		panic(err)
	}
	out := make([]byte, len(data))
	dec := cipher.NewCBCDecrypter(block, iv[:16])
	dec.CryptBlocks(out, data)
	return out
}
