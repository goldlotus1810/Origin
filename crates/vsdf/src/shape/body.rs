//! # body — NodeBody: SDF + Spline binding cho mỗi Node
//!
//! Mỗi Node (chain_hash) có thể mang 1 "body":
//!   - SDF shape (hữu hình): hình dạng, kích thước, vật liệu
//!   - Spline curves (vô hình): intensity, force, temperature, frequency, emotion
//!
//! Cách học:
//!   "lửa trông như quả cầu sáng" → ghi SDF=Sphere, emission=high
//!   "lửa nóng"                   → ghi Spline.temperature = high curve
//!   "lửa lay động"               → ghi Spline.force = oscillate curve
//!   "lửa làm tôi ấm"            → ghi Spline.emotion.valence = positive curve
//!
//! Mỗi lần học = cập nhật 1/5 chiều trong chain:
//!   Shape    → SDF primitive + params
//!   Relation → cách kết nối (Silk edges, không ở đây)
//!   Valence  → Spline.emotion.valence
//!   Arousal  → Spline.emotion.arousal
//!   Time     → Spline temporal behavior (tần số, envelope)
//!
//! QT9: Append-only — mỗi learn event thêm version mới, không xóa cũ.

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use crate::scene::{Material, Transform};
use crate::sdf::{SdfKind, SdfParams};
use crate::spline::VectorSpline;

// ─────────────────────────────────────────────────────────────────────────────
// SplineSet — 5 chiều vô hình của 1 node
// ─────────────────────────────────────────────────────────────────────────────

/// 5 chiều vô hình — mỗi chiều là 1 VectorSpline.
///
/// QT6: "HAI CHIỀU TỒN TẠI" — SDF = hữu hình, Spline = vô hình.
#[derive(Debug, Clone)]
pub struct SplineSet {
    /// Intensity (ánh sáng) — t=0..1 map qua thời gian
    pub intensity: VectorSpline,
    /// Force (gió, lực) — hướng + cường độ theo t
    pub force: VectorSpline,
    /// Temperature (nhiệt) — nóng/lạnh theo t
    pub temperature: VectorSpline,
    /// Frequency (âm thanh) — tần số/nhịp theo t
    pub frequency: VectorSpline,
    /// Emotion valence — cảm xúc theo t (dùng cho affect rendering)
    pub emotion_v: VectorSpline,
    /// Emotion arousal — cường độ cảm xúc theo t
    pub emotion_a: VectorSpline,
}

impl SplineSet {
    /// Empty — tất cả flat(0.0).
    pub fn empty() -> Self {
        Self {
            intensity: VectorSpline::new(),
            force: VectorSpline::new(),
            temperature: VectorSpline::new(),
            frequency: VectorSpline::new(),
            emotion_v: VectorSpline::new(),
            emotion_a: VectorSpline::new(),
        }
    }

    /// Evaluate tất cả splines tại thời điểm t.
    pub fn evaluate(&self, t: f32) -> SplineSnapshot {
        SplineSnapshot {
            intensity: self.intensity.evaluate(t),
            force: self.force.evaluate(t),
            temperature: self.temperature.evaluate(t),
            frequency: self.frequency.evaluate(t),
            emotion_v: self.emotion_v.evaluate(t),
            emotion_a: self.emotion_a.evaluate(t),
        }
    }

    /// Có bất kỳ spline nào non-empty không?
    pub fn has_data(&self) -> bool {
        !self.intensity.is_empty()
            || !self.force.is_empty()
            || !self.temperature.is_empty()
            || !self.frequency.is_empty()
            || !self.emotion_v.is_empty()
            || !self.emotion_a.is_empty()
    }
}

/// Snapshot — giá trị tại 1 thời điểm t.
#[derive(Debug, Clone, Copy)]
pub struct SplineSnapshot {
    /// Ánh sáng 0..1
    pub intensity: f32,
    /// Lực 0..1
    pub force: f32,
    /// Nhiệt 0..1
    pub temperature: f32,
    /// Tần số 0..1
    pub frequency: f32,
    /// Valence -1..+1
    pub emotion_v: f32,
    /// Arousal 0..1
    pub emotion_a: f32,
}

// ─────────────────────────────────────────────────────────────────────────────
// NodeBody — SDF + Spline cho 1 node
// ─────────────────────────────────────────────────────────────────────────────

/// Dimension nào đang được cập nhật.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyDimension {
    /// Shape dimension → cập nhật SDF kind/params/material
    Shape,
    /// Valence dimension → cập nhật emotion_v spline
    Valence,
    /// Arousal dimension → cập nhật emotion_a spline
    Arousal,
    /// Time dimension → cập nhật temporal behavior (intensity, force, temp, freq)
    Time,
    /// Specific immaterial dimension
    Intensity,
    /// Force/wind
    Force,
    /// Temperature
    Temperature,
    /// Frequency/sound
    Frequency,
}

