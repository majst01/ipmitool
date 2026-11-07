//! DCMI (Data Center Management Interface) commands. Mirrors the `dcmi`
//! command in `lib/ipmi_dcmi.c`.
//!
//! Currently only `dcmi power reading [<sample_time>]` is implemented; the
//! remaining subcommands are stubbed. The request/response layouts follow
//! `struct power_reading` in `include/ipmitool/ipmi_dcmi.h`.

use ipmi_core::cc;
use ipmi_core::consts;
use ipmi_core::interface::Interface;
use ipmi_core::types::{Request, Response};

/// Group Extension Identification byte for DCMI requests.
const IPMI_DCMI: u8 = 0xDC;
/// DCMI `Get Power Reading` command.
const IPMI_DCMI_GETRED: u8 = 0x02;

/// A DCMI command-table row: `(value, name, description)`.
///
/// The final row of every table is a terminator with an empty name; its value
/// is what `str2val2` returns for an unknown name, matching the C reference.
type CmdTable = &'static [(u16, &'static str, &'static str)];

/// Main DCMI commands (`dcmi_cmd_vals`).
const DCMI_CMD_VALS: CmdTable = &[
    (0x00, "discover", "Used to discover supported DCMI capabilities"),
    (0x01, "power", "Platform power limit command options"),
    (0x02, "sensors", "Prints the available DCMI sensors"),
    (0x03, "asset_tag", "Prints the platform's asset tag"),
    (0x04, "set_asset_tag", "Sets the platform's asset tag"),
    (0x05, "get_mc_id_string", "Get management controller ID string"),
    (0x06, "set_mc_id_string", "Set management controller ID string"),
    (0x07, "thermalpolicy", "Thermal policy get/set"),
    (0x08, "get_temp_reading", "Get Temperature Readings"),
    (0x09, "get_conf_param", "Get DCMI Config Parameters"),
    (0x0A, "set_conf_param", "Set DCMI Config Parameters"),
    (0x0B, "oob_discover", "Ping/Pong Message for DCMI Discovery"),
    (0xFF, "", ""),
];

/// Power-management commands (`dcmi_pwrmgmt_vals`).
const DCMI_PWRMGT_VALS: CmdTable = &[
    (0x00, "reading", "Get power related readings from the system"),
    (0x01, "get_limit", "Get the configured power limits"),
    (0x02, "set_limit", "Set a power limit option"),
    (0x03, "activate", "Activate the set power limit"),
    (0x04, "deactivate", "Deactivate the set power limit"),
    (0xFF, "", ""),
];

/// Power-reading sample-time selector values (`dcmi_sampling_vals`).
const DCMI_SAMPLING_VALS: CmdTable = &[
    (0x05, "5_sec", ""),
    (0x0f, "15_sec", ""),
    (0x1e, "30_sec", ""),
    (0x41, "1_min", ""),
    (0x43, "3_min", ""),
    (0x47, "7_min", ""),
    (0x4f, "15_min", ""),
    (0x5e, "30_min", ""),
    (0x81, "1_hour", ""),
    (0x00, "", ""),
];

/// DCMI-specific completion codes (`dcmi_ccode_vals`).
const DCMI_CCODE_VALS: &[(u16, &str)] = &[
    (0x80, "Parameter not supported"),
    (0x81, "Something else has already claimed these parameters"),
    (0x82, "Not supported or failed to write a read-only parameter"),
    (0x83, "Access mode is not supported"),
    (0x84, "Power/Thermal limit out of range"),
    (0x85, "Correction/Exception time out of range"),
    (0x89, "Sample/Statistics Reporting period out of range"),
    (0x8A, "Power limit already active"),
];

/// Entry point for `ipmitool dcmi ...`.
pub fn main(intf: &mut dyn Interface, args: &[String]) -> i32 {
    let Some(cmd) = args.first() else {
        print_strs(
            DCMI_CMD_VALS,
            "Data Center Management Interface commands",
            false,
        );
        return -1;
    };

    match str2val2(cmd, DCMI_CMD_VALS) {
        0x01 => {
            // `power` with no subcommand prints the usage and returns early,
            // before the trailing newline `ipmi_dcmi_main` emits after a break.
            let Some(sub) = args.get(1) else {
                print_strs(DCMI_PWRMGT_VALS, "power <command>", false);
                return -1;
            };
            let rc = power(intf, sub, &args[2..]);
            outln!(); // `ipmi_dcmi_main` prints "\n" after handling `power`.
            rc
        }
        0x00 | 0x02..=0x0B => {
            errln!("dcmi {cmd}: not implemented in this build");
            -1
        }
        _ => {
            // Unknown command (and `help`): print the command list.
            print_strs(
                DCMI_CMD_VALS,
                "Data Center Management Interface commands",
                false,
            );
            -1
        }
    }
}

