//! # delta — Delta Inheritance
//!
//! SDF chỉ lưu đầy đủ tại L4 (node đại diện).
//! L5+ chỉ lưu delta — sự khác biệt so với node cha.
//!
//! ```text
//! L2: ~21 roots    — Shape, Life, Place...
//! L3: ~55 branches — Animal, Planet, Vehicle...
//! L4: ~144 sub     — Dog, Planet         ← SDF đầy đủ ở đây
//! L5+: leaves      — Earth, Labrador...  ← delta only
//!
//! L4 "Planet"   → Sphere r=1.0 (chuẩn hóa)     ~32B
//! L5 "Earth"    → delta: r×6371, tilt=23.5°      ~20B
//! L5 "Mars"     → delta: r×3389, redness_mat     ~20B
//! L6 "Earth@2024" → delta of delta: terrain_bump ~15B
//! ```
//!
//! Tiết kiệm: 99.9% giống nhau → chỉ lưu 0.1% khác biệt.
//! Khi render: walk_up → L4 SDF base → apply delta chain.

extern crate alloc;
use alloc::vec::Vec;
use crate::sdf::{SdfKind, SdfParams, Vec3, sdf};

// ─────────────────────────────────────────────────────────────────────────────
// SdfDelta — sự khác biệt so với node cha
// ─────────────────────────────────────────────────────────────────────────────

/// Sự khác biệt so với SDF cha — compact, chỉ lưu những gì thay đổi.
///
/// None = giữ nguyên từ cha. Some = override.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub struct SdfDelta {
    /// Đổi loại hình học (hiếm)
    pub kind:    Option<SdfKind>,
    /// Scale toàn bộ (>1 = lớn hơn)
    pub scale:   Option<f32>,
    /// Dịch chuyển tâm
    pub offset:  Option<Vec3>,
    /// Thay đổi r (additive)
    pub r_delta: Option<f32>,
    /// Thay đổi h (additive)
    pub h_delta: Option<f32>,
}

impl SdfDelta {
    /// Không thay đổi gì.
    pub const IDENTITY: Self = Self {
        kind: None, scale: None, offset: None, r_delta: None, h_delta: None,
    };

    pub fn scale(factor: f32) -> Self {
        Self { scale: Some(factor), ..Self::IDENTITY }
    }

    pub fn translate(offset: Vec3) -> Self {
        Self { offset: Some(offset), ..Self::IDENTITY }
    }

    pub fn r_add(dr: f32) -> Self {
        Self { r_delta: Some(dr), ..Self::IDENTITY }
    }

    pub fn is_identity(&self) -> bool {
        self.kind.is_none() && self.scale.is_none() &&
        self.offset.is_none() && self.r_delta.is_none() && self.h_delta.is_none()
    }

    /// Bytes khi encode — compact.
    pub fn encoded_bytes(&self) -> usize {
        1                                                  // flags byte
        + self.kind.map_or(0, |_| 1)                      // kind: 1B
        + self.scale.map_or(0, |_| 4)                     // scale: f32
        + self.offset.map_or(0, |_| 12)                   // Vec3: 3×f32
        + self.r_delta.map_or(0, |_| 4)                   // r_delta: f32
        + self.h_delta.map_or(0, |_| 4)                   // h_delta: f32
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DeltaChain — path từ L4 xuống node hiện tại
// ─────────────────────────────────────────────────────────────────────────────

/// Chuỗi deltas để resolve SDF của node con.
///
/// Xây bằng cách walk_up từ node → đến L4 ancestor → thu thập deltas.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct DeltaChain {
    /// L4 base — SDF đầy đủ
    pub base_kind:   SdfKind,
    pub base_params: SdfParams,
    /// Deltas theo thứ tự L5, L6, ...
    pub deltas:      Vec<SdfDelta>,
}

impl DeltaChain {
    /// Tạo từ L4 base SDF.
    pub fn from_base(kind: SdfKind, params: SdfParams) -> Self {
        Self { base_kind: kind, base_params: params, deltas: Vec::new() }
    }

    /// Thêm delta (bỏ qua identity).
    pub fn push(&mut self, d: SdfDelta) {
        if !d.is_identity() { self.deltas.push(d); }
    }

    /// Độ sâu: 0 = chỉ L4, 1 = L5, 2 = L6...
    pub fn depth(&self) -> usize { self.deltas.len() }

    /// Tổng bytes lưu trữ.
    ///
    /// So sánh với `full_storage()` để thấy tiết kiệm.
    pub fn delta_storage(&self) -> usize {
        // L4 full (32B) + mỗi delta
        32 + self.deltas.iter().map(|d| d.encoded_bytes()).sum::<usize>()
    }

    /// Bytes nếu lưu full SDF tại mỗi layer.
    pub fn full_storage(&self) -> usize {
        32 * (1 + self.deltas.len()) // L4 + mỗi child full
    }

