//! RMCP+ session state and key derivation (RAKP). Mirrors the RAKP logic in
//! `src/plugins/lanplus/lanplus.c` and `lanplus_crypt.c`.

use ipmi_core::consts;

use crate::crypto;

/// Session negotiation states, mirroring `enum LANPLUS_SESSION_STATE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Presession,
    OpenSessionSent,
    OpenSessionReceived,
    Rakp1Sent,
    Rakp2Received,
    Rakp3Sent,
    Active,
    CloseSent,
}

/// IPMI v2 / RMCP+ specific session data.
#[derive(Debug, Clone)]
pub struct V2Data {
    pub state: SessionState,
    pub requested_auth_alg: u8,
    pub requested_integrity_alg: u8,
    pub requested_crypt_alg: u8,
    pub auth_alg: u8,
    pub integrity_alg: u8,
    pub crypt_alg: u8,
    pub max_priv_level: u8,
    pub console_id: u32,
    pub bmc_id: u32,
    pub console_rand: [u8; 16],
    pub bmc_rand: [u8; 16],
    pub bmc_guid: [u8; 16],
    pub requested_role: u8,
    pub rakp2_return_code: u8,
    pub sik: Vec<u8>,
    pub k1: Vec<u8>,
    pub k2: Vec<u8>,
}

impl Default for V2Data {
    fn default() -> Self {
        V2Data {
            state: SessionState::Presession,
            requested_auth_alg: consts::AUTH_RAKP_NONE,
            requested_integrity_alg: consts::INTEGRITY_NONE,
            requested_crypt_alg: consts::CRYPT_NONE,
            auth_alg: consts::AUTH_RAKP_NONE,
            integrity_alg: consts::INTEGRITY_NONE,
            crypt_alg: consts::CRYPT_NONE,
            max_priv_level: 0,
            console_id: 0,
            bmc_id: 0,
            console_rand: [0; 16],
            bmc_rand: [0; 16],
            bmc_guid: [0; 16],
            requested_role: 0,
            rakp2_return_code: 0,
            sik: Vec::new(),
            k1: Vec::new(),
            k2: Vec::new(),
        }
    }
}

/// A live RMCP+ session.
#[derive(Debug, Clone)]
pub struct Session {
    pub state: SessionState,
    pub timeout: u32,
    /// User authentication key (Kuid), i.e. the zero padded password.
    pub authcode: [u8; consts::IPMI_AUTHCODE_BUFFER_SIZE],
    pub out_seq: u32,
    pub v2: V2Data,
}

impl Session {
    pub fn new(timeout: u32, authcode: [u8; consts::IPMI_AUTHCODE_BUFFER_SIZE]) -> Self {
        Session {
            state: SessionState::Presession,
            timeout,
            authcode,
            out_seq: 1,
            v2: V2Data::default(),
        }
    }

    pub fn active(&self) -> bool {
        self.v2.state == SessionState::Active
    }

    pub fn authcode_len(&self) -> usize {
        match self.v2.integrity_alg {
            consts::INTEGRITY_HMAC_SHA1_96 => 12,
            consts::INTEGRITY_HMAC_MD5_128 | consts::INTEGRITY_MD5_128 => 16,
            consts::INTEGRITY_HMAC_SHA256_128 => 16,
            _ => 0,
        }
    }

    /// Verify the key exchange auth code the BMC sent in RAKP 2.
    pub fn rakp2_matches(&self, bmc_mac: &[u8], username: &[u8]) -> bool {
        if self.v2.auth_alg == consts::AUTH_RAKP_NONE {
            return true;
        }
        let mut buf = Vec::with_capacity(58 + username.len());
        buf.extend_from_slice(&self.v2.console_id.to_le_bytes()); // SIDm
        buf.extend_from_slice(&self.v2.bmc_id.to_le_bytes()); // SIDc
        buf.extend_from_slice(&self.v2.console_rand); // Rm
        buf.extend_from_slice(&self.v2.bmc_rand); // Rc
        buf.extend_from_slice(&self.v2.bmc_guid); // GUIDc
        buf.push(self.v2.requested_role); // ROLEm
        buf.push(username.len() as u8); // ULENGTHm
        buf.extend_from_slice(username);

        let mac = crypto::hmac_auth(self.v2.auth_alg, &self.authcode, &buf);
        bmc_mac.len() >= mac.len() && bmc_mac[..mac.len()] == mac[..]
    }

