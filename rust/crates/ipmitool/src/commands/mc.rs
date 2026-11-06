//! Management Controller commands. Mirrors the `mc`/`bmc` commands in
//! `lib/ipmi_mc.c` (info, reset, selftest, getenables, setenables, guid).

use ipmi_core::cc;
use ipmi_core::consts;
use ipmi_core::interface::Interface;
use ipmi_core::types::Request;

/// Additional device support bits, indexed by bit number.
const ADTL_DEV_SUPPORT: [&str; 8] = [
    "Sensor Device",
    "SDR Repository Device",
    "SEL Device",
    "FRU Inventory Device",
    "IPMB Event Receiver",
    "IPMB Event Generator",
    "Bridge",
    "Chassis Device",
];

struct EnablesBit {
    name: &'static str,
    desc: &'static str,
    mask: u8,
}

const MC_ENABLES: [EnablesBit; 7] = [
    EnablesBit {
        name: "recv_msg_intr",
        desc: "Receive Message Queue Interrupt",
        mask: 1 << 0,
    },
    EnablesBit {
        name: "event_msg_intr",
        desc: "Event Message Buffer Full Interrupt",
        mask: 1 << 1,
    },
    EnablesBit {
        name: "event_msg",
        desc: "Event Message Buffer",
        mask: 1 << 2,
    },
    EnablesBit {
        name: "system_event_log",
        desc: "System Event Logging",
        mask: 1 << 3,
    },
    EnablesBit {
        name: "oem0",
        desc: "OEM 0",
        mask: 1 << 5,
    },
    EnablesBit {
        name: "oem1",
        desc: "OEM 1",
        mask: 1 << 6,
    },
    EnablesBit {
        name: "oem2",
        desc: "OEM 2",
        mask: 1 << 7,
    },
];

pub fn main(intf: &mut dyn Interface, args: &[String]) -> i32 {
    let Some(cmd) = args.first().map(String::as_str) else {
        eprintln!("Not enough parameters given.");
        print_usage();
        return -1;
    };
    let rest = &args[1..];
    match cmd {
        "help" => {
            print_usage();
            0
        }
        "reset" => reset(intf, rest),
        "info" => info(intf),
        "guid" => guid(intf, rest),
        "getenables" => get_enables(intf),
        "setenables" => set_enables(intf, rest),
        "selftest" => selftest(intf),
        other => {
            eprintln!("Invalid mc command: {other}");
            -1
        }
    }
}

fn info(intf: &mut dyn Interface) -> i32 {
    let req = Request::new(consts::NETFN_APP, consts::IPMI_GET_DEVICE_ID);
    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => {
            eprintln!("Get Device ID command failed");
            return -1;
        }
    };
    if !rsp.ok() {
        eprintln!(
            "Get Device ID command failed: {}",
            cc::completion_code_str(rsp.ccode)
        );
        return -1;
    }
    let d = &rsp.data;
    if d.len() < 11 {
        eprintln!("Get Device ID command failed: short response");
        return -1;
    }

    let device_id = d[0];
    let device_revision = d[1];
    let fw_rev1 = d[2];
    let fw_rev2 = d[3];
    let ipmi_version = d[4];
    let adtl = d[5];
    let manufacturer = d[6] as u32 | (d[7] as u32) << 8 | (d[8] as u32) << 16;
    let product_id = d[9] as u16 | (d[10] as u16) << 8;

    println!("Device ID                 : {device_id}");
    println!("Device Revision           : {}", device_revision & 0x0f);
    println!(
        "Firmware Revision         : {}.{:02x}",
        fw_rev1 & 0x7f,
        fw_rev2
    );
    println!(
        "IPMI Version              : {:x}.{:x}",
        ipmi_version & 0x0f,
        (ipmi_version & 0xf0) >> 4
    );
    let registry = ipmi_core::oem::OemRegistry::load();

    println!("Manufacturer ID           : {manufacturer}");
    println!(
        "Manufacturer Name         : {}",
        registry.manufacturer_name(manufacturer)
    );
    println!(
        "Product ID                : {product_id} (0x{:02x}{:02x})",
        d[10], d[9]
    );
    println!(
        "Product Name              : {}",
        registry.product_name(manufacturer, product_id)
    );
    println!(
        "Device Available          : {}",
        if fw_rev1 & 0x80 != 0 { "no" } else { "yes" }
    );
    println!(
        "Provides Device SDRs      : {}",
        if device_revision & 0x80 != 0 {
            "yes"
        } else {
            "no"
        }
    );
    println!("Additional Device Support :");
    for (i, name) in ADTL_DEV_SUPPORT.iter().enumerate() {
        if adtl & (1 << i) != 0 {
            println!("    {name}");
        }
    }
    if d.len() == 15 {
        println!("Aux Firmware Rev Info     : ");
        println!(
            "    0x{:02x}\n    0x{:02x}\n    0x{:02x}\n    0x{:02x}",
            d[11], d[12], d[13], d[14]
        );
    }
    0
}

