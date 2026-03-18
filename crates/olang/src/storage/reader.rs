//! # reader — Parse origin.olang
//!
//! Đọc và parse file origin.olang.
//! Dùng để: startup (rebuild Registry), crash recovery (replay).

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::molecular::MolecularChain;
use crate::writer::{
    HEADER_SIZE, MAGIC, RT_ALIAS, RT_AMEND, RT_CURVE, RT_EDGE, RT_HEBBIAN, RT_KNOWTREE, RT_NODE,
    RT_NODE_KIND, RT_SLIM_KNOWTREE, RT_STM, VERSION, VERSION_V03, VERSION_V04,
};

/// Read a little-endian u64 from a slice at offset. Caller must ensure pos+8 ≤ data.len().
#[inline]
fn read_u64_le(data: &[u8], pos: usize) -> u64 {
    let bytes: [u8; 8] = [
        data[pos], data[pos+1], data[pos+2], data[pos+3],
        data[pos+4], data[pos+5], data[pos+6], data[pos+7],
    ];
    u64::from_le_bytes(bytes)
}

/// Read a little-endian i64 from a slice at offset. Caller must ensure pos+8 ≤ data.len().
#[inline]
fn read_i64_le(data: &[u8], pos: usize) -> i64 {
    read_u64_le(data, pos) as i64
}

/// Read a little-endian f32 from a slice at offset. Caller must ensure pos+4 ≤ data.len().
#[inline]
fn read_f32_le(data: &[u8], pos: usize) -> f32 {
    f32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
}

/// Read a little-endian u32 from a slice at offset. Caller must ensure pos+4 ≤ data.len().
#[inline]
fn read_u32_le(data: &[u8], pos: usize) -> u32 {
    u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
}

/// Read a little-endian u16 from a slice at offset. Caller must ensure pos+2 ≤ data.len().
#[inline]
fn read_u16_le(data: &[u8], pos: usize) -> u16 {
    u16::from_le_bytes([data[pos], data[pos + 1]])
}

// ─────────────────────────────────────────────────────────────────────────────
// Parsed records
// ─────────────────────────────────────────────────────────────────────────────

/// Node record đã parse.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ParsedNode {
    pub chain: MolecularChain,
    pub layer: u8,
    pub is_qr: bool,
    pub timestamp: i64,
    pub file_offset: u64,
}

/// Edge record đã parse.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ParsedEdge {
    pub from_hash: u64,
    pub to_hash: u64,
    pub edge_type: u8,
    pub timestamp: i64,
    pub file_offset: u64,
}

/// Alias record đã parse.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ParsedAlias {
    pub name: String,
    pub chain_hash: u64,
    pub timestamp: i64,
    pub file_offset: u64,
}

/// NodeKind record đã parse — gán NodeKind cho node.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ParsedNodeKind {
    /// FNV-1a hash của chain được gán kind.
    pub chain_hash: u64,
    /// NodeKind byte (0=Alphabet, 1=Knowledge, ..., 9=System).
    pub kind: u8,
    pub timestamp: i64,
    pub file_offset: u64,
}

/// Amendment record đã parse — append-only rollback.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ParsedAmend {
    /// Offset của record bị supersede.
    pub target_offset: u64,
    /// Lý do amend.
    pub reason: String,
    pub timestamp: i64,
    pub file_offset: u64,
}

/// STM Observation record đã parse.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ParsedStm {
    pub chain_hash: u64,
    pub valence: f32,
    pub arousal: f32,
    pub dominance: f32,
    pub intensity: f32,
    pub fire_count: u32,
    pub maturity: u8,
    pub layer: u8,
    pub timestamp: i64,
    pub file_offset: u64,
}

/// HebbianLink record đã parse.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ParsedHebbian {
    pub from_hash: u64,
    pub to_hash: u64,
    pub weight: u8,
    pub fire_count: u16,
    pub timestamp: i64,
    pub file_offset: u64,
}

/// KnowTree CompactNode record đã parse.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ParsedKnowTree {
    pub data: Vec<u8>,
    pub timestamp: i64,
    pub file_offset: u64,
}

/// SlimKnowTree node record đã parse — spec-compliant format.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ParsedSlimKnowTree {
    pub hash: u64,
    pub tagged: Vec<u8>,
    pub layer: u8,
    pub timestamp: i64,
    pub file_offset: u64,
}

