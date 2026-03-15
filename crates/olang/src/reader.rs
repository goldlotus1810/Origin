//! # reader — Parse origin.olang
//!
//! Đọc và parse file origin.olang.
//! Dùng để: startup (rebuild Registry), crash recovery (replay).

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

use crate::molecular::MolecularChain;
use crate::writer::{MAGIC, VERSION, HEADER_SIZE, RT_NODE, RT_EDGE, RT_ALIAS};

// ─────────────────────────────────────────────────────────────────────────────
// Parsed records
// ─────────────────────────────────────────────────────────────────────────────

/// Node record đã parse.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ParsedNode {
    pub chain:       MolecularChain,
    pub layer:       u8,
    pub is_qr:       bool,
    pub timestamp:   i64,
    pub file_offset: u64,
}

/// Edge record đã parse.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ParsedEdge {
    pub from_hash:   u64,
    pub to_hash:     u64,
    pub edge_type:   u8,
    pub timestamp:   i64,
    pub file_offset: u64,
}

/// Alias record đã parse.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ParsedAlias {
    pub name:        String,
    pub chain_hash:  u64,
    pub timestamp:   i64,
    pub file_offset: u64,
}

/// Lỗi khi parse.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// File quá ngắn
    TooShort,
    /// Magic bytes sai
    BadMagic,
    /// Version không hỗ trợ
    UnsupportedVersion,
    /// Record type không biết
    UnknownRecordType(u8),
    /// Dữ liệu bị cắt
    Truncated,
    /// Chain bytes không hợp lệ
    InvalidChain,
}

// ─────────────────────────────────────────────────────────────────────────────
// OlangReader
// ─────────────────────────────────────────────────────────────────────────────

/// Parser cho origin.olang.
pub struct OlangReader<'a> {
    data:       &'a [u8],
    created_at: i64,
}

impl<'a> OlangReader<'a> {
    /// Parse header và tạo reader.
    pub fn new(data: &'a [u8]) -> Result<Self, ParseError> {
        if data.len() < HEADER_SIZE { return Err(ParseError::TooShort); }
        if data[0..4] != MAGIC   { return Err(ParseError::BadMagic); }
        if data[4] != VERSION       { return Err(ParseError::UnsupportedVersion); }

        let created_at = i64::from_le_bytes(data[5..13].try_into().unwrap());
        Ok(Self { data, created_at })
    }

    /// Timestamp khi file được tạo.
    pub fn created_at(&self) -> i64 { self.created_at }

    /// Parse tất cả records.
    pub fn parse_all(&self) -> Result<ParsedFile, ParseError> {
        let mut nodes:   Vec<ParsedNode>  = Vec::new();
        let mut edges:   Vec<ParsedEdge>  = Vec::new();
        let mut aliases: Vec<ParsedAlias> = Vec::new();

        let mut pos = HEADER_SIZE;

        while pos < self.data.len() {
            let record_offset = pos as u64;
            let rt = self.data[pos];
            pos += 1;

            match rt {
                RT_NODE => {
                    // [chain_len: u8][chain: N×5][layer: u8][is_qr: u8][ts: 8]
                    if pos + 1 > self.data.len() { return Err(ParseError::Truncated); }
                    let chain_len = self.data[pos] as usize;
                    pos += 1;

                    let chain_bytes_len = chain_len * 5;
                    if pos + chain_bytes_len + 1 + 1 + 8 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }

                    let chain_bytes = &self.data[pos..pos+chain_bytes_len];
                    let chain = MolecularChain::from_bytes(chain_bytes)
                        .ok_or(ParseError::InvalidChain)?;
                    pos += chain_bytes_len;

                    let layer = self.data[pos]; pos += 1;
                    let is_qr = self.data[pos] != 0; pos += 1;
                    let ts    = i64::from_le_bytes(self.data[pos..pos+8].try_into().unwrap());
                    pos += 8;

                    nodes.push(ParsedNode { chain, layer, is_qr, timestamp: ts, file_offset: record_offset });
                }

                RT_EDGE => {
                    // [from: 8][to: 8][type: 1][ts: 8] = 25 bytes
                    if pos + 25 > self.data.len() { return Err(ParseError::Truncated); }

                    let from = u64::from_le_bytes(self.data[pos..pos+8].try_into().unwrap()); pos += 8;
                    let to   = u64::from_le_bytes(self.data[pos..pos+8].try_into().unwrap()); pos += 8;
                    let et   = self.data[pos]; pos += 1;
                    let ts   = i64::from_le_bytes(self.data[pos..pos+8].try_into().unwrap()); pos += 8;

                    edges.push(ParsedEdge { from_hash: from, to_hash: to, edge_type: et, timestamp: ts, file_offset: record_offset });
                }

                RT_ALIAS => {
                    // [name_len: u8][name: N][hash: 8][ts: 8]
                    if pos + 1 > self.data.len() { return Err(ParseError::Truncated); }
                    let name_len = self.data[pos] as usize; pos += 1;

                    if pos + name_len + 8 + 8 > self.data.len() { return Err(ParseError::Truncated); }

                    let name_bytes = &self.data[pos..pos+name_len]; pos += name_len;
                    let name = String::from_utf8_lossy(name_bytes).into_owned();
                    let hash = u64::from_le_bytes(self.data[pos..pos+8].try_into().unwrap()); pos += 8;
                    let ts   = i64::from_le_bytes(self.data[pos..pos+8].try_into().unwrap()); pos += 8;

                    aliases.push(ParsedAlias { name, chain_hash: hash, timestamp: ts, file_offset: record_offset });
                }

                other => return Err(ParseError::UnknownRecordType(other)),
            }
        }