fn reset(intf: &mut dyn Interface, args: &[String]) -> i32 {
    let Some(kind) = args.first().map(String::as_str) else {
        eprintln!("Not enough parameters given.");
        eprintln!("usage: mc reset <warm|cold>");
        return -1;
    };
    if kind == "help" {
        eprintln!("usage: mc reset <warm|cold>");
        return 0;
    }
    let (cmd, label) = match kind {
        "cold" => (0x02u8, "cold"),
        "warm" => (0x03u8, "warm"),
        other => {
            eprintln!("Invalid mc reset command: {other}");
            eprintln!("usage: mc reset <warm|cold>");
            return -1;
        }
    };

    let req = Request::new(consts::NETFN_APP, cmd);
    let rsp = intf.sendrecv(&req);
    match rsp {
        // A cold reset commonly gets no answer; that is expected.
        Err(_) if label == "cold" => {}
        Err(_) => {
            eprintln!("MC reset command failed.");
            return -1;
        }
        Ok(rsp) if !rsp.ok() => {
            eprintln!(
                "MC reset command failed: {}",
                cc::completion_code_str(rsp.ccode)
            );
            return -1;
        }
        Ok(_) => {}
    }
    println!("Sent {label} reset command to MC");
    0
}

fn get_enables(intf: &mut dyn Interface) -> i32 {
    let req = Request::new(consts::NETFN_APP, consts::IPMI_GET_GLOBAL_ENABLES);
    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => {
            eprintln!("Get Global Enables command failed");
            return -1;
        }
    };
    if !rsp.ok() {
        eprintln!(
            "Get Global Enables command failed: {}",
            cc::completion_code_str(rsp.ccode)
        );
        return -1;
    }
    let en = rsp.data.first().copied().unwrap_or(0);
    for bf in &MC_ENABLES {
        println!(
            "{:<40} : {}abled",
            bf.desc,
            if en & bf.mask != 0 { "en" } else { "dis" }
        );
    }
    0
}

fn set_enables(intf: &mut dyn Interface, args: &[String]) -> i32 {
    if args.is_empty() {
        print_usage();
        return -1;
    }
    if args[0] == "help" {
        print_usage();
        return 0;
    }

    let req = Request::new(consts::NETFN_APP, consts::IPMI_GET_GLOBAL_ENABLES);
    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => {
            eprintln!("Get Global Enables command failed");
            return -1;
        }
    };
    if !rsp.ok() {
        eprintln!(
            "Get Global Enables command failed: {}",
            cc::completion_code_str(rsp.ccode)
        );
        return -1;
    }
    let original = rsp.data.first().copied().unwrap_or(0);
    let mut en = original;

    for arg in args {
        for bf in &MC_ENABLES {
            let prefix = format!("{}=", bf.name);
            let Some(setting) = arg.strip_prefix(&prefix) else {
                continue;
            };
            match setting {
                "off" => {
                    println!("Disabling {}", bf.desc);
                    en &= !bf.mask;
                }
                "on" => {
                    println!("Enabling {}", bf.desc);
                    en |= bf.mask;
                }
                _ => eprintln!("Unrecognized option: {arg}"),
            }
        }
    }

    if en == original {
        println!("\nNothing to change...");
        return get_enables(intf);
    }

    let mut set = Request::new(consts::NETFN_APP, consts::IPMI_SET_GLOBAL_ENABLES);
    set.data = vec![en];
    let rsp = match intf.sendrecv(&set) {
        Ok(rsp) => rsp,
        Err(_) => {
            eprintln!("Set Global Enables command failed");
            return -1;
        }
    };
    if !rsp.ok() {
        eprintln!(
            "Set Global Enables command failed: {}",
            cc::completion_code_str(rsp.ccode)
        );
        return -1;
    }

    println!("\nVerifying...");
    get_enables(intf)
}

