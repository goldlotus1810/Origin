//! # codec — ISLCodec
//!
//! Encode/decode ISLMessage → bytes.
//! AES-256-GCM ready: key field có nhưng encryption optional
//! (no_std — crypto crate cần feature flag riêng).
//!
//! Hiện tại: encode/decode không encrypt (plaintext, dùng cho local IPC).
//! Khi cần encrypt: thêm aes-gcm feature.

extern crate alloc;
use crate::message::{ISLFrame, ISLMessage};
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// ISLError
// ─────────────────────────────────────────────────────────────────────────────

/// Lỗi codec.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ISLError {
    TooShort,
    InvalidMsgType,
    InvalidChecksum,
    BodyTooLong,
    /// Khi dùng encryption: authentication failure
    AuthFailed,
}

// ─────────────────────────────────────────────────────────────────────────────
// ISLCodec
// ─────────────────────────────────────────────────────────────────────────────

/// Codec cho ISLMessage.
///
/// Plaintext mode mặc định.
/// Key field dành cho AES-256-GCM khi enable feature `encrypt`.
#[allow(missing_docs)]
pub struct ISLCodec {
    /// AES-256 key (32 bytes). Dùng khi feature `encrypt` enabled.
    #[cfg_attr(not(feature = "encrypt"), allow(dead_code))]
    key: [u8; 32],
    /// Dùng checksum XOR đơn giản để verify integrity (plaintext mode).
    pub use_checksum: bool,
    /// Nonce counter cho AES-GCM (tăng mỗi lần encrypt).
    #[cfg(feature = "encrypt")]
    nonce_counter: u64,
}

impl ISLCodec {
    /// Tạo codec plaintext.
    pub fn new() -> Self {
        Self {
            key: [0u8; 32],
            use_checksum: true,
            #[cfg(feature = "encrypt")]
            nonce_counter: 0,
        }
    }

    /// Tạo với key cho AES-256-GCM.
    pub fn with_key(key: [u8; 32]) -> Self {
        Self {
            key,
            use_checksum: true,
            #[cfg(feature = "encrypt")]
            nonce_counter: 0,
        }
    }

    /// Encode ISLMessage → bytes (12B + optional 1B checksum).
    pub fn encode(&self, msg: &ISLMessage) -> Vec<u8> {
        let mut out = Vec::with_capacity(13);
        out.extend_from_slice(&msg.to_bytes());
        if self.use_checksum {
            out.push(checksum(&msg.to_bytes()));
        }
        out
    }

    /// Decode bytes → ISLMessage.
    pub fn decode(&self, bytes: &[u8]) -> Result<ISLMessage, ISLError> {
        let min_len = if self.use_checksum { 13 } else { 12 };
        if bytes.len() < min_len {
            return Err(ISLError::TooShort);
        }

        if self.use_checksum {
            let expected = checksum(&bytes[..12]);
            if bytes[12] != expected {
                return Err(ISLError::InvalidChecksum);
            }
        }

        ISLMessage::from_bytes(&bytes[..12]).ok_or(ISLError::InvalidMsgType)
    }

    /// Encode ISLFrame → bytes.
    pub fn encode_frame(&self, frame: &ISLFrame) -> Result<Vec<u8>, ISLError> {
        if frame.body.len() > 65535 {
            return Err(ISLError::BodyTooLong);
        }
        let mut out = self.encode(&frame.header);
        // Body length thay thế checksum byte cuối nếu có checksum
        // Để đơn giản: dùng frame.to_bytes() + checksum riêng
        let raw = frame.to_bytes();
        // Ghi đè bằng full frame bytes
        out.clear();
        out.extend_from_slice(&raw);
        if self.use_checksum {
            let cs = checksum(&raw);
            out.push(cs);
        }
        Ok(out)
    }

    /// Decode bytes → ISLFrame.
    pub fn decode_frame(&self, bytes: &[u8]) -> Result<ISLFrame, ISLError> {
        let min_len = 14 + if self.use_checksum { 1 } else { 0 };
        if bytes.len() < min_len {
            return Err(ISLError::TooShort);
        }

        let raw_end = if self.use_checksum {
            bytes.len() - 1
        } else {
            bytes.len()
        };

        if self.use_checksum {
            let expected = checksum(&bytes[..raw_end]);
            if bytes[raw_end] != expected {
                return Err(ISLError::InvalidChecksum);
            }
        }

        ISLFrame::from_bytes(&bytes[..raw_end]).ok_or(ISLError::InvalidMsgType)
    }
}

impl Default for ISLCodec {
    fn default() -> Self {
        Self::new()
    }
}

/// XOR checksum — nhẹ, không cần crypto.
fn checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc ^ b)
}

// ─────────────────────────────────────────────────────────────────────────────
// AES-256-GCM Encryption (feature = "encrypt")
// ─────────────────────────────────────────────────────────────────────────────

