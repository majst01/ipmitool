//! The `fru` command: read and print Field Replaceable Unit inventory.
//!
//! Mirrors the read paths of `lib/ipmi_fru.c` (`fru print`, `fru list`,
//! `fru read`). The destructive `write`/`edit` commands and the PICMG OEM
//! extension record decoder are not implemented yet; see `rust/README.md`.

use std::fs::File;
use std::io::Write as _;

use ipmi_core::cc;
use ipmi_core::consts;
use ipmi_core::interface::Interface;
use ipmi_core::oem::OemRegistry;
use ipmi_core::types::Request;

use crate::commands::mc;
use crate::verbose;

const NETFN_STORAGE: u8 = consts::NETFN_STORAGE;
const GET_FRU_INFO: u8 = 0x10;
const GET_FRU_DATA: u8 = 0x11;

const FRU_END_OF_FIELDS: u8 = 0xc1;
const FRU_BLOCK_SZ: u32 = 8;
const FRU_AREA_MAXIMUM_BLOCK_SZ: u32 = 32;

/// `IPMI_LAN_MAX_RESPONSE_SIZE` from `src/plugins/lanplus/lanplus.c`. The FRU
/// reader derives its per-request read size from this (see [`read_fru_area`]).
const LANPLUS_MAX_RESPONSE_SIZE: u16 = 34;

const FRU_RECORD_TYPE_POWER_SUPPLY_INFORMATION: u8 = 0x00;
const FRU_RECORD_TYPE_DC_OUTPUT: u8 = 0x01;
const FRU_RECORD_TYPE_DC_LOAD: u8 = 0x02;
const FRU_RECORD_TYPE_MANAGEMENT_ACCESS: u8 = 0x03;
const FRU_RECORD_TYPE_OEM_EXTENSION: u8 = 0xc0;
const FRU_RECORD_FORMAT_EOL_MASK: u8 = 0x80;

const COMBINED_VOLTAGE_DESC: [&str; 4] = ["12 V", "-12 V", "5 V", "3.3 V"];

const CHASSIS_TYPE_DESC: [&str; 30] = [
    "Unspecified",
    "Other",
    "Unknown",
    "Desktop",
    "Low Profile Desktop",
    "Pizza Box",
    "Mini Tower",
    "Tower",
    "Portable",
    "LapTop",
    "Notebook",
    "Hand Held",
    "Docking Station",
    "All in One",
    "Sub Notebook",
    "Space-saving",
    "Lunch Box",
    "Main Server Chassis",
    "Expansion Chassis",
    "SubChassis",
    "Bus Expansion Chassis",
    "Peripheral Chassis",
    "RAID Chassis",
    "Rack Mount Chassis",
    "Sealed-case PC",
    "Multi-system Chassis",
    "CompactPCI",
    "AdvancedTCA",
    "Blade",
    "Blade Enclosure",
];

/// Information returned by `Get FRU Inventory Area Info`.
#[derive(Clone, Copy)]
struct FruInfo {
    size: u32,
    access: bool,
    max_read_size: u32,
}

