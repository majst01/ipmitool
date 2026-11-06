//! The lanplus transport: UDP socket handling, RMCP+ session establishment and
//! the [`Interface`] implementation consumed by the CLI.
//!
//! This mirrors `ipmi_lanplus_open` and friends from
//! `src/plugins/lanplus/lanplus.c`.  Bridging and Serial-over-LAN are not yet
//! implemented; the transport targets a locally attached BMC.

use std::net::{ToSocketAddrs, UdpSocket};
use std::time::Duration;

use ipmi_core::consts;
use ipmi_core::error::{Error, Result};
use ipmi_core::interface::Interface;
use ipmi_core::types::{CipherSuite, Request, Response, SessionParams};

use crate::message::{self, SessionHeader};
use crate::session::{Session, SessionState};

/// Default RMCP+ UDP port.
pub const IPMI_LANPLUS_PORT: u16 = 0x026f;
/// Default per-attempt timeout in seconds.
const DEFAULT_TIMEOUT: u32 = 1;
/// Default number of retries before giving up.
const DEFAULT_RETRY: u32 = 4;
/// Largest datagram we will read from the BMC.
const MAX_DATAGRAM: usize = 4096;
/// The console session id ipmitool deliberately chooses (easily spotted in
/// packet captures).
const CONSOLE_ID: u32 = 0xA0A2_A3A4;

/// Standard cipher-suite record start byte.
const STANDARD_CIPHER_SUITE: u8 = 0xc0;
/// OEM cipher-suite record start byte.
const OEM_CIPHER_SUITE: u8 = 0xc1;
/// The `Get Channel Cipher Suites` command.
const GET_CHANNEL_CIPHER_SUITES: u8 = 0x54;
/// The LAN channel used for IPMI over LAN.
const IPMI_LAN_CHANNEL_E: u8 = 0x0e;

/// A live lanplus (IPMI v2 / RMCP+) transport.
pub struct LanPlus {
    params: SessionParams,
    session: Option<Session>,
    socket: Option<UdpSocket>,
    opened: bool,
    /// Rolling 6-bit IPMI request sequence number.
    rq_seq: u8,
}

impl LanPlus {
    /// Create a new transport with the given session parameters (hostname,
    /// credentials, privilege level, ...).
    pub fn new(params: SessionParams) -> Self {
        LanPlus {
            params,
            session: None,
            socket: None,
            opened: false,
            rq_seq: 0,
        }
    }

    fn session_active(&self) -> bool {
        matches!(
            self.session.as_ref().map(|s| s.v2.state),
            Some(SessionState::Active)
        )
    }

    /// Next 6-bit request sequence number, wrapping at 64 like the C code.
    fn next_rq_seq(&mut self) -> u8 {
        self.rq_seq = self.rq_seq.wrapping_add(1);
        if self.rq_seq >= 64 {
            self.rq_seq = 0;
        }
        self.rq_seq
    }

    /// Open the UDP socket and remember it.
    fn open_socket(&mut self) -> Result<()> {
        let host = self
            .params
            .hostname
            .clone()
            .ok_or_else(|| Error::Invalid("no hostname specified".into()))?;
        let port = self.params.port;
        let timeout = Duration::from_secs(self.params.timeout.max(1) as u64);

        let mut last_err = None;
        let mut addrs = (host.as_str(), port)
            .to_socket_addrs()
            .map_err(|e| Error::Resolution(format!("{host}: {e}")))?;
        let addr = addrs
            .next()
            .ok_or_else(|| Error::Resolution(host.clone()))?;

        let socket = UdpSocket::bind(("0.0.0.0", 0)).map_err(Error::Io)?;
        socket.set_read_timeout(Some(timeout)).map_err(Error::Io)?;
        socket.set_write_timeout(Some(timeout)).map_err(Error::Io)?;
        // The first datagram sent to UDP/623 may trigger an ICMP port
        // unreachable which surfaces as ECONNREFUSED on a connected socket.
        // Keep the socket unconnected and use send/recv to avoid that, but
        // connect() gives us cleaner error semantics.  ipmitool tolerates the
        // error by simply retrying, so we connect.
        if let Err(e) = socket.connect(addr) {
            last_err = Some(e);
        }
        if let Some(e) = last_err {
            return Err(Error::Io(e));
        }
        self.socket = Some(socket);
        Ok(())
    }

