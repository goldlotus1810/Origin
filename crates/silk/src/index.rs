//! # index — SilkIndex: Implicit 5D Connectivity
//!
//! Silk không phải dữ liệu. Silk là HỆ QUẢ TOÁN HỌC của không gian 5D.
//! Khi 2 node chia sẻ base value trên bất kỳ chiều nào → Silk TỰ TỒN TẠI.
//!
//! 37 kênh Silk cơ bản: 8 Shape + 8 Relation + 8 Valence zone + 8 Arousal zone + 5 Time
//! 31 mẫu compound: C(5,1)=5 + C(5,2)=10 + C(5,3)=10 + C(5,4)=5 + C(5,5)=1
//! = 1147 kiểu quan hệ có nghĩa (37 × 31)
//!
//! Storage: 0 bytes cho edges. Chỉ index node → bucket.

extern crate alloc;
use alloc::vec::Vec;

use crate::graph::MolSummary;

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Số base categories cho Shape (8 primitives: Sphere, Capsule, Box, Cone, Torus, Union, Intersect, Diff)
const SHAPE_BASES: usize = 8;
/// Số base categories cho Relation (8: Member, Subset, Equiv, Orthogonal, Compose, Causes, Similar, DerivedFrom)
const RELATION_BASES: usize = 8;
/// Số zones cho Valence (0x00..0xFF ÷ 32 = 8 zones)
const VALENCE_ZONES: usize = 8;
/// Số zones cho Arousal (0x00..0xFF ÷ 32 = 8 zones)
const AROUSAL_ZONES: usize = 8;
/// Số base categories cho Time (5: Static, Slow, Medium, Fast, Instant)
const TIME_BASES: usize = 5;

/// Tổng kênh Silk = 37
pub const TOTAL_CHANNELS: usize =
    SHAPE_BASES + RELATION_BASES + VALENCE_ZONES + AROUSAL_ZONES + TIME_BASES;

// ─────────────────────────────────────────────────────────────────────────────
// SilkChannel — 1 chiều kết nối
// ─────────────────────────────────────────────────────────────────────────────

/// Một chiều Silk mà 2 nodes chia sẻ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SilkDim {
    Shape(u8),    // base category 1..8
    Relation(u8), // base category 1..8
    Valence(u8),  // zone 0..7
    Arousal(u8),  // zone 0..7
    Time(u8),     // base category 1..5
}

/// Kết quả so sánh implicit Silk giữa 2 nodes.
#[derive(Debug, Clone)]
pub struct ImplicitSilk {
    /// Các chiều chia sẻ (0..5 chiều)
    pub shared_dims: Vec<SilkDim>,
    /// Sức mạnh kết nối = shared_count / 5.0 (base)
    /// + precision bonus nếu cùng exact value (không chỉ cùng base)
    pub strength: f32,
    /// Số chiều chia sẻ (0..5)
    pub shared_count: u8,
}

impl ImplicitSilk {
    /// Không có kết nối implicit.
    pub fn none() -> Self {
        Self {
            shared_dims: Vec::new(),
            strength: 0.0,
            shared_count: 0,
        }
    }