fn selftest(intf: &mut dyn Interface) -> i32 {
    let req = Request::new(consts::NETFN_APP, consts::IPMI_GET_SELF_TEST_RESULTS);
    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => {
            eprintln!("No response from devices");
            return -1;
        }
    };
    if !rsp.ok() {
        eprintln!("Bad response: ({})", cc::completion_code_str(rsp.ccode));
        return -1;
    }
    let code = rsp.data.first().copied().unwrap_or(0);
    let test = rsp.data.get(1).copied().unwrap_or(0);
    match code {
        0x55 => {
            println!("Selftest: passed");
            0
        }
        0x56 => {
            println!("Selftest: not implemented");
            -1
        }
        0x57 => {
            println!("Selftest: device corrupted");
            const CHECKS: [(u8, &str); 8] = [
                (0x80, " -> SEL device not accessible"),
                (0x40, " -> SDR repository not accessible"),
                (0x20, "FRU device not accessible"),
                (0x10, "IPMB signal lines do not respond"),
                (0x08, "SDR repository empty"),
                (0x04, "Internal Use Area corrupted"),
                (0x02, "Controller update boot block corrupted"),
                (0x01, "controller operational firmware corrupted"),
            ];
            for (mask, msg) in CHECKS {
                if test & mask != 0 {
                    println!("{msg}");
                }
            }
            -1
        }
        0x58 => {
            println!("Selftest     : fatal error");
            println!("Failure code : {test:02x}");
            -1
        }
        0xff => {
            println!("Selftest: reserved");
            -1
        }
        _ => {
            println!("Selftest: unknown status");
            -1
        }
    }
}

// ---------------------------------------------------------------------------
// GUID handling
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum GuidMode {
    Rfc4122,
    Ipmi,
    Smbios,
    Auto,
    Dump,
}

fn guid(intf: &mut dyn Interface, args: &[String]) -> i32 {
    let mode = match args.first().map(String::as_str) {
        Some("rfc") | Some("rfc4122") => GuidMode::Rfc4122,
        Some("smbios") => GuidMode::Smbios,
        Some("ipmi") => GuidMode::Ipmi,
        Some("auto") => GuidMode::Auto,
        Some("dump") => GuidMode::Dump,
        Some(other) if other != "help" => {
            eprintln!("Invalid GUID mode: {other}");
            return -1;
        }
        _ => GuidMode::Auto,
    };

    let req = Request::new(consts::NETFN_APP, 0x37); // Get System GUID
    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => {
            eprintln!("Get GUID command failed");
            return -1;
        }
    };
    if !rsp.ok() {
        eprintln!(
            "Get GUID command failed: {}",
            cc::completion_code_str(rsp.ccode)
        );
        return -1;
    }
    if rsp.data.len() < 16 {
        eprintln!("Get GUID command failed: short response");
        return -1;
    }
    let data: [u8; 16] = rsp.data[..16].try_into().unwrap();

    let parsed = parse_guid(&data, mode);
    println!("System GUID   : {parsed}");

    if mode == GuidMode::Auto {
        print!("GUID Encoding : {}", parsed.mode_name());
        if parsed.mode != GuidMode::Ipmi {
            print!(" (WARNING: IPMI Specification violation!)");
        }
        println!();
    }
    if parsed.mode != GuidMode::Dump {
        print!("GUID Version  : {}", parsed.version_name());
        if parsed.version == 0 {
            println!(" ({})", parsed.version_nibble());
        } else if parsed.version == 1 {
            println!();
            println!("Timestamp     : {}", format_unix(parsed.time));
        } else {
            println!();
        }
    }
    0
}

struct ParsedGuid {
    mode: GuidMode,
    version: u8,
    time_low: u32,
    time_mid: u16,
    time_hi: u16,
    clock: u16,
    node: [u8; 6],
    time: i64,
    raw: [u8; 16],
}

impl ParsedGuid {
    fn version_nibble(&self) -> u8 {
        ((self.raw[8] as u16 | (self.raw[9] as u16) << 8) >> 12) as u8 & 0xf
    }

    fn version_name(&self) -> &'static str {
        match self.version {
            0 => "Unknown/unsupported",
            1 => "Time-based",
            2 => "DCE Security with POSIX UIDs (not for IPMI)",
            3 => "Name-based using MD5",
            4 => "Random or pseudo-random",
            5 => "Name-based using SHA-1",
            _ => "Unknown/unsupported",
        }
    }

    fn mode_name(&self) -> &'static str {
        match self.mode {
            GuidMode::Ipmi => "IPMI",
            GuidMode::Rfc4122 => "RFC4122",
            GuidMode::Smbios => "SMBIOS",
            GuidMode::Dump => "Unknown (data dumped)",
            GuidMode::Auto => "Automatic (if you see this, report a bug)",
        }
    }
}

impl std::fmt::Display for ParsedGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.mode == GuidMode::Dump {
            let s: Vec<String> = self.raw.iter().map(|b| format!("{b:02x}")).collect();
            return f.write_str(&s.join(""));
        }
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.time_low,
            self.time_mid,
            self.time_hi,
            self.clock,
            self.node[0],
            self.node[1],
            self.node[2],
            self.node[3],
            self.node[4],
            self.node[5]
        )
    }
}

