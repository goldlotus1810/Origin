//! # registry — Sổ cái
//!
//! Ghi lại tất cả mọi thứ được tạo ra trong HomeOS.
//! Append-only. Không xóa. Không sửa.
//! HomeOS đọc sổ cái → thấy mình → tạo ra cái mới.
//!
//! ## Cấu trúc:
//!   chain_index:  BTreeMap<u64, u64>         — hash → file offset
//!   name_index:   BTreeMap<&str, u64>        — alias → hash
//!   layer_rep:    [Option<u64>; 256]          — Lx → NodeLx hash
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
use alloc::vec::Vec;
use alloc::string::String;

use crate::molecular::MolecularChain;
use crate::lca::lca_weighted;

// ─────────────────────────────────────────────────────────────────────────────
// Entry — một record trong sổ cái
// ─────────────────────────────────────────────────────────────────────────────

/// Một entry trong sổ cái Registry.
#[derive(Debug, Clone)]
pub struct RegistryEntry {
    /// FNV-1a hash của MolecularChain — địa chỉ duy nhất
    pub chain_hash:  u64,
    /// Tầng (L0=0, L1=1, L2=2, ...)
    pub layer:       u8,
    /// Offset trong origin.olang file
    pub file_offset: u64,
    /// Timestamp khi tạo (nanoseconds)
    pub created_at:  i64,
    /// Trạng thái: false=ĐN (đang học), true=QR (đã chứng minh)
    pub is_qr:       bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────

/// Sổ cái của HomeOS.
///
/// In-memory index được rebuild từ origin.olang lúc startup.
/// Mọi thay đổi được ghi vào file TRƯỚC khi cập nhật RAM.
pub struct Registry {
    /// chain_hash → RegistryEntry (sorted by hash for binary search)
    entries: Vec<(u64, RegistryEntry)>,

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

    /// Cache: all chains for LCA computation
    /// (layer, chain_hash, chain) — dùng khi cập nhật layer_rep
    chain_cache: Vec<(u8, u64, MolecularChain)>,
}

impl Registry {
    /// Tạo Registry rỗng.
    pub fn new() -> Self {
        Self {
            entries:      Vec::new(),
            names:        Vec::new(),
            layer_rep:    [None; 16],
            branch_wm:    Vec::new(),
            qr_supersede: Vec::new(),
            chain_cache:  Vec::new(),
        }
    }

    // ── Insert ───────────────────────────────────────────────────────────────

    /// Đăng ký node mới vào sổ cái.
    ///
    /// Gọi SAU KHI đã ghi vào file (QT8).
    /// Tự động cập nhật layer_rep qua LCA.
    pub fn insert(
        &mut self,
        chain:       &MolecularChain,
        layer:       u8,
        file_offset: u64,
        created_at:  i64,
        is_qr:       bool,
    ) -> u64 {
        let hash = chain.chain_hash();

        // Kiểm tra đã có chưa (QT1: ○(x)==x)
        if self.lookup_hash(hash).is_some() {
            return hash; // Đã có — không insert lại
        }

        let entry = RegistryEntry { chain_hash: hash, layer, file_offset, created_at, is_qr };

        // Insert vào entries (sorted by hash)
        let pos = self.entries.partition_point(|&(h, _)| h < hash);
        self.entries.insert(pos, (hash, entry));

        // Cập nhật chain_cache cho LCA
        self.chain_cache.push((layer, hash, chain.clone()));

        // Cập nhật layer_rep qua LCA
        self.update_layer_rep(layer, chain);

        hash
    }

    /// Đăng ký alias (ngôn ngữ tự nhiên → node).
    ///
    /// "lửa" → hash(🔥)
    /// "fire" → hash(🔥)
    /// Không tạo node mới — chỉ thêm alias.
    pub fn register_alias(&mut self, name: &str, chain_hash: u64) {
        // Kiểm tra đã có chưa
        if self.lookup_name(name).is_some() { return; }
        let pos = self.names.partition_point(|(n, _)| n.as_str() < name);
        self.names.insert(pos, (String::from(name), chain_hash));
    }

    // ── Lookup ───────────────────────────────────────────────────────────────

