//! # message — ISLMessage 12 bytes
//!
//! Base: 12 bytes vs JSON ~280 bytes → nhỏ hơn 95.7%
//!
//! Layout:
//!   [0..4]  from:     ISLAddress (4B)
//!   [4..8]  to:       ISLAddress (4B)
//!   [8]     msg_type: MsgType    (1B)
//!   [9..12] payload:  [u8; 3]    (3B)
//!
//! Payload lớn hơn → dùng ChainPayload + MolecularChain đính kèm.

extern crate alloc;
use crate::address::ISLAddress;
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// MsgType — 1 byte
// ─────────────────────────────────────────────────────────────────────────────

/// Loại message — 1 byte.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MsgType {
    Text = 0x01,         // text từ user
    Query = 0x02,        // tra cứu knowledge
    Learn = 0x03,        // dạy hệ thống
    Propose = 0x04,      // đề xuất ĐN → QR
    ActuatorCmd = 0x05,  // lệnh thiết bị
    Tick = 0x06,         // heartbeat
    Dream = 0x07,        // kích hoạt dream
    Emergency = 0x08,    // cảnh báo khẩn
    Approved = 0x09,     // AAM approve
    Broadcast = 0x0A,    // broadcast toàn hệ thống
    ChainPayload = 0x0B, // kèm MolecularChain
    Ack = 0x0C,          // acknowledge
    Nack = 0x0D,         // negative acknowledge
    Program = 0x0E,      // yêu cầu LeoAI lập trình + chạy VM
}

impl MsgType {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Text),
            0x02 => Some(Self::Query),
            0x03 => Some(Self::Learn),
            0x04 => Some(Self::Propose),
            0x05 => Some(Self::ActuatorCmd),
            0x06 => Some(Self::Tick),
            0x07 => Some(Self::Dream),
            0x08 => Some(Self::Emergency),
            0x09 => Some(Self::Approved),
            0x0A => Some(Self::Broadcast),
            0x0B => Some(Self::ChainPayload),
            0x0C => Some(Self::Ack),
            0x0D => Some(Self::Nack),
            0x0E => Some(Self::Program),
            _ => None,
        }
    }

    /// Message này cần xử lý ngay lập tức (priority).
    pub fn is_urgent(self) -> bool {
        matches!(self, Self::Emergency | Self::Tick)
    }

    /// Message này cần ACK.
    pub fn needs_ack(self) -> bool {
        matches!(self, Self::ActuatorCmd | Self::Propose | Self::Approved | Self::Program)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ISLMessage — 12 bytes base
// ─────────────────────────────────────────────────────────────────────────────

/// Message cơ bản 12 bytes.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ISLMessage {
    pub from: ISLAddress,
    pub to: ISLAddress,
    pub msg_type: MsgType,
    pub payload: [u8; 3],
}

impl ISLMessage {
    pub const SIZE: usize = 12;

    /// Tạo message cơ bản.
    pub fn new(from: ISLAddress, to: ISLAddress, msg_type: MsgType) -> Self {
        Self {
            from,
            to,
            msg_type,
            payload: [0; 3],
        }
    }

    /// Tạo với payload.
    pub fn with_payload(
        from: ISLAddress,
        to: ISLAddress,
        msg_type: MsgType,
        payload: [u8; 3],
    ) -> Self {
        Self {
            from,
            to,
            msg_type,
            payload,
        }
    }

    /// Heartbeat message.
    pub fn tick(from: ISLAddress, seq: u8) -> Self {
        Self::with_payload(from, ISLAddress::BROADCAST, MsgType::Tick, [seq, 0, 0])
    }

    /// Emergency broadcast.
    pub fn emergency(from: ISLAddress, code: u8) -> Self {
        Self::with_payload(
            from,
            ISLAddress::BROADCAST,
            MsgType::Emergency,
            [code, 0, 0],
        )
    }

    /// Actuator command: address + cmd byte.
    pub fn actuator(from: ISLAddress, device: ISLAddress, cmd: u8, value: u8) -> Self {
        Self::with_payload(from, device, MsgType::ActuatorCmd, [cmd, value, 0])
    }

    /// ACK cho một message.
    pub fn ack(from: ISLAddress, to: ISLAddress, msg_type: MsgType) -> Self {
        Self::with_payload(from, to, MsgType::Ack, [msg_type as u8, 0, 0])
    }

    /// NACK — từ chối thực hiện.
    pub fn nack(from: ISLAddress, to: ISLAddress, msg_type: MsgType) -> Self {
        Self::with_payload(from, to, MsgType::Nack, [msg_type as u8, 0, 0])
    }

    /// Serialize → 12 bytes.
    pub fn to_bytes(self) -> [u8; Self::SIZE] {
        let f = self.from.to_bytes();
        let t = self.to.to_bytes();
        [
            f[0],
            f[1],
            f[2],
            f[3],
            t[0],
            t[1],
            t[2],
            t[3],
            self.msg_type as u8,
            self.payload[0],
            self.payload[1],
            self.payload[2],
        ]
    }

