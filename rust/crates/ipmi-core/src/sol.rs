//! Serial-over-LAN (SOL) packet types and configuration helpers.
//!
//! This is the Rust counterpart to the SOL payload handling in
//! `src/plugins/lanplus/lanplus.c` and `lib/ipmi_sol.c`.

use crate::error::{Error, Result};

// ---------------------------------------------------------------------------
// Application commands used to manage SOL.
// ---------------------------------------------------------------------------
pub const IPMI_ACTIVATE_PAYLOAD: u8 = 0x48;
pub const IPMI_DEACTIVATE_PAYLOAD: u8 = 0x49;
pub const IPMI_SET_USER_PAYLOAD_ACCESS: u8 = 0x4c;
pub const IPMI_GET_USER_PAYLOAD_ACCESS: u8 = 0x4d;

// ---------------------------------------------------------------------------
// Transport commands to read/write the SOL configuration.
// ---------------------------------------------------------------------------
pub const IPMI_SET_SOL_CONFIG_PARAMETERS: u8 = 0x21;
pub const IPMI_GET_SOL_CONFIG_PARAMETERS: u8 = 0x22;

// ---------------------------------------------------------------------------
// SOL configuration parameter selectors.
// ---------------------------------------------------------------------------
pub const PARAM_SET_IN_PROGRESS: u8 = 0x00;
pub const PARAM_SOL_ENABLE: u8 = 0x01;
pub const PARAM_SOL_AUTHENTICATION: u8 = 0x02;
pub const PARAM_CHARACTER_INTERVAL: u8 = 0x03;
pub const PARAM_SOL_RETRY: u8 = 0x04;
pub const PARAM_NON_VOLATILE_BIT_RATE: u8 = 0x05;
pub const PARAM_VOLATILE_BIT_RATE: u8 = 0x06;
pub const PARAM_PAYLOAD_CHANNEL: u8 = 0x07;
pub const PARAM_PAYLOAD_PORT: u8 = 0x08;

/// Description used in diagnostics, matching `sol_parameter_vals`.
pub fn parameter_name(param: u8) -> &'static str {
    match param {
        PARAM_SET_IN_PROGRESS => "Set In Progress (0)",
        PARAM_SOL_ENABLE => "Enable (1)",
        PARAM_SOL_AUTHENTICATION => "Authentication (2)",
        PARAM_CHARACTER_INTERVAL => "Character Interval (3)",
        PARAM_SOL_RETRY => "Retry (4)",
        PARAM_NON_VOLATILE_BIT_RATE => "Nonvolatile Bitrate (5)",
        PARAM_VOLATILE_BIT_RATE => "Volatile Bitrate (6)",
        PARAM_PAYLOAD_CHANNEL => "Payload Channel (7)",
        PARAM_PAYLOAD_PORT => "Payload Port (8)",
        _ => "Unknown",
    }
}

/// Set-in-progress values, matching `ipmi_set_in_progress_vals`.
pub fn set_in_progress_str(val: u8) -> &'static str {
    match val & 0x03 {
        0x00 => "set-complete",
        0x01 => "set-in-progress",
        0x02 => "commit-write",
        _ => "Unknown",
    }
}

/// Bit-rate values, matching `ipmi_bit_rate_vals`.
pub fn bit_rate_str(val: u8) -> &'static str {
    match val & 0x0f {
        0x00 => "IPMI-Over-Serial-Setting",
        0x06 => "9.6",
        0x07 => "19.2",
        0x08 => "38.4",
        0x09 => "57.6",
        0x0a => "115.2",
        _ => "Unknown",
    }
}

/// SOL configuration parameters returned by
/// `Get SOL Configuration Parameters`.
#[derive(Debug, Default, Clone)]
pub struct SolConfigParams {
    pub set_in_progress: u8,
    pub enabled: bool,
    pub force_encryption: bool,
    pub force_authentication: bool,
    pub privilege_level: u8,
    pub character_accumulate_level: u8,
    pub character_send_threshold: u8,
    pub retry_count: u8,
    pub retry_interval: u8,
    pub non_volatile_bit_rate: u8,
    pub volatile_bit_rate: u8,
    pub payload_channel: u8,
    pub payload_port: u16,
}

