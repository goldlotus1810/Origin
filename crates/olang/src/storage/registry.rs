//! # registry — Sổ cái
//!
//! Ghi lại tất cả mọi thứ được tạo ra trong HomeOS.
//! Append-only. Không xóa. Không sửa.
//! HomeOS đọc sổ cái → thấy mình → tạo ra cái mới.
//!
//! ## v2 — Codepoint-based index (T8)
//!
//! Registry index by codepoint (u32), NOT only by hash.
//! - `cp_index`: sorted Vec<(u32, usize)> — codepoint → entries index
//! - `lookup_codepoint(cp)` — O(log n) binary search
//! - L0 bootstrap: 9,584 nodes from UCD table (not 35 hardcoded)
//!
//! UDC = hệ tọa độ của chúng ta. UTF-32 codepoints = alias.
//! Registry stores UDC nodes, referenced by codepoint alias.
//!
//! ## Cấu trúc:
//!   entries:      Vec<(u64, RegistryEntry)>  — hash → entry (legacy, kept for compat)
//!   cp_index:     Vec<(u32, usize)>          — codepoint → entries index (v2 primary)
//!   name_index:   Vec<(String, u64)>         — alias → hash
//!   layer_rep:    [Option<u64>; 16]           — Lx → NodeLx hash
//!   branch_wm:    Vec<(u64, u8)>             — branch → leaf_layer
//!   qr_supersede: Vec<(u64, u64)>            — old → new QR hash
//!
//! ## Thứ tự bắt buộc (QT8):
//!   1. file.append(node)      ← TRƯỚC TIÊN
//!   2. registry.insert(hash)  ← sau khi file OK
//!   3. layer_rep.update(LCA)  ← cập nhật đại diện
//!   4. silk.connect(node)     ← nối Silk
//!   5. log.append(event)      ← CUỐI CÙNG

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::lca::{lca, lca_weighted};
use crate::molecular::MolecularChain;

// ─────────────────────────────────────────────────────────────────────────────
// NodeKind — phân loại node theo nhóm L1
// ─────────────────────────────────────────────────────────────────────────────

/// Nhóm node — mọi thứ tạo ra đều thuộc 1 nhóm.
///
/// L1 là nơi tập hợp toàn bộ "bản thiết kế" của HomeOS:
///   - Knowledge: kiến thức L0 device + kiến thức học được
///   - Memory: STM observations, trí nhớ ngắn hạn
///   - Agent: AAM, LeoAI, Chief, Worker
///   - Skill: 24 skills (7 instinct + 15 domain + worker skills)
///   - Program: VM ops, functions, compiler components
///   - Device: thiết bị đang kết nối
///   - Sensor: cảm biến của device
///   - Emotion: emotion nodes từ ConversationCurve
///
/// Đây chính là DNA — khi clone sang thiết bị mới, chỉ cần copy L1 nodes
/// → thiết bị tự biết mình có gì, biết làm gì, không cần train lại.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum NodeKind {
    /// L0 Unicode alphabet — innate, immutable (9,584 UDC nodes from udc.json)
    Alphabet = 0,
    /// Knowledge — kiến thức đã học, concepts, truths
    Knowledge = 1,
    /// Memory — STM observations, trí nhớ ngắn hạn
    Memory = 2,
    /// Agent — AAM, LeoAI, Chief, Worker definitions
    Agent = 3,
    /// Skill — stateless functions (7 instinct + 15 domain + 4 worker)
    Skill = 4,
    /// Program — VM ops, built-in functions, compiler components
    Program = 5,
    /// Device — thiết bị đang kết nối HomeOS
    Device = 6,
    /// Sensor — cảm biến của HomeOS/device
    Sensor = 7,
    /// Emotion — emotion states, conversation curve points
    Emotion = 8,
    /// System — internal housekeeping (layer reps, branch markers)
    System = 9,
}

