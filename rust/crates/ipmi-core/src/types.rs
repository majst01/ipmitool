//! Core protocol data types.

use crate::consts;

/// An IPMI request message.
#[derive(Debug, Clone, Default)]
pub struct Request {
    pub netfn: u8,
    pub lun: u8,
    pub cmd: u8,
    pub data: Vec<u8>,
}

impl Request {
    pub fn new(netfn: u8, cmd: u8) -> Self {
        Request {
            netfn,
            lun: 0,
            cmd,
            data: Vec::new(),
        }
    }

    pub fn with_data(mut self, data: impl Into<Vec<u8>>) -> Self {
        self.data = data.into();
        self
    }

    pub fn with_lun(mut self, lun: u8) -> Self {
        self.lun = lun;
        self
    }
}

/// An IPMI response message.
#[derive(Debug, Clone, Default)]
pub struct Response {
    pub ccode: u8,
    pub data: Vec<u8>,
    pub netfn: u8,
    pub cmd: u8,
    pub rq_seq: u8,
}

impl Response {
    pub fn ok(&self) -> bool {
        self.ccode == 0
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// One of the standard IPMI v2 cipher suites (table 22-19).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CipherSuite {
    pub id: u8,
    pub auth_alg: u8,
    pub integrity_alg: u8,
    pub crypt_alg: u8,
}

impl CipherSuite {
    /// Look up the algorithms for a standard cipher suite id. Returns `None`
    /// for reserved / unsupported suites.
    pub fn from_id(id: u8) -> Option<CipherSuite> {
        use consts::*;
        let (auth_alg, integrity_alg, crypt_alg) = match id {
            0 => (AUTH_RAKP_NONE, INTEGRITY_NONE, CRYPT_NONE),
            1 => (AUTH_RAKP_HMAC_SHA1, INTEGRITY_NONE, CRYPT_NONE),
            2 => (AUTH_RAKP_HMAC_SHA1, INTEGRITY_HMAC_SHA1_96, CRYPT_NONE),
            3 => (
                AUTH_RAKP_HMAC_SHA1,
                INTEGRITY_HMAC_SHA1_96,
                CRYPT_AES_CBC_128,
            ),
            4 => (AUTH_RAKP_HMAC_SHA1, INTEGRITY_HMAC_SHA1_96, CRYPT_XRC4_128),
            5 => (AUTH_RAKP_HMAC_SHA1, INTEGRITY_HMAC_SHA1_96, CRYPT_XRC4_40),
            6 => (AUTH_RAKP_HMAC_MD5, INTEGRITY_NONE, CRYPT_NONE),
            7 => (AUTH_RAKP_HMAC_MD5, INTEGRITY_HMAC_MD5_128, CRYPT_NONE),
            8 => (
                AUTH_RAKP_HMAC_MD5,
                INTEGRITY_HMAC_MD5_128,
                CRYPT_AES_CBC_128,
            ),
            9 => (AUTH_RAKP_HMAC_MD5, INTEGRITY_HMAC_MD5_128, CRYPT_XRC4_128),
            10 => (AUTH_RAKP_HMAC_MD5, INTEGRITY_HMAC_MD5_128, CRYPT_XRC4_40),
            11 => (AUTH_RAKP_HMAC_MD5, INTEGRITY_MD5_128, CRYPT_NONE),
            12 => (AUTH_RAKP_HMAC_MD5, INTEGRITY_MD5_128, CRYPT_AES_CBC_128),
            13 => (AUTH_RAKP_HMAC_MD5, INTEGRITY_MD5_128, CRYPT_XRC4_128),
            14 => (AUTH_RAKP_HMAC_MD5, INTEGRITY_MD5_128, CRYPT_XRC4_40),
            15 => (AUTH_RAKP_HMAC_SHA256, INTEGRITY_NONE, CRYPT_NONE),
            16 => (AUTH_RAKP_HMAC_SHA256, INTEGRITY_HMAC_SHA256_128, CRYPT_NONE),
            17 => (
                AUTH_RAKP_HMAC_SHA256,
                INTEGRITY_HMAC_SHA256_128,
                CRYPT_AES_CBC_128,
            ),
            _ => return None,
        };
        Some(CipherSuite {
            id,
            auth_alg,
            integrity_alg,
            crypt_alg,
        })
    }

