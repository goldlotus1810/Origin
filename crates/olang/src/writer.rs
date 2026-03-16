//! # writer — Append-only binary writer
//!
//! Ghi nodes và edges vào origin.olang.
//! Append-only — không bao giờ overwrite hay delete (QT8).
//!
//! ## File format (origin.olang):
//!
//! ```text
//! [MAGIC: 4 bytes "○LNG"]
//! [VERSION: 1 byte = 0x03]
//! [CREATED: 8 bytes i64 nanoseconds]
//! [records...]
//!
//! Record types:
//!   0x01 = NodeRecord
//!   0x02 = EdgeRecord
//!   0x03 = AliasRecord
//!
//! NodeRecord:
//!   [0x01][chain_len: u8][chain: N×5 bytes][layer: u8]
//!   [is_qr: u8][timestamp: 8 bytes i64]
//!   Total: 1 + 1 + N×5 + 1 + 1 + 8 = 12 + N×5 bytes
//!
//! EdgeRecord:
//!   [0x02][from_hash: 8 bytes][to_hash: 8 bytes][edge_type: u8]
//!   [timestamp: 8 bytes i64]
//!   Total: 1 + 8 + 8 + 1 + 8 = 26 bytes
//!
//! AliasRecord:
//!   [0x03][name_len: u8][name: N bytes][chain_hash: 8 bytes]
//!   [timestamp: 8 bytes i64]
//!   Total: 1 + 1 + N + 8 + 8 = 18 + N bytes
//! ```

extern crate alloc;
use alloc::vec::Vec;

use crate::molecular::MolecularChain;

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Magic bytes: "○LNG" = 0xE2 0x97 0x8B 0x4C (○ = U+25CB)
pub const MAGIC: [u8; 4] = [0xE2, 0x97, 0x8B, 0x4C];

/// Version hiện tại — v0.04 (thêm RT_AMEND)
pub const VERSION: u8 = 0x04;
/// Version trước — v0.03 (vẫn đọc được)
pub const VERSION_V03: u8 = 0x03;

/// Record type bytes
/// Record type: Node
pub const RT_NODE: u8 = 0x01;
/// Record type: Edge
pub const RT_EDGE: u8 = 0x02;
/// Record type: Alias
pub const RT_ALIAS: u8 = 0x03;
/// Record type: Amendment — supersede a previous record (append-only rollback)
///
/// Format: [0x04][target_offset: 8][reason_len: u8][reason: N][timestamp: 8]
/// Total: 1 + 8 + 1 + N + 8 = 18 + N bytes
///
/// QT8 compliant: không xóa record cũ — chỉ đánh dấu "đã thay thế".
pub const RT_AMEND: u8 = 0x04;

/// Header size: MAGIC(4) + VERSION(1) + CREATED(8) = 13 bytes
pub const HEADER_SIZE: usize = 13;

// ─────────────────────────────────────────────────────────────────────────────
// OlangWriter
// ─────────────────────────────────────────────────────────────────────────────

/// Append-only writer cho origin.olang.
///
/// Ghi vào in-memory buffer. Persist bằng `as_bytes()`.
/// Trong production: flush to disk sau mỗi write (QT8).
#[allow(missing_docs)]
pub struct OlangWriter {
    buf: Vec<u8>,
    write_count: u64,
}

impl OlangWriter {
    /// Tạo writer mới với header.
    pub fn new(created_at: i64) -> Self {
        let mut w = Self {
            buf: Vec::new(),
            write_count: 0,
        };
        w.write_header(created_at);
        w
    }

    /// Tạo writer từ existing bytes (append mode).
    pub fn from_existing(existing: Vec<u8>) -> Self {
        Self {
            buf: existing,
            write_count: 0,
        }
    }

    fn write_header(&mut self, created_at: i64) {
        self.buf.extend_from_slice(&MAGIC);
        self.buf.push(VERSION);
        self.buf.extend_from_slice(&created_at.to_le_bytes());
    }