/// ConversationCurve turn record đã parse.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ParsedCurve {
    pub valence: f32,
    pub fx_dn: f32,
    pub timestamp: i64,
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
    data: &'a [u8],
    created_at: i64,
    version: u8,
}

impl<'a> OlangReader<'a> {
    /// Parse header và tạo reader.
    pub fn new(data: &'a [u8]) -> Result<Self, ParseError> {
        if data.len() < HEADER_SIZE {
            return Err(ParseError::TooShort);
        }
        if data[0..4] != MAGIC {
            return Err(ParseError::BadMagic);
        }
        let version = data[4];
        // Accept v0.03, v0.04, and v0.05 (backward compatible)
        if version != VERSION && version != VERSION_V04 && version != VERSION_V03 {
            return Err(ParseError::UnsupportedVersion);
        }

        let created_at = read_i64_le(data, 5);
        Ok(Self {
            data,
            created_at,
            version,
        })
    }

    /// Timestamp khi file được tạo.
    pub fn created_at(&self) -> i64 {
        self.created_at
    }

    /// Parse tất cả records (v0.03 + v0.04 + v0.05 compatible).
    pub fn parse_all(&self) -> Result<ParsedFile, ParseError> {
        let mut nodes: Vec<ParsedNode> = Vec::new();
        let mut edges: Vec<ParsedEdge> = Vec::new();
        let mut aliases: Vec<ParsedAlias> = Vec::new();
        let mut amends: Vec<ParsedAmend> = Vec::new();
        let mut node_kinds: Vec<ParsedNodeKind> = Vec::new();
        let mut stm_records: Vec<ParsedStm> = Vec::new();
        let mut hebbian_records: Vec<ParsedHebbian> = Vec::new();
        let mut knowtree_records: Vec<ParsedKnowTree> = Vec::new();
        let mut slim_knowtree_records: Vec<ParsedSlimKnowTree> = Vec::new();
        let mut curve_records: Vec<ParsedCurve> = Vec::new();

        let mut pos = HEADER_SIZE;

        while pos < self.data.len() {
            let record_offset = pos as u64;
            let rt = self.data[pos];
            pos += 1;

            match rt {
                RT_NODE => {
                    if pos + 1 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }

                    let chain = if self.version >= VERSION {
                        // v0.05: tagged format [mol_count][mol_1_tagged]...
                        let tagged_start = pos;
                        let chain = MolecularChain::from_tagged_bytes(&self.data[pos..])
                            .ok_or(ParseError::InvalidChain)?;
                        // Advance past: 1 (mol_count) + sum of tagged molecule sizes
                        pos = tagged_start + chain.tagged_byte_size();
                    chain
                    } else {
                        // v0.03-v0.04: legacy fixed [chain_len: u8][chain: N×5]
                        let chain_len = self.data[pos] as usize;
                        pos += 1;
                        let chain_bytes_len = chain_len * 5;
                        if pos + chain_bytes_len + 1 + 1 + 8 > self.data.len() {
                            return Err(ParseError::Truncated);
                        }
                        let chain_bytes = &self.data[pos..pos + chain_bytes_len];
                        let chain = MolecularChain::from_bytes(chain_bytes)
                            .ok_or(ParseError::InvalidChain)?;
                        pos += chain_bytes_len;
                        chain
                    };

                    if pos + 1 + 1 + 8 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }
                    let layer = self.data[pos];
                    pos += 1;
                    let is_qr = self.data[pos] != 0;
                    pos += 1;
                    let ts = read_i64_le(self.data, pos);
                    pos += 8;

                    nodes.push(ParsedNode {
                        chain,
                        layer,
                        is_qr,
                        timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_EDGE => {
                    // [from: 8][to: 8][type: 1][ts: 8] = 25 bytes
                    if pos + 25 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }

                    let from = read_u64_le(self.data, pos);
                    pos += 8;
                    let to = read_u64_le(self.data, pos);
                    pos += 8;
                    let et = self.data[pos];
                    pos += 1;
                    let ts = read_i64_le(self.data, pos);
                    pos += 8;

                    edges.push(ParsedEdge {
                        from_hash: from,
                        to_hash: to,
                        edge_type: et,
                        timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_ALIAS => {
                    // [name_len: u8][name: N][hash: 8][ts: 8]
                    if pos + 1 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }
                    let name_len = self.data[pos] as usize;
                    pos += 1;

                    if pos + name_len + 8 + 8 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }

                    let name_bytes = &self.data[pos..pos + name_len];
                    pos += name_len;
                    let name = String::from_utf8_lossy(name_bytes).into_owned();
                    let hash = read_u64_le(self.data, pos);
                    pos += 8;
                    let ts = read_i64_le(self.data, pos);
                    pos += 8;

                    aliases.push(ParsedAlias {
                        name,
                        chain_hash: hash,
                        timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_AMEND => {
                    // [target_offset: 8][reason_len: u8][reason: N][ts: 8]
                    if pos + 8 + 1 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }
                    let target = read_u64_le(self.data, pos);
                    pos += 8;
                    let reason_len = self.data[pos] as usize;
                    pos += 1;

                    if pos + reason_len + 8 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }

                    let reason_bytes = &self.data[pos..pos + reason_len];
                    pos += reason_len;
                    let reason = String::from_utf8_lossy(reason_bytes).into_owned();
                    let ts = read_i64_le(self.data, pos);
                    pos += 8;

                    amends.push(ParsedAmend {
                        target_offset: target,
                        reason,
                        timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_NODE_KIND => {
                    // [chain_hash: 8][kind: 1][ts: 8] = 17 bytes
                    if pos + 8 + 1 + 8 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }

                    let hash = read_u64_le(self.data, pos);
                    pos += 8;
                    let kind = self.data[pos];
                    pos += 1;
                    let ts = read_i64_le(self.data, pos);
                    pos += 8;

                    node_kinds.push(ParsedNodeKind {
                        chain_hash: hash,
                        kind,
                        timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_STM => {
                    // [chain_hash:8][v:4][a:4][d:4][i:4][fire:4][mat:1][layer:1][ts:8] = 38
                    if pos + 38 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }
                    let hash = read_u64_le(self.data, pos); pos += 8;
                    let v = read_f32_le(self.data, pos); pos += 4;
                    let a = read_f32_le(self.data, pos); pos += 4;
                    let d = read_f32_le(self.data, pos); pos += 4;
                    let i = read_f32_le(self.data, pos); pos += 4;
                    let fc = read_u32_le(self.data, pos); pos += 4;
                    let mat = self.data[pos]; pos += 1;
                    let layer = self.data[pos]; pos += 1;
                    let ts = read_i64_le(self.data, pos); pos += 8;
                    stm_records.push(ParsedStm {
                        chain_hash: hash, valence: v, arousal: a,
                        dominance: d, intensity: i, fire_count: fc,
                        maturity: mat, layer, timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_HEBBIAN => {
                    // [from:8][to:8][weight:1][fire:2][ts:8] = 27
                    if pos + 27 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }
                    let from = read_u64_le(self.data, pos); pos += 8;
                    let to = read_u64_le(self.data, pos); pos += 8;
                    let w = self.data[pos]; pos += 1;
                    let fc = read_u16_le(self.data, pos); pos += 2;
                    let ts = read_i64_le(self.data, pos); pos += 8;
                    hebbian_records.push(ParsedHebbian {
                        from_hash: from, to_hash: to, weight: w,
                        fire_count: fc, timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_KNOWTREE => {
                    // [data_len:2][data:N][ts:8]
                    if pos + 2 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }
                    let data_len = read_u16_le(self.data, pos) as usize; pos += 2;
                    if pos + data_len + 8 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }
                    let data = self.data[pos..pos + data_len].to_vec(); pos += data_len;
                    let ts = read_i64_le(self.data, pos); pos += 8;
                    knowtree_records.push(ParsedKnowTree {
                        data, timestamp: ts, file_offset: record_offset,
                    });
                }

                RT_SLIM_KNOWTREE => {
                    // [hash:8][tagged_len:1][tagged:1-6][layer:1][ts:8]
                    if pos + 8 + 1 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }
                    let hash = read_u64_le(self.data, pos); pos += 8;
                    let tagged_len = self.data[pos] as usize; pos += 1;
                    if tagged_len == 0 || tagged_len > 32 || pos + tagged_len + 1 + 8 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }
                    let tagged = self.data[pos..pos + tagged_len].to_vec(); pos += tagged_len;
                    let layer = self.data[pos]; pos += 1;
                    let ts = read_i64_le(self.data, pos); pos += 8;
                    slim_knowtree_records.push(ParsedSlimKnowTree {
                        hash, tagged, layer, timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_CURVE => {
                    // [valence:4][fx_dn:4][ts:8] = 16
                    if pos + 16 > self.data.len() {
                        return Err(ParseError::Truncated);
                    }
                    let v = read_f32_le(self.data, pos); pos += 4;
                    let dn = read_f32_le(self.data, pos); pos += 4;
                    let ts = read_i64_le(self.data, pos); pos += 8;
                    curve_records.push(ParsedCurve {
                        valence: v, fx_dn: dn, timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                other => return Err(ParseError::UnknownRecordType(other)),
            }
        }

        // Build amended_offsets set for filtering
        let amended_offsets: alloc::collections::BTreeSet<u64> =
            amends.iter().map(|a| a.target_offset).collect();

        Ok(ParsedFile {
            nodes,
            edges,
            aliases,
            amends,
            node_kinds,
            stm_records,
            hebbian_records,
            knowtree_records,
            slim_knowtree_records,
            curve_records,
            amended_offsets,
            created_at: self.created_at,
        })
    }
}

/// Kết quả parse + recovery info.
#[derive(Debug, Clone)]
pub struct RecoveryInfo {
    /// Số records đã parse thành công.
    pub records_recovered: usize,
    /// Byte offset nơi parse dừng lại.
    pub last_good_offset: usize,
    /// Tổng bytes trong file.
    pub total_bytes: usize,
    /// Lỗi gặp phải (nếu có).
    pub error: Option<ParseError>,
}

impl<'a> OlangReader<'a> {
    /// Parse best-effort — khôi phục bao nhiêu records có thể.
    ///
    /// Không trả Err: luôn trả ParsedFile + RecoveryInfo.
    /// Dùng khi boot sau crash hoặc disk corruption.
    /// UnknownRecordType → dừng parse (vì không biết record length).
    pub fn parse_recoverable(&self) -> (ParsedFile, RecoveryInfo) {
        let mut nodes: Vec<ParsedNode> = Vec::new();
        let mut edges: Vec<ParsedEdge> = Vec::new();
        let mut aliases: Vec<ParsedAlias> = Vec::new();
        let mut amends: Vec<ParsedAmend> = Vec::new();
        let mut node_kinds: Vec<ParsedNodeKind> = Vec::new();
        let mut stm_records: Vec<ParsedStm> = Vec::new();
        let mut hebbian_records: Vec<ParsedHebbian> = Vec::new();
        let mut knowtree_records: Vec<ParsedKnowTree> = Vec::new();
        let mut slim_knowtree_records: Vec<ParsedSlimKnowTree> = Vec::new();
        let mut curve_records: Vec<ParsedCurve> = Vec::new();

        let mut pos = HEADER_SIZE;
        let mut error = None;

        while pos < self.data.len() {
            let record_offset = pos as u64;
            let rt = self.data[pos];
            pos += 1;

            match rt {
                RT_NODE => {
                    if pos + 1 > self.data.len() {
                        error = Some(ParseError::Truncated);
                        break;
                    }

                    let chain_result = if self.version >= VERSION {
                        // v0.05: tagged format
                        match MolecularChain::from_tagged_bytes(&self.data[pos..]) {
                            Some(chain) => {
                                let size = chain.tagged_byte_size();
                                pos += size;
                                Some(chain)
                            }
                            None => None,
                        }
                    } else {
                        // v0.03-v0.04: legacy fixed format
                        let chain_len = self.data[pos] as usize;
                        pos += 1;
                        let chain_bytes_len = chain_len * 5;
                        if pos + chain_bytes_len + 1 + 1 + 8 > self.data.len() {
                            error = Some(ParseError::Truncated);
                            break;
                        }
                        let chain_bytes = &self.data[pos..pos + chain_bytes_len];
                        match MolecularChain::from_bytes(chain_bytes) {
                            Some(chain) => {
                                pos += chain_bytes_len;
                                Some(chain)
                            }
                            None => None,
                        }
                    };

                    match chain_result {
                        Some(chain) => {
                            if pos + 1 + 1 + 8 > self.data.len() {
                                error = Some(ParseError::Truncated);
                                break;
                            }
                            let layer = self.data[pos];
                            pos += 1;
                            let is_qr = self.data[pos] != 0;
                            pos += 1;
                            let ts =
                                read_i64_le(self.data, pos);
                            pos += 8;
                            nodes.push(ParsedNode {
                                chain,
                                layer,
                                is_qr,
                                timestamp: ts,
                                file_offset: record_offset,
                            });
                        }
                        None => {
                            error = Some(ParseError::InvalidChain);
                            break;
                        }
                    }
                }

                RT_EDGE => {
                    if pos + 25 > self.data.len() {
                        error = Some(ParseError::Truncated);
                        break;
                    }

                    let from = read_u64_le(self.data, pos);
                    pos += 8;
                    let to = read_u64_le(self.data, pos);
                    pos += 8;
                    let et = self.data[pos];
                    pos += 1;
                    let ts = read_i64_le(self.data, pos);
                    pos += 8;
                    edges.push(ParsedEdge {
                        from_hash: from,
                        to_hash: to,
                        edge_type: et,
                        timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_ALIAS => {
                    if pos + 1 > self.data.len() {
                        error = Some(ParseError::Truncated);
                        break;
                    }
                    let name_len = self.data[pos] as usize;
                    pos += 1;

                    if pos + name_len + 8 + 8 > self.data.len() {
                        error = Some(ParseError::Truncated);
                        break;
                    }

                    let name_bytes = &self.data[pos..pos + name_len];
                    pos += name_len;
                    let name = String::from_utf8_lossy(name_bytes).into_owned();
                    let hash = read_u64_le(self.data, pos);
                    pos += 8;
                    let ts = read_i64_le(self.data, pos);
                    pos += 8;
                    aliases.push(ParsedAlias {
                        name,
                        chain_hash: hash,
                        timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_AMEND => {
                    if pos + 8 + 1 > self.data.len() {
                        error = Some(ParseError::Truncated);
                        break;
                    }
                    let target = read_u64_le(self.data, pos);
                    pos += 8;
                    let reason_len = self.data[pos] as usize;
                    pos += 1;
                    if pos + reason_len + 8 > self.data.len() {
                        error = Some(ParseError::Truncated);
                        break;
                    }
                    let reason_bytes = &self.data[pos..pos + reason_len];
                    pos += reason_len;
                    let reason = String::from_utf8_lossy(reason_bytes).into_owned();
                    let ts = read_i64_le(self.data, pos);
                    pos += 8;
                    amends.push(ParsedAmend {
                        target_offset: target,
                        reason,
                        timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_NODE_KIND => {
                    if pos + 8 + 1 + 8 > self.data.len() {
                        error = Some(ParseError::Truncated);
                        break;
                    }
                    let hash = read_u64_le(self.data, pos);
                    pos += 8;
                    let kind = self.data[pos];
                    pos += 1;
                    let ts = read_i64_le(self.data, pos);
                    pos += 8;
                    node_kinds.push(ParsedNodeKind {
                        chain_hash: hash,
                        kind,
                        timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_STM => {
                    if pos + 38 > self.data.len() {
                        error = Some(ParseError::Truncated);
                        break;
                    }
                    let hash = read_u64_le(self.data, pos); pos += 8;
                    let v = read_f32_le(self.data, pos); pos += 4;
                    let a = read_f32_le(self.data, pos); pos += 4;
                    let d = read_f32_le(self.data, pos); pos += 4;
                    let i = read_f32_le(self.data, pos); pos += 4;
                    let fc = read_u32_le(self.data, pos); pos += 4;
                    let mat = self.data[pos]; pos += 1;
                    let layer = self.data[pos]; pos += 1;
                    let ts = read_i64_le(self.data, pos); pos += 8;
                    stm_records.push(ParsedStm {
                        chain_hash: hash, valence: v, arousal: a,
                        dominance: d, intensity: i, fire_count: fc,
                        maturity: mat, layer, timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_HEBBIAN => {
                    if pos + 27 > self.data.len() {
                        error = Some(ParseError::Truncated);
                        break;
                    }
                    let from = read_u64_le(self.data, pos); pos += 8;
                    let to = read_u64_le(self.data, pos); pos += 8;
                    let w = self.data[pos]; pos += 1;
                    let fc = read_u16_le(self.data, pos); pos += 2;
                    let ts = read_i64_le(self.data, pos); pos += 8;
                    hebbian_records.push(ParsedHebbian {
                        from_hash: from, to_hash: to, weight: w,
                        fire_count: fc, timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_KNOWTREE => {
                    if pos + 2 > self.data.len() {
                        error = Some(ParseError::Truncated);
                        break;
                    }
                    let data_len = read_u16_le(self.data, pos) as usize; pos += 2;
                    if pos + data_len + 8 > self.data.len() {
                        error = Some(ParseError::Truncated);
                        break;
                    }
                    let data = self.data[pos..pos + data_len].to_vec(); pos += data_len;
                    let ts = read_i64_le(self.data, pos); pos += 8;
                    knowtree_records.push(ParsedKnowTree {
                        data, timestamp: ts, file_offset: record_offset,
                    });
                }

                RT_SLIM_KNOWTREE => {
                    // [hash:8][tagged_len:1][tagged:1-6][layer:1][ts:8]
                    if pos + 8 + 1 > self.data.len() {
                        error = Some(ParseError::Truncated);
                        break;
                    }
                    let hash = read_u64_le(self.data, pos); pos += 8;
                    let tagged_len = self.data[pos] as usize; pos += 1;
                    if tagged_len == 0 || tagged_len > 32 || pos + tagged_len + 1 + 8 > self.data.len() {
                        error = Some(ParseError::Truncated);
                        break;
                    }
                    let tagged = self.data[pos..pos + tagged_len].to_vec(); pos += tagged_len;
                    let layer = self.data[pos]; pos += 1;
                    let ts = read_i64_le(self.data, pos); pos += 8;
                    slim_knowtree_records.push(ParsedSlimKnowTree {
                        hash, tagged, layer, timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                RT_CURVE => {
                    if pos + 16 > self.data.len() {
                        error = Some(ParseError::Truncated);
                        break;
                    }
                    let v = read_f32_le(self.data, pos); pos += 4;
                    let dn = read_f32_le(self.data, pos); pos += 4;
                    let ts = read_i64_le(self.data, pos); pos += 8;
                    curve_records.push(ParsedCurve {
                        valence: v, fx_dn: dn, timestamp: ts,
                        file_offset: record_offset,
                    });
                }

                other => {
                    error = Some(ParseError::UnknownRecordType(other));
                    break;
                }
            }
        }

        let records_recovered = nodes.len() + edges.len() + aliases.len() + amends.len()
            + node_kinds.len() + stm_records.len() + hebbian_records.len()
            + knowtree_records.len() + slim_knowtree_records.len() + curve_records.len();
        let amended_offsets: alloc::collections::BTreeSet<u64> =
            amends.iter().map(|a| a.target_offset).collect();
        let file = ParsedFile {
            nodes,
            edges,
            aliases,
            amends,
            node_kinds,
            stm_records,
            hebbian_records,
            knowtree_records,
            slim_knowtree_records,
            curve_records,
            amended_offsets,
            created_at: self.created_at,
        };
        let info = RecoveryInfo {
            records_recovered,
            last_good_offset: pos,
            total_bytes: self.data.len(),
            error,
        };
        (file, info)
    }
}

/// Kết quả parse đầy đủ.
#[allow(missing_docs)]
pub struct ParsedFile {
    /// Node records.
    pub nodes: Vec<ParsedNode>,
    /// Edge records.
    pub edges: Vec<ParsedEdge>,
    /// Alias records.
    pub aliases: Vec<ParsedAlias>,
    /// Amendment records (v0.04+).
    pub amends: Vec<ParsedAmend>,
    /// NodeKind records (v0.05+) — gán NodeKind cho nodes.
    pub node_kinds: Vec<ParsedNodeKind>,
    /// STM Observation records — restore ShortTermMemory on boot.
    pub stm_records: Vec<ParsedStm>,
    /// HebbianLink records — restore Silk learned weights on boot.
    pub hebbian_records: Vec<ParsedHebbian>,
    /// KnowTree CompactNode records — restore L2+ knowledge on boot (legacy 0x08).
    pub knowtree_records: Vec<ParsedKnowTree>,
    /// SlimKnowTree node records — spec-compliant format (0x0A).
    pub slim_knowtree_records: Vec<ParsedSlimKnowTree>,
    /// ConversationCurve turn records — replay to reconstruct curve on boot.
    pub curve_records: Vec<ParsedCurve>,
    /// Offsets đã bị amend — dùng để filter records.
    pub amended_offsets: alloc::collections::BTreeSet<u64>,
    /// Timestamp khi file được tạo.
    pub created_at: i64,
}

impl ParsedFile {
    /// Số nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    /// Số edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
    /// Số aliases.
    pub fn alias_count(&self) -> usize {
        self.aliases.len()
    }

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
    use crate::encoder::encode_codepoint;
    use crate::writer::OlangWriter;


    fn roundtrip(write: impl FnOnce(&mut OlangWriter)) -> ParsedFile {
        let mut w = OlangWriter::new(42);
        write(&mut w);
        let bytes = w.into_bytes();
        let reader = OlangReader::new(&bytes).expect("parse header");
        reader.parse_all().expect("parse all")
    }

    #[test]
    fn reader_bad_magic() {
        let bad = [0x00u8, 0x01, 0x02, 0x03, 0x03, 0, 0, 0, 0, 0, 0, 0, 0];
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
        let chain = encode_codepoint(0x1F525); // 🔥
        let pf = roundtrip(|w| {
            w.append_node(&chain, 0, false, 1000).unwrap();
        });

        assert_eq!(pf.node_count(), 1);
        let n = &pf.nodes[0];
        assert_eq!(n.chain, chain, "Chain roundtrip đúng");
        assert_eq!(n.layer, 0);
        assert!(!n.is_qr);
        assert_eq!(n.timestamp, 1000);
    }

    #[test]
    fn roundtrip_qr_node() {
        let chain = encode_codepoint(0x1F4A7); // 💧
        let pf = roundtrip(|w| {
            w.append_node(&chain, 2, true, 5000).unwrap();
        });

        let n = &pf.nodes[0];
        assert!(n.is_qr, "QR flag preserve");
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
        assert_eq!(e.to_hash, 0xEF56_7890);
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
        let chain = encode_codepoint(0x1F525);
        let hash = chain.chain_hash();

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
        let cps = [0x1F525u32, 0x1F4A7, 0x2744, 0x25CF, 0x2208];

        let pf = roundtrip(|w| {
            for (i, &cp) in cps.iter().enumerate() {
                let chain = encode_codepoint(cp);
                w.append_node(&chain, (i % 3) as u8, i % 2 == 0, i as i64 * 1000)
                    .unwrap();
            }
        });

        assert_eq!(pf.node_count(), cps.len());
        // Verify thứ tự giữ nguyên (append-only)
        for (i, &cp) in cps.iter().enumerate() {
            let expected = encode_codepoint(cp);
            assert_eq!(pf.nodes[i].chain, expected, "Node[{}] chain phải đúng", i);
        }
    }

    #[test]
    fn file_offsets_increasing() {
        let pf = roundtrip(|w| {
            w.append_node(&encode_codepoint(0x1F525), 0, false, 1000)
                .unwrap();
            w.append_node(&encode_codepoint(0x1F4A7), 0, false, 2000)
                .unwrap();
            w.append_edge(0x01, 0x02, 0x01, 3000);
        });

        // Offsets phải tăng dần
        assert!(
            pf.nodes[0].file_offset < pf.nodes[1].file_offset,
            "Node offsets tăng dần"
        );
        assert!(
            pf.nodes[1].file_offset < pf.edges[0].file_offset,
            "Edge offset sau node offset"
        );
    }

    #[test]
    fn crash_recovery_partial_write() {
        // Giả lập crash: file bị cắt giữa chừng
        let mut w = OlangWriter::new(0);
        w.append_node(&encode_codepoint(0x1F525), 0, false, 1000)
            .unwrap();
        w.append_node(&encode_codepoint(0x1F4A7), 0, false, 2000)
            .unwrap();

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

    // ── parse_recoverable — best-effort crash recovery ────────────────────

    #[test]
    fn recoverable_full_file() {
        let chain = encode_codepoint(0x1F525);
        let mut w = OlangWriter::new(0);
        w.append_node(&chain, 0, false, 1000).unwrap();
        w.append_edge(0xABCD, 0xEF12, 0x01, 2000);
        let bytes = w.into_bytes();

        let reader = OlangReader::new(&bytes).unwrap();
        let (pf, info) = reader.parse_recoverable();

        assert_eq!(pf.node_count(), 1);
        assert_eq!(pf.edge_count(), 1);
        assert_eq!(info.records_recovered, 2);
        assert!(info.error.is_none(), "No error for complete file");
    }

    #[test]
    fn recoverable_truncated_recovers_first() {
        let mut w = OlangWriter::new(0);
        w.append_node(&encode_codepoint(0x1F525), 0, false, 1000)
            .unwrap();
        w.append_node(&encode_codepoint(0x1F4A7), 0, false, 2000)
            .unwrap();
        let full = w.into_bytes();

        // Truncate last record
        let truncated = &full[..full.len() - 5];
        let reader = OlangReader::new(truncated).unwrap();
        let (pf, info) = reader.parse_recoverable();

        // First node should be recovered
        assert!(pf.node_count() >= 1, "At least 1 node recovered");
        assert!(
            matches!(info.error, Some(ParseError::Truncated)),
            "Truncated error detected"
        );
        assert!(
            info.last_good_offset < info.total_bytes,
            "Parse stopped before end: {} < {}",
            info.last_good_offset,
            info.total_bytes
        );
    }

    #[test]
    fn recoverable_unknown_record_type() {
        let mut w = OlangWriter::new(0);
        w.append_node(&encode_codepoint(0x1F525), 0, false, 1000)
            .unwrap();
        let mut bytes = w.into_bytes();

        // Append garbage record type
        bytes.push(0xFE); // unknown record type
        bytes.extend_from_slice(&[0u8; 20]);

        let reader = OlangReader::new(&bytes).unwrap();
        let (pf, info) = reader.parse_recoverable();

        assert_eq!(pf.node_count(), 1, "Valid node recovered before corruption");
        assert!(matches!(
            info.error,
            Some(ParseError::UnknownRecordType(0xFE))
        ));
    }

    // ── New record types roundtrip tests ──────────────────────────────────

    #[test]
    fn roundtrip_stm_record() {
        let pf = roundtrip(|w| {
            w.append_stm(0xDEAD_BEEF, -0.5, 0.7, 0.3, 0.8, 5, 0x01, 1, 4000);
        });
        assert_eq!(pf.stm_records.len(), 1);
        let s = &pf.stm_records[0];
        assert_eq!(s.chain_hash, 0xDEAD_BEEF);
        assert!((s.valence - (-0.5)).abs() < 0.001);
        assert!((s.arousal - 0.7).abs() < 0.001);
        assert!((s.dominance - 0.3).abs() < 0.001);
        assert!((s.intensity - 0.8).abs() < 0.001);
        assert_eq!(s.fire_count, 5);
        assert_eq!(s.maturity, 0x01);
        assert_eq!(s.layer, 1);
        assert_eq!(s.timestamp, 4000);
    }

    #[test]
    fn roundtrip_hebbian_record() {
        let pf = roundtrip(|w| {
            w.append_hebbian(0xAAAA, 0xBBBB, 200, 42, 5000);
        });
        assert_eq!(pf.hebbian_records.len(), 1);
        let h = &pf.hebbian_records[0];
        assert_eq!(h.from_hash, 0xAAAA);
        assert_eq!(h.to_hash, 0xBBBB);
        assert_eq!(h.weight, 200);
        assert_eq!(h.fire_count, 42);
        assert_eq!(h.timestamp, 5000);
    }

    #[test]
    fn roundtrip_knowtree_record() {
        let data = [0x01u8, 0x02, 0x03, 0x04, 0x05];
        let pf = roundtrip(|w| {
            w.append_knowtree(&data, 6000).unwrap();
        });
        assert_eq!(pf.knowtree_records.len(), 1);
        let k = &pf.knowtree_records[0];
        assert_eq!(k.data, data);
        assert_eq!(k.timestamp, 6000);
    }

    #[test]
    fn roundtrip_curve_record() {
        let pf = roundtrip(|w| {
            w.append_curve(-0.3, 0.15, 7000);
        });
        assert_eq!(pf.curve_records.len(), 1);
        let c = &pf.curve_records[0];
        assert!((c.valence - (-0.3)).abs() < 0.001);
        assert!((c.fx_dn - 0.15).abs() < 0.001);
        assert_eq!(c.timestamp, 7000);
    }

    #[test]
    fn roundtrip_mixed_all_record_types() {
        let chain = encode_codepoint(0x1F525);
        let hash = chain.chain_hash();
        let pf = roundtrip(|w| {
            w.append_node(&chain, 0, false, 1000).unwrap();
            w.append_edge(hash, 0xDEAD, 0x01, 1001);
            w.append_alias("fire", hash, 1002).unwrap();
            w.append_stm(hash, -0.5, 0.7, 0.3, 0.8, 1, 0x00, 0, 1003);
            w.append_hebbian(hash, 0xDEAD, 128, 3, 1004);
            w.append_knowtree(&[0x42], 1005).unwrap();
            w.append_curve(-0.2, 0.1, 1006);
        });
        assert_eq!(pf.node_count(), 1);
        assert_eq!(pf.edge_count(), 1);
        assert_eq!(pf.alias_count(), 1);
        assert_eq!(pf.stm_records.len(), 1);
        assert_eq!(pf.hebbian_records.len(), 1);
        assert_eq!(pf.knowtree_records.len(), 1);
        assert_eq!(pf.curve_records.len(), 1);
    }
}
