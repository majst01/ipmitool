//! Serial-over-LAN commands. Mirrors `lib/ipmi_sol.c` and the SOL parts of
//! `src/plugins/lanplus/lanplus.c`.

use std::io::Write;

use ipmi_core::cc;
use ipmi_core::consts;
use ipmi_core::interface::Interface;
use ipmi_core::sol::{
    self, SolActivateRequest, SolConfigParams, SolPacket, IPMI_GET_USER_PAYLOAD_ACCESS,
    IPMI_SET_USER_PAYLOAD_ACCESS, PARAM_CHARACTER_INTERVAL, PARAM_NON_VOLATILE_BIT_RATE,
    PARAM_PAYLOAD_CHANNEL, PARAM_PAYLOAD_PORT, PARAM_SET_IN_PROGRESS, PARAM_SOL_AUTHENTICATION,
    PARAM_SOL_ENABLE, PARAM_SOL_RETRY, PARAM_VOLATILE_BIT_RATE,
};
use ipmi_core::types::Request;
use ipmi_core::values;

/// Default IPMI LAN channel.
const DEFAULT_CHANNEL: u8 = 0x0e;
/// Seconds between SOL keepalive probes.
const KEEPALIVE_TIMEOUT: u64 = 15;
/// Number of consecutive keepalive failures tolerated before exiting.
const MAX_KEEPALIVE_RETRY: u32 = 6;

pub fn main(intf: &mut dyn Interface, args: &[String]) -> i32 {
    let Some(cmd) = args.first().map(String::as_str) else {
        print_usage();
        return -1;
    };
    let rest = &args[1..];
    match cmd {
        "help" => {
            print_usage();
            0
        }
        "info" => info(intf, rest),
        "payload" => payload(intf, rest),
        "set" => set(intf, rest),
        "activate" => activate(intf, rest),
        "deactivate" => deactivate(intf, rest),
        "looptest" => {
            eprintln!("Error: sol looptest is not implemented");
            -1
        }
        other => {
            eprintln!("Invalid sol command: {other}");
            print_usage();
            -1
        }
    }
}

fn print_usage() {
    eprintln!("SOL Commands: info [<channel number>]");
    eprintln!("              set <parameter> <value> [channel]");
    eprintln!("              payload <enable|disable|status> [channel] [userid]");
    eprintln!("              activate [<usesolkeepalive|nokeepalive>] [instance=<number>]");
    eprintln!("              deactivate [instance=<number>]");
}

/// Parse a decimal or `0x`-prefixed byte.
fn parse_u8(s: &str) -> Option<u8> {
    let s = s.trim();
    let v = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u16::from_str_radix(hex, 16).ok()?
    } else {
        s.parse::<u16>().ok()?
    };
    u8::try_from(v).ok()
}

/// Parse the optional channel argument.
fn parse_channel(args: &[String], index: usize) -> Result<u8, i32> {
    match args.get(index) {
        None => Ok(DEFAULT_CHANNEL),
        Some(s) => parse_u8(s).ok_or_else(|| {
            eprintln!("Invalid channel number: {s}");
            -1
        }),
    }
}

// ---------------------------------------------------------------------------
// sol info
// ---------------------------------------------------------------------------

/// Read a single SOL configuration parameter. Returns `None` when the BMC
/// reports it as unsupported (completion code 0x80).
fn get_param(intf: &mut dyn Interface, channel: u8, selector: u8) -> Result<Option<Vec<u8>>, i32> {
    let mut req = Request::new(consts::NETFN_TRANSPORT, sol::IPMI_GET_SOL_CONFIG_PARAMETERS);
    req.data = vec![channel, selector, 0x00, 0x00];
    let rsp = intf.sendrecv(&req).map_err(|_| {
        eprintln!(
            "Error: No response requesting SOL parameter '{}'",
            sol::parameter_name(selector)
        );
        -1
    })?;
    match rsp.ccode {
        0x00 => Ok(Some(rsp.data)),
        0x80 => {
            eprintln!(
                "Info: SOL parameter '{}' not supported",
                sol::parameter_name(selector)
            );
            Ok(None)
        }
        code => {
            eprintln!(
                "Error requesting SOL parameter '{}': {}",
                sol::parameter_name(selector),
                cc::completion_code_str(code)
            );
            Err(-1)
        }
    }
}

