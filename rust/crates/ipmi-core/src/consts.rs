//! IPMI protocol constants.
//!
//! Values are taken from the IPMI v1.5/v2.0 specification and mirror
//! `include/ipmitool/ipmi.h`, `ipmi_constants.h` and `ipmi_cc.h` from the C
//! implementation.

/// Maximum size of an IPMI message payload.
pub const IPMI_BUF_SIZE: usize = 1024;
/// Largest message digest produced by any supported algorithm (SHA-256).
pub const IPMI_MAX_MD_SIZE: usize = 0x20;
/// Size of the user authentication key buffer (`Kuid`).
pub const IPMI_AUTHCODE_BUFFER_SIZE: usize = 20;
/// BMC key (`Kg`) buffer size (key plus NUL).
pub const IPMI_KG_BUFFER_SIZE: usize = 21;

// ---------------------------------------------------------------------------
// Network function codes (already in "final" form, not shifted into the LUN
// byte). Mask with `<< 2` when placing on the wire.
// ---------------------------------------------------------------------------
pub const NETFN_CHASSIS: u8 = 0x0;
pub const NETFN_BRIDGE: u8 = 0x2;
pub const NETFN_SE: u8 = 0x4;
pub const NETFN_APP: u8 = 0x6;
pub const NETFN_FIRMWARE: u8 = 0x8;
pub const NETFN_STORAGE: u8 = 0xa;
pub const NETFN_TRANSPORT: u8 = 0xc;
pub const NETFN_PICMG: u8 = 0x2c;
pub const NETFN_DCGRP: u8 = 0x2c;
pub const NETFN_OEM: u8 = 0x2e;
pub const NETFN_ISOL: u8 = 0x34;
pub const NETFN_TSOL: u8 = 0x30;

pub const BMC_SLAVE_ADDR: u8 = 0x20;
pub const REMOTE_SWID: u8 = 0x81;

// ---------------------------------------------------------------------------
// Session authentication types (the "auth type / format" byte).
// ---------------------------------------------------------------------------
pub const SESSION_AUTHTYPE_NONE: u8 = 0x0;
pub const SESSION_AUTHTYPE_MD2: u8 = 0x1;
pub const SESSION_AUTHTYPE_MD5: u8 = 0x2;
pub const SESSION_AUTHTYPE_KEY: u8 = 0x4;
pub const SESSION_AUTHTYPE_PASSWORD: u8 = SESSION_AUTHTYPE_KEY;
pub const SESSION_AUTHTYPE_OEM: u8 = 0x5;
pub const SESSION_AUTHTYPE_RMCP_PLUS: u8 = 0x6;

// ---------------------------------------------------------------------------
// Privilege levels.
// ---------------------------------------------------------------------------
pub const SESSION_PRIV_UNSPECIFIED: u8 = 0x0;
pub const SESSION_PRIV_CALLBACK: u8 = 0x1;
pub const SESSION_PRIV_USER: u8 = 0x2;
pub const SESSION_PRIV_OPERATOR: u8 = 0x3;
pub const SESSION_PRIV_ADMIN: u8 = 0x4;
pub const SESSION_PRIV_OEM: u8 = 0x5;
pub const SESSION_PRIV_NOACCESS: u8 = 0xf;

// ---------------------------------------------------------------------------
// RAKP authentication algorithms (table 13-17).
// ---------------------------------------------------------------------------
pub const AUTH_RAKP_NONE: u8 = 0x00;
pub const AUTH_RAKP_HMAC_SHA1: u8 = 0x01;
pub const AUTH_RAKP_HMAC_MD5: u8 = 0x02;
pub const AUTH_RAKP_HMAC_SHA256: u8 = 0x03;

// ---------------------------------------------------------------------------
// Integrity algorithms (table 13-18).
// ---------------------------------------------------------------------------
pub const INTEGRITY_NONE: u8 = 0x00;
pub const INTEGRITY_HMAC_SHA1_96: u8 = 0x01;
pub const INTEGRITY_HMAC_MD5_128: u8 = 0x02;
pub const INTEGRITY_MD5_128: u8 = 0x03;
pub const INTEGRITY_HMAC_SHA256_128: u8 = 0x04;

// ---------------------------------------------------------------------------
// Confidentiality algorithms (table 13-19).
// ---------------------------------------------------------------------------
pub const CRYPT_NONE: u8 = 0x00;
pub const CRYPT_AES_CBC_128: u8 = 0x01;
pub const CRYPT_XRC4_128: u8 = 0x02;
pub const CRYPT_XRC4_40: u8 = 0x03;