/// Một node's "body" — cách nó tồn tại trong không gian vật lý + vô hình.
///
/// ```text
/// MolecularChain [S][R][V][A][T] ← 5 bytes tĩnh (DNA)
///       ↕ binding
/// NodeBody {
///   sdf_kind + params + material   ← Shape dimension (hữu hình)
///   splines: SplineSet              ← V/A/Time dimensions (vô hình)
/// }
/// ```
///
/// Mỗi lần "học" = cập nhật 1 dimension:
///   "lửa tròn"    → sdf_kind = Sphere (Shape)
///   "lửa nóng"    → temperature spline = high (Time/immaterial)
///   "lửa vui"     → emotion_v spline = positive (Valence)
#[derive(Debug, Clone)]
pub struct NodeBody {
    /// Chain hash — identity of this node
    pub chain_hash: u64,
    /// SDF primitive (None = chưa có hình dạng)
    pub sdf_kind: Option<SdfKind>,
    /// SDF parameters (radius, height, etc.)
    pub sdf_params: SdfParams,
    /// Visual material (color, roughness, emission)
    pub material: Material,
    /// World transform (position, scale, rotation)
    pub transform: Transform,
    /// Spline curves — immaterial dimensions
    pub splines: SplineSet,
    /// Version counter — mỗi learn event tăng 1 (QT9 append-only)
    pub version: u32,
}

impl NodeBody {
    /// Tạo body mới cho 1 chain_hash.
    pub fn new(chain_hash: u64) -> Self {
        Self {
            chain_hash,
            sdf_kind: None,
            sdf_params: SdfParams::default(),
            material: Material::DEFAULT,
            transform: Transform::IDENTITY,
            splines: SplineSet::empty(),
            version: 0,
        }
    }

    /// Learn SDF shape — "cái này trông như X".
    pub fn learn_shape(&mut self, kind: SdfKind, params: SdfParams) {
        self.sdf_kind = Some(kind);
        self.sdf_params = params;
        self.version += 1;
    }

    /// Learn material — "cái này màu X, sáng Y".
    pub fn learn_material(&mut self, material: Material) {
        self.material = material;
        self.version += 1;
    }

    /// Learn intensity spline — "cái này sáng thế này theo thời gian".
    pub fn learn_intensity(&mut self, spline: VectorSpline) {
        self.splines.intensity = spline;
        self.version += 1;
    }

    /// Learn force spline — "gió/lực thế này".
    pub fn learn_force(&mut self, spline: VectorSpline) {
        self.splines.force = spline;
        self.version += 1;
    }

    /// Learn temperature spline — "nóng/lạnh thế này".
    pub fn learn_temperature(&mut self, spline: VectorSpline) {
        self.splines.temperature = spline;
        self.version += 1;
    }

    /// Learn frequency spline — "âm thanh thế này".
    pub fn learn_frequency(&mut self, spline: VectorSpline) {
        self.splines.frequency = spline;
        self.version += 1;
    }

    /// Learn emotion valence spline — "cảm xúc thế này".
    pub fn learn_emotion_v(&mut self, spline: VectorSpline) {
        self.splines.emotion_v = spline;
        self.version += 1;
    }

    /// Learn emotion arousal spline — "cường độ cảm xúc thế này".
    pub fn learn_emotion_a(&mut self, spline: VectorSpline) {
        self.splines.emotion_a = spline;
        self.version += 1;
    }

    /// Có hình dạng chưa?
    pub fn has_shape(&self) -> bool {
        self.sdf_kind.is_some()
    }

    /// Có bất kỳ data nào chưa?
    pub fn has_data(&self) -> bool {
        self.has_shape() || self.splines.has_data()
    }