/// Fetch the full SOL configuration, mirroring `ipmi_get_sol_info`.
fn get_sol_info(intf: &mut dyn Interface, channel: u8) -> Result<SolConfigParams, i32> {
    let mut p = SolConfigParams::default();

    if let Some(d) = get_param(intf, channel, PARAM_SET_IN_PROGRESS)? {
        if d.len() >= 2 {
            p.set_in_progress = d[1];
        }
    }
    if let Some(d) = get_param(intf, channel, PARAM_SOL_ENABLE)? {
        if d.len() >= 2 {
            p.enabled = d[1] & 0x01 != 0;
        }
    }
    if let Some(d) = get_param(intf, channel, PARAM_SOL_AUTHENTICATION)? {
        if d.len() >= 2 {
            p.force_encryption = d[1] & 0x80 != 0;
            p.force_authentication = d[1] & 0x40 != 0;
            p.privilege_level = d[1] & 0x0f;
        }
    }
    if let Some(d) = get_param(intf, channel, PARAM_CHARACTER_INTERVAL)? {
        if d.len() >= 3 {
            p.character_accumulate_level = d[1];
            p.character_send_threshold = d[2];
        }
    }
    if let Some(d) = get_param(intf, channel, PARAM_SOL_RETRY)? {
        if d.len() >= 3 {
            p.retry_count = d[1];
            p.retry_interval = d[2];
        }
    }
    if let Some(d) = get_param(intf, channel, PARAM_NON_VOLATILE_BIT_RATE)? {
        if d.len() >= 2 {
            p.non_volatile_bit_rate = d[1] & 0x0f;
        }
    }
    if let Some(d) = get_param(intf, channel, PARAM_VOLATILE_BIT_RATE)? {
        if d.len() >= 2 {
            p.volatile_bit_rate = d[1] & 0x0f;
        }
    }
    match get_param(intf, channel, PARAM_PAYLOAD_CHANNEL)? {
        Some(d) if d.len() >= 2 => p.payload_channel = d[1],
        _ => p.payload_channel = channel,
    }
    match get_param(intf, channel, PARAM_PAYLOAD_PORT)? {
        Some(d) if d.len() >= 3 => p.payload_port = d[1] as u16 | (d[2] as u16) << 8,
        _ => p.payload_port = intf.session_params().port,
    }
    Ok(p)
}

fn info(intf: &mut dyn Interface, args: &[String]) -> i32 {
    if args.len() > 1 {
        print_usage();
        return -1;
    }
    let channel = match parse_channel(args, 0) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let params = match get_sol_info(intf, channel) {
        Ok(p) => p,
        Err(code) => return code,
    };
    println!(
        "Set in progress                 : {}",
        sol::set_in_progress_str(params.set_in_progress)
    );
    println!(
        "Enabled                         : {}",
        if params.enabled { "true" } else { "false" }
    );
    println!(
        "Force Encryption                : {}",
        if params.force_encryption {
            "true"
        } else {
            "false"
        }
    );
    println!(
        "Force Authentication            : {}",
        if params.force_authentication {
            "true"
        } else {
            "false"
        }
    );
    println!(
        "Privilege Level                 : {}",
        values::privlvl_str(params.privilege_level)
    );
    println!(
        "Character Accumulate Level (ms) : {}",
        params.character_accumulate_level as u16 * 5
    );
    println!(
        "Character Send Threshold        : {}",
        params.character_send_threshold
    );
    println!("Retry Count                     : {}", params.retry_count);
    println!(
        "Retry Interval (ms)             : {}",
        params.retry_interval as u16 * 10
    );
    println!(
        "Volatile Bit Rate (kbps)        : {}",
        sol::bit_rate_str(params.volatile_bit_rate)
    );
    println!(
        "Non-Volatile Bit Rate (kbps)    : {}",
        sol::bit_rate_str(params.non_volatile_bit_rate)
    );
    println!(
        "Payload Channel                 : {} (0x{:02x})",
        params.payload_channel, params.payload_channel
    );
    println!("Payload Port                    : {}", params.payload_port);
    0
}

// ---------------------------------------------------------------------------
// sol payload
// ---------------------------------------------------------------------------