impl FruInfo {
    fn new() -> Self {
        FruInfo {
            size: 0,
            access: false,
            max_read_size: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Command dispatch (`ipmi_fru_main`).
// ---------------------------------------------------------------------------

pub fn main(intf: &mut dyn Interface, args: &[String]) -> i32 {
    let Some(cmd) = args.first().map(String::as_str) else {
        return print_all(intf);
    };

    match cmd {
        "help" => {
            print_help();
            0
        }
        "print" | "list" => {
            if args.len() > 1 {
                if args[1] == "help" {
                    eprintln!("fru print [fru id] - print information about FRU(s)");
                    return 0;
                }
                match is_fru_id(&args[1]) {
                    Some(id) => print_device(intf, id),
                    None => -1,
                }
            } else {
                print_all(intf)
            }
        }
        "read" => read(intf, &args[1..]),
        "write" | "edit" | "get" | "internaluse" | "upgEkey" => {
            eprintln!("fru {cmd}: not implemented in this build");
            -1
        }
        other => {
            eprintln!("Invalid FRU command: {other}");
            print_help();
            -1
        }
    }
}

/// Parse a FRU id, printing the C error message on failure.
fn is_fru_id(s: &str) -> Option<u8> {
    match crate::args::parse_u8(s) {
        Some(id) => Some(id),
        None => {
            eprintln!("FRU ID '{s}' is either invalid or out of range.");
            None
        }
    }
}

fn print_help() {
    eprintln!("FRU Commands:  print read write upgEkey edit internaluse get");
}

// ---------------------------------------------------------------------------
// `fru read` (`ipmi_fru_read_to_bin`).
// ---------------------------------------------------------------------------

fn read(intf: &mut dyn Interface, args: &[String]) -> i32 {
    if args.first().map(String::as_str) == Some("help") {
        eprintln!("fru read <fru id> <fru file>");
        eprintln!("Note: FRU ID and file(incl. full path) must be specified.");
        eprintln!("Example: ipmitool fru read 0 /root/fru.bin");
        return 0;
    }
    if args.len() < 2 {
        eprintln!("Not enough parameters given.");
        eprintln!("fru read <fru id> <fru file>");
        return -1;
    }
    let Some(fru_id) = is_fru_id(&args[0]) else {
        return -1;
    };
    let file = &args[1];
    if is_valid_filename(file) != 0 {
        return -1;
    }
    if verbose::level() > 0 {
        println!("FRU ID           : {fru_id}");
        println!("FRU File         : {file}");
    }
    read_to_bin(intf, file, fru_id);

    0
}

fn is_valid_filename(name: &str) -> i32 {
    if name.is_empty() {
        eprintln!("File/path is invalid.");
        return -2;
    }
    if name.len() >= 512 {
        eprintln!("File/path must be shorter than 512 bytes.");
        return -3;
    }
    0
}

fn read_to_bin(intf: &mut dyn Interface, file_name: &str, fru_id: u8) {
    let mut fru = match query_fru_info(intf, fru_id) {
        Ok(fru) => fru,
        Err(FruInfoError::NoResponse) => return,
        Err(FruInfoError::Ccode(c)) => {
            if c == cc::CC_TIMEOUT {
                println!("  Timeout accessing FRU info. (Device not present?)");
            }
            return;
        }
    };

    if verbose::level() > 0 {
        println!("Fru Size   = {} bytes", fru.size);
        println!("Fru Access = {:x}h", if fru.access { 1 } else { 0 });
    }

    println!("Fru Size         : {} bytes", fru.size);
    let size = fru.size;
    let data = read_fru_area(intf, &mut fru, fru_id, 0, size).unwrap_or_default();

    match File::create(file_name) {
        Ok(mut f) => {
            if let Err(e) = f.write_all(&data) {
                eprintln!("Error writing file {file_name}: {e}");
                return;
            }
            println!("Done");
        }
        Err(_) => eprintln!("Error opening file {file_name}"),
    }
}

// ---------------------------------------------------------------------------
// `fru print` (`__ipmi_fru_print` and `ipmi_fru_print_all`).
// ---------------------------------------------------------------------------

fn print_device(intf: &mut dyn Interface, id: u8) -> i32 {
    let mut fru = match query_fru_info(intf, id) {
        Ok(fru) => fru,
        Err(FruInfoError::NoResponse) => {
            println!(" Device not present (No Response)");
            return -1;
        }
        Err(FruInfoError::Ccode(c)) => {
            println!(" Device not present ({})", cc::completion_code_str(c));
            return -1;
        }
    };

    if fru.size < 1 {
        eprintln!(" Invalid FRU size {}", fru.size);
        return -1;
    }

    // Retrieve the common header (8 bytes).
    let header = match read_fru_area(intf, &mut fru, id, 0, 8) {
        Some(h) => h,
        None => return 1,
    };
    if verbose::level() > 1 {
        printbuf(&header, "FRU DATA");
    }
    // read_fru_area returns 8 bytes; the caller-visible header starts at 1 in
    // the raw response, but here we read the FRU data directly.
    let header = &header[..8];

    if header[0] != 1 {
        eprintln!(" Unknown FRU header version 0x{:02x}", header[0]);
        return -1;
    }

    let off_internal = header[1] as u32 * 8;
    let off_chassis = header[2] as u32 * 8;
    let off_board = header[3] as u32 * 8;
    let off_product = header[4] as u32 * 8;
    let off_multi = header[5] as u32 * 8;

    if verbose::level() > 0 {
        log_debug(&format!("fru.header.version:         0x{:x}", header[0]));
        log_debug(&format!("fru.header.offset.internal: 0x{off_internal:x}"));
        log_debug(&format!("fru.header.offset.chassis:  0x{off_chassis:x}"));
        log_debug(&format!("fru.header.offset.board:    0x{off_board:x}"));
        log_debug(&format!("fru.header.offset.product:  0x{off_product:x}"));
        log_debug(&format!("fru.header.offset.multi:    0x{off_multi:x}"));
    }

    if off_chassis >= 8 {
        print_chassis(intf, &mut fru, id, off_chassis);
    }
    if off_board >= 8 {
        print_board(intf, &mut fru, id, off_board);
    }
    if off_product >= 8 {
        print_product(intf, &mut fru, id, off_product);
    }

    if verbose::level() == 0 {
        return 0;
    }
    if off_multi >= 8 {
        print_multirec(intf, &mut fru, id, off_multi);
    }
    0
}

fn print_all(intf: &mut dyn Interface) -> i32 {
    println!("FRU Device Description : Builtin FRU Device (ID 0)");

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
    // `adtl_device_support` (FRU Inventory Device bit) is byte 5.
    let adtl = rsp.data.get(5).copied().unwrap_or(0);
    let mut rc = 0;
    if adtl & 0x08 != 0 {
        rc = print_device(intf, 0);
        println!();
    }

    // The C implementation then walks the SDR repository for FRU and MC
    // locator records. SDR iteration is not implemented in this build, so only
    // the builtin device is printed (see rust/README.md).
    rc
}

/// Failure modes of `Get FRU Inventory Area Info`, matching the two error
/// branches (transport failure vs. a completion code) that the C callers report
/// differently.
enum FruInfoError {
    /// The request got no response at all.
    NoResponse,
    /// The BMC returned a non-zero completion code.
    Ccode(u8),
}

/// Send `Get FRU Inventory Area Info` for FRU `id`.
fn query_fru_info(intf: &mut dyn Interface, id: u8) -> Result<FruInfo, FruInfoError> {
    let req = Request::new(NETFN_STORAGE, GET_FRU_INFO).with_data([id]);
    let rsp = intf.sendrecv(&req).map_err(|_| FruInfoError::NoResponse)?;
    if !rsp.ok() {
        return Err(FruInfoError::Ccode(rsp.ccode));
    }
    if rsp.data.len() < 3 {
        return Err(FruInfoError::NoResponse);
    }
    let mut fru = FruInfo::new();
    fru.size = (rsp.data[1] as u32) << 8 | rsp.data[0] as u32;
    fru.access = rsp.data[2] & 0x1 != 0;
    Ok(fru)
}

// ---------------------------------------------------------------------------
// Section printers.
// ---------------------------------------------------------------------------

fn print_chassis(intf: &mut dyn Interface, fru: &mut FruInfo, id: u8, offset: u32) {
    let Some(tmp) = read_fru_area(intf, fru, id, offset, 2) else {
        return;
    };
    let fru_len = 8 * tmp[1] as usize;
    if fru_len == 0 {
        return;
    }
    let Some(data) = read_fru_area(intf, fru, id, offset, fru_len as u32) else {
        return;
    };

    let chassis_type = if data[2] as usize > CHASSIS_TYPE_DESC.len() - 1 {
        2
    } else {
        data[2] as usize
    };
    println!(
        " Chassis Type          : {}",
        CHASSIS_TYPE_DESC[chassis_type]
    );

    let mut i = 3usize;
    print_field(&data, &mut i, " Chassis Part Number   : ");
    print_field(&data, &mut i, " Chassis Serial        : ");

    while i < fru_len && data.get(i) != Some(&FRU_END_OF_FIELDS) {
        let j = i;
        print_field(&data, &mut i, " Chassis Extra         : ");
        if i == j {
            break;
        }
    }

    println!(
        " Chassis Area Checksum : {}",
        if checksum_is_valid(&data, fru_len) {
            "OK"
        } else {
            "INVALID"
        }
    );
}

fn print_board(intf: &mut dyn Interface, fru: &mut FruInfo, id: u8, offset: u32) {
    let Some(tmp) = read_fru_area(intf, fru, id, offset, 2) else {
        return;
    };
    let fru_len = 8 * tmp[1] as usize;
    if fru_len == 0 {
        return;
    }
    let Some(data) = read_fru_area(intf, fru, id, offset, fru_len as u32) else {
        return;
    };

    let mut i = 3usize;
    println!(
        " Board Mfg Date        : {}",
        format_fru_timestamp(&data, i)
    );
    i += 3;

    print_field(&data, &mut i, " Board Mfg             : ");
    print_field(&data, &mut i, " Board Product         : ");
    print_field(&data, &mut i, " Board Serial          : ");
    print_field(&data, &mut i, " Board Part Number     : ");
    print_field_verbose(&data, &mut i, " Board FRU ID          : ");

    while i < fru_len && data.get(i) != Some(&FRU_END_OF_FIELDS) {
        let j = i;
        print_field(&data, &mut i, " Board Extra           : ");
        if i == j {
            break;
        }
    }

    println!(
        " Board Area Checksum   : {}",
        if checksum_is_valid(&data, fru_len) {
            "OK"
        } else {
            "INVALID"
        }
    );
}

fn print_product(intf: &mut dyn Interface, fru: &mut FruInfo, id: u8, offset: u32) {
    let Some(tmp) = read_fru_area(intf, fru, id, offset, 2) else {
        return;
    };
    let fru_len = 8 * tmp[1] as usize;
    if fru_len == 0 {
        return;
    }
    let Some(data) = read_fru_area(intf, fru, id, offset, fru_len as u32) else {
        return;
    };

    let mut i = 3usize;
    print_field(&data, &mut i, " Product Manufacturer  : ");
    print_field(&data, &mut i, " Product Name          : ");
    print_field(&data, &mut i, " Product Part Number   : ");
    print_field(&data, &mut i, " Product Version       : ");
    print_field(&data, &mut i, " Product Serial        : ");
    print_field(&data, &mut i, " Product Asset Tag     : ");
    print_field_verbose(&data, &mut i, " Product FRU ID        : ");

    while i < fru_len && data.get(i) != Some(&FRU_END_OF_FIELDS) {
        let j = i;
        print_field(&data, &mut i, " Product Extra         : ");
        if i == j {
            break;
        }
    }

    println!(
        " Product Area Checksum : {}",
        if checksum_is_valid(&data, fru_len) {
            "OK"
        } else {
            "INVALID"
        }
    );
}

/// Read one FRU area string at `*i`, print it if non-empty, and advance `*i`.
fn print_field(data: &[u8], i: &mut usize, label: &str) {
    if let Some(s) = get_fru_area_str(data, i) {
        if !s.is_empty() {
            println!("{label}{s}");
        }
    }
}

/// Like [`print_field`] but only printed at verbosity >= 1 (FRU ID fields).
fn print_field_verbose(data: &[u8], i: &mut usize, label: &str) {
    if let Some(s) = get_fru_area_str(data, i) {
        if !s.is_empty() && verbose::level() > 0 {
            println!("{label}{s}");
        }
    }
}

// ---------------------------------------------------------------------------
// Multi-record area printer.
// ---------------------------------------------------------------------------

fn print_multirec(intf: &mut dyn Interface, fru: &mut FruInfo, id: u8, offset: u32) {
    let mut last_off = offset;
    let mut registry: Option<OemRegistry> = None;

    while let Some(hdr) = read_fru_area(intf, fru, id, last_off, 5) {
        let rtype = hdr[0];
        let format = hdr[1];
        let rlen = hdr[2] as u32;

        let body = if rlen > 0 {
            match read_fru_area(intf, fru, id, last_off + 5, rlen) {
                Some(b) => b,
                None => break,
            }
        } else {
            Vec::new()
        };
        last_off += rlen + 5;

        match rtype {
            FRU_RECORD_TYPE_POWER_SUPPLY_INFORMATION => print_power_supply(&body),
            FRU_RECORD_TYPE_DC_OUTPUT => print_dc_output(&body),
            FRU_RECORD_TYPE_DC_LOAD => print_dc_load(&body),
            FRU_RECORD_TYPE_MANAGEMENT_ACCESS => print_management_access(&body, rlen),
            FRU_RECORD_TYPE_OEM_EXTENSION => {
                let iana = u32::from(byte(&body, 0))
                    | u32::from(byte(&body, 1)) << 8
                    | u32::from(byte(&body, 2)) << 16;
                if iana == ipmi_core::oem::IPMI_OEM_PICMG {
                    println!("  PICMG Extension Record");
                    eprintln!(
                        "  (PICMG extension record decoding is not implemented in this build)"
                    );
                } else {
                    let reg = registry.get_or_insert_with(OemRegistry::load);
                    println!("  OEM ({}) Record", reg.manufacturer_name(iana));
                }
            }
            _ => {}
        }

        if format & FRU_RECORD_FORMAT_EOL_MASK != 0 {
            break;
        }
    }

    log_debug(&format!(
        "Multi-Record area ends at: {} ({:x}h)",
        last_off, last_off
    ));
}

fn print_power_supply(d: &[u8]) {
    let capacity = le16(d, 0) & 0x0fff;
    let peak_va = le16(d, 2);
    let inrush_current = byte(d, 4);
    let inrush_interval = byte(d, 5);
    let lowend_input1 = le16(d, 6);
    let highend_input1 = le16(d, 8);
    let lowend_input2 = le16(d, 10);
    let highend_input2 = le16(d, 12);
    let lowend_freq = byte(d, 14);
    let highend_freq = byte(d, 15);
    let dropout_tolerance = byte(d, 16);
    let flags = byte(d, 17);
    let predictive_fail = flags & 0x01 != 0;
    let pfc = flags & 0x02 != 0;
    let autoswitch = flags & 0x04 != 0;
    let hotswap = flags & 0x08 != 0;
    let tach = flags & 0x10 != 0;
    let peak_cap_ht = le16(d, 18);
    let peak_hold_up_time = (peak_cap_ht & 0xf000) >> 12;
    let peak_capacity = peak_cap_ht & 0x0fff;
    let combined_byte = byte(d, 20);
    let combined_voltage1 = (combined_byte >> 4) & 0x0f;
    let combined_voltage2 = combined_byte & 0x0f;
    let combined_capacity = le16(d, 21);
    let rps_threshold = byte(d, 23);

    println!(" Power Supply Record");
    println!("  Capacity                   : {capacity} W");
    println!("  Peak VA                    : {peak_va} VA");
    println!("  Inrush Current             : {inrush_current} A");
    println!("  Inrush Interval            : {inrush_interval} ms");
    println!(
        "  Input Voltage Range 1      : {}-{} V",
        lowend_input1 / 100,
        highend_input1 / 100
    );
    println!(
        "  Input Voltage Range 2      : {}-{} V",
        lowend_input2 / 100,
        highend_input2 / 100
    );
    println!("  Input Frequency Range      : {lowend_freq}-{highend_freq} Hz");
    println!("  A/C Dropout Tolerance      : {dropout_tolerance} ms");

    let tail = if predictive_fail {
        if rps_threshold != 0 {
            if tach {
                "'Two pulses per rotation'"
            } else {
                "'One pulse per rotation'"
            }
        } else if tach {
            "'Failure on pin de-assertion'"
        } else {
            "'Failure on pin assertion'"
        }
    } else {
        ""
    };
    println!(
        "  Flags                      : {}{}{}{}{}",
        if predictive_fail {
            "'Predictive fail' "
        } else {
            ""
        },
        if pfc {
            "'Power factor correction' "
        } else {
            ""
        },
        if autoswitch {
            "'Autoswitch voltage' "
        } else {
            ""
        },
        if hotswap { "'Hot swap' " } else { "" },
        tail
    );
    println!("  Peak capacity              : {peak_capacity} W");
    println!("  Peak capacity holdup       : {peak_hold_up_time} s");
    if combined_capacity == 0 {
        println!("  Combined capacity          : not specified");
    } else {
        println!(
            "  Combined capacity          : {} W ({} and {})",
            combined_capacity,
            voltage_desc(combined_voltage1),
            voltage_desc(combined_voltage2)
        );
    }
    if predictive_fail {
        println!("  Fan lower threshold        : {rps_threshold} RPS");
    }
}

fn voltage_desc(idx: u8) -> &'static str {
    COMBINED_VOLTAGE_DESC
        .get(idx as usize)
        .copied()
        .unwrap_or("")
}

fn print_dc_output(d: &[u8]) {
    let b0 = byte(d, 0);
    let output_number = b0 & 0x0f;
    let standby = b0 & 0x80 != 0;
    let nominal_voltage = le16(d, 1) as i16;
    let max_neg_dev = le16(d, 3) as i16;
    let max_pos_dev = le16(d, 5) as i16;
    let ripple_and_noise = le16(d, 7);
    let min_current = le16(d, 9);
    let max_current = le16(d, 11);

    println!(" DC Output Record");
    println!("  Output Number              : {output_number}");
    println!(
        "  Standby power              : {}",
        if standby { "Yes" } else { "No" }
    );
    println!(
        "  Nominal voltage            : {:.2} V",
        nominal_voltage as f64 / 100.0
    );
    println!(
        "  Max negative deviation     : {:.2} V",
        max_neg_dev as f64 / 100.0
    );
    println!(
        "  Max positive deviation     : {:.2} V",
        max_pos_dev as f64 / 100.0
    );
    println!("  Ripple and noise pk-pk     : {ripple_and_noise} mV");
    println!(
        "  Minimum current draw       : {:.3} A",
        min_current as f64 / 1000.0
    );
    println!(
        "  Maximum current draw       : {:.3} A",
        max_current as f64 / 1000.0
    );
}

fn print_dc_load(d: &[u8]) {
    let output_number = byte(d, 0) & 0x0f;
    let nominal_voltage = le16(d, 1) as i16;
    let min_voltage = le16(d, 3) as i16;
    let max_voltage = le16(d, 5) as i16;
    let ripple_and_noise = le16(d, 7);
    let min_current = le16(d, 9);
    let max_current = le16(d, 11);

    println!(" DC Load Record");
    println!("  Output Number              : {output_number}");
    println!(
        "  Nominal voltage            : {:.2} V",
        nominal_voltage as f64 / 100.0
    );
    println!(
        "  Min voltage allowed        : {:.2} V",
        min_voltage as f64 / 100.0
    );
    println!(
        "  Max voltage allowed        : {:.2} V",
        max_voltage as f64 / 100.0
    );
    println!("  Ripple and noise pk-pk     : {ripple_and_noise} mV");
    println!(
        "  Minimum current load       : {:.3} A",
        min_current as f64 / 1000.0
    );
    println!(
        "  Maximum current load       : {:.3} A",
        max_current as f64 / 1000.0
    );
}

fn print_management_access(body: &[u8], rlen: u32) {
    let subtype = byte(body, 0);
    let data = if body.len() > 1 { &body[1..] } else { &[][..] };
    let datalen = rlen.saturating_sub(1) as usize;

    // name, minlen, maxlen, is_uuid
    let (name, minlen, maxlen, is_uuid) = match subtype {
        1 => ("System Management URL", 16, 256, false),
        2 => ("System Name", 8, 64, false),
        3 => ("System Ping Address", 8, 64, false),
        4 => ("Component Management URL", 16, 256, false),
        5 => ("Component Name", 8, 64, false),
        6 => ("Component Ping Address", 8, 64, false),
        7 => ("System Unique ID", 16, 16, true),
        _ => {
            eprintln!(
                "Unsupported subtype 0x{subtype:02x} found for multi-record area management record"
            );
            return;
        }
    };

    if datalen < minlen || datalen > maxlen {
        eprintln!("Wrong data length {datalen}, must be {minlen} < X < {maxlen}");
    }

    let value = if is_uuid {
        mc::guid_auto_str(data)
    } else {
        let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
        String::from_utf8_lossy(&data[..end.min(datalen)]).into_owned()
    };
    println!(" {name:<22}: {value}");
}

// ---------------------------------------------------------------------------
// FRU area reading (`read_fru_area`).
// ---------------------------------------------------------------------------

/// Read `length` bytes from `offset` in FRU device `id`. The returned buffer is
/// always `length` bytes; bytes past the end of the device are zero. `None` on
/// failure.
fn read_fru_area(
    intf: &mut dyn Interface,
    fru: &mut FruInfo,
    id: u8,
    offset: u32,
    length: u32,
) -> Option<Vec<u8>> {
    if offset > fru.size {
        eprintln!("Read FRU Area offset incorrect: {} > {}", offset, fru.size);
        return None;
    }

    // The output buffer always covers the full requested length (C zero-fills
    // the region past the end of the device).
    let mut buf = vec![0u8; length as usize];

    let mut read_len = length;
    let mut finish = offset + length;
    if finish > fru.size {
        finish = fru.size;
        read_len = finish - offset;
    }

    if fru.max_read_size == 0 {
        // Mirror the C `read_fru_area` computation for the lanplus transport:
        // `ipmi_intf_get_max_response_data_size()` is `IPMI_LAN_MAX_RESPONSE_SIZE`
        // (34), minus one byte for the count and one more for the checksum.
        fru.max_read_size = LANPLUS_MAX_RESPONSE_SIZE as u32 - 2;
        if fru.access {
            fru.max_read_size &= !1;
        }
    }

    let mut off = offset;
    let mut size_left = read_len;

    while off < finish {
        let addr = if fru.access { off >> 1 } else { off };
        let want = (finish - off).min(fru.max_read_size).min(255) as u8;
        let req = Request::new(NETFN_STORAGE, GET_FRU_DATA).with_data([
            id,
            (addr & 0xff) as u8,
            ((addr >> 8) & 0xff) as u8,
            want,
        ]);

        let rsp = match intf.sendrecv(&req) {
            Ok(rsp) => rsp,
            Err(_) => {
                eprintln!("FRU Read failed");
                return None;
            }
        };

        if !rsp.ok() {
            if fru_cc_rq2big(rsp.ccode) && fru.max_read_size > FRU_BLOCK_SZ {
                if fru.max_read_size > FRU_AREA_MAXIMUM_BLOCK_SZ {
                    fru.max_read_size -= FRU_BLOCK_SZ;
                } else {
                    fru.max_read_size -= 1;
                }
                log_info(&format!(
                    "Retrying FRU read with request size {}",
                    fru.max_read_size
                ));
                continue;
            }
            eprintln!("FRU Read failed: {}", cc::completion_code_str(rsp.ccode));
            return None;
        }

        let count = if rsp.data.is_empty() {
            0
        } else if fru.access {
            u32::from(rsp.data[0]) << 1
        } else {
            u32::from(rsp.data[0])
        };

        if rsp.data.is_empty() || count as usize > rsp.data.len() - 1 || count > size_left {
            println!(" Not enough buffer size");
            return None;
        }

        let start = (off - offset) as usize;
        buf[start..start + count as usize].copy_from_slice(&rsp.data[1..1 + count as usize]);
        off += count;
        size_left -= count;

        // A device may report a size larger than it can serve; stop early so
        // the caller can parse whatever was returned.
        if count == 0 && off < finish {
            return Some(buf);
        }
    }

    if off < finish {
        return None;
    }
    Some(buf)
}

fn fru_cc_rq2big(code: u8) -> bool {
    code == cc::CC_REQ_DATA_INV_LENGTH
        || code == cc::CC_REQ_DATA_FIELD_EXCEED
        || code == cc::CC_CANT_RET_NUM_REQ_BYTES
}

// ---------------------------------------------------------------------------
// FRU string decoding (`get_fru_area_str`).
// ---------------------------------------------------------------------------

const BCD_PLUS: &[u8; 16] = b"0123456789 -.:,_";

/// Decode one FRU area string field at `*offset`, advancing `*offset` past it.
/// `None` when the field is empty (length 0).
fn get_fru_area_str(data: &[u8], offset: &mut usize) -> Option<String> {
    let mut off = *offset;
    if off >= data.len() {
        return None;
    }
    let first = data[off];
    off += 1;

    let typecode = (first & 0xc0) >> 6;
    let len = (first & 0x3f) as usize;

    let size = match typecode {
        0 | 1 => len * 2,
        2 => (len / 3 + 1) * 4,
        _ => len,
    };
    if size < 1 {
        *offset = off;
        return None;
    }

    let avail = data.len().saturating_sub(off);
    let s = match typecode {
        0 => {
            // Binary/unspecified: hex dump, two characters per byte.
            let n = len.min(avail);
            let mut s = String::with_capacity(n * 2);
            for &b in &data[off..off + n] {
                s.push_str(&format!("{b:02x}"));
            }
            s
        }
        1 => {
            // BCD plus.
            let mut s = String::with_capacity(size);
            for k in 0..size {
                let nib = if k % 2 == 1 {
                    data.get(off + k / 2).map(|b| b & 0x0f)
                } else {
                    data.get(off + k / 2).map(|b| (b >> 4) & 0x0f)
                };
                match nib {
                    Some(n) => s.push(BCD_PLUS[n as usize] as char),
                    None => break,
                }
            }
            s
        }
        2 => {
            // 6-bit ASCII, little-endian packing.
            let mut s = String::with_capacity(size);
            let mut i = 0;
            while i < len {
                let k = (len - i).min(3);
                let mut bits: u32 = 0;
                for b in 0..k {
                    if let Some(&v) = data.get(off + i + b) {
                        bits |= u32::from(v) << (8 * b as u32);
                    }
                }
                for _ in 0..4 {
                    s.push((((bits & 0x3f) as u8) + 0x20) as char);
                    bits >>= 6;
                }
                i += 3;
            }
            s
        }
        _ => {
            // 8-bit ASCII.
            let n = len.min(avail);
            String::from_utf8_lossy(&data[off..off + n]).into_owned()
        }
    };

    *offset = off + len;
    Some(s)
}

fn checksum_is_valid(area: &[u8], len: usize) -> bool {
    if len == 0 || len > area.len() {
        return false;
    }
    let mut checksum: u8 = 0;
    for &b in &area[..len - 1] {
        checksum = checksum.wrapping_add(b);
    }
    area[len - 1] == checksum.wrapping_neg()
}

// ---------------------------------------------------------------------------
// Small helpers.
// ---------------------------------------------------------------------------

fn byte(d: &[u8], i: usize) -> u8 {
    d.get(i).copied().unwrap_or(0)
}

fn le16(d: &[u8], i: usize) -> u16 {
    u16::from(byte(d, i)) | u16::from(byte(d, i + 1)) << 8
}

/// Format a board manufacturing date (3 little-endian bytes, minutes since
/// 1996) the way `ipmi_timestamp_string()` does for a normal timestamp.
fn format_fru_timestamp(data: &[u8], i: usize) -> String {
    let fru_ts = u32::from(byte(data, i))
        | u32::from(byte(data, i + 1)) << 8
        | u32::from(byte(data, i + 2)) << 16;
    if fru_ts == 0 {
        return "Unspecified".to_string();
    }
    format_unix_timestamp(i64::from(fru_ts) * 60 + 820_454_400)
}

#[cfg(unix)]
fn format_unix_timestamp(ts: i64) -> String {
    unsafe {
        let t = ts as libc::time_t;
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&t, &mut tm).is_null() {
            return "Unspecified".to_string();
        }
        let fmt = b"%c %Z\0";
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
fn format_unix_timestamp(_ts: i64) -> String {
    "Unspecified".to_string()
}

fn log_info(msg: &str) {
    if verbose::level() >= 1 {
        eprintln!("{msg}");
    }
}

fn log_debug(msg: &str) {
    if verbose::level() >= 2 {
        eprintln!("{msg}");
    }
}

/// Hex dump in `printbuf()` format (stderr, gated by verbosity).
fn printbuf(data: &[u8], desc: &str) {
    if data.is_empty() || verbose::level() < 1 {
        return;
    }
    eprintln!("{desc} ({} bytes)", data.len());
    for (i, b) in data.iter().enumerate() {
        if i % 16 == 0 && i != 0 {
            eprintln!();
        }
        eprint!(" {b:02x}");
    }
    eprintln!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_fru_field_is_decoded() {
        // Type 11b (8-bit ASCII), length 3, followed by "ABC".
        let data = [0xc3, b'A', b'B', b'C'];
        let mut off = 0;
        assert_eq!(get_fru_area_str(&data, &mut off).as_deref(), Some("ABC"));
        assert_eq!(off, 4);
    }

    #[test]
    fn bcd_plus_fru_field_is_decoded() {
        // Type 01b (BCD plus), length 2 -> four nibbles "1234".
        let data = [0x42, 0x12, 0x34];
        let mut off = 0;
        assert_eq!(get_fru_area_str(&data, &mut off).as_deref(), Some("1234"));
        assert_eq!(off, 3);
    }

    #[test]
    fn six_bit_ascii_fru_field_is_decoded() {
        // Type 10b (6-bit ASCII), length 3. "ABC" encoded as 6-bit values
        // (0x21, 0x22, 0x23) packed little-endian into 3 bytes.
        let packed: u32 = 0x21 | (0x22 << 6) | (0x23 << 12);
        let data = [
            0x83,
            packed as u8,
            (packed >> 8) as u8,
            (packed >> 16) as u8,
        ];
        let mut off = 0;
        let s = get_fru_area_str(&data, &mut off).unwrap();
        assert_eq!(s, "ABC ");
        assert_eq!(off, 4);
    }

    #[test]
    fn zero_length_field_returns_none() {
        let data = [0x00];
        let mut off = 0;
        assert!(get_fru_area_str(&data, &mut off).is_none());
        assert_eq!(off, 1);
    }

    #[test]
    fn checksum_detects_valid_and_invalid_areas() {
        let valid = [0x01u8, 0x02, 0x03, 0xfa];
        assert!(checksum_is_valid(&valid, valid.len()));
        let invalid = [0x01u8, 0x02, 0x03, 0x00];
        assert!(!checksum_is_valid(&invalid, invalid.len()));
    }

    #[test]
    fn zero_fru_timestamp_is_unspecified() {
        assert_eq!(format_fru_timestamp(&[0, 0, 0], 0), "Unspecified");
    }

    #[test]
    fn voltage_descriptions_match_spec() {
        assert_eq!(voltage_desc(0), "12 V");
        assert_eq!(voltage_desc(3), "3.3 V");
        assert_eq!(voltage_desc(9), "");
    }
}
