//! RMCP+ wire format: message construction, confidentiality, integrity and
//! parsing. This mirrors `ipmi_lanplus_build_v2x_msg`, `getIpmiPayloadWireRep`
//! and the `read_*` functions from the C implementation.

use ipmi_core::checksum::ipmi_csum;
use ipmi_core::consts;
use ipmi_core::types::{Request, Response};

use crate::crypto;
use crate::session::{Session, SessionState};

// Session header offsets (table 13-8).
const OFF_AUTHTYPE: usize = 0x04;
const OFF_PAYLOAD_TYPE: usize = 0x05;
const OFF_SESSION_ID: usize = 0x06;
const OFF_PAYLOAD_SIZE: usize = 0x0e;
const OFF_PAYLOAD: usize = 0x10;

pub const AES_BLOCK_SIZE: usize = 0x10;

/// Parsed RMCP+ session header.
#[derive(Debug, Clone, Copy)]
pub struct SessionHeader {
    pub authtype: u8,
    pub payload_type: u8,
    pub encrypted: bool,
    pub authenticated: bool,
    pub session_id: u32,
    pub msglen: u16,
    /// Offset of the payload within the datagram.
    pub offset: usize,
}

/// Build the IPMI message wire representation (non-bridged).
pub fn request_payload(req: &Request, my_addr: u8, rq_seq: u8) -> Vec<u8> {
    let my_addr = if my_addr == 0 {
        consts::BMC_SLAVE_ADDR
    } else {
        my_addr
    };
    let mut m = Vec::with_capacity(req.data.len() + 7);
    // Only local requests (target == my address) are supported at present.
    let _ = my_addr;
    m.push(consts::BMC_SLAVE_ADDR); // rsAddr
    m.push((req.netfn << 2) | (req.lun & 3));
    let cs = m.len() - 1;
    let c = ipmi_csum(&m[..2]);
    let _ = cs;
    m.push(c);
    let cs = m.len();
    m.push(consts::REMOTE_SWID); // rqAddr
    m.push(rq_seq << 2);
    m.push(req.cmd);
    m.extend_from_slice(&req.data);
    let c = ipmi_csum(&m[cs..]);
    m.push(c);
    m
}

/// Encrypt a payload with the confidentiality algorithm, adding the
/// confidentiality header (IV) and trailer.
pub fn encrypt_payload(crypt_alg: u8, k2: &[u8], input: &[u8]) -> Vec<u8> {
    if crypt_alg == consts::CRYPT_NONE {
        return input.to_vec();
    }
    debug_assert_eq!(crypt_alg, consts::CRYPT_AES_CBC_128);
    let mod_ = (input.len() + 1) % AES_BLOCK_SIZE;
    let pad = if mod_ == 0 { 0 } else { AES_BLOCK_SIZE - mod_ };
    let mut padded = input.to_vec();
    for i in 0..pad {
        padded.push((i + 1) as u8);
    }
    padded.push(pad as u8);

    let iv = crypto::random_bytes(AES_BLOCK_SIZE);
    let ct = crypto::aes_cbc_encrypt(&k2[..16], &iv, &padded);
    let mut out = iv;
    out.extend_from_slice(&ct);
    out
}

/// Decrypt a payload, returning the plaintext with the confidentiality
/// padding removed. Returns `Err` on malformed padding.
pub fn decrypt_payload(crypt_alg: u8, k2: &[u8], input: &[u8]) -> Result<Vec<u8>, String> {
    if crypt_alg == consts::CRYPT_NONE {
        return Ok(input.to_vec());
    }
    if input.len() < AES_BLOCK_SIZE * 2 {
        return Err("encrypted payload too short".into());
    }
    let iv = &input[..AES_BLOCK_SIZE];
    let ct = &input[AES_BLOCK_SIZE..];
    let dec = crypto::aes_cbc_decrypt(&k2[..16], iv, ct);
    let conf_pad = dec[dec.len() - 1] as usize;
    if conf_pad + 1 > dec.len() {
        return Err("malformed confidentiality padding".into());
    }
    let payload_size = dec.len() - conf_pad - 1;
    for i in 0..conf_pad {
        if dec[payload_size + i] != (i + 1) as u8 {
            return Err("malformed payload padding".into());
        }
    }
    Ok(dec[..payload_size].to_vec())
}