fn payload(intf: &mut dyn Interface, args: &[String]) -> i32 {
    if args.is_empty() || args.len() > 3 {
        print_usage();
        return -1;
    }
    let channel = match parse_channel(args, 1) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let userid = match args.get(2) {
        None => 1,
        Some(s) => match parse_u8(s) {
            Some(id) => id,
            None => {
                eprintln!("Invalid user id: {s}");
                return -1;
            }
        },
    };
    let userid = userid & 0x3f;

    match args[0].as_str() {
        "enable" | "disable" => {
            let enable = args[0] == "enable";
            let mut req = Request::new(consts::NETFN_APP, IPMI_SET_USER_PAYLOAD_ACCESS);
            req.data = vec![
                channel & 0x0f,
                (userid & 0x3f) | if enable { 0 } else { 0x40 },
                0x02,
                0,
                0,
                0,
            ];
            match intf.sendrecv(&req) {
                Ok(rsp) if rsp.ccode == 0 => 0,
                Ok(rsp) => {
                    eprintln!(
                        "Error {}abling SOL payload for user {} on channel {}: {}",
                        if enable { "en" } else { "dis" },
                        userid,
                        channel,
                        cc::completion_code_str(rsp.ccode)
                    );
                    -1
                }
                Err(_) => {
                    eprintln!(
                        "Error {}abling SOL payload for user {} on channel {}",
                        if enable { "en" } else { "dis" },
                        userid,
                        channel
                    );
                    -1
                }
            }
        }
        "status" => {
            let mut req = Request::new(consts::NETFN_APP, IPMI_GET_USER_PAYLOAD_ACCESS);
            req.data = vec![channel & 0x0f, userid & 0x3f];
            let rsp = match intf.sendrecv(&req) {
                Ok(rsp) => rsp,
                Err(_) => {
                    eprintln!("Error. No valid response received.");
                    return -1;
                }
            };
            if rsp.ccode != 0 {
                eprintln!(
                    "Error getting SOL payload status for user {} on channel {}: {}",
                    userid,
                    channel,
                    cc::completion_code_str(rsp.ccode)
                );
                return -1;
            }
            if rsp.data.len() != 4 {
                eprintln!(
                    "Error parsing SOL payload status for user {userid} on channel {channel}"
                );
                return -1;
            }
            println!(
                "User {} on channel {} is {}abled",
                userid,
                channel,
                if rsp.data[0] & 0x02 != 0 { "en" } else { "dis" }
            );
            0
        }
        other => {
            eprintln!("Invalid payload action: {other}");
            print_usage();
            -1
        }
    }
}

// ---------------------------------------------------------------------------
// sol set
// ---------------------------------------------------------------------------

/// Issue a single `Set SOL Configuration Parameters` request, returning the
/// completion code.
fn send_set(intf: &mut dyn Interface, channel: u8, selector: u8, values: &[u8]) -> Result<u8, i32> {
    let mut req = Request::new(consts::NETFN_TRANSPORT, sol::IPMI_SET_SOL_CONFIG_PARAMETERS);
    let mut data = vec![channel, selector];
    data.extend_from_slice(values);
    req.data = data;
    match intf.sendrecv(&req) {
        Ok(rsp) => Ok(rsp.ccode),
        Err(_) => {
            eprintln!(
                "Error setting SOL parameter '{}'",
                sol::parameter_name(selector)
            );
            Err(-1)
        }
    }
}

fn set_in_progress(intf: &mut dyn Interface, channel: u8, value: u8) -> Result<u8, i32> {
    send_set(intf, channel, PARAM_SET_IN_PROGRESS, &[value])
}

/// Parse a value in `min..=max`, printing the same diagnostics as C.
fn parse_range(value: &str, param: &str, min: u8, max: u8) -> Result<u8, i32> {
    match parse_u8(value) {
        Some(v) if v >= min && v <= max => Ok(v),
        _ => {
            eprintln!("Invalid value {value} for parameter {param}");
            eprintln!("Valid values are {min}-{max}");
            Err(-1)
        }
    }
}

