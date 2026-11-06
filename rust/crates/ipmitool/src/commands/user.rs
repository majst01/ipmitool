//! The `user` command: manage IPMI user accounts and channel access.
//!
//! Mirrors `lib/ipmi_user.c`: `summary`, `list`, `set name`, `set password`,
//! `enable`, `disable`, `priv` and `test`.

use std::io::Write as _;

use ipmi_core::cc;
use ipmi_core::consts;
use ipmi_core::interface::Interface;
use ipmi_core::types::Request;
use ipmi_core::values;

use crate::args::parse_num;
use crate::csv;

const NETFN_APP: u8 = consts::NETFN_APP;

const SET_USER_ACCESS: u8 = 0x43;
const GET_USER_ACCESS: u8 = 0x44;
const SET_USER_NAME: u8 = 0x45;
const GET_USER_NAME: u8 = 0x46;
const SET_USER_PASSWORD: u8 = 0x47;

const PASSWORD_DISABLE_USER: u8 = 0x00;
const PASSWORD_ENABLE_USER: u8 = 0x01;
const PASSWORD_SET_PASSWORD: u8 = 0x02;
const PASSWORD_TEST_PASSWORD: u8 = 0x03;

/// The user id is a 6-bit field in bits [5:0].
const UID_MASK: u8 = 0x3F;
const UID_MIN: u8 = 1;
const UID_MAX: u8 = 63;

/// `0x0E` tells the BMC to use the channel the request arrived on.
const CHANNEL_CURRENT: u8 = 0x0E;

const USER_PW_IPMI15_LEN: usize = 16;
const USER_PW_IPMI20_LEN: usize = 20;
const USER_PW_MAX_LEN: usize = USER_PW_IPMI20_LEN;

/// `IPMI_UID(id)`.
fn uid(id: u8) -> u8 {
    id & UID_MASK
}

/// Access rights returned by `Get User Access`.
#[derive(Default, Clone)]
struct UserAccess {
    callin_callback: bool,
    channel: u8,
    enabled_user_ids: u8,
    enable_status: u8,
    fixed_user_ids: u8,
    ipmi_messaging: bool,
    link_auth: bool,
    max_user_ids: u8,
    privilege_limit: u8,
    session_limit: u8,
    user_id: u8,
}

// ---------------------------------------------------------------------------
// Command dispatch (`ipmi_user_main`).
// ---------------------------------------------------------------------------

pub fn main(intf: &mut dyn Interface, args: &[String]) -> i32 {
    let Some(cmd) = args.first().map(String::as_str) else {
        eprintln!("Not enough parameters given.");
        print_usage();
        return -1;
    };

    match cmd {
        "help" => {
            print_usage();
            0
        }
        "summary" => summary(intf, args),
        "list" => list(intf, args),
        "test" => test(intf, args),
        "set" => {
            if args.len() >= 3 && args[1] == "password" {
                password(intf, args)
            } else if args.len() >= 2 && args[1] == "name" {
                name(intf, args)
            } else {
                print_usage();
                -1
            }
        }
        "priv" => priv_cmd(intf, args),
        "disable" | "enable" => mod_user(intf, args),
        other => {
            eprintln!("Invalid user command: '{other}'");
            print_usage();
            -1
        }
    }
}

fn print_usage() {
    eprintln!("User Commands:");
    eprintln!("               summary      [<channel number>]");
    eprintln!("               list         [<channel number>]");
    eprintln!("               set name     <user id> <username>");
    eprintln!("               set password <user id> [<password> [<16|20>]]");
    eprintln!("               disable      <user id>");
    eprintln!("               enable       <user id>");
    eprintln!("               priv         <user id> <privilege level> [<channel number>]");
    eprintln!("                     Privilege levels:");
    eprintln!("                      * 0x1 - Callback");
    eprintln!("                      * 0x2 - User");
    eprintln!("                      * 0x3 - Operator");
    eprintln!("                      * 0x4 - Administrator");
    eprintln!("                      * 0x5 - OEM Proprietary");
    eprintln!("                      * 0xF - No Access");
    eprintln!();
    eprintln!("               test         <user id> <16|20> [<password>]");
    eprintln!();
}

// ---------------------------------------------------------------------------
// Subcommands.
// ---------------------------------------------------------------------------

