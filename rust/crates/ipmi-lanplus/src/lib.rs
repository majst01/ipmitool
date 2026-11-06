//! IPMI v2.0 / RMCP+ LAN transport.
//!
//! This crate implements the `lanplus` interface: UDP framing, the RAKP
//! session-establishment handshake and authenticated/encrypted IPMI message
//! exchange.  It is the Rust counterpart to `src/plugins/lanplus/`.

pub mod crypto;
pub mod message;
pub mod session;
pub mod transport;

pub use transport::{LanPlus, IPMI_LANPLUS_PORT};
