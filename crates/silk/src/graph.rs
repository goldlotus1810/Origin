//! # graph — SilkGraph
//!
//! In-memory graph của tất cả Silk edges.
//! Sorted by (from_hash, to_hash, kind) cho binary search O(log n).
//!
//! ## Dual-threshold cluster (Dream):
//!   cluster_score(A,B) =
//!     0.3 × chain_similarity(A,B) +
//!     0.4 × hebbian_weight(A,B) +
//!     0.3 × co_activation_count(A,B) / max_count
//!   cluster nếu score ≥ 0.6

extern crate alloc;
use alloc::vec::Vec;

use crate::edge::{SilkEdge, EdgeKind, EmotionTag};
use crate::hebbian::{
    hebbian_strengthen, hebbian_decay, blend_emotion,
    should_promote, fib, PROMOTE_WEIGHT,
};

// ─────────────────────────────────────────────────────────────────────────────
// SilkGraph
// ─────────────────────────────────────────────────────────────────────────────

/// In-memory Silk graph.
///
/// Tất cả edges sorted by key (from, to, kind) để binary search.
pub struct SilkGraph {
    edges: Vec<SilkEdge>,
}

impl SilkGraph {
    /// Tạo graph rỗng.
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    // ── Insert / Connect ─────────────────────────────────────────────────────

    /// Thêm structural edge (weight=1.0, bất biến).
    pub fn connect_structural(
        &mut self,
        from: u64, to: u64,
        kind: EdgeKind, ts: i64,
    ) {
        if self.find_edge(from, to, kind).is_some() { return; }
        let edge = SilkEdge::structural(from, to, kind, ts);
        self.insert_sorted(edge);
    }

    /// Thêm hoặc cập nhật associative edge.
    ///
    /// Nếu edge đã có → Hebbian strengthen.
    /// Nếu chưa có → tạo mới với weight thấp.
    pub fn co_activate(
        &mut self,
        from:    u64,
        to:      u64,
        emotion: EmotionTag,
        reward:  f32,
        ts:      i64,
    ) {
        let key = (from, to, EdgeKind::Assoc.as_byte());

        if let Some(idx) = self.find_edge_idx(from, to, EdgeKind::Assoc) {
            // Edge đã có → strengthen
            let e = &mut self.edges[idx];
            e.weight     = hebbian_strengthen(e.weight, reward);
            e.emotion    = blend_emotion(e.emotion, emotion, emotion.intensity);
            e.fire_count = e.fire_count.saturating_add(1);
            e.updated_at = ts;
        } else {
            // Edge chưa có → tạo mới
            let _ = key;
            let edge = SilkEdge::associative(from, to, emotion, ts);
            self.insert_sorted(edge);
        }
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
        from: u64, to: u64,
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
            e.weight     = hebbian_strengthen(e.weight, reward);
            e.emotion    = blend_emotion(e.emotion, emotion, emotion.intensity);
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
        self.edges.retain(|e| !e.kind.is_associative() || e.weight >= 0.01);
    }

    // ── Lookup ───────────────────────────────────────────────────────────────

    /// Tìm edge bằng (from, to, kind).
    pub fn find_edge(&self, from: u64, to: u64, kind: EdgeKind) -> Option<&SilkEdge> {
        self.find_edge_idx(from, to, kind)
            .map(|i| &self.edges[i])
    }

    /// Tất cả edges từ một node.
    pub fn edges_from(&self, from: u64) -> Vec<&SilkEdge> {
        self.edges.iter()
            .filter(|e| e.from_hash == from)
            .collect()
    }

    /// Tất cả edges đến một node.
    pub fn edges_to(&self, to: u64) -> Vec<&SilkEdge> {
        self.edges.iter()
            .filter(|e| e.to_hash == to)
            .collect()
    }

