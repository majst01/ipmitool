//! Rust rewrite of the ipmitool CLI, currently supporting the `lanplus`
//! interface and the `raw`, `chassis`, `power` and `mc` commands.

mod args;
mod commands;

use std::process::ExitCode;

use ipmi_core::interface::Interface;
use ipmi_core::types::SessionParams;
use ipmi_lanplus::LanPlus;

/// Version string reported by `-V`, matching the C build's `VERSION`.
const VERSION: &str = "1.8.19";

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    match run(&argv) {
        Ok(code) => ExitCode::from(code as u8),
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::FAILURE
        }
    }
}

fn run(argv: &[String]) -> Result<i32, String> {
    let parsed = args::parse(argv)?;
    let (opts, command, cmd_args) = match parsed {
        args::Parsed::Help => {
            print_usage();
            return Ok(0);
        }
        args::Parsed::Version => {
            println!("ipmitool version {VERSION}");
            return Ok(0);
        }
        args::Parsed::Run(opts, command, cmd_args) => (opts, command, cmd_args),
    };

    // Determine the interface: default to lanplus when a hostname is supplied,
    // otherwise refuse (we have no local/BMC-direct transport yet).
    // Only lanplus is implemented; it is also the default when a hostname is
    // supplied, matching ipmitool's preference for LAN access.
    let intf_name = opts.intf.clone().unwrap_or_else(|| "lanplus".to_string());
    if intf_name != "lanplus" {
        return Err(format!("Interface {intf_name} not supported"));
    }

    let mut params = SessionParams::default();
    if let Some(host) = &opts.hostname {
        params.hostname = Some(host.clone());
    }
    params.port = opts.port;
    if let Some(u) = &opts.username {
        params.set_username(u);
    }
    if let Some(p) = &opts.password {
        if p.len() > 20 {
            return Err("lanplus: password is longer than 20 bytes.".to_string());
        }
        params.set_password(p);
    }
    params.privlvl = opts.privlvl;
    params.lookupbit = opts.lookupbit;
    params.authtype_set = opts.authtype;
    params.timeout = opts.timeout;
    params.retry = opts.retry;
    params.target_lun = opts.target_lun;
    params.target_addr = opts.target_addr;
    params.target_channel = opts.target_channel;
    params.transit_addr = opts.transit_addr;
    params.transit_channel = opts.transit_channel;
    params.my_addr = opts.my_addr as u32;
    params.cipher_suite = opts.cipher;
    params.sol_escape_char = opts.escape_char;
    if let Some(kg) = &opts.kgkey {
        let bytes = kg.as_bytes();
        let n = bytes.len().min(params.kg.len());
        params.kg[..n].copy_from_slice(&bytes[..n]);
    }

    if params.hostname.is_none() {
        return Err("No hostname specified!".to_string());
    }

    let mut intf = LanPlus::new(params);
    if let Err(e) = intf.open() {
        return Err(format!(
            "Error: Unable to establish IPMI v2 / RMCP+ session: {e}"
        ));
    }

    let code = dispatch(&mut intf, &command, &cmd_args);

    intf.close();
    Ok(code)
}

fn dispatch(intf: &mut dyn Interface, command: &str, args: &[String]) -> i32 {
    match command {
        "raw" => commands::raw::main(intf, args),
        "chassis" => commands::chassis::main(intf, args),
        "power" => commands::chassis::power_shortcut(intf, args),
        "mc" | "bmc" => commands::mc::main(intf, args),
        "sol" => commands::sol::main(intf, args),
        other => {
            eprintln!("Invalid command: {other}");
            -1
        }
    }
}

fn print_usage() {
    println!("ipmitool version {VERSION}");
    println!("usage: ipmitool [options...] <command>");
    println!("       -h             This help");
    println!("       -V             Show version information");
    println!("       -v             Verbose (can use multiple times)");
    println!("       -c             Display output in comma separated format");
    println!("       -I intf        Interface to use (only lanplus is supported)");
    println!("       -H hostname    Remote host name for LAN interface");
    println!("       -p port        Remote RMCP port [default=623]");
    println!("       -U username    Remote session username");
    println!("       -P password    Remote session password");
    println!("       -C ciphersuite Cipher suite to be used by lanplus interface");
    println!("       -L level       Remote session privilege level [default=ADMINISTRATOR]");
    println!("       -m address     Set local IPMB address");
    println!("       -b channel     Set destination channel for bridged request");
    println!("       -l lun         Set destination lun for raw commands");
    println!("       -N seconds     Specify timeout for lanplus interface [default=1]");
    println!("       -R retry       Set the number of retries [default=4]");
    println!();
    println!("Commands:");
    println!("  raw         Send a RAW IPMI request and print response");
    println!("  chassis     Get chassis status and set power state");
    println!("  power       Shortcut to chassis power commands");
    println!("  mc          Management Controller status and global enables");
    println!("  sol         Configure and connect to the Serial-over-LAN interface");
}