/// Build a complete IPMI v2 / RMCP+ message. Updates `session.out_seq`.
pub fn build_v2x(session: &mut Session, payload_type: u8, payload: &[u8]) -> Vec<u8> {
    let active = session.v2.state == SessionState::Active;

    let mut m = Vec::with_capacity(payload.len() + 64);
    // RMCP header
    m.extend_from_slice(&[0x06, 0x00, 0xff, 0x07]);

    // IPMI session header
    m.push(consts::SESSION_AUTHTYPE_RMCP_PLUS);
    let mut pt = payload_type;
    if active {
        if session.v2.crypt_alg != consts::CRYPT_NONE {
            pt |= 0x80;
        }
        if session.v2.integrity_alg != consts::INTEGRITY_NONE {
            pt |= 0x40;
        }
    }
    m.push(pt);
    if active {
        m.extend_from_slice(&session.v2.bmc_id.to_le_bytes());
        m.extend_from_slice(&session.out_seq.to_le_bytes());
    } else {
        m.extend_from_slice(&[0; 4]);
        m.extend_from_slice(&[0; 4]);
    }
    m.extend_from_slice(&[0, 0]); // payload size placeholder

    // Payload (possibly encrypted)
    let body = if active {
        encrypt_payload(session.v2.crypt_alg, &session.v2.k2.clone(), payload)
    } else {
        payload.to_vec()
    };
    let plen = body.len() as u16;
    m[OFF_PAYLOAD_SIZE..OFF_PAYLOAD_SIZE + 2].copy_from_slice(&plen.to_le_bytes());
    m.extend_from_slice(&body);

    // Session trailer: integrity pad, pad length, next header, authcode.
    if active && session.v2.integrity_alg != consts::INTEGRITY_NONE {
        let length_before_authcode = OFF_PAYLOAD + body.len() + 2;
        let pad = if length_before_authcode % 4 != 0 {
            4 - (length_before_authcode % 4)
        } else {
            0
        };
        m.extend(std::iter::repeat(0xff).take(pad));
        m.push(pad as u8);
        m.push(0x07);
        // The integrity HMAC covers the session header from the auth-type byte
        // through the end of the trailer, i.e. everything built so far after
        // `OFF_AUTHTYPE`.
        let auth_len = session.authcode_len();
        let mac = crypto::hmac_integrity(
            session.v2.integrity_alg,
            &session.v2.k1.clone(),
            &m[OFF_AUTHTYPE..],
        );
        m.extend_from_slice(&mac[..auth_len]);
    }

    session.next_out_seq();
    m
}

/// Build a pre-session IPMI v1.5 message (used for the Get Channel
/// Authentication Capabilities command).
pub fn build_v15(session: &mut Session, req: &Request) -> Vec<u8> {
    let mut m = Vec::new();
    m.extend_from_slice(&[0x06, 0x00, 0xff, 0x07]); // RMCP
    m.push(consts::SESSION_AUTHTYPE_NONE);
    m.extend_from_slice(&session.out_seq.to_le_bytes());
    m.extend_from_slice(&[0; 4]); // session id
    m.push((req.data.len() + 7) as u8); // message length

    let cs = m.len();
    m.push(consts::BMC_SLAVE_ADDR);
    m.push(req.netfn << 2);
    let c = ipmi_csum(&m[cs..]);
    m.push(c);
    let cs = m.len();
    m.push(consts::REMOTE_SWID);
    m.push(0); // rq seq / lun
    m.push(req.cmd);
    m.extend_from_slice(&req.data);
    let c = ipmi_csum(&m[cs..]);
    m.push(c);

    // NB: ipmitool does not advance the session sequence number for v1.5
    // pre-session messages, so neither do we.
    m
}

/// Parse the RMCP+ session header from a received datagram.
pub fn parse_session_header(data: &[u8]) -> Option<SessionHeader> {
    if data.len() < OFF_AUTHTYPE + 1 {
        return None;
    }
    let authtype = data[OFF_AUTHTYPE];
    if authtype == consts::SESSION_AUTHTYPE_RMCP_PLUS {
        if data.len() < OFF_PAYLOAD {
            return None;
        }
        Some(SessionHeader {
            authtype,
            payload_type: data[OFF_PAYLOAD_TYPE] & 0x3f,
            encrypted: data[OFF_PAYLOAD_TYPE] & 0x80 != 0,
            authenticated: data[OFF_PAYLOAD_TYPE] & 0x40 != 0,
            session_id: u32::from_le_bytes(
                data[OFF_SESSION_ID..OFF_SESSION_ID + 4].try_into().ok()?,
            ),
            msglen: u16::from_le_bytes(
                data[OFF_PAYLOAD_SIZE..OFF_PAYLOAD_SIZE + 2]
                    .try_into()
                    .ok()?,
            ),
            offset: OFF_PAYLOAD,
        })
    } else {
        // IPMI v1.5 style session header (13 bytes after RMCP, 1-byte length).
        if data.len() < 15 {
            return None;
        }
        Some(SessionHeader {
            authtype,
            payload_type: consts::PAYLOAD_TYPE_IPMI,
            encrypted: false,
            authenticated: false,
            session_id: u32::from_le_bytes(data[6..10].try_into().ok()?),
            msglen: data[13] as u16,
            offset: 14,
        })
    }
}