/// Nonce size cho AES-256-GCM: 12 bytes.
#[cfg(feature = "encrypt")]
pub const NONCE_SIZE: usize = 12;

/// Tag size cho AES-256-GCM: 16 bytes.
#[cfg(feature = "encrypt")]
pub const TAG_SIZE: usize = 16;

#[cfg(feature = "encrypt")]
impl ISLCodec {
    /// Tạo nonce 12 bytes từ counter.
    /// Bytes 0..8 = counter (little-endian), bytes 8..12 = 0.
    fn next_nonce(&mut self) -> [u8; NONCE_SIZE] {
        let mut nonce = [0u8; NONCE_SIZE];
        nonce[..8].copy_from_slice(&self.nonce_counter.to_le_bytes());
        self.nonce_counter += 1;
        nonce
    }

    /// Encrypt plaintext → `[nonce:12B][ciphertext+tag]`.
    ///
    /// AES-256-GCM: authenticated encryption.
    /// Output = 12 + plaintext.len() + 16 bytes.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, ISLError> {
        use aes_gcm::aead::generic_array::GenericArray;
        use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit};

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&self.key));
        let nonce_bytes = self.next_nonce();
        let nonce = GenericArray::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| ISLError::AuthFailed)?;

        let mut out = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    /// Decrypt `[nonce:12B][ciphertext+tag]` → plaintext.
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>, ISLError> {
        use aes_gcm::aead::generic_array::GenericArray;
        use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit};

        if encrypted.len() < NONCE_SIZE + TAG_SIZE {
            return Err(ISLError::TooShort);
        }

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&self.key));
        let nonce = GenericArray::from_slice(&encrypted[..NONCE_SIZE]);
        let ciphertext = &encrypted[NONCE_SIZE..];

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| ISLError::AuthFailed)
    }

    /// Encode + encrypt ISLMessage.
    ///
    /// Output: `[nonce:12B][encrypted(12B msg):12B+16B tag]` = 40 bytes.
    pub fn encode_encrypted(&mut self, msg: &ISLMessage) -> Result<Vec<u8>, ISLError> {
        let plain = msg.to_bytes();
        self.encrypt(&plain)
    }

    /// Decrypt + decode ISLMessage.
    pub fn decode_encrypted(&self, bytes: &[u8]) -> Result<ISLMessage, ISLError> {
        let plain = self.decrypt(bytes)?;
        if plain.len() < 12 {
            return Err(ISLError::TooShort);
        }
        ISLMessage::from_bytes(&plain[..12]).ok_or(ISLError::InvalidMsgType)
    }

    /// Encode + encrypt ISLFrame.
    pub fn encode_frame_encrypted(&mut self, frame: &ISLFrame) -> Result<Vec<u8>, ISLError> {
        if frame.body.len() > 65535 {
            return Err(ISLError::BodyTooLong);
        }
        let plain = frame.to_bytes();
        self.encrypt(&plain)
    }

    /// Decrypt + decode ISLFrame.
    pub fn decode_frame_encrypted(&self, bytes: &[u8]) -> Result<ISLFrame, ISLError> {
        let plain = self.decrypt(bytes)?;
        if plain.len() < 14 {
            return Err(ISLError::TooShort);
        }
        ISLFrame::from_bytes(&plain).ok_or(ISLError::InvalidMsgType)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::ISLAddress;
    use crate::message::MsgType;

    fn addr(l: u8, g: u8, s: u8, i: u8) -> ISLAddress {
        ISLAddress::new(l, g, s, i)
    }

    #[test]
    fn encode_decode_roundtrip() {
        let codec = ISLCodec::new();
        let msg = ISLMessage::tick(addr(1, 0, 0, 1), 7);
        let bytes = codec.encode(&msg);
        let dec = codec.decode(&bytes).unwrap();
        assert_eq!(msg, dec);
    }

    #[test]
    fn checksum_detects_corruption() {
        let codec = ISLCodec::new();
        let msg = ISLMessage::emergency(addr(0, 0, 0, 1), 0xFE);
        let mut bytes = codec.encode(&msg);
        bytes[5] ^= 0xFF; // corrupt byte 5
        assert_eq!(codec.decode(&bytes), Err(ISLError::InvalidChecksum));
    }

    #[test]
    fn frame_encode_decode() {
        let codec = ISLCodec::new();
        let msg = ISLMessage::new(addr(0, 0, 0, 1), addr(1, 0, 0, 2), MsgType::Text);
        let frame = ISLFrame::with_body(msg, b"xin chao the gioi".to_vec());

        let bytes = codec.encode_frame(&frame).unwrap();
        let dec = codec.decode_frame(&bytes).unwrap();

        assert_eq!(dec.header, frame.header);
        assert_eq!(dec.body, frame.body);
    }

    #[test]
    fn too_short_returns_error() {
        let codec = ISLCodec::new();
        assert_eq!(codec.decode(&[0u8; 5]), Err(ISLError::TooShort));
        assert_eq!(codec.decode_frame(&[0u8; 8]), Err(ISLError::TooShort));
    }

    #[test]
    fn wire_size_vs_json() {
        // 12 bytes ISL vs ~280 bytes JSON
        let msg_size = ISLMessage::SIZE;
        let json_size = 280usize; // typical JSON command
        let saving = (json_size - msg_size) * 100 / json_size;
        assert!(saving > 90, "ISL tiết kiệm >90% so với JSON: {}%", saving);
    }

    #[test]
    fn no_checksum_mode() {
        let mut codec = ISLCodec::new();
        codec.use_checksum = false;
        let msg = ISLMessage::tick(addr(0, 0, 0, 1), 1);
        let bytes = codec.encode(&msg);
        assert_eq!(bytes.len(), 12, "No checksum = 12 bytes");
        let dec = codec.decode(&bytes).unwrap();
        assert_eq!(msg, dec);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Encryption tests (feature = "encrypt")
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "encrypt"))]
mod encrypt_tests {
    use super::*;
    use crate::address::ISLAddress;
    use crate::message::MsgType;

