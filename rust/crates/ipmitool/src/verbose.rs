//! Process-wide verbosity level, mirroring the C global `verbose` (incremented
//! once per `-v`). Commands that change their output on `-v` (currently `fru`)
//! read it through here.

use std::sync::atomic::{AtomicU32, Ordering};

static VERBOSE: AtomicU32 = AtomicU32::new(0);

/// Record the verbosity level collected from the command line.
pub fn set(level: u32) {
    VERBOSE.store(level, Ordering::Relaxed);
}

/// Current verbosity level (`0` when `-v` was not given).
pub fn level() -> u32 {
    VERBOSE.load(Ordering::Relaxed)
}
