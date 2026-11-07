//! Mock-`Interface` tests: exercise command logic end-to-end without a BMC by
//! scripting the transport and capturing stdout/stderr.

use ipmi_core::cc;
use ipmi_core::consts;
use ipmi_core::error::Error;
use ipmi_core::sol;

use crate::commands;
use crate::output::capture;
use crate::testutil::{ccode, ok, MockInterface};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

// ---------------------------------------------------------------------------
// raw
// ---------------------------------------------------------------------------

#[test]
fn raw_sends_request_and_prints_response() {
    let ((code, reqs), out, err) = capture(|| {
        let mut m = MockInterface::new(|_| Ok(ok(vec![0xde, 0xad, 0xbe, 0xef])));
        m.params.target_lun = 0x02;
        m.params.target_channel = 0x07;
        let code = commands::raw::main(&mut m, &args(&["0x06", "0x01", "0xaa", "0xbb"]));
        (code, m.recorded())
    });

    assert_eq!(code, 0);
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].netfn, consts::NETFN_APP);
    assert_eq!(reqs[0].cmd, 0x01);
    assert_eq!(reqs[0].lun, 0x02);
    assert_eq!(reqs[0].data, vec![0xaa, 0xbb]);
    assert!(
        err.contains("RAW REQ (channel=0x7 netfn=0x6 lun=0x2 cmd=0x1 data_len=2)"),
        "stderr: {err}"
    );
    assert!(err.contains("RAW RSP (4 bytes)"), "stderr: {err}");
    assert_eq!(out.trim(), "de ad be ef");
}

#[test]
fn raw_invalid_netfn() {
    let ((code, reqs), _out, err) = capture(|| {
        let mut m = MockInterface::always_ok();
        let code = commands::raw::main(&mut m, &args(&["bogus", "0x01"]));
        (code, m.recorded())
    });
    assert_eq!(code, -1);
    assert!(err.contains("Given netfn \"bogus\" is invalid."), "stderr: {err}");
    assert!(reqs.is_empty());
}

#[test]
fn raw_completion_error() {
    let (code, _out, err) = capture(|| {
        let mut m = MockInterface::new(|_| Ok(ccode(cc::CC_INV_CMD)));
        commands::raw::main(&mut m, &args(&["0x06", "0x01"]))
    });
    assert_eq!(code, -1);
    assert!(err.contains("Invalid command"), "stderr: {err}");
}

// ---------------------------------------------------------------------------
// chassis / power
// ---------------------------------------------------------------------------

#[test]
fn power_status_on_and_off() {
    for (byte, label) in [(0x01u8, "Chassis Power is on"), (0x00, "Chassis Power is off")] {
        let (code, out, _err) = capture(move || {
            let mut m = MockInterface::new(move |_| Ok(ok(vec![byte])));
            commands::chassis::power_shortcut(&mut m, &args(&["status"]))
        });
        assert_eq!(code, 0);
        assert!(out.contains(label), "stdout: {out}");
    }
}

#[test]
fn power_control_sends_control_byte() {
    let cases: [(&str, u8, &str); 6] = [
        ("on", consts::CHASSIS_CTL_POWER_UP, "Chassis Power Control: Up/On"),
        ("off", consts::CHASSIS_CTL_POWER_DOWN, "Chassis Power Control: Down/Off"),
        ("cycle", consts::CHASSIS_CTL_POWER_CYCLE, "Chassis Power Control: Cycle"),
        ("reset", consts::CHASSIS_CTL_HARD_RESET, "Chassis Power Control: Reset"),
        ("diag", consts::CHASSIS_CTL_PULSE_DIAG, "Chassis Power Control: Diag"),
        ("soft", consts::CHASSIS_CTL_ACPI_SOFT, "Chassis Power Control: Soft"),
    ];
    for (arg, ctl, expected) in cases {
        let ((code, reqs), out, _err) = capture(move || {
            let mut m = MockInterface::always_ok();
            let code = commands::chassis::power_shortcut(&mut m, &args(&[arg]));
            (code, m.recorded())
        });
        assert_eq!(code, 0, "arg {arg}");
        assert_eq!(reqs.len(), 1, "arg {arg}");
        assert_eq!(reqs[0].cmd, 0x02, "arg {arg}");
        assert_eq!(reqs[0].data, vec![ctl], "arg {arg}");
        assert!(out.contains(expected), "arg {arg} stdout: {out}");
    }
}