    /// Lookup bằng chain_hash — O(log n).
    pub fn lookup_hash(&self, hash: u64) -> Option<&RegistryEntry> {
        self.entries
            .binary_search_by_key(&hash, |&(h, _)| h)
            .ok()
            .map(|i| &self.entries[i].1)
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

    /// Đại diện của tầng Lx (NodeLx).
    pub fn layer_rep(&self, layer: u8) -> Option<u64> {
        if layer < 16 { self.layer_rep[layer as usize] } else { None }
    }

    /// Leaf layer của một nhánh.
    pub fn branch_leaf_layer(&self, branch_hash: u64) -> Option<u8> {
        self.branch_wm.iter()
            .find(|&&(h, _)| h == branch_hash)
            .map(|&(_, l)| l)
    }

    /// Kiểm tra QR hash có bị supersede không.
    /// Trả về hash của QR mới hơn nếu bị supersede.
    pub fn superseded_by(&self, qr_hash: u64) -> Option<u64> {
        self.qr_supersede.iter()
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
        if let Some(entry) = self.branch_wm.iter_mut()
            .find(|(h, _)| *h == branch_hash)
        {
            entry.1 = leaf_layer;
        } else {
            self.branch_wm.push((branch_hash, leaf_layer));
        }
    }

    // ── Layer representative ─────────────────────────────────────────────────

    /// Cập nhật NodeLx = LCA của tất cả nodes trong tầng Lx.
    fn update_layer_rep(&mut self, layer: u8, new_chain: &MolecularChain) {
        if layer >= 16 { return; }

        // Collect tất cả chains trong cùng tầng
        let same_layer: Vec<(&MolecularChain, u32)> = self.chain_cache.iter()
            .filter(|(l, _, _)| *l == layer)
            .map(|(_, _, c)| (c, 1u32))
            .collect();

        if same_layer.is_empty() {
            self.layer_rep[layer as usize] = Some(new_chain.chain_hash());
            return;
        }

        // LCA của toàn bộ tầng
        let rep_chain = lca_weighted(&same_layer);
        self.layer_rep[layer as usize] = Some(rep_chain.chain_hash());
    }

    // ── Stats ────────────────────────────────────────────────────────────────

    /// Tổng số entries.
    pub fn len(&self) -> usize { self.entries.len() }

    /// Registry có rỗng không.
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    /// Số aliases đã đăng ký.
    pub fn alias_count(&self) -> usize { self.names.len() }

    /// Tất cả entries theo tầng.
    pub fn entries_in_layer(&self, layer: u8) -> Vec<&RegistryEntry> {
        self.entries.iter()
            .filter(|(_, e)| e.layer == layer)
            .map(|(_, e)| e)
            .collect()
    }

    /// Tất cả QR entries.
    pub fn qr_entries(&self) -> Vec<&RegistryEntry> {
        self.entries.iter()
            .filter(|(_, e)| e.is_qr)
            .map(|(_, e)| e)
            .collect()
    }

    /// Tất cả ĐN entries (chưa promote).
    pub fn dn_entries(&self) -> Vec<&RegistryEntry> {
        self.entries.iter()
            .filter(|(_, e)| !e.is_qr)
            .map(|(_, e)| e)
            .collect()
    }
}

impl Default for Registry {
    fn default() -> Self { Self::new() }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::encode_codepoint;

    fn skip_if_empty() -> bool { ucd::table_len() == 0 }

    #[test]
    fn registry_new_empty() {
        let r = Registry::new();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
        assert_eq!(r.alias_count(), 0);
    }

    #[test]
    fn insert_and_lookup() {
        if skip_if_empty() { return; }
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525); // 🔥
        let hash  = r.insert(&chain, 0, 0, 1000, false);

        let entry = r.lookup_hash(hash).expect("phải tìm được sau insert");
        assert_eq!(entry.layer, 0);
        assert_eq!(entry.is_qr, false);
        assert_eq!(entry.chain_hash, hash);
    }

    #[test]
    fn insert_idempotent() {
        if skip_if_empty() { return; }
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525);

        let h1 = r.insert(&chain, 0, 0,    1000, false);
        let h2 = r.insert(&chain, 0, 9999, 2000, false); // thử insert lại
        assert_eq!(h1, h2, "Insert lại cùng chain → cùng hash");
        assert_eq!(r.len(), 1, "Không duplicate");
    }

    #[test]
    fn lookup_chain() {
        if skip_if_empty() { return; }
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F4A7); // 💧
        r.insert(&chain, 2, 100, 1000, true);