// ---------------------------------------------------------------------------
// RMCP+ payload types (table 13-16).
// ---------------------------------------------------------------------------
pub const PAYLOAD_TYPE_IPMI: u8 = 0x00;
pub const PAYLOAD_TYPE_SOL: u8 = 0x01;
pub const PAYLOAD_TYPE_OEM: u8 = 0x02;
pub const PAYLOAD_TYPE_RMCP_OPEN_REQUEST: u8 = 0x10;
pub const PAYLOAD_TYPE_RMCP_OPEN_RESPONSE: u8 = 0x11;
pub const PAYLOAD_TYPE_RAKP_1: u8 = 0x12;
pub const PAYLOAD_TYPE_RAKP_2: u8 = 0x13;
pub const PAYLOAD_TYPE_RAKP_3: u8 = 0x14;
pub const PAYLOAD_TYPE_RAKP_4: u8 = 0x15;

// ---------------------------------------------------------------------------
// RAKP status / return codes (table 13-15).
// ---------------------------------------------------------------------------
pub const RAKP_STATUS_NO_ERRORS: u8 = 0x00;
pub const RAKP_STATUS_INSUFFICIENT_RESOURCES_FOR_SESSION: u8 = 0x01;
pub const RAKP_STATUS_INVALID_SESSION_ID: u8 = 0x02;
pub const RAKP_STATUS_INVALID_PAYLOAD_TYPE: u8 = 0x03;
pub const RAKP_STATUS_INVALID_AUTHENTICATION_ALGORITHM: u8 = 0x04;
pub const RAKP_STATUS_INVALID_INTEGRITY_ALGORITHM: u8 = 0x05;
pub const RAKP_STATUS_NO_MATCHING_AUTHENTICATION_PAYLOAD: u8 = 0x06;
pub const RAKP_STATUS_NO_MATCHING_INTEGRITY_PAYLOAD: u8 = 0x07;
pub const RAKP_STATUS_INACTIVE_SESSION_ID: u8 = 0x08;
pub const RAKP_STATUS_INVALID_ROLE: u8 = 0x09;
pub const RAKP_STATUS_UNAUTHORIZED_ROLE_REQUESTED: u8 = 0x0a;
pub const RAKP_STATUS_INSUFFICIENT_RESOURCES_FOR_ROLE: u8 = 0x0b;
pub const RAKP_STATUS_INVALID_NAME_LENGTH: u8 = 0x0c;
pub const RAKP_STATUS_UNAUTHORIZED_NAME: u8 = 0x0d;
pub const RAKP_STATUS_UNAUTHORIZED_GUID: u8 = 0x0e;
pub const RAKP_STATUS_INVALID_INTEGRITY_CHECK_VALUE: u8 = 0x0f;
pub const RAKP_STATUS_INVALID_CONFIDENTIALITY_ALGORITHM: u8 = 0x10;
pub const RAKP_STATUS_NO_CIPHER_SUITE_MATCH: u8 = 0x11;
pub const RAKP_STATUS_ILLEGAL_PARAMETER: u8 = 0x12;

// ---------------------------------------------------------------------------
// Chassis command/parameter constants.
// ---------------------------------------------------------------------------
pub const CHASSIS_CTL_POWER_DOWN: u8 = 0x0;
pub const CHASSIS_CTL_POWER_UP: u8 = 0x1;
pub const CHASSIS_CTL_POWER_CYCLE: u8 = 0x2;
pub const CHASSIS_CTL_HARD_RESET: u8 = 0x3;
pub const CHASSIS_CTL_PULSE_DIAG: u8 = 0x4;
pub const CHASSIS_CTL_ACPI_SOFT: u8 = 0x5;

pub const CHASSIS_POLICY_NO_CHANGE: u8 = 0x3;
pub const CHASSIS_POLICY_ALWAYS_ON: u8 = 0x2;
pub const CHASSIS_POLICY_PREVIOUS: u8 = 0x1;
pub const CHASSIS_POLICY_ALWAYS_OFF: u8 = 0x0;

// ---------------------------------------------------------------------------
// Application commands used by the interfaces themselves.
// ---------------------------------------------------------------------------
pub const IPMI_GET_CHANNEL_AUTH_CAP: u8 = 0x38;
pub const IPMI_GET_CHANNEL_CIPHER_SUITES: u8 = 0x54;
pub const IPMI_CLOSE_SESSION: u8 = 0x3c;
pub const IPMI_SET_SESSION_PRIVILEGE: u8 = 0x3b;
pub const IPMI_GET_DEVICE_ID: u8 = 0x01;
pub const IPMI_GET_SELF_TEST_RESULTS: u8 = 0x04;
pub const IPMI_GET_GLOBAL_ENABLES: u8 = 0x2f;
pub const IPMI_SET_GLOBAL_ENABLES: u8 = 0x2e;
