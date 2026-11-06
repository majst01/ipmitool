//! Chassis commands: `chassis ...` and the top-level `power` shortcut.
//! Mirrors `lib/ipmi_chassis.c`.

use ipmi_core::cc;
use ipmi_core::consts;
use ipmi_core::interface::Interface;
use ipmi_core::types::Request;
use ipmi_core::values;

use crate::args::parse_u8;

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
        "status" => status(intf),
        "selftest" => selftest(intf),
        "power" => power(intf, rest),
        "identify" => identify(intf, rest),
        "poh" => poh(intf),
        "restart_cause" => restart_cause(intf),
        "policy" => policy(intf, rest),
        other => {
            eprintln!("Invalid chassis command: {other}");
            -1
        }
    }
}

/// Top-level `power` shortcut, which forwards to `chassis power`.
pub fn power_shortcut(intf: &mut dyn Interface, args: &[String]) -> i32 {
    power(intf, args)
}

fn power(intf: &mut dyn Interface, args: &[String]) -> i32 {
    let Some(cmd) = args.first().map(String::as_str) else {
        eprintln!("chassis power Commands: status, on, off, cycle, reset, diag, soft");
        return 0;
    };
    if cmd == "help" {
        eprintln!("chassis power Commands: status, on, off, cycle, reset, diag, soft");
        return 0;
    }
    if cmd == "status" {
        return print_power_status(intf);
    }
    let ctl = match cmd {
        "up" | "on" => consts::CHASSIS_CTL_POWER_UP,
        "down" | "off" => consts::CHASSIS_CTL_POWER_DOWN,
        "cycle" => consts::CHASSIS_CTL_POWER_CYCLE,
        "reset" => consts::CHASSIS_CTL_HARD_RESET,
        "diag" => consts::CHASSIS_CTL_PULSE_DIAG,
        "acpi" | "soft" => consts::CHASSIS_CTL_ACPI_SOFT,
        other => {
            eprintln!("Invalid chassis power command: {other}");
            return -1;
        }
    };
    power_control(intf, ctl)
}

fn power_status(intf: &mut dyn Interface) -> Result<u8, i32> {
    let req = Request::new(consts::NETFN_CHASSIS, 0x1);
    let rsp = intf.sendrecv(&req).map_err(|_| {
        eprintln!("Unable to get Chassis Power Status");
        -1
    })?;
    if !rsp.ok() {
        eprintln!(
            "Get Chassis Power Status failed: {}",
            cc::completion_code_str(rsp.ccode)
        );
        return Err(-1);
    }
    Ok(rsp.data.first().copied().unwrap_or(0) & 1)
}

fn print_power_status(intf: &mut dyn Interface) -> i32 {
    match power_status(intf) {
        Ok(ps) => {
            println!("Chassis Power is {}", if ps != 0 { "on" } else { "off" });
            0
        }
        Err(code) => code,
    }
}

fn power_control(intf: &mut dyn Interface, ctl: u8) -> i32 {
    let mut req = Request::new(consts::NETFN_CHASSIS, 0x2);
    req.data = vec![ctl];
    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => {
            eprintln!(
                "Unable to set Chassis Power Control to {}",
                values::power_control_str(ctl)
            );
            return -1;
        }
    };
    if !rsp.ok() {
        eprintln!(
            "Set Chassis Power Control to {} failed: {}",
            values::power_control_str(ctl),
            cc::completion_code_str(rsp.ccode)
        );
        return -1;
    }
    println!("Chassis Power Control: {}", values::power_control_str(ctl));
    0
}

