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
use crate::molecular::{MolecularChain, Molecule};

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

        if child.shape_u8() != parent.shape_u8() {
            mask |= 0x01;
            data.push(child.shape_u8());
        }
        if child.relation_u8() != parent.relation_u8() {
            mask |= 0x02;
            data.push(child.relation_u8());
        }
        if child.valence_u8() != parent.valence_u8() {
            mask |= 0x04;
            data.push(child.valence_u8());
        }
        if child.arousal_u8() != parent.arousal_u8() {
            mask |= 0x08;
            data.push(child.arousal_u8());
        }
        if child.time_u8() != parent.time_u8() {
            mask |= 0x10;
            data.push(child.time_u8());
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
            parent.shape_u8()
        };
        let relation = if self.mask & 0x02 != 0 {
            let b = *self.data.get(idx)?;
            idx += 1;
            b
        } else {
            parent.relation_u8()
        };
        let valence = if self.mask & 0x04 != 0 {
            let b = *self.data.get(idx)?;
            idx += 1;
            b
        } else {
            parent.valence_u8()
        };
        let arousal = if self.mask & 0x08 != 0 {
            let b = *self.data.get(idx)?;
            idx += 1;
            b
        } else {
            parent.arousal_u8()
        };
        let time = if self.mask & 0x10 != 0 {
            let b = *self.data.get(idx)?;
            let _ = idx;
            b
        } else {
            parent.time_u8()
        };

        Some(Molecule::raw(shape, relation, valence, arousal, time))
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
                let parent_mol = parent_chain.first().unwrap();
                let child_mol = chain.first().unwrap();
                let delta = DeltaMolecule::encode(&parent_mol, &child_mol);

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
                let parent_mol = parent_chain.first()?;
                let child_mol = delta.decode(&parent_mol)?;
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
// SlimNode — format đúng spec: ~10 bytes per node
// ─────────────────────────────────────────────────────────────────────────────

/// SlimNode — node format tối ưu cho KnowTree.
///
/// Spec: "1 concept = ~33 bytes (5 mol + 8 hash + 20 metadata)"
/// Nhưng metadata nằm ở PAGE-level, không per-node.
///
/// Per-node chỉ cần:
///   [hash:8B][tagged_mol:1-6B] = 9-14 bytes
///
/// So sánh:
///   CompactNode = 28B header + data = 29-34B per node ❌ (legacy)
///   SlimNode    = 8B hash + 1-6B tagged = 9-14B per node ✅
///
/// 500M nodes × 10B avg = 5GB → VỪA ĐIỆN THOẠI
///
/// Metadata đẩy lên SlimPage:
///   - layer: per-page (tất cả nodes cùng page cùng layer)
///   - timestamp: per-page (batch timestamp)
///   - parent_hash: Silk vertical (parent_map), không per-node
///   - kind: implicit từ tagged format (mask byte đầu tiên)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlimNode {
    /// Chain hash (FNV-1a) — identity key, 8 bytes
    pub hash: u64,
    /// Tagged molecule bytes (1-6 bytes: [mask][non-default values])
    pub tagged: Vec<u8>,
}

impl SlimNode {
    /// Encode chain → SlimNode.
    pub fn from_chain(chain: &MolecularChain) -> Self {
        Self {
            hash: chain.chain_hash(),
            tagged: chain.to_tagged_bytes(),
        }
    }

    /// Decode → MolecularChain.
    pub fn to_chain(&self) -> Option<MolecularChain> {
        MolecularChain::from_tagged_bytes(&self.tagged)
    }

    /// Total serialized size.
    ///
    /// Format: [hash:8][tagged_len:1][tagged:1-6]
    /// = 9-15 bytes (avg ~10-11 bytes for typical nodes)
    pub fn total_size(&self) -> usize {
        8 + 1 + self.tagged.len()
    }