fn summary(intf: &mut dyn Interface, args: &[String]) -> i32 {
    let channel = match args.len() {
        1 => CHANNEL_CURRENT,
        2 => match is_channel_num(&args[1]) {
            Some(c) => c,
            None => return -1,
        },
        _ => {
            print_usage();
            return -1;
        }
    };
    print_summary(intf, channel)
}

fn list(intf: &mut dyn Interface, args: &[String]) -> i32 {
    let channel = match args.len() {
        1 => CHANNEL_CURRENT,
        2 => match is_channel_num(&args[1]) {
            Some(c) => c,
            None => return -1,
        },
        _ => {
            print_usage();
            return -1;
        }
    };
    print_list(intf, channel)
}

fn test(intf: &mut dyn Interface, args: &[String]) -> i32 {
    // args[0] == "test", args[1] == user id, args[2] == 16|20, args[3] optional password
    if args.len() != 3 && args.len() != 4 {
        print_usage();
        return -1;
    }
    let Some(user_id) = is_user_id(&args[1]) else {
        return -1;
    };
    let password_length = match parse_num(&args[2]) {
        Some(v) if v == USER_PW_IPMI15_LEN as u32 || v == USER_PW_IPMI20_LEN as u32 => v as usize,
        _ => {
            eprintln!("Given password length '{}' is invalid.", args[2]);
            eprintln!("Expected value is either 16 or 20.");
            return -1;
        }
    };
    let password = if args.len() == 3 {
        ask_password(user_id)
    } else {
        args[3].clone()
    };
    test_password(
        intf,
        user_id,
        &password,
        password_length == USER_PW_IPMI20_LEN,
    )
}

fn password(intf: &mut dyn Interface, args: &[String]) -> i32 {
    // args[0] == "set", args[1] == "password", args[2] == user id,
    // args[3] optional password, args[4] optional 16|20
    let Some(user_id) = is_user_id(&args[2]) else {
        return -1;
    };

    let password = if args.len() == 3 {
        let first = ask_password(user_id);
        let second = ask_password(user_id);
        let compare_len = second.len().min(USER_PW_MAX_LEN + 1);
        if !strncmp_eq(&first, &second, compare_len) {
            eprintln!("Passwords do not match or are longer than {USER_PW_MAX_LEN}");
            return -1;
        }
        first
    } else {
        args[3].clone()
    };

    let password_len = password.len().min(USER_PW_MAX_LEN + 1);
    let mut password_type = USER_PW_IPMI15_LEN;
    if args.len() > 4 {
        match parse_num(&args[4]) {
            Some(v) if v == USER_PW_IPMI15_LEN as u32 || v == USER_PW_IPMI20_LEN as u32 => {
                password_type = v as usize;
            }
            _ => {
                eprintln!("Invalid password length '{}'", args[4]);
                return -1;
            }
        }
    } else if password_len > USER_PW_IPMI15_LEN {
        password_type = USER_PW_IPMI20_LEN;
    }

    if password_len > password_type {
        eprintln!("Password is too long (> {password_type} bytes)");
        return -1;
    }

    let ccode = set_user_password(
        intf,
        user_id,
        PASSWORD_SET_PASSWORD,
        Some(&password),
        password_type > USER_PW_IPMI15_LEN,
    );
    if eval_ccode(ccode) != 0 {
        eprintln!("Set User Password command failed (user {user_id})");
        -1
    } else {
        println!("Set User Password command successful (user {user_id})");
        0
    }
}

fn priv_cmd(intf: &mut dyn Interface, args: &[String]) -> i32 {
    // args[0] == "priv", args[1] == user id, args[2] == priv, args[3] optional channel
    if args.len() != 3 && args.len() != 4 {
        print_usage();
        return -1;
    }
    let channel = if args.len() == 4 {
        match is_channel_num(&args[3]) {
            Some(c) => c,
            None => return -1,
        }
    } else {
        CHANNEL_CURRENT
    };
    let Some(privilege_limit) = is_priv_limit(&args[2]) else {
        return -1;
    };
    let Some(user_id) = is_user_id(&args[1]) else {
        return -1;
    };
    let ua = UserAccess {
        channel,
        privilege_limit,
        user_id,
        ..Default::default()
    };

    let ccode = set_user_access(intf, &ua, true);
    if eval_ccode(ccode) != 0 {
        eprintln!("Set Privilege Level command failed (user {user_id})");
        -1
    } else {
        println!("Set Privilege Level command successful (user {user_id})");
        0
    }
}

