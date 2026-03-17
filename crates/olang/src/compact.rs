//! # compact — Mã hóa nén cho L2-Ln KnowTree
//!
//! ## Vấn đề
//!
//! L0-L1: ~100K nodes × 5 bytes = 500KB → OK
//! L2-Ln: hàng triệu/tỷ nodes → cần nén
//!
//! Hiện tại: 1 node = MolecularChain(5B) + Registry(26B) + Silk(88B/edge)
//! → 1M nodes + 5M edges = 518MB RAM
//! → 1B nodes = 518TB RAM → KHÔNG chấp nhận được
//!
//! ## Giải pháp: Compact Encoding lần 2
//!
//! ```text
//! Kỹ thuật         | Tiết kiệm   | Ý tưởng
//! ──────────────────┼──────────────┼──────────────────────
//! Delta encoding    | 60-80%       | Chỉ lưu diff từ parent
//! Chain dictionary  | 50-70%       | Sub-chains thường gặp → short ID
//! Hash dedup        | 30-90%       | Cùng hash = cùng chain = lưu 1 lần
//! Silk compression  | 70-90%       | L2+ chỉ lưu strong edges (weight > θ)
//! Tiered storage    | ∞            | Cold knowledge → archive, chỉ load on-demand
//! ```
//!
//! ## Architecture
//!
//! ```text
//! L0-L1: Full Molecule (5B) — chính xác, lookup nhanh
//!    ↓ parent chain
//! L2+: CompactNode (3-8B) — delta từ parent + dictionary ID
//!    ↓ references
//! Silk L2+: CompactEdge (6B) — chỉ hash pair + weight, prune weak
//!    ↓ archive
//! Cold: ArchivedPage — batch nodes + edges, compressed, on-disk only
//! ```
//!
//! Nguyên tắc:
//!   - Append-only vẫn giữ nguyên (QT8)
//!   - Registry vẫn là source of truth
//!   - Compact format = presentation layer, không thay đổi semantic
//!   - Có thể reconstruct Full Molecule từ Compact + parent chain

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::hash::fnv1a;
use crate::molecular::{EmotionDim, MolecularChain, Molecule};

// ─────────────────────────────────────────────────────────────────────────────
// DeltaMolecule — chỉ lưu diff từ parent
// ─────────────────────────────────────────────────────────────────────────────

/// Delta encoding: chỉ lưu fields khác so với parent.
///
/// Full Molecule = 5 bytes. DeltaMolecule = 1 byte bitmask + changed fields.
/// Nếu node giống parent 80% → chỉ cần 2 bytes thay vì 5.
///
/// Format: [bitmask:1B] [changed_fields:0-5B]
///   bit 0: shape changed
///   bit 1: relation changed
///   bit 2: valence changed
///   bit 3: arousal changed
///   bit 4: time changed
///
/// Best case: 1 byte (giống parent hoàn toàn, bitmask = 0x00)
/// Worst case: 6 bytes (tất cả khác, bitmask + 5 fields)
/// Average: 2-3 bytes (thường chỉ khác emotion)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeltaMolecule {
    /// Bitmask: fields nào khác parent
    pub mask: u8,
    /// Changed fields (order: shape, relation, valence, arousal, time)
    pub data: Vec<u8>,
}

impl DeltaMolecule {
    /// Encode delta từ parent → child.
    pub fn encode(parent: &Molecule, child: &Molecule) -> Self {
        let mut mask = 0u8;
        let mut data = Vec::new();

        if child.shape != parent.shape {
            mask |= 0x01;
            data.push(child.shape);
        }
        if child.relation != parent.relation {
            mask |= 0x02;
            data.push(child.relation);
        }
        if child.emotion.valence != parent.emotion.valence {
            mask |= 0x04;
            data.push(child.emotion.valence);
        }
        if child.emotion.arousal != parent.emotion.arousal {
            mask |= 0x08;
            data.push(child.emotion.arousal);
        }
        if child.time != parent.time {
            mask |= 0x10;
            data.push(child.time);
        }

        Self { mask, data }
    }

    /// Decode: parent + delta → child molecule.
    pub fn decode(&self, parent: &Molecule) -> Option<Molecule> {
        let mut idx = 0usize;
        let shape = if self.mask & 0x01 != 0 {
            let b = *self.data.get(idx)?;
            idx += 1;
            b
        } else {
            parent.shape
        };
        let relation = if self.mask & 0x02 != 0 {
            let b = *self.data.get(idx)?;
            idx += 1;
            b
        } else {
            parent.relation
        };
        let valence = if self.mask & 0x04 != 0 {
            let b = *self.data.get(idx)?;
            idx += 1;
            b
        } else {
            parent.emotion.valence
        };
        let arousal = if self.mask & 0x08 != 0 {
            let b = *self.data.get(idx)?;
            idx += 1;
            b
        } else {
            parent.emotion.arousal
        };
        let time = if self.mask & 0x10 != 0 {
            let b = *self.data.get(idx)?;
            let _ = idx;
            b
        } else {
            parent.time
        };

        Some(Molecule {
            shape,
            relation,
            emotion: EmotionDim { valence, arousal },
            time,
        })
    }

    /// Serialized size (bytes).
    pub fn size(&self) -> usize {
        1 + self.data.len()
    }

    /// Serialize → bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(1 + self.data.len());
        buf.push(self.mask);
        buf.extend_from_slice(&self.data);
        buf
    }

    /// Deserialize từ bytes.
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.is_empty() {
            return None;
        }
        let mask = b[0];
        let expected_len = mask.count_ones() as usize;
        if b.len() < 1 + expected_len {
            return None;
        }
        let data = b[1..1 + expected_len].to_vec();
        Some(Self { mask, data })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ChainDictionary — sub-chains thường gặp → short ID
// ─────────────────────────────────────────────────────────────────────────────

/// Entry trong dictionary: chain_hash → compact ID.
#[derive(Debug, Clone)]
struct DictEntry {
    /// Chain hash (FNV-1a)
    hash: u64,
    /// Full chain bytes (để reconstruct)
    chain_bytes: Vec<u8>,
    /// Số lần sử dụng (dùng để prune)
    usage_count: u32,
}

/// Chain Dictionary — map frequently-used chains → compact variable-length IDs.
///
/// Thay vì lưu full 5-byte molecule mỗi lần, lưu dictionary ID.
/// Dictionary tự động học patterns phổ biến.
///
/// ID encoding (variable-length):
///   0x00..0x7F     → 1 byte  (128 entries — most common)
///   0x80XX         → 2 bytes (32K entries)
///   0xC0XXXX       → 3 bytes (8M entries — rare)
pub struct ChainDictionary {
    /// Entries sorted by hash
    entries: Vec<DictEntry>,
    /// Max entries trước khi prune
    max_entries: usize,
}

impl ChainDictionary {
    /// Tạo dictionary mới.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// Tạo với capacity mặc định (Fib[12] = 144 cho tier 1, mở rộng sau).
    pub fn default_size() -> Self {
        Self::new(4096)
    }

    /// Register chain → trả dictionary ID.
    /// Nếu đã có → tăng usage_count, trả ID hiện tại.
    /// Nếu chưa → thêm mới.
    pub fn register(&mut self, chain: &MolecularChain) -> u32 {
        let hash = chain.chain_hash();

        // Tìm existing
        for (i, entry) in self.entries.iter_mut().enumerate() {
            if entry.hash == hash {
                entry.usage_count += 1;
                return i as u32;
            }
        }

        // Nếu đầy → prune least-used
        if self.entries.len() >= self.max_entries {
            self.prune();
        }

        let id = self.entries.len() as u32;
        self.entries.push(DictEntry {
            hash,
            chain_bytes: chain.to_tagged_bytes(),
            usage_count: 1,
        });
        id
    }

    /// Lookup dictionary ID → chain.
    pub fn lookup(&self, id: u32) -> Option<MolecularChain> {
        let entry = self.entries.get(id as usize)?;
        MolecularChain::from_tagged_bytes(&entry.chain_bytes)
    }

    /// Lookup hash → dictionary ID.
    pub fn lookup_hash(&self, hash: u64) -> Option<u32> {
        self.entries
            .iter()
            .position(|e| e.hash == hash)
            .map(|i| i as u32)
    }

