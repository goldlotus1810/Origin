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
//!   0x04 = AmendRecord
//!   0x05 = NodeKindRecord
//!
//! NodeRecord (v0.06 — u16 chain links):
//!   [0x01][link_count: u16_le][u16_le × N][layer: u8]
//!   [is_qr: u8][timestamp: 8 bytes i64]
//!   Total: 1 + 2 + N×2 + 1 + 1 + 8 bytes
//!
//! NodeRecord (v0.05 legacy — tagged molecule encoding):
//!   [0x01][mol_count: u8][tagged_chain_bytes...][layer: u8]
//!   [is_qr: u8][timestamp: 8 bytes i64]
//!
//! NodeRecord (v0.03-v0.04 legacy — fixed 5-byte molecules):
//!   [0x01][chain_len: u8][chain: N×5 bytes][layer: u8]
//!   [is_qr: u8][timestamp: 8 bytes i64]
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

/// Version hiện tại — v0.06 (u16 chain links)
pub const VERSION: u8 = 0x06;
/// Version trước — v0.05 (tagged molecule encoding, vẫn đọc được)
pub const VERSION_V05: u8 = 0x05;
/// Version trước — v0.04 (thêm RT_AMEND, vẫn đọc được)
pub const VERSION_V04: u8 = 0x04;
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
/// Record type: NodeKind — gán NodeKind cho một node đã có trong sổ cái.
///
/// Format: [0x05][chain_hash: 8][kind: u8][timestamp: 8]
/// Total: 1 + 8 + 1 + 8 = 18 bytes
///
/// Cho phép origin.olang lưu trữ NodeKind (Skill/Agent/Program/Sensor/...)
/// → L0 đọc file → biết mình có gì → cuốn sổ cái đầy đủ.
pub const RT_NODE_KIND: u8 = 0x05;

/// Record type: STM Observation — persist short-term memory vào origin.olang.
///
/// Format: [0x06][chain_hash: 8][valence: 4][arousal: 4][dominance: 4][intensity: 4]
///         [fire_count: 4][maturity: 1][layer: 1][timestamp: 8]
/// Total: 1 + 8 + 16 + 4 + 1 + 1 + 8 = 39 bytes
///
/// QT8: origin.olang = bộ nhớ duy nhất, RAM = cache tạm.
pub const RT_STM: u8 = 0x06;

/// Record type: HebbianLink — persist learned Silk weight vào origin.olang.
///
/// Format: [0x07][from_hash: 8][to_hash: 8][weight: 1][fire_count: 2][timestamp: 8]
/// Total: 1 + 8 + 8 + 1 + 2 + 8 = 28 bytes
pub const RT_HEBBIAN: u8 = 0x07;

/// Record type: KnowTree CompactNode — persist L2+ knowledge vào origin.olang.
///
/// Format: [0x08][data_len: 2][compact_node_bytes: N][timestamp: 8]
/// Total: 1 + 2 + N + 8 bytes
pub const RT_KNOWTREE: u8 = 0x08;

/// Record type: ConversationCurve — persist emotion trajectory vào origin.olang.
///
/// Format: [0x09][valence: 4][fx_dn: 4][timestamp: 8]
/// Total: 1 + 4 + 4 + 8 = 17 bytes
///
/// Mỗi turn ghi 1 record. Boot replay → reconstruct curve.
pub const RT_CURVE: u8 = 0x09;

/// Record type: SlimKnowTree node — spec-compliant ~18B per record.
///
/// Format: [0x0A][hash: 8][tagged_len: 1][tagged: 1-6][layer: 1][timestamp: 8]
/// Total: 1 + 8 + 1 + (1-6) + 1 + 8 = 20-25 bytes per record
///
/// So sánh với RT_KNOWTREE (0x08):
///   0x08: CompactNode bytes (28B header + variable) — legacy, phình
///   0x0A: SlimNode (hash + tagged mol + layer + ts) — spec-compliant
///
/// 500M records × 22B avg = 11GB → fits on phone with room to spare.
pub const RT_SLIM_KNOWTREE: u8 = 0x0A;

/// Record type: AuthHeader — master key identity for origin.olang.
///
/// Format: [0x0B][auth_header: 113][timestamp: 8]
/// Total: 1 + 113 + 8 = 122 bytes
///
/// Append-only: mỗi lần setup/change → append record mới.
/// Boot replay: last RT_AUTH record wins.
pub const RT_AUTH: u8 = 0x0B;