fn parse_bit_rate(value: &str) -> Result<u8, i32> {
    match value {
        "serial" => Ok(0x00),
        "9.6" => Ok(0x06),
        "19.2" => Ok(0x07),
        "38.4" => Ok(0x08),
        "57.6" => Ok(0x09),
        "115.2" => Ok(0x0a),
        _ => {
            eprintln!("Invalid value \"{value}\" for parameter \"bit-rate\"");
            eprintln!("Valid values are serial, 9.6 19.2, 38.4, 57.6 and 115.2");
            Err(-1)
        }
    }
}

fn set(intf: &mut dyn Interface, args: &[String]) -> i32 {
    // Usage: set <param> <value> [channel] [noguard]
    if args.len() < 2 || args.len() > 4 {
        print_set_usage();
        return -1;
    }
    let param = args[0].as_str();
    let value = args[1].as_str();

    let mut guard = true;
    let mut channel = DEFAULT_CHANNEL;
    for extra in &args[2..] {
        if extra == "noguard" {
            guard = false;
        } else if let Some(c) = parse_u8(extra) {
            channel = c;
        } else {
            eprintln!("Invalid channel number: {extra}");
            return -1;
        }
    }

    // Per-parameter preparation: selector plus the value bytes.
    let (selector, values); // values exclude the selector byte (C's data[2..])
    match param {
        "set-in-progress" => {
            guard = false;
            let code = match value {
                "set-complete" => 0x00,
                "set-in-progress" => 0x01,
                "commit-write" => 0x02,
                _ => {
                    eprintln!("Invalid value {value} for parameter {param}");
                    eprintln!("Valid values are set-complete, set-in-progress and commit-write");
                    return -1;
                }
            };
            selector = PARAM_SET_IN_PROGRESS;
            values = vec![code];
        }
        "enabled" => {
            let code = match value {
                "true" => 0x01,
                "false" => 0x00,
                _ => {
                    eprintln!("Invalid value {value} for parameter {param}");
                    eprintln!("Valid values are true and false");
                    return -1;
                }
            };
            selector = PARAM_SOL_ENABLE;
            values = vec![code];
        }
        "force-encryption" | "force-authentication" | "privilege-level" => {
            let current = match get_sol_info(intf, channel) {
                Ok(p) => p,
                Err(code) => return code,
            };
            let mut byte = current.privilege_level;
            if current.force_encryption {
                byte |= 0x80;
            }
            if current.force_authentication {
                byte |= 0x40;
            }
            match param {
                "force-encryption" => match value {
                    "true" => byte |= 0x80,
                    "false" => byte &= !0x80,
                    _ => return invalid_bool(param, value),
                },
                "force-authentication" => match value {
                    "true" => byte |= 0x40,
                    "false" => byte &= !0x40,
                    _ => return invalid_bool(param, value),
                },
                _ => {
                    let lvl = match value {
                        "user" => 0x02,
                        "operator" => 0x03,
                        "admin" => 0x04,
                        "oem" => 0x05,
                        _ => {
                            eprintln!("Invalid value {value} for parameter {param}");
                            eprintln!("Valid values are user, operator, admin, and oem");
                            return -1;
                        }
                    };
                    byte = (byte & !0x0f) | lvl;
                }
            }
            selector = PARAM_SOL_AUTHENTICATION;
            values = vec![byte];
        }
        "character-accumulate-level" => {
            let level = match parse_range(value, param, 1, 255) {
                Ok(v) => v,
                Err(code) => return code,
            };
            let current = match get_sol_info(intf, channel) {
                Ok(p) => p,
                Err(code) => return code,
            };
            selector = PARAM_CHARACTER_INTERVAL;
            values = vec![level, current.character_send_threshold];
        }
        "character-send-threshold" => {
            let threshold = match parse_range(value, param, 0, 255) {
                Ok(v) => v,
                Err(code) => return code,
            };
            let current = match get_sol_info(intf, channel) {
                Ok(p) => p,
                Err(code) => return code,
            };
            selector = PARAM_CHARACTER_INTERVAL;
            values = vec![current.character_accumulate_level, threshold];
        }
        "retry-count" => {
            let count = match parse_range(value, param, 0, 7) {
                Ok(v) => v,
                Err(code) => return code,
            };
            let current = match get_sol_info(intf, channel) {
                Ok(p) => p,
                Err(code) => return code,
            };
            selector = PARAM_SOL_RETRY;
            values = vec![count, current.retry_interval];
        }
        "retry-interval" => {
            let interval = match parse_range(value, param, 0, 255) {
                Ok(v) => v,
                Err(code) => return code,
            };
            let current = match get_sol_info(intf, channel) {
                Ok(p) => p,
                Err(code) => return code,
            };
            selector = PARAM_SOL_RETRY;
            values = vec![current.retry_count, interval];
        }
        "non-volatile-bit-rate" => {
            let rate = match parse_bit_rate(value) {
                Ok(v) => v,
                Err(code) => return code,
            };
            selector = PARAM_NON_VOLATILE_BIT_RATE;
            values = vec![rate];
        }
        "volatile-bit-rate" => {
            let rate = match parse_bit_rate(value) {
                Ok(v) => v,
                Err(code) => return code,
            };
            selector = PARAM_VOLATILE_BIT_RATE;
            values = vec![rate];
        }
        _ => {
            eprintln!("Error: invalid SOL parameter {param}");
            return -1;
        }
    }

    // Guarded write sequence, matching ipmi_sol_set_param.
    if guard {
        if let Err(code) = set_in_progress(intf, channel, 0x01) {
            return code;
        }
    }

    let ccode = match send_set(intf, channel, selector, &values) {
        Ok(c) => c,
        Err(code) => return code,
    };

    let is_commit_write = param == "set-in-progress" && value == "commit-write";
    if !is_commit_write && ccode != 0 {
        match ccode {
            0x80 => eprintln!("Error setting SOL parameter '{param}': Parameter not supported"),
            0x81 => eprintln!(
                "Error setting SOL parameter '{param}': Attempt to set set-in-progress when not in set-complete state"
            ),
            0x82 => {
                eprintln!("Error setting SOL parameter '{param}': Attempt to write read-only parameter")
            }
            0x83 => {
                eprintln!("Error setting SOL parameter '{param}': Attempt to write write-only parameter")
            }
            _ => eprintln!(
                "Error setting SOL parameter '{param}' to '{value}': {}",
                cc::completion_code_str(ccode)
            ),
        }
        if guard {
            let _ = set_in_progress(intf, channel, 0x00);
        }
        return -1;
    }

    if guard {
        let _ = set_in_progress(intf, channel, 0x02); // commit-write, may be unimplemented
        if set_in_progress(intf, channel, 0x00).is_err() {
            eprintln!("Error could not set \"set-in-progress\" to \"set-complete\"");
            return -1;
        }
    }
    0
}

