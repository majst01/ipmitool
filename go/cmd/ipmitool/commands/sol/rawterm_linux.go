//go:build linux

package sol

import (
	"syscall"
	"unsafe"
)

// pollIn is POLLIN.
const pollIn int16 = 0x001

// tcsadrain is TCSADRAIN (TCSETSW) on Linux; it waits for output to drain
// before applying the terminal attributes, matching libc's TCSADRAIN.
const tcsadrain uintptr = 0x5403

// pollFd mirrors Linux `struct pollfd`.
type pollFd struct {
	fd      int32
	events  int16
	revents int16
}

// rawTerminal is an RAII guard that switches stdin into raw mode and restores
// it on return.
type rawTerminal struct {
	saved syscall.Termios
}

func ioctlTermios(req uintptr, tio *syscall.Termios) error {
	_, _, errno := syscall.Syscall(syscall.SYS_IOCTL, uintptr(syscall.Stdin), req, uintptr(unsafe.Pointer(tio)))
	if errno != 0 {
		return errno
	}
	return nil
}

// enterRawTerminal mirrors `RawTerminal::enter`: it puts stdin into raw mode,
// returning an error (rather than panicking) when stdin is not a terminal.
func enterRawTerminal() (*rawTerminal, error) {
	var tio syscall.Termios
	if err := ioctlTermios(syscall.TCGETS, &tio); err != nil {
		return nil, err
	}
	saved := tio
	tio.Iflag |= syscall.IGNPAR
	tio.Iflag &^= syscall.ISTRIP |
		syscall.INLCR |
		syscall.IGNCR |
		syscall.ICRNL |
		syscall.IXON |
		syscall.IXANY |
		syscall.IXOFF
	tio.Lflag &^= syscall.ISIG |
		syscall.ICANON |
		syscall.ECHO |
		syscall.ECHOE |
		syscall.ECHOK |
		syscall.ECHONL |
		syscall.IEXTEN
	tio.Oflag &^= syscall.OPOST
	tio.Cc[syscall.VMIN] = 1
	tio.Cc[syscall.VTIME] = 0
	if err := ioctlTermios(tcsadrain, &tio); err != nil {
		return nil, err
	}
	return &rawTerminal{saved: saved}, nil
}

func (t *rawTerminal) restore() {
	_ = ioctlTermios(tcsadrain, &t.saved)
}

// poll waits for events on the given file descriptors, like libc poll(2).
func poll(fds []pollFd, timeoutMs int) (int, error) {
	n, _, errno := syscall.Syscall(syscall.SYS_POLL, uintptr(unsafe.Pointer(&fds[0])), uintptr(len(fds)), uintptr(timeoutMs))
	if errno != 0 {
		return 0, errno
	}
	return int(n), nil
}