impl NodeKind {
    /// Parse from byte.
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Alphabet),
            1 => Some(Self::Knowledge),
            2 => Some(Self::Memory),
            3 => Some(Self::Agent),
            4 => Some(Self::Skill),
            5 => Some(Self::Program),
            6 => Some(Self::Device),
            7 => Some(Self::Sensor),
            8 => Some(Self::Emotion),
            9 => Some(Self::System),
            _ => None,
        }
    }

    /// Display name for reporting.
    pub fn label(self) -> &'static str {
        match self {
            Self::Alphabet => "Alphabet",
            Self::Knowledge => "Knowledge",
            Self::Memory => "Memory",
            Self::Agent => "Agent",
            Self::Skill => "Skill",
            Self::Program => "Program",
            Self::Device => "Device",
            Self::Sensor => "Sensor",
            Self::Emotion => "Emotion",
            Self::System => "System",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry — một record trong sổ cái
// ─────────────────────────────────────────────────────────────────────────────

/// Một entry trong sổ cái Registry.
#[derive(Debug, Clone)]
pub struct RegistryEntry {
    /// FNV-1a hash của MolecularChain — địa chỉ duy nhất
    pub chain_hash: u64,
    /// Tầng (L0=0, L1=1, L2=2, ...)
    pub layer: u8,
    /// Offset trong origin.olang file
    pub file_offset: u64,
    /// Timestamp khi tạo (nanoseconds)
    pub created_at: i64,
    /// Trạng thái: false=ĐN (đang học), true=QR (đã chứng minh)
    pub is_qr: bool,
    /// Nhóm node — phân loại theo chức năng
    pub kind: NodeKind,
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

/// Sổ cái của HomeOS.
///
/// In-memory index được rebuild từ origin.olang lúc startup.
/// Mọi thay đổi được ghi vào file TRƯỚC khi cập nhật RAM.
#[allow(missing_docs)]
pub struct Registry {
    /// chain_hash → RegistryEntry (sorted by hash for binary search)
    entries: Vec<(u64, RegistryEntry)>,

    /// v2 codepoint index: codepoint → chain_hash.
    /// Sorted by codepoint for O(log n) binary search.
    /// Primary lookup path in v2 — codepoint IS the UDC address.
    /// Two-step lookup: cp → hash → entry (both O(log n)).
    cp_index: Vec<(u32, u64)>,

    /// alias (name) → chain_hash
    /// "lửa" → hash(🔥), "fire" → hash(🔥)
    names: Vec<(String, u64)>,

    /// Lx representative: layer → chain_hash của NodeLx
    /// NodeLx = LCA của toàn bộ tầng Lx
    layer_rep: [Option<u64>; 16],

    /// branch_watermark: branch_hash → leaf_layer
    /// Mỗi nhánh có leaf_layer riêng (không global Ln-1)
    branch_wm: Vec<(u64, u8)>,

    /// QR supersession: old_hash → new_hash
    /// B supersedes A: old=hash(A), new=hash(B)
    qr_supersede: Vec<(u64, u64)>,

    /// Reverse index: chain_hash → first alias name (sorted by hash for O(log n) lookup).
    /// Built during finalize_bulk(), maintained incrementally during normal inserts.
    hash_to_name: Vec<(u64, String)>,

    /// Current representative chain per layer — for incremental LCA.
    /// Only 16 slots (one per layer), replaced on each insert.
    /// Saves ~41 bytes/node vs storing all chains (at 1M nodes = 41 MB saved).
    layer_rep_chain: [Option<MolecularChain>; 16],

    /// Temporary chain cache used ONLY during bulk insert mode.
    /// Cleared after finalize_bulk() to free memory.
    bulk_chains: Vec<(u8, MolecularChain)>,

    /// Bulk mode: skip per-insert sorting + layer_rep update.
    /// Call `finalize_bulk()` when done.
    bulk_mode: bool,

    /// Number of sorted entries at start of bulk (for lookup split).
    bulk_sorted_prefix: usize,

    /// Layers that need layer_rep recalculation after bulk insert.
    dirty_layers: u16, // bitmask: bit i = layer i needs recalc
}

impl Registry {
    /// Tạo Registry rỗng.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            cp_index: Vec::new(),
            names: Vec::new(),
            hash_to_name: Vec::new(),
            layer_rep: [None; 16],
            branch_wm: Vec::new(),
            qr_supersede: Vec::new(),
            layer_rep_chain: [
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
            ],
            bulk_chains: Vec::new(),
            bulk_mode: false,
            bulk_sorted_prefix: 0,
            dirty_layers: 0,
        }
    }

    /// Enter bulk insert mode — skips O(n²) sorting + LCA per insert.
    /// MUST call `finalize_bulk()` after all inserts.
    pub fn begin_bulk(&mut self) {
        self.bulk_mode = true;
        self.bulk_sorted_prefix = self.entries.len();
    }

    /// Finalize bulk insert: sort entries + recalculate layer_rep.
    /// O(n log n) sort + O(k) LCA calls (k = dirty layers).
    pub fn finalize_bulk(&mut self) {
        self.bulk_mode = false;
        self.bulk_sorted_prefix = 0;

        // Sort entries by hash, then by layer DESC (so L1 comes before L0 for same hash)
        // This ensures dedup keeps L1 (with correct NodeKind) over L0 duplicate
        self.entries.sort_by(|a, b| {
            a.0.cmp(&b.0).then(b.1.layer.cmp(&a.1.layer))
        });

        // Deduplicate: keep first (= lowest layer = L1 over L0)
        self.entries.dedup_by_key(|&mut (h, _)| h);

        // Sort codepoint index (populated via insert_codepoint during bulk)
        self.sort_cp_index();

        // Sort names by name (binary search requirement)
        self.names.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
        self.names.dedup_by(|(a, _), (b, _)| a == b);

        // Build reverse index: hash → first alias name (sorted by hash)
        self.hash_to_name = self.names.iter()
            .map(|(name, hash)| (*hash, name.clone()))
            .collect();
        self.hash_to_name.sort_unstable_by_key(|(h, _)| *h);
        self.hash_to_name.dedup_by_key(|(h, _)| *h);

        // Recalculate layer_rep for dirty layers — ONE LCA per layer
        for layer in 0..16u8 {
            if self.dirty_layers & (1 << layer) != 0 {
                self.recalc_layer_rep(layer);
            }
        }
        self.dirty_layers = 0;

        // Free bulk_chains — no longer needed after layer_rep is computed.
        // Saves ~41 bytes per node (at 1M nodes = 41 MB freed).
        self.bulk_chains = Vec::new();
    }

    /// Recalculate layer representative from bulk_chains — O(n) for that layer.
    fn recalc_layer_rep(&mut self, layer: u8) {
        let same_layer: Vec<(&MolecularChain, u32)> = self
            .bulk_chains
            .iter()
            .filter(|(l, _)| *l == layer)
            .map(|(_, c)| (c, 1u32))
            .collect();

        if same_layer.is_empty() {
            self.layer_rep[layer as usize] = None;
            self.layer_rep_chain[layer as usize] = None;
        } else {
            let rep = lca_weighted(&same_layer);
            self.layer_rep[layer as usize] = Some(rep.chain_hash());
            self.layer_rep_chain[layer as usize] = Some(rep);
        }
    }

    // ── Insert ───────────────────────────────────────────────────────────────

    /// Đăng ký node mới vào sổ cái.
    ///
    /// Gọi SAU KHI đã ghi vào file (QT8).
    /// Tự động cập nhật layer_rep qua LCA.
    /// Đăng ký node mới vào sổ cái (default kind = Knowledge).
    ///
    /// Gọi SAU KHI đã ghi vào file (QT8).
    /// Tự động cập nhật layer_rep qua LCA.
    pub fn insert(
        &mut self,
        chain: &MolecularChain,
        layer: u8,
        file_offset: u64,
        created_at: i64,
        is_qr: bool,
    ) -> u64 {
        let kind = if layer == 0 { NodeKind::Alphabet } else { NodeKind::Knowledge };
        self.insert_with_kind(chain, layer, file_offset, created_at, is_qr, kind)
    }

    /// Đăng ký node mới với NodeKind cụ thể.
    pub fn insert_with_kind(
        &mut self,
        chain: &MolecularChain,
        layer: u8,
        file_offset: u64,
        created_at: i64,
        is_qr: bool,
        kind: NodeKind,
    ) -> u64 {
        let hash = chain.chain_hash();

        // Kiểm tra đã có chưa (QT1: ○(x)==x)
        if self.lookup_hash(hash).is_some() {
            return hash; // Đã có — không insert lại
        }

        let entry = RegistryEntry {
            chain_hash: hash,
            layer,
            file_offset,
            created_at,
            is_qr,
            kind,
        };

        if self.bulk_mode {
            // Bulk mode: push unsorted, defer layer_rep
            self.entries.push((hash, entry));
            self.bulk_chains.push((layer, chain.clone()));
            self.dirty_layers |= 1 << layer;
        } else {
            // Normal mode: insert sorted + incremental layer_rep
            let pos = self.entries.partition_point(|&(h, _)| h < hash);
            self.entries.insert(pos, (hash, entry));
            self.update_layer_rep_incremental(layer, chain);
        }

        hash
    }

    // ── v2 Codepoint-based insert/lookup ─────────────────────────────────────

    /// v2: Đăng ký node mới bằng codepoint (UDC address).
    ///
    /// Codepoint = UDC position. UTF-32 chỉ là alias.
    /// Tạo entry + cập nhật cả hash index và codepoint index.
    #[allow(clippy::too_many_arguments)]
    pub fn insert_codepoint(
        &mut self,
        codepoint: u32,
        chain: &MolecularChain,
        layer: u8,
        file_offset: u64,
        created_at: i64,
        is_qr: bool,
        kind: NodeKind,
    ) -> u64 {
        let hash = self.insert_with_kind(chain, layer, file_offset, created_at, is_qr, kind);

        // Add to codepoint index
        if self.bulk_mode {
            // Bulk mode: push unsorted, sort at finalize_bulk
            self.cp_index.push((codepoint, hash));
        } else {
            let pos = self.cp_index.partition_point(|&(cp, _)| cp < codepoint);
            if pos >= self.cp_index.len() || self.cp_index[pos].0 != codepoint {
                self.cp_index.insert(pos, (codepoint, hash));
            }
        }

        hash
    }

    /// v2: Lookup bằng codepoint — O(log n) → O(log n).
    ///
    /// Codepoint = UDC address. Đây là lookup path chính trong v2.
    /// Two-step: cp → hash (binary search cp_index) → entry (binary search entries).
    pub fn lookup_codepoint(&self, codepoint: u32) -> Option<&RegistryEntry> {
        self.cp_index
            .binary_search_by_key(&codepoint, |&(cp, _)| cp)
            .ok()
            .and_then(|i| {
                let hash = self.cp_index[i].1;
                self.lookup_hash(hash)
            })
    }

    /// Sort and deduplicate cp_index — called after finalize_bulk().
    fn sort_cp_index(&mut self) {
        self.cp_index.sort_unstable_by_key(|&(cp, _)| cp);
        self.cp_index.dedup_by_key(|&mut (cp, _)| cp);
    }

    /// Đăng ký alias (ngôn ngữ tự nhiên → node).
    ///
    /// "lửa" → hash(🔥)
    /// "fire" → hash(🔥)
    /// Không tạo node mới — chỉ thêm alias.
    pub fn register_alias(&mut self, name: &str, chain_hash: u64) {
        if self.bulk_mode {
            // Bulk mode: push unsorted, sort at finalize
            self.names.push((String::from(name), chain_hash));
        } else {
            // Normal mode: insert sorted
            if self.lookup_name(name).is_some() {
                return;
            }
            let pos = self.names.partition_point(|(n, _)| n.as_str() < name);
            self.names.insert(pos, (String::from(name), chain_hash));

            // Maintain reverse index — only add if hash not already present
            let rev_pos = self.hash_to_name.partition_point(|(h, _)| *h < chain_hash);
            if rev_pos >= self.hash_to_name.len() || self.hash_to_name[rev_pos].0 != chain_hash {
                self.hash_to_name.insert(rev_pos, (chain_hash, String::from(name)));
            }
        }
    }

    /// Cập nhật NodeKind cho một entry đã có trong sổ cái.
    ///
    /// Dùng khi load RT_NODE_KIND records từ origin.olang.
    /// Nếu hash không tồn tại → bỏ qua (node chưa được load).
    pub fn set_kind(&mut self, chain_hash: u64, kind: NodeKind) {
        if let Ok(idx) = self.entries.binary_search_by_key(&chain_hash, |&(h, _)| h) {
            self.entries[idx].1.kind = kind;
        }
    }

    // ── Lookup ───────────────────────────────────────────────────────────────

    /// Lookup bằng chain_hash — O(log n) normal, O(log p + k) bulk mode.
    ///
    /// In bulk mode: binary search sorted prefix (p pre-bulk entries),
    /// skip unsorted tail (k bulk entries are unique UCD — dedup at finalize).
    pub fn lookup_hash(&self, hash: u64) -> Option<&RegistryEntry> {
        if self.bulk_mode {
            // Binary search sorted prefix only (pre-bulk entries)
            let prefix = &self.entries[..self.bulk_sorted_prefix];
            prefix
                .binary_search_by_key(&hash, |&(h, _)| h)
                .ok()
                .map(|i| &prefix[i].1)
            // NOTE: does NOT search bulk tail — bulk entries come from UCD
            // which has unique codepoints. Duplicates resolved at finalize_bulk().
        } else {
            self.entries
                .binary_search_by_key(&hash, |&(h, _)| h)
                .ok()
                .map(|i| &self.entries[i].1)
        }
    }

    /// Lookup bằng chain — tính hash rồi lookup.
    pub fn lookup_chain(&self, chain: &MolecularChain) -> Option<&RegistryEntry> {
        self.lookup_hash(chain.chain_hash())
    }

    /// Lookup bằng alias (name).
    pub fn lookup_name(&self, name: &str) -> Option<u64> {
        self.names
            .binary_search_by(|(n, _)| n.as_str().cmp(name))
            .ok()
            .map(|i| self.names[i].1)
    }

    /// Reverse lookup: hash → first alias name (nếu có) — O(log n).
    pub fn lookup_name_by_hash(&self, chain_hash: u64) -> Option<String> {
        // Use reverse index if available
        if !self.hash_to_name.is_empty() {
            return self.hash_to_name
                .binary_search_by_key(&chain_hash, |(h, _)| *h)
                .ok()
                .map(|i| self.hash_to_name[i].1.clone());
        }
        self.names
            .iter()
            .find(|(_, h)| *h == chain_hash)
            .map(|(name, _)| name.clone())
    }

    /// Đại diện của tầng Lx (NodeLx).
    pub fn layer_rep(&self, layer: u8) -> Option<u64> {
        if layer < 16 {
            self.layer_rep[layer as usize]
        } else {
            None
        }
    }

    /// Leaf layer của một nhánh.
    pub fn branch_leaf_layer(&self, branch_hash: u64) -> Option<u8> {
        self.branch_wm
            .iter()
            .find(|&&(h, _)| h == branch_hash)
            .map(|&(_, l)| l)
    }

    /// Kiểm tra QR hash có bị supersede không.
    /// Trả về hash của QR mới hơn nếu bị supersede.
    pub fn superseded_by(&self, qr_hash: u64) -> Option<u64> {
        self.qr_supersede
            .iter()
            .find(|&&(old, _)| old == qr_hash)
            .map(|&(_, new)| new)
    }

    // ── QR Supersession ──────────────────────────────────────────────────────

    /// Đánh dấu QR_old bị supersede bởi QR_new.
    ///
    /// QR_old vẫn tồn tại trong sổ cái (QT8).
    /// Query QR_old → nhận QR_new + ghi chú "deprecated".
    pub fn supersede_qr(&mut self, old_hash: u64, new_hash: u64) {
        if !self.qr_supersede.iter().any(|&(o, _)| o == old_hash) {
            self.qr_supersede.push((old_hash, new_hash));
        }
    }

    // ── Branch watermark ─────────────────────────────────────────────────────

    /// Cập nhật leaf_layer của một nhánh.
    pub fn update_branch_wm(&mut self, branch_hash: u64, leaf_layer: u8) {
        if let Some(entry) = self.branch_wm.iter_mut().find(|(h, _)| *h == branch_hash) {
            entry.1 = leaf_layer;
        } else {
            self.branch_wm.push((branch_hash, leaf_layer));
        }
    }

    // ── Layer representative ─────────────────────────────────────────────────

    /// Incremental layer_rep update — O(1) per insert instead of O(n).
    /// Merges new_chain with current representative via LCA.
    fn update_layer_rep_incremental(&mut self, layer: u8, new_chain: &MolecularChain) {
        if layer >= 16 {
            return;
        }

        let idx = layer as usize;
        match self.layer_rep_chain[idx].take() {
            Some(current_rep) => {
                // Merge with existing representative
                let merged = lca(&current_rep, new_chain);
                self.layer_rep[idx] = Some(merged.chain_hash());
                self.layer_rep_chain[idx] = Some(merged);
            }
            None => {
                // First chain in this layer
                self.layer_rep[idx] = Some(new_chain.chain_hash());
                self.layer_rep_chain[idx] = Some(new_chain.clone());
            }
        }
    }

    // ── Stats ────────────────────────────────────────────────────────────────

    /// Tổng số entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Registry có rỗng không.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Số aliases đã đăng ký.
    pub fn alias_count(&self) -> usize {
        self.names.len()
    }

    /// v2: Số codepoints đã đăng ký trong cp_index.
    pub fn codepoint_count(&self) -> usize {
        self.cp_index.len()
    }

    /// Reverse lookup: tìm alias đầu tiên cho chain_hash — O(log n) via reverse index.
    pub fn alias_for_hash(&self, hash: u64) -> Option<&str> {
        // Use reverse index if available (post-finalize or normal mode)
        if !self.hash_to_name.is_empty() {
            return self.hash_to_name
                .binary_search_by_key(&hash, |(h, _)| *h)
                .ok()
                .map(|i| self.hash_to_name[i].1.as_str());
        }
        // Fallback: linear scan (during bulk mode or empty reverse index)
        self.names
            .iter()
            .find(|(_, h)| *h == hash)
            .map(|(name, _)| name.as_str())
    }

    /// Tất cả entries theo tầng.
    pub fn entries_in_layer(&self, layer: u8) -> Vec<&RegistryEntry> {
        self.entries
            .iter()
            .filter(|(_, e)| e.layer == layer)
            .map(|(_, e)| e)
            .collect()
    }

    /// Tất cả QR entries.
    pub fn qr_entries(&self) -> Vec<&RegistryEntry> {
        self.entries
            .iter()
            .filter(|(_, e)| e.is_qr)
            .map(|(_, e)| e)
            .collect()
    }

    /// Tất cả ĐN entries (chưa promote).
    pub fn dn_entries(&self) -> Vec<&RegistryEntry> {
        self.entries
            .iter()
            .filter(|(_, e)| !e.is_qr)
            .map(|(_, e)| e)
            .collect()
    }

    // ── NodeKind queries ──────────────────────────────────────────────────

    /// Tất cả entries theo NodeKind.
    pub fn entries_by_kind(&self, kind: NodeKind) -> Vec<&RegistryEntry> {
        self.entries
            .iter()
            .filter(|(_, e)| e.kind == kind)
            .map(|(_, e)| e)
            .collect()
    }

    /// Đếm entries theo NodeKind.
    pub fn count_by_kind(&self, kind: NodeKind) -> usize {
        self.entries.iter().filter(|(_, e)| e.kind == kind).count()
    }

    // ── Memory stats ──────────────────────────────────────────────────────

    /// Estimated RAM usage in bytes.
    ///
    /// Returns (entries_bytes, aliases_bytes, misc_bytes, total_bytes).
    pub fn memory_usage(&self) -> (usize, usize, usize, usize) {
        // entries: Vec<(u64, RegistryEntry)> — each ~35 bytes + Vec overhead
        let entry_size = core::mem::size_of::<(u64, RegistryEntry)>();
        let entries_bytes = self.entries.capacity() * entry_size;

        // names: Vec<(String, u64)> — String = 24B overhead + heap content + u64
        let name_overhead = core::mem::size_of::<(String, u64)>();
        let name_heap: usize = self.names.iter().map(|(s, _)| s.len()).sum();
        let aliases_bytes = self.names.capacity() * name_overhead + name_heap;

        // reverse index: hash_to_name
        let rev_overhead = core::mem::size_of::<(u64, String)>();
        let rev_heap: usize = self.hash_to_name.iter().map(|(_, s)| s.len()).sum();
        let rev_bytes = self.hash_to_name.capacity() * rev_overhead + rev_heap;

        // cp_index: Vec<(u32, u64)>
        let cp_index_bytes = self.cp_index.capacity() * core::mem::size_of::<(u32, u64)>();

        // misc: layer_rep_chain, branch_wm, qr_supersede, bulk_chains, cp_index
        let rep_chain_bytes: usize = self.layer_rep_chain.iter()
            .filter_map(|o| o.as_ref())
            .map(|c| c.0.len() * 5 + 24) // Vec overhead + molecules
            .sum();
        let branch_bytes = self.branch_wm.capacity() * core::mem::size_of::<(u64, u8)>();
        let qr_bytes = self.qr_supersede.capacity() * core::mem::size_of::<(u64, u64)>();
        let bulk_bytes = self.bulk_chains.capacity()
            * core::mem::size_of::<(u8, MolecularChain)>();
        let misc_bytes = rep_chain_bytes + branch_bytes + qr_bytes + bulk_bytes + rev_bytes + cp_index_bytes + 144;

        let total = entries_bytes + aliases_bytes + misc_bytes;
        (entries_bytes, aliases_bytes, misc_bytes, total)
    }

    // ── Tiered eviction ──────────────────────────────────────────────────

    /// Evict L2+ entries from RAM — returns evicted entries for TieredStore.
    ///
    /// Keeps L0 + L1 in RAM (always hot). Removes L2+ entries from
    /// `entries` Vec, freeing memory. Caller stores them in TieredStore.
    ///
    /// At 1B nodes: 139 GB → ~196 KB (L0+L1 only).
    pub fn evict_cold(&mut self, min_layer: u8) -> Vec<(u64, RegistryEntry)> {
        let mut evicted = Vec::new();
        let mut kept = Vec::new();

        for (hash, entry) in self.entries.drain(..) {
            if entry.layer >= min_layer {
                evicted.push((hash, entry));
            } else {
                kept.push((hash, entry));
            }
        }
        self.entries = kept;
        evicted
    }

    /// Count entries by layer.
    pub fn count_by_layer(&self, layer: u8) -> usize {
        self.entries.iter().filter(|(_, e)| e.layer == layer).count()
    }

    /// Summary: đếm từng nhóm NodeKind.
    pub fn kind_summary(&self) -> Vec<(NodeKind, usize)> {
        let kinds = [
            NodeKind::Alphabet, NodeKind::Knowledge, NodeKind::Memory,
            NodeKind::Agent, NodeKind::Skill, NodeKind::Program,
            NodeKind::Device, NodeKind::Sensor, NodeKind::Emotion,
            NodeKind::System,
        ];
        kinds.iter()
            .map(|&k| (k, self.count_by_kind(k)))
            .filter(|(_, c)| *c > 0)
            .collect()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::encode_codepoint;


    #[test]
    fn registry_new_empty() {
        let r = Registry::new();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
        assert_eq!(r.alias_count(), 0);
    }

    #[test]
    fn insert_and_lookup() {
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525); // 🔥
        let hash = r.insert(&chain, 0, 0, 1000, false);

        let entry = r.lookup_hash(hash).expect("phải tìm được sau insert");
        assert_eq!(entry.layer, 0);
        assert!(!entry.is_qr);
        assert_eq!(entry.chain_hash, hash);
    }

    #[test]
    fn insert_idempotent() {
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525);

        let h1 = r.insert(&chain, 0, 0, 1000, false);
        let h2 = r.insert(&chain, 0, 9999, 2000, false); // thử insert lại
        assert_eq!(h1, h2, "Insert lại cùng chain → cùng hash");
        assert_eq!(r.len(), 1, "Không duplicate");
    }

    #[test]
    fn lookup_chain() {
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F4A7); // 💧
        r.insert(&chain, 2, 100, 1000, true);

        let entry = r.lookup_chain(&chain).expect("lookup_chain phải tìm được");
        assert_eq!(entry.layer, 2);
        assert!(entry.is_qr);
    }

    #[test]
    fn register_alias() {
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525); // 🔥
        let hash = r.insert(&chain, 0, 0, 1000, false);

        r.register_alias("lửa", hash);
        r.register_alias("fire", hash);
        r.register_alias("feu", hash);

        assert_eq!(r.lookup_name("lửa"), Some(hash));
        assert_eq!(r.lookup_name("fire"), Some(hash));
        assert_eq!(r.lookup_name("feu"), Some(hash));
        assert_eq!(r.lookup_name("water"), None);
        assert_eq!(r.alias_count(), 3);
    }

    #[test]
    fn alias_idempotent() {
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525);
        let hash = r.insert(&chain, 0, 0, 1000, false);

        r.register_alias("fire", hash);
        r.register_alias("fire", hash); // duplicate
        assert_eq!(r.alias_count(), 1, "Alias không duplicate");
    }

    #[test]
    fn layer_rep_single() {
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525); // 🔥
        let _hash = r.insert(&chain, 0, 0, 1000, false);

        // NodeL0 phải được set sau khi insert node L0 đầu tiên
        let rep = r.layer_rep(0);
        assert!(rep.is_some(), "layer_rep(0) phải có sau insert");
    }

    #[test]
    fn layer_rep_multiple() {
        let mut r = Registry::new();

        // Insert 3 nodes vào L2
        r.insert(&encode_codepoint(0x1F525), 2, 0, 1000, false); // 🔥
        r.insert(&encode_codepoint(0x1F4A7), 2, 100, 1000, false); // 💧
        r.insert(&encode_codepoint(0x2744), 2, 200, 1000, false); // ❄

        // NodeL2 = LCA(🔥, 💧, ❄) — phải tồn tại
        let rep = r.layer_rep(2);
        assert!(rep.is_some(), "layer_rep(2) phải có sau 3 inserts");
    }

    #[test]
    fn qr_supersession() {
        let mut r = Registry::new();

        let old_chain = encode_codepoint(0x25CB); // ○ (giả sử QR cũ sai)
        let new_chain = encode_codepoint(0x25CF); // ● (QR mới đúng hơn)

        let old_hash = r.insert(&old_chain, 2, 0, 1000, true);
        let new_hash = r.insert(&new_chain, 2, 100, 2000, true);

        r.supersede_qr(old_hash, new_hash);

        assert_eq!(
            r.superseded_by(old_hash),
            Some(new_hash),
            "old QR bị supersede bởi new QR"
        );
        assert_eq!(r.superseded_by(new_hash), None, "new QR không bị supersede");

        // old QR vẫn tồn tại (QT8: không xóa)
        assert!(
            r.lookup_hash(old_hash).is_some(),
            "old QR vẫn tồn tại trong sổ cái (QT8)"
        );
    }

    #[test]
    fn branch_watermark() {
        let mut r = Registry::new();
        let branch_hash = 0xDEADBEEF_u64;

        r.update_branch_wm(branch_hash, 5);
        assert_eq!(r.branch_leaf_layer(branch_hash), Some(5));

        // Update
        r.update_branch_wm(branch_hash, 6);
        assert_eq!(
            r.branch_leaf_layer(branch_hash),
            Some(6),
            "leaf_layer tăng khi Dream promote"
        );
    }

    #[test]
    fn entries_in_layer() {
        let mut r = Registry::new();

        r.insert(&encode_codepoint(0x1F525), 0, 0, 1000, false);
        r.insert(&encode_codepoint(0x1F4A7), 0, 100, 1000, false);
        r.insert(&encode_codepoint(0x2744), 2, 200, 1000, false);

        assert_eq!(r.entries_in_layer(0).len(), 2, "2 nodes ở L0");
        assert_eq!(r.entries_in_layer(2).len(), 1, "1 node ở L2");
        assert_eq!(r.entries_in_layer(5).len(), 0, "0 nodes ở L5");
    }

    #[test]
    fn qr_and_dn_separation() {
        let mut r = Registry::new();

        r.insert(&encode_codepoint(0x1F525), 0, 0, 1000, true); // QR
        r.insert(&encode_codepoint(0x1F4A7), 0, 100, 1000, false); // ĐN
        r.insert(&encode_codepoint(0x2744), 0, 200, 1000, false); // ĐN

        assert_eq!(r.qr_entries().len(), 1, "1 QR entry");
        assert_eq!(r.dn_entries().len(), 2, "2 ĐN entries");
    }

    #[test]
    fn lookup_nonexistent() {
        let r = Registry::new();
        assert!(r.lookup_hash(0xDEADBEEF).is_none());
        assert!(r.lookup_name("nonexistent").is_none());
    }

    #[test]
    fn multiple_aliases_same_node() {
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525);
        let hash = r.insert(&chain, 0, 0, 1000, false);

        // Nhiều ngôn ngữ cùng trỏ về 1 node
        for alias in &["lửa", "fire", "feu", "feuer", "fuego", "огонь", "火"] {
            r.register_alias(alias, hash);
        }

        assert_eq!(r.alias_count(), 7);
        for alias in &["lửa", "fire", "feu", "feuer", "fuego", "огонь", "火"] {
            assert_eq!(
                r.lookup_name(alias),
                Some(hash),
                "Alias '{}' phải trỏ về cùng node",
                alias
            );
        }
    }

    // ── NodeKind tests ─────────────────────────────────────────────────────

    #[test]
    fn node_kind_roundtrip() {
        for b in 0u8..=9 {
            let kind = NodeKind::from_byte(b).unwrap();
            assert_eq!(kind as u8, b);
            assert!(!kind.label().is_empty());
        }
        assert!(NodeKind::from_byte(10).is_none());
    }

    #[test]
    fn insert_with_kind() {
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525); // 🔥
        let h = r.insert_with_kind(&chain, 1, 0, 0, true, NodeKind::Skill);
        let entry = r.lookup_hash(h).unwrap();
        assert_eq!(entry.kind, NodeKind::Skill);
        assert_eq!(entry.layer, 1);
    }

    #[test]
    fn insert_default_kind() {
        let mut r = Registry::new();
        // L0 insert → Alphabet
        let c0 = encode_codepoint(0x25CB);
        let h0 = r.insert(&c0, 0, 0, 0, true);
        assert_eq!(r.lookup_hash(h0).unwrap().kind, NodeKind::Alphabet);
        // L1 insert → Knowledge (default)
        let c1 = encode_codepoint(0x1F525);
        let h1 = r.insert(&c1, 1, 0, 0, false);
        assert_eq!(r.lookup_hash(h1).unwrap().kind, NodeKind::Knowledge);
    }

    #[test]
    fn entries_by_kind() {
        let mut r = Registry::new();
        // Use codepoints from UCD groups: Box Drawing (SDF) + Misc Symbols (EMOTICON)
        let c1 = encode_codepoint(0x2500); // ─ Box Drawing
        let c2 = encode_codepoint(0x2502); // │ Box Drawing
        let c3 = encode_codepoint(0x2654); // ♔ Chess King (Misc Symbols)
        r.insert_with_kind(&c1, 1, 0, 0, true, NodeKind::Skill);
        r.insert_with_kind(&c2, 1, 1, 0, true, NodeKind::Skill);
        r.insert_with_kind(&c3, 1, 2, 0, true, NodeKind::Agent);
        assert_eq!(r.entries_by_kind(NodeKind::Skill).len(), 2);
        assert_eq!(r.entries_by_kind(NodeKind::Agent).len(), 1);
        assert_eq!(r.entries_by_kind(NodeKind::Device).len(), 0);
    }

    #[test]
    fn kind_summary() {
        let mut r = Registry::new();
        let c1 = encode_codepoint(0x2500); // ─ Box Drawing
        let c2 = encode_codepoint(0x2654); // ♔ Chess King
        r.insert_with_kind(&c1, 1, 0, 0, true, NodeKind::Skill);
        r.insert_with_kind(&c2, 1, 1, 0, true, NodeKind::Agent);
        let summary = r.kind_summary();
        assert!(summary.iter().any(|(k, c)| *k == NodeKind::Skill && *c == 1));
        assert!(summary.iter().any(|(k, c)| *k == NodeKind::Agent && *c == 1));
    }

    #[test]
    fn evict_cold_removes_l2_plus() {
        let mut r = Registry::new();
        // Use codepoints from different groups to avoid hash collisions
        let c0 = encode_codepoint(0x1F525); // 🔥 L0 (EMOTICON)
        let c1 = encode_codepoint(0x25CF);  // ● L1 (SDF)
        let c2 = encode_codepoint(0x2208);  // ∈ L2 (MATH)
        let c3 = encode_codepoint(0x1D11E); // 𝄞 L3 (MUSICAL)

        r.insert(&c0, 0, 0, 0, false);
        r.insert(&c1, 1, 1, 0, false);
        let h2 = r.insert(&c2, 2, 2, 0, false);
        let h3 = r.insert(&c3, 3, 3, 0, false);

        assert_eq!(r.len(), 4);

        // Evict L2+
        let evicted = r.evict_cold(2);
        assert_eq!(evicted.len(), 2, "Should evict 2 L2+ entries");
        assert_eq!(r.len(), 2, "Should keep 2 L0+L1 entries");

        // L2+ hashes are gone from RAM
        assert!(r.lookup_hash(h2).is_none());
        assert!(r.lookup_hash(h3).is_none());

        // L0+L1 still in RAM
        assert!(r.lookup_hash(c0.chain_hash()).is_some());
        assert!(r.lookup_hash(c1.chain_hash()).is_some());
    }

    #[test]
    fn memory_usage_returns_nonzero() {
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525);
        r.insert(&chain, 0, 0, 0, false);
        r.register_alias("fire", chain.chain_hash());

        let (entries, aliases, _misc, total) = r.memory_usage();
        assert!(entries > 0, "Entries bytes > 0");
        assert!(aliases > 0, "Aliases bytes > 0");
        assert!(total > 0, "Total > 0");
        assert_eq!(total, entries + aliases + _misc);
    }

    #[test]
    fn reverse_index_after_bulk() {
        let mut r = Registry::new();
        r.begin_bulk();
        let chain = encode_codepoint(0x1F525);
        let hash = r.insert(&chain, 0, 0, 0, false);
        r.register_alias("fire", hash);
        r.finalize_bulk();

        // Reverse index should work after finalize
        let name = r.alias_for_hash(hash);
        assert_eq!(name, Some("fire"));
    }

    // ── v2 Codepoint-based tests ────────────────────────────────────────────

    #[test]
    fn insert_codepoint_and_lookup() {
        let mut r = Registry::new();
        let cp: u32 = 0x1F525; // 🔥
        let chain = encode_codepoint(cp);
        let hash = r.insert_codepoint(cp, &chain, 0, 0, 1000, true, NodeKind::Alphabet);

        // Lookup by codepoint
        let entry = r.lookup_codepoint(cp).expect("lookup_codepoint must find it");
        assert_eq!(entry.chain_hash, hash);
        assert_eq!(entry.layer, 0);
        assert_eq!(entry.kind, NodeKind::Alphabet);

        // Lookup by hash still works
        assert!(r.lookup_hash(hash).is_some());

        // Codepoint count
        assert_eq!(r.codepoint_count(), 1);
    }

    #[test]
    fn insert_codepoint_bulk_mode() {
        let mut r = Registry::new();
        r.begin_bulk();

        let cps = [0x25CF_u32, 0x25A0, 0x1F525, 0x2208]; // ●, ■, 🔥, ∈
        for &cp in &cps {
            let chain = encode_codepoint(cp);
            r.insert_codepoint(cp, &chain, 0, 0, 1000, true, NodeKind::Alphabet);
        }

        r.finalize_bulk();

        // All should be findable by codepoint
        for &cp in &cps {
            assert!(
                r.lookup_codepoint(cp).is_some(),
                "cp 0x{:04X} must be found after finalize_bulk",
                cp
            );
        }
        assert_eq!(r.codepoint_count(), 4);

        // Non-existent codepoint
        assert!(r.lookup_codepoint(0xFFFF).is_none());
    }

    #[test]
    fn lookup_codepoint_nonexistent() {
        let r = Registry::new();
        assert!(r.lookup_codepoint(0x1F525).is_none());
        assert_eq!(r.codepoint_count(), 0);
    }

    #[test]
    fn reverse_index_normal_mode() {
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525);
        let hash = r.insert(&chain, 0, 0, 0, false);
        r.register_alias("fire", hash);

        // Reverse index should work in normal mode
        assert_eq!(r.alias_for_hash(hash), Some("fire"));
        assert_eq!(r.lookup_name_by_hash(hash), Some(String::from("fire")));
    }
}

impl core::fmt::Debug for Registry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Registry")
            .field("len", &self.entries.len())
            .field("aliases", &self.names.len())
            .finish()
    }
}