/// Record type: Parent — Silk vertical parent_map persistence (T15/14.3).
///
/// Format: [0x0C][child_hash: 8][parent_hash: 8][timestamp: 8]
/// Total: 1 + 8 + 8 + 8 = 25 bytes
///
/// Boot replay: rebuild SilkGraph.parent_map for bottom-up traversal.
/// 8,846 pointers × 25B = ~221 KB on disk.
pub const RT_PARENT: u8 = 0x0C;

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

    /// Tạo writer rỗng KHÔNG có header — dùng khi append vào file đã có header.
    pub fn new_append() -> Self {
        Self {
            buf: Vec::new(),
            write_count: 0,
        }
    }

    fn write_header(&mut self, created_at: i64) {
        self.buf.extend_from_slice(&MAGIC);
        self.buf.push(VERSION);
        self.buf.extend_from_slice(&created_at.to_le_bytes());
    }

    /// Ghi NodeRecord (v0.06 u16 chain links).
    ///
    /// Format: `[0x01][link_count: u16_le][u16_le × N][layer][is_qr][ts:8]`
    /// Mỗi link = 2 bytes (u16 little-endian).
    /// Returns offset của record trong file.
    pub fn append_node(
        &mut self,
        chain: &MolecularChain,
        layer: u8,
        is_qr: bool,
        timestamp: i64,
    ) -> Result<u64, WriteError> {
        if chain.len() > 65535 {
            return Err(WriteError::ChainTooLong);
        }

        let offset = self.buf.len() as u64;

        self.buf.push(RT_NODE);
        // v0.06: [link_count: u16_le][u16_le × N]
        let count = chain.len() as u16;
        self.buf.extend_from_slice(&count.to_le_bytes());
        for &bits in &chain.0 {
            self.buf.extend_from_slice(&bits.to_le_bytes());
        }
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

    /// Ghi NodeKindRecord — gán NodeKind cho một chain_hash.
    ///
    /// Ghi SAU append_node() (QT8: node phải tồn tại trước).
    /// Cho phép origin.olang trở thành cuốn sổ cái đầy đủ:
    /// L0 đọc file → thấy node + kind → biết mình có Skill/Agent/Program/Sensor gì.
    pub fn append_node_kind(
        &mut self,
        chain_hash: u64,
        kind: u8,
        timestamp: i64,
    ) -> u64 {
        let offset = self.buf.len() as u64;

        self.buf.push(RT_NODE_KIND);
        self.buf.extend_from_slice(&chain_hash.to_le_bytes());
        self.buf.push(kind);
        self.buf.extend_from_slice(&timestamp.to_le_bytes());

        self.write_count += 1;
        offset
    }

    /// Ghi STM Observation record.
    ///
    /// Persist 1 observation vào origin.olang (QT8: file = bộ nhớ duy nhất).
    #[allow(clippy::too_many_arguments)]
    pub fn append_stm(
        &mut self,
        chain_hash: u64,
        valence: f32,
        arousal: f32,
        dominance: f32,
        intensity: f32,
        fire_count: u32,
        maturity: u8,
        layer: u8,
        timestamp: i64,
    ) -> u64 {
        let offset = self.buf.len() as u64;
        self.buf.push(RT_STM);
        self.buf.extend_from_slice(&chain_hash.to_le_bytes());
        self.buf.extend_from_slice(&valence.to_le_bytes());
        self.buf.extend_from_slice(&arousal.to_le_bytes());
        self.buf.extend_from_slice(&dominance.to_le_bytes());
        self.buf.extend_from_slice(&intensity.to_le_bytes());
        self.buf.extend_from_slice(&fire_count.to_le_bytes());
        self.buf.push(maturity);
        self.buf.push(layer);
        self.buf.extend_from_slice(&timestamp.to_le_bytes());
        self.write_count += 1;
        offset
    }

    /// Ghi HebbianLink record.
    ///
    /// Persist 1 learned Silk weight vào origin.olang.
    pub fn append_hebbian(
        &mut self,
        from_hash: u64,
        to_hash: u64,
        weight: u8,
        fire_count: u16,
        timestamp: i64,
    ) -> u64 {
        let offset = self.buf.len() as u64;
        self.buf.push(RT_HEBBIAN);
        self.buf.extend_from_slice(&from_hash.to_le_bytes());
        self.buf.extend_from_slice(&to_hash.to_le_bytes());
        self.buf.push(weight);
        self.buf.extend_from_slice(&fire_count.to_le_bytes());
        self.buf.extend_from_slice(&timestamp.to_le_bytes());
        self.write_count += 1;
        offset
    }

    /// Ghi KnowTree CompactNode record.
    ///
    /// Persist 1 L2+ compact node vào origin.olang.
    pub fn append_knowtree(
        &mut self,
        compact_bytes: &[u8],
        timestamp: i64,
    ) -> Result<u64, WriteError> {
        if compact_bytes.len() > u16::MAX as usize {
            return Err(WriteError::NameTooLong); // reuse error variant
        }
        let offset = self.buf.len() as u64;
        self.buf.push(RT_KNOWTREE);
        self.buf.extend_from_slice(&(compact_bytes.len() as u16).to_le_bytes());
        self.buf.extend_from_slice(compact_bytes);
        self.buf.extend_from_slice(&timestamp.to_le_bytes());
        self.write_count += 1;
        Ok(offset)
    }

    /// Ghi SlimKnowTree node record — spec-compliant format.
    ///
    /// Per-node: [0x0A][hash:8][tagged_len:1][tagged:1-6][layer:1][ts:8]
    /// Thay thế append_knowtree() cho writes mới.
    pub fn append_slim_knowtree(
        &mut self,
        hash: u64,
        tagged_bytes: &[u8],
        layer: u8,
        timestamp: i64,
    ) -> Result<u64, WriteError> {
        if tagged_bytes.is_empty() || tagged_bytes.len() > 32 {
            return Err(WriteError::ChainTooLong);
        }
        let offset = self.buf.len() as u64;
        self.buf.push(RT_SLIM_KNOWTREE);
        self.buf.extend_from_slice(&hash.to_le_bytes());
        self.buf.push(tagged_bytes.len() as u8);
        self.buf.extend_from_slice(tagged_bytes);
        self.buf.push(layer);
        self.buf.extend_from_slice(&timestamp.to_le_bytes());
        self.write_count += 1;
        Ok(offset)
    }

    /// Ghi ConversationCurve turn record.
    ///
    /// Mỗi turn ghi 1 record: valence + fx_dn + ts.
    /// Boot replay tất cả records → reconstruct curve.
    pub fn append_curve(
        &mut self,
        valence: f32,
        fx_dn: f32,
        timestamp: i64,
    ) -> u64 {
        let offset = self.buf.len() as u64;
        self.buf.push(RT_CURVE);
        self.buf.extend_from_slice(&valence.to_le_bytes());
        self.buf.extend_from_slice(&fx_dn.to_le_bytes());
        self.buf.extend_from_slice(&timestamp.to_le_bytes());
        self.write_count += 1;
        offset
    }

    /// Ghi Parent record — Silk vertical parent_map persistence (T15/14.3).
    ///
    /// Format: [0x0C][child_hash: 8][parent_hash: 8][ts: 8]
    /// Boot replay → SilkGraph.register_parent(child, parent).
    pub fn append_parent(
        &mut self,
        child_hash: u64,
        parent_hash: u64,
        timestamp: i64,
    ) -> u64 {
        let offset = self.buf.len() as u64;
        self.buf.push(RT_PARENT);
        self.buf.extend_from_slice(&child_hash.to_le_bytes());
        self.buf.extend_from_slice(&parent_hash.to_le_bytes());
        self.buf.extend_from_slice(&timestamp.to_le_bytes());
        self.write_count += 1;
        offset
    }

    /// Ghi AuthHeader record — master key identity.
    ///
    /// Append-only: last record wins on boot replay.
    /// Format: [0x0B][auth_header: 113][ts: 8]
    pub fn append_auth(
        &mut self,
        auth_bytes: &[u8; 113],
        timestamp: i64,
    ) -> u64 {
        let offset = self.buf.len() as u64;
        self.buf.push(RT_AUTH);
        self.buf.extend_from_slice(auth_bytes);
        self.buf.extend_from_slice(&timestamp.to_le_bytes());
        self.write_count += 1;
        offset
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
        let mut w = OlangWriter::new(0);
        let chain = encode_codepoint(0x1F525); // 🔥
        let before = w.size();
        let offset = w.append_node(&chain, 0, false, 1000).unwrap();

        assert_eq!(offset, before as u64, "Offset phải là vị trí trước khi ghi");
        // NodeRecord v0.06: 1(type) + 2(link_count) + N*2(u16 links) + 1(layer) + 1(is_qr) + 8(ts)
        let expected = 1 + 2 + chain.len() * 2 + 1 + 1 + 8;
        assert_eq!(
            w.size() - before,
            expected,
            "NodeRecord size = {} bytes cho {}-mol chain (v0.06)",
            expected,
            chain.len(),
        );
        assert_eq!(w.write_count(), 1);
    }

    #[test]
    fn write_node_qr() {
        let mut w = OlangWriter::new(0);
        let chain = encode_codepoint(0x1F525);
        w.append_node(&chain, 0, true, 1000).unwrap();

        // Verify QR flag
        let bytes = w.as_bytes();
        // v0.06: QR byte at HEADER + 1(type) + 2(count) + N*2(links) + 1(layer)
        let chain_data_size = 2 + chain.len() * 2;
        let qr_pos = HEADER_SIZE + 1 + chain_data_size + 1;
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
