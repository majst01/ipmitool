//! `valstr`-style enumerations used by commands for parsing arguments and
//! printing results.

/// A name/description pair used when printing help and status.
pub struct ValStr {
    pub val: u8,
    pub name: &'static str,
}

impl ValStr {
    const fn new(val: u8, name: &'static str) -> Self {
        Self { val, name }
    }
}

const PRIVLVL_VALS: &[ValStr] = &[
    ValStr::new(crate::consts::SESSION_PRIV_CALLBACK, "CALLBACK"),
    ValStr::new(crate::consts::SESSION_PRIV_USER, "USER"),
    ValStr::new(crate::consts::SESSION_PRIV_OPERATOR, "OPERATOR"),
    ValStr::new(crate::consts::SESSION_PRIV_ADMIN, "ADMINISTRATOR"),
    ValStr::new(crate::consts::SESSION_PRIV_OEM, "OEM"),
    ValStr::new(crate::consts::SESSION_PRIV_NOACCESS, "NO ACCESS"),
];

const AUTH_TYPE_VALS: &[ValStr] = &[
    ValStr::new(0x00, "NONE"),
    ValStr::new(0x01, "MD2"),
    ValStr::new(0x02, "MD5"),
    ValStr::new(0x04, "PASSWORD"),
    ValStr::new(0x05, "OEM"),
];

const NETFN_VALS: &[ValStr] = &[
    ValStr::new(crate::consts::NETFN_CHASSIS, "Chassis"),
    ValStr::new(crate::consts::NETFN_BRIDGE, "Bridge"),
    ValStr::new(crate::consts::NETFN_SE, "SensorEvent"),
    ValStr::new(crate::consts::NETFN_APP, "Application"),
    ValStr::new(crate::consts::NETFN_FIRMWARE, "Firmware"),
    ValStr::new(crate::consts::NETFN_STORAGE, "Storage"),
    ValStr::new(crate::consts::NETFN_TRANSPORT, "Transport"),
    ValStr::new(crate::consts::NETFN_PICMG, "PICMG"),
    ValStr::new(crate::consts::NETFN_OEM, "OEM"),
    ValStr::new(crate::consts::NETFN_ISOL, "ISOL"),
    ValStr::new(crate::consts::NETFN_TSOL, "TSOL"),
];

const POWER_CONTROL_VALS: &[ValStr] = &[
    ValStr::new(0x0, "Down/Off"),
    ValStr::new(0x1, "Up/On"),
    ValStr::new(0x2, "Cycle"),
    ValStr::new(0x3, "Reset"),
    ValStr::new(0x4, "Diag"),
    ValStr::new(0x5, "Soft"),
];

const CHASSIS_RESTART_CAUSE_VALS: &[ValStr] = &[
    ValStr::new(0x0, "unknown"),
    ValStr::new(0x1, "chassis power control command"),
    ValStr::new(0x2, "reset via pushbutton"),
    ValStr::new(0x3, "power-up via pushbutton"),
    ValStr::new(0x4, "watchdog expired"),
    ValStr::new(0x5, "OEM"),
    ValStr::new(0x6, "power-up due to always-restore power policy"),
    ValStr::new(0x7, "power-up due to restore-previous power policy"),
    ValStr::new(0x8, "reset via PEF"),
    ValStr::new(0x9, "power-cycle via PEF"),
    ValStr::new(0xa, "soft reset"),
    ValStr::new(0xb, "power-up via RTC wakeup"),
];

fn lookup(vals: &[ValStr], s: &str, case_sensitive: bool) -> Option<u8> {
    for v in vals {
        let matches = if case_sensitive {
            v.name == s
        } else {
            v.name.eq_ignore_ascii_case(s)
        };
        if matches {
            return Some(v.val);
        }
    }
    None
}

fn name_of(vals: &[ValStr], val: u8) -> String {
    for v in vals {
        if v.val == val {
            return v.name.to_string();
        }
    }
    format!("Unknown (0x{val:02x})")
}

/// Parse a privilege level name (case-insensitive). Returns `None` if
/// unrecognised.
pub fn privlvl_from_str(s: &str) -> Option<u8> {
    lookup(PRIVLVL_VALS, s, false)
}

pub fn privlvl_str(val: u8) -> String {
    name_of(PRIVLVL_VALS, val)
}

pub fn authtype_from_str(s: &str) -> Option<u8> {
    lookup(AUTH_TYPE_VALS, s, false)
}

pub fn netfn_from_str(s: &str) -> Option<u8> {
    lookup(NETFN_VALS, s, false)
}

pub fn power_control_str(val: u8) -> String {
    name_of(POWER_CONTROL_VALS, val)
}

pub fn chassis_restart_cause_str(val: u8) -> String {
    name_of(CHASSIS_RESTART_CAUSE_VALS, val)
}

pub fn print_valstr(vals: &[ValStr], title: &str) {
    if !title.is_empty() {
        eprintln!("{title}");
    }
    for v in vals {
        eprintln!("\t{:<16}  {}", v.val, v.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Strings mirror the C tables in lib/ipmi_strings.c
    // (ipmi_chassis_power_control_vals / ipmi_chassis_restart_cause_vals).
    #[test]
    fn power_control_strings_match_c() {
        assert_eq!(power_control_str(0x0), "Down/Off");
        assert_eq!(power_control_str(0x1), "Up/On");
        assert_eq!(power_control_str(0x2), "Cycle");
        assert_eq!(power_control_str(0x3), "Reset");
        assert_eq!(power_control_str(0x4), "Diag");
        assert_eq!(power_control_str(0x5), "Soft");
    }

    #[test]
    fn restart_cause_strings_match_c() {
        assert_eq!(
            chassis_restart_cause_str(0x1),
            "chassis power control command"
        );
        assert_eq!(chassis_restart_cause_str(0x3), "power-up via pushbutton");
        assert_eq!(chassis_restart_cause_str(0x4), "watchdog expired");
        assert_eq!(chassis_restart_cause_str(0x8), "reset via PEF");
        assert_eq!(chassis_restart_cause_str(0x9), "power-cycle via PEF");
        assert_eq!(chassis_restart_cause_str(0xa), "soft reset");
        assert_eq!(chassis_restart_cause_str(0xb), "power-up via RTC wakeup");
    }
}
