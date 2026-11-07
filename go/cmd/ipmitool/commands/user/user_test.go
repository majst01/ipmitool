package user

// Ported from the Rust `commands/user.rs` unit tests.

import (
	"testing"

	"codeberg.org/ipmitool/ipmitool/internal/mock"
)

func TestUIDMasksToSixBits(t *testing.T) {
	if uid(0x41) != 1 || uid(0x3F) != 0x3F || uid(0x00) != 0x00 {
		t.Fatalf("uid masking wrong: %#x %#x %#x", uid(0x41), uid(0x3F), uid(0x00))
	}
}

func TestStrncmpMatchesCSemantics(t *testing.T) {
	if !strncmpEq("abc", "abc", 3) {
		t.Error("identical strings should match")
	}
	if !strncmpEq("abc", "abd", 2) {
		t.Error("prefix comparison should match")
	}
	if strncmpEq("abc", "abd", 3) {
		t.Error("differing third byte should not match")
	}
	// Shorter left side compares against NUL past the end.
	if strncmpEq("ab", "abc", 3) {
		t.Error("NUL vs 'c' should not match")
	}
}

func TestValidatorsAcceptAndRejectExpectedRanges(t *testing.T) {
	// Expected failures print diagnostics; capture them.
	mock.Capture(func() {
		if id, ok := isUserID("1"); !ok || id != 1 {
			t.Errorf("isUserID(1) = %d,%v", id, ok)
		}
		if id, ok := isUserID("63"); !ok || id != 63 {
			t.Errorf("isUserID(63) = %d,%v", id, ok)
		}
		if _, ok := isUserID("0"); ok {
			t.Error("isUserID(0) should fail")
		}
		if _, ok := isUserID("64"); ok {
			t.Error("isUserID(64) should fail")
		}
		if c, ok := isChannelNum("0xb"); !ok || c != 0xb {
			t.Errorf("isChannelNum(0xb) = %d,%v", c, ok)
		}
		if c, ok := isChannelNum("0xe"); !ok || c != 0xe {
			t.Errorf("isChannelNum(0xe) = %d,%v", c, ok)
		}
		if _, ok := isChannelNum("0xc"); ok {
			t.Error("isChannelNum(0xc) should fail")
		}
		if p, ok := isPrivLimit("4"); !ok || p != 4 {
			t.Errorf("isPrivLimit(4) = %d,%v", p, ok)
		}
		if p, ok := isPrivLimit("0xf"); !ok || p != 0xf {
			t.Errorf("isPrivLimit(0xf) = %d,%v", p, ok)
		}
		if _, ok := isPrivLimit("6"); ok {
			t.Error("isPrivLimit(6) should fail")
		}
	})
}

func TestPasswordFlagByteEncodesLengthAndUID(t *testing.T) {
	// 16-byte password: high bit clear. 20-byte password: high bit set.
	if uid(5) != 5 {
		t.Fatalf("uid(5) = %d", uid(5))
	}
	if 0x80|uid(5) != 0x85 {
		t.Fatalf("20-byte flag for uid 5 = %#x", 0x80|uid(5))
	}
	if 0x80|uid(0x41) != 0x81 {
		t.Fatalf("20-byte flag for uid 0x41 = %#x", 0x80|uid(0x41))
	}
}
