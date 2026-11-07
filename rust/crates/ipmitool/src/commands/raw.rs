//! The `raw` command: send an arbitrary IPMI request and print the response.
//! Mirrors `ipmi_raw_main` in `lib/ipmi_raw.c`.

use ipmi_core::cc;
use ipmi_core::interface::Interface;
use ipmi_core::types::Request;
use ipmi_core::values;

use crate::args::parse_u8;

pub fn main(intf: &mut dyn Interface, args: &[String]) -> i32 {
    if args.len() == 1 && args[0] == "help" {
        print_help();
        return 0;
    }
    if args.len() < 2 {
        errln!("Not enough parameters given.");
        print_help();
        return -1;
    }
    if args.len() > 256 {
        errln!("Raw command input limit (256 bytes) exceeded");
        return -1;
    }

    let lun = intf.session_params().target_lun;
    let netfn = match resolve_netfn(&args[0]) {
        Some(n) => n,
        None => {
            errln!("Given netfn \"{}\" is invalid.", args[0]);
            return -1;
        }
    };
    let cmd = match parse_u8(&args[1]) {
        Some(v) => v,
        None => {
            errln!("Given command \"{}\" is invalid.", args[1]);
            return -1;
        }
    };

    let mut data = Vec::with_capacity(args.len() - 2);
    for a in &args[2..] {
        match parse_u8(a) {
            Some(v) => data.push(v),
            None => {
                errln!("Given data \"{a}\" is invalid.");
                return -1;
            }
        }
    }

    let channel = intf.session_params().target_channel & 0x0f;
    errln!(
        "RAW REQ (channel=0x{channel:x} netfn=0x{netfn:x} lun=0x{lun:x} cmd=0x{cmd:x} data_len={})",
        data.len()
    );

    let mut req = Request::new(netfn, cmd);
    req.lun = lun;
    req.data = data;

    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(e) => {
            errln!(
                "Unable to send RAW command (channel=0x{channel:x} netfn=0x{netfn:x} lun=0x{lun:x} cmd=0x{cmd:x})"
            );
            errln!("{e}");
            return -1;
        }
    };

    if !rsp.ok() {
        errln!(
            "Unable to send RAW command (channel=0x{channel:x} netfn=0x{netfn:x} lun=0x{lun:x} cmd=0x{cmd:x} rsp=0x{:x}): {}",
            rsp.ccode,
            cc::completion_code_str(rsp.ccode)
        );
        return -1;
    }

    errln!("RAW RSP ({} bytes)", rsp.data.len());
    for (i, b) in rsp.data.iter().enumerate() {
        if i % 16 == 0 && i != 0 {
            outln!();
        }
        out!(" {b:02x}");
    }
    outln!();

    0
}

fn resolve_netfn(s: &str) -> Option<u8> {
    if let Some(n) = values::netfn_from_str(s) {
        return Some(n);
    }
    parse_u8(s)
}

fn print_help() {
    errln!("RAW Commands:  raw <netfn> <cmd> [data]");
    errln!("Network Function Codes");
    for (val, name) in [
        ("0", "Chassis"),
        ("2", "Bridge"),
        ("4", "SensorEvent"),
        ("6", "Application"),
        ("8", "Firmware"),
        ("a", "Storage"),
        ("c", "Transport"),
    ] {
        errln!("\t{val:<16}  {name}");
    }
    errln!("(can also use raw hex values)");
}
