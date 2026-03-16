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
use alloc::vec::Vec;
use alloc::string::String;

use crate::molecular::{Molecule, MolecularChain, ShapeBase, RelationBase, EmotionDim, TimeDim};
use crate::hash::fnv1a;

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
            data.push(child.shape.as_byte());
        }
        if child.relation != parent.relation {
            mask |= 0x02;
            data.push(child.relation.as_byte());
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
            data.push(child.time.as_byte());
        }

        Self { mask, data }
    }

    /// Decode: parent + delta → child molecule.
    pub fn decode(&self, parent: &Molecule) -> Option<Molecule> {
        let mut idx = 0usize;
        let shape = if self.mask & 0x01 != 0 {
            let b = *self.data.get(idx)?;
            idx += 1;
            ShapeBase::from_byte(b)?
        } else {
            parent.shape
        };
        let relation = if self.mask & 0x02 != 0 {
            let b = *self.data.get(idx)?;
            idx += 1;
            RelationBase::from_byte(b)?
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
            TimeDim::from_byte(b)?
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
        if b.is_empty() { return None; }
        let mask = b[0];
        let expected_len = mask.count_ones() as usize;
        if b.len() < 1 + expected_len { return None; }
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
            chain_bytes: chain.to_bytes(),
            usage_count: 1,
        });
        id
    }

    /// Lookup dictionary ID → chain.
    pub fn lookup(&self, id: u32) -> Option<MolecularChain> {
        let entry = self.entries.get(id as usize)?;
        MolecularChain::from_bytes(&entry.chain_bytes)
    }

    /// Lookup hash → dictionary ID.
    pub fn lookup_hash(&self, hash: u64) -> Option<u32> {
        self.entries.iter().position(|e| e.hash == hash).map(|i| i as u32)
    }

    /// Số entries.
    pub fn len(&self) -> usize { self.entries.len() }

    /// Dictionary rỗng?
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    /// Prune: xóa 25% entries ít dùng nhất.
    fn prune(&mut self) {
        if self.entries.len() < 4 { return; }

        // Sort by usage (ascending) → remove bottom 25%
        let mut indexed: Vec<(usize, u32)> = self.entries.iter()
            .enumerate()
            .map(|(i, e)| (i, e.usage_count))
            .collect();
        indexed.sort_by_key(|&(_, count)| count);

        let remove_count = self.entries.len() / 4;
        let to_remove: Vec<usize> = indexed.iter()
            .take(remove_count)
            .map(|&(i, _)| i)
            .collect();

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
        if bytes.is_empty() { return None; }
        let first = bytes[0];
        if first < 0x80 {
            Some((first as u32, 1))
        } else if first < 0xC0 {
            if bytes.len() < 2 { return None; }
            let id = ((first as u32 & 0x3F) << 8) | bytes[1] as u32;
            Some((id + 0x80, 2))
        } else {
            if bytes.len() < 3 { return None; }
            let id = ((first as u32 & 0x3F) << 16)
                   | ((bytes[1] as u32) << 8)
                   | bytes[2] as u32;
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
    Full       = 0x00,
    /// Delta từ parent: bitmask + changed bytes
    Delta      = 0x01,
    /// Dictionary reference: variable-length ID
    DictRef    = 0x02,
    /// Duplicate: chỉ hash (đã tồn tại)
    Dedup      = 0x03,
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

                // Nếu delta nhỏ hơn full → dùng delta
                if delta.size() < 5 {
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
        if dict_id < 0x4080 { // 2 bytes hoặc ít hơn
            return Self {
                hash,
                parent_hash: parent.map_or(0, |p| p.chain_hash()),
                layer,
                kind: CompactKind::DictRef,
                data: ChainDictionary::encode_id(dict_id),
                created_at: ts,
            };
        }

        // 4. Full encoding (fallback)
        Self {
            hash,
            parent_hash: parent.map_or(0, |p| p.chain_hash()),
            layer,
            kind: CompactKind::Full,
            data: chain.to_bytes(),
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
            CompactKind::Full => {
                MolecularChain::from_bytes(&self.data)
            }
            CompactKind::Delta => {
                let parent_chain = parent?;
                if parent_chain.is_empty() { return None; }
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
        if b.len() < 28 { return None; } // minimum header
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
        if b.len() < 28 + data_len { return None; }
        let data = b[28..28 + data_len].to_vec();
        Some(Self { hash, parent_hash, layer, kind, data, created_at })
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
    pub fn encode(from: u64, to: u64, weight: f32, relation: RelationBase) -> Self {
        Self {
            from_hash: (from & 0xFFFFFFFF) as u32,
            to_hash: (to & 0xFFFFFFFF) as u32,
            weight: (weight.clamp(0.0, 1.0) * 255.0) as u8,
            relation: relation.as_byte(),
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
            from_hash: u32::from_be_bytes(b[0..4].try_into().unwrap()),
            to_hash: u32::from_be_bytes(b[4..8].try_into().unwrap()),
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
        if self.is_full() { return false; }
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
    pub fn prune(edges: &[(u64, u64, f32, RelationBase)], threshold: f32) -> Vec<CompactEdge> {
        edges.iter()
            .filter(|&&(_, _, w, _)| w >= threshold)
            .map(|&(from, to, w, rel)| CompactEdge::encode(from, to, w, rel))
            .collect()
    }

    /// Prune với Fibonacci threshold mặc định.
    pub fn prune_default(edges: &[(u64, u64, f32, RelationBase)]) -> Vec<CompactEdge> {
        Self::prune(edges, Self::FIBONACCI_THRESHOLD)
    }

    /// Thống kê prune: bao nhiêu edges bị loại.
    pub fn prune_stats(edges: &[(u64, u64, f32, RelationBase)], threshold: f32) -> (usize, usize) {
        let total = edges.len();
        let kept = edges.iter().filter(|&&(_, _, w, _)| w >= threshold).count();
        (kept, total - kept)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecular::{ShapeBase, RelationBase, EmotionDim, TimeDim};

    fn test_mol(shape: u8, rel: u8, v: u8, a: u8, t: u8) -> Molecule {
        Molecule {
            shape: ShapeBase::from_byte(shape).unwrap(),
            relation: RelationBase::from_byte(rel).unwrap(),
            emotion: EmotionDim { valence: v, arousal: a },
            time: TimeDim::from_byte(t).unwrap(),
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
        let child  = test_mol(0x01, 0x01, 0xC0, 0x80, 0x03); // valence changed
        let delta = DeltaMolecule::encode(&parent, &child);
        assert_eq!(delta.mask, 0x04, "Only valence bit set");
        assert_eq!(delta.size(), 2, "1 bitmask + 1 changed byte");
    }

    #[test]
    fn delta_all_fields_changed() {
        let parent = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let child  = test_mol(0x02, 0x06, 0xC0, 0x40, 0x05);
        let delta = DeltaMolecule::encode(&parent, &child);
        assert_eq!(delta.mask, 0x1F, "All 5 bits set");
        assert_eq!(delta.size(), 6, "1 bitmask + 5 changed bytes");
    }

    #[test]
    fn delta_roundtrip() {
        let parent = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let child  = test_mol(0x01, 0x06, 0xC0, 0x80, 0x03); // relation + valence
        let delta = DeltaMolecule::encode(&parent, &child);
        let decoded = delta.decode(&parent).unwrap();
        assert_eq!(decoded, child);
    }

    #[test]
    fn delta_bytes_roundtrip() {
        let parent = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let child  = test_mol(0x02, 0x01, 0x80, 0xFF, 0x03);
        let delta = DeltaMolecule::encode(&parent, &child);
        let bytes = delta.to_bytes();
        let decoded = DeltaMolecule::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, delta);
    }

    #[test]
    fn delta_savings_typical() {
        // Typical L2 node: chỉ khác emotion valence từ parent
        let parent = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let child  = test_mol(0x01, 0x01, 0x90, 0x80, 0x03);
        let delta = DeltaMolecule::encode(&parent, &child);
        assert!(delta.size() < 5, "Delta {} bytes < Full 5 bytes", delta.size());
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
        let parent = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let child  = MolecularChain::single(test_mol(0x01, 0x01, 0x90, 0x80, 0x03));
        let mut dict = ChainDictionary::new(100);
        dict.register(&parent); // pre-register parent
        let node = CompactNode::encode(&child, Some(&parent), &mut dict, 2, 1000);
        // Delta should be chosen (2 bytes < 5 bytes full)
        assert_eq!(node.kind, CompactKind::Delta, "Should use delta encoding");
        assert!(node.data_size() < 5, "Delta data < full 5 bytes");
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
        let child  = MolecularChain::single(test_mol(0x01, 0x01, 0xC0, 0x80, 0x03));
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
            0xDEADBEEF12345678, 0xCAFEBABE87654321,
            0.75, RelationBase::Causes,
        );
        let bytes = edge.to_bytes();
        assert_eq!(bytes.len(), 10, "CompactEdge = 10 bytes (vs 46 full)");
    }

    #[test]
    fn compact_edge_roundtrip() {
        let edge = CompactEdge::encode(
            0x1234567890ABCDEF, 0xFEDCBA0987654321,
            0.85, RelationBase::Similar,
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
        let edge = CompactEdge::encode(1, 2, 0.5, RelationBase::Member);
        let restored = edge.weight_f32();
        assert!((restored - 0.5).abs() < 0.005, "Weight quantization ~0.5: {}", restored);
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
            let edge = CompactEdge::encode(i as u64, (i + 1) as u64, 0.6, RelationBase::Causes);
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
        page.push_edge(CompactEdge::encode(1, 2, 0.5, RelationBase::Member));

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
            (1u64, 2, 0.9, RelationBase::Causes),
            (2, 3, 0.1, RelationBase::Similar),
            (3, 4, 0.5, RelationBase::Member),
            (4, 5, 0.4, RelationBase::Equiv),
        ];
        let pruned = SilkPruner::prune_default(&edges);
        // Threshold 0.382 → keep 0.9, 0.5, 0.4
        assert_eq!(pruned.len(), 3, "Keep edges >= 0.382");
    }

    #[test]
    fn pruner_stats() {
        let edges = alloc::vec![
            (1u64, 2, 0.9, RelationBase::Causes),
            (2, 3, 0.1, RelationBase::Similar),
            (3, 4, 0.5, RelationBase::Member),
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
        assert!(edge_ratio < 0.25, "Edge compression ratio {:.2} < 0.25", edge_ratio);
    }
}