    /// Classify compound pattern từ shared_dims.
    ///
    /// 31 mẫu = C(5,1) + C(5,2) + C(5,3) + C(5,4) + C(5,5).
    /// None nếu shared_count == 0.
    pub fn compound_kind(&self) -> Option<CompoundKind> {
        if self.shared_count == 0 {
            return None;
        }
        let has_s = self.shared_dims.iter().any(|d| matches!(d, SilkDim::Shape(_)));
        let has_r = self.shared_dims.iter().any(|d| matches!(d, SilkDim::Relation(_)));
        let has_v = self.shared_dims.iter().any(|d| matches!(d, SilkDim::Valence(_)));
        let has_a = self.shared_dims.iter().any(|d| matches!(d, SilkDim::Arousal(_)));
        let has_t = self.shared_dims.iter().any(|d| matches!(d, SilkDim::Time(_)));

        Some(match (has_s, has_r, has_v, has_a, has_t) {
            // 5 chiều (1 mẫu)
            (true,  true,  true,  true,  true)  => CompoundKind::Identical,
            // 4 chiều (5 mẫu)
            (false, true,  true,  true,  true)  => CompoundKind::AllButShape,
            (true,  false, true,  true,  true)  => CompoundKind::AllButRelation,
            (true,  true,  false, true,  true)  => CompoundKind::AllButValence,
            (true,  true,  true,  false, true)  => CompoundKind::AllButArousal,
            (true,  true,  true,  true,  false) => CompoundKind::AllButTime,
            // 3 chiều (10 mẫu)
            (true,  true,  true,  false, false) => CompoundKind::ShapeRelationValence,
            (true,  true,  false, true,  false) => CompoundKind::ShapeRelationArousal,
            (true,  true,  false, false, true)  => CompoundKind::ShapeRelationTime,
            (true,  false, true,  true,  false) => CompoundKind::ShapeValenceArousal,
            (true,  false, true,  false, true)  => CompoundKind::ShapeValenceTime,
            (true,  false, false, true,  true)  => CompoundKind::ShapeArousalTime,
            (false, true,  true,  true,  false) => CompoundKind::RelationValenceArousal,
            (false, true,  true,  false, true)  => CompoundKind::RelationValenceTime,
            (false, true,  false, true,  true)  => CompoundKind::RelationArousalTime,
            (false, false, true,  true,  true)  => CompoundKind::ValenceArousalTime,
            // 2 chiều (10 mẫu)
            (true,  true,  false, false, false) => CompoundKind::ShapeRelation,
            (true,  false, true,  false, false) => CompoundKind::ShapeValence,
            (true,  false, false, true,  false) => CompoundKind::ShapeArousal,
            (true,  false, false, false, true)  => CompoundKind::ShapeTime,
            (false, true,  true,  false, false) => CompoundKind::RelationValence,
            (false, true,  false, true,  false) => CompoundKind::RelationArousal,
            (false, true,  false, false, true)  => CompoundKind::RelationTime,
            (false, false, true,  true,  false) => CompoundKind::ValenceArousal,
            (false, false, true,  false, true)  => CompoundKind::ValenceTime,
            (false, false, false, true,  true)  => CompoundKind::ArousalTime,
            // 1 chiều (5 mẫu)
            (true,  false, false, false, false) => CompoundKind::ShapeOnly,
            (false, true,  false, false, false) => CompoundKind::RelationOnly,
            (false, false, true,  false, false) => CompoundKind::ValenceOnly,
            (false, false, false, true,  false) => CompoundKind::ArousalOnly,
            (false, false, false, false, true)  => CompoundKind::TimeOnly,
            // Impossible (shared_count > 0 nhưng no dims) → unreachable
            (false, false, false, false, false) => return None,
        })
    }
}

/// 31 compound patterns — phân loại kiểu quan hệ theo số chiều chung.
///
/// C(5,1)=5, C(5,2)=10, C(5,3)=10, C(5,4)=5, C(5,5)=1 = 31 mẫu.
/// 37 kênh × 31 mẫu = 1147 kiểu quan hệ có nghĩa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompoundKind {
    // ── 1 chiều chung (5 mẫu) ──────────────────────────────────────────
    ShapeOnly,
    RelationOnly,
    ValenceOnly,
    ArousalOnly,
    TimeOnly,

    // ── 2 chiều chung (10 mẫu) ─────────────────────────────────────────
    ShapeRelation,
    ShapeValence,     // ẩn dụ thị giác
    ShapeArousal,
    ShapeTime,        // animation family
    RelationValence,  // moral analog
    RelationArousal,
    RelationTime,
    ValenceArousal,   // empathy link
    ValenceTime,
    ArousalTime,

    // ── 3 chiều chung (10 mẫu) ─────────────────────────────────────────
    ShapeRelationValence,   // gần như cùng khái niệm
    ShapeRelationArousal,
    ShapeRelationTime,
    ShapeValenceArousal,
    ShapeValenceTime,
    ShapeArousalTime,
    RelationValenceArousal,
    RelationValenceTime,
    RelationArousalTime,
    ValenceArousalTime,

    // ── 4 chiều chung (5 mẫu) ──────────────────────────────────────────
    AllButShape,      // khác hình, giống hết → ẩn dụ sâu
    AllButRelation,
    AllButValence,
    AllButArousal,
    AllButTime,

    // ── 5 chiều chung (1 mẫu) ──────────────────────────────────────────
    Identical,        // cùng node
}

