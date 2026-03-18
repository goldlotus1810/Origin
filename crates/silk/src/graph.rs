//! # graph — SilkGraph
//!
//! 3-layer Silk architecture:
//!
//! 1. **Implicit** (SilkIndex): 37 buckets × Ln-1 — 0 bytes edges.
//!    "Silk = hệ quả toán học của 5D, không phải dữ liệu."
//!
//! 2. **Learned** (HebbianLink): slim 19-byte connections.
//!    Hebbian = PHÁT HIỆN kết nối implicit, không TẠO mới.
//!    Emotion nằm trong V+A của node, không trên edge.
//!
//! 3. **Structural** (SilkEdge): backward compat, parent pointers.
//!
//! ## Unified query:
//!   unified_weight(A, B) = max(implicit_strength, hebbian_weight)
//!   unified_neighbors(A) = implicit ∪ hebbian (merged by hash)

extern crate alloc;
use alloc::vec::Vec;

use crate::edge::{EdgeKind, EmotionTag, HebbianLink, SilkEdge};
use crate::hebbian::{
    blend_emotion, fib, hebbian_decay, hebbian_strengthen, should_promote, PROMOTE_WEIGHT,
};
use crate::index::SilkIndex;

// ─────────────────────────────────────────────────────────────────────────────
// MolSummary — Lightweight 5D coordinates cho Silk
// ─────────────────────────────────────────────────────────────────────────────

/// Tóm tắt 5D của một Molecule — chỉ giữ bytes cần thiết cho similarity.
///
/// Silk crate là no_std, không import olang::molecular::Molecule.
/// Caller (learning.rs, runtime) truyền MolSummary khi co-activate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MolSummary {
    pub shape: u8,
    pub relation: u8,
    pub valence: u8,
    pub arousal: u8,
    pub time: u8,
}

impl MolSummary {
    /// So sánh 5D — trả về similarity ∈ [0.0, 1.0].
    ///
    /// Mỗi chiều chia sẻ cùng base → +0.2 (toàn phần).
    /// Mỗi chiều gần nhau (delta < 32) → +0.1 (bán phần).
    /// 5 chiều × 0.2 = 1.0 max (identical).
    pub fn similarity(&self, other: &Self) -> f32 {
        let mut score = 0.0f32;

        // Shape: cùng base (8 categories)
        let s_base_a = if self.shape == 0 { 1 } else { ((self.shape - 1) % 8) + 1 };
        let s_base_b = if other.shape == 0 { 1 } else { ((other.shape - 1) % 8) + 1 };
        if s_base_a == s_base_b {
            score += 0.20;
        }

        // Relation: cùng base (8 categories)
        let r_base_a = if self.relation == 0 { 1 } else { ((self.relation - 1) % 8) + 1 };
        let r_base_b = if other.relation == 0 { 1 } else { ((other.relation - 1) % 8) + 1 };
        if r_base_a == r_base_b {
            score += 0.20;
        }

        // Valence: gần nhau → similar (delta-based)
        let v_delta = (self.valence as i16 - other.valence as i16).unsigned_abs();
        if v_delta < 16 {
            score += 0.20;
        } else if v_delta < 48 {
            score += 0.10;
        }

        // Arousal: gần nhau → similar
        let a_delta = (self.arousal as i16 - other.arousal as i16).unsigned_abs();
        if a_delta < 16 {
            score += 0.20;
        } else if a_delta < 48 {
            score += 0.10;
        }

        // Time: cùng base (5 categories)
        let t_base_a = if self.time == 0 { 3 } else { ((self.time - 1) % 5) + 1 };
        let t_base_b = if other.time == 0 { 3 } else { ((other.time - 1) % 5) + 1 };
        if t_base_a == t_base_b {
            score += 0.20;
        }

        score
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SilkGraph
// ─────────────────────────────────────────────────────────────────────────────

/// Unified neighbor result: hash + combined weight.
#[derive(Debug, Clone, Copy)]
pub struct SilkNeighbor {
    /// Node hash
    pub hash: u64,
    /// Combined weight: max(implicit_strength, hebbian_weight)
    pub weight: f32,
    /// Implicit strength from 5D (0.0 if not in index)
    pub implicit: f32,
    /// Hebbian learned weight (0.0 if not learned)
    pub hebbian: f32,
    /// Số chiều implicit chia sẻ (0..5)
    pub shared_dims: u8,
}

/// In-memory Silk graph — 3-layer architecture.
///
/// Layer 1: SilkIndex (implicit 5D, 0 bytes edges)
/// Layer 2: HebbianLink (slim learned connections, 19 bytes each)
/// Layer 3: SilkEdge (structural, backward compat)
pub struct SilkGraph {
    /// Structural + legacy associative edges (backward compat)
    edges: Vec<SilkEdge>,
    /// Implicit 5D index — 37 buckets, 0-cost connections
    index: SilkIndex,
    /// Slim Hebbian links — learned co-activations (sorted by key)
    learned: Vec<HebbianLink>,
}

impl SilkGraph {
    /// Tạo graph rỗng.
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            index: SilkIndex::new(),
            learned: Vec::new(),
        }
    }

    // ── SilkIndex access ────────────────────────────────────────────────────

    /// Access implicit 5D index.
    pub fn index(&self) -> &SilkIndex {
        &self.index
    }

    /// Mutable access to index (for registering nodes).
    pub fn index_mut(&mut self) -> &mut SilkIndex {
        &mut self.index
    }

    /// Index a node into the implicit 5D buckets.
    pub fn index_node(&mut self, hash: u64, mol: &MolSummary) {
        self.index.index_node(hash, mol);
    }

