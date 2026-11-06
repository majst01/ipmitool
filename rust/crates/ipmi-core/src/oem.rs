//! OEM / IANA Private Enterprise Number registry.
//!
//! This is the Rust counterpart to `ipmi_oem_info` and
//! `ipmi_oem_product_info` from `lib/ipmi_strings.c`. Manufacturer names are
//! loaded at runtime from the IANA PEN registry (the same
//! `enterprise-numbers` file the C implementation reads), while product names
//! come from the generated [`oem_data`] table.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

mod oem_data;

pub use oem_data::{OemProduct, IPMI_OEM_PICMG, OEM_PRODUCT_INFO};

/// IANA PEN used for the "Unknown" manufacturer, as per `IPMI_OEM_UNKNOWN`.
const OEM_UNKNOWN: u32 = 0;
/// IANA PEN used for the "Unspecified" manufacturer (`IPMI_OEM_RESERVED`).
const OEM_RESERVED: u32 = 0x000F_FFFF;
/// Sentinel enterprise number for the built-in debug entry (`IPMI_OEM_DEBUG`).
const OEM_DEBUG: u32 = 0x00FF_FFFE;
/// Debug manufacturer name, from `ipmi_oem_info_tail`.
const OEM_DEBUG_NAME: &str = "A Debug Assisting Company, Ltd.";

/// Format an unmatched value the way C's `unknown_val_str()` does.
pub fn unknown_val_str(val: u32) -> String {
    format!("Unknown (0x{val:02X})")
}

/// Runtime registry of IANA Private Enterprise Numbers mapped to names.
#[derive(Default)]
pub struct OemRegistry {
    names: HashMap<u32, String>,
}

impl OemRegistry {
    /// An empty registry, used when the PEN file cannot be read. Manufacturer
    /// lookups still resolve the built-in head and tail entries.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Load the registry from the first readable IANA PEN file, mirroring
    /// `oem_info_list_load()`.
    pub fn load() -> Self {
        let mut names = HashMap::new();
        for path in search_paths() {
            if let Ok(text) = fs::read_to_string(&path) {
                parse_registry(&text, &mut names);
                break;
            }
        }
        Self { names }
    }

    /// Number of dynamically loaded registry entries.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Resolve a manufacturer id to a name, matching `OEM_MFG_STRING()`.
    pub fn manufacturer_name(&self, id: u32) -> String {
        match id {
            OEM_UNKNOWN => "Unknown".to_string(),
            OEM_RESERVED => "Unspecified".to_string(),
            _ => {
                if let Some(name) = self.names.get(&id) {
                    name.clone()
                } else if id == OEM_DEBUG {
                    OEM_DEBUG_NAME.to_string()
                } else {
                    unknown_val_str(id)
                }
            }
        }
    }

    /// Resolve a product name from the manufacturer and product id, matching
    /// `OEM_PROD_STRING()`. As in C, PICMG entries match every manufacturer.
    pub fn product_name(&self, oem: u32, product: u16) -> String {
        for entry in OEM_PRODUCT_INFO {
            if (entry.oem == oem || entry.oem == IPMI_OEM_PICMG) && entry.product == product {
                return entry.name.to_string();
            }
        }
        unknown_val_str(u32::from(product))
    }
}

/// Candidate PEN files, in precedence order. The system paths cover both the
/// ipmitool-installed `enterprise-numbers` and the `iana-etc`
/// `enterprise-numbers.txt`.
fn search_paths() -> Vec<PathBuf> {
    let file_names = ["enterprise-numbers", "enterprise-numbers.txt"];
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/usr/share/misc"));
    }
    dirs.push(PathBuf::from("/usr/share/misc"));
    dirs.push(PathBuf::from("/usr/local/share/misc"));

    let mut paths = Vec::new();
    for dir in dirs {
        for name in file_names {
            paths.push(dir.join(name));
        }
    }
    paths
}

/// Parse the PEN registry lines into `names`. See `oem_info_list_load()` for
/// the expected format; entries read later in the file win on duplicates, as
/// they end up earlier in the C lookup array.
fn parse_registry(text: &str, names: &mut HashMap<u32, String>) {
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if !line.starts_with(|c: char| c.is_ascii_digit()) {
            i += 1;
            continue;
        }

        let digit_end = line
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(line.len());
        let pen = line[..digit_end].parse::<u64>().ok();
        i += 1;

        // The organization name is expected on the immediately following line
        // and that line is consumed whether or not it is valid.
        if i >= lines.len() {
            break;
        }
        let org = lines[i];
        i += 1;

        if let (Some(pen), Some(name)) = (pen, org_name(org)) {
            if pen <= u64::from(u32::MAX) {
                names.insert(pen as u32, name);
            }
        }
    }
}

/// Extract the organization name from an indented registry line. Requires
/// exactly two leading spaces, as C's `count_bytes(line, ' ') == 2` check.
fn org_name(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    if bytes.len() < 3 {
        return None;
    }
    if bytes[0] != b' ' || bytes[1] != b' ' || bytes[2] == b' ' {
        return None;
    }
    Some(line[2..].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_registry_entries() {
        let text = "PRIVATE ENTERPRISE NUMBERS\n\n343\n  Intel Corporation\n    Adam\n1\n  Foo Bar\n    baz\n";
        let mut names = HashMap::new();
        parse_registry(text, &mut names);
        assert_eq!(
            names.get(&343).map(String::as_str),
            Some("Intel Corporation")
        );
        assert_eq!(names.get(&1).map(String::as_str), Some("Foo Bar"));
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn ignores_malformed_org_lines() {
        let text = "10\n   Three spaces\n20\nnot indented\n30\n  Good Name\n";
        let mut names = HashMap::new();
        parse_registry(text, &mut names);
        assert!(!names.contains_key(&10));
        assert!(!names.contains_key(&20));
        assert_eq!(names.get(&30).map(String::as_str), Some("Good Name"));
    }

    #[test]
    fn builtin_manufacturer_names() {
        let reg = OemRegistry::empty();
        assert_eq!(reg.manufacturer_name(0), "Unknown");
        assert_eq!(reg.manufacturer_name(OEM_RESERVED), "Unspecified");
        assert_eq!(reg.manufacturer_name(OEM_DEBUG), OEM_DEBUG_NAME);
        assert_eq!(reg.manufacturer_name(999), "Unknown (0x3E7)");
    }

    #[test]
    fn product_name_lookup() {
        let reg = OemRegistry::empty();
        // IPMI_OEM_INTEL = 343, product 0x000C = "TSRLT2".
        assert_eq!(reg.product_name(343, 0x000C), "TSRLT2");
        assert_eq!(reg.product_name(343, 0xFFFF), "Unknown (0xFFFF)");
    }
}