/// `dcmi power <sub> [args...]` (`ipmi_dcmi_parse_power`).
fn power(intf: &mut dyn Interface, sub: &str, rest: &[String]) -> i32 {
    match str2val2(sub, DCMI_PWRMGT_VALS) {
        0x00 => {
            let mut sample_time = 0u8;
            if let Some(s) = rest.first() {
                let v = str2val2(s, DCMI_SAMPLING_VALS);
                if v == 0 {
                    print_strs(
                        DCMI_SAMPLING_VALS,
                        "Invalid sample time. Valid times are: ",
                        true,
                    );
                    outln!();
                    return -1;
                }
                sample_time = v as u8;
            }
            pwr_rd(intf, sample_time)
        }
        0x01..=0x04 => {
            errln!("dcmi power {sub}: not implemented in this build");
            -1
        }
        _ => {
            // `ipmi_dcmi_parse_power` leaves `rc` at 0 for the default case.
            print_strs(DCMI_PWRMGT_VALS, "power <command>", false);
            0
        }
    }
}

/// `ipmi_dcmi_pwr_rd`: fetch and print the DCMI power reading.
fn pwr_rd(intf: &mut dyn Interface, sample_time: u8) -> i32 {
    let mut msg_data = [0u8; 4];
    msg_data[0] = IPMI_DCMI;
    if sample_time != 0 {
        msg_data[1] = 0x02; // Enhanced Power Statistics
        msg_data[2] = sample_time;
    } else {
        msg_data[1] = 0x01; // Mode Power Status
        msg_data[2] = 0x00; // reserved
    }
    msg_data[3] = 0x00; // reserved

    let mut req = Request::new(consts::NETFN_DCGRP, IPMI_DCMI_GETRED);
    req.data = msg_data.to_vec();

    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => {
            errln!("\n    Unable to get DCMI information");
            return -1;
        }
    };
    if chk_rsp(&rsp) {
        return -1;
    }

    let d = &rsp.data;
    let curr_pwr = u16le(d, 1);
    let min_sample = u16le(d, 3);
    let max_sample = u16le(d, 5);
    let avg_pwr = u16le(d, 7);
    let time_stamp = u32le(d, 9);
    let sample = u32le(d, 13);
    let state = byte(d, 17);

    outln!();
    outln!("    Instantaneous power reading:              {curr_pwr:8} Watts");
    outln!("    Minimum during sampling period:           {min_sample:8} Watts");
    outln!("    Maximum during sampling period:           {max_sample:8} Watts");
    outln!("    Average power reading over sample period: {avg_pwr:8} Watts");
    out!(
        "    IPMI timestamp:                           {}",
        timestamp_numeric(time_stamp)
    );
    out!("    Sampling period:                          ");
    if sample_time != 0 {
        outln!("{} ", val2str2((sample & 0xffff) as u16, DCMI_SAMPLING_VALS));
    } else {
        outln!("{:08} Seconds.", sample / 1000);
    }
    out!("    Power reading state is:                   ");
    if state & 0x40 == 0x40 {
        out!("activated");
    } else {
        out!("deactivated");
    }
    outln!();
    outln!();
    0
}

/// `chk_rsp`: returns `true` when the response should abort the command.
fn chk_rsp(rsp: &Response) -> bool {
    if (0x80..=0x8F).contains(&rsp.ccode) {
        errln!(
            "\n    DCMI request failed because: {} ({:x})",
            dcmi_ccode_str(rsp.ccode),
            rsp.ccode
        );
        return true;
    } else if rsp.ccode != 0 {
        errln!(
            "\n    DCMI request failed because: {} ({:x})",
            cc::completion_code_str(rsp.ccode),
            rsp.ccode
        );
        return true;
    }
    if rsp.data.first().copied() != Some(IPMI_DCMI) {
        // The C prints this to stdout without a trailing newline.
        out!(
            "\n    A valid DCMI command was not returned! ({:x})",
            rsp.data.first().copied().unwrap_or(0)
        );
        return true;
    }
    false
}

// ---------------------------------------------------------------------------
// Helpers mirroring the C `str2val2` / `val2str2` / `print_strs`.
// ---------------------------------------------------------------------------

/// Case-insensitive name lookup; returns the terminator value on a miss.
fn str2val2(name: &str, table: CmdTable) -> u16 {
    for &(val, s, _) in table {
        if s.is_empty() {
            return val; // terminator
        }
        if s.eq_ignore_ascii_case(name) {
            return val;
        }
    }
    0
}

/// Value lookup; unknown values render like the C `val2str2` (lowercase hex).
fn val2str2(val: u16, table: CmdTable) -> String {
    for &(v, s, _) in table {
        if s.is_empty() {
            break;
        }
        if v == val {
            return s.to_string();
        }
    }
    format!("Unknown (0x{val:x})")
}

