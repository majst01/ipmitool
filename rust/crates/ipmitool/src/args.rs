//! Hand-written command-line parser mirroring ipmitool's `getopt` handling in
//! `lib/ipmi_main.c`.  We do not depend on an argument-parsing crate so that
//! behaviour (including combined short flags such as `-vv`) matches closely.

use ipmi_core::consts;
use ipmi_core::values;

/// Parsed command line.
pub enum Parsed {
    /// Run a command with the collected options.
    Run(Options, String, Vec<String>),
    /// `-h` / `help`: print usage and exit successfully.
    Help,
    /// `-V`: print version information and exit successfully.
    Version,
}

/// Options collected from the command line.
pub struct Options {
    pub intf: Option<String>,
    pub hostname: Option<String>,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub cipher: Option<u8>,
    pub privlvl: u8,
    pub lookupbit: u8,
    pub authtype: Option<u8>,
    pub timeout: u32,
    pub retry: u32,
    pub target_lun: u8,
    pub target_addr: u8,
    pub target_channel: u8,
    pub transit_addr: u8,
    pub transit_channel: u8,
    pub my_addr: u8,
    pub kgkey: Option<String>,
    pub verbose: u32,
    pub csv: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            intf: None,
            hostname: None,
            port: 0,
            username: None,
            password: None,
            cipher: None,
            privlvl: consts::SESSION_PRIV_ADMIN,
            lookupbit: 0x10,
            authtype: None,
            timeout: 0,
            retry: 0,
            target_lun: 0,
            target_addr: 0,
            target_channel: 0,
            transit_addr: 0,
            transit_channel: 0,
            my_addr: consts::BMC_SLAVE_ADDR,
            kgkey: None,
            verbose: 0,
            csv: false,
        }
    }
}

/// Parse a number that may be decimal or hex (`0x..`) into a `u32`.
pub fn parse_num(s: &str) -> Option<u32> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u32>().ok()
    }
}

/// Parse a `u8` the way ipmitool's `str2uchar` does.
pub fn parse_u8(s: &str) -> Option<u8> {
    parse_num(s).and_then(|v| u8::try_from(v).ok())
}

