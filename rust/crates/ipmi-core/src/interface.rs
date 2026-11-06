//! The transport interface abstraction. This is the Rust equivalent of
//! `struct ipmi_intf` in the C code: a small vtable plus session parameters.

use crate::error::Result;
use crate::types::{Request, Response, SessionParams};

/// A transport capable of carrying IPMI requests to a BMC.
pub trait Interface {
    /// Short interface name (e.g. `"lanplus"`).
    fn name(&self) -> &str;

    /// Open the transport and, where applicable, establish a session.
    fn open(&mut self) -> Result<()>;

    /// Close the transport, terminating any active session.
    fn close(&mut self);

    /// Whether the transport is currently open.
    fn is_open(&self) -> bool;

    /// Send a request and wait for the corresponding response.
    fn sendrecv(&mut self, req: &Request) -> Result<Response>;

    /// Immutable access to the configured session parameters.
    fn session_params(&self) -> &SessionParams;

    /// Mutable access to the configured session parameters.
    fn session_params_mut(&mut self) -> &mut SessionParams;

    /// Destination LUN for raw commands.
    fn target_lun(&self) -> u8 {
        self.session_params().target_lun
    }

    /// Destination channel for raw commands.
    fn target_channel(&self) -> u8 {
        self.session_params().target_channel
    }
}