/// Request parameters for `Activate Payload`.
#[derive(Debug, Clone, Copy)]
pub struct SolActivateRequest {
    pub instance: u8,
    pub encryption: bool,
    pub authentication: bool,
    /// Whether the BMC should assert CTS (`IPMI_SOL_BMC_ASSERTS_CTS_*`).
    pub assert_cts: bool,
}

impl Default for SolActivateRequest {
    fn default() -> Self {
        Self {
            instance: 1,
            encryption: true,
            authentication: true,
            assert_cts: false,
        }
    }
}

/// Response to `Activate Payload`.
#[derive(Debug, Clone, Copy)]
pub struct SolActivateResponse {
    pub inbound_payload_size: u16,
    pub outbound_payload_size: u16,
    pub port: u16,
}

impl SolActivateResponse {
    /// Parse the 12-byte `Activate Payload` response payload.
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() != 12 {
            return Err(Error::Protocol(format!(
                "Unexpected data length ({}) received in payload activation response",
                data.len()
            )));
        }
        Ok(Self {
            inbound_payload_size: u16::from_le_bytes([data[6], data[7]]),
            outbound_payload_size: u16::from_le_bytes([data[8], data[9]]),
            port: u16::from_le_bytes([data[10], data[11]]),
        })
    }
}

// ---------------------------------------------------------------------------
// SOL payload packet.
// ---------------------------------------------------------------------------

/// Send-side flag: assert RING/WOR.
pub const SOL_FLAG_ASSERT_RING_WOR: u8 = 0x20;
/// Send-side flag: generate BREAK.
pub const SOL_FLAG_GENERATE_BREAK: u8 = 0x10;
/// Send-side flag: deassert CTS.
pub const SOL_FLAG_DEASSERT_CTS: u8 = 0x08;
/// Send-side flag: deassert DCD/DSR.
pub const SOL_FLAG_DEASSERT_DCD_DSR: u8 = 0x04;
/// Send-side flag: flush inbound data.
pub const SOL_FLAG_FLUSH_INBOUND: u8 = 0x02;
/// Send-side flag: flush outbound data.
pub const SOL_FLAG_FLUSH_OUTBOUND: u8 = 0x01;

/// Receive-side flag: NACK.
pub const SOL_FLAG_NACK: u8 = 0x40;
/// Receive-side flag: transfer unavailable.
pub const SOL_FLAG_TRANSFER_UNAVAILABLE: u8 = 0x20;
/// Receive-side flag: SOL inactive.
pub const SOL_FLAG_SOL_INACTIVE: u8 = 0x10;
/// Receive-side flag: transmit overrun.
pub const SOL_FLAG_TRANSMIT_OVERRUN: u8 = 0x08;
/// Receive-side flag: break detected.
pub const SOL_FLAG_BREAK_DETECTED: u8 = 0x04;

/// A single Serial-over-LAN payload packet.
#[derive(Debug, Default, Clone)]
pub struct SolPacket {
    /// Packet sequence number (4 bits on the wire).
    pub sequence: u8,
    /// Acknowledged packet number (4 bits on the wire).
    pub acked: u8,
    /// Accepted character count.
    pub accepted: u8,
    /// Raw flag byte; interpret with the `SOL_FLAG_*` constants.
    pub flags: u8,
    /// Character data carried by this packet.
    pub data: Vec<u8>,
}

impl SolPacket {
    /// Build an empty ACK for the given sequence number.
    pub fn ack(sequence: u8, accepted: u8) -> Self {
        Self {
            sequence: 0,
            acked: sequence,
            accepted,
            flags: 0,
            data: Vec::new(),
        }
    }