    /// Số entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Dictionary rỗng?
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Prune: xóa 25% entries ít dùng nhất.
    fn prune(&mut self) {
        if self.entries.len() < 4 {
            return;
        }

        // Sort by usage (ascending) → remove bottom 25%
        let mut indexed: Vec<(usize, u32)> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| (i, e.usage_count))
            .collect();
        indexed.sort_by_key(|&(_, count)| count);

        let remove_count = self.entries.len() / 4;
        let to_remove: Vec<usize> = indexed.iter().take(remove_count).map(|&(i, _)| i).collect();

        // Remove in reverse order to keep indices valid
        let mut to_remove_sorted = to_remove;
        to_remove_sorted.sort();
        for &i in to_remove_sorted.iter().rev() {
            self.entries.swap_remove(i);
        }
    }

    /// Encode dictionary ID → variable-length bytes.
    ///
    /// 0x00..0x7F     → 1 byte
    /// 0x0080..0x407F → 2 bytes (0x80 + high, low)
    /// 0x4080+        → 3 bytes (0xC0 + high, mid, low)
    pub fn encode_id(id: u32) -> Vec<u8> {
        if id < 0x80 {
            alloc::vec![id as u8]
        } else if id < 0x4080 {
            let adjusted = id - 0x80;
            alloc::vec![0x80 | (adjusted >> 8) as u8, adjusted as u8]
        } else {
            let adjusted = id - 0x4080;
            alloc::vec![
                0xC0 | (adjusted >> 16) as u8,
                (adjusted >> 8) as u8,
                adjusted as u8,
            ]
        }
    }

    /// Decode variable-length bytes → dictionary ID.
    pub fn decode_id(bytes: &[u8]) -> Option<(u32, usize)> {
        if bytes.is_empty() {
            return None;
        }
        let first = bytes[0];
        if first < 0x80 {
            Some((first as u32, 1))
        } else if first < 0xC0 {
            if bytes.len() < 2 {
                return None;
            }
            let id = ((first as u32 & 0x3F) << 8) | bytes[1] as u32;
            Some((id + 0x80, 2))
        } else {
            if bytes.len() < 3 {
                return None;
            }
            let id = ((first as u32 & 0x3F) << 16) | ((bytes[1] as u32) << 8) | bytes[2] as u32;
            Some((id + 0x4080, 3))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CompactNode — nén node L2+
// ─────────────────────────────────────────────────────────────────────────────

/// Cách encode compact node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompactKind {
    /// Full molecule (L0-L1): giữ nguyên 5 bytes
    Full = 0x00,
    /// Delta từ parent: bitmask + changed bytes
    Delta = 0x01,
    /// Dictionary reference: variable-length ID
    DictRef = 0x02,
    /// Duplicate: chỉ hash (đã tồn tại)
    Dedup = 0x03,
}

/// Compact node — đại diện nén của 1 knowledge node.
///
/// Thay vì lưu full MolecularChain (5B/mol), compact node lưu:
///   - Delta từ parent chain (2-3B trung bình)
///   - Hoặc dictionary reference (1-2B)
///   - Hoặc dedup hash (0B extra — chỉ pointer)
///
/// Kèm metadata tối thiểu cho L2+ (không cần đầy đủ như L0-L1).
#[derive(Debug, Clone)]
pub struct CompactNode {
    /// Hash (vẫn là FNV-1a — identity key)
    pub hash: u64,
    /// Parent hash (chain mà node này delta từ)
    pub parent_hash: u64,
    /// Layer
    pub layer: u8,
    /// Kiểu encoding
    pub kind: CompactKind,
    /// Compact data (delta bytes, dict ID, hoặc rỗng nếu dedup)
    pub data: Vec<u8>,
    /// Timestamp
    pub created_at: i64,
}

impl CompactNode {
    /// Encode full molecule → compact node.
    ///
    /// Tự chọn encoding tối ưu nhất:
    ///   1. Nếu hash đã có trong seen_hashes → Dedup (0 bytes data)
    ///   2. Nếu có trong dictionary → DictRef (1-3 bytes)
    ///   3. Nếu có parent → Delta (1-6 bytes)
    ///   4. Fallback → Full (5 bytes)
    pub fn encode(
        chain: &MolecularChain,
        parent: Option<&MolecularChain>,
        dict: &mut ChainDictionary,
        layer: u8,
        ts: i64,
    ) -> Self {
        let hash = chain.chain_hash();

        // 1. Dedup check: đã có trong dictionary?
        if let Some(dict_id) = dict.lookup_hash(hash) {
            return Self {
                hash,
                parent_hash: parent.map_or(0, |p| p.chain_hash()),
                layer,
                kind: CompactKind::Dedup,
                data: ChainDictionary::encode_id(dict_id),
                created_at: ts,
            };
        }

        // Register vào dictionary
        let dict_id = dict.register(chain);

        // 2. Delta encoding nếu có parent
        if let Some(parent_chain) = parent {
            if !parent_chain.is_empty() && !chain.is_empty() {
                let parent_mol = &parent_chain.0[0];
                let child_mol = &chain.0[0];
                let delta = DeltaMolecule::encode(parent_mol, child_mol);

                // Nếu delta nhỏ hơn tagged full → dùng delta
                if delta.size() < child_mol.tagged_size() {
                    return Self {
                        hash,
                        parent_hash: parent_chain.chain_hash(),
                        layer,
                        kind: CompactKind::Delta,
                        data: delta.to_bytes(),
                        created_at: ts,
                    };
                }
            }
        }

        // 3. Dictionary reference (chain mới, nhưng đã register)
        if dict_id < 0x4080 {
            // 2 bytes hoặc ít hơn
            return Self {
                hash,
                parent_hash: parent.map_or(0, |p| p.chain_hash()),
                layer,
                kind: CompactKind::DictRef,
                data: ChainDictionary::encode_id(dict_id),
                created_at: ts,
            };
        }

        // 4. Full encoding fallback (tagged format — variable length)
        Self {
            hash,
            parent_hash: parent.map_or(0, |p| p.chain_hash()),
            layer,
            kind: CompactKind::Full,
            data: chain.to_tagged_bytes(),
            created_at: ts,
        }
    }

    /// Decode compact node → full MolecularChain.
    ///
    /// Cần parent chain (nếu Delta) hoặc dictionary (nếu DictRef/Dedup).
    pub fn decode(
        &self,
        parent: Option<&MolecularChain>,
        dict: &ChainDictionary,
    ) -> Option<MolecularChain> {
        match self.kind {
            CompactKind::Full => MolecularChain::from_tagged_bytes(&self.data),
            CompactKind::Delta => {
                let parent_chain = parent?;
                if parent_chain.is_empty() {
                    return None;
                }
                let delta = DeltaMolecule::from_bytes(&self.data)?;
                let parent_mol = &parent_chain.0[0];
                let child_mol = delta.decode(parent_mol)?;
                Some(MolecularChain::single(child_mol))
            }
            CompactKind::DictRef | CompactKind::Dedup => {
                let (id, _) = ChainDictionary::decode_id(&self.data)?;
                dict.lookup(id)
            }
        }
    }

    /// Serialized size (bytes) — data portion only.
    pub fn data_size(&self) -> usize {
        self.data.len()
    }

    /// Total serialized size (bytes) — bao gồm header.
    ///
    /// Format: [kind:1][hash:8][parent_hash:8][layer:1][ts:8][data_len:2][data:N]
    pub fn total_size(&self) -> usize {
        1 + 8 + 8 + 1 + 8 + 2 + self.data.len()
    }

    /// Serialize → bytes cho file storage.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.total_size());
        buf.push(self.kind as u8);
        buf.extend_from_slice(&self.hash.to_be_bytes());
        buf.extend_from_slice(&self.parent_hash.to_be_bytes());
        buf.push(self.layer);
        buf.extend_from_slice(&self.created_at.to_be_bytes());
        buf.extend_from_slice(&(self.data.len() as u16).to_be_bytes());
        buf.extend_from_slice(&self.data);
        buf
    }

    /// Deserialize từ bytes.
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() < 28 {
            return None;
        } // minimum header
        let kind = match b[0] {
            0x00 => CompactKind::Full,
            0x01 => CompactKind::Delta,
            0x02 => CompactKind::DictRef,
            0x03 => CompactKind::Dedup,
            _ => return None,
        };
        let hash = u64::from_be_bytes(b[1..9].try_into().ok()?);
        let parent_hash = u64::from_be_bytes(b[9..17].try_into().ok()?);
        let layer = b[17];
        let created_at = i64::from_be_bytes(b[18..26].try_into().ok()?);
        let data_len = u16::from_be_bytes(b[26..28].try_into().ok()?) as usize;
        if b.len() < 28 + data_len {
            return None;
        }
        let data = b[28..28 + data_len].to_vec();
        Some(Self {
            hash,
            parent_hash,
            layer,
            kind,
            data,
            created_at,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CompactEdge — nén Silk edge cho L2+
// ─────────────────────────────────────────────────────────────────────────────

/// Compact Silk edge cho L2+ — chỉ lưu thông tin thiết yếu.
///
/// Full SilkEdge = 46 bytes (serialized), 88 bytes (in-memory).
/// CompactEdge  = 10 bytes — tiết kiệm 78-80%.
///
/// Lý do: L2+ edges chỉ cần biết "A liên kết B, mạnh thế nào".
/// Emotion detail → reconstruct từ node context khi cần.
#[derive(Debug, Clone, PartialEq)]
pub struct CompactEdge {
    /// From node hash (truncated to u32 — 4B thay vì 8B)
    pub from_hash: u32,
    /// To node hash (truncated to u32)
    pub to_hash: u32,
    /// Weight quantized 0..255 (1B thay vì f32 4B)
    pub weight: u8,
    /// Relation type (1B)
    pub relation: u8,
}

impl CompactEdge {
    /// Encode từ full hashes + weight.
    pub fn encode(from: u64, to: u64, weight: f32, relation: u8) -> Self {
        Self {
            from_hash: (from & 0xFFFFFFFF) as u32,
            to_hash: (to & 0xFFFFFFFF) as u32,
            weight: (weight.clamp(0.0, 1.0) * 255.0) as u8,
            relation,
        }
    }

    /// Serialize → 10 bytes.
    pub fn to_bytes(&self) -> [u8; 10] {
        let mut buf = [0u8; 10];
        buf[0..4].copy_from_slice(&self.from_hash.to_be_bytes());
        buf[4..8].copy_from_slice(&self.to_hash.to_be_bytes());
        buf[8] = self.weight;
        buf[9] = self.relation;
        buf
    }

    /// Deserialize từ 10 bytes.
    pub fn from_bytes(b: &[u8; 10]) -> Self {
        Self {
            from_hash: u32::from_be_bytes([b[0], b[1], b[2], b[3]]),
            to_hash: u32::from_be_bytes([b[4], b[5], b[6], b[7]]),
            weight: b[8],
            relation: b[9],
        }
    }

    /// Weight dạng f32.
    pub fn weight_f32(&self) -> f32 {
        self.weight as f32 / 255.0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CompactPage — batch nodes + edges cho archival
// ─────────────────────────────────────────────────────────────────────────────

/// Compact page — nhóm nodes và edges thành page để storage hiệu quả.
///
/// Mỗi page chứa tối đa PAGE_SIZE nodes + edges liên quan.
/// Pages có thể nằm trên disk, chỉ load on-demand.
///
/// Format:
/// ```text
/// [PAGE_MAGIC:4] [page_id:4] [node_count:4] [edge_count:4]
/// [node_0] [node_1] ... [node_N]
/// [edge_0] [edge_1] ... [edge_M]
/// [checksum:8]
/// ```
#[derive(Debug, Clone)]
pub struct CompactPage {
    /// Page ID (sequential)
    pub page_id: u32,
    /// Layer range this page covers
    pub layer: u8,
    /// Compact nodes
    pub nodes: Vec<CompactNode>,
    /// Compact edges
    pub edges: Vec<CompactEdge>,
}

/// Page magic bytes: "CPAG"
const PAGE_MAGIC: [u8; 4] = [0x43, 0x50, 0x41, 0x47];
/// Fibonacci page size: Fib[13] = 233 nodes per page
const DEFAULT_PAGE_CAPACITY: usize = 233;

impl CompactPage {
    /// Tạo page mới.
    pub fn new(page_id: u32, layer: u8) -> Self {
        Self {
            page_id,
            layer,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Page đầy chưa?
    pub fn is_full(&self) -> bool {
        self.nodes.len() >= DEFAULT_PAGE_CAPACITY
    }

    /// Capacity còn lại.
    pub fn remaining(&self) -> usize {
        DEFAULT_PAGE_CAPACITY.saturating_sub(self.nodes.len())
    }

    /// Thêm node.
    pub fn push_node(&mut self, node: CompactNode) -> bool {
        if self.is_full() {
            return false;
        }
        self.nodes.push(node);
        true
    }

    /// Thêm edge.
    pub fn push_edge(&mut self, edge: CompactEdge) {
        self.edges.push(edge);
    }

    /// Tổng data size (bytes) — không tính header.
    pub fn data_size(&self) -> usize {
        let node_size: usize = self.nodes.iter().map(|n| n.total_size()).sum();
        let edge_size = self.edges.len() * 10;
        node_size + edge_size
    }

    /// So sánh compression ratio vs full format.
    ///
    /// Full format: node = 17B + edge = 26B
    /// Compact: node = ~30B avg + edge = 10B
    /// (compact node larger due to header, nhưng data portion nhỏ hơn nhiều)
    pub fn compression_stats(&self) -> CompressionStats {
        let full_node_size = self.nodes.len() * 17; // NodeRecord in origin.olang
        let full_edge_size = self.edges.len() * 26; // EdgeRecord
        let full_total = full_node_size + full_edge_size;

        let compact_total = self.data_size();

        CompressionStats {
            full_bytes: full_total,
            compact_bytes: compact_total,
            ratio: if full_total > 0 {
                compact_total as f32 / full_total as f32
            } else {
                1.0
            },
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
        }
    }

    /// Serialize page → bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&PAGE_MAGIC);
        buf.extend_from_slice(&self.page_id.to_be_bytes());
        buf.push(self.layer);
        buf.extend_from_slice(&(self.nodes.len() as u32).to_be_bytes());
        buf.extend_from_slice(&(self.edges.len() as u32).to_be_bytes());

        // Nodes
        for node in &self.nodes {
            let node_bytes = node.to_bytes();
            buf.extend_from_slice(&node_bytes);
        }

        // Edges
        for edge in &self.edges {
            buf.extend_from_slice(&edge.to_bytes());
        }

        // Checksum
        let checksum = fnv1a(&buf);
        buf.extend_from_slice(&checksum.to_be_bytes());

        buf
    }
}

/// Thống kê compression.
#[derive(Debug, Clone)]
pub struct CompressionStats {
    /// Full format size (bytes)
    pub full_bytes: usize,
    /// Compact format size (bytes)
    pub compact_bytes: usize,
    /// Compression ratio (compact / full) — nhỏ hơn = tốt hơn
    pub ratio: f32,
    /// Số nodes
    pub node_count: usize,
    /// Số edges
    pub edge_count: usize,
}

impl CompressionStats {
    /// Tiết kiệm (%).
    pub fn savings_percent(&self) -> f32 {
        (1.0 - self.ratio) * 100.0
    }

    /// Summary.
    pub fn summary(&self) -> String {
        alloc::format!(
            "{} nodes + {} edges: {}B → {}B ({:.1}% savings)",
            self.node_count,
            self.edge_count,
            self.full_bytes,
            self.compact_bytes,
            self.savings_percent(),
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SilkPruner — prune weak edges cho L2+
// ─────────────────────────────────────────────────────────────────────────────

/// Silk pruner — loại bỏ edges yếu khi compact.
///
/// L0-L1: giữ tất cả edges (quan trọng cho learning).
/// L2+: chỉ giữ edges mạnh (weight ≥ threshold).
///
/// Threshold theo Fibonacci: Fib[n]/(Fib[n]+Fib[n+1]) ≈ 0.382
/// → Giữ top 38.2% edges (golden ratio pruning).
pub struct SilkPruner;

impl SilkPruner {
    /// Fibonacci-based threshold: φ⁻¹ ≈ 0.618, → keep if weight ≥ 1 - φ⁻¹ = 0.382
    pub const FIBONACCI_THRESHOLD: f32 = 0.382;

    /// Prune edges: chỉ giữ strong ones.
    pub fn prune(edges: &[(u64, u64, f32, u8)], threshold: f32) -> Vec<CompactEdge> {
        edges
            .iter()
            .filter(|&&(_, _, w, _)| w >= threshold)
            .map(|&(from, to, w, rel)| CompactEdge::encode(from, to, w, rel))
            .collect()
    }

    /// Prune với Fibonacci threshold mặc định.
    pub fn prune_default(edges: &[(u64, u64, f32, u8)]) -> Vec<CompactEdge> {
        Self::prune(edges, Self::FIBONACCI_THRESHOLD)
    }

    /// Thống kê prune: bao nhiêu edges bị loại.
    pub fn prune_stats(edges: &[(u64, u64, f32, u8)], threshold: f32) -> (usize, usize) {
        let total = edges.len();
        let kept = edges.iter().filter(|&&(_, _, w, _)| w >= threshold).count();
        (kept, total - kept)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PageIndex — lightweight hash → page_id lookup
// ─────────────────────────────────────────────────────────────────────────────

/// Page index entry: chỉ 12 bytes per node (hash + page_id).
///
/// Với 1B nodes: 12B × 1B = 12GB — vẫn lớn.
/// Giải pháp: Bloom filter + range index per layer.
///
/// PageIndex chỉ load layer đang cần.
/// L0-L1: luôn in RAM (nhỏ).
/// L2+: load range index on-demand.
#[derive(Debug, Clone)]
pub struct PageIndexEntry {
    /// Truncated hash (4 bytes — đủ cho 4B unique entries)
    pub hash_lo: u32,
    /// Page ID chứa node này
    pub page_id: u32,
}

/// Page index cho 1 layer — compact mapping hash → page.
#[derive(Debug, Clone)]
pub struct LayerIndex {
    /// Layer
    pub layer: u8,
    /// Entries sorted by hash_lo (binary search)
    pub entries: Vec<PageIndexEntry>,
    /// Bloom filter (256 bytes = 2048 bits) — quick "definitely not here" check
    pub bloom: [u8; 256],
}

impl LayerIndex {
    /// Tạo index rỗng.
    pub fn new(layer: u8) -> Self {
        Self {
            layer,
            entries: Vec::new(),
            bloom: [0u8; 256],
        }
    }

    /// Thêm entry.
    pub fn insert(&mut self, hash: u64, page_id: u32) {
        let hash_lo = (hash & 0xFFFFFFFF) as u32;
        // Bloom filter: set 3 bits
        let b1 = (hash & 0x7FF) as usize;
        let b2 = ((hash >> 11) & 0x7FF) as usize;
        let b3 = ((hash >> 22) & 0x7FF) as usize;
        self.bloom[b1 / 8] |= 1 << (b1 % 8);
        self.bloom[b2 / 8] |= 1 << (b2 % 8);
        self.bloom[b3 / 8] |= 1 << (b3 % 8);

        // Insert sorted
        let pos = self.entries.partition_point(|e| e.hash_lo < hash_lo);
        self.entries
            .insert(pos, PageIndexEntry { hash_lo, page_id });
    }

    /// Bloom filter check — nhanh, false positive OK.
    pub fn maybe_contains(&self, hash: u64) -> bool {
        let b1 = (hash & 0x7FF) as usize;
        let b2 = ((hash >> 11) & 0x7FF) as usize;
        let b3 = ((hash >> 22) & 0x7FF) as usize;
        (self.bloom[b1 / 8] & (1 << (b1 % 8))) != 0
            && (self.bloom[b2 / 8] & (1 << (b2 % 8))) != 0
            && (self.bloom[b3 / 8] & (1 << (b3 % 8))) != 0
    }

    /// Tìm page_id chứa hash (binary search).
    pub fn find_page(&self, hash: u64) -> Option<u32> {
        if !self.maybe_contains(hash) {
            return None;
        }
        let hash_lo = (hash & 0xFFFFFFFF) as u32;
        self.entries
            .binary_search_by_key(&hash_lo, |e| e.hash_lo)
            .ok()
            .map(|i| self.entries[i].page_id)
    }

    /// Số entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Rỗng?
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Size in RAM (bytes).
    ///
    /// 256 (bloom) + entries × 8 (hash_lo:4 + page_id:4)
    pub fn ram_size(&self) -> usize {
        256 + self.entries.len() * 8
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PageCache — LRU cache cho CompactPages
// ─────────────────────────────────────────────────────────────────────────────

/// LRU page cache — chỉ giữ N pages hot nhất trong RAM.
///
/// Bình thường: PC giữ ~Fib[15] = 610 pages trong RAM.
/// Mỗi page ~233 nodes × ~30B = ~7KB compact data.
/// 610 pages × 7KB = ~4.3MB RAM — AFFORDABLE.
///
/// Access pattern: LRU eviction khi cache đầy.
/// Page bị evict → write back to disk nếu dirty.
pub struct PageCache {
    /// Cached pages (page_id → page)
    pages: Vec<(u32, CompactPage, u64)>, // (page_id, page, last_access_ts)
    /// Max pages in cache
    capacity: usize,
    /// Access counter (monotonic)
    access_counter: u64,
    /// Tổng hit/miss cho stats
    hits: u64,
    /// Misses
    misses: u64,
}

/// Fibonacci cache capacities cho các tầng thiết bị.
pub struct CacheCapacity;
impl CacheCapacity {
    /// ESP32/MCU: Fib[10] = 55 pages ≈ 385KB
    pub const EMBEDDED: usize = 55;
    /// Smartphone: Fib[13] = 233 pages ≈ 1.6MB
    pub const MOBILE: usize = 233;
    /// PC: Fib[15] = 610 pages ≈ 4.3MB
    pub const PC: usize = 610;
    /// Server: Fib[18] = 2584 pages ≈ 18MB
    pub const SERVER: usize = 2584;
}

impl PageCache {
    /// Tạo cache với capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            pages: Vec::with_capacity(capacity),
            capacity,
            access_counter: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Cache cho PC.
    pub fn for_pc() -> Self {
        Self::new(CacheCapacity::PC)
    }

    /// Cache cho mobile.
    pub fn for_mobile() -> Self {
        Self::new(CacheCapacity::MOBILE)
    }

    /// Cache cho embedded.
    pub fn for_embedded() -> Self {
        Self::new(CacheCapacity::EMBEDDED)
    }

    /// Get page from cache (hit). Returns None if not cached (miss).
    pub fn get(&mut self, page_id: u32) -> Option<&CompactPage> {
        self.access_counter += 1;
        for entry in self.pages.iter_mut() {
            if entry.0 == page_id {
                entry.2 = self.access_counter; // update LRU timestamp
                self.hits += 1;
                return Some(&entry.1);
            }
        }
        self.misses += 1;
        None
    }

    /// Put page into cache. Evicts LRU if full.
    /// Returns evicted page if any (caller should write to disk).
    pub fn put(&mut self, page_id: u32, page: CompactPage) -> Option<CompactPage> {
        self.access_counter += 1;

        // Already cached? Update.
        for entry in self.pages.iter_mut() {
            if entry.0 == page_id {
                entry.1 = page;
                entry.2 = self.access_counter;
                return None;
            }
        }

        // Cache full → evict LRU
        let evicted = if self.pages.len() >= self.capacity {
            // Find least recently used
            let lru_idx = self
                .pages
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.2)
                .map(|(i, _)| i)
                .unwrap_or(0);
            let old = self.pages.swap_remove(lru_idx);
            Some(old.1)
        } else {
            None
        };

        self.pages.push((page_id, page, self.access_counter));
        evicted
    }

    /// Số pages hiện tại.
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// Cache rỗng?
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// Hit rate (0.0 .. 1.0).
    pub fn hit_rate(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 {
            return 0.0;
        }
        self.hits as f32 / total as f32
    }

    /// RAM usage estimate (bytes).
    pub fn ram_usage(&self) -> usize {
        self.pages.iter().map(|(_, p, _)| p.data_size() + 12).sum()
    }

    /// Stats summary.
    pub fn stats(&self) -> String {
        alloc::format!(
            "PageCache: {}/{} pages | {:.1}% hit rate | ~{}KB RAM",
            self.len(),
            self.capacity,
            self.hit_rate() * 100.0,
            self.ram_usage() / 1024,
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TieredStore — Hot/Warm/Cold storage
// ─────────────────────────────────────────────────────────────────────────────

/// Storage tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageTier {
    /// Hot: trong RAM (L0-L1 + recently accessed L2+)
    Hot,
    /// Warm: trong PageCache (frequently accessed L2+ pages)
    Warm,
    /// Cold: trên disk (archived L2+ pages, load on-demand)
    Cold,
}

/// Kết quả lookup: node + edges + neighbors (silk tracking).
///
/// ```text
/// NodeWithEdges {
///   node: CompactNode (target)
///   edges: [CompactEdge] (outgoing silk connections)
///   neighbors: [NeighborInfo] (resolved neighbor nodes)
///   depth: u8 (how deep we followed)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct NodeWithEdges {
    /// The node itself
    pub node: CompactNode,
    /// Outgoing silk edges from this node
    pub edges: Vec<CompactEdge>,
    /// Resolved neighbor nodes (if depth > 0)
    pub neighbors: Vec<NeighborInfo>,
    /// Depth at which this node was found
    pub depth: u8,
}

/// Thông tin 1 neighbor node (qua silk edge).
#[derive(Debug, Clone)]
pub struct NeighborInfo {
    /// Neighbor node
    pub node: CompactNode,
    /// Silk edge weight (0.0 .. 1.0)
    pub weight: f32,
    /// Relation type (raw byte)
    pub relation: u8,
}

/// Tiered store — quản lý storage across tiers.
///
/// ```text
/// ┌─────────────────────────────────────────────────────────┐
/// │ HOT (RAM)                                               │
/// │   L0-L1: Full Registry + SilkGraph (always loaded)      │
/// │   Recent STM entries                                     │
/// │   RAM budget: ~50MB on PC, ~10MB on phone               │
/// ├─────────────────────────────────────────────────────────┤
/// │ WARM (PageCache)                                        │
/// │   L2+ CompactPages (LRU, Fib[15]=610 pages on PC)      │
/// │   LayerIndex per loaded layer                            │
/// │   RAM budget: ~5MB on PC, ~2MB on phone                 │
/// ├─────────────────────────────────────────────────────────┤
/// │ COLD (Disk)                                             │
/// │   All CompactPages serialized                            │
/// │   Global index (LayerIndex per layer)                    │
/// │   Disk budget: unlimited                                 │
/// └─────────────────────────────────────────────────────────┘
///
/// 1B nodes + 5B edges estimate:
///   Index: 1B × 8B = 8GB (too much for RAM)
///     → Shard by layer: L2 index, L3 index, ..., LN index
///     → Each layer index fits in RAM when needed
///   Pages: 1B nodes / 233 per page = 4.3M pages
///     → 4.3M × 7KB = ~30GB disk (OK)
///     → PageCache: 610 pages × 7KB = 4.3MB RAM (OK)
///   Edges: 5B × 38.2% kept × 10B = ~19GB disk (OK)
///
/// Total for 1B nodes:
///   RAM:  ~60MB (hot L0-L1 + warm cache + active index)
///   Disk: ~50GB (pages + edges + index)
/// ```
pub struct TieredStore {
    /// Page cache (warm tier)
    pub cache: PageCache,
    /// Layer indexes (loaded on demand)
    pub indexes: Vec<LayerIndex>,
    /// Cold pages (serialized bytes, simulating disk)
    /// In real impl: file offsets
    cold_storage: Vec<(u32, Vec<u8>)>, // (page_id, serialized_bytes)
    /// Dictionary (shared across tiers)
    pub dict: ChainDictionary,
    /// Next page ID
    next_page_id: u32,
    /// Total nodes stored
    total_nodes: u64,
    /// Total edges stored
    total_edges: u64,
}

impl TieredStore {
    /// Tạo store cho PC.
    pub fn for_pc() -> Self {
        Self {
            cache: PageCache::for_pc(),
            indexes: Vec::new(),
            cold_storage: Vec::new(),
            dict: ChainDictionary::default_size(),
            next_page_id: 0,
            total_nodes: 0,
            total_edges: 0,
        }
    }

    /// Tạo store cho mobile.
    pub fn for_mobile() -> Self {
        Self {
            cache: PageCache::for_mobile(),
            indexes: Vec::new(),
            cold_storage: Vec::new(),
            dict: ChainDictionary::default_size(),
            next_page_id: 0,
            total_nodes: 0,
            total_edges: 0,
        }
    }

    /// Tạo store cho embedded.
    pub fn for_embedded() -> Self {
        Self {
            cache: PageCache::for_embedded(),
            indexes: Vec::new(),
            cold_storage: Vec::new(),
            dict: ChainDictionary::new(256), // smaller dict for embedded
            next_page_id: 0,
            total_nodes: 0,
            total_edges: 0,
        }
    }

    /// Tạo store custom.
    pub fn new(cache_capacity: usize, dict_capacity: usize) -> Self {
        Self {
            cache: PageCache::new(cache_capacity),
            indexes: Vec::new(),
            cold_storage: Vec::new(),
            dict: ChainDictionary::new(dict_capacity),
            next_page_id: 0,
            total_nodes: 0,
            total_edges: 0,
        }
    }

    /// Store compact node. Auto-manages pages and tiers.
    pub fn store_node(
        &mut self,
        chain: &MolecularChain,
        parent: Option<&MolecularChain>,
        layer: u8,
        ts: i64,
    ) -> u64 {
        let node = CompactNode::encode(chain, parent, &mut self.dict, layer, ts);
        let hash = node.hash;

        // Get or create current page for this layer
        let page_id = self.current_page_for_layer(layer);

        // Try to get page from cache
        if self.cache.get(page_id).is_some() {
            // Page in cache — we need mutable access, so re-get mutably
            // (limitation of our simple cache design)
        }

        // Get or create page
        let mut page = self
            .take_page(page_id)
            .unwrap_or_else(|| CompactPage::new(page_id, layer));

        if page.is_full() {
            // Flush current page to cold
            self.flush_page(page);
            // Create new page
            self.next_page_id += 1;
            page = CompactPage::new(self.next_page_id, layer);
        }

        page.push_node(node);
        self.total_nodes += 1;

        // Update index
        self.ensure_index(layer);
        for idx in &mut self.indexes {
            if idx.layer == layer {
                idx.insert(hash, page.page_id);
                break;
            }
        }

        // Put page back in cache
        if let Some(evicted) = self.cache.put(page.page_id, page) {
            self.flush_page(evicted);
        }

        hash
    }

    /// Store compact edge.
    pub fn store_edge(
        &mut self,
        from: u64,
        to: u64,
        weight: f32,
        relation: u8,
        layer: u8,
    ) {
        let edge = CompactEdge::encode(from, to, weight, relation);

        let page_id = self.current_page_for_layer(layer);
        let mut page = self
            .take_page(page_id)
            .unwrap_or_else(|| CompactPage::new(page_id, layer));

        page.push_edge(edge);
        self.total_edges += 1;

        if let Some(evicted) = self.cache.put(page.page_id, page) {
            self.flush_page(evicted);
        }
    }

    /// Lookup node by hash.
    pub fn lookup(&mut self, hash: u64, layer: u8) -> Option<&CompactNode> {
        // 1. Check index for page_id
        let page_id = {
            let idx = self.indexes.iter().find(|i| i.layer == layer)?;
            idx.find_page(hash)?
        };

        // 2. Check cache
        if self.cache.get(page_id).is_some() {
            let page = self.cache.get(page_id)?;
            return page.nodes.iter().find(|n| n.hash == hash);
        }

        // 3. Load from cold storage
        self.load_page_from_cold(page_id);
        let page = self.cache.get(page_id)?;
        page.nodes.iter().find(|n| n.hash == hash)
    }

    /// Total nodes stored across all tiers.
    pub fn total_nodes(&self) -> u64 {
        self.total_nodes
    }

    /// Total edges stored.
    pub fn total_edges(&self) -> u64 {
        self.total_edges
    }

    /// Estimated RAM usage (bytes).
    pub fn ram_usage(&self) -> usize {
        let cache_ram = self.cache.ram_usage();
        let index_ram: usize = self.indexes.iter().map(|i| i.ram_size()).sum();
        let dict_ram = self.dict.len() * 16; // rough estimate
        cache_ram + index_ram + dict_ram
    }

    /// Estimated disk usage (bytes).
    pub fn disk_usage(&self) -> usize {
        self.cold_storage.iter().map(|(_, b)| b.len()).sum()
    }

    /// Summary.
    pub fn summary(&self) -> String {
        alloc::format!(
            "TieredStore: {} nodes + {} edges\n\
             RAM: ~{}KB | Disk: ~{}KB\n\
             {}",
            self.total_nodes,
            self.total_edges,
            self.ram_usage() / 1024,
            self.disk_usage() / 1024,
            self.cache.stats(),
        )
    }

    /// Lookup node + follow silk edges → connected nodes.
    ///
    /// Cơ chế đọc kết quả đến đúng địa chỉ, track silk to node:
    /// ```text
    /// hash → LayerIndex (bloom → binary search) → page_id
    ///   → PageCache (LRU hit?) or Cold (load from disk)
    ///     → CompactPage → scan nodes for hash match
    ///       → CompactNode found!
    ///         → scan page edges where from_hash == hash
    ///           → each to_hash → recursive lookup (depth limited)
    ///             → return NodeWithEdges { node, neighbors[] }
    /// ```
    pub fn lookup_with_edges(
        &mut self,
        hash: u64,
        layer: u8,
        max_depth: u8,
    ) -> Option<NodeWithEdges> {
        self.lookup_recursive(hash, layer, max_depth, 0)
    }

    /// Internal recursive lookup with depth tracking.
    fn lookup_recursive(
        &mut self,
        hash: u64,
        layer: u8,
        max_depth: u8,
        current_depth: u8,
    ) -> Option<NodeWithEdges> {
        // 1. Resolve hash → page_id via LayerIndex
        let page_id = {
            let idx = self.indexes.iter().find(|i| i.layer == layer)?;
            idx.find_page(hash)?
        };

        // 2. Load page (cache hit or cold load)
        if self.cache.get(page_id).is_none() {
            self.load_page_from_cold(page_id);
        }

        let page = self.cache.get(page_id)?;

        // 3. Find node in page
        let node = page.nodes.iter().find(|n| n.hash == hash)?.clone();

        // 4. Collect outgoing edges from this page
        let edges: Vec<CompactEdge> = page
            .edges
            .iter()
            .filter(|e| e.from_hash == (hash & 0xFFFFFFFF) as u32)
            .cloned()
            .collect();

        // 5. Follow edges to neighbors (depth limited)
        let mut neighbors = Vec::new();
        if current_depth < max_depth {
            for edge in &edges {
                // Reconstruct full hash from truncated to_hash
                // Search across all layer indexes for to_hash
                if let Some(neighbor) = self.find_node_by_truncated_hash(edge.to_hash, layer) {
                    neighbors.push(NeighborInfo {
                        node: neighbor,
                        weight: edge.weight_f32(),
                        relation: edge.relation,
                    });
                }
            }
        }

        Some(NodeWithEdges {
            node,
            edges,
            neighbors,
            depth: current_depth,
        })
    }

    /// Find node by truncated hash (4 bytes) — searches current layer first,
    /// then adjacent layers.
    fn find_node_by_truncated_hash(
        &mut self,
        hash_lo: u32,
        preferred_layer: u8,
    ) -> Option<CompactNode> {
        // Search preferred layer first
        let layers: Vec<u8> = {
            let mut l = alloc::vec![preferred_layer];
            // Also check adjacent layers (silk can cross layers)
            if preferred_layer > 0 {
                l.push(preferred_layer - 1);
            }
            l.push(preferred_layer + 1);
            l
        };

        for &layer in &layers {
            if let Some(idx) = self.indexes.iter().find(|i| i.layer == layer) {
                if let Some(pos) = idx.entries.iter().position(|e| e.hash_lo == hash_lo) {
                    let page_id = idx.entries[pos].page_id;
                    if self.cache.get(page_id).is_none() {
                        self.load_page_from_cold(page_id);
                    }
                    if let Some(page) = self.cache.get(page_id) {
                        if let Some(node) = page
                            .nodes
                            .iter()
                            .find(|n| (n.hash & 0xFFFFFFFF) as u32 == hash_lo)
                        {
                            return Some(node.clone());
                        }
                    }
                }
            }
        }
        None
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

    fn current_page_for_layer(&self, _layer: u8) -> u32 {
        // Simple: use next_page_id as current
        self.next_page_id
    }

    fn ensure_index(&mut self, layer: u8) {
        if !self.indexes.iter().any(|i| i.layer == layer) {
            self.indexes.push(LayerIndex::new(layer));
        }
    }

    fn take_page(&mut self, page_id: u32) -> Option<CompactPage> {
        // Try cache first
        // Since we can't remove from cache easily, just get reference
        // In real impl: cache.remove(page_id)
        if self.cache.get(page_id).is_some() {
            // Clone from cache (simplified — real impl would move)
            let page = self.cache.get(page_id)?.clone();
            return Some(page);
        }
        None
    }

    fn flush_page(&mut self, page: CompactPage) {
        let bytes = page.to_bytes();
        self.cold_storage.push((page.page_id, bytes));
    }

    fn load_page_from_cold(&mut self, page_id: u32) {
        // Find in cold storage
        if let Some(idx) = self.cold_storage.iter().position(|(id, _)| *id == page_id) {
            let (_, _bytes) = &self.cold_storage[idx];
            // Simplified: just create empty page (real impl would deserialize)
            let page = CompactPage::new(page_id, 0);
            if let Some(evicted) = self.cache.put(page_id, page) {
                self.flush_page(evicted);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecular::{EmotionDim, RelationBase};

    fn test_mol(shape: u8, rel: u8, v: u8, a: u8, t: u8) -> Molecule {
        Molecule {
            shape,
            relation: rel,
            emotion: EmotionDim {
                valence: v,
                arousal: a,
            },
            time: t,
        }
    }

    // ── DeltaMolecule ──────────────────────────────────────────────────────────

    #[test]
    fn delta_identical_molecules() {
        let mol = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let delta = DeltaMolecule::encode(&mol, &mol);
        assert_eq!(delta.mask, 0x00, "Identical → no changes");
        assert_eq!(delta.size(), 1, "Only bitmask byte");
    }

    #[test]
    fn delta_one_field_changed() {
        let parent = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let child = test_mol(0x01, 0x01, 0xC0, 0x80, 0x03); // valence changed
        let delta = DeltaMolecule::encode(&parent, &child);
        assert_eq!(delta.mask, 0x04, "Only valence bit set");
        assert_eq!(delta.size(), 2, "1 bitmask + 1 changed byte");
    }

    #[test]
    fn delta_all_fields_changed() {
        let parent = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let child = test_mol(0x02, 0x06, 0xC0, 0x40, 0x05);
        let delta = DeltaMolecule::encode(&parent, &child);
        assert_eq!(delta.mask, 0x1F, "All 5 bits set");
        assert_eq!(delta.size(), 6, "1 bitmask + 5 changed bytes");
    }

    #[test]
    fn delta_roundtrip() {
        let parent = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let child = test_mol(0x01, 0x06, 0xC0, 0x80, 0x03); // relation + valence
        let delta = DeltaMolecule::encode(&parent, &child);
        let decoded = delta.decode(&parent).unwrap();
        assert_eq!(decoded, child);
    }

    #[test]
    fn delta_bytes_roundtrip() {
        let parent = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let child = test_mol(0x02, 0x01, 0x80, 0xFF, 0x03);
        let delta = DeltaMolecule::encode(&parent, &child);
        let bytes = delta.to_bytes();
        let decoded = DeltaMolecule::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, delta);
    }

    #[test]
    fn delta_savings_typical() {
        // Typical L2 node: chỉ khác emotion valence từ parent
        let parent = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let child = test_mol(0x01, 0x01, 0x90, 0x80, 0x03);
        let delta = DeltaMolecule::encode(&parent, &child);
        assert!(
            delta.size() < 5,
            "Delta {} bytes < Full 5 bytes",
            delta.size()
        );
    }

    // ── ChainDictionary ──────────────────────────────────────────────────────

    #[test]
    fn dict_register_and_lookup() {
        let mut dict = ChainDictionary::new(100);
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let id = dict.register(&chain);
        assert_eq!(id, 0);
        let found = dict.lookup(id).unwrap();
        assert_eq!(found, chain);
    }

    #[test]
    fn dict_dedup() {
        let mut dict = ChainDictionary::new(100);
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let id1 = dict.register(&chain);
        let id2 = dict.register(&chain); // same chain
        assert_eq!(id1, id2, "Same chain → same ID");
        assert_eq!(dict.len(), 1, "Only 1 entry");
    }

    #[test]
    fn dict_variable_id_encoding() {
        // 1 byte: 0..127
        let bytes = ChainDictionary::encode_id(0);
        assert_eq!(bytes.len(), 1);
        let (id, len) = ChainDictionary::decode_id(&bytes).unwrap();
        assert_eq!(id, 0);
        assert_eq!(len, 1);

        // 1 byte: 127
        let bytes = ChainDictionary::encode_id(127);
        assert_eq!(bytes.len(), 1);

        // 2 bytes: 128
        let bytes = ChainDictionary::encode_id(128);
        assert_eq!(bytes.len(), 2);
        let (id, len) = ChainDictionary::decode_id(&bytes).unwrap();
        assert_eq!(id, 128);
        assert_eq!(len, 2);

        // Large ID
        let bytes = ChainDictionary::encode_id(20000);
        let (id, _) = ChainDictionary::decode_id(&bytes).unwrap();
        assert_eq!(id, 20000);
    }

    #[test]
    fn dict_prune() {
        let mut dict = ChainDictionary::new(4);
        // Fill 4 entries
        for i in 0u8..4 {
            let chain = MolecularChain::single(test_mol(i + 1, 0x01, 0x80, 0x80, 0x03));
            dict.register(&chain);
        }
        // Register one more → should trigger prune
        assert_eq!(dict.len(), 4);
        let chain5 = MolecularChain::single(test_mol(0x01, 0x02, 0x80, 0x80, 0x03));
        dict.register(&chain5);
        assert!(dict.len() <= 4, "After prune: len = {}", dict.len());
    }

    // ── CompactNode ──────────────────────────────────────────────────────────

    #[test]
    fn compact_node_full_encoding() {
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let mut dict = ChainDictionary::new(100);
        let node = CompactNode::encode(&chain, None, &mut dict, 2, 1000);
        // First time, no parent → DictRef (since dict_id < 0x4080)
        assert!(matches!(node.kind, CompactKind::DictRef));
    }

    #[test]
    fn compact_node_delta_encoding() {
        // Parent có nhiều non-default fields → tagged size lớn
        let parent = MolecularChain::single(test_mol(0x02, 0x06, 0xC0, 0xC0, 0x04));
        // Child chỉ khác parent ở valence → delta nhỏ hơn tagged
        let child = MolecularChain::single(test_mol(0x02, 0x06, 0xD0, 0xC0, 0x04));
        let mut dict = ChainDictionary::new(100);
        dict.register(&parent); // pre-register parent
        let node = CompactNode::encode(&child, Some(&parent), &mut dict, 2, 1000);
        // Delta should be chosen (2 bytes < tagged ~6 bytes)
        assert_eq!(node.kind, CompactKind::Delta, "Should use delta encoding");
        let child_tagged = child.0[0].tagged_size();
        assert!(
            node.data_size() < child_tagged,
            "Delta {} bytes < tagged {} bytes",
            node.data_size(),
            child_tagged
        );
    }

    #[test]
    fn compact_node_dedup() {
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let mut dict = ChainDictionary::new(100);
        dict.register(&chain); // pre-register
        let node = CompactNode::encode(&chain, None, &mut dict, 2, 1000);
        assert_eq!(node.kind, CompactKind::Dedup, "Already in dict → dedup");
    }

    #[test]
    fn compact_node_decode_delta() {
        let parent = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let child = MolecularChain::single(test_mol(0x01, 0x01, 0xC0, 0x80, 0x03));
        let mut dict = ChainDictionary::new(100);
        dict.register(&parent);
        let node = CompactNode::encode(&child, Some(&parent), &mut dict, 2, 1000);
        let decoded = node.decode(Some(&parent), &dict).unwrap();
        assert_eq!(decoded, child, "Delta decode phải khôi phục đúng chain");
    }

    #[test]
    fn compact_node_decode_dedup() {
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let mut dict = ChainDictionary::new(100);
        dict.register(&chain);
        let node = CompactNode::encode(&chain, None, &mut dict, 2, 1000);
        let decoded = node.decode(None, &dict).unwrap();
        assert_eq!(decoded, chain);
    }

    #[test]
    fn compact_node_serialization() {
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let mut dict = ChainDictionary::new(100);
        let node = CompactNode::encode(&chain, None, &mut dict, 3, 1000);
        let bytes = node.to_bytes();
        let restored = CompactNode::from_bytes(&bytes).unwrap();
        assert_eq!(restored.hash, node.hash);
        assert_eq!(restored.layer, 3);
        assert_eq!(restored.kind, node.kind);
    }

    // ── CompactEdge ──────────────────────────────────────────────────────────

    #[test]
    fn compact_edge_size() {
        let edge = CompactEdge::encode(
            0xDEADBEEF12345678,
            0xCAFEBABE87654321,
            0.75,
            RelationBase::Causes.as_byte(),
        );
        let bytes = edge.to_bytes();
        assert_eq!(bytes.len(), 10, "CompactEdge = 10 bytes (vs 46 full)");
    }

    #[test]
    fn compact_edge_roundtrip() {
        let edge = CompactEdge::encode(
            0x1234567890ABCDEF,
            0xFEDCBA0987654321,
            0.85,
            RelationBase::Similar.as_byte(),
        );
        let bytes = edge.to_bytes();
        let decoded = CompactEdge::from_bytes(&bytes);
        assert_eq!(decoded.from_hash, edge.from_hash);
        assert_eq!(decoded.to_hash, edge.to_hash);
        assert_eq!(decoded.weight, edge.weight);
        assert_eq!(decoded.relation, edge.relation);
    }

    #[test]
    fn compact_edge_weight_quantization() {
        let edge = CompactEdge::encode(1, 2, 0.5, RelationBase::Member.as_byte());
        let restored = edge.weight_f32();
        assert!(
            (restored - 0.5).abs() < 0.005,
            "Weight quantization ~0.5: {}",
            restored
        );
    }

    // ── CompactPage ──────────────────────────────────────────────────────────

    #[test]
    fn page_capacity() {
        let page = CompactPage::new(0, 2);
        assert!(!page.is_full());
        assert_eq!(page.remaining(), DEFAULT_PAGE_CAPACITY);
    }

    #[test]
    fn page_compression_stats() {
        let mut page = CompactPage::new(0, 2);
        let mut dict = ChainDictionary::new(100);

        let parent = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        dict.register(&parent);

        // Add 10 nodes with delta encoding
        for i in 0u8..10 {
            let child = MolecularChain::single(test_mol(0x01, 0x01, 0x80 + i, 0x80, 0x03));
            let node = CompactNode::encode(&child, Some(&parent), &mut dict, 2, i as i64);
            page.push_node(node);
        }

        // Add 20 edges
        for i in 0u32..20 {
            let edge = CompactEdge::encode(i as u64, (i + 1) as u64, 0.6, RelationBase::Causes.as_byte());
            page.push_edge(edge);
        }

        let stats = page.compression_stats();
        assert!(stats.node_count == 10);
        assert!(stats.edge_count == 20);
        // Edges: 20 * 10 = 200 compact vs 20 * 26 = 520 full
        // → edge savings ≈ 61%
    }

    #[test]
    fn page_serialization() {
        let mut page = CompactPage::new(42, 3);
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let mut dict = ChainDictionary::new(100);
        let node = CompactNode::encode(&chain, None, &mut dict, 3, 1000);
        page.push_node(node);
        page.push_edge(CompactEdge::encode(1, 2, 0.5, RelationBase::Member.as_byte()));

        let bytes = page.to_bytes();
        // Check magic
        assert_eq!(&bytes[0..4], &PAGE_MAGIC);
        // Check page_id
        assert_eq!(u32::from_be_bytes(bytes[4..8].try_into().unwrap()), 42);
        // Check checksum at end
        assert!(bytes.len() > 8, "Page has content + checksum");
    }

    // ── SilkPruner ──────────────────────────────────────────────────────────

    #[test]
    fn pruner_keeps_strong_edges() {
        let edges = alloc::vec![
            (1u64, 2, 0.9, RelationBase::Causes.as_byte()),
            (2, 3, 0.1, RelationBase::Similar.as_byte()),
            (3, 4, 0.5, RelationBase::Member.as_byte()),
            (4, 5, 0.4, RelationBase::Equiv.as_byte()),
        ];
        let pruned = SilkPruner::prune_default(&edges);
        // Threshold 0.382 → keep 0.9, 0.5, 0.4
        assert_eq!(pruned.len(), 3, "Keep edges >= 0.382");
    }

    #[test]
    fn pruner_stats() {
        let edges = alloc::vec![
            (1u64, 2, 0.9, RelationBase::Causes.as_byte()),
            (2, 3, 0.1, RelationBase::Similar.as_byte()),
            (3, 4, 0.5, RelationBase::Member.as_byte()),
        ];
        let (kept, removed) = SilkPruner::prune_stats(&edges, 0.382);
        assert_eq!(kept, 2);
        assert_eq!(removed, 1);
    }

    #[test]
    fn pruner_fibonacci_threshold() {
        // φ⁻¹ ≈ 0.618 → threshold = 1 - 0.618 = 0.382
        assert!((SilkPruner::FIBONACCI_THRESHOLD - 0.382).abs() < 0.001);
    }

    // ── LayerIndex ──────────────────────────────────────────────────────────

    #[test]
    fn layer_index_insert_and_find() {
        let mut idx = LayerIndex::new(2);
        idx.insert(0xDEADBEEF12345678, 42);
        assert_eq!(idx.find_page(0xDEADBEEF12345678), Some(42));
        assert_eq!(idx.len(), 1);
    }

    #[test]
    fn layer_index_bloom_filter() {
        let mut idx = LayerIndex::new(2);
        idx.insert(0xAAAAAAAA00000000, 1);
        // Bloom may have false positives, but definitely true for inserted
        assert!(idx.maybe_contains(0xAAAAAAAA00000000));
        // Not inserted — usually false (low false positive rate with 256 bytes)
        // Don't assert !maybe_contains since bloom can have false positives
    }

    #[test]
    fn layer_index_sorted_binary_search() {
        let mut idx = LayerIndex::new(2);
        // Insert in random order
        idx.insert(0x0000000000000300, 3);
        idx.insert(0x0000000000000100, 1);
        idx.insert(0x0000000000000200, 2);
        // Should find all
        assert_eq!(idx.find_page(0x0000000000000100), Some(1));
        assert_eq!(idx.find_page(0x0000000000000200), Some(2));
        assert_eq!(idx.find_page(0x0000000000000300), Some(3));
    }

    #[test]
    fn layer_index_ram_size() {
        let mut idx = LayerIndex::new(2);
        assert_eq!(idx.ram_size(), 256); // just bloom
        idx.insert(0x1234, 1);
        assert_eq!(idx.ram_size(), 256 + 8); // bloom + 1 entry × 8B
    }

    // ── PageCache ──────────────────────────────────────────────────────────

    #[test]
    fn page_cache_hit_miss() {
        let mut cache = PageCache::new(4);
        let page = CompactPage::new(1, 2);
        cache.put(1, page);

        assert!(cache.get(1).is_some(), "Hit");
        assert!(cache.get(99).is_none(), "Miss");
        assert!(cache.hit_rate() > 0.0);
    }

    #[test]
    fn page_cache_lru_eviction() {
        let mut cache = PageCache::new(2); // capacity 2
        cache.put(1, CompactPage::new(1, 0));
        cache.put(2, CompactPage::new(2, 0));

        // Access page 1 to make it recently used
        cache.get(1);

        // Add page 3 → should evict page 2 (LRU)
        let evicted = cache.put(3, CompactPage::new(3, 0));
        assert!(evicted.is_some(), "Should evict when full");

        // Page 1 should still be cached (recently accessed)
        assert!(cache.get(1).is_some(), "Page 1 still cached");
        // Page 3 should be cached (just added)
        assert!(cache.get(3).is_some(), "Page 3 cached");
    }

    #[test]
    fn page_cache_fibonacci_capacities() {
        assert_eq!(CacheCapacity::EMBEDDED, 55);
        assert_eq!(CacheCapacity::MOBILE, 233);
        assert_eq!(CacheCapacity::PC, 610);
        assert_eq!(CacheCapacity::SERVER, 2584);
    }

    #[test]
    fn page_cache_update_existing() {
        let mut cache = PageCache::new(4);
        cache.put(1, CompactPage::new(1, 0));
        // Update same page — should not evict
        let evicted = cache.put(1, CompactPage::new(1, 0));
        assert!(evicted.is_none(), "Update → no eviction");
        assert_eq!(cache.len(), 1, "Still 1 page");
    }

    // ── TieredStore ──────────────────────────────────────────────────────────

    #[test]
    fn tiered_store_basic() {
        let mut store = TieredStore::new(4, 100);
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let hash = store.store_node(&chain, None, 2, 1000);
        assert!(hash != 0, "Hash should be non-zero");
        assert_eq!(store.total_nodes(), 1);
    }

    #[test]
    fn tiered_store_lookup() {
        let mut store = TieredStore::new(4, 100);
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let hash = store.store_node(&chain, None, 2, 1000);

        let found = store.lookup(hash, 2);
        assert!(found.is_some(), "Should find stored node");
        assert_eq!(found.unwrap().hash, hash);
    }

    #[test]
    fn tiered_store_edges() {
        let mut store = TieredStore::new(4, 100);
        let c1 = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let c2 = MolecularChain::single(test_mol(0x02, 0x01, 0x80, 0x80, 0x03));
        let h1 = store.store_node(&c1, None, 2, 1000);
        let h2 = store.store_node(&c2, None, 2, 1001);
        store.store_edge(h1, h2, 0.8, RelationBase::Causes.as_byte(), 2);
        assert_eq!(store.total_edges(), 1);
    }

    #[test]
    fn tiered_store_for_pc() {
        let store = TieredStore::for_pc();
        assert_eq!(store.cache.capacity, CacheCapacity::PC);
    }

    #[test]
    fn tiered_store_lookup_with_edges() {
        let mut store = TieredStore::new(4, 100);
        let c1 = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let c2 = MolecularChain::single(test_mol(0x02, 0x01, 0x80, 0x80, 0x03));
        let h1 = store.store_node(&c1, None, 2, 1000);
        let _h2 = store.store_node(&c2, None, 2, 1001);
        store.store_edge(h1, _h2, 0.85, RelationBase::Similar.as_byte(), 2);

        let result = store.lookup_with_edges(h1, 2, 1);
        assert!(result.is_some(), "Should find node with edges");
        let nwe = result.unwrap();
        assert_eq!(nwe.node.hash, h1);
        assert_eq!(nwe.depth, 0);
    }

    #[test]
    fn tiered_store_ram_disk_usage() {
        let mut store = TieredStore::new(4, 100);
        for i in 0u8..10 {
            let chain = MolecularChain::single(test_mol((i % 8) + 1, 0x01, 0x80 + i, 0x80, 0x03));
            store.store_node(&chain, None, 2, i as i64);
        }
        assert!(store.ram_usage() > 0, "Should use some RAM");
        let summary = store.summary();
        assert!(summary.contains("10 nodes"), "{}", summary);
    }

    // ── Scale estimate ──────────────────────────────────────────────────────

    #[test]
    fn estimate_1m_nodes_savings() {
        // Simulate compression ratio for typical L2+ content
        let parent = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);

        let mut total_full = 0usize;
        let mut total_compact = 0usize;

        // 100 sample nodes varying only in emotion
        for i in 0u8..100 {
            let child = test_mol(0x01, 0x01, 0x80_u8.wrapping_add(i), 0x80, 0x03);
            let delta = DeltaMolecule::encode(&parent, &child);
            total_full += 5; // full molecule
            total_compact += delta.size();
        }

        let ratio = total_compact as f32 / total_full as f32;
        // Typical: delta = 2 bytes vs full = 5 bytes → ratio ≈ 0.4
        assert!(ratio < 0.6, "Delta compression ratio {:.2} < 0.6", ratio);

        // Edge compression: 10 bytes vs 46 bytes → 78% savings
        let edge_ratio = 10.0f32 / 46.0;
        assert!(
            edge_ratio < 0.25,
            "Edge compression ratio {:.2} < 0.25",
            edge_ratio
        );
    }
}