#[test]
fn chassis_status_prints_labels() {
    // Power on, overload+interlock, restore policy previous; ac-failed + command;
    // intrusion + cooling fault.
    let (code, out, _err) = capture(|| {
        let mut m = MockInterface::new(|_| Ok(ok(vec![0x01 | 0x02 | 0x04 | 0x20, 0x01 | 0x10, 0x01 | 0x08])));
        commands::chassis::main(&mut m, &args(&["status"]))
    });
    assert_eq!(code, 0);
    for want in [
        "System Power         : on",
        "Power Overload       : true",
        "Power Interlock      : active",
        "Power Restore Policy : previous",
        "ac-failed ",
        "command",
        "Chassis Intrusion    : active",
        "Cooling/Fan Fault    : true",
    ] {
        assert!(out.contains(want), "missing {want:?} in: {out}");
    }
}

#[test]
fn chassis_identify_force() {
    let ((code, reqs), out, _err) = capture(|| {
        let mut m = MockInterface::always_ok();
        let code = commands::chassis::main(&mut m, &args(&["identify", "force"]));
        (code, m.recorded())
    });
    assert_eq!(code, 0);
    assert_eq!(reqs[0].cmd, 0x04);
    assert_eq!(reqs[0].data, vec![0x00, 0x01]);
    assert!(out.contains("Chassis identify interval: indefinite"), "stdout: {out}");
}

#[test]
fn chassis_selftest_passed() {
    let (code, out, _err) = capture(|| {
        let mut m = MockInterface::new(|_| Ok(ok(vec![0x55, 0x00])));
        commands::chassis::main(&mut m, &args(&["selftest"]))
    });
    assert_eq!(code, 0);
    assert!(out.contains("Self Test Results    : passed"), "stdout: {out}");
}

// ---------------------------------------------------------------------------
// mc
// ---------------------------------------------------------------------------

fn device_id_data() -> Vec<u8> {
    let mut d = vec![0u8; 15];
    d[0] = 0x20; // device id
    d[1] = 0x81; // revision, bit7 -> provides SDRs
    d[2] = 0x01;
    d[3] = 0x23;
    d[4] = 0x02; // IPMI 2.0
    d[5] = 0x05; // sensor + SEL
    d[6] = (343 & 0xff) as u8;
    d[7] = ((343 >> 8) & 0xff) as u8;
    d[8] = 0; // manufacturer id high byte (343 fits in two bytes)
    d[9] = 0x0c;
    d[10] = 0x00;
    d
}

#[test]
fn mc_info() {
    let (code, out, _err) = capture(|| {
        let mut m = MockInterface::new(|_| Ok(ok(device_id_data())));
        commands::mc::main(&mut m, &args(&["info"]))
    });
    assert_eq!(code, 0);
    for want in [
        "Device ID                 : 32",
        "Firmware Revision         : 1.23",
        "IPMI Version              : 2.0",
        "Manufacturer ID           : 343",
        "Product ID                : 12 (0x000c)",
        "Product Name              : TSRLT2",
        "Device Available          : yes",
        "Provides Device SDRs      : yes",
        "    Sensor Device",
        "    SEL Device",
        "Aux Firmware Rev Info     : ",
    ] {
        assert!(out.contains(want), "missing {want:?} in: {out}");
    }
}

#[test]
fn mc_getenables() {
    let (code, out, _err) = capture(|| {
        let mut m = MockInterface::new(|_| Ok(ok(vec![0x0c])));
        commands::mc::main(&mut m, &args(&["getenables"]))
    });
    assert_eq!(code, 0);
    assert!(
        out.contains("Event Message Buffer                     : enabled"),
        "stdout: {out}"
    );
    assert!(
        out.contains("Receive Message Queue Interrupt          : disabled"),
        "stdout: {out}"
    );
}

#[test]
fn mc_reset_cold_tolerates_no_response() {
    let (code, out, _err) = capture(|| {
        let mut m = MockInterface::new(|_| Err(Error::Timeout));
        commands::mc::main(&mut m, &args(&["reset", "cold"]))
    });
    assert_eq!(code, 0);
    assert!(out.contains("Sent cold reset command to MC"), "stdout: {out}");
}