    /// Send a raw datagram.
    fn send_raw(&self, data: &[u8]) -> Result<()> {
        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| Error::Protocol("socket not open".into()))?;
        socket.send(data).map_err(Error::Io)?;
        Ok(())
    }

    /// Receive and parse a single datagram, validating its authcode and
    /// decrypting the payload when necessary.
    fn recv_payload(&mut self) -> Result<(SessionHeader, Vec<u8>)> {
        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| Error::Protocol("socket not open".into()))?;
        let mut buf = vec![0u8; MAX_DATAGRAM];
        let n = match socket.recv(&mut buf) {
            Ok(n) => n,
            Err(e) => return Err(map_io(e)),
        };
        buf.truncate(n);

        let header = message::parse_session_header(&buf)
            .ok_or_else(|| Error::Protocol("malformed packet header".into()))?;

        if let Some(session) = &self.session {
            if !message::valid_authcode(&buf, &header, session) {
                return Err(Error::Session(
                    "received message with invalid authcode".into(),
                ));
            }
        }

        let start = header.offset;
        let end = start + header.msglen as usize;
        if end > buf.len() {
            return Err(Error::Protocol("truncated session payload".into()));
        }
        let mut payload = buf[start..end].to_vec();

        if header.encrypted && self.session_active() {
            let session = self.session.as_ref().unwrap();
            let alg = session.v2.crypt_alg;
            let k2 = session.v2.k2.clone();
            payload = message::decrypt_payload(alg, &k2, &payload).map_err(Error::Protocol)?;
        }

        Ok((header, payload))
    }

    /// Build, send and dispatch a single IPMI request, returning the matching
    /// response.  Handles both the pre-session v1.5 auth-capabilities probe and
    /// regular v2.x messages.
    fn send_ipmi(&mut self, req: &Request) -> Result<Response> {
        let active = self.session_active();
        // The auth-capabilities probe must go out in IPMI v1.5 format so we can
        // ask an arbitrary BMC whether it speaks RMCP+ at all.
        let v15 = req.netfn == consts::NETFN_APP
            && req.cmd == consts::IPMI_GET_CHANNEL_AUTH_CAP
            && !active;

        let retries = self.params.retry.max(1);

        for _ in 0..retries {
            let seq = if v15 { 0 } else { self.next_rq_seq() };
            let msg = {
                let session = self
                    .session
                    .as_mut()
                    .ok_or_else(|| Error::Protocol("no session".into()))?;
                if v15 {
                    message::build_v15(session, req)
                } else {
                    let payload = message::request_payload(req, self.params.my_addr as u8, seq);
                    message::build_v2x(session, consts::PAYLOAD_TYPE_IPMI, &payload)
                }
            };

            self.send_raw(&msg)?;

            loop {
                match self.recv_payload() {
                    Ok((header, payload)) => {
                        if header.payload_type != consts::PAYLOAD_TYPE_IPMI {
                            continue;
                        }
                        if let Some(resp) = message::parse_ipmi_response(&payload) {
                            // Ignore responses that belong to another request
                            // (the BMC may still be answering an earlier retry).
                            if resp.rq_seq == seq && resp.cmd == req.cmd {
                                return Ok(resp);
                            }
                        }
                    }
                    // A timeout means we should retry the whole request.
                    Err(Error::Timeout) => break,
                    Err(e) => return Err(e),
                }
            }
        }

        Err(Error::Timeout)
    }

    /// Issue `Get Channel Authentication Capabilities` and report whether the
    /// channel advertises IPMI v2.0 support.
    fn get_auth_capabilities(&mut self) -> Result<bool> {
        let data = [IPMI_LAN_CHANNEL_E | 0x80, self.params.privlvl];
        let mut req = Request::new(consts::NETFN_APP, consts::IPMI_GET_CHANNEL_AUTH_CAP);
        req.data = data.to_vec();

        match self.send_ipmi(&req) {
            Ok(rsp) if rsp.ok() && rsp.data.len() >= 2 => Ok(rsp.data[1] & 0x80 != 0),
            Ok(_rsp) => {
                // Retry without the "request IPMI v2 data" bit, as ipmitool does.
                req.data[0] &= 0x7f;
                let rsp = self.send_ipmi(&req)?;
                if !rsp.ok() {
                    return Err(Error::Completion {
                        code: rsp.ccode,
                        command: Some("Get Channel Authentication Capabilities".into()),
                    });
                }
                Ok(rsp.data.get(1).map(|b| b & 0x80 != 0).unwrap_or(false))
            }
            Err(Error::Timeout) => Err(Error::Timeout),
            Err(e) => Err(e),
        }
    }

    /// Query the channel cipher suite list and return the ids the BMC offers.
    fn channel_cipher_suites(&mut self) -> Result<Vec<u8>> {
        let mut ids = Vec::new();
        let mut index: u8 = 0;
        loop {
            let data = [
                IPMI_LAN_CHANNEL_E,
                0, // payload type: 0 = IPMI, non-zero = SOL
                0x80 | index,
            ];
            let mut req = Request::new(consts::NETFN_APP, GET_CHANNEL_CIPHER_SUITES);
            req.data = data.to_vec();
            let rsp = self.send_ipmi(&req)?;
            if !rsp.ok() || rsp.data.is_empty() {
                return Err(Error::Completion {
                    code: rsp.ccode,
                    command: Some("Get Channel Cipher Suites".into()),
                });
            }
            // First byte is the channel number, the rest is record data.
            let records = &rsp.data[1..];
            parse_cipher_suite_records(records, &mut ids);
            // We can only read 16 bytes at a time; keep going while full.
            if records.len() < 16 || index >= 0x3f {
                break;
            }
            index += 1;
        }
        Ok(ids)
    }

    /// Replicate `ipmi_find_best_cipher_suite`: prefer 17 (SHA-256/AES) then 3,
    /// falling back to 3 which IPMI v2 mandates.
    fn find_best_cipher_suite(&mut self) -> u8 {
        const PREFERRED: [u8; 2] = [17, 3];
        if let Ok(ids) = self.channel_cipher_suites() {
            for want in PREFERRED {
                if ids.contains(&want) {
                    return want;
                }
            }
        }
        3
    }

    /// Set the session privilege level once the session is active.
    fn set_session_privilege(&mut self) -> Result<()> {
        let privlvl = self.params.privlvl;
        if privlvl <= consts::SESSION_PRIV_USER {
            return Ok(());
        }
        let mut req = Request::new(consts::NETFN_APP, consts::IPMI_SET_SESSION_PRIVILEGE);
        req.data = vec![privlvl];
        let rsp = self.send_ipmi(&req)?;
        if !rsp.ok() {
            return Err(Error::Completion {
                code: rsp.ccode,
                command: Some("Set Session Privilege Level".into()),
            });
        }
        Ok(())
    }

    /// Perform the Open Session / RAKP 1 / RAKP 3 handshake.
    fn negotiate(&mut self) -> Result<()> {
        self.open_session()?;
        self.rakp1()?;
        self.rakp3()?;
        Ok(())
    }

    fn open_session(&mut self) -> Result<()> {
        let suite = self.requested_suite()?;
        let console_id = self
            .session
            .as_ref()
            .map(|s| s.v2.console_id)
            .unwrap_or(CONSOLE_ID);
        let privlvl = self.params.privlvl;
        let payload = message::open_session_request(privlvl, console_id, suite);

        {
            let session = self.session.as_mut().unwrap();
            session.v2.state = SessionState::OpenSessionSent;
        }

        let msg = {
            let session = self.session.as_mut().unwrap();
            message::build_v2x(session, consts::PAYLOAD_TYPE_RMCP_OPEN_REQUEST, &payload)
        };
        self.send_raw(&msg)?;

        let (header, body) = self.recv_payload()?;
        if header.payload_type != consts::PAYLOAD_TYPE_RMCP_OPEN_RESPONSE {
            return Err(Error::Session("unexpected response to Open Session".into()));
        }
        if body.len() < 36 {
            return Err(Error::Protocol("short Open Session response".into()));
        }
        let rc = body[1];
        if rc != consts::RAKP_STATUS_NO_ERRORS {
            return Err(Error::Session(format!(
                "Open Session rejected (0x{rc:02x})"
            )));
        }
        let returned_console = u32::from_le_bytes(body[4..8].try_into().unwrap());
        if returned_console != console_id {
            // Non-fatal, matching ipmitool's warning.
        }
        let max_priv = body[2];
        let bmc_id = u32::from_le_bytes(body[8..12].try_into().unwrap());
        let auth_alg = body[16];
        let integrity_alg = body[24];
        let crypt_alg = body[32];

        {
            let session = self.session.as_mut().unwrap();
            session.v2.max_priv_level = max_priv;
            session.v2.bmc_id = bmc_id;
            session.v2.auth_alg = auth_alg;
            session.v2.integrity_alg = integrity_alg;
            session.v2.crypt_alg = crypt_alg;
            session.v2.state = SessionState::OpenSessionReceived;
        }

        if auth_alg != suite.auth_alg
            || integrity_alg != suite.integrity_alg
            || crypt_alg != suite.crypt_alg
        {
            return Err(Error::Session(
                "BMC did not agree to the requested cipher suite algorithms".into(),
            ));
        }
        Ok(())
    }

    fn rakp1(&mut self) -> Result<()> {
        let username = self.params.username_bytes().to_vec();
        let (bmc_id, console_rand, role) = {
            let session = self.session.as_mut().unwrap();
            session.v2.console_rand = crate::crypto::random_bytes(16)
                .try_into()
                .map_err(|_| Error::Protocol("random number size".into()))?;
            let role = self.params.privlvl | self.params.lookupbit;
            session.v2.requested_role = role;
            (session.v2.bmc_id, session.v2.console_rand, role)
        };

        let payload = message::rakp1_message(bmc_id, &console_rand, role, &username);
        let msg = {
            let session = self.session.as_mut().unwrap();
            session.v2.state = SessionState::Rakp1Sent;
            message::build_v2x(session, consts::PAYLOAD_TYPE_RAKP_1, &payload)
        };
        self.send_raw(&msg)?;

        let (header, body) = self.recv_payload()?;
        if header.payload_type != consts::PAYLOAD_TYPE_RAKP_2 {
            return Err(Error::Session("unexpected response to RAKP 1".into()));
        }
        {
            let session = self.session.as_mut().unwrap();
            session.v2.state = SessionState::Rakp2Received;
        }
        if body.len() < 8 {
            return Err(Error::Protocol("short RAKP 2 message".into()));
        }
        let rc = body[1];
        if rc != consts::RAKP_STATUS_NO_ERRORS {
            return Err(Error::Session(format!("RAKP 2 rejected (0x{rc:02x})")));
        }
        if body.len() < 24 {
            return Err(Error::Protocol("short RAKP 2 message".into()));
        }
        let mut bmc_rand = [0u8; 16];
        bmc_rand.copy_from_slice(&body[8..24]);
        let mut bmc_guid = [0u8; 16];
        if body.len() >= 40 {
            bmc_guid.copy_from_slice(&body[24..40]);
        }

        let auth_alg = self.session.as_ref().unwrap().v2.auth_alg;
        let mac_len = auth_digest_len(auth_alg);
        let matches = {
            let session = self.session.as_mut().unwrap();
            session.v2.bmc_rand = bmc_rand;
            session.v2.bmc_guid = bmc_guid;
            let mac = body.get(40..40 + mac_len).unwrap_or(&[]);
            session.rakp2_matches(mac, &username)
        };

        let session = self.session.as_mut().unwrap();
        if matches {
            session.v2.rakp2_return_code = consts::RAKP_STATUS_NO_ERRORS;
            Ok(())
        } else {
            session.v2.rakp2_return_code = consts::RAKP_STATUS_INVALID_INTEGRITY_CHECK_VALUE;
            Err(Error::Session("RAKP 2 HMAC is invalid".into()))
        }
    }

    fn rakp3(&mut self) -> Result<()> {
        let username = self.params.username_bytes().to_vec();
        let (bmc_id, rc, authcode) = {
            let session = self.session.as_ref().unwrap();
            let authcode = if session.v2.rakp2_return_code == consts::RAKP_STATUS_NO_ERRORS {
                session.generate_rakp3_authcode(&username)
            } else {
                Vec::new()
            };
            (session.v2.bmc_id, session.v2.rakp2_return_code, authcode)
        };

        let payload = message::rakp3_message(bmc_id, rc, &authcode);

        // Derive keys only when the exchange succeeded.
        if rc == consts::RAKP_STATUS_NO_ERRORS {
            let kg = self.params.kg;
            let session = self.session.as_mut().unwrap();
            session.generate_sik(&kg, &username);
            session.generate_k1();
            session.generate_k2();
        }

        let msg = {
            let session = self.session.as_mut().unwrap();
            session.v2.state = SessionState::Rakp3Sent;
            message::build_v2x(session, consts::PAYLOAD_TYPE_RAKP_3, &payload)
        };
        self.send_raw(&msg)?;

        if rc != consts::RAKP_STATUS_NO_ERRORS {
            return Err(Error::Session("RAKP 2 exchange failed".into()));
        }

        let (header, body) = self.recv_payload()?;
        if header.payload_type != consts::PAYLOAD_TYPE_RAKP_4 {
            return Err(Error::Session("unexpected response to RAKP 3".into()));
        }
        if body.len() < 8 {
            return Err(Error::Protocol("short RAKP 4 message".into()));
        }
        if body[1] != consts::RAKP_STATUS_NO_ERRORS {
            return Err(Error::Session(format!(
                "RAKP 4 rejected (0x{:02x})",
                body[1]
            )));
        }

        let auth_alg = self.session.as_ref().unwrap().v2.auth_alg;
        let mac_len = match auth_alg {
            consts::AUTH_RAKP_HMAC_MD5 => 16,
            consts::AUTH_RAKP_NONE => 0,
            _ => 12,
        };
        let matches = {
            let session = self.session.as_ref().unwrap();
            let mac = body.get(8..8 + mac_len).unwrap_or(&[]);
            session.rakp4_matches(mac)
        };
        if !matches {
            return Err(Error::Session(
                "RAKP 4 integrity check value is invalid".into(),
            ));
        }

        self.session.as_mut().unwrap().v2.state = SessionState::Active;
        Ok(())
    }

    fn requested_suite(&self) -> Result<CipherSuite> {
        let id = self
            .params
            .cipher_suite
            .ok_or_else(|| Error::Protocol("cipher suite not negotiated".into()))?;
        CipherSuite::from_id(id).ok_or_else(|| Error::Unsupported(format!("cipher suite {id}")))
    }

    /// Send the `Close Session` command if we have an active session.
    fn close_session(&mut self) {
        if !self.session_active() {
            return;
        }
        let bmc_id = match self.session.as_ref() {
            Some(s) => s.v2.bmc_id,
            None => return,
        };
        let mut req = Request::new(consts::NETFN_APP, consts::IPMI_CLOSE_SESSION);
        req.data = bmc_id.to_le_bytes().to_vec();
        self.session.as_mut().unwrap().v2.state = SessionState::CloseSent;
        let _ = self.send_ipmi(&req);
    }
}