/// Validate the authcode of a received session message.
pub fn valid_authcode(data: &[u8], header: &SessionHeader, session: &Session) -> bool {
    if session.v2.state != SessionState::Active
        || header.authtype != consts::SESSION_AUTHTYPE_RMCP_PLUS
        || !header.authenticated
        || session.v2.integrity_alg == consts::INTEGRITY_NONE
    {
        return true;
    }
    let auth_len = session.authcode_len();
    if data.len() < OFF_AUTHTYPE + auth_len {
        return false;
    }
    let bmc = &data[data.len() - auth_len..];
    let mac = crypto::hmac_integrity(
        session.v2.integrity_alg,
        &session.v2.k1.clone(),
        &data[OFF_AUTHTYPE..data.len() - auth_len],
    );
    mac.len() >= auth_len && bmc == &mac[..auth_len]
}

/// Parse an IPMI response payload (already decrypted) into a [`Response`].
pub fn parse_ipmi_response(payload: &[u8]) -> Option<Response> {
    if payload.len() < 7 {
        return None;
    }
    let netfn = payload[1] >> 2;
    let rq_seq = payload[4] >> 2;
    let cmd = payload[5];
    let ccode = payload[6];
    let data_end = payload.len().saturating_sub(1); // trailing checksum
    let data = if data_end > 7 {
        payload[7..data_end].to_vec()
    } else {
        Vec::new()
    };
    Some(Response {
        ccode,
        data,
        netfn,
        cmd,
        rq_seq,
    })
}

/// Build the 32-byte Open Session Request payload.
pub fn open_session_request(
    privlvl: u8,
    console_id: u32,
    suite: ipmi_core::CipherSuite,
) -> Vec<u8> {
    let mut m = vec![0u8; 32];
    m[0] = 0; // message tag
    m[1] = if privlvl != consts::SESSION_PRIV_ADMIN {
        privlvl
    } else {
        0
    };
    m[4..8].copy_from_slice(&console_id.to_le_bytes());
    // Authentication payload
    m[8] = 0;
    m[11] = 8;
    m[12] = suite.auth_alg;
    // Integrity payload
    m[16] = 1;
    m[19] = 8;
    m[20] = suite.integrity_alg;
    // Confidentiality payload
    m[24] = 2;
    m[27] = 8;
    m[28] = suite.crypt_alg;
    m
}

/// Build the 44-byte RAKP 1 message (truncated for short usernames).
pub fn rakp1_message(bmc_id: u32, console_rand: &[u8; 16], role: u8, username: &[u8]) -> Vec<u8> {
    let mut m = vec![0u8; 44];
    m[0] = 0; // message tag
    m[4..8].copy_from_slice(&bmc_id.to_le_bytes());
    m[8..24].copy_from_slice(console_rand);
    m[24] = role;
    m[27] = username.len().min(16) as u8;
    m[28..28 + username.len().min(16)].copy_from_slice(&username[..username.len().min(16)]);
    let len = 44 - (16 - username.len().min(16));
    m.truncate(len);
    m
}