#[test]
fn mc_selftest_fatal() {
    let (code, out, _err) = capture(|| {
        let mut m = MockInterface::new(|_| Ok(ok(vec![0x58, 0x42])));
        commands::mc::main(&mut m, &args(&["selftest"]))
    });
    assert_eq!(code, -1);
    assert!(out.contains("Selftest     : fatal error"), "stdout: {out}");
    assert!(out.contains("Failure code : 42"), "stdout: {out}");
}

// ---------------------------------------------------------------------------
// user
// ---------------------------------------------------------------------------

#[test]
fn user_summary() {
    let ((code, reqs), out, _err) = capture(|| {
        let mut m = MockInterface::new(|_| Ok(ok(vec![10, 3, 1, 0x04])));
        let code = commands::user::main(&mut m, &args(&["summary"]));
        (code, m.recorded())
    });
    assert_eq!(code, 0);
    assert!(out.contains("Maximum IDs\t    : 10"), "stdout: {out}");
    assert!(out.contains("Enabled User Count  : 3"), "stdout: {out}");
    assert!(out.contains("Fixed Name Count    : 1"), "stdout: {out}");
    assert_eq!(reqs[0].cmd, 0x44);
    assert_eq!(reqs[0].data[0], 0x0e);
}

#[test]
fn user_list() {
    let (code, out, _err) = capture(|| {
        let mut m = MockInterface::new(|req| match (req.netfn, req.cmd) {
            (consts::NETFN_APP, 0x44) => match req.data[1] {
                1 => Ok(ok(vec![2, 0x03, 1, 0x14])),
                2 => Ok(ok(vec![2, 0x02, 0, 0x04])),
                _ => Ok(ok(vec![2, 0, 0, 0])),
            },
            (consts::NETFN_APP, 0x46) => {
                let name = match req.data[0] {
                    1 => "root",
                    2 => "admin",
                    _ => "",
                };
                let mut d = vec![0u8; 16];
                d[..name.len()].copy_from_slice(name.as_bytes());
                Ok(ok(d))
            }
            _ => Ok(ccode(0xc1)),
        });
        commands::user::main(&mut m, &args(&["list"]))
    });
    assert_eq!(code, 0);
    assert!(
        out.contains("ID  Name\t     Callin  Link Auth\tIPMI Msg   Channel Priv Limit"),
        "stdout: {out}"
    );
    assert!(out.contains("1   root"), "stdout: {out}");
    assert!(out.contains("2   admin"), "stdout: {out}");
    assert!(out.contains("ADMINISTRATOR"), "stdout: {out}");
}

#[test]
fn user_set_name() {
    let ((code, reqs), _out, err) = capture(|| {
        let mut m = MockInterface::always_ok();
        let code = commands::user::main(&mut m, &args(&["set", "name", "3", "bob"]));
        (code, m.recorded())
    });
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(reqs[0].cmd, 0x45);
    assert_eq!(reqs[0].data[0], 3);
    assert_eq!(&reqs[0].data[1..4], b"bob");
    assert_eq!(reqs[0].data.len(), 17);
}

#[test]
fn user_disable_enable() {
    for (cmd, op) in [("disable", 0x00u8), ("enable", 0x01)] {
        let ((code, reqs), _out, err) = capture(move || {
            let mut m = MockInterface::always_ok();
            let code = commands::user::main(&mut m, &args(&[cmd, "5"]));
            (code, m.recorded())
        });
        assert_eq!(code, 0, "{cmd} stderr: {err}");
        assert_eq!(reqs[0].cmd, 0x47);
        assert_eq!(reqs[0].data[0], 5);
        assert_eq!(reqs[0].data[1], 0x03 & op);
    }
}

#[test]
fn user_test_password_success() {
    let ((code, reqs), out, _err) = capture(|| {
        let mut m = MockInterface::always_ok();
        let code = commands::user::main(&mut m, &args(&["test", "2", "20", "secret"]));
        (code, m.recorded())
    });
    assert_eq!(code, 0);
    assert_eq!(reqs[0].data[0], 0x80 | 2);
    assert_eq!(reqs[0].data[1], 0x03);
    assert_eq!(&reqs[0].data[2..8], b"secret");
    assert!(out.contains("Success"), "stdout: {out}");
}

// ---------------------------------------------------------------------------
// sol
// ---------------------------------------------------------------------------

const GET_SOL: u8 = sol::IPMI_GET_SOL_CONFIG_PARAMETERS;