    /// % tiết kiệm so với full storage.
    pub fn savings_pct(&self) -> f32 {
        if self.deltas.is_empty() { return 0.0; }
        let full  = self.full_storage() as f32;
        let delta = self.delta_storage() as f32;
        (full - delta) / full * 100.0
    }

    /// Resolve: apply tất cả deltas → (SdfKind, SdfParams) cuối.
    pub fn resolve(&self) -> (SdfKind, SdfParams) {
        let mut kind   = self.base_kind;
        let mut params = self.base_params.clone();

        for d in &self.deltas {
            if let Some(k) = d.kind  { kind = k; }
            if let Some(s) = d.scale {
                params.r = (params.r * s).max(0.001);
                params.h = (params.h * s).max(0.001);
            }
            if let Some(dr) = d.r_delta { params.r = (params.r + dr).max(0.001); }
            if let Some(dh) = d.h_delta { params.h = (params.h + dh).max(0.001); }
        }

        (kind, params)
    }

    /// Evaluate SDF tại p — apply offset deltas, rồi evaluate.
    pub fn evaluate(&self, p: Vec3) -> f32 {
        let mut offset = Vec3::new(0.0, 0.0, 0.0);
        for d in &self.deltas {
            if let Some(off) = d.offset { offset = offset.add(off); }
        }
        let (kind, params) = self.resolve();
        sdf(kind, p.sub(offset), &params)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// NodeSdf — lưu trữ SDF trong node
// ─────────────────────────────────────────────────────────────────────────────

/// Cách lưu SDF trong một node của KnowledgeTree.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum NodeSdf {
    /// L0-L4: SDF đầy đủ — tự resolve được
    Full { kind: SdfKind, params: SdfParams },
    /// L5+: chỉ delta — cần walk_up để resolve
    Delta(SdfDelta),
    /// Node không có hình dạng (text/emotion only)
    None,
}

impl NodeSdf {
    /// Bytes lưu trữ.
    pub fn storage_bytes(&self) -> usize {
        match self {
            NodeSdf::Full  { .. }  => 32,
            NodeSdf::Delta(d)      => d.encoded_bytes(),
            NodeSdf::None          => 0,
        }
    }