/// DCMI completion-code string (`val2str` over `dcmi_ccode_vals`).
fn dcmi_ccode_str(code: u8) -> String {
    for &(v, s) in DCMI_CCODE_VALS {
        if v == u16::from(code) {
            return s.to_string();
        }
    }
    format!("Unknown (0x{code:02X})")
}

/// Mirror of the C `print_strs` with a non-negative log level (stderr).
fn print_strs(table: CmdTable, title: &str, verthorz: bool) {
    errln!("\n{title}");
    let last = table.len().saturating_sub(1); // terminator index
    for (i, &(_, s, d)) in table.iter().enumerate() {
        if i == last {
            break;
        }
        errln!("    {s}    {d}");
        if verthorz && i + 1 < last {
            out!(" | ");
        }
    }
    if !verthorz {
        errln!();
    }
}

fn byte(data: &[u8], i: usize) -> u8 {
    data.get(i).copied().unwrap_or(0)
}

fn u16le(data: &[u8], i: usize) -> u16 {
    u16::from(byte(data, i)) | u16::from(byte(data, i + 1)) << 8
}

fn u32le(data: &[u8], i: usize) -> u32 {
    u32::from(byte(data, i))
        | u32::from(byte(data, i + 1)) << 8
        | u32::from(byte(data, i + 2)) << 16
        | u32::from(byte(data, i + 3)) << 24
}

// ---------------------------------------------------------------------------
// Timestamp formatting (`ipmi_timestamp_numeric`).
// ---------------------------------------------------------------------------

/// Format an IPMI timestamp the way `ipmi_timestamp_numeric()` does: an
/// absolute epoch timestamp uses `%x %X %Z` in local time; a "special" value
/// (`< 0x20000000`) is seconds since BMC power-on, rendered in UTC with an
/// `S+` prefix.
#[cfg(unix)]
fn timestamp_numeric(ts: u32) -> String {
    if ts == 0xFFFF_FFFF {
        return "Unspecified".to_string();
    }
    let (utc, fmt): (bool, &[u8]) = if ts < 0x2000_0000 {
        if ts < 86_400 {
            (true, b"S+ %H:%M:%S\0")
        } else {
            (true, b"S+ %y/%j %H:%M:%S\0")
        }
    } else {
        (false, b"%x %X %Z\0")
    };

    unsafe {
        let t = ts as libc::time_t;
        let mut tm: libc::tm = std::mem::zeroed();
        let ok = if utc {
            !libc::gmtime_r(&t, &mut tm).is_null()
        } else {
            !libc::localtime_r(&t, &mut tm).is_null()
        };
        if !ok {
            return "Unspecified".to_string();
        }
        let mut buf = [0 as libc::c_char; 80];
        let n = libc::strftime(
            buf.as_mut_ptr(),
            buf.len(),
            fmt.as_ptr() as *const libc::c_char,
            &tm,
        );
        if n == 0 {
            return "Unspecified".to_string();
        }
        let bytes: Vec<u8> = buf[..n].iter().map(|&c| c as u8).collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

#[cfg(not(unix))]
fn timestamp_numeric(ts: u32) -> String {
    if ts == 0xFFFF_FFFF {
        "Unspecified".to_string()
    } else {
        format!("Unknown ({ts})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str2val2_is_case_insensitive_and_falls_back() {
        assert_eq!(str2val2("reading", DCMI_PWRMGT_VALS), 0x00);
        assert_eq!(str2val2("READING", DCMI_PWRMGT_VALS), 0x00);
        assert_eq!(str2val2("bogus", DCMI_PWRMGT_VALS), 0xFF);
        assert_eq!(str2val2("15_min", DCMI_SAMPLING_VALS), 0x4f);
        assert_eq!(str2val2("bogus", DCMI_SAMPLING_VALS), 0x00);
    }

    #[test]
    fn val2str2_renders_unknown_with_lowercase_hex() {
        assert_eq!(val2str2(0x05, DCMI_SAMPLING_VALS), "5_sec");
        assert_eq!(val2str2(0x1234, DCMI_SAMPLING_VALS), "Unknown (0x1234)");
    }

    #[test]
    fn dcmi_ccode_strings_cover_the_range() {
        assert_eq!(dcmi_ccode_str(0x80), "Parameter not supported");
        assert_eq!(dcmi_ccode_str(0x8A), "Power limit already active");
        assert_eq!(dcmi_ccode_str(0x88), "Unknown (0x88)");
    }

    #[test]
    fn timestamp_relative_values_are_deterministic() {
        // Special timestamps render in UTC without a timezone offset.
        assert_eq!(timestamp_numeric(3661), "S+ 01:01:01");
        assert_eq!(timestamp_numeric(0xFFFF_FFFF), "Unspecified");
    }
}