    // ── HebbianLink access ──────────────────────────────────────────────────

    /// Number of learned HebbianLinks.
    pub fn learned_count(&self) -> usize {
        self.learned.len()
    }

    /// Find a HebbianLink by (from, to).
    pub fn find_learned(&self, from: u64, to: u64) -> Option<&HebbianLink> {
        let key = (from, to);
        self.learned
            .binary_search_by_key(&key, |l| l.key())
            .ok()
            .map(|i| &self.learned[i])
    }

    /// Learned weight between two nodes (0.0 if not learned).
    pub fn learned_weight(&self, from: u64, to: u64) -> f32 {
        self.find_learned(from, to)
            .map(|l| l.weight_f32())
            .unwrap_or(0.0)
    }

    /// Strengthen or create a HebbianLink (slim co-activation).
    ///
    /// This is the NEW preferred way to learn connections:
    /// - No EmotionTag on edge (emotion lives in node V+A)
    /// - 19 bytes instead of 46 bytes
    /// - Hebbian = discovery of implicit 5D connection strength
    pub fn learn(&mut self, from: u64, to: u64, reward: f32) {
        let key = (from, to);
        match self.learned.binary_search_by_key(&key, |l| l.key()) {
            Ok(idx) => {
                let link = &mut self.learned[idx];
                let w = hebbian_strengthen(link.weight_f32(), reward);
                link.set_weight(w);
                link.fire_count = link.fire_count.saturating_add(1);
            }
            Err(pos) => {
                self.learned.insert(pos, HebbianLink::new(from, to));
            }
        }
    }

    /// Strengthen with implicit 5D similarity boost.
    ///
    /// If molecules share ≥2 dims → reward boosted.
    pub fn learn_mol(
        &mut self,
        from: u64,
        to: u64,
        from_mol: Option<&MolSummary>,
        to_mol: Option<&MolSummary>,
        reward: f32,
    ) {
        let sim_bonus = match (from_mol, to_mol) {
            (Some(a), Some(b)) => SilkIndex::implicit_silk(a, b).strength,
            _ => 0.0,
        };

        let boosted = if sim_bonus >= 0.3 {
            reward * (1.0 + sim_bonus * 0.5)
        } else {
            reward
        };

        let key = (from, to);
        match self.learned.binary_search_by_key(&key, |l| l.key()) {
            Ok(idx) => {
                let link = &mut self.learned[idx];
                let w = hebbian_strengthen(link.weight_f32(), boosted);
                link.set_weight(w);
                link.fire_count = link.fire_count.saturating_add(1);
            }
            Err(pos) => {
                let mut link = HebbianLink::new(from, to);
                // Nodes sharing ≥3 dims start stronger
                if sim_bonus >= 0.6 {
                    link.set_weight((link.weight_f32() + sim_bonus * 0.3).min(0.8));
                }
                self.learned.insert(pos, link);
            }
        }
    }

    /// Decay all learned links by elapsed time.
    pub fn decay_learned(&mut self, elapsed_ns: i64) {
        for link in &mut self.learned {
            let w = hebbian_decay(link.weight_f32(), elapsed_ns);
            link.set_weight(w);
        }
        // Remove links that decayed below threshold
        self.learned.retain(|l| l.weight_f32() >= 0.01);
    }

    // ── Unified query (implicit + learned) ──────────────────────────────────

    /// Unified weight between 2 nodes: max(implicit, hebbian, legacy_assoc).
    ///
    /// This is the PREFERRED query method — combines all 3 layers.
    pub fn unified_weight(&self, from: u64, to: u64, from_mol: Option<&MolSummary>, to_mol: Option<&MolSummary>) -> f32 {
        // Layer 1: implicit from 5D
        let implicit = match (from_mol, to_mol) {
            (Some(a), Some(b)) => SilkIndex::implicit_silk(a, b).strength,
            _ => 0.0,
        };

        // Layer 2: learned (HebbianLink)
        let hebb = self.learned_weight(from, to)
            .max(self.learned_weight(to, from));

        // Layer 3: legacy (SilkEdge assoc)
        let legacy = self.assoc_weight(from, to)
            .max(self.assoc_weight(to, from));

        implicit.max(hebb).max(legacy)
    }