fn invalid_bool(param: &str, value: &str) -> i32 {
    eprintln!("Invalid value {value} for parameter {param}");
    eprintln!("Valid values are true and false");
    -1
}

fn print_set_usage() {
    eprintln!("SOL set parameters and values:");
    eprintln!("  set-in-progress             set-complete | set-in-progress | commit-write");
    eprintln!("  enabled                     true | false");
    eprintln!("  force-encryption            true | false");
    eprintln!("  force-authentication        true | false");
    eprintln!("  privilege-level             user | operator | admin | oem");
    eprintln!("  character-accumulate-level  <in 5 ms increments>");
    eprintln!("  character-send-threshold    N");
    eprintln!("  retry-count                 N");
    eprintln!("  retry-interval              <in 10 ms increments>");
    eprintln!("  non-volatile-bit-rate       serial | 9.6 | 19.2 | 38.4 | 57.6 | 115.2");
    eprintln!("  volatile-bit-rate           serial | 9.6 | 19.2 | 38.4 | 57.6 | 115.2");
}

// ---------------------------------------------------------------------------
// sol activate / deactivate
// ---------------------------------------------------------------------------

fn deactivate(intf: &mut dyn Interface, args: &[String]) -> i32 {
    let mut instance = 1u8;
    for arg in args {
        if let Some(v) = arg.strip_prefix("instance=") {
            match parse_u8(v) {
                Some(i) if (1..=15).contains(&i) => instance = i,
                _ => {
                    eprintln!("Given instance '{v}' is invalid.");
                    print_usage();
                    return -1;
                }
            }
        } else {
            print_usage();
            return -1;
        }
    }
    match intf.sol_deactivate(instance) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error de-activating SOL payload: {e}");
            -1
        }
    }
}