fn mod_user(intf: &mut dyn Interface, args: &[String]) -> i32 {
    // args[0] == "disable"|"enable", args[1] == user id
    if args.len() != 2 {
        print_usage();
        return -1;
    }
    let Some(user_id) = is_user_id(&args[1]) else {
        return -1;
    };
    let operation = if args[0] == "disable" {
        PASSWORD_DISABLE_USER
    } else {
        PASSWORD_ENABLE_USER
    };

    let ccode = set_user_password(intf, user_id, operation, None, false);
    if eval_ccode(ccode) != 0 {
        eprintln!("Set User Password command failed (user {user_id})");
        -1
    } else {
        0
    }
}

fn name(intf: &mut dyn Interface, args: &[String]) -> i32 {
    // args[0] == "set", args[1] == "name", args[2] == user id, args[3] == name
    if args.len() != 4 {
        print_usage();
        return -1;
    }
    let Some(user_id) = is_user_id(&args[2]) else {
        return -1;
    };
    if args[3].len() > 16 {
        eprintln!("Username is too long (> 16 bytes)");
        return -1;
    }
    set_username(intf, user_id, &args[3])
}

// ---------------------------------------------------------------------------
// Protocol operations (mirror `_ipmi_get_user_access` and friends).
// ---------------------------------------------------------------------------

/// Returns `0` on success, a negative value for a transport/length error, or a
/// positive completion code.
fn get_user_access(intf: &mut dyn Interface, ua: &mut UserAccess) -> i32 {
    let data = [ua.channel & 0x0F, uid(ua.user_id)];
    let req = Request::new(NETFN_APP, GET_USER_ACCESS).with_data(data);
    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => return -1,
    };
    if rsp.ccode != 0 {
        return rsp.ccode as i32;
    }
    if rsp.data.len() != 4 {
        return -2;
    }
    ua.max_user_ids = uid(rsp.data[0]);
    ua.enable_status = rsp.data[1] & 0xC0;
    ua.enabled_user_ids = uid(rsp.data[1]);
    ua.fixed_user_ids = uid(rsp.data[2]);
    ua.callin_callback = rsp.data[3] & 0x40 != 0;
    ua.link_auth = rsp.data[3] & 0x20 != 0;
    ua.ipmi_messaging = rsp.data[3] & 0x10 != 0;
    ua.privilege_limit = rsp.data[3] & 0x0F;
    0
}

/// Fetch the user name for `un.user_id` into `un.user_name`.
fn get_user_name(intf: &mut dyn Interface, un: &mut UserName) -> i32 {
    let req = Request::new(NETFN_APP, GET_USER_NAME).with_data([uid(un.user_id)]);
    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => return -1,
    };
    if rsp.ccode != 0 {
        return rsp.ccode as i32;
    }
    if rsp.data.len() != 16 {
        return -2;
    }
    let end = rsp.data.iter().position(|&b| b == 0).unwrap_or(16);
    un.user_name = String::from_utf8_lossy(&rsp.data[..end]).into_owned();
    0
}

fn set_user_access(intf: &mut dyn Interface, ua: &UserAccess, change_priv_limit_only: bool) -> i32 {
    let mut d0 = if change_priv_limit_only { 0x00 } else { 0x80 };
    if ua.callin_callback {
        d0 |= 0x40;
    }
    if ua.link_auth {
        d0 |= 0x20;
    }
    if ua.ipmi_messaging {
        d0 |= 0x10;
    }
    d0 |= ua.channel & 0x0F;
    let data = [
        d0,
        uid(ua.user_id),
        ua.privilege_limit & 0x0F,
        ua.session_limit & 0x0F,
    ];
    let req = Request::new(NETFN_APP, SET_USER_ACCESS).with_data(data);
    match intf.sendrecv(&req) {
        Ok(rsp) => rsp.ccode as i32,
        Err(_) => -1,
    }
}

fn set_user_password(
    intf: &mut dyn Interface,
    user_id: u8,
    operation: u8,
    password: Option<&str>,
    is_twenty_byte: bool,
) -> i32 {
    let data_len = if is_twenty_byte { 22 } else { 18 };
    let mut data = vec![0u8; data_len];
    data[0] = (if is_twenty_byte { 0x80 } else { 0x00 }) | uid(user_id);
    data[1] = 0x03 & operation;
    if let Some(password) = password {
        let bytes = password.as_bytes();
        let n = bytes.len().min(data_len - 2);
        data[2..2 + n].copy_from_slice(&bytes[..n]);
    }
    let req = Request::new(NETFN_APP, SET_USER_PASSWORD).with_data(data);
    match intf.sendrecv(&req) {
        Ok(rsp) => rsp.ccode as i32,
        Err(_) => -1,
    }
}