    /// Build a packet carrying character data.
    pub fn data(data: Vec<u8>) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }

    /// Build a packet requesting a break.
    pub fn generate_break() -> Self {
        Self {
            flags: SOL_FLAG_GENERATE_BREAK,
            ..Default::default()
        }
    }

    pub fn is_nack(&self) -> bool {
        self.flags & SOL_FLAG_NACK != 0
    }
    pub fn transfer_unavailable(&self) -> bool {
        self.flags & SOL_FLAG_TRANSFER_UNAVAILABLE != 0
    }
    pub fn sol_inactive(&self) -> bool {
        self.flags & SOL_FLAG_SOL_INACTIVE != 0
    }
    pub fn transmit_overrun(&self) -> bool {
        self.flags & SOL_FLAG_TRANSMIT_OVERRUN != 0
    }
    pub fn break_detected(&self) -> bool {
        self.flags & SOL_FLAG_BREAK_DETECTED != 0
    }

    /// Serialize for transmission (`getSolPayloadWireRep`).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + self.data.len());
        out.push(self.sequence & 0x0f);
        out.push(self.acked & 0x0f);
        out.push(self.accepted);
        out.push(self.flags);
        out.extend_from_slice(&self.data);
        out
    }

    /// Parse a decrypted SOL payload (`read_sol_packet`).
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }
        Some(Self {
            sequence: data[0] & 0x0f,
            acked: data[1] & 0x0f,
            accepted: data[2],
            flags: data[3],
            data: data[4..].to_vec(),
        })
    }

    /// Whether this packet acknowledges the given sequence number.
    pub fn acks(&self, sequence: u8) -> bool {
        self.acked == (sequence & 0x0f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sol_packet_round_trip() {
        let pkt = SolPacket {
            sequence: 3,
            acked: 2,
            accepted: 1,
            flags: SOL_FLAG_GENERATE_BREAK,
            data: vec![b'h', b'i'],
        };
        let bytes = pkt.to_bytes();
        assert_eq!(bytes, vec![3, 2, 1, 0x10, b'h', b'i']);
        let back = SolPacket::from_bytes(&bytes).unwrap();
        assert_eq!(back.sequence, 3);
        assert_eq!(back.acked, 2);
        assert_eq!(back.accepted, 1);
        assert_eq!(back.data, vec![b'h', b'i']);
    }

    #[test]
    fn sol_sequence_numbers_are_masked() {
        let pkt = SolPacket {
            sequence: 0xff,
            acked: 0xff,
            ..Default::default()
        };
        let bytes = pkt.to_bytes();
        assert_eq!(bytes[0], 0x0f);
        assert_eq!(bytes[1], 0x0f);
    }

    #[test]
    fn sol_recv_flags() {
        let pkt = SolPacket::from_bytes(&[1, 0, 4, 0x74, 0xaa]).unwrap();
        assert!(pkt.is_nack());
        assert!(pkt.transfer_unavailable());
        assert!(pkt.sol_inactive());
        assert!(pkt.break_detected());
        assert!(!pkt.transmit_overrun());
        assert_eq!(pkt.data, vec![0xaa]);
    }

    #[test]
    fn sol_ack_matches_sequence() {
        let ack = SolPacket::ack(5, 10);
        assert!(ack.acks(5));
        assert!(!ack.acks(4));
        assert!(ack.acks(0x15)); // masked to 4 bits
    }

    #[test]
    fn activate_response_parses_sizes_and_port() {
        let mut data = [0u8; 12];
        data[6] = 0x56;
        data[7] = 0x02;
        data[8] = 0x10;
        data[9] = 0x00;
        data[10] = 0x6f;
        data[11] = 0x02;
        let rsp = SolActivateResponse::parse(&data).unwrap();
        assert_eq!(rsp.inbound_payload_size, 0x0256);
        assert_eq!(rsp.outbound_payload_size, 0x0010);
        assert_eq!(rsp.port, 0x026f);
    }

    #[test]
    fn activate_response_rejects_wrong_length() {
        assert!(SolActivateResponse::parse(&[0u8; 4]).is_err());
    }
}
