//! Process output indirection.
//!
//! Commands print through the [`outln!`]/[`out!`]/[`errln!`]/[`err!`] macros,
//! which write here. By default output goes to the real stdout/stderr; tests
//! install a thread-local sink with [`capture`] to intercept it. This keeps the
//! command code identical to the C/Rust behaviour while making it testable
//! without a BMC.

use std::cell::RefCell;
use std::io::Write;

thread_local! {
    static OUT_SINK: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
    static ERR_SINK: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
}

/// Write `s` to stdout, or to the active capture sink.
pub fn write_out(s: &str) {
    if sink_push(&OUT_SINK, s) {
        return;
    }
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let _ = lock.write_all(s.as_bytes());
    let _ = lock.flush();
}

/// Write `s` to stderr, or to the active capture sink.
pub fn write_err(s: &str) {
    if sink_push(&ERR_SINK, s) {
        return;
    }
    let stderr = std::io::stderr();
    let mut lock = stderr.lock();
    let _ = lock.write_all(s.as_bytes());
    let _ = lock.flush();
}

fn sink_push(cell: &'static std::thread::LocalKey<RefCell<Option<Vec<u8>>>>, s: &str) -> bool {
    cell.with(|c| {
        let mut b = c.borrow_mut();
        if let Some(v) = b.as_mut() {
            v.extend_from_slice(s.as_bytes());
            true
        } else {
            false
        }
    })
}

/// Resets the thread-local sinks when dropped.
#[cfg(test)]
struct ResetGuard;

#[cfg(test)]
impl Drop for ResetGuard {
    fn drop(&mut self) {
        OUT_SINK.with(|c| *c.borrow_mut() = None);
        ERR_SINK.with(|c| *c.borrow_mut() = None);
    }
}

/// Capture everything written via the output macros while `f` runs, returning
/// its value together with the captured stdout and stderr.
#[cfg(test)]
pub fn capture<T>(f: impl FnOnce() -> T) -> (T, String, String) {
    OUT_SINK.with(|c| *c.borrow_mut() = Some(Vec::new()));
    ERR_SINK.with(|c| *c.borrow_mut() = Some(Vec::new()));
    let guard = ResetGuard;

    let value = f();

    let out = OUT_SINK.with(|c| c.borrow_mut().take()).unwrap_or_default();
    let err = ERR_SINK.with(|c| c.borrow_mut().take()).unwrap_or_default();
    drop(guard);
    (
        value,
        String::from_utf8_lossy(&out).into_owned(),
        String::from_utf8_lossy(&err).into_owned(),
    )
}

/// Print to the output sink, appending a newline (mirrors `println!`).
#[macro_export]
macro_rules! outln {
    () => {{ $crate::output::write_out("\n"); }};
    ($($arg:tt)*) => {{
        $crate::output::write_out(&format!($($arg)*));
        $crate::output::write_out("\n");
    }};
}

/// Print to the output sink without a trailing newline (mirrors `print!`).
#[macro_export]
macro_rules! out {
    ($($arg:tt)*) => {{ $crate::output::write_out(&format!($($arg)*)); }};
}

/// Print to the error sink, appending a newline (mirrors `eprintln!`).
#[macro_export]
macro_rules! errln {
    () => {{ $crate::output::write_err("\n"); }};
    ($($arg:tt)*) => {{
        $crate::output::write_err(&format!($($arg)*));
        $crate::output::write_err("\n");
    }};
}

/// Print to the error sink without a trailing newline (mirrors `eprint!`).
#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {{ $crate::output::write_err(&format!($($arg)*)); }};
}