fn set_username(intf: &mut dyn Interface, user_id: u8, name: &str) -> i32 {
    if name.len() >= 17 {
        return -1;
    }
    let mut data = vec![0u8; 17];
    data[0] = uid(user_id);
    let bytes = name.as_bytes();
    data[1..1 + bytes.len()].copy_from_slice(bytes);
    let req = Request::new(NETFN_APP, SET_USER_NAME).with_data(data);
    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => {
            eprintln!("Set User Name command failed (user {user_id}, name {name})");
            return -1;
        }
    };
    if rsp.ccode != 0 {
        eprintln!(
            "Set User Name command failed (user {user_id}, name {name}): {}",
            cc::completion_code_str(rsp.ccode)
        );
        return -1;
    }
    0
}

fn test_password(
    intf: &mut dyn Interface,
    user_id: u8,
    password: &str,
    is_twenty_byte_password: bool,
) -> i32 {
    let ret = set_user_password(
        intf,
        user_id,
        PASSWORD_TEST_PASSWORD,
        Some(password),
        is_twenty_byte_password,
    );
    match ret {
        0 => println!("Success"),
        0x80 => println!("Failure: password incorrect"),
        0x81 => println!("Failure: wrong password size"),
        _ => println!("Unknown error"),
    }
    if ret == 0 {
        0
    } else {
        -1
    }
}

// ---------------------------------------------------------------------------
// Output.
// ---------------------------------------------------------------------------

fn print_summary(intf: &mut dyn Interface, channel: u8) -> i32 {
    let mut ua = UserAccess {
        channel,
        user_id: 1,
        ..Default::default()
    };
    if eval_ccode(get_user_access(intf, &mut ua)) != 0 {
        return -1;
    }
    if csv::enabled() {
        println!(
            "{},{},{}",
            ua.max_user_ids, ua.enabled_user_ids, ua.fixed_user_ids
        );
    } else {
        println!("Maximum IDs\t    : {}", ua.max_user_ids);
        println!("Enabled User Count  : {}", ua.enabled_user_ids);
        println!("Fixed Name Count    : {}", ua.fixed_user_ids);
    }
    0
}

fn print_list(intf: &mut dyn Interface, channel: u8) -> i32 {
    let mut current_user_id: u8 = 1;
    let mut printed_header = false;
    loop {
        let mut ua = UserAccess {
            user_id: current_user_id,
            channel,
            ..Default::default()
        };
        if eval_ccode(get_user_access(intf, &mut ua)) != 0 {
            return -1;
        }
        let mut un = UserName {
            user_id: current_user_id,
            user_name: String::new(),
        };
        let ccode = get_user_name(intf, &mut un);
        if ccode == 0xCC {
            // Some BMCs return "Invalid data field in request" for user ids
            // that have no name; treat them as having an empty name.
            un.user_name.clear();
        } else if eval_ccode(ccode) != 0 {
            return -1;
        }

        if csv::enabled() {
            dump_user_access_csv(&un.user_name, &ua);
        } else {
            dump_user_access(&un.user_name, &ua, &mut printed_header);
        }

        current_user_id = current_user_id.wrapping_add(1);
        if !(current_user_id <= ua.max_user_ids && current_user_id <= UID_MAX) {
            break;
        }
    }
    0
}

fn dump_user_access(name: &str, ua: &UserAccess, printed_header: &mut bool) {
    if !*printed_header {
        println!("ID  Name\t     Callin  Link Auth\tIPMI Msg   Channel Priv Limit");
        *printed_header = true;
    }
    println!(
        "{:<4}{:<17}{:<8}{:<11}{:<11}{}",
        ua.user_id,
        name,
        if ua.callin_callback { "false" } else { "true " },
        if ua.link_auth { "true " } else { "false" },
        if ua.ipmi_messaging { "true " } else { "false" },
        values::privlvl_str(ua.privilege_limit)
    );
}

fn dump_user_access_csv(name: &str, ua: &UserAccess) {
    println!(
        "{},{},{},{},{},{}",
        ua.user_id,
        name,
        if ua.callin_callback { "false" } else { "true" },
        if ua.link_auth { "true" } else { "false" },
        if ua.ipmi_messaging { "true" } else { "false" },
        values::privlvl_str(ua.privilege_limit)
    );
}