impl CompoundKind {
    /// Số chiều chung.
    pub fn shared_count(&self) -> u8 {
        match self {
            Self::ShapeOnly | Self::RelationOnly | Self::ValenceOnly
            | Self::ArousalOnly | Self::TimeOnly => 1,

            Self::ShapeRelation | Self::ShapeValence | Self::ShapeArousal
            | Self::ShapeTime | Self::RelationValence | Self::RelationArousal
            | Self::RelationTime | Self::ValenceArousal | Self::ValenceTime
            | Self::ArousalTime => 2,

            Self::ShapeRelationValence | Self::ShapeRelationArousal
            | Self::ShapeRelationTime | Self::ShapeValenceArousal
            | Self::ShapeValenceTime | Self::ShapeArousalTime
            | Self::RelationValenceArousal | Self::RelationValenceTime
            | Self::RelationArousalTime | Self::ValenceArousalTime => 3,

            Self::AllButShape | Self::AllButRelation | Self::AllButValence
            | Self::AllButArousal | Self::AllButTime => 4,

            Self::Identical => 5,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SilkIndex — 37 buckets for implicit connectivity
// ─────────────────────────────────────────────────────────────────────────────

/// Index 5D cho implicit Silk connectivity.
///
/// Mỗi node được index vào 5 bucket (1 per dimension).
/// Query "ai cùng Shape=Sphere?" = O(1) bucket lookup.
///
/// 37 kênh × sorted Vec<u64> = toàn bộ implicit Silk network.
/// 0 bytes cho edges — chỉ lưu membership.
pub struct SilkIndex {
    /// Shape buckets: base_category (1..8) → sorted node hashes
    shape: [Vec<u64>; SHAPE_BASES],
    /// Relation buckets: base_category (1..8) → sorted node hashes
    relation: [Vec<u64>; RELATION_BASES],
    /// Valence zone buckets: zone (0..7) → sorted node hashes
    valence: [Vec<u64>; VALENCE_ZONES],
    /// Arousal zone buckets: zone (0..7) → sorted node hashes
    arousal: [Vec<u64>; AROUSAL_ZONES],
    /// Time buckets: base_category (1..5) → sorted node hashes
    time: [Vec<u64>; TIME_BASES],
    /// Tổng số nodes đã index
    node_count: usize,
}

impl SilkIndex {
    /// Tạo index rỗng.
    pub fn new() -> Self {
        Self {
            shape: Default::default(),
            relation: Default::default(),
            valence: Default::default(),
            arousal: Default::default(),
            time: Default::default(),
            node_count: 0,
        }
    }

    /// Index 1 node vào tất cả 5 buckets.
    ///
    /// Gọi khi node mới được tạo hoặc khi boot từ file.
    pub fn index_node(&mut self, hash: u64, mol: &MolSummary) {
        // Shape bucket
        let s_base = shape_base(mol.shape);
        if s_base < SHAPE_BASES {
            insert_sorted(&mut self.shape[s_base], hash);
        }

        // Relation bucket
        let r_base = relation_base(mol.relation);
        if r_base < RELATION_BASES {
            insert_sorted(&mut self.relation[r_base], hash);
        }

        // Valence zone (0..7)
        let v_zone = (mol.valence / 32) as usize;
        if v_zone < VALENCE_ZONES {
            insert_sorted(&mut self.valence[v_zone], hash);
        }

        // Arousal zone (0..7)
        let a_zone = (mol.arousal / 32) as usize;
        if a_zone < AROUSAL_ZONES {
            insert_sorted(&mut self.arousal[a_zone], hash);
        }

        // Time bucket
        let t_base = time_base(mol.time);
        if t_base < TIME_BASES {
            insert_sorted(&mut self.time[t_base], hash);
        }

        self.node_count += 1;
    }

    /// Remove node from all buckets (khi node bị prune — hiếm).
    pub fn remove_node(&mut self, hash: u64, mol: &MolSummary) {
        let s_base = shape_base(mol.shape);
        if s_base < SHAPE_BASES {
            remove_sorted(&mut self.shape[s_base], hash);
        }
        let r_base = relation_base(mol.relation);
        if r_base < RELATION_BASES {
            remove_sorted(&mut self.relation[r_base], hash);
        }
        let v_zone = (mol.valence / 32) as usize;
        if v_zone < VALENCE_ZONES {
            remove_sorted(&mut self.valence[v_zone], hash);
        }
        let a_zone = (mol.arousal / 32) as usize;
        if a_zone < AROUSAL_ZONES {
            remove_sorted(&mut self.arousal[a_zone], hash);
        }
        let t_base = time_base(mol.time);
        if t_base < TIME_BASES {
            remove_sorted(&mut self.time[t_base], hash);
        }
        self.node_count = self.node_count.saturating_sub(1);
    }

    // ── Implicit Silk queries ──────────────────────────────────────────────

    /// Tính implicit Silk giữa 2 nodes từ 5D position.
    ///
    /// Không cần tìm edge — chỉ so sánh tọa độ.
    /// O(1) — chỉ so sánh 5 bytes.
    pub fn implicit_silk(a: &MolSummary, b: &MolSummary) -> ImplicitSilk {
        let mut shared = Vec::new();
        let mut strength = 0.0f32;

        // Shape: cùng base (8 categories)
        let sa = shape_base(a.shape);
        let sb = shape_base(b.shape);
        if sa == sb && sa < SHAPE_BASES {
            let base_idx = (sa + 1) as u8;
            shared.push(SilkDim::Shape(base_idx));
            strength += if a.shape == b.shape { 0.20 } else { 0.15 };
        }

        // Relation: cùng base
        let ra = relation_base(a.relation);
        let rb = relation_base(b.relation);
        if ra == rb && ra < RELATION_BASES {
            let base_idx = (ra + 1) as u8;
            shared.push(SilkDim::Relation(base_idx));
            strength += if a.relation == b.relation { 0.20 } else { 0.15 };
        }

        // Valence: cùng zone (32-wide)
        let va = a.valence / 32;
        let vb = b.valence / 32;
        if va == vb {
            shared.push(SilkDim::Valence(va));
            // Precision bonus: closer values → stronger
            let delta = (a.valence as i16 - b.valence as i16).unsigned_abs();
            strength += if delta < 8 { 0.20 } else { 0.15 };
        }

        // Arousal: cùng zone
        let aa = a.arousal / 32;
        let ab = b.arousal / 32;
        if aa == ab {
            shared.push(SilkDim::Arousal(aa));
            let delta = (a.arousal as i16 - b.arousal as i16).unsigned_abs();
            strength += if delta < 8 { 0.20 } else { 0.15 };
        }

        // Time: cùng base (5 categories)
        let ta = time_base(a.time);
        let tb = time_base(b.time);
        if ta == tb && ta < TIME_BASES {
            let base_idx = (ta + 1) as u8;
            shared.push(SilkDim::Time(base_idx));
            strength += if a.time == b.time { 0.20 } else { 0.15 };
        }

        let count = shared.len() as u8;
        ImplicitSilk {
            shared_dims: shared,
            strength,
            shared_count: count,
        }
    }

    /// Tất cả nodes cùng bucket trên 1 chiều.
    ///
    /// Ví dụ: "tất cả nodes có Shape=Sphere"
    pub fn nodes_sharing_dim(&self, dim: SilkDim) -> &[u64] {
        match dim {
            SilkDim::Shape(base) => {
                let idx = (base.wrapping_sub(1)) as usize;
                if idx < SHAPE_BASES { &self.shape[idx] } else { &[] }
            }
            SilkDim::Relation(base) => {
                let idx = (base.wrapping_sub(1)) as usize;
                if idx < RELATION_BASES { &self.relation[idx] } else { &[] }
            }
            SilkDim::Valence(zone) => {
                let idx = zone as usize;
                if idx < VALENCE_ZONES { &self.valence[idx] } else { &[] }
            }
            SilkDim::Arousal(zone) => {
                let idx = zone as usize;
                if idx < AROUSAL_ZONES { &self.arousal[idx] } else { &[] }
            }
            SilkDim::Time(base) => {
                let idx = (base.wrapping_sub(1)) as usize;
                if idx < TIME_BASES { &self.time[idx] } else { &[] }
            }
        }
    }

    /// Implicit neighbors: tất cả nodes chia sẻ ÍT NHẤT 1 chiều với mol.
    ///
    /// Trả về (hash, shared_count) sorted by shared_count desc.
    /// Không bao gồm chính node đó.
    pub fn implicit_neighbors(&self, hash: u64, mol: &MolSummary) -> Vec<(u64, u8)> {
        use alloc::collections::BTreeMap;
        let mut counts: BTreeMap<u64, u8> = BTreeMap::new();

        // Shape bucket
        let s_base = shape_base(mol.shape);
        if s_base < SHAPE_BASES {
            for &h in &self.shape[s_base] {
                if h != hash {
                    *counts.entry(h).or_insert(0) += 1;
                }
            }
        }

        // Relation bucket
        let r_base = relation_base(mol.relation);
        if r_base < RELATION_BASES {
            for &h in &self.relation[r_base] {
                if h != hash {
                    *counts.entry(h).or_insert(0) += 1;
                }
            }
        }

        // Valence zone
        let v_zone = (mol.valence / 32) as usize;
        if v_zone < VALENCE_ZONES {
            for &h in &self.valence[v_zone] {
                if h != hash {
                    *counts.entry(h).or_insert(0) += 1;
                }
            }
        }

        // Arousal zone
        let a_zone = (mol.arousal / 32) as usize;
        if a_zone < AROUSAL_ZONES {
            for &h in &self.arousal[a_zone] {
                if h != hash {
                    *counts.entry(h).or_insert(0) += 1;
                }
            }
        }

        // Time bucket
        let t_base = time_base(mol.time);
        if t_base < TIME_BASES {
            for &h in &self.time[t_base] {
                if h != hash {
                    *counts.entry(h).or_insert(0) += 1;
                }
            }
        }

        let mut result: Vec<(u64, u8)> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1)); // desc by shared_count
        result
    }

    // ── Stats ──────────────────────────────────────────────────────────────

    /// Tổng số nodes đã index.
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    /// Estimated RAM usage in bytes.
    pub fn memory_usage(&self) -> usize {
        let mut total = core::mem::size_of::<Self>();
        for b in &self.shape {
            total += b.capacity() * 8;
        }
        for b in &self.relation {
            total += b.capacity() * 8;
        }
        for b in &self.valence {
            total += b.capacity() * 8;
        }
        for b in &self.arousal {
            total += b.capacity() * 8;
        }
        for b in &self.time {
            total += b.capacity() * 8;
        }
        total
    }

    /// Bucket stats: (dimension_name, bucket_index, count).
    pub fn bucket_stats(&self) -> Vec<(&'static str, usize, usize)> {
        let mut stats = Vec::new();
        for (i, b) in self.shape.iter().enumerate() {
            stats.push(("Shape", i, b.len()));
        }
        for (i, b) in self.relation.iter().enumerate() {
            stats.push(("Relation", i, b.len()));
        }
        for (i, b) in self.valence.iter().enumerate() {
            stats.push(("Valence", i, b.len()));
        }
        for (i, b) in self.arousal.iter().enumerate() {
            stats.push(("Arousal", i, b.len()));
        }
        for (i, b) in self.time.iter().enumerate() {
            stats.push(("Time", i, b.len()));
        }
        stats
    }
}

impl Default for SilkIndex {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: extract base category from hierarchical byte
// ─────────────────────────────────────────────────────────────────────────────

/// Extract Shape base: value → 0-based index (0..7).
/// base = ((value - 1) % 8) for value > 0, else 0 (Sphere default).
fn shape_base(value: u8) -> usize {
    if value == 0 { 0 } else { ((value - 1) % 8) as usize }
}

/// Extract Relation base: same encoding as Shape.
fn relation_base(value: u8) -> usize {
    if value == 0 { 0 } else { ((value - 1) % 8) as usize }
}

/// Extract Time base: value → 0-based index (0..4).
/// base = ((value - 1) % 5) for value > 0, else 2 (Medium default).
fn time_base(value: u8) -> usize {
    if value == 0 { 2 } else { ((value - 1) % 5) as usize }
}

/// Insert hash into sorted vec (no duplicates).
fn insert_sorted(vec: &mut Vec<u64>, hash: u64) {
    let pos = vec.partition_point(|&h| h < hash);
    if pos < vec.len() && vec[pos] == hash {
        return; // already indexed
    }
    vec.insert(pos, hash);
}

/// Remove hash from sorted vec.
fn remove_sorted(vec: &mut Vec<u64>, hash: u64) {
    let pos = vec.partition_point(|&h| h < hash);
    if pos < vec.len() && vec[pos] == hash {
        vec.remove(pos);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn fire_mol() -> MolSummary {
        // 🔥 = [Sphere, Causes, V=0xC0, A=0xC0, Fast]
        MolSummary { shape: 0x01, relation: 0x06, valence: 0xC0, arousal: 0xC0, time: 0x04 }
    }

    fn anger_mol() -> MolSummary {
        // 😡 = [Sphere, Causes, V=0xC0, A=0xC0, Fast]
        MolSummary { shape: 0x01, relation: 0x06, valence: 0xC0, arousal: 0xC0, time: 0x04 }
    }

    fn ice_mol() -> MolSummary {
        // ❄️ = [Sphere, Causes, V=0x30, A=0x30, Slow]
        MolSummary { shape: 0x01, relation: 0x06, valence: 0x30, arousal: 0x30, time: 0x02 }
    }

    fn sadness_mol() -> MolSummary {
        // buồn = [Ring, Member, V=0x30, A=0x30, Slow]
        MolSummary { shape: 0x05, relation: 0x01, valence: 0x30, arousal: 0x30, time: 0x02 }
    }

    fn default_mol() -> MolSummary {
        MolSummary { shape: 0x01, relation: 0x01, valence: 0x80, arousal: 0x80, time: 0x03 }
    }

    // ── Implicit Silk ────────────────────────────────────────────────────────

    #[test]
    fn implicit_silk_identical_nodes() {
        let fire = fire_mol();
        let anger = anger_mol();
        let silk = SilkIndex::implicit_silk(&fire, &anger);
        assert_eq!(silk.shared_count, 5, "🔥 và 😡 = 5/5 chiều giống");
        assert!((silk.strength - 1.0).abs() < 0.01, "Strength = 1.0 (identical)");
    }

    #[test]
    fn implicit_silk_partial_match() {
        let fire = fire_mol();
        let ice = ice_mol();
        let silk = SilkIndex::implicit_silk(&fire, &ice);
        // Shape=Sphere (same), Relation=Causes (same), V khác zone, A khác zone, Time khác
        assert_eq!(silk.shared_count, 2, "🔥 và ❄️ = 2/5 chiều giống (Shape + Relation)");
        assert!(silk.strength > 0.0 && silk.strength < 0.5);
    }

    #[test]
    fn implicit_silk_no_match() {
        let fire = fire_mol();
        let sad = sadness_mol();
        // Shape: Sphere(0x01) vs Ring(0x05) → khác base
        // Relation: Causes(0x06) vs Member(0x01) → khác base
        // Valence: 0xC0 (zone 6) vs 0x30 (zone 1) → khác zone
        // Arousal: 0xC0 (zone 6) vs 0x30 (zone 1) → khác zone
        // Time: Fast(0x04) vs Slow(0x02) → khác base
        let silk = SilkIndex::implicit_silk(&fire, &sad);
        assert_eq!(silk.shared_count, 0, "🔥 và buồn = 0 chiều chung");
        assert_eq!(silk.strength, 0.0);
    }

    #[test]
    fn implicit_silk_valence_arousal_same_zone() {
        let a = MolSummary { shape: 0x01, relation: 0x01, valence: 0x60, arousal: 0x60, time: 0x03 };
        let b = MolSummary { shape: 0x02, relation: 0x02, valence: 0x65, arousal: 0x68, time: 0x04 };
        let silk = SilkIndex::implicit_silk(&a, &b);
        // V: 0x60/32=3, 0x65/32=3 → same zone
        // A: 0x60/32=3, 0x68/32=3 → same zone
        assert!(silk.shared_dims.iter().any(|d| matches!(d, SilkDim::Valence(3))));
        assert!(silk.shared_dims.iter().any(|d| matches!(d, SilkDim::Arousal(3))));
        assert_eq!(silk.shared_count, 2);
    }

    // ── SilkIndex node operations ────────────────────────────────────────────

    #[test]
    fn index_and_lookup() {
        let mut idx = SilkIndex::new();
        idx.index_node(0xA, &fire_mol());
        idx.index_node(0xB, &ice_mol());
        idx.index_node(0xC, &anger_mol());

        // 🔥 and 😡 share Shape=Sphere bucket
        let sphere_nodes = idx.nodes_sharing_dim(SilkDim::Shape(1)); // Sphere = base 1
        assert!(sphere_nodes.contains(&0xA), "Fire in Sphere bucket");
        assert!(sphere_nodes.contains(&0xB), "Ice in Sphere bucket");
        assert!(sphere_nodes.contains(&0xC), "Anger in Sphere bucket");
    }

    #[test]
    fn implicit_neighbors_count() {
        let mut idx = SilkIndex::new();
        let fire = fire_mol();
        let anger = anger_mol();
        let ice = ice_mol();
        let sad = sadness_mol();

        idx.index_node(0xA, &fire);
        idx.index_node(0xB, &anger);
        idx.index_node(0xC, &ice);
        idx.index_node(0xD, &sad);

        let neighbors = idx.implicit_neighbors(0xA, &fire);
        // 😡 = 5 chiều chung → first
        // ❄️ = 2 chiều chung → second
        // buồn = 0 chiều chung → not included
        assert!(neighbors.len() >= 2, "At least anger + ice: {:?}", neighbors);

        // Anger should have highest shared count
        let anger_entry = neighbors.iter().find(|&&(h, _)| h == 0xB);
        assert!(anger_entry.is_some());
        assert_eq!(anger_entry.unwrap().1, 5, "Anger shares 5 dims with fire");
    }

    #[test]
    fn remove_node_from_index() {
        let mut idx = SilkIndex::new();
        let fire = fire_mol();
        idx.index_node(0xA, &fire);
        assert_eq!(idx.node_count(), 1);

        idx.remove_node(0xA, &fire);
        assert_eq!(idx.node_count(), 0);

        let sphere_nodes = idx.nodes_sharing_dim(SilkDim::Shape(1));
        assert!(!sphere_nodes.contains(&0xA));
    }

    #[test]
    fn index_idempotent() {
        let mut idx = SilkIndex::new();
        let fire = fire_mol();
        idx.index_node(0xA, &fire);
        idx.index_node(0xA, &fire); // duplicate
        // node_count increments but bucket has no dupes
        let sphere_nodes = idx.nodes_sharing_dim(SilkDim::Shape(1));
        let count = sphere_nodes.iter().filter(|&&h| h == 0xA).count();
        assert_eq!(count, 1, "No duplicate in bucket");
    }

    // ── Stats ────────────────────────────────────────────────────────────────

    #[test]
    fn memory_usage_small() {
        let mut idx = SilkIndex::new();
        for i in 0u64..100 {
            idx.index_node(i, &default_mol());
        }
        let mem = idx.memory_usage();
        // 100 nodes × 5 buckets × 8 bytes = 4000 bytes + overhead
        assert!(mem < 10_000, "100 nodes should use < 10KB: {} bytes", mem);
    }

    #[test]
    fn bucket_stats_populated() {
        let mut idx = SilkIndex::new();
        idx.index_node(0xA, &fire_mol());
        let stats = idx.bucket_stats();
        assert_eq!(stats.len(), TOTAL_CHANNELS);

        // Fire (Shape=Sphere, base=0x01) → shape bucket 0 has 1 node
        let shape_0 = stats.iter().find(|s| s.0 == "Shape" && s.1 == 0).unwrap();
        assert_eq!(shape_0.2, 1);
    }

    // ── Base extraction ──────────────────────────────────────────────────────

    #[test]
    fn shape_base_extraction() {
        assert_eq!(shape_base(0x01), 0); // Sphere = base 0
        assert_eq!(shape_base(0x02), 1); // Capsule = base 1
        assert_eq!(shape_base(0x09), 0); // Sphere sub 1 = still base 0
        assert_eq!(shape_base(0x0A), 1); // Capsule sub 1 = still base 1
        assert_eq!(shape_base(0x00), 0); // Default → Sphere
    }

    #[test]
    fn time_base_extraction() {
        assert_eq!(time_base(0x01), 0); // Static
        assert_eq!(time_base(0x02), 1); // Slow
        assert_eq!(time_base(0x03), 2); // Medium
        assert_eq!(time_base(0x04), 3); // Fast
        assert_eq!(time_base(0x05), 4); // Instant
        assert_eq!(time_base(0x06), 0); // Wraps to Static sub 1
        assert_eq!(time_base(0x00), 2); // Default → Medium
    }

    #[test]
    fn valence_zone_boundaries() {
        // Zone 0: 0x00..0x1F, Zone 1: 0x20..0x3F, ..., Zone 7: 0xE0..0xFF
        assert_eq!(0x00u8 / 32, 0);
        assert_eq!(0x1Fu8 / 32, 0);
        assert_eq!(0x20u8 / 32, 1);
        assert_eq!(0x80u8 / 32, 4); // neutral → zone 4
        assert_eq!(0xC0u8 / 32, 6);
        assert_eq!(0xFFu8 / 32, 7);
    }

    // ── CompoundKind classification ──────────────────────────────────────

    #[test]
    fn compound_kind_identical() {
        let fire = fire_mol();
        let anger = anger_mol();
        let silk = SilkIndex::implicit_silk(&fire, &anger);
        assert_eq!(silk.compound_kind(), Some(CompoundKind::Identical));
        assert_eq!(CompoundKind::Identical.shared_count(), 5);
    }

    #[test]
    fn compound_kind_two_dims() {
        let fire = fire_mol();
        let ice = ice_mol();
        let silk = SilkIndex::implicit_silk(&fire, &ice);
        // Shape=Sphere (same), Relation=Causes (same) → ShapeRelation
        assert_eq!(silk.compound_kind(), Some(CompoundKind::ShapeRelation));
        assert_eq!(silk.compound_kind().unwrap().shared_count(), 2);
    }

    #[test]
    fn compound_kind_none_when_no_match() {
        let fire = fire_mol();
        let sad = sadness_mol();
        let silk = SilkIndex::implicit_silk(&fire, &sad);
        assert_eq!(silk.compound_kind(), None);
    }

    #[test]
    fn compound_kind_all_31_variants() {
        // Verify enum covers all 31 patterns
        let all: [CompoundKind; 31] = [
            CompoundKind::ShapeOnly, CompoundKind::RelationOnly,
            CompoundKind::ValenceOnly, CompoundKind::ArousalOnly, CompoundKind::TimeOnly,
            CompoundKind::ShapeRelation, CompoundKind::ShapeValence,
            CompoundKind::ShapeArousal, CompoundKind::ShapeTime,
            CompoundKind::RelationValence, CompoundKind::RelationArousal,
            CompoundKind::RelationTime, CompoundKind::ValenceArousal,
            CompoundKind::ValenceTime, CompoundKind::ArousalTime,
            CompoundKind::ShapeRelationValence, CompoundKind::ShapeRelationArousal,
            CompoundKind::ShapeRelationTime, CompoundKind::ShapeValenceArousal,
            CompoundKind::ShapeValenceTime, CompoundKind::ShapeArousalTime,
            CompoundKind::RelationValenceArousal, CompoundKind::RelationValenceTime,
            CompoundKind::RelationArousalTime, CompoundKind::ValenceArousalTime,
            CompoundKind::AllButShape, CompoundKind::AllButRelation,
            CompoundKind::AllButValence, CompoundKind::AllButArousal, CompoundKind::AllButTime,
            CompoundKind::Identical,
        ];
        assert_eq!(all.len(), 31);
        // Verify shared_count distribution: 5+10+10+5+1 = 31
        let ones = all.iter().filter(|k| k.shared_count() == 1).count();
        let twos = all.iter().filter(|k| k.shared_count() == 2).count();
        let threes = all.iter().filter(|k| k.shared_count() == 3).count();
        let fours = all.iter().filter(|k| k.shared_count() == 4).count();
        let fives = all.iter().filter(|k| k.shared_count() == 5).count();
        assert_eq!((ones, twos, threes, fours, fives), (5, 10, 10, 5, 1));
    }
}