fn status(intf: &mut dyn Interface) -> i32 {
    let req = Request::new(consts::NETFN_CHASSIS, 0x1);
    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => {
            eprintln!("Error sending Chassis Status command");
            return -1;
        }
    };
    if !rsp.ok() {
        eprintln!(
            "Error sending Chassis Status command: {}",
            cc::completion_code_str(rsp.ccode)
        );
        return -1;
    }
    let d = &rsp.data;
    if d.len() < 3 {
        eprintln!("Error sending Chassis Status command: short response");
        return -1;
    }

    println!(
        "System Power         : {}",
        if d[0] & 0x1 != 0 { "on" } else { "off" }
    );
    println!(
        "Power Overload       : {}",
        if d[0] & 0x2 != 0 { "true" } else { "false" }
    );
    println!(
        "Power Interlock      : {}",
        if d[0] & 0x4 != 0 {
            "active"
        } else {
            "inactive"
        }
    );
    println!(
        "Main Power Fault     : {}",
        if d[0] & 0x8 != 0 { "true" } else { "false" }
    );
    println!(
        "Power Control Fault  : {}",
        if d[0] & 0x10 != 0 { "true" } else { "false" }
    );
    print!("Power Restore Policy : ");
    match (d[0] & 0x60) >> 5 {
        0x0 => println!("always-off"),
        0x1 => println!("previous"),
        0x2 => println!("always-on"),
        _ => println!("unknown"),
    }

    print!("Last Power Event     : ");
    if d[1] & 0x1 != 0 {
        print!("ac-failed ");
    }
    if d[1] & 0x2 != 0 {
        print!("overload ");
    }
    if d[1] & 0x4 != 0 {
        print!("interlock ");
    }
    if d[1] & 0x8 != 0 {
        print!("fault ");
    }
    if d[1] & 0x10 != 0 {
        print!("command");
    }
    println!();

    println!(
        "Chassis Intrusion    : {}",
        if d[2] & 0x1 != 0 {
            "active"
        } else {
            "inactive"
        }
    );
    println!(
        "Front-Panel Lockout  : {}",
        if d[2] & 0x2 != 0 {
            "active"
        } else {
            "inactive"
        }
    );
    println!(
        "Drive Fault          : {}",
        if d[2] & 0x4 != 0 { "true" } else { "false" }
    );
    println!(
        "Cooling/Fan Fault    : {}",
        if d[2] & 0x8 != 0 { "true" } else { "false" }
    );

    if d.len() > 3 {
        if d[3] == 0 {
            println!("Front Panel Control  : none");
        } else {
            println!(
                "Sleep Button Disable : {}",
                if d[3] & 0x80 != 0 {
                    "allowed"
                } else {
                    "not allowed"
                }
            );
            println!(
                "Diag Button Disable  : {}",
                if d[3] & 0x40 != 0 {
                    "allowed"
                } else {
                    "not allowed"
                }
            );
            println!(
                "Reset Button Disable : {}",
                if d[3] & 0x20 != 0 {
                    "allowed"
                } else {
                    "not allowed"
                }
            );
            println!(
                "Power Button Disable : {}",
                if d[3] & 0x10 != 0 {
                    "allowed"
                } else {
                    "not allowed"
                }
            );
            println!(
                "Sleep Button Disabled: {}",
                if d[3] & 0x08 != 0 { "true" } else { "false" }
            );
            println!(
                "Diag Button Disabled : {}",
                if d[3] & 0x04 != 0 { "true" } else { "false" }
            );
            println!(
                "Reset Button Disabled: {}",
                if d[3] & 0x02 != 0 { "true" } else { "false" }
            );
            println!(
                "Power Button Disabled: {}",
                if d[3] & 0x01 != 0 { "true" } else { "false" }
            );
        }
    }
    0
}

fn identify(intf: &mut dyn Interface, args: &[String]) -> i32 {
    let mut interval: u8 = 0;
    let mut force_on = false;
    let mut len = 0usize;

    match args.first().map(String::as_str) {
        None => {}
        Some("help") => {
            eprintln!("chassis identify <interval>");
            eprintln!("                 default is 15 seconds");
            eprintln!("                 0 to turn off");
            eprintln!("                 force to turn on indefinitely");
            return 0;
        }
        Some("force") => {
            force_on = true;
            len = 2;
        }
        Some(v) => match parse_u8(v) {
            Some(val) => {
                interval = val;
                len = 1;
            }
            None => {
                eprintln!("Invalid interval given.");
                return -1;
            }
        },
    }

    let mut req = Request::new(consts::NETFN_CHASSIS, 0x4);
    if len == 2 {
        req.data = vec![interval, 1];
    } else if len == 1 {
        req.data = vec![interval];
    }

    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => {
            eprintln!("Unable to set Chassis Identify");
            return -1;
        }
    };
    if !rsp.ok() {
        eprintln!(
            "Set Chassis Identify failed: {}",
            cc::completion_code_str(rsp.ccode)
        );
        if force_on {
            eprintln!("Chassis may not support Force Identify On");
        }
        return -1;
    }

    print!("Chassis identify interval: ");
    if args.is_empty() {
        println!("default (15 seconds)");
    } else if force_on {
        println!("indefinite");
    } else if interval == 0 {
        println!("off");
    } else {
        println!("{interval} seconds");
    }
    0
}

fn poh(intf: &mut dyn Interface) -> i32 {
    let req = Request::new(consts::NETFN_CHASSIS, 0xf);
    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => {
            eprintln!("Unable to get Chassis Power-On-Hours");
            return -1;
        }
    };
    if !rsp.ok() {
        eprintln!(
            "Get Chassis Power-On-Hours failed: {}",
            cc::completion_code_str(rsp.ccode)
        );
        return -1;
    }
    if rsp.data.len() < 5 {
        eprintln!("Get Chassis Power-On-Hours failed: short response");
        return -1;
    }
    let mins_per_count = rsp.data[0];
    let count = u32::from_le_bytes(rsp.data[1..5].try_into().unwrap());
    let total_minutes = count as f64 * mins_per_count as f64;
    let days = (total_minutes / 1440.0) as u32;
    let rem = total_minutes - days as f64 * 1440.0;
    let hours = (rem / 60.0) as u32;
    let minutes = rem - hours as f64 * 60.0;

    if mins_per_count < 60 {
        println!(
            "POH Counter  : {days} days, {hours} hours, {} minutes",
            minutes as u32
        );
    } else {
        println!("POH Counter  : {days} days, {hours} hours");
    }
    0
}