fn map_io(e: std::io::Error) -> Error {
    use std::io::ErrorKind::*;
    match e.kind() {
        WouldBlock | TimedOut | Interrupted => Error::Timeout,
        _ => Error::Io(e),
    }
}

/// Number of bytes in the RAKP key-exchange authcode for an auth algorithm.
fn auth_digest_len(alg: u8) -> usize {
    match alg {
        consts::AUTH_RAKP_HMAC_SHA1 => 20,
        consts::AUTH_RAKP_HMAC_MD5 => 16,
        consts::AUTH_RAKP_HMAC_SHA256 => 32,
        _ => 0,
    }
}

/// Parse the fixed-width cipher-suite records returned by the BMC.
fn parse_cipher_suite_records(records: &[u8], ids: &mut Vec<u8>) {
    let mut offset = 0;
    while offset < records.len() {
        match records[offset] {
            STANDARD_CIPHER_SUITE => {
                if offset + 5 > records.len() {
                    break;
                }
                ids.push(records[offset + 1]);
                offset += 5;
            }
            OEM_CIPHER_SUITE => {
                if offset + 8 > records.len() {
                    break;
                }
                ids.push(records[offset + 1]);
                offset += 8;
            }
            _ => break,
        }
    }
}

impl Interface for LanPlus {
    fn name(&self) -> &str {
        "lanplus"
    }