// ---------------------------------------------------------------------------
// Helpers.
// ---------------------------------------------------------------------------

/// Mirror `eval_ccode()`: print the C error text and collapse everything to a
/// plain success/failure result.
fn eval_ccode(ccode: i32) -> i32 {
    if ccode == 0 {
        return 0;
    }
    if ccode < 0 {
        match ccode {
            -1 => eprintln!("IPMI response is NULL."),
            -2 => eprintln!("Unexpected data length received."),
            -3 => eprintln!("Invalid function parameter."),
            -4 => eprintln!("ipmitool: malloc failure."),
            _ => {}
        }
        -1
    } else {
        eprintln!(
            "IPMI command failed: {}",
            cc::completion_code_str(ccode as u8)
        );
        -1
    }
}

fn is_channel_num(s: &str) -> Option<u8> {
    match crate::args::parse_u8(s) {
        Some(c) if c <= 0xB || (0xE..=0xF).contains(&c) => Some(c),
        _ => {
            eprintln!("Given Channel number '{s}' is either invalid or out of range.");
            eprintln!("Channel number must be from ranges: <0x0..0xB>, <0xE..0xF>");
            None
        }
    }
}

fn is_user_id(s: &str) -> Option<u8> {
    match crate::args::parse_u8(s) {
        Some(id) if (UID_MIN..=UID_MAX).contains(&id) => Some(id),
        _ => {
            eprintln!("Given User ID '{s}' is either invalid or out of range.");
            eprintln!("User ID is limited to range <{UID_MIN}..{UID_MAX}>.");
            None
        }
    }
}

fn is_priv_limit(s: &str) -> Option<u8> {
    match crate::args::parse_u8(s) {
        Some(p) if (0x01..=0x05).contains(&p) || p == 0x0F => Some(p),
        _ => {
            eprintln!("Given Privilege Limit '{s}' is invalid.");
            eprintln!("Privilege Limit is limited to <0x1..0x5> and <0xF>.");
            None
        }
    }
}

/// `ipmi_user_build_password_prompt()` + `ask_password()`. Echo is not
/// disabled, matching the existing `-a` limitation.
fn ask_password(user_id: u8) -> String {
    eprint!("Password for user {user_id}: ");
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line);
    line.trim_end_matches(['\r', '\n']).to_string()
}

/// `strncmp(a, b, n) == 0`, treating past-the-end bytes as NUL.
fn strncmp_eq(a: &str, b: &str, n: usize) -> bool {
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    for i in 0..n {
        let x = ab.get(i).copied().unwrap_or(0);
        let y = bb.get(i).copied().unwrap_or(0);
        if x != y {
            return false;
        }
    }
    true
}

/// A user name as fetched by `Get User Name`.
struct UserName {
    user_id: u8,
    user_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uid_masks_to_six_bits() {
        assert_eq!(uid(0x41), 1);
        assert_eq!(uid(0x3F), 0x3F);
        assert_eq!(uid(0x00), 0x00);
    }

    #[test]
    fn strncmp_matches_c_semantics() {
        assert!(strncmp_eq("abc", "abc", 3));
        assert!(strncmp_eq("abc", "abd", 2));
        assert!(!strncmp_eq("abc", "abd", 3));
        // Shorter left side compares against NUL past the end.
        assert!(!strncmp_eq("ab", "abc", 3));
    }

    #[test]
    fn validators_accept_and_reject_expected_ranges() {
        assert_eq!(is_user_id("1"), Some(1));
        assert_eq!(is_user_id("63"), Some(63));
        assert_eq!(is_user_id("0"), None);
        assert_eq!(is_user_id("64"), None);

        assert_eq!(is_channel_num("0xb"), Some(0xb));
        assert_eq!(is_channel_num("0xe"), Some(0xe));
        assert_eq!(is_channel_num("0xc"), None);

        assert_eq!(is_priv_limit("4"), Some(4));
        assert_eq!(is_priv_limit("0xf"), Some(0xf));
        assert_eq!(is_priv_limit("6"), None);
    }

    #[test]
    fn password_flags_byte_encodes_length_and_uid() {
        // 16-byte password: high bit clear. 20-byte password: high bit set.
        assert_eq!(uid(5), 5);
        assert_eq!(0x80u8 | uid(5), 0x85);
        assert_eq!(0x80u8 | uid(0x41), 0x81);
    }
}