    /// Unified neighbors: implicit + learned + legacy merged.
    ///
    /// Returns sorted by weight desc.
    pub fn unified_neighbors(&self, hash: u64, mol: Option<&MolSummary>) -> Vec<SilkNeighbor> {
        use alloc::collections::BTreeMap;
        let mut map: BTreeMap<u64, SilkNeighbor> = BTreeMap::new();

        // Layer 1: implicit neighbors from index
        if let Some(m) = mol {
            for (h, shared) in self.index.implicit_neighbors(hash, m) {
                let implicit_str = shared as f32 * 0.20; // approximate
                map.insert(h, SilkNeighbor {
                    hash: h,
                    weight: implicit_str,
                    implicit: implicit_str,
                    hebbian: 0.0,
                    shared_dims: shared,
                });
            }
        }

        // Layer 2: learned links (from this node)
        for link in &self.learned {
            if link.from_hash == hash || link.to_hash == hash {
                let other = if link.from_hash == hash { link.to_hash } else { link.from_hash };
                let w = link.weight_f32();
                let entry = map.entry(other).or_insert(SilkNeighbor {
                    hash: other,
                    weight: 0.0,
                    implicit: 0.0,
                    hebbian: 0.0,
                    shared_dims: 0,
                });
                entry.hebbian = entry.hebbian.max(w);
                entry.weight = entry.weight.max(w);
            }
        }

        // Layer 3: legacy edges (backward compat)
        for e in &self.edges {
            if e.from_hash == hash || e.to_hash == hash {
                let other = if e.from_hash == hash { e.to_hash } else { e.from_hash };
                let entry = map.entry(other).or_insert(SilkNeighbor {
                    hash: other,
                    weight: 0.0,
                    implicit: 0.0,
                    hebbian: 0.0,
                    shared_dims: 0,
                });
                entry.weight = entry.weight.max(e.weight);
            }
        }

        let mut result: Vec<SilkNeighbor> = map.into_values().collect();
        result.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(core::cmp::Ordering::Equal));
        result
    }

    /// Promote candidates from learned links (Dream input).
    pub fn learned_promote_candidates(&self, depth: usize) -> Vec<(u64, u64, f32)> {
        let max_fire = self.learned.iter().map(|l| l.fire_count as u32).max().unwrap_or(1);

        let mut candidates: Vec<(u64, u64, f32)> = self.learned
            .iter()
            .filter(|l| should_promote(l.weight_f32(), l.fire_count as u32, depth))
            .map(|l| {
                let fire_ratio = l.fire_count as f32 / max_fire as f32;
                let score = 0.4 * l.weight_f32() + 0.3 * fire_ratio;
                (l.from_hash, l.to_hash, score)
            })
            .collect();

        candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(core::cmp::Ordering::Equal));
        candidates
    }

    // ── Insert / Connect ─────────────────────────────────────────────────────

    /// Thêm structural edge (weight=1.0, bất biến).
    pub fn connect_structural(&mut self, from: u64, to: u64, kind: EdgeKind, ts: i64) {
        if self.find_edge(from, to, kind).is_some() {
            return;
        }
        let edge = SilkEdge::structural(from, to, kind, ts);
        self.insert_sorted(edge);
    }

    /// Thêm hoặc cập nhật associative edge.
    ///
    /// Nếu edge đã có → Hebbian strengthen.
    /// Nếu chưa có → tạo mới với weight thấp.
    ///
    /// **QT11**: Caller PHẢI đảm bảo from và to cùng tầng (Ln-1).
    /// Dùng `co_activate_same_layer()` để có kiểm tra tầng tự động,
    /// hoặc `co_activate_cross_layer()` cho kết nối khác tầng.
    pub fn co_activate(&mut self, from: u64, to: u64, emotion: EmotionTag, reward: f32, ts: i64) {
        let key = (from, to, EdgeKind::Assoc.as_byte());

        if let Some(idx) = self.find_edge_idx(from, to, EdgeKind::Assoc) {
            // Edge đã có → strengthen
            let e = &mut self.edges[idx];
            e.weight = hebbian_strengthen(e.weight, reward);
            e.emotion = blend_emotion(e.emotion, emotion, emotion.intensity);
            e.fire_count = e.fire_count.saturating_add(1);
            e.updated_at = ts;
        } else {
            // Edge chưa có → tạo mới
            let _ = key;
            let edge = SilkEdge::associative(from, to, emotion, ts);
            self.insert_sorted(edge);
        }
    }

    /// Co-activate dùng 5D molecular similarity — **đây là cách đúng**.
    ///
    /// Similarity giữa 2 molecules boost cả reward lẫn initial weight:
    ///   - similarity >= 0.4 → reward *= (1 + similarity * 0.5)
    ///   - Nodes chia sẻ nhiều chiều → kết nối mạnh hơn tự nhiên
    ///
    /// Đây là "Silk = công thức quan hệ" — kết nối xuất phát từ 5D,
    /// không chỉ từ temporal proximity.
    #[allow(clippy::too_many_arguments)]
    pub fn co_activate_mol(
        &mut self,
        from: u64,
        to: u64,
        from_mol: Option<MolSummary>,
        to_mol: Option<MolSummary>,
        emotion: EmotionTag,
        reward: f32,
        ts: i64,
    ) {
        // Tính 5D similarity bonus
        let sim_bonus = match (from_mol, to_mol) {
            (Some(a), Some(b)) => a.similarity(&b),
            _ => 0.0,
        };

        // Boost reward dựa trên similarity: nodes cùng chiều → kết nối mạnh hơn
        let boosted_reward = if sim_bonus >= 0.4 {
            reward * (1.0 + sim_bonus * 0.5)
        } else {
            reward
        };

        let key = (from, to, EdgeKind::Assoc.as_byte());

        if let Some(idx) = self.find_edge_idx(from, to, EdgeKind::Assoc) {
            let e = &mut self.edges[idx];
            e.weight = hebbian_strengthen(e.weight, boosted_reward);
            e.emotion = blend_emotion(e.emotion, emotion, emotion.intensity);
            e.fire_count = e.fire_count.saturating_add(1);
            e.updated_at = ts;
        } else {
            let _ = key;
            let mut edge = SilkEdge::associative(from, to, emotion, ts);
            // Nodes chia sẻ >=3 chiều → start stronger (implicit Silk)
            if sim_bonus >= 0.6 {
                edge.weight = (edge.weight + sim_bonus * 0.3).min(0.8);
            }
            self.insert_sorted(edge);
        }
    }

    /// Co-activate với kiểm tra tầng (QT11 enforcement).
    ///
    /// Chỉ cho phép kết nối cùng tầng. Nếu khác tầng → trả về false
    /// và KHÔNG tạo/cập nhật edge. Dùng `co_activate_cross_layer()`
    /// cho kết nối khác tầng có kiểm soát.
    #[allow(clippy::too_many_arguments)]
    pub fn co_activate_same_layer(
        &mut self,
        from: u64,
        to: u64,
        from_layer: u8,
        to_layer: u8,
        emotion: EmotionTag,
        reward: f32,
        ts: i64,
    ) -> bool {
        if from_layer != to_layer {
            return false; // QT11: Silk chỉ ở Ln-1 — từ chối cross-layer
        }
        self.co_activate(from, to, emotion, reward, ts);
        true
    }

    /// Cross-layer co-activation — kết nối giữa 2 tầng khác nhau.
    ///
    /// QT⑫ mở rộng: cho phép cross-layer Silk khi:
    ///   1. Weight đạt Fib[n+2] threshold (n = abs(layer_a - layer_b))
    ///   2. fire_count >= Fib[n+2] (nghiêm hơn same-layer Fib[n])
    ///
    /// `layers` = (from_layer, to_layer).
    /// Caller (LeoAI/Chief) chịu trách nhiệm kiểm AAM approve.
    /// Returns true nếu edge đủ điều kiện cross-layer.
    #[allow(clippy::too_many_arguments)]
    pub fn co_activate_cross_layer(
        &mut self,
        from: u64,
        to: u64,
        layers: (u8, u8),
        emotion: EmotionTag,
        reward: f32,
        ts: i64,
    ) -> bool {
        let layer_diff = (layers.0 as i16 - layers.1 as i16).unsigned_abs() as usize;

        // Same-layer → delegate to normal co_activate
        if layer_diff == 0 {
            self.co_activate(from, to, emotion, reward, ts);
            return true;
        }

        // Cross-layer: Fib[n+2] threshold — stricter
        let threshold = fib(layer_diff + 2);

        // Check existing edge
        if let Some(idx) = self.find_edge_idx(from, to, EdgeKind::Assoc) {
            let e = &mut self.edges[idx];
            e.weight = hebbian_strengthen(e.weight, reward);
            e.emotion = blend_emotion(e.emotion, emotion, emotion.intensity);
            e.fire_count = e.fire_count.saturating_add(1);
            e.updated_at = ts;

            // Cross-layer edge chỉ "hoạt động" khi đủ threshold
            e.fire_count >= threshold && e.weight >= PROMOTE_WEIGHT
        } else {
            // Tạo mới — bắt đầu yếu, cần nhiều co-activation
            let edge = SilkEdge::associative(from, to, emotion, ts);
            self.insert_sorted(edge);
            false // chưa đủ threshold
        }
    }

    // ── Restore từ file ────────────────────────────────────────────────────

    /// Restore edge từ file — dùng khi boot từ origin.olang.
    ///
    /// Không dùng Hebbian strengthen — giữ nguyên weight ban đầu.
    /// Edge đã có thì skip (idempotent).
    pub fn restore_edge(&mut self, from: u64, to: u64, edge_type: u8, ts: i64) {
        use crate::edge::ModalitySource;
        let kind = EdgeKind::from_byte(edge_type).unwrap_or(EdgeKind::Assoc);
        if self.find_edge(from, to, kind).is_some() {
            return; // đã có — skip
        }
        let edge = SilkEdge {
            from_hash: from,
            to_hash: to,
            kind,
            weight: 0.30, // minimum saveable weight
            emotion: EmotionTag::NEUTRAL,
            fire_count: 1,
            created_at: ts,
            updated_at: ts,
            source: ModalitySource::Text,
            confidence: 0.5,
        };
        self.insert_sorted(edge);
    }

    // ── Decay ────────────────────────────────────────────────────────────────

    /// Decay tất cả associative edges theo thời gian đã trôi qua.
    ///
    /// Gọi mỗi khi HomeOS wake up hoặc sau khoảng thời gian dài.
    pub fn decay_all(&mut self, elapsed_ns: i64) {
        for e in &mut self.edges {
            if e.kind.is_associative() {
                e.weight = hebbian_decay(e.weight, elapsed_ns);
            }
        }
        // Xóa edges đã quá yếu (weight < 0.01)
        self.edges
            .retain(|e| !e.kind.is_associative() || e.weight >= 0.01);
    }

    // ── Maintain — chăm sóc Ln-1 ──────────────────────────────────────────

    /// Chăm sóc vườn Silk: decay + cắt tỉa overflow.
    ///
    /// Gọi định kỳ (mỗi N turns hoặc trước Dream).
    /// 1. Decay tất cả associative edges theo thời gian.
    /// 2. Nếu vượt max_edges → cắt tỉa edges yếu nhất (giữ structural).
    ///
    /// Trả về số edges đã bị cắt tỉa.
    pub fn maintain(&mut self, elapsed_ns: i64, max_edges: usize) -> usize {
        let before = self.edges.len();

        // 1. Decay theo thời gian — φ⁻¹ mỗi 24h
        self.decay_all(elapsed_ns);

        // 2. Overflow pruning — giữ max_edges, cắt associative yếu nhất
        let assoc_count = self.assoc_count();
        let structural_count = self.structural_count();

        if max_edges > 0 && self.edges.len() > max_edges {
            // Capacity cho associative = max_edges - structural
            let assoc_cap = max_edges.saturating_sub(structural_count);

            if assoc_count > assoc_cap {
                // Thu thập weights → tìm threshold cắt
                let mut assoc_weights: Vec<f32> = self
                    .edges
                    .iter()
                    .filter(|e| e.kind.is_associative())
                    .map(|e| e.weight)
                    .collect();
                assoc_weights.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));

                // Cắt (assoc_count - assoc_cap) edges yếu nhất
                let cut = assoc_count - assoc_cap;
                if cut < assoc_weights.len() {
                    let threshold = assoc_weights[cut];
                    let mut removed = 0usize;
                    self.edges.retain(|e| {
                        if !e.kind.is_associative() {
                            return true; // giữ structural
                        }
                        if removed < cut && e.weight < threshold {
                            removed += 1;
                            false // cắt
                        } else {
                            true // giữ
                        }
                    });
                }
            }
        }

        before - self.edges.len()
    }

    // ── Lookup ───────────────────────────────────────────────────────────────

    /// Tìm edge bằng (from, to, kind).
    pub fn find_edge(&self, from: u64, to: u64, kind: EdgeKind) -> Option<&SilkEdge> {
        self.find_edge_idx(from, to, kind).map(|i| &self.edges[i])
    }

    /// Tất cả edges từ một node.
    pub fn edges_from(&self, from: u64) -> Vec<&SilkEdge> {
        self.edges.iter().filter(|e| e.from_hash == from).collect()
    }

    /// Tất cả edges đến một node.
    pub fn edges_to(&self, to: u64) -> Vec<&SilkEdge> {
        self.edges.iter().filter(|e| e.to_hash == to).collect()
    }

    /// Neighbors của một node (cả 2 chiều).
    pub fn neighbors(&self, hash: u64) -> Vec<u64> {
        let mut ns: Vec<u64> = self
            .edges
            .iter()
            .filter_map(|e| {
                if e.from_hash == hash {
                    Some(e.to_hash)
                } else if e.to_hash == hash {
                    Some(e.from_hash)
                } else {
                    None
                }
            })
            .collect();
        ns.sort_unstable();
        ns.dedup();
        ns
    }

    /// Weight của associative edge (from, to).
    pub fn assoc_weight(&self, from: u64, to: u64) -> f32 {
        self.find_edge(from, to, EdgeKind::Assoc)
            .map(|e| e.weight)
            .unwrap_or(0.0)
    }

    // ── Dream cluster detection ──────────────────────────────────────────────

    /// Tính cluster_score(A, B) cho Dream — đầy đủ 3 thành phần.
    ///
    /// score = 0.3 × chain_similarity + 0.4 × hebbian_weight + 0.3 × fire_ratio
    ///
    /// `mol_a`, `mol_b`: 5D coordinates của 2 nodes.
    /// Nếu không có mol → chain_similarity = 0 (fallback cũ).
    pub fn cluster_score(
        &self,
        hash_a: u64,
        hash_b: u64,
        mol_a: Option<MolSummary>,
        mol_b: Option<MolSummary>,
        max_fire_count: u32,
    ) -> f32 {
        let weight = self
            .assoc_weight(hash_a, hash_b)
            .max(self.assoc_weight(hash_b, hash_a));

        let fire = self
            .find_edge(hash_a, hash_b, EdgeKind::Assoc)
            .or_else(|| self.find_edge(hash_b, hash_a, EdgeKind::Assoc))
            .map(|e| e.fire_count)
            .unwrap_or(0);

        let fire_ratio = if max_fire_count > 0 {
            fire as f32 / max_fire_count as f32
        } else {
            0.0
        };

        let chain_sim = match (mol_a, mol_b) {
            (Some(a), Some(b)) => a.similarity(&b),
            _ => 0.0,
        };

        0.3 * chain_sim + 0.4 * weight + 0.3 * fire_ratio
    }

    /// Tính cluster_score(A, B) cho Dream — partial (không có 5D data).
    ///
    /// Backwards-compatible: score = 0.4 × hebbian_weight + 0.3 × fire_ratio
    pub fn cluster_score_partial(&self, hash_a: u64, hash_b: u64, max_fire_count: u32) -> f32 {
        self.cluster_score(hash_a, hash_b, None, None, max_fire_count)
    }

    /// Tìm candidates cần promote tại một tầng (Dream input).
    ///
    /// Trả về Vec<(hash_a, hash_b, score)> sorted by score desc.
    pub fn promote_candidates(&self, depth: usize) -> Vec<(u64, u64, f32)> {
        let max_fire = self
            .edges
            .iter()
            .filter(|e| e.kind.is_associative())
            .map(|e| e.fire_count)
            .max()
            .unwrap_or(1);

        let mut candidates: Vec<(u64, u64, f32)> = self
            .edges
            .iter()
            .filter(|e| e.kind.is_associative() && should_promote(e.weight, e.fire_count, depth))
            .map(|e| {
                let score = self.cluster_score_partial(e.from_hash, e.to_hash, max_fire);
                (e.from_hash, e.to_hash, score)
            })
            .collect();

        candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(core::cmp::Ordering::Equal));
        candidates
    }

    // ── Stats ────────────────────────────────────────────────────────────────

    /// Tổng số edges.
    /// Iterator qua tất cả edges.
    pub fn all_edges(&self) -> impl Iterator<Item = &SilkEdge> {
        self.edges.iter()
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Graph có rỗng không.
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Số associative edges.
    pub fn assoc_count(&self) -> usize {
        self.edges
            .iter()
            .filter(|e| e.kind.is_associative())
            .count()
    }

    /// Số structural edges.
    pub fn structural_count(&self) -> usize {
        self.edges.iter().filter(|e| e.kind.is_structural()).count()
    }

    /// Số unique nodes (distinct from/to hashes).
    pub fn node_count(&self) -> usize {
        let mut nodes = alloc::collections::BTreeSet::new();
        for e in &self.edges {
            nodes.insert(e.from_hash);
            nodes.insert(e.to_hash);
        }
        nodes.len()
    }

    // ── Memory stats ──────────────────────────────────────────────────────

    /// Estimated RAM usage in bytes.
    pub fn memory_usage(&self) -> usize {
        let edge_size = core::mem::size_of::<SilkEdge>();
        self.edges.capacity() * edge_size
    }

    // ── Internal ─────────────────────────────────────────────────────────────

    fn find_edge_idx(&self, from: u64, to: u64, kind: EdgeKind) -> Option<usize> {
        let key = (from, to, kind.as_byte());
        self.edges.iter().position(|e| e.key() == key)
    }

    fn insert_sorted(&mut self, edge: SilkEdge) {
        let key = edge.key();
        let pos = self.edges.partition_point(|e| e.key() < key);
        self.edges.insert(pos, edge);
    }
}

