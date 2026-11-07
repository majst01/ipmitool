// Package lanplus implements the IPMI v2.0 / RMCP+ LAN transport.
//
// This package is the Go counterpart to the Rust `ipmi-lanplus` crate, which
// in turn mirrors `src/plugins/lanplus/`. It implements the `lanplus`
// interface: UDP framing, the RAKP session-establishment handshake and
// authenticated/encrypted IPMI message exchange.
package lanplus

// The public surface of the Rust crate (`pub use transport::{LanPlus,
// IPMI_LANPLUS_PORT};`) is already exported from this package as the `LanPlus`
// type and the `IPMILanPlusPort` constant; no re-export shim is required in Go.