    /// Evaluate node tại thời điểm t → snapshot tất cả splines.
    pub fn evaluate(&self, t: f32) -> SplineSnapshot {
        self.splines.evaluate(t)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BodyStore — chain_hash → NodeBody registry
// ─────────────────────────────────────────────────────────────────────────────

/// Registry: map chain_hash → NodeBody.
///
/// Khi learning pipeline dạy "lửa hình cầu" hoặc "nước mát":
///   1. Lookup chain_hash trong BodyStore
///   2. Nếu chưa có → tạo mới
///   3. Ghi dimension tương ứng
///   4. Version++
///
/// BodyStore nằm trong vsdf crate (no_std compatible).
/// Runtime/agents gọi qua reference.
pub struct BodyStore {
    /// chain_hash → NodeBody
    bodies: BTreeMap<u64, NodeBody>,
    /// Access counter per hash — LFU eviction khi RAM pressure
    access_count: BTreeMap<u64, u32>,
    /// Maximum number of bodies to keep in RAM (0 = unlimited)
    max_bodies: usize,
}

impl BodyStore {
    /// Tạo empty store — unlimited bodies.
    pub fn new() -> Self {
        Self {
            bodies: BTreeMap::new(),
            access_count: BTreeMap::new(),
            max_bodies: 0, // unlimited
        }
    }

    /// Tạo store với RAM limit.
    ///
    /// Khi vượt `max_bodies`, evict body ít dùng nhất (LFU).
    /// Công thức giảm tải RAM: chỉ giữ active bodies trong memory.
    pub fn with_capacity(max_bodies: usize) -> Self {
        Self {
            bodies: BTreeMap::new(),
            access_count: BTreeMap::new(),
            max_bodies,
        }
    }

    /// Get or create NodeBody for a chain_hash.
    pub fn get_or_create(&mut self, chain_hash: u64) -> &mut NodeBody {
        *self.access_count.entry(chain_hash).or_insert(0) += 1;
        self.bodies
            .entry(chain_hash)
            .or_insert_with(|| NodeBody::new(chain_hash))
    }

    /// Lookup (read-only) — tăng access counter.
    pub fn get(&self, chain_hash: u64) -> Option<&NodeBody> {
        // Note: không tăng counter ở get() immutable — chỉ get_mut/get_or_create tăng
        self.bodies.get(&chain_hash)
    }

    /// Lookup (mutable) — tăng access counter.
    pub fn get_mut(&mut self, chain_hash: u64) -> Option<&mut NodeBody> {
        if self.bodies.contains_key(&chain_hash) {
            *self.access_count.entry(chain_hash).or_insert(0) += 1;
        }
        self.bodies.get_mut(&chain_hash)
    }

    /// Evict least-frequently-used bodies khi vượt capacity.
    ///
    /// Giảm tải RAM: chỉ giữ `max_bodies` active nhất.
    /// Bodies bị evict có thể tái tạo từ Molecule bytes (body_from_molecule).
    /// Trả về số bodies đã evict.
    pub fn evict_lfu(&mut self) -> usize {
        if self.max_bodies == 0 || self.bodies.len() <= self.max_bodies {
            return 0;
        }

        let to_evict = self.bodies.len() - self.max_bodies;

        // Collect (hash, access_count) → sort → remove least used
        let mut candidates: Vec<(u64, u32)> = self
            .bodies
            .keys()
            .map(|&h| (h, self.access_count.get(&h).copied().unwrap_or(0)))
            .collect();
        candidates.sort_by_key(|&(_, count)| count);

        let mut evicted = 0;
        for (hash, _) in candidates.into_iter().take(to_evict) {
            self.bodies.remove(&hash);
            self.access_count.remove(&hash);
            evicted += 1;
        }

        evicted
    }

    /// Learn shape cho 1 node.
    pub fn learn_shape(&mut self, chain_hash: u64, kind: SdfKind, params: SdfParams) {
        self.get_or_create(chain_hash).learn_shape(kind, params);
    }

    /// Learn material cho 1 node.
    pub fn learn_material(&mut self, chain_hash: u64, material: Material) {
        self.get_or_create(chain_hash).learn_material(material);
    }

    /// Learn spline cho 1 dimension.
    pub fn learn_spline(
        &mut self,
        chain_hash: u64,
        dim: BodyDimension,
        spline: VectorSpline,
    ) {
        let body = self.get_or_create(chain_hash);
        match dim {
            BodyDimension::Intensity => body.learn_intensity(spline),
            BodyDimension::Force => body.learn_force(spline),
            BodyDimension::Temperature => body.learn_temperature(spline),
            BodyDimension::Frequency => body.learn_frequency(spline),
            BodyDimension::Valence => body.learn_emotion_v(spline),
            BodyDimension::Arousal => body.learn_emotion_a(spline),
            BodyDimension::Shape | BodyDimension::Time => {
                // Shape dùng learn_shape(), Time = meta
            }
        }
    }

    /// Total bodies stored.
    pub fn len(&self) -> usize {
        self.bodies.len()
    }

    /// Empty?
    pub fn is_empty(&self) -> bool {
        self.bodies.is_empty()
    }

    /// Tất cả bodies có SDF shape.
    pub fn bodies_with_shape(&self) -> impl Iterator<Item = (&u64, &NodeBody)> {
        self.bodies.iter().filter(|(_, b)| b.has_shape())
    }

    /// Tất cả chain_hashes.
    pub fn all_hashes(&self) -> impl Iterator<Item = &u64> {
        self.bodies.keys()
    }

    /// RAM estimate in bytes.
    pub fn ram_usage(&self) -> usize {
        // Rough: per body = ~200 bytes (struct) + spline segments
        // Per access_count entry = ~12 bytes (u64 + u32)
        let access_overhead = self.access_count.len() * 12;
        self.bodies.len() * 200
            + self
                .bodies
                .values()
                .map(|b| {
                    (b.splines.intensity.len()
                        + b.splines.force.len()
                        + b.splines.temperature.len()
                        + b.splines.frequency.len()
                        + b.splines.emotion_v.len()
                        + b.splines.emotion_a.len())
                        * 16 // 4 f32 per segment
                })
                .sum::<usize>()
            + access_overhead
    }

    /// Current capacity limit (0 = unlimited).
    pub fn max_bodies(&self) -> usize {
        self.max_bodies
    }

    /// Set capacity limit. Immediately evicts if over limit.
    pub fn set_max_bodies(&mut self, max: usize) -> usize {
        self.max_bodies = max;
        self.evict_lfu()
    }
}

impl Default for BodyStore {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Seed helpers — tạo body mặc định từ UCD shape byte
// ─────────────────────────────────────────────────────────────────────────────

/// Map ShapeBase byte → SdfKind mặc định.
///
/// ShapeBase (8 categories) → SdfKind (18 primitives):
///   Sphere(0x01) → SdfKind::Sphere
///   Plane(0x02)  → SdfKind::Plane
///   Box(0x03)    → SdfKind::Box
///   Cone(0x04)   → SdfKind::Cone
///   Torus(0x05)  → SdfKind::Torus
///   Union(0x06)  → SdfKind::Capsule  (closest visual)
///   Intersect(0x07) → SdfKind::Cylinder
///   Subtract(0x08)  → SdfKind::CutSphere
pub fn shape_base_to_sdf(shape_base: u8) -> SdfKind {
    match shape_base {
        0x01 => SdfKind::Sphere,
        0x02 => SdfKind::Plane,
        0x03 => SdfKind::Box,
        0x04 => SdfKind::Cone,
        0x05 => SdfKind::Torus,
        0x06 => SdfKind::Capsule,
        0x07 => SdfKind::Cylinder,
        0x08 => SdfKind::CutSphere,
        _ => SdfKind::Sphere,
    }
}

/// Tạo NodeBody hoàn chỉnh từ Molecule 5 bytes — CÔNG THỨC, không phải dữ liệu.
///
/// Mọi thuộc tính được TÍNH từ 5 bytes, không lưu trữ:
///
/// ```text
/// [Shape]    → SdfKind + SdfParams (r, h, r2 từ sub_index)
/// [Relation] → alpha, roughness (relation type ảnh hưởng vật chất)
/// [Valence]  → color temperature (warm/cool), emotion_v spline
/// [Arousal]  → emission, scale, force, emotion_a spline
/// [Time]     → intensity envelope, temperature curve, frequency
/// ```
///
/// Deterministic: cùng 5 bytes → cùng NodeBody. Có thể evict khỏi RAM
/// rồi tái tạo bất kỳ lúc nào.
pub fn body_from_molecule(
    chain_hash: u64,
    shape: u8,
    valence: u8,
    arousal: u8,
    time: u8,
) -> NodeBody {
    body_from_molecule_full(chain_hash, shape, 0x01, valence, arousal, time)
}

/// Full 5D → Body computation. Dùng tất cả 5 chiều.
///
/// Đây là "công thức chính" — mỗi chiều đóng góp vào thuộc tính vật lý:
///   Shape    → hình dạng + kích thước
///   Relation → tính chất bề mặt (alpha, roughness)
///   Valence  → màu sắc + cảm xúc
///   Arousal  → năng lượng (emission, scale, force)
///   Time     → dynamics (intensity, temperature, frequency envelopes)
pub fn body_from_molecule_full(
    chain_hash: u64,
    shape: u8,
    relation: u8,
    valence: u8,
    arousal: u8,
    time: u8,
) -> NodeBody {
    // ── Extract base + sub_index from hierarchical encoding ──────────
    let shape_base = if shape == 0 { 1 } else { ((shape - 1) % 8) + 1 };
    let shape_sub = if shape == 0 { 0 } else { (shape - 1) / 8 };
    let relation_base = if relation == 0 { 1 } else { ((relation - 1) % 8) + 1 };
    let time_base = if time == 0 { 3 } else { ((time - 1) % 5) + 1 };
    let time_sub = if time == 0 { 0 } else { (time - 1) / 5 };

    // ── Normalized values ────────────────────────────────────────────
    let v_norm = (valence as f32 - 128.0) / 128.0; // -1.0 .. +1.0
    let a_norm = arousal as f32 / 255.0; // 0.0 .. 1.0

    // ── 1. Shape → SdfKind + SdfParams ──────────────────────────────
    // sub_index scales the default size: more variation = smaller details
    let size_factor = 0.3 + 0.7 / (1.0 + shape_sub as f32 * 0.3); // 0.3..1.0
    let sdf_kind = shape_base_to_sdf(shape_base);
    let sdf_params = match sdf_kind {
        SdfKind::Sphere | SdfKind::Ellipsoid | SdfKind::Octahedron => {
            SdfParams::sphere(size_factor * 0.5)
        }
        SdfKind::Box | SdfKind::RoundBox => {
            let s = size_factor * 0.4;
            SdfParams::sdf_box(s, s * 0.8, s) // slightly flattened
        }
        SdfKind::Cone | SdfKind::Pyramid => {
            SdfParams::cone(size_factor * 0.3, size_factor * 0.6)
        }
        SdfKind::Torus | SdfKind::Link => {
            SdfParams::torus(size_factor * 0.4, size_factor * 0.15)
        }
        SdfKind::Capsule | SdfKind::Cylinder | SdfKind::HexPrism | SdfKind::TriPrism => {
            SdfParams::capsule(size_factor * 0.2, size_factor * 0.5)
        }
        SdfKind::CutSphere | SdfKind::CutHollow | SdfKind::DeathStar => {
            SdfParams::sphere(size_factor * 0.45)
        }
        SdfKind::Plane | SdfKind::SolidAngle => SdfParams::default(),
    };

    // ── 2. Relation → surface properties ────────────────────────────
    // Relation type influences how the node "connects" to the world visually
    let (rel_alpha, rel_roughness) = match relation_base {
        1 => (1.0, 0.5),  // Member — solid, medium rough
        2 => (0.9, 0.3),  // Subset — slightly transparent, smoother
        3 => (1.0, 0.1),  // Equivalent — solid, very smooth (mirror-like)
        4 => (0.7, 0.8),  // Orthogonal — semi-transparent, rough
        5 => (1.0, 0.4),  // Compose — solid, moderate
        6 => (0.95, 0.2), // Causes — near-solid, smooth (directional energy)
        7 => (0.85, 0.6), // Approximate — slightly hazy
        8 => (0.8, 0.5),  // Inverse — semi-transparent
        _ => (1.0, 0.5),
    };

    // ── 3. Valence → color temperature ──────────────────────────────
    // Positive = warm (red/orange/yellow), Negative = cool (blue/purple)
    // Neutral = natural gray
    let (mat_r, mat_g, mat_b) = if v_norm > 0.3 {
        // Warm: orange → yellow as v increases
        let t = (v_norm - 0.3) / 0.7; // 0..1
        (0.9 + t * 0.1, 0.4 + t * 0.5, 0.1 + t * 0.1)
    } else if v_norm < -0.3 {
        // Cool: blue → purple as v decreases
        let t = (-v_norm - 0.3) / 0.7; // 0..1
        (0.15 + t * 0.2, 0.3 - t * 0.1, 0.6 + t * 0.35)
    } else {
        // Neutral zone: warm gray with subtle tint
        let t = (v_norm + 0.3) / 0.6; // 0..1 within [-0.3, 0.3]
        (0.5 + t * 0.2, 0.5 + t * 0.1, 0.6 - t * 0.15)
    };

    // ── 4. Arousal → energy properties ──────────────────────────────
    // High arousal = glowing, larger, more forceful
    let emission = a_norm * a_norm * 0.8; // quadratic: low arousal barely glows
    let scale = 0.7 + a_norm * 0.6; // 0.7..1.3

    // ── 5. Time → all 4 temporal splines ────────────────────────────
    // Time base determines the envelope shape for intensity, temperature,
    // force, and frequency. time_sub adds phase variation.
    let phase = time_sub as f32 * 0.15; // 0.0..~0.6 phase shift

    let intensity_spline = match time_base {
        1 => VectorSpline::flat(0.4 + a_norm * 0.3), // Static: level depends on arousal
        2 => VectorSpline::linear(0.2 + phase, 0.6 + a_norm * 0.2), // Slow rise
        3 => VectorSpline::flat(0.5 + a_norm * 0.2),                // Medium steady
        4 => VectorSpline::linear(0.1 + phase, 0.8 + a_norm * 0.2), // Fast rise
        5 => VectorSpline::linear(phase, 1.0),                      // Instant
        _ => VectorSpline::flat(0.5),
    };

    // Temperature: arousal heats things up, time modulates
    let temp_base = a_norm * 0.6; // hot things have high arousal
    let temperature_spline = match time_base {
        1 => VectorSpline::flat(temp_base),
        2 => VectorSpline::linear(temp_base * 0.5, temp_base),
        3 => VectorSpline::flat(temp_base * 0.8),
        4 => VectorSpline::linear(temp_base * 0.3, temp_base * 1.2),
        5 => VectorSpline::linear(0.0, temp_base * 1.5),
        _ => VectorSpline::flat(temp_base),
    };

    // Force: arousal = energy, time = how it's released
    let force_base = a_norm * 0.5;
    let force_spline = match time_base {
        1 => VectorSpline::flat(force_base * 0.3), // Static = minimal force
        2 => VectorSpline::linear(0.0, force_base), // Slow build
        3 => VectorSpline::flat(force_base * 0.6),  // Steady
        4 => VectorSpline::linear(force_base * 0.2, force_base), // Fast build
        5 => VectorSpline::linear(0.0, force_base * 1.5), // Burst
        _ => VectorSpline::flat(0.0),
    };

    // Frequency: time determines rhythm, arousal determines pitch
    let freq_base = 0.2 + a_norm * 0.5;
    let frequency_spline = match time_base {
        1 => VectorSpline::flat(0.0),              // Static = no rhythm
        2 => VectorSpline::flat(freq_base * 0.3),  // Slow = low freq
        3 => VectorSpline::flat(freq_base * 0.5),  // Medium
        4 => VectorSpline::flat(freq_base * 0.8),  // Fast
        5 => VectorSpline::flat(freq_base),         // Instant = highest
        _ => VectorSpline::flat(0.0),
    };

    // ── Assemble NodeBody ────────────────────────────────────────────
    let mut body = NodeBody::new(chain_hash);
    body.sdf_kind = Some(sdf_kind);
    body.sdf_params = sdf_params;

    // Material: combine all dimensions
    body.material = Material {
        r: mat_r,
        g: mat_g,
        b: mat_b,
        alpha: rel_alpha,
        roughness: rel_roughness,
        emission,
    };

    // Transform: arousal → scale
    body.transform.scale = scale;

    // Splines: computed from time × arousal
    body.splines.intensity = intensity_spline;
    body.splines.temperature = temperature_spline;
    body.splines.force = force_spline;
    body.splines.frequency = frequency_spline;

    // Emotion splines from valence/arousal (only when non-default)
    if valence != 0x80 {
        body.splines.emotion_v = VectorSpline::flat(v_norm);
    }
    if arousal != 0x80 {
        body.splines.emotion_a = VectorSpline::flat(a_norm);
    }

    body
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spline::BezierSegment;

    #[test]
    fn node_body_new_empty() {
        let body = NodeBody::new(0x1234);
        assert_eq!(body.chain_hash, 0x1234);
        assert!(!body.has_shape());
        assert!(!body.has_data());
        assert_eq!(body.version, 0);
    }

    #[test]
    fn node_body_learn_shape() {
        let mut body = NodeBody::new(0x1234);
        body.learn_shape(SdfKind::Sphere, SdfParams::default());
        assert!(body.has_shape());
        assert_eq!(body.sdf_kind, Some(SdfKind::Sphere));
        assert_eq!(body.version, 1);
    }

    #[test]
    fn node_body_learn_spline() {
        let mut body = NodeBody::new(0x1234);
        body.learn_temperature(VectorSpline::flat(0.9));
        assert!(body.has_data());
        assert!(!body.has_shape()); // chưa có SDF
        let snap = body.evaluate(0.5);
        assert!((snap.temperature - 0.9).abs() < 1e-5);
        assert_eq!(body.version, 1);
    }

    #[test]
    fn node_body_multi_learn() {
        let mut body = NodeBody::new(0xAAAA);
        // Learn shape → version 1
        body.learn_shape(SdfKind::Sphere, SdfParams::default());
        // Learn temperature → version 2
        body.learn_temperature(VectorSpline::flat(0.8));
        // Learn emotion → version 3
        body.learn_emotion_v(VectorSpline::linear(-0.5, 0.5));
        assert_eq!(body.version, 3);
        assert!(body.has_shape());
        assert!(body.has_data());

        // Evaluate emotion at t=0 and t=1
        let snap0 = body.evaluate(0.0);
        let snap1 = body.evaluate(1.0);
        assert!(snap0.emotion_v < 0.0, "t=0 → negative valence");
        assert!(snap1.emotion_v > 0.0, "t=1 → positive valence");
    }

    #[test]
    fn body_store_get_or_create() {
        let mut store = BodyStore::new();
        assert!(store.is_empty());

        store.learn_shape(0x1111, SdfKind::Box, SdfParams::default());
        assert_eq!(store.len(), 1);

        // Same hash → same body
        store.learn_spline(0x1111, BodyDimension::Temperature, VectorSpline::flat(0.7));
        assert_eq!(store.len(), 1);

        let body = store.get(0x1111).unwrap();
        assert_eq!(body.sdf_kind, Some(SdfKind::Box));
        assert_eq!(body.version, 2); // shape + temperature
    }

    #[test]
    fn body_store_multiple_nodes() {
        let mut store = BodyStore::new();
        store.learn_shape(0x1111, SdfKind::Sphere, SdfParams::default());
        store.learn_shape(0x2222, SdfKind::Torus, SdfParams::default());
        store.learn_shape(0x3333, SdfKind::Cone, SdfParams::default());
        assert_eq!(store.len(), 3);
        assert_eq!(store.bodies_with_shape().count(), 3);
    }

    #[test]
    fn body_from_molecule_fire() {
        // 🔥 FIRE: shape=Sphere, valence=0xE0 (high positive), arousal=0xD0 (high), time=Fast
        let body = body_from_molecule(0xF1AE, 0x01, 0xE0, 0xD0, 0x04);
        assert_eq!(body.sdf_kind, Some(SdfKind::Sphere));
        // Valence positive → warm color
        assert!(body.material.r > 0.7, "Fire = warm red");
        // Arousal high → emission (quadratic)
        assert!(body.material.emission > 0.3, "Fire = glowing");
        // Arousal high → larger scale
        assert!(body.transform.scale > 1.0, "Fire = energetic, larger");
        // Time=Fast → rising intensity
        let snap0 = body.evaluate(0.0);
        let snap1 = body.evaluate(1.0);
        assert!(snap1.intensity > snap0.intensity, "Fast = rising intensity");
        // Temperature should be significant (high arousal)
        assert!(snap1.temperature > 0.3, "Fire = hot");
        // Force should build (fast time)
        assert!(snap1.force > snap0.force, "Fast = building force");
    }

    #[test]
    fn body_from_molecule_water() {
        // 💧 WATER: shape=Sphere, valence=0x80 (neutral), arousal=0x40 (low), time=Slow
        let body = body_from_molecule(0x0A7E, 0x01, 0x80, 0x40, 0x02);
        assert_eq!(body.sdf_kind, Some(SdfKind::Sphere));
        // Arousal low → low emission (quadratic → very low)
        assert!(body.material.emission < 0.1, "Water = calm, low emission");
        // Arousal low → smaller scale
        assert!(body.transform.scale < 1.0, "Water = calm, smaller");
        // Slow time → slow build temperature
        let snap0 = body.evaluate(0.0);
        let snap1 = body.evaluate(1.0);
        assert!(snap1.temperature > snap0.temperature, "Slow = warming up");
    }

    #[test]
    fn body_from_molecule_negative_valence() {
        // sad concept: valence=0x30 (negative), arousal=0x60
        let body = body_from_molecule(0xBBBB, 0x01, 0x30, 0x60, 0x03);
        // Negative valence → cool blue
        assert!(body.material.b > 0.6, "Negative = cool blue");
        assert!(body.material.r < body.material.b, "Blue dominates red");
    }

    #[test]
    fn body_from_molecule_full_5d() {
        // Test full 5D computation with relation
        // Orthogonal relation → semi-transparent, rough
        let body = body_from_molecule_full(0xABCD, 0x03, 0x04, 0xA0, 0xB0, 0x03);
        assert_eq!(body.sdf_kind, Some(SdfKind::Box));
        assert!(body.material.alpha < 0.8, "Orthogonal = semi-transparent");
        assert!(body.material.roughness > 0.7, "Orthogonal = rough");
    }

    #[test]
    fn body_from_molecule_deterministic() {
        // Same 5 bytes → identical body (core formula guarantee)
        let a = body_from_molecule_full(0x1234, 0x09, 0x03, 0xC0, 0x90, 0x04);
        let b = body_from_molecule_full(0x1234, 0x09, 0x03, 0xC0, 0x90, 0x04);
        assert_eq!(a.sdf_kind, b.sdf_kind);
        assert!((a.material.r - b.material.r).abs() < 1e-6);
        assert!((a.material.emission - b.material.emission).abs() < 1e-6);
        assert!((a.transform.scale - b.transform.scale).abs() < 1e-6);
        let sa = a.evaluate(0.5);
        let sb = b.evaluate(0.5);
        assert!((sa.intensity - sb.intensity).abs() < 1e-6);
        assert!((sa.temperature - sb.temperature).abs() < 1e-6);
    }

    #[test]
    fn body_from_molecule_static_no_frequency() {
        // time_base=1 (Static) → frequency should be 0 (no rhythm)
        let body = body_from_molecule(0x5555, 0x01, 0x80, 0x80, 0x01);
        let snap = body.evaluate(0.5);
        assert!((snap.frequency).abs() < 1e-5, "Static = no frequency");
    }

    #[test]
    fn body_from_molecule_shape_sub_scales_size() {
        // shape=0x01 (Sphere, sub=0) vs shape=0x09 (Sphere, sub=1)
        let big = body_from_molecule(0x1111, 0x01, 0x80, 0x80, 0x03);
        let small = body_from_molecule(0x2222, 0x09, 0x80, 0x80, 0x03);
        assert!(
            big.sdf_params.r > small.sdf_params.r,
            "Higher sub_index = smaller size: {} vs {}",
            big.sdf_params.r,
            small.sdf_params.r
        );
    }

    #[test]
    fn spline_set_evaluate() {
        let mut set = SplineSet::empty();
        set.intensity = VectorSpline::flat(0.9);
        set.temperature = VectorSpline::linear(0.0, 1.0);

        let snap = set.evaluate(0.5);
        assert!((snap.intensity - 0.9).abs() < 1e-5);
        assert!((snap.temperature - 0.5).abs() < 0.05);
        assert_eq!(snap.force, 0.0); // empty → 0
    }

    #[test]
    fn shape_base_to_sdf_all() {
        assert_eq!(shape_base_to_sdf(0x01), SdfKind::Sphere);
        assert_eq!(shape_base_to_sdf(0x02), SdfKind::Plane);
        assert_eq!(shape_base_to_sdf(0x03), SdfKind::Box);
        assert_eq!(shape_base_to_sdf(0x04), SdfKind::Cone);
        assert_eq!(shape_base_to_sdf(0x05), SdfKind::Torus);
        assert_eq!(shape_base_to_sdf(0x06), SdfKind::Capsule);
        assert_eq!(shape_base_to_sdf(0x07), SdfKind::Cylinder);
        assert_eq!(shape_base_to_sdf(0x08), SdfKind::CutSphere);
        assert_eq!(shape_base_to_sdf(0xFF), SdfKind::Sphere); // fallback
    }

    #[test]
    fn body_store_ram_usage() {
        let mut store = BodyStore::new();
        for i in 0..10u64 {
            store.learn_shape(i, SdfKind::Sphere, SdfParams::default());
            store.learn_spline(i, BodyDimension::Intensity, VectorSpline::flat(0.5));
        }
        assert!(store.ram_usage() > 0);
    }

    #[test]
    fn fire_complete_body() {
        // Complete learning sequence for fire
        let mut store = BodyStore::new();
        let fire_hash = 0x1F525_u64;

        // 1. Learn shape: fire = sphere (hình cầu)
        store.learn_shape(fire_hash, SdfKind::Sphere, SdfParams { r: 0.5, ..SdfParams::default() });

        // 2. Learn material: fire = orange, glowing
        store.learn_material(fire_hash, Material {
            r: 1.0, g: 0.5, b: 0.1,
            alpha: 0.9, roughness: 0.1, emission: 0.8,
        });

        // 3. Learn temperature: fire = hot, increasing
        store.learn_spline(fire_hash, BodyDimension::Temperature, VectorSpline::linear(0.7, 1.0));

        // 4. Learn intensity: fire = bright, flickering
        let mut flicker = VectorSpline::new();
        flicker.push(BezierSegment { p0: 0.8, p1: 1.0, p2: 0.7, p3: 0.9 });
        flicker.push(BezierSegment { p0: 0.9, p1: 0.6, p2: 1.0, p3: 0.85 });
        store.learn_spline(fire_hash, BodyDimension::Intensity, flicker);

        // 5. Learn emotion: fire = exciting (positive valence, high arousal)
        store.learn_spline(fire_hash, BodyDimension::Valence, VectorSpline::flat(0.7));
        store.learn_spline(fire_hash, BodyDimension::Arousal, VectorSpline::flat(0.9));

        let body = store.get(fire_hash).unwrap();
        assert_eq!(body.version, 6); // 6 learn events
        assert_eq!(body.sdf_kind, Some(SdfKind::Sphere));
        assert!((body.material.emission - 0.8).abs() < 1e-5);

        // Evaluate at t=0.5
        let snap = body.evaluate(0.5);
        assert!(snap.temperature > 0.7, "Fire is hot");
        assert!(snap.intensity > 0.5, "Fire is bright");
        assert!(snap.emotion_v > 0.5, "Fire is positive");
        assert!(snap.emotion_a > 0.8, "Fire is exciting");
    }

    // ── BodyStore eviction tests ────────────────────────────────────────────

    #[test]
    fn body_store_with_capacity_evicts_lfu() {
        let mut store = BodyStore::with_capacity(3);

        // Create 5 bodies — will exceed capacity
        for i in 0..5u64 {
            let body = store.get_or_create(i);
            body.learn_shape(SdfKind::Sphere, SdfParams::default());
        }

        // Access body 0 and 4 multiple times → high frequency
        for _ in 0..5 {
            store.get_or_create(0);
            store.get_or_create(4);
        }

        assert_eq!(store.len(), 5);

        // Evict → keep top 3 most accessed
        let evicted = store.evict_lfu();
        assert_eq!(evicted, 2, "Should evict 2 bodies");
        assert_eq!(store.len(), 3);

        // Body 0 and 4 (high access) should survive
        assert!(store.get(0).is_some(), "High-access body 0 should survive");
        assert!(store.get(4).is_some(), "High-access body 4 should survive");
    }

    #[test]
    fn body_store_set_max_bodies() {
        let mut store = BodyStore::new(); // unlimited

        for i in 0..10u64 {
            store.get_or_create(i);
        }
        assert_eq!(store.len(), 10);

        // Set limit → immediate eviction
        let evicted = store.set_max_bodies(5);
        assert_eq!(evicted, 5);
        assert_eq!(store.len(), 5);
    }

    #[test]
    fn body_store_no_evict_under_capacity() {
        let mut store = BodyStore::with_capacity(10);
        for i in 0..5u64 {
            store.get_or_create(i);
        }
        let evicted = store.evict_lfu();
        assert_eq!(evicted, 0, "Under capacity → no eviction");
    }
}