impl Default for SilkGraph {
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

    fn emo(v: f32, a: f32) -> EmotionTag {
        EmotionTag::new(v, a, 0.5, 0.5)
    }

    #[test]
    fn graph_empty() {
        let g = SilkGraph::new();
        assert!(g.is_empty());
        assert_eq!(g.len(), 0);
    }

    #[test]
    fn connect_structural() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xA, 0xB, EdgeKind::Member, 1000);
        assert_eq!(g.len(), 1);

        let e = g.find_edge(0xA, 0xB, EdgeKind::Member).unwrap();
        assert_eq!(e.weight, 1.0, "Structural = weight 1.0");
        assert_eq!(e.kind, EdgeKind::Member);
    }

    #[test]
    fn structural_idempotent() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xA, 0xB, EdgeKind::Member, 1000);
        g.connect_structural(0xA, 0xB, EdgeKind::Member, 2000); // duplicate
        assert_eq!(g.len(), 1, "Không duplicate structural edge");
    }

    #[test]
    fn co_activate_creates_assoc() {
        let mut g = SilkGraph::new();
        g.co_activate(0xA, 0xB, emo(-0.6, 0.8), 0.8, 1000);
        assert_eq!(g.assoc_count(), 1);
        let w = g.assoc_weight(0xA, 0xB);
        assert!(w > 0.0 && w < 0.5, "Assoc starts weak: w={}", w);
    }

    #[test]
    fn co_activate_strengthens() {
        let mut g = SilkGraph::new();
        let emotion = emo(-0.5, 0.7);

        // Co-activate nhiều lần
        for _ in 0..20 {
            g.co_activate(0xA, 0xB, emotion, 0.8, 1000);
        }

        let w = g.assoc_weight(0xA, 0xB);
        assert!(w > 0.5, "Nhiều co-activation → weight tăng: w={}", w);
    }

    #[test]
    fn co_activate_emotion_carries() {
        let mut g = SilkGraph::new();
        // Lửa và nguy hiểm co-activate lúc arousal cao
        let high_emotion = emo(-0.8, 0.95);
        g.co_activate(0xF1BE_u64, 0xDA4E_u64, high_emotion, 1.0, 1000);

        let e = g
            .find_edge(0xF1BE_u64, 0xDA4E_u64, EdgeKind::Assoc)
            .unwrap();
        // Edge phải mang màu cảm xúc của khoảnh khắc đó
        assert!(
            e.emotion.arousal > 0.5,
            "Edge mang arousal cao của khoảnh khắc: a={}",
            e.emotion.arousal
        );
    }

    #[test]
    fn decay_reduces_weight() {
        let mut g = SilkGraph::new();
        g.co_activate(0xA, 0xB, emo(0.0, 0.5), 1.0, 0);

        // Force weight cao
        for _ in 0..50 {
            g.co_activate(0xA, 0xB, emo(0.0, 0.5), 1.0, 0);
        }

        let w_before = g.assoc_weight(0xA, 0xB);
        g.decay_all(86_400_000_000_000); // 1 ngày
        let w_after = g.assoc_weight(0xA, 0xB);

        assert!(
            w_after < w_before,
            "Decay phải giảm weight: {} → {}",
            w_before,
            w_after
        );
    }

    #[test]
    fn structural_not_decayed() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xA, 0xB, EdgeKind::Member, 0);

        g.decay_all(86_400_000_000_000 * 365); // 1 năm
        let e = g.find_edge(0xA, 0xB, EdgeKind::Member).unwrap();
        assert_eq!(e.weight, 1.0, "Structural edge không bị decay");
    }

    #[test]
    fn neighbors() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xA, 0xB, EdgeKind::Member, 0);
        g.connect_structural(0xA, 0xC, EdgeKind::Causes, 0);
        g.connect_structural(0xD, 0xA, EdgeKind::DerivedFrom, 0);

        let ns = g.neighbors(0xA);
        assert!(ns.contains(&0xB));
        assert!(ns.contains(&0xC));
        assert!(ns.contains(&0xD));
        assert_eq!(ns.len(), 3);
    }

    #[test]
    fn promote_candidates_threshold() {
        let mut g = SilkGraph::new();

        // Co-activate nhiều lần để đạt threshold
        for _ in 0..100 {
            g.co_activate(0xA, 0xB, emo(-0.5, 0.8), 1.0, 0);
        }

        // fib(1) = 1 → depth=1 dễ promote nhất
        let candidates = g.promote_candidates(1);
        assert!(
            !candidates.is_empty(),
            "Sau 100 co-activations phải có candidates"
        );
    }

    #[test]
    fn cluster_score_increases_with_weight() {
        let mut g = SilkGraph::new();
        g.co_activate(0xA, 0xB, emo(0.0, 0.5), 0.5, 0);
        let score1 = g.cluster_score_partial(0xA, 0xB, 10);

        for _ in 0..20 {
            g.co_activate(0xA, 0xB, emo(0.0, 0.5), 0.5, 0);
        }
        let score2 = g.cluster_score_partial(0xA, 0xB, 10);

        assert!(
            score2 > score1,
            "Thêm co-activation → score tăng: {} → {}",
            score1,
            score2
        );
    }

    #[test]
    fn edges_from_correct() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xA, 0xB, EdgeKind::Member, 0);
        g.connect_structural(0xA, 0xC, EdgeKind::Causes, 0);
        g.connect_structural(0xB, 0xA, EdgeKind::Similar, 0);

        let from_a = g.edges_from(0xA);
        assert_eq!(from_a.len(), 2, "2 edges from A");
    }

    // ── QT11: Same-layer enforcement ──────────────────────────────────────

    #[test]
    fn same_layer_accepted() {
        let mut g = SilkGraph::new();
        let ok = g.co_activate_same_layer(0xA, 0xB, 3, 3, emo(-0.5, 0.7), 0.8, 1000);
        assert!(ok, "Same layer → accepted");
        assert_eq!(g.assoc_count(), 1);
    }

    #[test]
    fn different_layer_rejected() {
        let mut g = SilkGraph::new();
        let ok = g.co_activate_same_layer(0xA, 0xB, 3, 5, emo(-0.5, 0.7), 0.8, 1000);
        assert!(!ok, "Different layers → rejected by QT11");
        assert_eq!(g.assoc_count(), 0, "No edge created");
    }

    // ── Cross-layer Silk ───────────────────────────────────────────────────

    #[test]
    fn cross_layer_same_layer_always_true() {
        let mut g = SilkGraph::new();
        let ok = g.co_activate_cross_layer(0xA, 0xB, (3, 3), emo(-0.5, 0.7), 0.8, 1000);
        assert!(ok, "Same-layer → always accepted");
        assert_eq!(g.assoc_count(), 1);
    }

    #[test]
    fn cross_layer_starts_weak() {
        let mut g = SilkGraph::new();
        // L4→L5: diff=1, threshold=Fib[3]=3
        let ok = g.co_activate_cross_layer(0xA, 0xB, (4, 5), emo(-0.5, 0.7), 0.8, 1000);
        assert!(!ok, "First cross-layer co-activation → not enough");
        assert_eq!(g.assoc_count(), 1, "Edge created but weak");
    }

    #[test]
    fn cross_layer_needs_fib_n2_threshold() {
        let mut g = SilkGraph::new();
        // L3→L5: diff=2, threshold=Fib[4]=5
        for i in 0..100 {
            let ok = g.co_activate_cross_layer(0xA, 0xB, (3, 5), emo(-0.5, 0.7), 1.0, i * 1000);
            if ok {
                // Must have fire_count >= 5 AND weight >= 0.7
                let e = g.find_edge(0xA, 0xB, EdgeKind::Assoc).unwrap();
                assert!(e.fire_count >= 5, "fire_count >= Fib[4]=5");
                assert!(e.weight >= 0.7, "weight >= PROMOTE_WEIGHT");
                return;
            }
        }
        panic!("100 co-activations should eventually pass Fib[4]=5 threshold");
    }

    #[test]
    fn cross_layer_deeper_is_harder() {
        // diff=1 → Fib[3]=3, diff=3 → Fib[5]=8
        use crate::hebbian::fib;
        assert!(fib(3) < fib(5), "Deeper diff → higher threshold");
    }

    #[test]
    fn multiple_edge_kinds_same_pair() {
        let mut g = SilkGraph::new();
        // Cùng 1 cặp node có thể có nhiều loại edge khác nhau
        g.connect_structural(0xA, 0xB, EdgeKind::Member, 0);
        g.connect_structural(0xA, 0xB, EdgeKind::Similar, 0);
        g.co_activate(0xA, 0xB, emo(0.0, 0.5), 0.5, 0);

        assert_eq!(g.len(), 3, "3 loại edge khác nhau cùng cặp");
        assert!(g.find_edge(0xA, 0xB, EdgeKind::Member).is_some());
        assert!(g.find_edge(0xA, 0xB, EdgeKind::Similar).is_some());
        assert!(g.find_edge(0xA, 0xB, EdgeKind::Assoc).is_some());
    }

    // ── Maintain — chăm sóc Ln-1 ───────────────────────────────────────────

    #[test]
    fn maintain_decays_and_prunes() {
        let mut g = SilkGraph::new();
        // Tạo edge yếu
        g.co_activate(0xA, 0xB, emo(-0.3, 0.5), 0.3, 0);
        assert_eq!(g.assoc_count(), 1);

        // Maintain với 30 ngày → edge phải bị decay đến < 0.01 và bị xóa
        let thirty_days = 86_400_000_000_000i64 * 30;
        let pruned = g.maintain(thirty_days, 0);
        assert!(pruned > 0, "Edge yếu sau 30 ngày phải bị cắt");
        assert_eq!(g.assoc_count(), 0);
    }

    #[test]
    fn maintain_overflow_pruning() {
        let mut g = SilkGraph::new();
        // Structural: 2 edges (không bị cắt)
        g.connect_structural(0x01, 0x02, EdgeKind::Member, 0);
        g.connect_structural(0x03, 0x04, EdgeKind::Causes, 0);

        // Associative: 10 edges với weight khác nhau
        for i in 0u64..10 {
            g.co_activate(0x100 + i, 0x200 + i, emo(0.0, 0.5), 0.5, 0);
        }
        // Strengthen 5 trong số đó
        for i in 0u64..5 {
            for _ in 0..20 {
                g.co_activate(0x100 + i, 0x200 + i, emo(0.0, 0.5), 0.8, 0);
            }
        }

        assert_eq!(g.structural_count(), 2);
        assert_eq!(g.assoc_count(), 10);

        // Max = 7 (2 structural + 5 assoc capacity)
        let pruned = g.maintain(0, 7);
        assert!(pruned > 0, "Overflow phải cắt: pruned={}", pruned);
        assert!(g.len() <= 7, "Tổng edges <= max: {}", g.len());
        // Structural vẫn còn
        assert_eq!(g.structural_count(), 2, "Structural không bị cắt");
    }

    #[test]
    fn maintain_structural_untouched() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xA, 0xB, EdgeKind::Member, 0);
        g.connect_structural(0xC, 0xD, EdgeKind::Causes, 0);

        let pruned = g.maintain(86_400_000_000_000 * 365, 1); // 1 năm, max=1
        assert_eq!(pruned, 0, "Structural edges không bao giờ bị cắt");
        assert_eq!(g.len(), 2);
    }

    #[test]
    fn maintain_zero_elapsed_no_decay() {
        let mut g = SilkGraph::new();
        for _ in 0..30 {
            g.co_activate(0xA, 0xB, emo(0.0, 0.5), 1.0, 0);
        }
        let w_before = g.assoc_weight(0xA, 0xB);
        g.maintain(0, 0);
        let w_after = g.assoc_weight(0xA, 0xB);
        assert_eq!(w_before, w_after, "0 elapsed → không decay");
    }

    // ── MolSummary 5D similarity tests ──────────────────────────────────────

    #[test]
    fn mol_similarity_identical() {
        let a = MolSummary { shape: 0x01, relation: 0x01, valence: 0xC0, arousal: 0xC0, time: 0x04 };
        assert!((a.similarity(&a) - 1.0).abs() < 0.01, "Identical → 1.0");
    }

    #[test]
    fn mol_similarity_same_base_different_sub() {
        // Sphere sub 0 vs Sphere sub 1 — cùng base
        let a = MolSummary { shape: 0x01, relation: 0x01, valence: 0x80, arousal: 0x80, time: 0x03 };
        let b = MolSummary { shape: 0x09, relation: 0x01, valence: 0x80, arousal: 0x80, time: 0x03 };
        let sim = a.similarity(&b);
        assert!(sim >= 0.8, "Same base on all dims → high sim: {}", sim);
    }

    #[test]
    fn mol_similarity_different_shape() {
        // Sphere vs Capsule — only shape differs
        let a = MolSummary { shape: 0x01, relation: 0x01, valence: 0x80, arousal: 0x80, time: 0x03 };
        let b = MolSummary { shape: 0x02, relation: 0x01, valence: 0x80, arousal: 0x80, time: 0x03 };
        let sim = a.similarity(&b);
        assert!((0.6..1.0).contains(&sim), "4/5 dims match → 0.8: {}", sim);
    }

    #[test]
    fn mol_similarity_very_different() {
        let a = MolSummary { shape: 0x01, relation: 0x01, valence: 0x00, arousal: 0x00, time: 0x01 };
        let b = MolSummary { shape: 0x04, relation: 0x06, valence: 0xFF, arousal: 0xFF, time: 0x05 };
        let sim = a.similarity(&b);
        assert!(sim < 0.2, "Completely different → low sim: {}", sim);
    }

    #[test]
    fn co_activate_mol_boosts_similar_nodes() {
        let mut g = SilkGraph::new();
        let fire = MolSummary { shape: 0x01, relation: 0x01, valence: 0xC0, arousal: 0xC0, time: 0x04 };
        let sun = MolSummary { shape: 0x01, relation: 0x01, valence: 0xC0, arousal: 0x90, time: 0x04 };

        // With mol similarity (fire ≈ sun: 4/5 dims match)
        g.co_activate_mol(0xA, 0xB, Some(fire), Some(sun), emo(0.5, 0.7), 0.5, 1000);
        let w_mol = g.assoc_weight(0xA, 0xB);

        // Without mol similarity
        let mut g2 = SilkGraph::new();
        g2.co_activate(0xA, 0xB, emo(0.5, 0.7), 0.5, 1000);
        let w_plain = g2.assoc_weight(0xA, 0xB);

        assert!(w_mol >= w_plain, "5D similarity should boost edge: mol={} plain={}", w_mol, w_plain);
    }

    #[test]
    fn cluster_score_with_5d() {
        let mut g = SilkGraph::new();
        g.co_activate(0xA, 0xB, emo(0.5, 0.7), 0.8, 1000);

        let fire = MolSummary { shape: 0x01, relation: 0x01, valence: 0xC0, arousal: 0xC0, time: 0x04 };
        let sun = MolSummary { shape: 0x01, relation: 0x01, valence: 0xC0, arousal: 0x90, time: 0x04 };

        let score_5d = g.cluster_score(0xA, 0xB, Some(fire), Some(sun), 1);
        let score_partial = g.cluster_score_partial(0xA, 0xB, 1);

        assert!(score_5d > score_partial, "5D similarity adds chain_sim component: {} > {}", score_5d, score_partial);
    }
}