    /// Ghi NodeRecord.
    ///
    /// Returns offset của record trong file.
    pub fn append_node(
        &mut self,
        chain: &MolecularChain,
        layer: u8,
        is_qr: bool,
        timestamp: i64,
    ) -> Result<u64, WriteError> {
        let chain_bytes = chain.to_bytes();
        let chain_len = chain_bytes.len() / 5; // số molecules

        if chain_len > 255 {
            return Err(WriteError::ChainTooLong);
        }

        let offset = self.buf.len() as u64;

        self.buf.push(RT_NODE);
        self.buf.push(chain_len as u8);
        self.buf.extend_from_slice(&chain_bytes);
        self.buf.push(layer);
        self.buf.push(if is_qr { 0x01 } else { 0x00 });
        self.buf.extend_from_slice(&timestamp.to_le_bytes());

        self.write_count += 1;
        Ok(offset)
    }

    /// Ghi EdgeRecord.
    pub fn append_edge(
        &mut self,
        from_hash: u64,
        to_hash: u64,
        edge_type: u8,
        timestamp: i64,
    ) -> u64 {
        let offset = self.buf.len() as u64;

        self.buf.push(RT_EDGE);
        self.buf.extend_from_slice(&from_hash.to_le_bytes());
        self.buf.extend_from_slice(&to_hash.to_le_bytes());
        self.buf.push(edge_type);
        self.buf.extend_from_slice(&timestamp.to_le_bytes());

        self.write_count += 1;
        offset
    }

    /// Ghi AliasRecord.
    pub fn append_alias(
        &mut self,
        name: &str,
        chain_hash: u64,
        timestamp: i64,
    ) -> Result<u64, WriteError> {
        let name_bytes = name.as_bytes();
        if name_bytes.len() > 255 {
            return Err(WriteError::NameTooLong);
        }

        let offset = self.buf.len() as u64;

        self.buf.push(RT_ALIAS);
        self.buf.push(name_bytes.len() as u8);
        self.buf.extend_from_slice(name_bytes);
        self.buf.extend_from_slice(&chain_hash.to_le_bytes());
        self.buf.extend_from_slice(&timestamp.to_le_bytes());

        self.write_count += 1;
        Ok(offset)
    }

    /// Ghi AmendRecord — append-only rollback.
    ///
    /// Supersede một record cũ tại `target_offset` với lý do.
    /// QT8: record cũ VẪN CÒN trong file — chỉ bị đánh dấu amended.
    pub fn append_amend(
        &mut self,
        target_offset: u64,
        reason: &str,
        timestamp: i64,
    ) -> Result<u64, WriteError> {
        let reason_bytes = reason.as_bytes();
        if reason_bytes.len() > 255 {
            return Err(WriteError::NameTooLong);
        }

        let offset = self.buf.len() as u64;

        self.buf.push(RT_AMEND);
        self.buf.extend_from_slice(&target_offset.to_le_bytes());
        self.buf.push(reason_bytes.len() as u8);
        self.buf.extend_from_slice(reason_bytes);
        self.buf.extend_from_slice(&timestamp.to_le_bytes());

        self.write_count += 1;
        Ok(offset)
    }

    /// Raw bytes của file (để flush to disk hoặc test).
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    /// Kích thước file hiện tại.
    pub fn size(&self) -> usize {
        self.buf.len()
    }

    /// Số records đã ghi.
    pub fn write_count(&self) -> u64 {
        self.write_count
    }

    /// Consume writer → bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WriteError
// ─────────────────────────────────────────────────────────────────────────────

/// Lỗi khi ghi.
#[derive(Debug, Clone, PartialEq)]
pub enum WriteError {
    /// Chain quá dài (> 255 molecules)
    ChainTooLong,
    /// Name quá dài (> 255 bytes)
    NameTooLong,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::encode_codepoint;
    use alloc::string::String;

    fn skip_if_empty() -> bool {
        ucd::table_len() == 0
    }

    #[test]
    fn writer_header() {
        let w = OlangWriter::new(1000);
        let bytes = w.as_bytes();
        assert!(
            bytes.len() >= HEADER_SIZE,
            "Header phải có ít nhất {} bytes",
            HEADER_SIZE
        );
        assert_eq!(&bytes[0..4], &MAGIC, "Magic bytes đúng");
        assert_eq!(bytes[4], VERSION, "Version đúng");
        // CREATED = bytes[5..13] = 1000i64 LE
        let created = i64::from_le_bytes(bytes[5..13].try_into().unwrap());
        assert_eq!(created, 1000);
    }