fn sol_param_response(selector: u8) -> Vec<u8> {
    match selector {
        sol::PARAM_SET_IN_PROGRESS => vec![selector, 0x00],
        sol::PARAM_SOL_ENABLE => vec![selector, 0x01],
        sol::PARAM_SOL_AUTHENTICATION => vec![selector, 0x80 | 0x40 | 0x04],
        sol::PARAM_CHARACTER_INTERVAL => vec![selector, 0x0a, 0x05],
        sol::PARAM_SOL_RETRY => vec![selector, 0x03, 0x02],
        sol::PARAM_NON_VOLATILE_BIT_RATE => vec![selector, 0x0a],
        sol::PARAM_VOLATILE_BIT_RATE => vec![selector, 0x0a],
        sol::PARAM_PAYLOAD_CHANNEL => vec![selector, 0x01],
        sol::PARAM_PAYLOAD_PORT => vec![selector, 0x6f, 0x02],
        _ => vec![selector],
    }
}

#[test]
fn sol_info() {
    let (code, out, _err) = capture(|| {
        let mut m = MockInterface::new(|req| {
            if req.netfn == consts::NETFN_TRANSPORT && req.cmd == GET_SOL {
                Ok(ok(sol_param_response(req.data[1])))
            } else {
                Ok(ccode(cc::CC_INV_CMD))
            }
        });
        commands::sol::main(&mut m, &args(&["info"]))
    });
    assert_eq!(code, 0);
    for want in [
        "Set in progress                 : set-complete",
        "Enabled                         : true",
        "Force Encryption                : true",
        "Force Authentication            : true",
        "Privilege Level                 : ADMINISTRATOR",
        "Character Accumulate Level (ms) : 50",
        "Character Send Threshold        : 5",
        "Retry Count                     : 3",
        "Retry Interval (ms)             : 20",
        "Volatile Bit Rate (kbps)        : 115.2",
        "Non-Volatile Bit Rate (kbps)    : 115.2",
        "Payload Channel                 : 1 (0x01)",
        "Payload Port                    : 623",
    ] {
        assert!(out.contains(want), "missing {want:?} in: {out}");
    }
}

#[test]
fn sol_payload_status() {
    let ((code, reqs), out, _err) = capture(|| {
        let mut m = MockInterface::new(|_| Ok(ok(vec![0x02, 0, 0, 0])));
        let code = commands::sol::main(&mut m, &args(&["payload", "status", "1", "2"]));
        (code, m.recorded())
    });
    assert_eq!(code, 0);
    assert_eq!(reqs[0].cmd, sol::IPMI_GET_USER_PAYLOAD_ACCESS);
    assert_eq!(&reqs[0].data[..2], &[1, 2]);
    assert!(out.contains("User 2 on channel 1 is enabled"), "stdout: {out}");
}

#[test]
fn sol_set_enabled_guarded_sequence() {
    let ((code, reqs), _out, err) = capture(|| {
        let mut m = MockInterface::always_ok();
        let code = commands::sol::main(&mut m, &args(&["set", "enabled", "true"]));
        (code, m.recorded())
    });
    assert_eq!(code, 0, "stderr: {err}");
    let want: Vec<Vec<u8>> = vec![
        vec![0x0e, sol::PARAM_SET_IN_PROGRESS, 0x01],
        vec![0x0e, sol::PARAM_SOL_ENABLE, 0x01],
        vec![0x0e, sol::PARAM_SET_IN_PROGRESS, 0x02],
        vec![0x0e, sol::PARAM_SET_IN_PROGRESS, 0x00],
    ];
    assert_eq!(reqs.len(), want.len());
    for (got, w) in reqs.iter().zip(want.iter()) {
        assert_eq!(&got.data, w, "guarded sequence mismatch");
    }
}

#[test]
fn sol_deactivate_unsupported() {
    let (code, _out, err) = capture(|| {
        let mut m = MockInterface::always_ok();
        commands::sol::main(&mut m, &args(&["deactivate"]))
    });
    assert_eq!(code, -1);
    assert!(err.contains("Error de-activating SOL payload"), "stderr: {err}");
}

// ---------------------------------------------------------------------------
// dcmi
// ---------------------------------------------------------------------------