fn restart_cause(intf: &mut dyn Interface) -> i32 {
    let req = Request::new(consts::NETFN_CHASSIS, 0x7);
    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => {
            eprintln!("Unable to get Chassis Restart Cause");
            return -1;
        }
    };
    if !rsp.ok() {
        eprintln!(
            "Get Chassis Restart Cause failed: {}",
            cc::completion_code_str(rsp.ccode)
        );
        return -1;
    }
    let cause = rsp.data.first().copied().unwrap_or(0) & 0xf;
    println!(
        "System restart cause: {}",
        values::chassis_restart_cause_str(cause)
    );
    0
}

fn policy(intf: &mut dyn Interface, args: &[String]) -> i32 {
    let Some(cmd) = args.first().map(String::as_str) else {
        print_policy_help();
        return 0;
    };
    if cmd == "help" {
        print_policy_help();
        return 0;
    }
    let ctl = match cmd {
        "list" => consts::CHASSIS_POLICY_NO_CHANGE,
        "always-on" => consts::CHASSIS_POLICY_ALWAYS_ON,
        "previous" => consts::CHASSIS_POLICY_PREVIOUS,
        "always-off" => consts::CHASSIS_POLICY_ALWAYS_OFF,
        other => {
            eprintln!("Invalid chassis policy: {other}");
            return -1;
        }
    };

    let mut req = Request::new(consts::NETFN_CHASSIS, 0x6);
    req.data = vec![ctl];
    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => {
            eprintln!("Error in Power Restore Policy command");
            return -1;
        }
    };
    if !rsp.ok() {
        eprintln!(
            "Power Restore Policy command failed: {}",
            cc::completion_code_str(rsp.ccode)
        );
        return -1;
    }

    if ctl == consts::CHASSIS_POLICY_NO_CHANGE {
        let bits = rsp.data.first().copied().unwrap_or(0);
        print!("Supported chassis power policy:  ");
        if bits & (1 << consts::CHASSIS_POLICY_ALWAYS_OFF) != 0 {
            print!("always-off ");
        }
        if bits & (1 << consts::CHASSIS_POLICY_ALWAYS_ON) != 0 {
            print!("always-on ");
        }
        if bits & (1 << consts::CHASSIS_POLICY_PREVIOUS) != 0 {
            print!("previous");
        }
        println!();
    } else {
        print!("Set chassis power restore policy to ");
        match ctl {
            consts::CHASSIS_POLICY_ALWAYS_ON => println!("always-on"),
            consts::CHASSIS_POLICY_ALWAYS_OFF => println!("always-off"),
            consts::CHASSIS_POLICY_PREVIOUS => println!("previous"),
            _ => println!("unknown"),
        }
    }
    0
}

fn selftest(intf: &mut dyn Interface) -> i32 {
    let req = Request::new(consts::NETFN_APP, consts::IPMI_GET_SELF_TEST_RESULTS);
    let rsp = match intf.sendrecv(&req) {
        Ok(rsp) => rsp,
        Err(_) => {
            eprintln!("Error sending Get Self Test command");
            return -1;
        }
    };
    if !rsp.ok() {
        eprintln!(
            "Error sending Get Self Test command: {}",
            cc::completion_code_str(rsp.ccode)
        );
        return -1;
    }
    let code = rsp.data.first().copied().unwrap_or(0);
    let test = rsp.data.get(1).copied().unwrap_or(0);
    print!("Self Test Results    : ");
    match code {
        0x55 => println!("passed"),
        0x56 => println!("not implemented"),
        0x57 => {
            println!("device error");
            const BROKEN: [&str; 8] = [
                "firmware corrupted",
                "boot block corrupted",
                "FRU Internal Use Area corrupted",
                "SDR Repository empty",
                "IPMB not responding",
                "cannot access BMC FRU",
                "cannot access SDR Repository",
                "cannot access SEL Device",
            ];
            for (i, name) in BROKEN.iter().enumerate() {
                if test & (1 << i) != 0 {
                    println!("                       [{name}]");
                }
            }
        }
        0x58 => println!("Fatal hardware error: {test:02x}h"),
        _ => println!("Device-specific failure {code:02x}h:{test:02x}h"),
    }
    0
}

fn print_policy_help() {
    eprintln!("chassis policy <state>");
    eprintln!("   list        : return supported policies");
    eprintln!("   always-on   : turn on when power is restored");
    eprintln!("   previous    : return to previous state when power is restored");
    eprintln!("   always-off  : stay off after power is restored");
}

fn print_usage() {
    eprintln!(
        "Chassis Commands:\n  status, power, policy, restart_cause\n  poh, identify, selftest,\n  bootdev, bootparam, bootmbox"
    );
}