    fn open(&mut self) -> Result<()> {
        if self.opened {
            return Ok(());
        }

        if self.params.port == 0 {
            self.params.port = IPMI_LANPLUS_PORT;
        }
        if self.params.privlvl == 0 {
            self.params.privlvl = consts::SESSION_PRIV_ADMIN;
        }
        if self.params.timeout == 0 {
            self.params.timeout = DEFAULT_TIMEOUT;
        }
        if self.params.retry == 0 {
            self.params.retry = DEFAULT_RETRY;
        }

        self.open_socket()?;

        let authcode = *self.params.password_bytes();
        let mut session = Session::new(self.params.timeout, authcode);
        session.v2.console_id = CONSOLE_ID;
        self.session = Some(session);

        if !self.get_auth_capabilities()? {
            return Err(Error::Unsupported(
                "BMC does not support IPMI v2 / RMCP+".into(),
            ));
        }

        if self.params.cipher_suite.is_none() {
            let best = self.find_best_cipher_suite();
            self.params.cipher_suite = Some(best);
        }
        let suite = self.requested_suite()?;
        {
            let session = self.session.as_mut().unwrap();
            session.v2.requested_auth_alg = suite.auth_alg;
            session.v2.requested_integrity_alg = suite.integrity_alg;
            session.v2.requested_crypt_alg = suite.crypt_alg;
        }

        let retries = self.params.retry.max(1);
        let mut last_err = Error::Timeout;
        for _ in 0..retries {
            match self.negotiate() {
                Ok(()) => {
                    last_err = Error::Timeout; // sentinel: no error
                    break;
                }
                Err(Error::Timeout) => {
                    // Reset state and retry the whole handshake.
                    self.session.as_mut().unwrap().v2.state = SessionState::Presession;
                    last_err = Error::Timeout;
                }
                Err(e) => return Err(e),
            }
        }
        if !self.session_active() {
            return Err(last_err);
        }

        self.set_session_privilege()?;
        self.opened = true;
        Ok(())
    }

    fn close(&mut self) {
        if self.opened {
            self.close_session();
        }
        self.socket = None;
        self.session = None;
        self.opened = false;
    }

    fn is_open(&self) -> bool {
        self.opened
    }

    fn sendrecv(&mut self, req: &Request) -> Result<Response> {
        if !self.opened {
            self.open()?;
        }
        self.send_ipmi(req)
    }

    fn session_params(&self) -> &SessionParams {
        &self.params
    }

    fn session_params_mut(&mut self) -> &mut SessionParams {
        &mut self.params
    }
}