fn be16(b: &[u8]) -> u16 {
    u16::from_be_bytes([b[0], b[1]])
}
fn be32(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}
fn le16(b: &[u8]) -> u16 {
    u16::from_le_bytes([b[0], b[1]])
}
fn le32(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

fn parse_guid(data: &[u8; 16], requested: GuidMode) -> ParsedGuid {
    // Fields for each interpretation.
    // ipmi_guid_t: node[0..6], clock le[6..8], time_hi le[8..10],
    //              time_mid le[10..12], time_low le[12..16]
    // rfc_guid_t:  time_low be[0..4], time_mid be[4..6], time_hi be[6..8],
    //              clock be[8..10], node[10..16]
    let modes = [GuidMode::Rfc4122, GuidMode::Ipmi, GuidMode::Smbios];

    let mut chosen = if matches!(requested, GuidMode::Auto | GuidMode::Dump) {
        None
    } else {
        Some(requested)
    };

    if requested == GuidMode::Auto {
        for m in modes {
            let (t_hi, _, _, _) = guid_fields(data, m);
            let ver = ((t_hi >> 12) & 0xf) as u8;
            if (1..=5).contains(&ver) {
                chosen = Some(m);
                if ver == 1 {
                    let seconds = guid_time(data, m);
                    if time_valid(seconds) {
                        break;
                    }
                }
            }
        }
    }

    let Some(mode) = chosen else {
        return ParsedGuid {
            mode: (if requested == GuidMode::Auto {
                GuidMode::Dump
            } else {
                requested
            }),
            version: 0,
            time_low: 0,
            time_mid: 0,
            time_hi: 0,
            clock: 0,
            node: [0; 6],
            time: 0,
            raw: *data,
        };
    };

    let (time_hi, time_mid, time_low, clock) = guid_fields(data, mode);
    let version = ((time_hi >> 12) & 0xf) as u8;
    let version = if version > 5 { 0 } else { version };
    let node = if mode == GuidMode::Ipmi {
        let mut n = [0u8; 6];
        for (i, b) in data[..6].iter().rev().enumerate() {
            n[i] = *b;
        }
        n
    } else {
        let mut n = [0u8; 6];
        n.copy_from_slice(&data[10..16]);
        n
    };
    let time = if version == 1 {
        guid_time(data, mode)
    } else {
        0
    };

    ParsedGuid {
        mode,
        version,
        time_low,
        time_mid,
        time_hi,
        clock,
        node,
        time,
        raw: *data,
    }
}

fn guid_fields(data: &[u8; 16], mode: GuidMode) -> (u16, u16, u32, u16) {
    match mode {
        GuidMode::Ipmi => (
            le16(&data[8..10]),
            le16(&data[10..12]),
            le32(&data[12..16]),
            le16(&data[6..8]),
        ),
        GuidMode::Rfc4122 => (
            be16(&data[6..8]),
            be16(&data[4..6]),
            be32(&data[0..4]),
            be16(&data[8..10]),
        ),
        GuidMode::Smbios => (
            le16(&data[6..8]),
            le16(&data[4..6]),
            le32(&data[0..4]),
            be16(&data[8..10]),
        ),
        _ => (0, 0, 0, 0),
    }
}

fn guid_time(data: &[u8; 16], mode: GuidMode) -> i64 {
    let (time_hi, time_mid, time_low, _) = guid_fields(data, mode);
    let hi = (time_hi & 0x0fff) as u64; // strip version nibble
    let gregorian = (hi << 48) | ((time_mid as u64) << 32) | time_low as u64;
    let seconds = (gregorian / 10_000_000) as i64;
    seconds - 12_219_292_800
}

fn time_valid(t: i64) -> bool {
    // Between the Unix epoch and now (with a little slack for clock skew).
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    t >= 0 && t <= now + 86400
}

/// Format a Unix timestamp as `YYYY-MM-DD HH:MM:SS UTC` without pulling in an
/// external date crate.
fn format_unix(t: i64) -> String {
    if t < 0 {
        return "Unspecified".to_string();
    }
    let days = t / 86400;
    let secs = t % 86400;
    let (year, month, day) = civil_from_days(days);
    let hour = secs / 3600;
    let minute = (secs % 3600) / 60;
    let second = secs % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} UTC")
}

/// Howard Hinnant's `civil_from_days` algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn print_usage() {
    eprintln!(
        "MC Commands:\n  info, reset, selftest, getenables, setenables, guid,\n  watchdog, getsysinfo, setsysinfo"
    );
}