    #[test]
    fn write_node() {
        if skip_if_empty() {
            return;
        }
        let mut w = OlangWriter::new(0);
        let chain = encode_codepoint(0x1F525); // 🔥
        let before = w.size();
        let offset = w.append_node(&chain, 0, false, 1000).unwrap();

        assert_eq!(offset, before as u64, "Offset phải là vị trí trước khi ghi");
        // NodeRecord: 1 + 1 + 1×5 + 1 + 1 + 8 = 17 bytes
        assert_eq!(
            w.size() - before,
            17,
            "NodeRecord size = 17 bytes cho 1-mol chain"
        );
        assert_eq!(w.write_count(), 1);
    }

    #[test]
    fn write_node_qr() {
        if skip_if_empty() {
            return;
        }
        let mut w = OlangWriter::new(0);
        let chain = encode_codepoint(0x1F525);
        w.append_node(&chain, 0, true, 1000).unwrap();

        // Verify QR flag
        let bytes = w.as_bytes();
        // QR byte: offset = HEADER_SIZE + 1(type) + 1(len) + 5(chain) + 1(layer) = HEADER_SIZE + 8
        let qr_pos = HEADER_SIZE + 1 + 1 + 5 + 1;
        assert_eq!(bytes[qr_pos], 0x01, "QR flag = 0x01");
    }

    #[test]
    fn write_edge() {
        let mut w = OlangWriter::new(0);
        let before = w.size();
        let offset = w.append_edge(0xABCD, 0xEF12, 0x01, 2000);

        assert_eq!(offset, before as u64);
        // EdgeRecord: 1 + 8 + 8 + 1 + 8 = 26 bytes
        assert_eq!(w.size() - before, 26, "EdgeRecord = 26 bytes");
    }

    #[test]
    fn write_alias() {
        let mut w = OlangWriter::new(0);
        let before = w.size();
        let offset = w.append_alias("lửa", 0xFEED, 3000).unwrap();

        assert_eq!(offset, before as u64);
        // AliasRecord: 1 + 1 + len("lửa") + 8 + 8
        // "lửa" = 6 bytes UTF-8
        let name_len = "lửa".as_bytes().len();
        let expected = 1 + 1 + name_len + 8 + 8;
        assert_eq!(w.size() - before, expected);
    }

    #[test]
    fn write_alias_too_long() {
        let mut w = OlangWriter::new(0);
        let long_name: String = (0..256).map(|_| 'a').collect();
        let result = w.append_alias(&long_name, 0x1234, 0);
        assert_eq!(result, Err(WriteError::NameTooLong));
    }

    #[test]
    fn write_sequence_offsets() {
        if skip_if_empty() {
            return;
        }
        let mut w = OlangWriter::new(0);

        let c1 = encode_codepoint(0x1F525);
        let c2 = encode_codepoint(0x1F4A7);

        let off1 = w.append_node(&c1, 0, false, 1000).unwrap();
        let off2 = w.append_node(&c2, 0, false, 2000).unwrap();

        assert!(off2 > off1, "Offsets tăng dần (append-only)");
        assert_eq!(w.write_count(), 2);
    }

    #[test]
    fn write_mixed_records() {
        if skip_if_empty() {
            return;
        }
        let mut w = OlangWriter::new(0);

        let chain = encode_codepoint(0x1F525);
        let hash = chain.chain_hash();

        w.append_node(&chain, 0, false, 1000).unwrap();
        w.append_alias("fire", hash, 1001).unwrap();
        w.append_edge(hash, 0xDEAD, 0x01, 1002);

        assert_eq!(w.write_count(), 3);
        assert!(w.size() > HEADER_SIZE);
    }

    #[test]
    fn writer_append_only_grows() {
        let mut w = OlangWriter::new(0);
        let mut prev_size = w.size();

        for i in 0u64..5 {
            w.append_edge(i, i + 1, 0x01, i as i64);
            assert!(w.size() > prev_size, "File chỉ tăng, không giảm");
            prev_size = w.size();
        }
    }
}