    fn addr(l: u8, g: u8, s: u8, i: u8) -> ISLAddress {
        ISLAddress::new(l, g, s, i)
    }

    fn test_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        for (i, b) in key.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(7).wrapping_add(0x42);
        }
        key
    }

    #[test]
    fn encrypt_decrypt_message_roundtrip() {
        let mut codec = ISLCodec::with_key(test_key());
        let msg = ISLMessage::tick(addr(1, 0, 0, 1), 7);
        let encrypted = codec.encode_encrypted(&msg).unwrap();

        // Output size: 12B nonce + 12B msg + 16B tag = 40B
        assert_eq!(encrypted.len(), 40);

        let dec = codec.decode_encrypted(&encrypted).unwrap();
        assert_eq!(msg, dec);
    }

    #[test]
    fn encrypt_decrypt_frame_roundtrip() {
        let mut codec = ISLCodec::with_key(test_key());
        let msg = ISLMessage::new(addr(0, 0, 0, 1), addr(1, 0, 0, 2), MsgType::Text);
        let frame = ISLFrame::with_body(msg, b"xin chao".to_vec());

        let encrypted = codec.encode_frame_encrypted(&frame).unwrap();
        let dec = codec.decode_frame_encrypted(&encrypted).unwrap();

        assert_eq!(dec.header, frame.header);
        assert_eq!(dec.body, frame.body);
    }

    #[test]
    fn tamper_detection() {
        let mut codec = ISLCodec::with_key(test_key());
        let msg = ISLMessage::emergency(addr(0, 0, 0, 1), 0xFE);
        let mut encrypted = codec.encode_encrypted(&msg).unwrap();

        // Corrupt 1 byte in ciphertext
        let idx = NONCE_SIZE + 3;
        encrypted[idx] ^= 0xFF;

        assert_eq!(
            codec.decode_encrypted(&encrypted),
            Err(ISLError::AuthFailed)
        );
    }

    #[test]
    fn wrong_key_fails() {
        let mut enc_codec = ISLCodec::with_key(test_key());
        let msg = ISLMessage::tick(addr(0, 0, 0, 1), 1);
        let encrypted = enc_codec.encode_encrypted(&msg).unwrap();

        // Decode with different key
        let wrong_key = [0xFFu8; 32];
        let dec_codec = ISLCodec::with_key(wrong_key);
        assert_eq!(
            dec_codec.decode_encrypted(&encrypted),
            Err(ISLError::AuthFailed)
        );
    }

    #[test]
    fn nonce_counter_increments() {
        let mut codec = ISLCodec::with_key(test_key());
        let msg = ISLMessage::tick(addr(0, 0, 0, 1), 1);

        let enc1 = codec.encode_encrypted(&msg).unwrap();
        let enc2 = codec.encode_encrypted(&msg).unwrap();

        // Same plaintext → different ciphertext (different nonce)
        assert_ne!(enc1, enc2);

        // Both decrypt correctly
        assert_eq!(codec.decode_encrypted(&enc1).unwrap(), msg);
        assert_eq!(codec.decode_encrypted(&enc2).unwrap(), msg);
    }

    #[test]
    fn too_short_encrypted_fails() {
        let codec = ISLCodec::with_key(test_key());
        // Less than NONCE_SIZE + TAG_SIZE = 28 bytes
        assert_eq!(codec.decode_encrypted(&[0u8; 20]), Err(ISLError::TooShort));
    }

    #[test]
    fn frame_body_too_long_encrypted() {
        let mut codec = ISLCodec::with_key(test_key());
        let msg = ISLMessage::tick(addr(0, 0, 0, 1), 1);
        let body = alloc::vec![0u8; 65536]; // > 65535
        let frame = ISLFrame::with_body(msg, body);
        assert_eq!(
            codec.encode_frame_encrypted(&frame),
            Err(ISLError::BodyTooLong)
        );
    }
}
