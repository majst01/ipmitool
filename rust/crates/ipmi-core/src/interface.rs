//! The transport interface abstraction. This is the Rust equivalent of
//! `struct ipmi_intf` in the C code: a small vtable plus session parameters.

use crate::error::{Error, Result};
use crate::sol::{SolActivateRequest, SolActivateResponse, SolPacket};
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

    /// Activate an SOL payload. Only transports that support SOL implement
    /// this; the default reports the feature as unsupported.
    fn sol_activate(&mut self, _req: &SolActivateRequest) -> Result<SolActivateResponse> {
        Err(Error::Unsupported(
            "Serial-over-LAN is not supported by this interface".into(),
        ))
    }

    /// Deactivate an SOL payload.
    fn sol_deactivate(&mut self, _instance: u8) -> Result<()> {
        Err(Error::Unsupported(
            "Serial-over-LAN is not supported by this interface".into(),
        ))
    }

    /// Send an SOL packet and return the acknowledgement.
    fn sol_send(&mut self, _pkt: &SolPacket) -> Result<SolPacket> {
        Err(Error::Unsupported(
            "Serial-over-LAN is not supported by this interface".into(),
        ))
    }

    /// Receive an SOL packet.
    fn sol_recv(&mut self) -> Result<SolPacket> {
        Err(Error::Unsupported(
            "Serial-over-LAN is not supported by this interface".into(),
        ))
    }

    /// Raw file descriptor for the underlying transport, used to poll for
    /// inbound data while in an interactive SOL session.
    fn raw_fd(&self) -> Option<i32> {
        None
    }
}