    /// Verify the integrity check value the BMC sent in RAKP 4.
    pub fn rakp4_matches(&self, bmc_mac: &[u8]) -> bool {
        if self.v2.auth_alg == consts::AUTH_RAKP_NONE {
            return true;
        }
        let mut buf = Vec::with_capacity(36);
        buf.extend_from_slice(&self.v2.console_rand); // Rm
        buf.extend_from_slice(&self.v2.bmc_id.to_le_bytes()); // SIDc
        buf.extend_from_slice(&self.v2.bmc_guid); // GUIDc

        let mac = crypto::hmac_auth(self.v2.auth_alg, &self.v2.sik, &buf);
        let cmp_len = match self.v2.auth_alg {
            consts::AUTH_RAKP_HMAC_MD5 => 16,
            _ => 12,
        };
        bmc_mac.len() >= cmp_len && mac.len() >= cmp_len && bmc_mac[..cmp_len] == mac[..cmp_len]
    }

    /// Generate the RAKP 3 key exchange auth code.
    pub fn generate_rakp3_authcode(&self, username: &[u8]) -> Vec<u8> {
        if self.v2.auth_alg == consts::AUTH_RAKP_NONE {
            return Vec::new();
        }
        let mut buf = Vec::with_capacity(22 + username.len());
        buf.extend_from_slice(&self.v2.bmc_rand); // Rc
        buf.extend_from_slice(&self.v2.console_id.to_le_bytes()); // SIDm
        buf.push(self.v2.requested_role); // ROLEm
        buf.push(username.len() as u8); // ULENGTHm
        buf.extend_from_slice(username);
        crypto::hmac_auth(self.v2.auth_alg, &self.authcode, &buf)
    }

    /// Derive the session integrity key (SIK).
    pub fn generate_sik(&mut self, kg: &[u8; consts::IPMI_KG_BUFFER_SIZE], username: &[u8]) {
        self.v2.sik.clear();
        if self.v2.auth_alg == consts::AUTH_RAKP_NONE {
            return;
        }
        let mut buf = Vec::with_capacity(34 + username.len());
        buf.extend_from_slice(&self.v2.console_rand); // Rm
        buf.extend_from_slice(&self.v2.bmc_rand); // Rc
        buf.push(self.v2.requested_role); // ROLEm
        buf.push(username.len() as u8); // ULENGTHm
        buf.extend_from_slice(username);

        let key: &[u8] = if kg[0] != 0 {
            &kg[..consts::IPMI_AUTHCODE_BUFFER_SIZE]
        } else {
            &self.authcode
        };
        self.v2.sik = crypto::hmac_auth(self.v2.auth_alg, key, &buf);
    }

    /// Derive K1 (message integrity key).
    pub fn generate_k1(&mut self) {
        if self.v2.auth_alg == consts::AUTH_RAKP_NONE {
            self.v2.k1 = vec![0x01; 20];
            return;
        }
        self.v2.k1 = crypto::hmac_auth(self.v2.auth_alg, &self.v2.sik.clone(), &[0x01; 20]);
    }

    /// Derive K2 (confidentiality key).
    pub fn generate_k2(&mut self) {
        if self.v2.auth_alg == consts::AUTH_RAKP_NONE {
            self.v2.k2 = vec![0x02; 20];
            return;
        }
        self.v2.k2 = crypto::hmac_auth(self.v2.auth_alg, &self.v2.sik.clone(), &[0x02; 20]);
    }

    /// Take the next outgoing sequence number (never zero).
    pub fn next_out_seq(&mut self) -> u32 {
        let s = self.out_seq;
        self.out_seq = self.out_seq.wrapping_add(1);
        if self.out_seq == 0 {
            self.out_seq = 1;
        }
        s
    }
}
