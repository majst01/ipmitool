//! Generic IPMI completion codes (IPMI spec table 5-2).

pub const CC_OK: u8 = 0x00;
pub const CC_NODE_BUSY: u8 = 0xc0;
pub const CC_INV_CMD: u8 = 0xc1;
pub const CC_INV_CMD_FOR_LUN: u8 = 0xc2;
pub const CC_TIMEOUT: u8 = 0xc3;
pub const CC_OUT_OF_SPACE: u8 = 0xc4;
pub const CC_RES_CANCELED: u8 = 0xc5;
pub const CC_REQ_DATA_TRUNC: u8 = 0xc6;
pub const CC_REQ_DATA_INV_LENGTH: u8 = 0xc7;
pub const CC_REQ_DATA_FIELD_EXCEED: u8 = 0xc8;
pub const CC_PARAM_OUT_OF_RANGE: u8 = 0xc9;
pub const CC_CANT_RET_NUM_REQ_BYTES: u8 = 0xca;
pub const CC_REQ_DATA_NOT_PRESENT: u8 = 0xcb;
pub const CC_INV_DATA_FIELD_IN_REQ: u8 = 0xcc;
pub const CC_ILL_SENSOR_OR_RECORD: u8 = 0xcd;
pub const CC_RESP_COULD_NOT_BE_PRV: u8 = 0xce;
pub const CC_CANT_RESP_DUPLI_REQ: u8 = 0xcf;
pub const CC_CANT_RESP_SDRR_UPDATE: u8 = 0xd0;
pub const CC_CANT_RESP_FIRM_UPDATE: u8 = 0xd1;
pub const CC_CANT_RESP_BMC_INIT: u8 = 0xd2;
pub const CC_DESTINATION_UNAVAILABLE: u8 = 0xd3;
pub const CC_INSUFFICIENT_PRIVILEGES: u8 = 0xd4;
pub const CC_NOT_SUPPORTED_PRESENT_STATE: u8 = 0xd5;
pub const CC_ILLEGAL_COMMAND_DISABLED: u8 = 0xd6;
pub const CC_UNSPECIFIED_ERROR: u8 = 0xff;

/// Human readable string for a completion code, matching ipmitool's
/// `completion_code_vals` table.
pub fn completion_code_str(cc: u8) -> String {
    let s = match cc {
        0x00 => "Command completed normally",
        0xc0 => "Node busy",
        0xc1 => "Invalid command",
        0xc2 => "Invalid command on LUN",
        0xc3 => "Timeout",
        0xc4 => "Out of space",
        0xc5 => "Reservation cancelled or invalid",
        0xc6 => "Request data truncated",
        0xc7 => "Request data length invalid",
        0xc8 => "Request data field length limit exceeded",
        0xc9 => "Parameter out of range",
        0xca => "Cannot return number of requested data bytes",
        0xcb => "Requested sensor, data, or record not found",
        0xcc => "Invalid data field in request",
        0xcd => "Command illegal for specified sensor or record type",
        0xce => "Command response could not be provided",
        0xcf => "Cannot execute duplicated request",
        0xd0 => "SDR Repository in update mode",
        0xd1 => "Device firmware in update mode",
        0xd2 => "BMC initialization in progress",
        0xd3 => "Destination unavailable",
        0xd4 => "Insufficient privilege level",
        0xd5 => "Command not supported in present state",
        0xd6 => "Cannot execute command, command disabled",
        0xff => "Unspecified error",
        _ => return format!("Unknown (0x{cc:02X})"),
    };
    s.to_string()
}