    /// Layer tối thiểu nên dùng loại này.
    pub fn recommended_from_layer(layer: u8) -> &'static str {
        match layer {
            0..=4 => "Full (L0-L4: SDF đầy đủ)",
            5..=6 => "Delta (L5-L6: delta ~20B)",
            _     => "Delta (L7+: delta of delta ~10B)",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Ví dụ thực tế: Planet hierarchy
// ─────────────────────────────────────────────────────────────────────────────

/// Tạo DeltaChain cho Trái Đất (L5, con của Planet L4).
///
/// Planet (L4): Sphere r=1.0 (chuẩn hóa)
/// Earth  (L5): scale=6371 (km), tilt offset nhỏ
pub fn earth_delta_chain() -> DeltaChain {
    let mut chain = DeltaChain::from_base(
        SdfKind::Sphere,
        SdfParams { r: 1.0, h: 0.0, r2: 0.0, b: Vec3::new(1.0, 1.0, 1.0) }, // Planet L4 base
    );
    // Earth: lớn hơn nhiều (normalized units)
    chain.push(SdfDelta::scale(6371.0));
    chain
}

/// Tạo DeltaChain cho Mars (L5, con của Planet L4).
pub fn mars_delta_chain() -> DeltaChain {
    let mut chain = DeltaChain::from_base(
        SdfKind::Sphere,
        SdfParams { r: 1.0, h: 0.0, r2: 0.0, b: Vec3::new(1.0, 1.0, 1.0) },
    );
    chain.push(SdfDelta::scale(3389.0)); // Mars nhỏ hơn Earth
    chain
}

/// Tạo DeltaChain cho Labrador (L5, con của Dog L4).
pub fn labrador_delta_chain() -> DeltaChain {
    let mut chain = DeltaChain::from_base(
        SdfKind::Capsule,
        SdfParams { r: 0.3, h: 0.6, r2: 0.0, b: Vec3::new(0.3, 0.3, 0.3) }, // Dog L4 base
    );
    chain.push(SdfDelta { scale: Some(1.3), r_delta: Some(0.05), ..SdfDelta::IDENTITY });
    chain
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn planet_base() -> DeltaChain {
        DeltaChain::from_base(
            SdfKind::Sphere,
            SdfParams { r: 1.0, h: 0.0, r2: 0.0, b: Vec3::new(1.0, 1.0, 1.0) },
        )
    }

    // ── SdfDelta ──────────────────────────────────────────────────────────────

    #[test]
    fn identity_changes_nothing() {
        assert!(SdfDelta::IDENTITY.is_identity());
        assert_eq!(SdfDelta::IDENTITY.encoded_bytes(), 1);
    }

    #[test]
    fn delta_smaller_than_full() {
        let d = SdfDelta::scale(2.0);
        assert!(d.encoded_bytes() < 32, "Delta {} < full 32B", d.encoded_bytes());
    }

    #[test]
    fn delta_with_all_fields_still_compact() {
        let d = SdfDelta {
            kind: Some(SdfKind::Sphere), scale: Some(1.5),
            offset: Some(Vec3::new(1.0, 0.0, 0.0)),
            r_delta: Some(0.1), h_delta: Some(0.2),
        };
        // 1 + 1 + 4 + 12 + 4 + 4 = 26B vs 32B full
        assert_eq!(d.encoded_bytes(), 26);
        assert!(d.encoded_bytes() < 32);
    }

    // ── DeltaChain ────────────────────────────────────────────────────────────

    #[test]
    fn no_delta_resolves_to_base() {
        let chain = planet_base();
        let (kind, params) = chain.resolve();
        assert_eq!(kind, SdfKind::Sphere);
        assert!((params.r - 1.0).abs() < 1e-5);
    }

    #[test]
    fn earth_larger_than_planet_base() {
        let chain = earth_delta_chain();
        let (_, params) = chain.resolve();
        assert!(params.r > 100.0, "Earth scale: {}", params.r);
    }

    #[test]
    fn earth_larger_than_mars() {
        let earth = earth_delta_chain();
        let mars  = mars_delta_chain();
        let (_, ep) = earth.resolve();
        let (_, mp) = mars.resolve();
        assert!(ep.r > mp.r, "Earth {} > Mars {}", ep.r, mp.r);
    }

    #[test]
    fn depth_tracking() {
        let mut chain = planet_base();
        assert_eq!(chain.depth(), 0);
        chain.push(SdfDelta::scale(6371.0));
        assert_eq!(chain.depth(), 1);
        chain.push(SdfDelta::translate(Vec3::new(0.1, 0.0, 0.0)));
        assert_eq!(chain.depth(), 2);
    }

    #[test]
    fn delta_storage_less_than_full() {
        let mut chain = planet_base();
        chain.push(SdfDelta::scale(6371.0));    // Earth L5
        chain.push(SdfDelta::r_add(0.5));       // Earth@2024 L6

        let delta = chain.delta_storage();
        let full  = chain.full_storage();
        assert!(delta < full, "Delta {} < Full {}", delta, full);
        assert!(chain.savings_pct() > 0.0, "Tiết kiệm: {:.1}%", chain.savings_pct());
    }

    #[test]
    fn l4_base_is_full_l5_is_delta() {
        assert_eq!(NodeSdf::Full { kind: SdfKind::Sphere, params: SdfParams { r: 1.0, h: 0.0, r2: 0.0, b: Vec3::new(1.0, 1.0, 1.0) } }.storage_bytes(), 32);
        assert!(NodeSdf::Delta(SdfDelta::scale(2.0)).storage_bytes() < 32);
        assert_eq!(NodeSdf::None.storage_bytes(), 0);
    }

    #[test]
    fn planet_hierarchy_savings() {
        // Planet(L4) + Earth(L5) + Mars(L5) + Venus(L5)
        let mut chain = planet_base();
        chain.push(SdfDelta::scale(6371.0));
        chain.push(SdfDelta::scale(3389.0));
        chain.push(SdfDelta::scale(6051.0));

        let savings = chain.savings_pct();
        // 4×32B = 128B full vs 32 + 3×~5B = ~47B delta → >60% savings
        assert!(savings > 50.0, "Tiết kiệm > 50%: {:.1}%", savings);
    }

    #[test]
    fn evaluate_sphere_at_surface() {
        let chain = earth_delta_chain();
        let p = Vec3::new(6371.0, 0.0, 0.0); // điểm trên bề mặt
        let d = chain.evaluate(p);
        // SDF tại bề mặt ≈ 0
        assert!(d.abs() < 1.0, "Bề mặt Trái Đất SDF ≈ 0: {}", d);
    }

    #[test]
    fn evaluate_inside_vs_outside() {
        let chain = earth_delta_chain();
        let inside  = chain.evaluate(Vec3::new(0.0, 0.0, 0.0));   // tâm
        let outside = chain.evaluate(Vec3::new(10000.0, 0.0, 0.0)); // ngoài
        assert!(inside  < 0.0, "Bên trong < 0: {}", inside);
        assert!(outside > 0.0, "Bên ngoài > 0: {}", outside);
    }

    #[test]
    fn labrador_bigger_than_base_dog() {
        let base_dog = DeltaChain::from_base(
            SdfKind::Capsule,
            SdfParams { r: 0.3, h: 0.6, r2: 0.0, b: Vec3::new(0.3, 0.3, 0.3) },
        );
        let labrador = labrador_delta_chain();
        let (_, bp) = base_dog.resolve();
        let (_, lp) = labrador.resolve();
        assert!(lp.r > bp.r, "Labrador r {} > Dog r {}", lp.r, bp.r);
    }

    #[test]
    fn recommended_layer_labels() {
        assert!(NodeSdf::recommended_from_layer(4).contains("Full"));
        assert!(NodeSdf::recommended_from_layer(5).contains("Delta"));
        assert!(NodeSdf::recommended_from_layer(7).contains("delta of delta"));
    }
}
