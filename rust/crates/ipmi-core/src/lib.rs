//! Core types, constants and the transport interface abstraction shared by the
//! `ipmi-lanplus` transport and the `ipmitool` CLI.
//!
//! This crate is the Rust counterpart to `include/ipmitool/ipmi.h` and
//! `ipmi_intf.h` from the original C implementation.

pub mod cc;
pub mod checksum;
pub mod consts;
pub mod error;
pub mod interface;
pub mod types;
pub mod values;

pub use error::{Error, Result};
pub use interface::Interface;
pub use types::{CipherSuite, Request, Response, SessionParams};
