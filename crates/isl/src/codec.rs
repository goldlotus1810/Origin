//! # codec — ISLCodec
//!
//! Encode/decode ISLMessage → bytes.
//! AES-256-GCM ready: key field có nhưng encryption optional
//! (no_std — crypto crate cần feature flag riêng).
//!
//! Hiện tại: encode/decode không encrypt (plaintext, dùng cho local IPC).
//! Khi cần encrypt: thêm aes-gcm feature.

extern crate alloc;
use alloc::vec::Vec;
use crate::message::{ISLMessage, ISLFrame, MsgType};
use crate::address::ISLAddress;

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
/// Key field dành cho AES-256-GCM khi enable feature.
#[allow(missing_docs)]
pub struct ISLCodec {
    /// AES-256 key (32 bytes). Hiện không dùng — placeholder cho future.
    key:  [u8; 32],
    /// Dùng checksum XOR đơn giản để verify integrity.
    pub use_checksum: bool,
}

impl ISLCodec {
    /// Tạo codec plaintext.
    pub fn new() -> Self {
        Self { key: [0u8; 32], use_checksum: true }
    }

    /// Tạo với key (dành cho future AES-GCM).
    pub fn with_key(key: [u8; 32]) -> Self {
        Self { key, use_checksum: true }
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
        if bytes.len() < min_len { return Err(ISLError::TooShort); }

        if self.use_checksum {
            let expected = checksum(&bytes[..12]);
            if bytes[12] != expected { return Err(ISLError::InvalidChecksum); }
        }

        ISLMessage::from_bytes(&bytes[..12]).ok_or(ISLError::InvalidMsgType)
    }

    /// Encode ISLFrame → bytes.
    pub fn encode_frame(&self, frame: &ISLFrame) -> Result<Vec<u8>, ISLError> {
        if frame.body.len() > 65535 { return Err(ISLError::BodyTooLong); }
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
        if bytes.len() < min_len { return Err(ISLError::TooShort); }

        let raw_end = if self.use_checksum { bytes.len() - 1 } else { bytes.len() };

        if self.use_checksum {
            let expected = checksum(&bytes[..raw_end]);
            if bytes[raw_end] != expected { return Err(ISLError::InvalidChecksum); }
        }

        ISLFrame::from_bytes(&bytes[..raw_end]).ok_or(ISLError::InvalidMsgType)
    }
}

impl Default for ISLCodec {
    fn default() -> Self { Self::new() }
}

/// XOR checksum — nhẹ, không cần crypto.
fn checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc ^ b)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(l: u8, g: u8, s: u8, i: u8) -> ISLAddress { ISLAddress::new(l, g, s, i) }

    #[test]
    fn encode_decode_roundtrip() {
        let codec = ISLCodec::new();
        let msg   = ISLMessage::tick(addr(1,0,0,1), 7);
        let bytes = codec.encode(&msg);
        let dec   = codec.decode(&bytes).unwrap();
        assert_eq!(msg, dec);
    }

    #[test]
    fn checksum_detects_corruption() {
        let codec = ISLCodec::new();
        let msg   = ISLMessage::emergency(addr(0,0,0,1), 0xFE);
        let mut bytes = codec.encode(&msg);
        bytes[5] ^= 0xFF; // corrupt byte 5
        assert_eq!(codec.decode(&bytes), Err(ISLError::InvalidChecksum));
    }

    #[test]
    fn frame_encode_decode() {
        let codec = ISLCodec::new();
        let msg   = ISLMessage::new(addr(0,0,0,1), addr(1,0,0,2), MsgType::Text);
        let frame = ISLFrame::with_body(msg, b"xin chao the gioi".to_vec());

        let bytes = codec.encode_frame(&frame).unwrap();
        let dec   = codec.decode_frame(&bytes).unwrap();

        assert_eq!(dec.header, frame.header);
        assert_eq!(dec.body,   frame.body);
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
        let msg_size  = ISLMessage::SIZE;
        let json_size = 280usize; // typical JSON command
        let saving    = (json_size - msg_size) * 100 / json_size;
        assert!(saving > 90, "ISL tiết kiệm >90% so với JSON: {}%", saving);
    }

    #[test]
    fn no_checksum_mode() {
        let mut codec      = ISLCodec::new();
        codec.use_checksum = false;
        let msg   = ISLMessage::tick(addr(0,0,0,1), 1);
        let bytes = codec.encode(&msg);
        assert_eq!(bytes.len(), 12, "No checksum = 12 bytes");
        let dec   = codec.decode(&bytes).unwrap();
        assert_eq!(msg, dec);
    }
}