        let entry = r.lookup_chain(&chain).expect("lookup_chain phải tìm được");
        assert_eq!(entry.layer, 2);
        assert!(entry.is_qr);
    }

    #[test]
    fn register_alias() {
        if skip_if_empty() { return; }
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525); // 🔥
        let hash  = r.insert(&chain, 0, 0, 1000, false);

        r.register_alias("lửa", hash);
        r.register_alias("fire", hash);
        r.register_alias("feu",  hash);

        assert_eq!(r.lookup_name("lửa"),  Some(hash));
        assert_eq!(r.lookup_name("fire"), Some(hash));
        assert_eq!(r.lookup_name("feu"),  Some(hash));
        assert_eq!(r.lookup_name("water"), None);
        assert_eq!(r.alias_count(), 3);
    }

    #[test]
    fn alias_idempotent() {
        if skip_if_empty() { return; }
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525);
        let hash = r.insert(&chain, 0, 0, 1000, false);

        r.register_alias("fire", hash);
        r.register_alias("fire", hash); // duplicate
        assert_eq!(r.alias_count(), 1, "Alias không duplicate");
    }

    #[test]
    fn layer_rep_single() {
        if skip_if_empty() { return; }
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525); // 🔥
        let hash = r.insert(&chain, 0, 0, 1000, false);

        // NodeL0 phải được set sau khi insert node L0 đầu tiên
        let rep = r.layer_rep(0);
        assert!(rep.is_some(), "layer_rep(0) phải có sau insert");
    }

    #[test]
    fn layer_rep_multiple() {
        if skip_if_empty() { return; }
        let mut r = Registry::new();

        // Insert 3 nodes vào L2
        r.insert(&encode_codepoint(0x1F525), 2, 0,   1000, false); // 🔥
        r.insert(&encode_codepoint(0x1F4A7), 2, 100, 1000, false); // 💧
        r.insert(&encode_codepoint(0x2744),  2, 200, 1000, false); // ❄

        // NodeL2 = LCA(🔥, 💧, ❄) — phải tồn tại
        let rep = r.layer_rep(2);
        assert!(rep.is_some(), "layer_rep(2) phải có sau 3 inserts");
    }

    #[test]
    fn qr_supersession() {
        if skip_if_empty() { return; }
        let mut r = Registry::new();

        let old_chain = encode_codepoint(0x25CB); // ○ (giả sử QR cũ sai)
        let new_chain = encode_codepoint(0x25CF); // ● (QR mới đúng hơn)

        let old_hash = r.insert(&old_chain, 2, 0,   1000, true);
        let new_hash = r.insert(&new_chain, 2, 100, 2000, true);

        r.supersede_qr(old_hash, new_hash);

        assert_eq!(r.superseded_by(old_hash), Some(new_hash),
            "old QR bị supersede bởi new QR");
        assert_eq!(r.superseded_by(new_hash), None,
            "new QR không bị supersede");

        // old QR vẫn tồn tại (QT8: không xóa)
        assert!(r.lookup_hash(old_hash).is_some(),
            "old QR vẫn tồn tại trong sổ cái (QT8)");
    }

    #[test]
    fn branch_watermark() {
        if skip_if_empty() { return; }
        let mut r = Registry::new();
        let branch_hash = 0xDEADBEEF_u64;

        r.update_branch_wm(branch_hash, 5);
        assert_eq!(r.branch_leaf_layer(branch_hash), Some(5));

        // Update
        r.update_branch_wm(branch_hash, 6);
        assert_eq!(r.branch_leaf_layer(branch_hash), Some(6),
            "leaf_layer tăng khi Dream promote");
    }

    #[test]
    fn entries_in_layer() {
        if skip_if_empty() { return; }
        let mut r = Registry::new();

        r.insert(&encode_codepoint(0x1F525), 0, 0,   1000, false);
        r.insert(&encode_codepoint(0x1F4A7), 0, 100, 1000, false);
        r.insert(&encode_codepoint(0x2744),  2, 200, 1000, false);

        assert_eq!(r.entries_in_layer(0).len(), 2, "2 nodes ở L0");
        assert_eq!(r.entries_in_layer(2).len(), 1, "1 node ở L2");
        assert_eq!(r.entries_in_layer(5).len(), 0, "0 nodes ở L5");
    }

    #[test]
    fn qr_and_dn_separation() {
        if skip_if_empty() { return; }
        let mut r = Registry::new();

        r.insert(&encode_codepoint(0x1F525), 0, 0,   1000, true);  // QR
        r.insert(&encode_codepoint(0x1F4A7), 0, 100, 1000, false); // ĐN
        r.insert(&encode_codepoint(0x2744),  0, 200, 1000, false); // ĐN

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
        if skip_if_empty() { return; }
        let mut r = Registry::new();
        let chain = encode_codepoint(0x1F525);
        let hash = r.insert(&chain, 0, 0, 1000, false);

        // Nhiều ngôn ngữ cùng trỏ về 1 node
        for alias in &["lửa", "fire", "feu", "feuer", "fuego", "огонь", "火"] {
            r.register_alias(alias, hash);
        }

        assert_eq!(r.alias_count(), 7);
        for alias in &["lửa", "fire", "feu", "feuer", "fuego", "огонь", "火"] {
            assert_eq!(r.lookup_name(alias), Some(hash),
                "Alias '{}' phải trỏ về cùng node", alias);
        }
    }
}