    /// Number of bytes emitted for an integrity authcode with this suite.
    pub fn authcode_len(&self) -> usize {
        match self.integrity_alg {
            consts::INTEGRITY_HMAC_SHA1_96 => 12,
            consts::INTEGRITY_HMAC_MD5_128 | consts::INTEGRITY_MD5_128 => 16,
            consts::INTEGRITY_HMAC_SHA256_128 => 16,
            _ => 0,
        }
    }
}

/// Session parameters configured on the command line, mirroring
/// `struct ipmi_session_params`.
#[derive(Debug, Clone)]
pub struct SessionParams {
    pub hostname: Option<String>,
    pub username: [u8; 17],
    /// Authentication key (`Kuid`): the raw password, zero padded.
    pub password: [u8; consts::IPMI_AUTHCODE_BUFFER_SIZE + 1],
    pub authtype_set: Option<u8>,
    pub privlvl: u8,
    pub cipher_suite: Option<u8>,
    pub port: u16,
    pub retry: u32,
    pub timeout: u32,
    pub kg: [u8; consts::IPMI_KG_BUFFER_SIZE],
    pub lookupbit: u8,
    pub sol_escape_char: u8,
    pub target_lun: u8,
    pub target_addr: u8,
    pub target_channel: u8,
    pub transit_addr: u8,
    pub transit_channel: u8,
    pub my_addr: u32,
}

impl Default for SessionParams {
    fn default() -> Self {
        SessionParams {
            hostname: None,
            username: [0; 17],
            password: [0; consts::IPMI_AUTHCODE_BUFFER_SIZE + 1],
            authtype_set: None,
            privlvl: consts::SESSION_PRIV_ADMIN,
            cipher_suite: None,
            port: 0,
            retry: 0,
            timeout: 0,
            kg: [0; consts::IPMI_KG_BUFFER_SIZE],
            lookupbit: 0x10,
            sol_escape_char: b'~',
            target_lun: 0,
            target_addr: 0,
            target_channel: 0,
            transit_addr: 0,
            transit_channel: 0,
            my_addr: consts::BMC_SLAVE_ADDR as u32,
        }
    }
}

impl SessionParams {
    pub fn set_username(&mut self, username: &str) {
        self.username = [0; 17];
        let bytes = username.as_bytes();
        let n = bytes.len().min(16);
        self.username[..n].copy_from_slice(&bytes[..n]);
    }

    pub fn set_password(&mut self, password: &str) {
        self.password = [0; consts::IPMI_AUTHCODE_BUFFER_SIZE + 1];
        let bytes = password.as_bytes();
        let n = bytes.len().min(consts::IPMI_AUTHCODE_BUFFER_SIZE);
        self.password[..n].copy_from_slice(&bytes[..n]);
    }

    pub fn username_bytes(&self) -> &[u8] {
        let len = self.username.iter().position(|&b| b == 0).unwrap_or(17);
        &self.username[..len]
    }

    pub fn password_bytes(&self) -> &[u8; consts::IPMI_AUTHCODE_BUFFER_SIZE] {
        self.password[..consts::IPMI_AUTHCODE_BUFFER_SIZE]
            .try_into()
            .expect("fixed size")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cipher_suite_3_uses_sha1_and_aes() {
        let s = CipherSuite::from_id(3).unwrap();
        assert_eq!(s.auth_alg, consts::AUTH_RAKP_HMAC_SHA1);
        assert_eq!(s.integrity_alg, consts::INTEGRITY_HMAC_SHA1_96);
        assert_eq!(s.crypt_alg, consts::CRYPT_AES_CBC_128);
        assert_eq!(s.authcode_len(), 12);
    }

    #[test]
    fn cipher_suite_17_is_sha256() {
        let s = CipherSuite::from_id(17).unwrap();
        assert_eq!(s.auth_alg, consts::AUTH_RAKP_HMAC_SHA256);
        assert_eq!(s.integrity_alg, consts::INTEGRITY_HMAC_SHA256_128);
        assert_eq!(s.authcode_len(), 16);
    }

    #[test]
    fn unknown_cipher_suite_is_none() {
        assert!(CipherSuite::from_id(0xff).is_none());
        assert!(CipherSuite::from_id(18).is_none());
    }

    #[test]
    fn password_is_nul_padded() {
        let mut p = SessionParams::default();
        p.set_password("abc");
        assert_eq!(&p.password_bytes()[..3], b"abc");
        assert!(p.password_bytes()[3..].iter().all(|&b| b == 0));
    }
}