/// Parse the command line.  Returns an error string on malformed input.
pub fn parse(argv: &[String]) -> Result<Parsed, String> {
    let mut opts = Options::default();
    let mut positionals: Vec<String> = Vec::new();

    let mut i = 0;
    while i < argv.len() {
        let arg = &argv[i];
        if arg == "--" {
            positionals.extend_from_slice(&argv[i + 1..]);
            break;
        }
        if arg.len() >= 2 && arg.starts_with('-') && arg != "-" {
            let chars: Vec<char> = arg[1..].chars().collect();
            let mut ci = 0;
            while ci < chars.len() {
                let c = chars[ci];
                // Options that take a value.
                let takes_value = matches!(
                    c,
                    'I' | 'H'
                        | 'p'
                        | 'U'
                        | 'P'
                        | 'f'
                        | 'k'
                        | 'y'
                        | 'S'
                        | 'D'
                        | 'O'
                        | 'C'
                        | 'L'
                        | 'A'
                        | 't'
                        | 'T'
                        | 'b'
                        | 'B'
                        | 'l'
                        | 'm'
                        | 'e'
                        | 'o'
                        | 'N'
                        | 'R'
                        | 'd'
                        | 'z'
                );
                let value = if takes_value {
                    let rest: String = chars[ci + 1..].iter().collect();
                    if !rest.is_empty() {
                        Some(rest)
                    } else {
                        i += 1;
                        argv.get(i).cloned()
                    }
                } else {
                    None
                };

                match c {
                    'I' => opts.intf = value,
                    'H' => opts.hostname = value,
                    'p' => {
                        opts.port = parse_num(&req(value, "-p")?)
                            .filter(|v| *v <= 65535)
                            .ok_or("invalid port")? as u16
                    }
                    'U' => {
                        let v = req(value, "-U")?;
                        if v.len() > 16 {
                            return Err("Username is too long (> 16 bytes)".into());
                        }
                        opts.username = Some(v);
                    }
                    'P' => opts.password = value,
                    'f' => {
                        let path = req(value, "-f")?;
                        opts.password = Some(read_password_file(&path)?);
                    }
                    'a' | 'Y' => {
                        opts.password = Some(prompt_password(if c == 'Y' {
                            "Key: "
                        } else {
                            "Password: "
                        }));
                    }
                    'E' => {
                        opts.password = std::env::var("IPMITOOL_PASSWORD")
                            .or_else(|_| std::env::var("IPMI_PASSWORD"))
                            .ok();
                    }
                    'k' => opts.kgkey = value,
                    'y' => opts.kgkey = value,
                    'K' => opts.kgkey = std::env::var("IPMI_KGKEY").ok(),
                    'C' => {
                        opts.cipher =
                            Some(parse_u8(&req(value, "-C")?).ok_or("invalid cipher suite")?)
                    }
                    'L' => {
                        let mut v = req(value, "-L")?;
                        if v.ends_with('+') {
                            opts.lookupbit = 0;
                            v.pop();
                        }
                        opts.privlvl = values::privlvl_from_str(&v)
                            .ok_or_else(|| format!("Invalid privilege level {v}"))?;
                    }
                    'A' => {
                        opts.authtype = Some(
                            values::authtype_from_str(&req(value, "-A")?)
                                .ok_or("invalid authentication type")?,
                        )
                    }
                    't' => {
                        opts.target_addr = parse_u8(&req(value, "-t")?).ok_or("invalid address")?
                    }
                    'T' => {
                        opts.transit_addr = parse_u8(&req(value, "-T")?).ok_or("invalid address")?
                    }
                    'b' => {
                        opts.target_channel =
                            parse_u8(&req(value, "-b")?).ok_or("invalid channel")?
                    }
                    'B' => {
                        opts.transit_channel =
                            parse_u8(&req(value, "-B")?).ok_or("invalid channel")?
                    }
                    'l' => opts.target_lun = parse_u8(&req(value, "-l")?).ok_or("invalid lun")?,
                    'm' => opts.my_addr = parse_u8(&req(value, "-m")?).ok_or("invalid address")?,
                    'e' => {
                        // SOL escape character; accepted and ignored for now.
                        let _ = value;
                    }
                    'o' | 'g' | 's' | 'S' | 'D' | 'O' | 'd' | 'z' => {
                        // OEM / device / serial / caches: accepted but unused.
                    }
                    'N' => opts.timeout = parse_num(&req(value, "-N")?).ok_or("invalid timeout")?,
                    'R' => {
                        opts.retry = parse_num(&req(value, "-R")?).ok_or("invalid retry count")?
                    }
                    'v' => opts.verbose += 1,
                    'c' => opts.csv = true,
                    '4' | '6' | 'Z' => {}
                    'h' => return Ok(Parsed::Help),
                    'V' => return Ok(Parsed::Version),
                    other => return Err(format!("invalid option -- '{other}'")),
                }
                ci += 1;
            }
        } else {
            positionals.push(arg.clone());
        }
        i += 1;
    }

    if positionals.is_empty() {
        return Err("no command provided".into());
    }
    let command = positionals.remove(0);
    if command == "help" {
        return Ok(Parsed::Help);
    }
    Ok(Parsed::Run(opts, command, positionals))
}

fn req(value: Option<String>, flag: &str) -> Result<String, String> {
    value.ok_or_else(|| format!("option {flag} requires an argument"))
}

fn read_password_file(path: &str) -> Result<String, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("Unable to read password from file {path}: {e}"))?;
    let line = contents.lines().next().unwrap_or("");
    Ok(line.trim_end_matches(['\r', '\n', '\t']).to_string())
}

/// Read a password from the terminal without disabling echo (there is no
/// external crate for that in this milestone).
fn prompt_password(prompt: &str) -> String {
    use std::io::Write;
    let mut stderr = std::io::stderr();
    let _ = write!(stderr, "{prompt}");
    let _ = stderr.flush();
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line);
    line.trim_end_matches(['\r', '\n']).to_string()
}