    /// Deserialize từ 12 bytes.
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() < Self::SIZE {
            return None;
        }
        let from = ISLAddress::from_bytes([b[0], b[1], b[2], b[3]]);
        let to = ISLAddress::from_bytes([b[4], b[5], b[6], b[7]]);
        let msg_type = MsgType::from_byte(b[8])?;
        Some(Self {
            from,
            to,
            msg_type,
            payload: [b[9], b[10], b[11]],
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ISLFrame — message + optional extended payload
// ─────────────────────────────────────────────────────────────────────────────

/// Frame hoàn chỉnh: base message + payload mở rộng.
///
/// Dùng khi cần gửi dữ liệu lớn hơn 3 bytes (text, chain...).
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub struct ISLFrame {
    pub header: ISLMessage,
    /// Payload mở rộng (nếu có).
    pub body: Vec<u8>,
}

impl ISLFrame {
    /// Frame chỉ có header.
    pub fn bare(msg: ISLMessage) -> Self {
        Self {
            header: msg,
            body: Vec::new(),
        }
    }

    /// Frame với body.
    pub fn with_body(msg: ISLMessage, body: Vec<u8>) -> Self {
        Self { header: msg, body }
    }

    /// Serialize hoàn chỉnh: 12B header + 2B len + body.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(ISLMessage::SIZE + 2 + self.body.len());
        out.extend_from_slice(&self.header.to_bytes());
        let len = self.body.len() as u16;
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(&self.body);
        out
    }

    /// Deserialize.
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() < ISLMessage::SIZE + 2 {
            return None;
        }
        let header = ISLMessage::from_bytes(&b[..ISLMessage::SIZE])?;
        let len = u16::from_be_bytes([b[12], b[13]]) as usize;
        if b.len() < ISLMessage::SIZE + 2 + len {
            return None;
        }
        let body = b[14..14 + len].to_vec();
        Some(Self { header, body })
    }

    /// Tổng kích thước khi serialize.
    pub fn wire_size(&self) -> usize {
        ISLMessage::SIZE + 2 + self.body.len()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(l: u8, g: u8, s: u8, i: u8) -> ISLAddress {
        ISLAddress::new(l, g, s, i)
    }

    #[test]
    fn message_size_12_bytes() {
        let msg = ISLMessage::new(addr(0, 0, 0, 0), addr(1, 0, 0, 1), MsgType::Text);
        assert_eq!(msg.to_bytes().len(), 12);
        assert_eq!(ISLMessage::SIZE, 12, "12 bytes base spec");
    }

    #[test]
    fn message_round_trip() {
        let from = addr(0x01, 0x02, 0x03, 0x04);
        let to = addr(0xFF, 0xFF, 0xFF, 0xFF);
        let msg = ISLMessage::with_payload(from, to, MsgType::Emergency, [0xAB, 0xCD, 0xEF]);
        let bytes = msg.to_bytes();
        let decoded = ISLMessage::from_bytes(&bytes).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn tick_message_format() {
        let src = addr(1, 0, 0, 5);
        let tick = ISLMessage::tick(src, 42);
        assert_eq!(tick.msg_type, MsgType::Tick);
        assert_eq!(tick.payload[0], 42); // seq
        assert!(tick.to.is_broadcast());
    }

    #[test]
    fn emergency_broadcast() {
        let msg = ISLMessage::emergency(addr(0, 0, 0, 1), 0xFF);
        assert_eq!(msg.msg_type, MsgType::Emergency);
        assert!(msg.to.is_broadcast());
        assert!(msg.msg_type.is_urgent());
    }

    #[test]
    fn actuator_cmd() {
        let from = addr(0, 0, 0, 1);
        let device = addr(1, 5, 0, 3);
        let msg = ISLMessage::actuator(from, device, 0x01, 0x00); // cmd=ON, val=0
        assert_eq!(msg.msg_type, MsgType::ActuatorCmd);
        assert_eq!(msg.payload[0], 0x01);
        assert_eq!(msg.payload[1], 0x00);
        assert!(msg.msg_type.needs_ack());
    }

    #[test]
    fn ack_message() {
        let msg = ISLMessage::ack(addr(1, 0, 0, 1), addr(0, 0, 0, 0), MsgType::ActuatorCmd);
        assert_eq!(msg.msg_type, MsgType::Ack);
        assert_eq!(msg.payload[0], MsgType::ActuatorCmd as u8);
    }

    #[test]
    fn frame_with_body_round_trip() {
        let msg = ISLMessage::new(addr(0, 0, 0, 1), addr(0, 0, 0, 2), MsgType::Text);
        let body = b"xin chao".to_vec();
        let frame = ISLFrame::with_body(msg, body.clone());

        let bytes = frame.to_bytes();
        let decoded = ISLFrame::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.header, msg);
        assert_eq!(decoded.body, body);
    }

    #[test]
    fn frame_wire_size_correct() {
        let msg = ISLMessage::new(addr(0, 0, 0, 1), addr(0, 0, 0, 2), MsgType::ChainPayload);
        let body = alloc::vec![0u8; 100];
        let frame = ISLFrame::with_body(msg, body);
        let bytes = frame.to_bytes();
        assert_eq!(bytes.len(), frame.wire_size());
        assert_eq!(frame.wire_size(), 12 + 2 + 100);
    }

    #[test]
    fn frame_bare_small() {
        let msg = ISLMessage::tick(addr(0, 0, 0, 1), 1);
        let frame = ISLFrame::bare(msg);
        assert_eq!(frame.wire_size(), 14); // 12 + 2 (len=0)
    }

    #[test]
    fn msg_type_urgent() {
        assert!(MsgType::Emergency.is_urgent());
        assert!(MsgType::Tick.is_urgent());
        assert!(!MsgType::Text.is_urgent());
        assert!(!MsgType::Query.is_urgent());
    }

    #[test]
    fn msg_type_needs_ack() {
        assert!(MsgType::ActuatorCmd.needs_ack());
        assert!(MsgType::Propose.needs_ack());
        assert!(!MsgType::Tick.needs_ack());
        assert!(!MsgType::Text.needs_ack());
    }

    #[test]
    fn from_bytes_invalid_returns_none() {
        assert!(ISLMessage::from_bytes(&[0u8; 5]).is_none()); // too short
                                                              // Invalid msg_type
        let mut b = [0u8; 12];
        b[8] = 0xFF; // unknown type
        assert!(ISLMessage::from_bytes(&b).is_none());
    }
}