        Ok(ParsedFile { nodes, edges, aliases, created_at: self.created_at })
    }
}

/// Kết quả parse đầy đủ.
#[allow(missing_docs)]
pub struct ParsedFile {
    pub nodes:      Vec<ParsedNode>,
    pub edges:      Vec<ParsedEdge>,
    pub aliases:    Vec<ParsedAlias>,
    pub created_at: i64,
}

impl ParsedFile {
    /// Số nodes.
    pub fn node_count(&self) -> usize { self.nodes.len() }
    /// Số edges.
    pub fn edge_count(&self) -> usize { self.edges.len() }
    /// Số aliases.
    pub fn alias_count(&self) -> usize { self.aliases.len() }

    /// Nodes theo tầng.
    pub fn nodes_in_layer(&self, layer: u8) -> Vec<&ParsedNode> {
        self.nodes.iter().filter(|n| n.layer == layer).collect()
    }

    /// QR nodes.
    pub fn qr_nodes(&self) -> Vec<&ParsedNode> {
        self.nodes.iter().filter(|n| n.is_qr).collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer::OlangWriter;
    use crate::encoder::encode_codepoint;

    fn skip_if_empty() -> bool { ucd::table_len() == 0 }

    fn roundtrip(write: impl FnOnce(&mut OlangWriter)) -> ParsedFile {
        let mut w = OlangWriter::new(42);
        write(&mut w);
        let bytes = w.into_bytes();
        let reader = OlangReader::new(&bytes).expect("parse header");
        reader.parse_all().expect("parse all")
    }

    #[test]
    fn reader_bad_magic() {
        let bad = [0x00u8, 0x01, 0x02, 0x03, 0x03, 0,0,0,0,0,0,0,0];
        let result = OlangReader::new(&bad);
        assert!(matches!(result, Err(ParseError::BadMagic)));
    }

    #[test]
    fn reader_too_short() {
        let result = OlangReader::new(&[0u8; 5]);
        assert!(matches!(result, Err(ParseError::TooShort)));
    }

    #[test]
    fn reader_empty_file() {
        let w = OlangWriter::new(1000);
        let bytes = w.into_bytes();
        let reader = OlangReader::new(&bytes).unwrap();
        assert_eq!(reader.created_at(), 1000);
        let pf = reader.parse_all().unwrap();
        assert_eq!(pf.node_count(), 0);
        assert_eq!(pf.edge_count(), 0);
        assert_eq!(pf.alias_count(), 0);
    }

    #[test]
    fn roundtrip_one_node() {
        if skip_if_empty() { return; }
        let chain = encode_codepoint(0x1F525); // 🔥
        let pf = roundtrip(|w| {
            w.append_node(&chain, 0, false, 1000).unwrap();
        });

        assert_eq!(pf.node_count(), 1);
        let n = &pf.nodes[0];
        assert_eq!(n.chain, chain, "Chain roundtrip đúng");
        assert_eq!(n.layer, 0);
        assert_eq!(n.is_qr, false);
        assert_eq!(n.timestamp, 1000);
    }

    #[test]
    fn roundtrip_qr_node() {
        if skip_if_empty() { return; }
        let chain = encode_codepoint(0x1F4A7); // 💧
        let pf = roundtrip(|w| {
            w.append_node(&chain, 2, true, 5000).unwrap();
        });

        let n = &pf.nodes[0];
        assert_eq!(n.is_qr, true, "QR flag preserve");
        assert_eq!(n.layer, 2);
    }

    #[test]
    fn roundtrip_edge() {
        let pf = roundtrip(|w| {
            w.append_edge(0xABCD_1234, 0xEF56_7890, 0x01, 2000);
        });

        assert_eq!(pf.edge_count(), 1);
        let e = &pf.edges[0];
        assert_eq!(e.from_hash, 0xABCD_1234);
        assert_eq!(e.to_hash,   0xEF56_7890);
        assert_eq!(e.edge_type, 0x01);
        assert_eq!(e.timestamp, 2000);
    }

    #[test]
    fn roundtrip_alias() {
        let pf = roundtrip(|w| {
            w.append_alias("lửa", 0xFEED_BEEF, 3000).unwrap();
        });

        assert_eq!(pf.alias_count(), 1);
        let a = &pf.aliases[0];
        assert_eq!(a.name, "lửa");
        assert_eq!(a.chain_hash, 0xFEED_BEEF);
        assert_eq!(a.timestamp, 3000);
    }

    #[test]
    fn roundtrip_mixed_records() {
        if skip_if_empty() { return; }
        let chain = encode_codepoint(0x1F525);
        let hash  = chain.chain_hash();

        let pf = roundtrip(|w| {
            w.append_node(&chain, 0, false, 1000).unwrap();
            w.append_alias("fire", hash, 1001).unwrap();
            w.append_edge(hash, 0xDEAD, 0x06, 1002);
        });

        assert_eq!(pf.node_count(), 1);
        assert_eq!(pf.alias_count(), 1);
        assert_eq!(pf.edge_count(), 1);
    }

    #[test]
    fn roundtrip_many_nodes() {
        if skip_if_empty() { return; }
        let cps = [0x1F525u32, 0x1F4A7, 0x2744, 0x25CF, 0x2208];

        let pf = roundtrip(|w| {
            for (i, &cp) in cps.iter().enumerate() {
                let chain = encode_codepoint(cp);
                w.append_node(&chain, (i % 3) as u8, i % 2 == 0, i as i64 * 1000).unwrap();
            }
        });

        assert_eq!(pf.node_count(), cps.len());
        // Verify thứ tự giữ nguyên (append-only)
        for (i, &cp) in cps.iter().enumerate() {
            let expected = encode_codepoint(cp);
            assert_eq!(pf.nodes[i].chain, expected,
                "Node[{}] chain phải đúng", i);
        }
    }

    #[test]
    fn file_offsets_increasing() {
        if skip_if_empty() { return; }
        let pf = roundtrip(|w| {
            w.append_node(&encode_codepoint(0x1F525), 0, false, 1000).unwrap();
            w.append_node(&encode_codepoint(0x1F4A7), 0, false, 2000).unwrap();
            w.append_edge(0x01, 0x02, 0x01, 3000);
        });

        // Offsets phải tăng dần
        assert!(pf.nodes[0].file_offset < pf.nodes[1].file_offset,
            "Node offsets tăng dần");
        assert!(pf.nodes[1].file_offset < pf.edges[0].file_offset,
            "Edge offset sau node offset");
    }

    #[test]
    fn crash_recovery_partial_write() {
        if skip_if_empty() { return; }
        // Giả lập crash: file bị cắt giữa chừng
        let mut w = OlangWriter::new(0);
        w.append_node(&encode_codepoint(0x1F525), 0, false, 1000).unwrap();
        w.append_node(&encode_codepoint(0x1F4A7), 0, false, 2000).unwrap();

        let full_bytes = w.into_bytes();

        // Cắt file đi 10 bytes (giả lập crash)
        let truncated = &full_bytes[..full_bytes.len() - 10];
        let reader = OlangReader::new(truncated).unwrap();
        let result = reader.parse_all();

        // Có thể parse được phần hợp lệ hoặc trả về Truncated error
        match result {
            Ok(pf) => {
                // Nếu parse được → ít nhất node đầu tiên phải đúng
                assert!(pf.node_count() >= 1, "Ít nhất 1 node parse được");
            }
            Err(ParseError::Truncated) => {
                // OK — crash detected, sẽ replay log để recover
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
