//! Cryptographic primitives used by RMCP+: HMAC dispatch, AES-128-CBC with
//! the IPMI confidentiality padding, and random number generation.

use aes::Aes128;
use cbc::cipher::{
    block_padding::NoPadding, generic_array::GenericArray, BlockDecryptMut, BlockEncryptMut,
    KeyIvInit,
};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha1::Sha1;
use sha2::{Digest as _, Sha256};

use ipmi_core::consts;

type HmacSha1 = Hmac<Sha1>;
type HmacMd5 = Hmac<md5::Md5>;
type HmacSha256 = Hmac<Sha256>;

type Aes128CbcEnc = cbc::Encryptor<Aes128>;
type Aes128CbcDec = cbc::Decryptor<Aes128>;

/// Generate `n` cryptographically random bytes.
pub fn random_bytes(n: usize) -> Vec<u8> {
    let mut buf = vec![0u8; n];
    rand::rngs::OsRng.fill_bytes(&mut buf);
    buf
}

/// HMAC using the authentication algorithm negotiated during RAKP.
pub fn hmac_auth(alg: u8, key: &[u8], data: &[u8]) -> Vec<u8> {
    match alg {
        consts::AUTH_RAKP_HMAC_SHA1 => {
            let mut m = HmacSha1::new_from_slice(key).expect("hmac accepts any key length");
            m.update(data);
            m.finalize().into_bytes().to_vec()
        }
        consts::AUTH_RAKP_HMAC_MD5 => {
            let mut m = HmacMd5::new_from_slice(key).expect("hmac accepts any key length");
            m.update(data);
            m.finalize().into_bytes().to_vec()
        }
        consts::AUTH_RAKP_HMAC_SHA256 => {
            let mut m = HmacSha256::new_from_slice(key).expect("hmac accepts any key length");
            m.update(data);
            m.finalize().into_bytes().to_vec()
        }
        _ => Vec::new(),
    }
}

/// HMAC using the integrity algorithm negotiated in the open-session exchange.
pub fn hmac_integrity(alg: u8, key: &[u8], data: &[u8]) -> Vec<u8> {
    match alg {
        consts::INTEGRITY_HMAC_SHA1_96 => {
            let mut m = HmacSha1::new_from_slice(key).expect("hmac accepts any key length");
            m.update(data);
            m.finalize().into_bytes().to_vec()
        }
        consts::INTEGRITY_HMAC_MD5_128 => {
            let mut m = HmacMd5::new_from_slice(key).expect("hmac accepts any key length");
            m.update(data);
            m.finalize().into_bytes().to_vec()
        }
        consts::INTEGRITY_MD5_128 => {
            // Plain MD5 over the packet, no key (IPMI spec table 13-18).
            let _ = key;
            md5::Md5::digest(data).to_vec()
        }
        consts::INTEGRITY_HMAC_SHA256_128 => {
            let mut m = HmacSha256::new_from_slice(key).expect("hmac accepts any key length");
            m.update(data);
            m.finalize().into_bytes().to_vec()
        }
        _ => Vec::new(),
    }
}

/// AES-128-CBC encryption with no additional padding. `data` must be a
/// multiple of 16 bytes; `key`/`iv` must be 16 bytes.
pub fn aes_cbc_encrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Vec<u8> {
    debug_assert_eq!(data.len() % 16, 0);
    let mut buf = data.to_vec();
    let len = buf.len();
    let enc = Aes128CbcEnc::new(
        GenericArray::from_slice(&key[..16]),
        GenericArray::from_slice(&iv[..16]),
    );
    enc.encrypt_padded_mut::<NoPadding>(&mut buf, len)
        .expect("input is block aligned")
        .to_vec()
}

/// AES-128-CBC decryption with no additional padding.
pub fn aes_cbc_decrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Vec<u8> {
    debug_assert_eq!(data.len() % 16, 0);
    let mut buf = data.to_vec();
    let dec = Aes128CbcDec::new(
        GenericArray::from_slice(&key[..16]),
        GenericArray::from_slice(&iv[..16]),
    );
    dec.decrypt_padded_mut::<NoPadding>(&mut buf)
        .expect("input is block aligned")
        .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_cbc_round_trip() {
        let key = [0x11u8; 16];
        let iv = [0x22u8; 16];
        let plain: Vec<u8> = (0..32).collect();
        let ct = aes_cbc_encrypt(&key, &iv, &plain);
        assert_ne!(ct, plain);
        assert_eq!(aes_cbc_decrypt(&key, &iv, &ct), plain);
    }

    #[test]
    fn hmac_sha1_has_expected_length() {
        let mac = hmac_integrity(consts::INTEGRITY_HMAC_SHA1_96, b"key", b"data");
        assert_eq!(mac.len(), 20); // full digest; caller truncates to 12
    }

    #[test]
    fn hmac_sha256_has_expected_length() {
        let mac = hmac_integrity(consts::INTEGRITY_HMAC_SHA256_128, b"key", b"data");
        assert_eq!(mac.len(), 32);
    }
}
