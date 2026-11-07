// Package globals holds process-wide output flags mirroring the C globals
// `verbose` and `csv_output` (set by `-v` and `-c`).
package globals

import "sync/atomic"

var verbose atomic.Uint32
var csvEnabled atomic.Bool

// SetVerbose records the verbosity level collected from the command line.
func SetVerbose(level uint32) { verbose.Store(level) }

// Verbose returns the current verbosity level (0 when -v was not given).
func Verbose() uint32 { return verbose.Load() }

// SetCSV records whether -c was given on the command line.
func SetCSV(enabled bool) { csvEnabled.Store(enabled) }

// CSV reports whether comma-separated output was requested.
func CSV() bool { return csvEnabled.Load() }