fn activate(intf: &mut dyn Interface, args: &[String]) -> i32 {
    let mut instance = 1u8;
    let mut disable_keepalive = false;
    let mut use_sol_keepalive = false;
    for arg in args {
        match arg.as_str() {
            "usesolkeepalive" => use_sol_keepalive = true,
            "nokeepalive" => disable_keepalive = true,
            _ => {
                if let Some(v) = arg.strip_prefix("instance=") {
                    match parse_u8(v) {
                        Some(i) if (1..=15).contains(&i) => instance = i,
                        _ => {
                            eprintln!("Given instance '{v}' is invalid.");
                            print_usage();
                            return -1;
                        }
                    }
                } else {
                    print_usage();
                    return -1;
                }
            }
        }
    }

    let req = SolActivateRequest {
        instance,
        encryption: true,
        authentication: true,
        assert_cts: false,
    };
    if let Err(e) = intf.sol_activate(&req) {
        eprintln!("Error activating SOL payload: {e}");
        return -1;
    }

    let escape_char = intf.session_params().sol_escape_char;
    println!(
        "[SOL Session operational.  Use {}? for help]",
        escape_char as char
    );

    let code = interactive(intf, disable_keepalive, use_sol_keepalive);

    // Best-effort cleanup, as in ipmi_sol_red_pill.
    if intf.sol_deactivate(instance).is_err() {
        // Ignore: the BMC may already have closed the payload.
    }
    code
}

// ---------------------------------------------------------------------------
// Interactive session
// ---------------------------------------------------------------------------

/// Process user input, handling escape sequences. Returns `Ok(0)` to
/// continue, `Ok(1)` to terminate, or `Err(())` if the BMC closed the session.
#[allow(clippy::too_many_arguments)]
fn process_input(
    intf: &mut dyn Interface,
    input: &[u8],
    escape_pending: &mut bool,
    last_was_cr: &mut bool,
    escape_char: u8,
) -> Result<u8, ()> {
    let mut out: Vec<u8> = Vec::with_capacity(input.len());
    let mut result = 0u8;

    for &ch in input {
        if *escape_pending {
            *escape_pending = false;
            match ch {
                b'.' => {
                    println!("{}. [terminated ipmitool]", escape_char as char);
                    result = 1;
                }
                0x1a | 0x18 => {
                    // ^Z / ^X: suspend. Real suspension needs SIGTSTP, which we
                    // do not attempt here; report and continue.
                    println!("{}^Z [suspend ipmitool]", escape_char as char);
                }
                b'B' => {
                    println!("{}B [send break]", escape_char as char);
                    let _ = intf.sol_send(&SolPacket::generate_break());
                }
                b'?' => print_escape_sequences(escape_char),
                _ => {
                    if ch != escape_char {
                        out.push(escape_char);
                    }
                    out.push(ch);
                }
            }
        } else if *last_was_cr && ch == escape_char {
            *escape_pending = true;
        } else {
            out.push(ch);
        }
        *last_was_cr = ch == b'\r' || ch == b'\n';
    }

    if !out.is_empty() {
        let pkt = SolPacket::data(out);
        if let Err(e) = intf.sol_send(&pkt) {
            eprintln!("Error sending SOL data: {e}");
            return Err(());
        }
    }
    Ok(result)
}

fn print_escape_sequences(escape_char: u8) {
    let e = escape_char as char;
    println!("{e}?");
    println!("\tSupported escape sequences:");
    println!("\t{e}.  - terminate connection");
    println!("\t{e}^Z - suspend ipmitool");
    println!("\t{e}^X - suspend ipmitool, but don't restore tty on restart");
    println!("\t{e}B  - send break");
    println!("\t{e}?  - this message");
    println!("\t{e}{e}  - send the escape character by typing it twice");
    println!("\t(Note that escapes are only recognized immediately after newline.)");
}