    /// Neighbors của một node (cả 2 chiều).
    pub fn neighbors(&self, hash: u64) -> Vec<u64> {
        let mut ns: Vec<u64> = self.edges.iter()
            .filter_map(|e| {
                if e.from_hash == hash { Some(e.to_hash) }
                else if e.to_hash == hash { Some(e.from_hash) }
                else { None }
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

    /// Tính cluster_score(A, B) cho Dream.
    ///
    /// score = 0.4 × hebbian_weight + 0.3 × co_activation_ratio
    /// (chain_similarity được tính bên ngoài bởi LCA engine)
    pub fn cluster_score_partial(
        &self,
        hash_a: u64,
        hash_b: u64,
        max_fire_count: u32,
    ) -> f32 {
        let weight = self.assoc_weight(hash_a, hash_b)
            .max(self.assoc_weight(hash_b, hash_a));

        let fire = self.find_edge(hash_a, hash_b, EdgeKind::Assoc)
            .or_else(|| self.find_edge(hash_b, hash_a, EdgeKind::Assoc))
            .map(|e| e.fire_count)
            .unwrap_or(0);

        let fire_ratio = if max_fire_count > 0 {
            fire as f32 / max_fire_count as f32
        } else { 0.0 };

        0.4 * weight + 0.3 * fire_ratio
    }

    /// Tìm candidates cần promote tại một tầng (Dream input).
    ///
    /// Trả về Vec<(hash_a, hash_b, score)> sorted by score desc.
    pub fn promote_candidates(
        &self,
        depth: usize,
    ) -> Vec<(u64, u64, f32)> {
        let max_fire = self.edges.iter()
            .filter(|e| e.kind.is_associative())
            .map(|e| e.fire_count)
            .max()
            .unwrap_or(1);

        let mut candidates: Vec<(u64, u64, f32)> = self.edges.iter()
            .filter(|e| {
                e.kind.is_associative()
                    && should_promote(e.weight, e.fire_count, depth)
            })
            .map(|e| {
                let score = self.cluster_score_partial(
                    e.from_hash, e.to_hash, max_fire,
                );
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

    pub fn len(&self) -> usize { self.edges.len() }

    /// Graph có rỗng không.
    pub fn is_empty(&self) -> bool { self.edges.is_empty() }

    /// Số associative edges.
    pub fn assoc_count(&self) -> usize {
        self.edges.iter().filter(|e| e.kind.is_associative()).count()
    }

    /// Số structural edges.
    pub fn structural_count(&self) -> usize {
        self.edges.iter().filter(|e| e.kind.is_structural()).count()
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
    fn default() -> Self { Self::new() }
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

        let e = g.find_edge(0xF1BE_u64, 0xDA4E_u64, EdgeKind::Assoc).unwrap();
        // Edge phải mang màu cảm xúc của khoảnh khắc đó
        assert!(e.emotion.arousal > 0.5,
            "Edge mang arousal cao của khoảnh khắc: a={}", e.emotion.arousal);
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

        assert!(w_after < w_before,
            "Decay phải giảm weight: {} → {}", w_before, w_after);
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
        assert!(!candidates.is_empty(),
            "Sau 100 co-activations phải có candidates");
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

        assert!(score2 > score1,
            "Thêm co-activation → score tăng: {} → {}", score1, score2);
    }

    #[test]
    fn edges_from_correct() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xA, 0xB, EdgeKind::Member,  0);
        g.connect_structural(0xA, 0xC, EdgeKind::Causes,  0);
        g.connect_structural(0xB, 0xA, EdgeKind::Similar, 0);

        let from_a = g.edges_from(0xA);
        assert_eq!(from_a.len(), 2, "2 edges from A");
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
            let ok = g.co_activate_cross_layer(
                0xA, 0xB, (3, 5), emo(-0.5, 0.7), 1.0, i * 1000,
            );
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
        g.connect_structural(0xA, 0xB, EdgeKind::Member,  0);
        g.connect_structural(0xA, 0xB, EdgeKind::Similar, 0);
        g.co_activate(0xA, 0xB, emo(0.0, 0.5), 0.5, 0);

        assert_eq!(g.len(), 3, "3 loại edge khác nhau cùng cặp");
        assert!(g.find_edge(0xA, 0xB, EdgeKind::Member).is_some());
        assert!(g.find_edge(0xA, 0xB, EdgeKind::Similar).is_some());
        assert!(g.find_edge(0xA, 0xB, EdgeKind::Assoc).is_some());
    }
}
