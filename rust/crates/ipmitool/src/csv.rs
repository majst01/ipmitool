//! Process-wide CSV output flag, mirroring the C global `csv_output` set by
//! `-c`. Commands that offer a comma-separated mode (currently `user`) read it
//! through here.

use std::sync::atomic::{AtomicBool, Ordering};

static CSV: AtomicBool = AtomicBool::new(false);

/// Record whether `-c` was given on the command line.
pub fn set(enabled: bool) {
    CSV.store(enabled, Ordering::Relaxed);
}

/// Whether comma-separated output was requested.
pub fn enabled() -> bool {
    CSV.load(Ordering::Relaxed)
}