#[cfg(unix)]
fn interactive(intf: &mut dyn Interface, disable_keepalive: bool, use_sol_keepalive: bool) -> i32 {
    use std::time::Instant;

    let term = match RawTerminal::enter() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error: Unable to enter raw terminal mode: {e}");
            return -1;
        }
    };

    let escape_char = intf.session_params().sol_escape_char;
    let fd = intf.raw_fd().unwrap_or(-1);
    let mut escape_pending = false;
    let mut last_was_cr = true;
    let mut should_exit = false;
    let mut bmc_closed = false;
    let mut keepalive_start = Instant::now();
    let mut keepalive_failures = 0u32;

    while !should_exit {
        if !disable_keepalive && keepalive_start.elapsed().as_secs() > KEEPALIVE_TIMEOUT {
            let ok = if use_sol_keepalive {
                intf.sol_send(&SolPacket::data(Vec::new())).is_ok()
            } else {
                let req = Request::new(consts::NETFN_APP, consts::IPMI_GET_DEVICE_ID);
                intf.sendrecv(&req).is_ok()
            };
            if ok {
                keepalive_start = Instant::now();
                keepalive_failures = 0;
            } else {
                keepalive_failures += 1;
                if keepalive_failures >= MAX_KEEPALIVE_RETRY {
                    eprintln!("Error: No response to keepalive - Terminating session");
                    should_exit = true;
                    continue;
                }
            }
        }

        let mut fds = [
            libc::pollfd {
                fd: libc::STDIN_FILENO,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        let n = unsafe { libc::poll(fds.as_mut_ptr(), 2, 500) };
        if n < 0 {
            eprintln!("Error: poll failed");
            should_exit = true;
            continue;
        }
        if n == 0 {
            continue;
        }

        if fds[0].revents & libc::POLLIN != 0 {
            let mut buf = vec![0u8; 256];
            let got = unsafe {
                libc::read(
                    libc::STDIN_FILENO,
                    buf.as_mut_ptr() as *mut libc::c_void,
                    buf.len(),
                )
            };
            if got <= 0 {
                should_exit = true;
            } else {
                buf.truncate(got as usize);
                match process_input(
                    intf,
                    &buf,
                    &mut escape_pending,
                    &mut last_was_cr,
                    escape_char,
                ) {
                    Ok(1) => should_exit = true,
                    Ok(_) => {}
                    Err(()) => {
                        should_exit = true;
                        bmc_closed = true;
                    }
                }
            }
        } else if fds[1].revents & libc::POLLIN != 0 {
            match intf.sol_recv() {
                Ok(pkt) => {
                    if !pkt.data.is_empty() {
                        let stdout = std::io::stdout();
                        let mut lock = stdout.lock();
                        let _ = lock.write_all(&pkt.data);
                        let _ = lock.flush();
                    }
                }
                Err(_) => {
                    eprintln!("SOL session closed by BMC");
                    should_exit = true;
                    bmc_closed = true;
                }
            }
        }
    }

    drop(term);
    if bmc_closed {
        1
    } else {
        0
    }
}

#[cfg(not(unix))]
fn interactive(
    _intf: &mut dyn Interface,
    _disable_keepalive: bool,
    _use_sol_keepalive: bool,
) -> i32 {
    eprintln!("Error: interactive SOL requires a Unix terminal");
    -1
}

/// RAII guard that switches stdin into raw mode and restores it on drop.
#[cfg(unix)]
struct RawTerminal {
    saved: libc::termios,
}

#[cfg(unix)]
impl RawTerminal {
    fn enter() -> std::io::Result<Self> {
        unsafe {
            let mut tio: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(libc::STDIN_FILENO, &mut tio) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let saved = tio;
            tio.c_iflag |= libc::IGNPAR;
            tio.c_iflag &= !(libc::ISTRIP
                | libc::INLCR
                | libc::IGNCR
                | libc::ICRNL
                | libc::IXON
                | libc::IXANY
                | libc::IXOFF);
            tio.c_lflag &= !(libc::ISIG
                | libc::ICANON
                | libc::ECHO
                | libc::ECHOE
                | libc::ECHOK
                | libc::ECHONL
                | libc::IEXTEN);
            tio.c_oflag &= !libc::OPOST;
            tio.c_cc[libc::VMIN] = 1;
            tio.c_cc[libc::VTIME] = 0;
            if libc::tcsetattr(libc::STDIN_FILENO, libc::TCSADRAIN, &tio) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(Self { saved })
        }
    }
}

#[cfg(unix)]
impl Drop for RawTerminal {
    fn drop(&mut self) {
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSADRAIN, &self.saved);
        }
    }
}