/// Build the RAKP 3 message (8-byte header plus optional auth code).
pub fn rakp3_message(bmc_id: u32, rakp2_rc: u8, authcode: &[u8]) -> Vec<u8> {
    let mut m = Vec::with_capacity(8 + authcode.len());
    m.push(0); // message tag
    m.push(rakp2_rc);
    m.extend_from_slice(&[0, 0]);
    m.extend_from_slice(&bmc_id.to_le_bytes());
    m.extend_from_slice(authcode);
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use ipmi_core::checksum::ipmi_csum;

    #[test]
    fn request_payload_checksums_are_valid() {
        let req = Request::new(0x06, 0x01).with_data([0x01u8, 0x02]);
        let payload = request_payload(&req, 0x20, 3);
        // First checksum covers rsAddr + netfn byte (and the checksum itself).
        assert_eq!(ipmi_csum(&payload[..3]), 0);
        // Second checksum covers rqAddr..data plus the trailing checksum.
        assert_eq!(ipmi_csum(&payload[3..]), 0);
        // netfn/cmd are placed correctly.
        assert_eq!(payload[1], 0x06 << 2);
        assert_eq!(payload[5], 0x01);
        assert_eq!(payload[3], consts::REMOTE_SWID);
        assert_eq!(payload[4], 3 << 2);
    }

    #[test]
    fn parse_ipmi_response_extracts_fields() {
        // rsAddr, netfn, csum, rqAddr, seq, cmd, ccode, data, csum
        let payload = vec![
            0x20,
            0x06 << 2,
            0x00,
            0x81,
            0x02 << 2,
            0x01,
            0x00,
            0xaa,
            0xbb,
            0x00,
        ];
        let rsp = parse_ipmi_response(&payload).unwrap();
        assert_eq!(rsp.netfn, 0x06);
        assert_eq!(rsp.rq_seq, 2);
        assert_eq!(rsp.cmd, 0x01);
        assert_eq!(rsp.ccode, 0x00);
        assert_eq!(rsp.data, vec![0xaa, 0xbb]);
    }

    #[test]
    fn confidentiality_round_trip() {
        let k2 = [0x33u8; 20];
        let plain = b"hello ipmi world".to_vec();
        let enc = encrypt_payload(consts::CRYPT_AES_CBC_128, &k2, &plain);
        let dec = decrypt_payload(consts::CRYPT_AES_CBC_128, &k2, &enc).unwrap();
        assert_eq!(dec, plain);
    }

    #[test]
    fn confidentiality_none_is_passthrough() {
        let data = b"unencrypted".to_vec();
        let enc = encrypt_payload(consts::CRYPT_NONE, &[], &data);
        assert_eq!(enc, data);
    }

    #[test]
    fn open_session_request_layout() {
        let suite = ipmi_core::CipherSuite::from_id(3).unwrap();
        let m = open_session_request(consts::SESSION_PRIV_ADMIN, 0xA0A2A3A4, suite);
        assert_eq!(m.len(), 32);
        assert_eq!(&m[4..8], &0xA0A2A3A4u32.to_le_bytes());
        assert_eq!(m[12], suite.auth_alg);
        assert_eq!(m[20], suite.integrity_alg);
        assert_eq!(m[28], suite.crypt_alg);
    }

    #[test]
    fn rakp1_message_truncates_username() {
        let msg = rakp1_message(0x01020304, &[0x11; 16], 0x14, b"admin");
        assert_eq!(msg.len(), 28 + 5);
        assert_eq!(msg[27], 5);
        assert_eq!(&msg[28..33], b"admin");
    }

    #[test]
    fn build_v2x_advances_sequence_and_marks_payload() {
        let mut session = Session::new(1, [0u8; 20]);
        session.v2.state = SessionState::Presession;
        let m = build_v2x(
            &mut session,
            consts::PAYLOAD_TYPE_RMCP_OPEN_REQUEST,
            &[1, 2, 3, 4],
        );
        assert_eq!(&m[..4], &[0x06, 0x00, 0xff, 0x07]);
        assert_eq!(m[OFF_AUTHTYPE], consts::SESSION_AUTHTYPE_RMCP_PLUS);
        assert_eq!(
            m[OFF_PAYLOAD_TYPE] & 0x3f,
            consts::PAYLOAD_TYPE_RMCP_OPEN_REQUEST
        );
        let plen = u16::from_le_bytes(
            m[OFF_PAYLOAD_SIZE..OFF_PAYLOAD_SIZE + 2]
                .try_into()
                .unwrap(),
        );
        assert_eq!(plen as usize, 4);
    }

    #[test]
    fn build_v2x_active_integrity_covers_header_and_trailer() {
        // Regression: the integrity HMAC must cover everything from the
        // auth-type byte to the end of the trailer, and the slice must not run
        // past the buffer. This previously used OFF_PAYLOAD as the length
        // instead of OFF_PAYLOAD - OFF_AUTHTYPE and panicked.
        let mut session = Session::new(1, [0u8; 20]);
        session.v2.state = SessionState::Active;
        session.v2.integrity_alg = consts::INTEGRITY_HMAC_SHA1_96;
        session.v2.crypt_alg = consts::CRYPT_NONE;
        session.v2.k1 = vec![0x11; 20];

        let m = build_v2x(
            &mut session,
            consts::PAYLOAD_TYPE_IPMI,
            &[1, 2, 3, 4, 5, 6, 7, 8],
        );

        let auth_len = 12; // HMAC-SHA1-96
        let trailer_end = m.len() - auth_len;
        // Next-header byte is the last byte before the authcode.
        assert_eq!(m[trailer_end - 1], 0x07);
        let expected = crypto::hmac_integrity(
            consts::INTEGRITY_HMAC_SHA1_96,
            &session.v2.k1.clone(),
            &m[OFF_AUTHTYPE..trailer_end],
        );
        assert_eq!(&m[trailer_end..], &expected[..auth_len]);
    }
}