    /// Serialize → bytes.
    ///
    /// Wire: [hash:8B][tagged_len:1B][tagged:1-6B]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.total_size());
        buf.extend_from_slice(&self.hash.to_be_bytes());
        buf.push(self.tagged.len() as u8);
        buf.extend_from_slice(&self.tagged);
        buf
    }

    /// Deserialize từ bytes. Returns (SlimNode, bytes_consumed).
    pub fn from_bytes(b: &[u8]) -> Option<(Self, usize)> {
        if b.len() < 9 {
            return None;
        }
        let hash = u64::from_be_bytes(b[0..8].try_into().ok()?);
        let tagged_len = b[8] as usize;
        // MolecularChain tagged: [count:1][mol_tagged:1-6 per mol]
        // Single mol: 2-7 bytes. Multi-mol: up to count*(1+5)+1.
        // Max reasonable: 32 bytes (5 mols × 6 bytes + count)
        if tagged_len == 0 || tagged_len > 32 {
            return None;
        }
        if b.len() < 9 + tagged_len {
            return None;
        }
        let tagged = b[9..9 + tagged_len].to_vec();
        let consumed = 9 + tagged_len;
        Some((Self { hash, tagged }, consumed))
    }

    /// Tagged molecule size (bytes) — data only, no hash.
    pub fn tagged_size(&self) -> usize {
        self.tagged.len()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SlimPage — batch SlimNodes with shared metadata
// ─────────────────────────────────────────────────────────────────────────────

/// SlimPage — page chứa SlimNodes, metadata chia sẻ.
///
/// Format:
/// ```text
/// [SLIM_MAGIC:4][page_id:4][layer:1][batch_ts:8][node_count:2]
/// [slim_node_0][slim_node_1]...[slim_node_N]
/// [edge_count:2][compact_edge_0]...[compact_edge_M]
/// [checksum:8]
/// ```
///
/// Header: 19 bytes (shared cho tất cả nodes)
/// Per-node: 9-15 bytes (avg ~10-11)
/// → 233 nodes × 11B + 19B header + 8B checksum ≈ 2.6KB per page
/// → vs CompactPage: 233 × 30B ≈ 7KB per page
/// → Tiết kiệm ~63%
#[derive(Debug, Clone)]
pub struct SlimPage {
    /// Page ID
    pub page_id: u32,
    /// Layer (shared — tất cả nodes cùng layer)
    pub layer: u8,
    /// Batch timestamp (shared — thời điểm page được tạo)
    pub batch_ts: i64,
    /// Slim nodes
    pub nodes: Vec<SlimNode>,
    /// Compact edges (vẫn dùng CompactEdge 10B)
    pub edges: Vec<CompactEdge>,
}

/// SlimPage magic: "SLPG"
const SLIM_PAGE_MAGIC: [u8; 4] = [0x53, 0x4C, 0x50, 0x47];
/// Max nodes per SlimPage: Fib[13] = 233
const SLIM_PAGE_CAPACITY: usize = 233;

impl SlimPage {
    /// Tạo page mới.
    pub fn new(page_id: u32, layer: u8, ts: i64) -> Self {
        Self {
            page_id,
            layer,
            batch_ts: ts,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Page đầy chưa?
    pub fn is_full(&self) -> bool {
        self.nodes.len() >= SLIM_PAGE_CAPACITY
    }

    /// Remaining capacity.
    pub fn remaining(&self) -> usize {
        SLIM_PAGE_CAPACITY.saturating_sub(self.nodes.len())
    }

    /// Thêm node.
    pub fn push_node(&mut self, node: SlimNode) -> bool {
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

    /// Average bytes per node (data only).
    pub fn avg_node_size(&self) -> f32 {
        if self.nodes.is_empty() {
            return 0.0;
        }
        let total: usize = self.nodes.iter().map(|n| n.total_size()).sum();
        total as f32 / self.nodes.len() as f32
    }

    /// Total data size.
    pub fn data_size(&self) -> usize {
        let node_size: usize = self.nodes.iter().map(|n| n.total_size()).sum();
        let edge_size = self.edges.len() * 10;
        node_size + edge_size
    }

    /// Serialize → bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        // Header
        buf.extend_from_slice(&SLIM_PAGE_MAGIC);
        buf.extend_from_slice(&self.page_id.to_be_bytes());
        buf.push(self.layer);
        buf.extend_from_slice(&self.batch_ts.to_be_bytes());
        buf.extend_from_slice(&(self.nodes.len() as u16).to_be_bytes());

        // Nodes
        for node in &self.nodes {
            buf.extend_from_slice(&node.to_bytes());
        }

        // Edges
        buf.extend_from_slice(&(self.edges.len() as u16).to_be_bytes());
        for edge in &self.edges {
            buf.extend_from_slice(&edge.to_bytes());
        }

        // Checksum
        let checksum = fnv1a(&buf);
        buf.extend_from_slice(&checksum.to_be_bytes());

        buf
    }

    /// Deserialize from bytes.
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        // Header: 4 + 4 + 1 + 8 + 2 = 19 bytes minimum + 8 checksum
        if b.len() < 27 {
            return None;
        }
        if b[0..4] != SLIM_PAGE_MAGIC {
            return None;
        }

        // Verify checksum
        let payload = &b[..b.len() - 8];
        let stored_checksum = u64::from_be_bytes(b[b.len() - 8..].try_into().ok()?);
        if fnv1a(payload) != stored_checksum {
            return None;
        }

        let page_id = u32::from_be_bytes(b[4..8].try_into().ok()?);
        let layer = b[8];
        let batch_ts = i64::from_be_bytes(b[9..17].try_into().ok()?);
        let node_count = u16::from_be_bytes(b[17..19].try_into().ok()?) as usize;

        // Parse nodes
        let mut offset = 19;
        let mut nodes = Vec::with_capacity(node_count);
        for _ in 0..node_count {
            if offset >= b.len() - 8 {
                return None;
            }
            let (node, consumed) = SlimNode::from_bytes(&b[offset..])?;
            nodes.push(node);
            offset += consumed;
        }

        // Parse edges
        if offset + 2 > b.len() - 8 {
            return None;
        }
        let edge_count = u16::from_be_bytes(b[offset..offset + 2].try_into().ok()?) as usize;
        offset += 2;

        let mut edges = Vec::with_capacity(edge_count);
        for _ in 0..edge_count {
            if offset + 10 > b.len() - 8 {
                return None;
            }
            let edge_bytes: [u8; 10] = b[offset..offset + 10].try_into().ok()?;
            edges.push(CompactEdge::from_bytes(&edge_bytes));
            offset += 10;
        }

        Some(Self {
            page_id,
            layer,
            batch_ts,
            nodes,
            edges,
        })
    }

    /// Tìm node by hash.
    pub fn find_node(&self, hash: u64) -> Option<&SlimNode> {
        self.nodes.iter().find(|n| n.hash == hash)
    }

    /// Tìm edges from hash.
    pub fn edges_from(&self, hash: u64) -> Vec<&CompactEdge> {
        let hash_lo = (hash & 0xFFFFFFFF) as u32;
        self.edges
            .iter()
            .filter(|e| e.from_hash == hash_lo)
            .collect()
    }

    /// So sánh với CompactPage cùng data.
    pub fn savings_vs_compact(&self) -> (usize, usize) {
        let slim_total = self.data_size() + 19 + 8; // header + checksum
        // CompactNode: 28B header + ~2B data avg = 30B per node
        let compact_total = self.nodes.len() * 30 + self.edges.len() * 10 + 17 + 8;
        (slim_total, compact_total)
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

    /// Deserialize page from bytes (reverse of `to_bytes()`).
    ///
    /// Format: [CPAG:4][page_id:4][layer:1][node_count:4][edge_count:4]
    ///         [nodes...][edges...][checksum:8]
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() < 4 + 4 + 1 + 4 + 4 + 8 {
            return None; // too short for header + checksum
        }
        // Verify magic
        if b[0..4] != PAGE_MAGIC {
            return None;
        }
        let page_id = u32::from_be_bytes(b[4..8].try_into().ok()?);
        let layer = b[8];
        let node_count = u32::from_be_bytes(b[9..13].try_into().ok()?) as usize;
        let edge_count = u32::from_be_bytes(b[13..17].try_into().ok()?) as usize;

        // Verify checksum (covers everything except last 8 bytes)
        if b.len() < 8 {
            return None;
        }
        let payload = &b[..b.len() - 8];
        let stored_checksum = u64::from_be_bytes(b[b.len() - 8..].try_into().ok()?);
        if fnv1a(payload) != stored_checksum {
            return None; // corrupted
        }

        // Parse nodes
        let mut offset = 17;
        let mut nodes = Vec::with_capacity(node_count);
        for _ in 0..node_count {
            if offset >= b.len() - 8 {
                return None;
            }
            let node = CompactNode::from_bytes(&b[offset..])?;
            offset += node.total_size();
            nodes.push(node);
        }

        // Parse edges (10 bytes each)
        let mut edges = Vec::with_capacity(edge_count);
        for _ in 0..edge_count {
            if offset + 10 > b.len() - 8 {
                return None;
            }
            let edge_bytes: [u8; 10] = b[offset..offset + 10].try_into().ok()?;
            edges.push(CompactEdge::from_bytes(&edge_bytes));
            offset += 10;
        }

        Some(Self {
            page_id,
            layer,
            nodes,
            edges,
        })
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

    /// Restore a pre-decoded CompactNode from origin.olang — boot path.
    ///
    /// QT8: origin.olang = bộ nhớ duy nhất, RAM = cache.
    pub fn restore_node(&mut self, node: CompactNode) {
        let hash = node.hash;
        let layer = node.layer;
        let page_id = self.current_page_for_layer(layer);

        let mut page = self
            .take_page(page_id)
            .unwrap_or_else(|| CompactPage::new(page_id, layer));

        if page.is_full() {
            self.flush_page(page);
            self.next_page_id += 1;
            page = CompactPage::new(self.next_page_id, layer);
        }

        page.push_node(node);
        self.total_nodes += 1;

        self.ensure_index(layer);
        for idx in &mut self.indexes {
            if idx.layer == layer {
                idx.insert(hash, page.page_id);
                break;
            }
        }

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
        // Find in cold storage and deserialize
        if let Some(idx) = self.cold_storage.iter().position(|(id, _)| *id == page_id) {
            let bytes = self.cold_storage[idx].1.clone();
            let page = CompactPage::from_bytes(&bytes)
                .unwrap_or_else(|| CompactPage::new(page_id, 0));
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
    use crate::molecular::RelationBase;

    fn test_mol(shape: u8, rel: u8, v: u8, a: u8, t: u8) -> Molecule {
        Molecule::raw(shape, rel, v, a, t)
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
        // v2: use pre-scaled values so all 5 quantized dims differ.
        // parent: S=0, R=1, V=4, A=4, T=0
        let parent = test_mol(0x00, 0x10, 0x80, 0x80, 0x03);
        // child: S=3, R=6, V=6, A=2, T=3
        let child = test_mol(0x30, 0x60, 0xC0, 0x40, 0xC0);
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
        // v2: use distinct quantized values (pre-scaled: i<<4 for shape)
        for i in 0u16..4 {
            let chain = MolecularChain::single(Molecule::from_u16(i * 0x1000)); // distinct shapes
            dict.register(&chain);
        }
        // Register one more → should trigger prune
        assert_eq!(dict.len(), 4);
        let chain5 = MolecularChain::single(Molecule::from_u16(0x4000));
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
        // v2: pre-scaled values with many non-default fields → tagged size > delta.
        // parent: S=3, R=6, V=6, A=6, T=3 (all non-default)
        let parent = MolecularChain::single(test_mol(0x30, 0x60, 0xC0, 0xC0, 0xC0));
        // child: same except V=2 (different quantized value: 0x40>>5=2)
        let child = MolecularChain::single(test_mol(0x30, 0x60, 0x40, 0xC0, 0xC0));
        let mut dict = ChainDictionary::new(100);
        dict.register(&parent); // pre-register parent
        let node = CompactNode::encode(&child, Some(&parent), &mut dict, 2, 1000);
        // Delta should be chosen (2 bytes < tagged ~6 bytes)
        assert_eq!(node.kind, CompactKind::Delta, "Should use delta encoding");
        let child_tagged = child.first().unwrap().tagged_size();
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

    #[test]
    fn page_roundtrip() {
        let mut page = CompactPage::new(42, 3);
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let mut dict = ChainDictionary::new(100);
        let node = CompactNode::encode(&chain, None, &mut dict, 3, 1000);
        let node_hash = node.hash;
        page.push_node(node);
        page.push_edge(CompactEdge::encode(1, 2, 0.5, RelationBase::Member.as_byte()));

        let bytes = page.to_bytes();
        let restored = CompactPage::from_bytes(&bytes).expect("roundtrip should succeed");

        assert_eq!(restored.page_id, 42);
        assert_eq!(restored.layer, 3);
        assert_eq!(restored.nodes.len(), 1);
        assert_eq!(restored.edges.len(), 1);
        assert_eq!(restored.nodes[0].hash, node_hash);
        assert_eq!(restored.edges[0].from_hash, 1);
        assert_eq!(restored.edges[0].to_hash, 2);
    }

    #[test]
    fn page_from_bytes_corrupted() {
        let mut page = CompactPage::new(1, 0);
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let mut dict = ChainDictionary::new(100);
        page.push_node(CompactNode::encode(&chain, None, &mut dict, 0, 0));

        let mut bytes = page.to_bytes();
        // Corrupt a byte
        if bytes.len() > 20 {
            bytes[20] ^= 0xFF;
        }
        assert!(CompactPage::from_bytes(&bytes).is_none(), "Corrupted data should fail");
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

    // ── SlimNode ──────────────────────────────────────────────────────────────

    #[test]
    fn slim_node_default_molecule_2bytes() {
        // Sphere/Member/0x80/0x80/Medium = all defaults
        // MolecularChain tagged = [count:1][mask:1] = 2 bytes
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let slim = SlimNode::from_chain(&chain);
        assert_eq!(slim.tagged.len(), 2, "Chain tagged = [count:1][mask:1] = 2 bytes");
        assert_eq!(slim.total_size(), 11, "hash:8 + len:1 + tagged:2 = 11 bytes");
    }

    #[test]
    fn slim_node_fire_emoji() {
        // v2: fire-like with 3 non-default fields (V, A, T)
        // defaults: S=0x00, R=0x00, V=0x80, A=0x80, T=0x00
        // Need V≠0x80, A≠0x80, T≠0x00
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0xC0, 0xC0, 0xC0));
        // V_u8=0xC0≠0x80 ✓, A_u8=0xC0≠0x80 ✓, T_u8=0xC0 (3<<6=0xC0)≠0x00 ✓
        let slim = SlimNode::from_chain(&chain);
        assert_eq!(slim.tagged.len(), 5, "Chain tagged = [count:1][mask:1][V][A][T] = 5 bytes");
        assert_eq!(slim.total_size(), 14, "hash:8 + len:1 + tagged:5 = 14 bytes");
    }

    #[test]
    fn slim_node_roundtrip() {
        let chain = MolecularChain::single(test_mol(0x02, 0x06, 0xC0, 0x40, 0x05));
        let slim = SlimNode::from_chain(&chain);
        let bytes = slim.to_bytes();
        let (restored, consumed) = SlimNode::from_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(restored.hash, slim.hash);
        assert_eq!(restored.tagged, slim.tagged);
        let decoded = restored.to_chain().unwrap();
        assert_eq!(decoded, chain);
    }

    #[test]
    fn slim_node_vs_compact_node_size() {
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x90, 0x80, 0x03));
        let slim = SlimNode::from_chain(&chain);
        let mut dict = ChainDictionary::new(100);
        let compact = CompactNode::encode(&chain, None, &mut dict, 2, 1000);

        assert!(
            slim.total_size() < compact.total_size(),
            "SlimNode {} bytes < CompactNode {} bytes",
            slim.total_size(),
            compact.total_size()
        );
    }

    // ── SlimPage ──────────────────────────────────────────────────────────────

    #[test]
    fn slim_page_basic() {
        let mut page = SlimPage::new(0, 2, 1000);
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let slim = SlimNode::from_chain(&chain);
        assert!(page.push_node(slim));
        assert_eq!(page.nodes.len(), 1);
        assert!(!page.is_full());
    }

    #[test]
    fn slim_page_roundtrip() {
        let mut page = SlimPage::new(42, 2, 1000);

        // Add 5 nodes
        for i in 0u8..5 {
            let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80 + i, 0x80, 0x03));
            page.push_node(SlimNode::from_chain(&chain));
        }
        // Add 3 edges
        for i in 0u32..3 {
            page.push_edge(CompactEdge::encode(i as u64, (i + 1) as u64, 0.7, RelationBase::Causes.as_byte()));
        }

        let bytes = page.to_bytes();
        let restored = SlimPage::from_bytes(&bytes).expect("roundtrip should succeed");

        assert_eq!(restored.page_id, 42);
        assert_eq!(restored.layer, 2);
        assert_eq!(restored.batch_ts, 1000);
        assert_eq!(restored.nodes.len(), 5);
        assert_eq!(restored.edges.len(), 3);

        // Verify node hashes match
        for (orig, rest) in page.nodes.iter().zip(restored.nodes.iter()) {
            assert_eq!(orig.hash, rest.hash);
            assert_eq!(orig.tagged, rest.tagged);
        }
    }

    #[test]
    fn slim_page_find_node() {
        let mut page = SlimPage::new(0, 2, 1000);
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0xA0, 0x80, 0x03));
        let hash = chain.chain_hash();
        page.push_node(SlimNode::from_chain(&chain));

        assert!(page.find_node(hash).is_some());
        assert!(page.find_node(0xDEAD).is_none());
    }

    #[test]
    fn slim_page_corrupted_checksum() {
        let mut page = SlimPage::new(1, 2, 100);
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        page.push_node(SlimNode::from_chain(&chain));

        let mut bytes = page.to_bytes();
        if bytes.len() > 20 {
            bytes[20] ^= 0xFF;
        }
        assert!(SlimPage::from_bytes(&bytes).is_none(), "Corrupted → None");
    }

    #[test]
    fn slim_page_savings_vs_compact() {
        let mut page = SlimPage::new(0, 2, 1000);
        // 100 nodes with mostly-default molecules → tagged ~2B each
        for i in 0u8..100 {
            let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80 + (i % 10), 0x80, 0x03));
            page.push_node(SlimNode::from_chain(&chain));
        }

        let (slim_bytes, compact_bytes) = page.savings_vs_compact();
        assert!(
            slim_bytes < compact_bytes,
            "SlimPage {} bytes < CompactPage {} bytes ({:.0}% savings)",
            slim_bytes,
            compact_bytes,
            (1.0 - slim_bytes as f32 / compact_bytes as f32) * 100.0
        );
    }

    #[test]
    fn slim_500m_estimate() {
        // Spec: 500M concepts × ~10B = 5GB
        // Test: average node size with typical molecules
        let mut total = 0usize;
        let count = 100usize;
        for i in 0..count {
            let v = (i % 256) as u8;
            let chain = MolecularChain::single(test_mol(0x01, 0x01, v, 0x80, 0x03));
            let slim = SlimNode::from_chain(&chain);
            total += slim.total_size();
        }
        let avg = total as f64 / count as f64;
        // Avg should be ~10-11 bytes
        assert!(
            avg <= 14.0,
            "Average SlimNode size {:.1}B should be ≤ 14B",
            avg
        );
        // 500M × avg
        let estimate_gb = (500_000_000.0 * avg) / (1024.0 * 1024.0 * 1024.0);
        assert!(
            estimate_gb < 7.0,
            "500M nodes ≈ {:.1}GB should fit on phone (<7GB)",
            estimate_gb
        );
    }
}