/// Build an 18-byte DCMI Get Power Reading response.
fn power_reading_data(curr: u16, min: u16, max: u16, avg: u16, time: u32, sample: u32, state: u8) -> Vec<u8> {
    let mut d = vec![0u8; 18];
    d[0] = 0xDC;
    d[1..3].copy_from_slice(&curr.to_le_bytes());
    d[3..5].copy_from_slice(&min.to_le_bytes());
    d[5..7].copy_from_slice(&max.to_le_bytes());
    d[7..9].copy_from_slice(&avg.to_le_bytes());
    d[9..13].copy_from_slice(&time.to_le_bytes());
    d[13..17].copy_from_slice(&sample.to_le_bytes());
    d[17] = state;
    d
}

#[test]
fn dcmi_power_reading() {
    let ((code, reqs), out, _err) = capture(|| {
        let mut m = MockInterface::new(|_| {
            // 3661 < 86400 → relative "S+ 01:01:01"; 3000 ms → 3 seconds.
            Ok(ok(power_reading_data(100, 90, 110, 95, 3661, 3000, 0x40)))
        });
        let code = commands::dcmi::main(&mut m, &args(&["power", "reading"]));
        (code, m.recorded())
    });

    assert_eq!(code, 0);
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].netfn, consts::NETFN_DCGRP);
    assert_eq!(reqs[0].cmd, 0x02);
    assert_eq!(reqs[0].data, vec![0xDC, 0x01, 0x00, 0x00]);
    for want in [
        "    Instantaneous power reading:                   100 Watts",
        "    Minimum during sampling period:                 90 Watts",
        "    Maximum during sampling period:                110 Watts",
        "    Average power reading over sample period:       95 Watts",
        "    IPMI timestamp:                           S+ 01:01:01",
        "    Sampling period:                          00000003 Seconds.",
        "    Power reading state is:                   activated",
    ] {
        assert!(out.contains(want), "missing {want:?} in: {out}");
    }
}

#[test]
fn dcmi_power_reading_enhanced_uses_sample_selector() {
    let ((code, reqs), out, _err) = capture(|| {
        let mut m = MockInterface::new(|_| {
            // sample field carries the same selector the request used.
            Ok(ok(power_reading_data(0, 0, 0, 0, 0, 0x41, 0x00)))
        });
        let code = commands::dcmi::main(&mut m, &args(&["power", "reading", "1_min"]));
        (code, m.recorded())
    });

    assert_eq!(code, 0);
    assert_eq!(reqs[0].data, vec![0xDC, 0x02, 0x41, 0x00]);
    assert!(out.contains("1_min "), "stdout: {out}");
    assert!(out.contains("deactivated"), "stdout: {out}");
}

#[test]
fn dcmi_power_reading_invalid_sample_time() {
    let (code, out, err) = capture(|| {
        let mut m = MockInterface::always_ok();
        commands::dcmi::main(&mut m, &args(&["power", "reading", "bogus"]))
    });
    assert_eq!(code, -1);
    assert!(
        err.contains("Invalid sample time. Valid times are: "),
        "stderr: {err}"
    );
    // The horizontal listing emits " | " separators on stdout.
    assert!(out.contains(" | "), "stdout: {out}");
}

#[test]
fn dcmi_power_reading_dcmi_ccode() {
    let (code, _out, err) = capture(|| {
        let mut m = MockInterface::new(|_| Ok(ccode(0x84)));
        commands::dcmi::main(&mut m, &args(&["power", "reading"]))
    });
    assert_eq!(code, -1);
    assert!(
        err.contains("DCMI request failed because: Power/Thermal limit out of range (84)"),
        "stderr: {err}"
    );
}

#[test]
fn dcmi_power_reading_wrong_group_extension() {
    let (code, out, _err) = capture(|| {
        let mut m = MockInterface::new(|_| {
            let mut d = power_reading_data(0, 0, 0, 0, 0, 0, 0);
            d[0] = 0x00;
            Ok(ok(d))
        });
        commands::dcmi::main(&mut m, &args(&["power", "reading"]))
    });
    assert_eq!(code, -1);
    assert!(
        out.contains("A valid DCMI command was not returned! (0)"),
        "stdout: {out}"
    );
}

#[test]
fn dcmi_help_lists_commands() {
    let (code, _out, err) = capture(|| {
        let mut m = MockInterface::always_ok();
        commands::dcmi::main(&mut m, &[] as &[String])
    });
    assert_eq!(code, -1);
    assert!(
        err.contains("Data Center Management Interface commands"),
        "stderr: {err}"
    );
    assert!(err.contains("    power    Platform power limit command options"), "stderr: {err}");
}

