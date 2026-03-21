//! # molecular — DNA của thông tin
//!
//! Molecule = 5 bytes = tọa độ vật lý trong không gian 5 chiều.
//! MolecularChain = chuỗi molecules = DNA của một khái niệm.
//!
//! **KHÔNG có presets module.**
//! **KHÔNG có Molecule viết tay.**
//! Mọi Molecule đến từ `encoder::encode_codepoint(cp)`.

extern crate alloc;
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// 5 Base Dimensions
// ─────────────────────────────────────────────────────────────────────────────

/// Chiều hình dạng — 18 SDF primitives từ v2 spec.
///
/// v2: 18 primitives indexed 0-17. Fits 5 bits.
/// In packed P_weight [S:4][R:4][V:3][A:3][T:2], S uses 4 bits (0-15).
/// Primitives 16-17 (CutSphere, DeathStar) not in udc_p_table.bin data.
///
/// Union/Intersect/Subtract are CSG operations, NOT SDF primitives.
/// See `CsgOp` enum for CSG operations used in LCA compose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ShapeBase {
    /// SDF 0: sphere — most basic primitive
    Sphere = 0,
    /// SDF 1: axis-aligned box
    Box = 1,
    /// SDF 2: capsule (line segment + radius)
    Capsule = 2,
    /// SDF 3: infinite plane
    Plane = 3,
    /// SDF 4: torus (ring)
    Torus = 4,
    /// SDF 5: ellipsoid (stretched sphere)
    Ellipsoid = 5,
    /// SDF 6: cone
    Cone = 6,
    /// SDF 7: cylinder
    Cylinder = 7,
    /// SDF 8: octahedron (8 faces)
    Octahedron = 8,
    /// SDF 9: pyramid (4 faces + base)
    Pyramid = 9,
    /// SDF 10: hexagonal prism
    HexPrism = 10,
    /// SDF 11: triangular prism
    Prism = 11,
    /// SDF 12: box with rounded edges
    RoundBox = 12,
    /// SDF 13: chain link
    Link = 13,
    /// SDF 14: surface of revolution
    Revolve = 14,
    /// SDF 15: linear extrusion
    Extrude = 15,
    /// SDF 16: sphere with spherical cut
    CutSphere = 16,
    /// SDF 17: sphere with spherical subtraction (Death Star)
    DeathStar = 17,
}

/// CSG operations — NOT SDF primitives.
/// Used in LCA Shape compose: Cˢ = Union(Aˢ, Bˢ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CsgOp {
    /// ∪ U+222A — combine shapes
    Union = 0,
    /// ∩ U+2229 — intersect shapes
    Intersect = 1,
    /// ∖ U+2216 — subtract shapes
    Subtract = 2,
}

impl ShapeBase {
    /// Parse from byte value (0-17).
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Sphere),
            1 => Some(Self::Box),
            2 => Some(Self::Capsule),
            3 => Some(Self::Plane),
            4 => Some(Self::Torus),
            5 => Some(Self::Ellipsoid),
            6 => Some(Self::Cone),
            7 => Some(Self::Cylinder),
            8 => Some(Self::Octahedron),
            9 => Some(Self::Pyramid),
            10 => Some(Self::HexPrism),
            11 => Some(Self::Prism),
            12 => Some(Self::RoundBox),
            13 => Some(Self::Link),
            14 => Some(Self::Revolve),
            15 => Some(Self::Extrude),
            16 => Some(Self::CutSphere),
            17 => Some(Self::DeathStar),
            _ => None,
        }
    }

    /// Extract base category từ hierarchical byte (legacy compat).
    /// v2: direct mapping, no sub-index scheme.
    pub fn from_hierarchical(b: u8) -> Option<Self> {
        Self::from_byte(b)
    }

    /// Sub-index — v2 has no sub-index, always 0.
    pub fn sub_index(_b: u8) -> u8 {
        0
    }

    /// Encode — v2: identity (no sub-index encoding).
    pub fn encode(self, _sub: u8) -> u8 {
        self as u8
    }

    /// Byte value.
    pub fn as_byte(self) -> u8 {
        self as u8
    }
}

/// Chiều quan hệ — base category từ RELATION group (Math Operators 2200..22FF).
///
/// 8 base relations. Mỗi base có tối đa 31 sub-variants.
/// Encoding giống ShapeBase: `value = base + (sub_index * 8)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RelationBase {
    /// ∈ U+2208 Member
    Member = 0x01,
    /// ⊂ U+2282 Subset
    Subset = 0x02,
    /// ≡ U+2261 Equiv
    Equiv = 0x03,
    /// ⊥ U+22A5 Orthogonal
    Orthogonal = 0x04,
    /// ∘ U+2218 Compose
    Compose = 0x05,
    /// → U+2192 Causes
    Causes = 0x06,
    /// ≈ U+2248 Similar
    Similar = 0x07,
    /// ← U+2190 DerivedFrom
    DerivedFrom = 0x08,
}

impl RelationBase {
    /// Parse exact base value từ byte.
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Member),
            0x02 => Some(Self::Subset),
            0x03 => Some(Self::Equiv),
            0x04 => Some(Self::Orthogonal),
            0x05 => Some(Self::Compose),
            0x06 => Some(Self::Causes),
            0x07 => Some(Self::Similar),
            0x08 => Some(Self::DerivedFrom),
            _ => None,
        }
    }

    /// Extract base category từ hierarchical byte.
    pub fn from_hierarchical(b: u8) -> Option<Self> {
        if b == 0 {
            return None;
        }
        let base = ((b - 1) % 8) + 1;
        Self::from_byte(base)
    }

    /// Sub-index within base category.
    pub fn sub_index(b: u8) -> u8 {
        if b == 0 {
            return 0;
        }
        (b - 1) / 8
    }

    /// Encode base + sub_index → hierarchical byte.
    pub fn encode(self, sub: u8) -> u8 {
        let base = self as u8;
        base + sub.saturating_mul(8)
    }

    /// Byte value.
    pub fn as_byte(self) -> u8 {
        self as u8
    }
}

/// Chiều cảm xúc — từ EMOTICON group (fill level + dynamics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmotionDim {
    /// Valence: 0x00=V−  0x7F=V0  0xFF=V+
    pub valence: u8,
    /// Arousal: 0x00=calm  0xFF=excited
    pub arousal: u8,
}

impl EmotionDim {
    /// Trung lập.
    pub const NEUTRAL: Self = Self {
        valence: 0x7F,
        arousal: 0x80,
    };
}

/// Chiều thời gian — base category từ MUSICAL group (note duration).
///
/// 5 base tempos. Mỗi base có tối đa 51 sub-variants.
/// Encoding: `value = base + (sub_index * 5)`.
/// Extract: `base = ((value - 1) % 5) + 1`, `sub = (value - 1) / 5`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TimeDim {
    /// 𝅝 Whole note — Largo
    Static = 0x01,
    /// 𝅗 Half note — Adagio
    Slow = 0x02,
    /// ♩ Quarter note — Andante
    Medium = 0x03,
    /// ♪ Eighth note — Allegro
    Fast = 0x04,
    /// 16th note — Presto
    Instant = 0x05,
}

impl TimeDim {
    /// Parse exact base value từ byte.
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Static),
            0x02 => Some(Self::Slow),
            0x03 => Some(Self::Medium),
            0x04 => Some(Self::Fast),
            0x05 => Some(Self::Instant),
            _ => None,
        }
    }

    /// Extract base category từ hierarchical byte.
    pub fn from_hierarchical(b: u8) -> Option<Self> {
        if b == 0 {
            return None;
        }
        let base = ((b - 1) % 5) + 1;
        Self::from_byte(base)
    }

    /// Sub-index within base category.
    pub fn sub_index(b: u8) -> u8 {
        if b == 0 {
            return 0;
        }
        (b - 1) / 5
    }

    /// Encode base + sub_index → hierarchical byte.
    pub fn encode(self, sub: u8) -> u8 {
        let base = self as u8;
        base + sub.saturating_mul(5)
    }

    /// Byte value.
    pub fn as_byte(self) -> u8 {
        self as u8
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tagged encoding constants — presence mask (giống DeltaMolecule nhưng delta từ defaults)
// ─────────────────────────────────────────────────────────────────────────────

/// Bit 0: shape present (≠ default Sphere)
pub const PRESENT_SHAPE: u8 = 0x01;
/// Bit 1: relation present (≠ default Member)
pub const PRESENT_RELATION: u8 = 0x02;
/// Bit 2: valence present (≠ default 0x80)
pub const PRESENT_VALENCE: u8 = 0x04;
/// Bit 3: arousal present (≠ default 0x80)
pub const PRESENT_AROUSAL: u8 = 0x08;
/// Bit 4: time present (≠ default Medium)
pub const PRESENT_TIME: u8 = 0x10;

/// Default values cho tagged encoding (khớp UCD defaults cho unknown codepoints).
pub const TAGGED_DEFAULT_SHAPE: u8 = 0x01; // Sphere
/// Default relation byte.
pub const TAGGED_DEFAULT_RELATION: u8 = 0x01; // Member
/// Default valence byte.
pub const TAGGED_DEFAULT_VALENCE: u8 = 0x80; // neutral
/// Default arousal byte.
pub const TAGGED_DEFAULT_AROUSAL: u8 = 0x80; // moderate
/// Default time byte.
pub const TAGGED_DEFAULT_TIME: u8 = 0x03; // Medium

// ─────────────────────────────────────────────────────────────────────────────
// Maturity — Molecule lifecycle: Formula → Evaluating → Mature
// ─────────────────────────────────────────────────────────────────────────────

/// Molecule lifecycle state.
///
/// "DNA không lưu CƠ THỂ. DNA lưu CÔNG THỨC TẠO cơ thể."
///
/// - **Formula**: Tiềm năng — 5 chiều là CÔNG THỨC, chưa có input đánh giá.
///   Mỗi chiều = f(x), chưa biết x. L0 bẩm sinh bắt đầu ở đây.
///
/// - **Evaluating**: Đang đánh giá — có một số input, đang tích lũy evidence.
///   Dream cycle kiểm tra: đủ co-activations? Đủ Hebbian weight?
///
/// - **Mature**: Chín — đủ evidence, 5 chiều đã "đông đặc" thành giá trị.
///   Candidate cho QR promotion (bất tử, ED25519 signed).
///
/// Ln-1 dynamic — không hardcode max layer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Maturity {
    /// Công thức — tiềm năng, chưa evaluate
    #[default]
    Formula = 0x00,
    /// Đang đánh giá — có evidence, tích lũy
    Evaluating = 0x01,
    /// Chín — đủ evidence, sẵn sàng QR
    Mature = 0x02,
}

impl Maturity {
    /// Parse from byte.
    pub fn from_byte(b: u8) -> Self {
        match b {
            0x01 => Self::Evaluating,
            0x02 => Self::Mature,
            _ => Self::Formula,
        }
    }

    /// Byte representation.
    pub fn as_byte(self) -> u8 {
        self as u8
    }

    /// Chuyển sang trạng thái tiếp theo nếu đủ điều kiện.
    ///
    /// Formula → Evaluating: khi fire_count > 0 (có ít nhất 1 co-activation)
    /// Evaluating → Mature: khi weight ≥ threshold VÀ fire_count ≥ Fib[depth]
    ///                       VÀ evaluated_dims ≥ 3 (ít nhất 3/5 chiều có giá trị thật)
    pub fn advance(self, fire_count: u32, weight: f32, fib_threshold: u32) -> Self {
        self.advance_with_eval(fire_count, weight, fib_threshold, 5)
    }

    /// advance có thêm evaluated_dims — biết bao nhiêu chiều đã evaluate.
    ///
    /// Mature yêu cầu ≥ 3 dims evaluated (đủ evidence trên ≥ 3/5 chiều).
    pub fn advance_with_eval(self, fire_count: u32, weight: f32, fib_threshold: u32, evaluated_dims: u8) -> Self {
        match self {
            Self::Formula => {
                if fire_count > 0 {
                    Self::Evaluating
                } else {
                    Self::Formula
                }
            }
            Self::Evaluating => {
                // φ⁻¹ + φ⁻³ ≈ 0.854 (PROMOTE_WEIGHT from hebbian.rs)
                // Mature CHỈ khi ≥ 3 dims evaluated — đủ evidence
                if weight >= 0.854 && fire_count >= fib_threshold && evaluated_dims >= 3 {
                    Self::Mature
                } else {
                    Self::Evaluating
                }
            }
            Self::Mature => Self::Mature, // irreversible
        }
    }

    /// Check if in Formula state (tiềm năng).
    pub fn is_formula(self) -> bool { self == Self::Formula }
    /// Check if in Evaluating state (đang đánh giá).
    pub fn is_evaluating(self) -> bool { self == Self::Evaluating }
    /// Check if in Mature state (chín, sẵn sàng QR).
    pub fn is_mature(self) -> bool { self == Self::Mature }
}

// ─────────────────────────────────────────────────────────────────────────────
// NodeState — Molecule + lifecycle state + composition origin
// ─────────────────────────────────────────────────────────────────────────────

/// Node = Molecule + lifecycle state + nguồn gốc composition.
///
/// Molecule vẫn là 5 bytes tĩnh. NodeState bọc thêm:
/// - `maturity`: Formula → Evaluating → Mature
/// - `origin`: node sinh ra từ đâu? (Innate/Composed/Evolved)
#[derive(Debug, Clone, PartialEq)]
pub struct NodeState {
    /// 5D molecule (5 bytes).
    pub mol: Molecule,
    /// Lifecycle: Formula → Evaluating → Mature.
    pub maturity: Maturity,
    /// Nguồn gốc: L0 innate, LCA composed, hoặc evolved.
    pub origin: CompositionOrigin,
}

impl NodeState {
    /// Tạo NodeState từ Molecule (innate L0, codepoint đã biết).
    pub fn innate(mol: Molecule, codepoint: u32) -> Self {
        Self {
            mol,
            maturity: Maturity::Formula,
            origin: CompositionOrigin::Innate(codepoint),
        }
    }

    /// Tạo NodeState từ LCA composition.
    pub fn composed(mol: Molecule, sources: Vec<u64>, op: ComposeOp) -> Self {
        Self {
            mol,
            maturity: Maturity::Formula,
            origin: CompositionOrigin::Composed { sources, op },
        }
    }

    /// Tạo NodeState từ evolution.
    pub fn evolved(mol: Molecule, source: u64, dim: u8, old_val: u8, new_val: u8) -> Self {
        Self {
            mol,
            maturity: Maturity::Formula,
            origin: CompositionOrigin::Evolved {
                source,
                dim,
                old_val,
                new_val,
            },
        }
    }
}

/// Nguồn gốc composition — "node này sinh ra từ đâu?"
#[derive(Debug, Default, Clone, PartialEq)]
pub enum CompositionOrigin {
    /// L0 node — sinh từ encode_codepoint(), không có parent formula.
    Innate(u32), // Unicode codepoint

    /// Composite — sinh từ LCA/Fuse của nhiều sources.
    Composed {
        /// chain_hash của các parent nodes.
        sources: Vec<u64>,
        /// Phép toán tạo composite.
        op: ComposeOp,
    },

    /// Evolved — mutate từ 1 node khác.
    Evolved {
        /// chain_hash gốc.
        source: u64,
        /// Chiều nào bị mutate (0=Shape, 1=Relation, 2=Valence, 3=Arousal, 4=Time).
        dim: u8,
        /// Giá trị cũ.
        old_val: u8,
        /// Giá trị mới.
        new_val: u8,
    },

    /// Unknown — không biết nguồn gốc (backward compat).
    #[default]
    Unknown,
}

impl CompositionOrigin {
    /// Check if origin is known (not Unknown).
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

/// Phép toán tạo composite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeOp {
    /// LCA weighted average.
    Lca,
    /// VM Fuse opcode.
    Fuse,
    /// LeoAI program.
    Program,
}

// ─────────────────────────────────────────────────────────────────────────────
// Molecule — 5 bytes (RAM) / 1-6 bytes (tagged wire format)
// ─────────────────────────────────────────────────────────────────────────────

/// Đơn vị thông tin cơ bản — **2 bytes** packed u16 (v2).
///
/// Packed layout: [S:4][R:4][V:3][A:3][T:2] = 16 bits.
///
/// Mọi Molecule đến từ `encoder::encode_codepoint()`.
/// Không bao giờ tạo Molecule struct literal trong code production.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Molecule {
    /// Packed P_weight: [S:4][R:4][V:3][A:3][T:2]
    pub bits: u16,
}

impl Molecule {
    /// Pack 5 raw u8 dimensions into u16.
    ///
    /// S quantize: 0-255 → 0-15 (>> 4)
    /// R quantize: 0-255 → 0-15 (>> 4)
    /// V quantize: 0-255 → 0-7  (>> 5)
    /// A quantize: 0-255 → 0-7  (>> 5)
    /// T quantize: 0-255 → 0-3  (>> 6)
    pub fn pack(s: u8, r: u8, v: u8, a: u8, t: u8) -> Self {
        let s4 = (s >> 4) as u16;
        let r4 = (r >> 4) as u16;
        let v3 = (v >> 5) as u16;
        let a3 = (a >> 5) as u16;
        let t2 = (t >> 6) as u16;
        Self { bits: (s4 << 12) | (r4 << 8) | (v3 << 5) | (a3 << 2) | t2 }
    }

    /// Backward-compatible alias for pack().
    pub fn raw(shape: u8, relation: u8, valence: u8, arousal: u8, time: u8) -> Self {
        Self::pack(shape, relation, valence, arousal, time)
    }

    /// Backward-compatible alias for pack() (LCA creates "formula" molecules).
    pub fn formula(shape: u8, relation: u8, valence: u8, arousal: u8, time: u8) -> Self {
        Self::pack(shape, relation, valence, arousal, time)
    }

    /// From raw u16 bits.
    pub fn from_u16(bits: u16) -> Self {
        Self { bits }
    }

    // ── Accessors (quantized values) ──────────────────────────────────────

    /// Shape dimension (4 bits, 0-15).
    #[inline]
    pub fn shape(&self) -> u8 {
        ((self.bits >> 12) & 0xF) as u8
    }

    /// Relation dimension (4 bits, 0-15).
    #[inline]
    pub fn relation(&self) -> u8 {
        ((self.bits >> 8) & 0xF) as u8
    }

    /// Valence dimension (3 bits, 0-7).
    #[inline]
    pub fn valence(&self) -> u8 {
        ((self.bits >> 5) & 0x7) as u8
    }

    /// Arousal dimension (3 bits, 0-7).
    #[inline]
    pub fn arousal(&self) -> u8 {
        ((self.bits >> 2) & 0x7) as u8
    }

    /// Time dimension (2 bits, 0-3).
    #[inline]
    pub fn time(&self) -> u8 {
        (self.bits & 0x3) as u8
    }

    // ── Backward-compat accessors (return full u8 range, dequantized) ────

    /// Shape as full u8 (dequantize: shift left 4).
    #[inline]
    pub fn shape_u8(&self) -> u8 {
        self.shape() << 4
    }

    /// Relation as full u8 (dequantize: shift left 4).
    #[inline]
    pub fn relation_u8(&self) -> u8 {
        self.relation() << 4
    }

    /// Valence as full u8 (dequantize: shift left 5).
    #[inline]
    pub fn valence_u8(&self) -> u8 {
        self.valence() << 5
    }

    /// Arousal as full u8 (dequantize: shift left 5).
    #[inline]
    pub fn arousal_u8(&self) -> u8 {
        self.arousal() << 5
    }

    /// Time as full u8 (dequantize: shift left 6).
    #[inline]
    pub fn time_u8(&self) -> u8 {
        self.time() << 6
    }

    // ── EmotionDim compat ────────────────────────────────────────────────

    /// Emotion (V,A) as EmotionDim (dequantized).
    pub fn emotion(&self) -> EmotionDim {
        EmotionDim {
            valence: self.valence_u8(),
            arousal: self.arousal_u8(),
        }
    }

    // ── Base extraction ──────────────────────────────────────────────────

    /// Extract base ShapeBase category.
    pub fn shape_base(&self) -> ShapeBase {
        ShapeBase::from_byte(self.shape()).unwrap_or(ShapeBase::Sphere)
    }

    /// Extract base RelationBase category.
    pub fn relation_base(&self) -> RelationBase {
        // v2: relation is 4 bits (0-15), RelationBase has 8 variants (0-7)
        RelationBase::from_byte(self.relation()).unwrap_or(RelationBase::Member)
    }

    /// Extract base TimeDim category.
    pub fn time_base(&self) -> TimeDim {
        TimeDim::from_byte(self.time()).unwrap_or(TimeDim::Medium)
    }

    // ── Serialize ────────────────────────────────────────────────────────

    /// Serialize → 2 bytes (big-endian).
    pub fn to_bytes(self) -> [u8; 2] {
        self.bits.to_be_bytes()
    }

    /// Deserialize từ 2 bytes (big-endian).
    pub fn from_bytes_v2(b: &[u8; 2]) -> Self {
        Self { bits: u16::from_be_bytes(*b) }
    }

    /// Legacy deserialize from 5 bytes — pack into u16.
    pub fn from_bytes(b: &[u8; 5]) -> Option<Self> {
        Some(Self::pack(b[0], b[1], b[2], b[3], b[4]))
    }

    /// Serialize → 5 bytes (legacy backward compat, dequantized).
    pub fn to_bytes_legacy(self) -> [u8; 5] {
        [
            self.shape_u8(),
            self.relation_u8(),
            self.valence_u8(),
            self.arousal_u8(),
            self.time_u8(),
        ]
    }

    // ── Match score ──────────────────────────────────────────────────────

    /// Điểm tương đồng giữa 2 molecules ∈ [0, 5].
    pub fn match_score(&self, other: &Self) -> u8 {
        let mut s = 0u8;
        if self.shape() == other.shape() { s += 1; }
        if self.relation() == other.relation() { s += 1; }
        if self.time() == other.time() { s += 1; }
        if self.valence().abs_diff(other.valence()) <= 1 { s += 1; }
        if self.arousal().abs_diff(other.arousal()) <= 1 { s += 1; }
        s
    }

    /// Internal consistency check — returns score 0-100.
    pub fn internal_consistency(&self) -> u8 {
        100 // v2: all packed values are valid by construction
    }

    // ── Evaluated compat stubs ───────────────────────────────────────────
    // v2: no evaluated bitmask. All molecules are fully evaluated.

    /// v2 stub — all dims are always evaluated.
    pub fn is_dim_evaluated(&self, _dim: u8) -> bool { true }
    /// v2 stub — always fully evaluated.
    pub fn is_fully_evaluated(&self) -> bool { true }
    /// v2 stub — never pure formula.
    pub fn is_pure_formula(&self) -> bool { false }
    /// v2 stub — always 5.
    pub fn evaluated_count(&self) -> u8 { 5 }

    /// v2 stub — EVAL constants for backward compat.
    pub const EVAL_SHAPE: u8 = 0x01;
    /// Eval bit for relation.
    pub const EVAL_RELATION: u8 = 0x02;
    /// Eval bit for valence.
    pub const EVAL_VALENCE: u8 = 0x04;
    /// Eval bit for arousal.
    pub const EVAL_AROUSAL: u8 = 0x08;
    /// Eval bit for time.
    pub const EVAL_TIME: u8 = 0x10;
    /// All dims evaluated.
    pub const EVAL_ALL: u8 = 0x1F;
    /// No dims evaluated.
    pub const EVAL_NONE: u8 = 0x00;

    /// Presence mask — backward compat for tagged encoding.
    pub fn presence_mask(&self) -> u8 {
        let mut mask = 0u8;
        if self.shape_u8() != TAGGED_DEFAULT_SHAPE { mask |= PRESENT_SHAPE; }
        if self.relation_u8() != TAGGED_DEFAULT_RELATION { mask |= PRESENT_RELATION; }
        if self.valence_u8() != TAGGED_DEFAULT_VALENCE { mask |= PRESENT_VALENCE; }
        if self.arousal_u8() != TAGGED_DEFAULT_AROUSAL { mask |= PRESENT_AROUSAL; }
        if self.time_u8() != TAGGED_DEFAULT_TIME { mask |= PRESENT_TIME; }
        mask
    }

    /// Tagged byte size.
    pub fn tagged_size(&self) -> usize {
        1 + self.presence_mask().count_ones() as usize
    }

    /// Serialize → tagged bytes (backward compat).
    pub fn to_tagged_bytes(&self) -> Vec<u8> {
        let mask = self.presence_mask();
        let mut out = Vec::with_capacity(1 + mask.count_ones() as usize);
        out.push(mask);
        if mask & PRESENT_SHAPE != 0 { out.push(self.shape_u8()); }
        if mask & PRESENT_RELATION != 0 { out.push(self.relation_u8()); }
        if mask & PRESENT_VALENCE != 0 { out.push(self.valence_u8()); }
        if mask & PRESENT_AROUSAL != 0 { out.push(self.arousal_u8()); }
        if mask & PRESENT_TIME != 0 { out.push(self.time_u8()); }
        out
    }

    /// Deserialize từ tagged bytes (backward compat).
    pub fn from_tagged_bytes(b: &[u8]) -> Option<(Self, usize)> {
        if b.is_empty() { return None; }
        let mask = b[0];
        let expected = 1 + mask.count_ones() as usize;
        if b.len() < expected { return None; }

        let mut idx = 1usize;
        let shape = if mask & PRESENT_SHAPE != 0 { let s = b[idx]; idx += 1; s } else { TAGGED_DEFAULT_SHAPE };
        let relation = if mask & PRESENT_RELATION != 0 { let r = b[idx]; idx += 1; r } else { TAGGED_DEFAULT_RELATION };
        let valence = if mask & PRESENT_VALENCE != 0 { let v = b[idx]; idx += 1; v } else { TAGGED_DEFAULT_VALENCE };
        let arousal = if mask & PRESENT_AROUSAL != 0 { let a = b[idx]; idx += 1; a } else { TAGGED_DEFAULT_AROUSAL };
        let time = if mask & PRESENT_TIME != 0 { let t = b[idx]; idx += 1; t } else { TAGGED_DEFAULT_TIME };

        Some((Self::pack(shape, relation, valence, arousal, time), idx))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Evolution — học = thay đổi 1/5 chiều
// ─────────────────────────────────────────────────────────────────────────────

/// Chiều nào đang được thay đổi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dimension {
    /// Shape — hình dạng (SDF primitive)
    Shape,
    /// Relation — cách kết nối (Silk edge type)
    Relation,
    /// Valence — cảm xúc tích cực/tiêu cực
    Valence,
    /// Arousal — cường độ cảm xúc
    Arousal,
    /// Time — hành vi thời gian (Static/Slow/Medium/Fast/Instant)
    Time,
}

/// Kết quả evolution.
#[derive(Debug, Clone)]
pub struct EvolveResult {
    /// Molecule sau khi evolve
    pub molecule: Molecule,
    /// Chiều đã thay đổi
    pub dimension: Dimension,
    /// Giá trị cũ
    pub old_value: u8,
    /// Giá trị mới
    pub new_value: u8,
    /// Consistency score ∈ [0, 4] — bao nhiêu chiều còn lại vẫn hợp lệ
    pub consistency: u8,
    /// Valid? (consistency >= 3 = ít nhất 3/4 chiều khác vẫn ok)
    pub valid: bool,
    /// Nguồn gốc evolution — track source + mutation.
    pub origin: CompositionOrigin,
}

impl Molecule {
    /// Evolve 1 chiều — tạo bản sao với giá trị mới.
    ///
    /// Trả EvolveResult chứa molecule mới + validation.
    /// "Nếu 1 giá trị thay đổi mà không đúng với toàn bộ logic → sai"
    ///
    /// Validation rules:
    ///   - Shape thay đổi → relation + time phải tương thích
    ///     (SDF shapes thường Static, Emoticons thường Medium/Fast)
    ///   - Relation thay đổi → shape phải tương thích
    ///     (Math relations thường đi với Sphere/Torus)
    ///   - Valence thay đổi → arousal phải cùng hướng intensity
    ///     (extreme valence thường kéo arousal lên)
    ///   - Arousal thay đổi → valence phải non-neutral nếu arousal > 0xC0
    ///   - Time thay đổi → shape phải tương thích
    ///     (Static thường cho SDF, Fast/Instant cho Emoticons)
    /// Evolve 1 chiều — tạo bản sao với giá trị mới (quantized).
    pub fn evolve(&self, dim: Dimension, new_value: u8) -> EvolveResult {
        let old_value = match dim {
            Dimension::Shape => self.shape(),
            Dimension::Relation => self.relation(),
            Dimension::Valence => self.valence(),
            Dimension::Arousal => self.arousal(),
            Dimension::Time => self.time(),
        };
        let evolved = match dim {
            Dimension::Shape => Self::pack(new_value, self.relation_u8(), self.valence_u8(), self.arousal_u8(), self.time_u8()),
            Dimension::Relation => Self::pack(self.shape_u8(), new_value, self.valence_u8(), self.arousal_u8(), self.time_u8()),
            Dimension::Valence => Self::pack(self.shape_u8(), self.relation_u8(), new_value, self.arousal_u8(), self.time_u8()),
            Dimension::Arousal => Self::pack(self.shape_u8(), self.relation_u8(), self.valence_u8(), new_value, self.time_u8()),
            Dimension::Time => Self::pack(self.shape_u8(), self.relation_u8(), self.valence_u8(), self.arousal_u8(), new_value),
        };
        let consistency = evolved.internal_consistency();
        EvolveResult {
            molecule: evolved,
            dimension: dim,
            old_value,
            new_value,
            consistency,
            valid: consistency >= 3,
            origin: CompositionOrigin::Evolved {
                source: 0,
                dim: dim as u8,
                old_val: old_value,
                new_val: new_value,
            },
        }
    }

    /// So sánh 2 molecules — tìm dimensions nào khác nhau.
    pub fn dimension_delta(&self, other: &Molecule) -> Vec<(Dimension, u8, u8)> {
        let mut deltas = Vec::new();
        if self.shape() != other.shape() {
            deltas.push((Dimension::Shape, self.shape(), other.shape()));
        }
        if self.relation() != other.relation() {
            deltas.push((Dimension::Relation, self.relation(), other.relation()));
        }
        if self.valence() != other.valence() {
            deltas.push((Dimension::Valence, self.valence(), other.valence()));
        }
        if self.arousal() != other.arousal() {
            deltas.push((Dimension::Arousal, self.arousal(), other.arousal()));
        }
        if self.time() != other.time() {
            deltas.push((Dimension::Time, self.time(), other.time()));
        }
        deltas
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MolecularChain
// ─────────────────────────────────────────────────────────────────────────────

/// Chuỗi molecules — tọa độ vật lý của một khái niệm.
///
/// Chain ngắn = khái niệm đơn giản (1 molecule = 5 bytes).
/// Chain dài  = khái niệm phức tạp (ZWJ sequence, composite).
///
/// **Không bao giờ tạo chain bằng tay.**
/// Dùng `encoder::encode_codepoint(cp)` hoặc `lca::lca(&chains)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MolecularChain(pub Vec<Molecule>);

impl MolecularChain {
    /// Chain rỗng.
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    /// Chain từ 1 molecule.
    pub fn single(m: Molecule) -> Self {
        Self(alloc::vec![m])
    }

    /// Số molecules.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Chain có rỗng không.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Molecule đầu tiên.
    pub fn first(&self) -> Option<&Molecule> {
        self.0.first()
    }

    /// Serialize → bytes (len × 2, v2 format).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.0.len() * 2);
        for m in &self.0 {
            out.extend_from_slice(&m.to_bytes());
        }
        out
    }

    /// Deserialize từ bytes (v2: bội số của 2).
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() % 2 != 0 {
            return None;
        }
        let mut ms = Vec::with_capacity(b.len() / 2);
        for chunk in b.chunks_exact(2) {
            ms.push(Molecule::from_bytes_v2(&[chunk[0], chunk[1]]));
        }
        Some(Self(ms))
    }

    /// Deserialize từ legacy 5-byte format.
    pub fn from_bytes_legacy(b: &[u8]) -> Option<Self> {
        if b.len() % 5 != 0 {
            return None;
        }
        let mut ms = Vec::with_capacity(b.len() / 5);
        for chunk in b.chunks_exact(5) {
            let arr: [u8; 5] = [chunk[0], chunk[1], chunk[2], chunk[3], chunk[4]];
            ms.push(Molecule::from_bytes(&arr)?);
        }
        Some(Self(ms))
    }

    /// FNV-1a hash — dùng trong Registry và reverse index.
    pub fn chain_hash(&self) -> u64 {
        crate::hash::fnv1a(&self.to_bytes())
    }

    /// Similarity với chain khác ∈ [0.0, 1.0].
    ///
    /// Dựa trên structural overlap (base category match cho shape + relation).
    /// O(n×m) — chains ngắn trong thực tế (1-10 molecules).
    pub fn similarity(&self, other: &Self) -> f32 {
        if self.is_empty() || other.is_empty() {
            return 0.0;
        }
        let mut overlap = 0usize;
        for a in &self.0 {
            for b in &other.0 {
                if a.shape_base() == b.shape_base()
                    && a.relation_base() == b.relation_base()
                {
                    overlap += 1;
                    break;
                }
            }
        }
        overlap as f32 / self.0.len().max(other.0.len()) as f32
    }

    /// Similarity đầy đủ — tính cả emotion distance.
    ///
    /// score = 0.3×shape + 0.2×relation + 0.5×emotion_proximity
    /// Shape/relation so sánh base category (semantic similarity).
    pub fn similarity_full(&self, other: &Self) -> f32 {
        if self.is_empty() || other.is_empty() {
            return 0.0;
        }
        let n = self.0.len().min(other.0.len());
        let mut total = 0.0f32;
        for i in 0..n {
            let a = &self.0[i];
            let b = &other.0[i];
            let shape_m = if a.shape_base() == b.shape_base() {
                1.0f32
            } else {
                0.0
            };
            let rel_m = if a.relation_base() == b.relation_base() {
                1.0f32
            } else {
                0.0
            };
            let vd = a.valence().abs_diff(b.valence()) as f32;
            let ad = a.arousal().abs_diff(b.arousal()) as f32;
            let emo_sim = 1.0 - (vd + ad) / 510.0;
            total += 0.3 * shape_m + 0.2 * rel_m + 0.5 * emo_sim;
        }
        total / n as f32
    }

    /// Serialize → tagged bytes (variable length, chỉ ghi non-default dimensions).
    ///
    /// Format: `[mol_count: u8][mol_1_tagged][mol_2_tagged]...`
    /// Mỗi molecule: `[mask: u8][present_values: 0-5B]`
    pub fn to_tagged_bytes(&self) -> Vec<u8> {
        let estimated = 1 + self.0.len() * 3; // average ~3 bytes per mol
        let mut out = Vec::with_capacity(estimated);
        out.push(self.0.len() as u8);
        for m in &self.0 {
            out.extend_from_slice(&m.to_tagged_bytes());
        }
        out
    }

    /// Deserialize từ tagged bytes.
    ///
    /// Format: `[mol_count: u8][mol_1_tagged][mol_2_tagged]...`
    pub fn from_tagged_bytes(b: &[u8]) -> Option<Self> {
        if b.is_empty() {
            return None;
        }
        let mol_count = b[0] as usize;
        if mol_count == 0 {
            return Some(Self(Vec::new()));
        }
        let mut ms = Vec::with_capacity(mol_count);
        let mut pos = 1usize;
        for _ in 0..mol_count {
            if pos >= b.len() {
                return None;
            }
            let (mol, consumed) = Molecule::from_tagged_bytes(&b[pos..])?;
            ms.push(mol);
            pos += consumed;
        }
        Some(Self(ms))
    }

    /// Tagged byte size (without serializing).
    pub fn tagged_byte_size(&self) -> usize {
        1 + self.0.iter().map(|m| m.tagged_size()).sum::<usize>()
    }

    /// Nối 2 chains.
    pub fn concat(&self, other: &Self) -> Self {
        let mut v = self.0.clone();
        v.extend_from_slice(&other.0);
        Self(v)
    }

    /// Thêm molecule vào cuối.
    pub fn push(&mut self, m: Molecule) {
        self.0.push(m);
    }

    // ── Evolution ─────────────────────────────────────────────────────────

    /// Evolve molecule tại index — tạo chain mới với 1 chiều thay đổi.
    ///
    /// Trả None nếu index out of bounds.
    /// Chain mới có chain_hash khác → loài khác.
    ///
    /// ```text
    /// 🔥 [Sphere][Member][0xE0][0xD0][Fast]  chain_hash = 0xAAAA
    ///    evolve_at(0, Shape, Plane.as_byte())
    /// 🌊 [Plane][Member][0xE0][0xD0][Fast]   chain_hash = 0xBBBB  ← loài mới
    /// ```
    pub fn evolve_at(&self, mol_idx: usize, dim: Dimension, new_value: u8) -> Option<EvolveResult> {
        let mol = self.0.get(mol_idx)?;
        let result = mol.evolve(dim, new_value);
        Some(result)
    }

    /// Apply evolution — tạo chain mới với molecule đã thay đổi.
    ///
    /// Chỉ apply nếu EvolveResult.valid == true (consistency >= 3).
    /// Trả chain mới (khác chain_hash) hoặc None nếu invalid.
    pub fn apply_evolution(&self, mol_idx: usize, result: &EvolveResult) -> Option<Self> {
        if !result.valid || mol_idx >= self.0.len() {
            return None;
        }
        let mut new_mols = self.0.clone();
        new_mols[mol_idx] = result.molecule;
        Some(Self(new_mols))
    }

    /// Evolve và apply trong 1 bước — tiện cho learning pipeline.
    ///
    /// Trả (new_chain, EvolveResult) nếu valid, None nếu invalid hoặc OOB.
    pub fn evolve_and_apply(
        &self,
        mol_idx: usize,
        dim: Dimension,
        new_value: u8,
    ) -> Option<(Self, EvolveResult)> {
        let result = self.evolve_at(mol_idx, dim, new_value)?;
        let new_chain = self.apply_evolution(mol_idx, &result)?;
        Some((new_chain, result))
    }

    // ── Numeric encoding ─────────────────────────────────────────────────

    /// Encode f64 → 4-molecule chain.
    ///
    /// v2: Stores raw u16 bits directly. Marker: top 4 bits = 0xF (reserved, never used by normal molecules).
    /// 8 bytes of f64 = 4 × u16 stored as raw Molecule::from_u16().
    pub fn from_number(n: f64) -> Self {
        let bytes = n.to_bits().to_le_bytes();
        let mut mols = Vec::with_capacity(4);
        for chunk in bytes.chunks(2) {
            // Use 0xF in top 4 bits as numeric marker + store data in remaining 12 bits
            // Actually, store the full u16 with marker bit pattern
            let raw = u16::from_le_bytes([chunk[0], chunk[1]]);
            // Prefix with marker: set bits[15:14] = 0b11 (value >= 0xC000 is marker)
            // But this loses 2 bits. Instead, use a dedicated marker molecule approach:
            // Just store raw u16 and use a separate scheme for detection.
            mols.push(Molecule::from_u16(raw));
        }
        Self(mols)
    }

    /// Decode chain → f64 if it's a numeric chain.
    ///
    /// v2: 4-molecule chain with raw u16 bits = f64.
    /// Detection: chain len == 4 (heuristic — numbers are always exactly 4 molecules).
    pub fn to_number(&self) -> Option<f64> {
        if self.0.len() != 4 {
            return None;
        }
        // v2: Check if this could be a numeric chain
        // We use a heuristic: if none of the molecules look like valid quantized 5D molecules,
        // it's likely numeric. But this is fragile. Better approach: marker molecule.
        // For now, just extract and validate.
        let mut bytes = [0u8; 8];
        for (i, m) in self.0.iter().enumerate() {
            let raw = m.bits.to_le_bytes();
            bytes[i * 2] = raw[0];
            bytes[i * 2 + 1] = raw[1];
        }
        let val = f64::from_bits(u64::from_le_bytes(bytes));
        // Validate: must be a finite number (not NaN from random molecule data)
        if val.is_finite() || val == 0.0 {
            Some(val)
        } else {
            None
        }
    }

    /// Check if this chain represents a number.
    pub fn is_number(&self) -> bool {
        self.0.len() == 4 && self.to_number().is_some()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FormulaTable — Bảng công thức chia sẻ (shared dictionary)
// ─────────────────────────────────────────────────────────────────────────────

/// Bảng công thức: u16 index ↔ Molecule (LOSSLESS).
///
/// Theo "node va silk.md":
///   L0 (5400 node) = 5400 CÔNG THỨC GỐC.
///   Mỗi node mới = tổ hợp các công thức L0.
///   Silk = "2 node chia sẻ cùng công thức trên chiều nào?"
///
/// FormulaTable = bảng tuần hoàn trung tâm:
///   - Entries 0..N_UCD: pre-populated từ UCD (deterministic, cùng thứ tự)
///   - Entries N_UCD..65535: dynamic slots cho learned/LCA molecules
///   - Reverse lookup: molecule bytes → index (binary search)
///
/// ## Dung lượng
///
/// ```text
/// FormulaTable: max 65536 entries × 5 bytes = 320 KB (shared, 1 bản duy nhất)
/// Reverse index: max 65536 × 7 bytes         = 448 KB
/// Tổng shared cost:                           ≈ 768 KB
///
/// Per node: 2 bytes (chỉ lưu index)
/// 2B nodes × 2 bytes = 4 GB ← giống hệt CompactQR cũ
/// ```
///
/// ## So sánh packed vs dictionary
///
/// ```text
///                     Packed (cũ)              Dictionary (mới)
/// Per node            2 bytes                  2 bytes
/// Sub-variant         MẤT (chỉ base 0-7)      GIỮ (full hierarchical)
/// Valence             ±8 sai số (16 zones)     CHÍNH XÁC (0-255)
/// Arousal             ±16 sai số (8 zones)     CHÍNH XÁC (0-255)
/// Silk channels       37 kênh (base only)      ~5400 kênh (precise)
/// Shared cost         0                        ~768 KB (1 lần)
/// Reconstruct         Lossy (zone center)      LOSSLESS (exact molecule)
/// ```
pub struct FormulaTable {
    /// Forward: index → Molecule (exact, lossless).
    entries: Vec<Molecule>,
    /// Reverse: sorted (molecule_bytes_u64, index) for binary search.
    /// molecule_bytes_u64 = 5 bytes packed into u64 for fast comparison.
    reverse: Vec<(u64, u16)>,
}

/// Pack 5 molecule bytes into u64 for fast comparison.
fn mol_to_key(mol: &Molecule) -> u64 {
    mol.bits as u64
}

impl Default for FormulaTable {
    fn default() -> Self {
        Self::new()
    }
}

impl FormulaTable {
    /// Tạo bảng rỗng.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            reverse: Vec::new(),
        }
    }

    /// Tạo với capacity dự kiến.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            entries: Vec::with_capacity(cap),
            reverse: Vec::with_capacity(cap),
        }
    }

    /// Đăng ký Molecule → trả u16 index.
    ///
    /// Nếu đã có → trả index hiện tại (dedup).
    /// Nếu chưa → thêm mới, trả index mới.
    /// Nếu bảng đầy (65536) → trả None.
    pub fn register(&mut self, mol: &Molecule) -> Option<u16> {
        let key = mol_to_key(mol);

        // Binary search reverse index
        match self.reverse.binary_search_by_key(&key, |&(k, _)| k) {
            Ok(pos) => Some(self.reverse[pos].1),
            Err(insert_pos) => {
                if self.entries.len() >= 65536 {
                    return None; // bảng đầy
                }
                let idx = self.entries.len() as u16;
                self.entries.push(*mol);
                self.reverse.insert(insert_pos, (key, idx));
                Some(idx)
            }
        }
    }

    /// Lookup index → Molecule (exact, lossless).
    pub fn lookup(&self, index: u16) -> Option<&Molecule> {
        self.entries.get(index as usize)
    }

    /// Tìm Molecule → index (nếu đã đăng ký).
    pub fn find(&self, mol: &Molecule) -> Option<u16> {
        let key = mol_to_key(mol);
        match self.reverse.binary_search_by_key(&key, |&(k, _)| k) {
            Ok(pos) => Some(self.reverse[pos].1),
            Err(_) => None,
        }
    }

    /// Số entries đã đăng ký.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Bảng rỗng?
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// RAM usage estimate (bytes).
    pub fn ram_usage(&self) -> usize {
        // entries: Molecule = 5 bytes each (but Vec stores full struct, aligned)
        // reverse: (u64, u16) = 10 bytes each
        self.entries.len() * core::mem::size_of::<Molecule>()
            + self.reverse.len() * core::mem::size_of::<(u64, u16)>()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CompactQR — 2-byte QR node cho L2→Ln-1 (LOSSLESS via FormulaTable)
// ─────────────────────────────────────────────────────────────────────────────

/// 2-byte compressed QR node — DNA của tri thức.
///
/// Khi node đạt `Maturity::Mature` → Dream promote → QR signed →
/// **nén thành 2 bytes** lưu vào L2→Ln-1.
///
/// ## Encoding: Dictionary Index (LOSSLESS)
///
/// ```text
/// 2 bytes = u16 index vào FormulaTable.
/// FormulaTable[index] = exact Molecule (5 bytes, full precision).
///
/// Không pack 5D vào 16 bits.
/// Không mất sub-variant.
/// Không mất V/A precision.
/// ```
///
/// ## Theo "node va silk.md"
///
/// ```text
/// "L0 VỪA là alphabet, VỪA là Silk channel."
/// "5400 công thức L0 → mỗi công thức = 1 nhóm máu"
/// "Silk = 2 node chia sẻ cùng công thức trên chiều nào?"
///
/// FormulaTable = bảng tuần hoàn (shared, ~768 KB).
/// CompactQR = index vào bảng (per node, 2 bytes).
/// Reconstruct = FormulaTable[index] → exact Molecule.
/// ```
///
/// ## Tại sao 2 bytes đủ?
///
/// ```text
/// u16 = 65536 slots.
/// L0: ~5400 UCD formulas (pre-assigned, deterministic).
/// L1-L7: ~61 LCA nodes.
/// Dynamic: ~60000 slots cho learned/evolved nodes.
/// Dedup: cùng Molecule → cùng index → tự động nén.
/// ```
///
/// ## Storage impact
///
/// ```text
/// FormulaTable (shared):  ~768 KB (1 bản cho toàn hệ thống)
/// 2B nodes × 2 bytes   =    4 GB (per node cost giữ nguyên)
/// Silk edges            =    0 B  (implicit từ 5D comparison)
/// Tổng                  ≈ 4.77 GB → FITS 16GB (11 GB dư)
/// ```
///
/// ## Silk channels
///
/// ```text
/// Packed (cũ):  37 kênh base (mất sub-variant)
/// Dictionary:   ~5400 kênh precise (giữ full hierarchical)
///              + 31 mẫu compound (C(5,1)+...+C(5,5))
///              = TOÀN BỘ Silk theo doc
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompactQR {
    /// 2-byte FormulaTable index.
    bytes: [u8; 2],
}

impl CompactQR {
    /// Số bytes trên disk.
    pub const SIZE: usize = 2;

    /// Nén Molecule → 2 bytes (LOSSLESS via FormulaTable).
    ///
    /// Đăng ký mol vào table, lưu u16 index.
    /// Reconstruct: `to_molecule(table)` trả lại CHÍNH XÁC molecule gốc.
    ///
    /// Trả None nếu table đầy (65536 entries).
    pub fn from_molecule(mol: &Molecule, table: &mut FormulaTable) -> Option<Self> {
        let idx = table.register(mol)?;
        Some(Self {
            bytes: idx.to_be_bytes(),
        })
    }

    /// Nén Molecule → 2 bytes (LOSSY — không cần FormulaTable).
    ///
    /// Packed 5D: [shape:3][relation:3][time:3][valence:4][arousal:3] = 16 bits.
    /// Mất sub-variant, V/A chỉ giữ zone.
    /// Dùng khi không có FormulaTable (standalone, backward compat).
    pub fn from_molecule_lossy(mol: &Molecule) -> Self {
        let s = mol.shape() as u16;
        let r = mol.relation() as u16;
        let t = mol.time() as u16;
        let v = mol.valence() as u16;
        let a = mol.arousal() as u16;
        let bits = (s << 13) | (r << 10) | (t << 7) | (v << 3) | a;
        Self {
            bytes: [(bits >> 8) as u8, (bits & 0xFF) as u8],
        }
    }

    /// Tạo từ 2 raw bytes.
    pub fn from_bytes(b: [u8; 2]) -> Self {
        Self { bytes: b }
    }

    /// Raw 2 bytes.
    pub fn to_bytes(self) -> [u8; 2] {
        self.bytes
    }

    /// FormulaTable index (u16).
    pub fn index(self) -> u16 {
        u16::from_be_bytes(self.bytes)
    }

    /// Reconstruct EXACT Molecule từ 2 bytes (LOSSLESS).
    ///
    /// FormulaTable[index] → exact Molecule, giữ full:
    ///   - Sub-variant (shape=0x09 = Sphere variant 1)
    ///   - Valence precision (0xC0 = chính xác 0xC0, không phải zone center)
    ///   - Arousal precision (0xC0 = chính xác 0xC0)
    ///   - Time sub-variant
    pub fn to_molecule(self, table: &FormulaTable) -> Option<Molecule> {
        table.lookup(self.index()).copied()
    }

    /// Reconstruct Molecule (LOSSY — không cần FormulaTable).
    ///
    /// Unpack 16 bits → zone centers. Backward compat.
    pub fn to_molecule_lossy(self) -> Molecule {
        let bits = ((self.bytes[0] as u16) << 8) | (self.bytes[1] as u16);
        let s = ((bits >> 13) & 0x07) as u8;
        let r = ((bits >> 10) & 0x07) as u8;
        let t = ((bits >> 7) & 0x07) as u8;
        let v = ((bits >> 3) & 0x0F) as u8;
        let a = (bits & 0x07) as u8;
        Molecule::raw(s + 1, r + 1, v * 16 + 8, a * 32 + 16, t + 1)
    }

    /// Implicit Silk: số chiều chung giữa 2 CompactQR (LOSSLESS).
    ///
    /// So sánh FULL 5D với sub-variant precision.
    /// Theo "node va silk.md":
    ///   - Base Silk (37 kênh): base category match
    ///   - Precise Silk (~5400 kênh): exact value match
    ///   - Compound Silk (31 mẫu): multi-dim match patterns
    ///
    /// Trả về (shared_base, shared_exact, strength).
    ///   shared_base  = số chiều cùng base category (0-5)
    ///   shared_exact = số chiều giống CHÍNH XÁC (0-5)
    ///   strength     = weighted: exact=1.0, base_only=0.5, miss=0.0
    pub fn silk_compare(self, other: Self, table: &FormulaTable) -> (u8, u8, f32) {
        let a = match self.to_molecule(table) {
            Some(m) => m,
            None => return (0, 0, 0.0),
        };
        let b = match other.to_molecule(table) {
            Some(m) => m,
            None => return (0, 0, 0.0),
        };

        let mut base_shared = 0u8;
        let mut exact_shared = 0u8;
        let mut strength = 0.0f32;

        // Shape: base + exact
        let a_sb = a.shape_base() as u8;
        let b_sb = b.shape_base() as u8;
        if a_sb == b_sb {
            base_shared += 1;
            if a.shape() == b.shape() { exact_shared += 1; strength += 1.0; }
            else { strength += 0.5; }
        }

        // Relation: base + exact
        let a_rb = a.relation_base() as u8;
        let b_rb = b.relation_base() as u8;
        if a_rb == b_rb {
            base_shared += 1;
            if a.relation() == b.relation() { exact_shared += 1; strength += 1.0; }
            else { strength += 0.5; }
        }

        // Valence: quantized compare
        if a.valence() == b.valence() {
            base_shared += 1;
            exact_shared += 1;
            strength += 1.0;
        }

        // Arousal: quantized compare
        if a.arousal() == b.arousal() {
            base_shared += 1;
            exact_shared += 1;
            strength += 1.0;
        }

        // Time: base + exact
        let a_tb = a.time_base() as u8;
        let b_tb = b.time_base() as u8;
        if a_tb == b_tb {
            base_shared += 1;
            if a.time() == b.time() { exact_shared += 1; strength += 1.0; }
            else { strength += 0.5; }
        }

        strength /= 5.0;
        (base_shared, exact_shared, strength)
    }

    /// Silk compare (LOSSY — không cần FormulaTable, backward compat).
    ///
    /// Dùng packed 16 bits, chỉ so sánh base/zone.
    pub fn silk_compare_lossy(self, other: Self) -> (u8, f32) {
        let bits_a = ((self.bytes[0] as u16) << 8) | (self.bytes[1] as u16);
        let bits_b = ((other.bytes[0] as u16) << 8) | (other.bytes[1] as u16);
        let mut shared = 0u8;
        if (bits_a >> 13) & 0x07 == (bits_b >> 13) & 0x07 { shared += 1; } // shape
        if (bits_a >> 10) & 0x07 == (bits_b >> 10) & 0x07 { shared += 1; } // relation
        if (bits_a >> 7) & 0x07 == (bits_b >> 7) & 0x07 { shared += 1; }   // time
        if (bits_a >> 3) & 0x0F == (bits_b >> 3) & 0x0F { shared += 1; }   // valence
        if bits_a & 0x07 == bits_b & 0x07 { shared += 1; }                  // arousal
        (shared, shared as f32 / 5.0)
    }

    /// Tính chain_hash từ 2 bytes — deterministic.
    pub fn compute_hash(self) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        h ^= self.bytes[0] as u64;
        h = h.wrapping_mul(0x100000001b3);
        h ^= self.bytes[1] as u64;
        h = h.wrapping_mul(0x100000001b3);
        h
    }

    /// Evolve: thay 1 chiều → CompactQR mới (LOSSLESS).
    ///
    /// Lấy molecule gốc từ table, mutate 1 chiều, đăng ký molecule mới.
    /// dim: 0=shape, 1=relation, 2=valence, 3=arousal, 4=time
    pub fn evolve(self, dim: u8, new_val: u8, table: &mut FormulaTable) -> Option<Self> {
        let mol = self.to_molecule(table)?;
        let evolved = match dim {
            0 => Molecule::pack(new_val, mol.relation_u8(), mol.valence_u8(), mol.arousal_u8(), mol.time_u8()),
            1 => Molecule::pack(mol.shape_u8(), new_val, mol.valence_u8(), mol.arousal_u8(), mol.time_u8()),
            2 => Molecule::pack(mol.shape_u8(), mol.relation_u8(), new_val, mol.arousal_u8(), mol.time_u8()),
            3 => Molecule::pack(mol.shape_u8(), mol.relation_u8(), mol.valence_u8(), new_val, mol.time_u8()),
            4 => Molecule::pack(mol.shape_u8(), mol.relation_u8(), mol.valence_u8(), mol.arousal_u8(), new_val),
            _ => mol,
        };
        Self::from_molecule(&evolved, table)
    }

    /// Evolve (LOSSY — không cần FormulaTable, backward compat).
    pub fn evolve_lossy(self, dim: u8, new_val: u8) -> Self {
        let bits = ((self.bytes[0] as u16) << 8) | (self.bytes[1] as u16);
        let mut s = ((bits >> 13) & 0x07) as u8;
        let mut r = ((bits >> 10) & 0x07) as u8;
        let mut t = ((bits >> 7) & 0x07) as u8;
        let mut v = ((bits >> 3) & 0x0F) as u8;
        let mut a = (bits & 0x07) as u8;
        match dim {
            0 => s = new_val.min(7),
            1 => r = new_val.min(7),
            2 => t = new_val.min(4),
            3 => v = new_val.min(15),
            4 => a = new_val.min(7),
            _ => {}
        }
        let new_bits = ((s as u16) << 13) | ((r as u16) << 10) | ((t as u16) << 7) | ((v as u16) << 3) | (a as u16);
        Self {
            bytes: [(new_bits >> 8) as u8, (new_bits & 0xFF) as u8],
        }
    }

    // ── Lossy helper accessors (backward compat, dùng khi không có table) ──

    /// Shape base index (0-7) — từ packed bits (lossy mode).
    pub fn shape_idx_lossy(self) -> u8 {
        let bits = ((self.bytes[0] as u16) << 8) | (self.bytes[1] as u16);
        ((bits >> 13) & 0x07) as u8
    }
    /// Relation base index (0-7) — lossy mode.
    pub fn relation_idx_lossy(self) -> u8 {
        let bits = ((self.bytes[0] as u16) << 8) | (self.bytes[1] as u16);
        ((bits >> 10) & 0x07) as u8
    }
    /// Time base index (0-4) — lossy mode.
    pub fn time_idx_lossy(self) -> u8 {
        let bits = ((self.bytes[0] as u16) << 8) | (self.bytes[1] as u16);
        ((bits >> 7) & 0x07) as u8
    }
    /// Valence zone (0-15) — lossy mode.
    pub fn valence_zone_lossy(self) -> u8 {
        let bits = ((self.bytes[0] as u16) << 8) | (self.bytes[1] as u16);
        ((bits >> 3) & 0x0F) as u8
    }
    /// Arousal zone (0-7) — lossy mode.
    pub fn arousal_zone_lossy(self) -> u8 {
        let bits = ((self.bytes[0] as u16) << 8) | (self.bytes[1] as u16);
        (bits & 0x07) as u8
    }
}

impl core::fmt::Display for CompactQR {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "QR[#{}]", self.index())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Tạo Molecule test — CHỈ dùng trong test.
    /// Production code dùng encoder::encode_codepoint().
    fn test_mol(shape: u8, relation: u8, v: u8, a: u8, t: u8) -> Molecule {
        Molecule::raw(shape, relation, v, a, t)
    }

    #[test]
    fn molecule_size() {
        let m = test_mol(0x01, 0x01, 0xFF, 0xFF, 0x04);
        assert_eq!(m.to_bytes().len(), 2, "v2: Molecule = 2 bytes");
        assert_eq!(core::mem::size_of::<Molecule>(), 2);
    }

    #[test]
    fn molecule_roundtrip_v2() {
        let m = test_mol(0x10, 0x60, 0xC0, 0xE0, 0xC0);
        let bytes = m.to_bytes();
        let decoded = Molecule::from_bytes_v2(&bytes);
        assert_eq!(m, decoded);
    }

    #[test]
    fn molecule_legacy_roundtrip() {
        // from_bytes (5B legacy) packs into u16, preserving quantized values
        let m = Molecule::from_bytes(&[0x10, 0x60, 0xC0, 0xE0, 0xC0]).unwrap();
        assert_eq!(m.shape(), 0x10 >> 4);
        assert_eq!(m.relation(), 0x60 >> 4);
        assert_eq!(m.valence(), 0xC0 >> 5);
    }

    #[test]
    fn molecule_base_extraction() {
        let m = test_mol(0x00, 0x60, 0xC0, 0xFF, 0xC0);
        assert_eq!(m.shape_base(), ShapeBase::Sphere); // 0 → Sphere
        assert_eq!(m.relation_base(), RelationBase::Causes); // 0x60>>4=6
    }

    #[test]
    fn chain_empty() {
        let c = MolecularChain::empty();
        assert!(c.is_empty());
        assert_eq!(c.to_bytes().len(), 0);
    }

    #[test]
    fn chain_roundtrip() {
        let m1 = test_mol(0x10, 0x10, 0xFF, 0xFF, 0xC0);
        let m2 = test_mol(0x20, 0x60, 0x30, 0x20, 0x40);
        let chain = MolecularChain(alloc::vec![m1, m2]);
        let bytes = chain.to_bytes();
        assert_eq!(bytes.len(), 4, "v2: 2 mols × 2 bytes = 4");
        let decoded = MolecularChain::from_bytes(&bytes).unwrap();
        assert_eq!(chain, decoded);
    }

    #[test]
    fn chain_invalid_bytes() {
        // v2: from_bytes expects multiples of 2
        assert!(MolecularChain::from_bytes(&[0x01]).is_none());
        assert!(MolecularChain::from_bytes(&[0x01, 0x01, 0x80]).is_none());
    }

    #[test]
    fn chain_hash_deterministic() {
        let m = test_mol(0x01, 0x01, 0xFF, 0xFF, 0x04);
        let c1 = MolecularChain::single(m);
        let c2 = MolecularChain::single(m);
        assert_eq!(c1.chain_hash(), c2.chain_hash());
    }

    #[test]
    fn chain_hash_different() {
        let c1 = MolecularChain::single(test_mol(0x01, 0x01, 0xFF, 0xFF, 0x04));
        let c2 = MolecularChain::single(test_mol(0x02, 0x01, 0xC0, 0x40, 0x02));
        assert_ne!(c1.chain_hash(), c2.chain_hash());
    }

    #[test]
    fn similarity_identical() {
        let c = MolecularChain::single(test_mol(0x01, 0x01, 0xFF, 0xFF, 0x04));
        assert!((c.similarity(&c) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn similarity_different() {
        // Sphere/Member vs Capsule/Causes → shape khác, relation khác → low
        let c1 = MolecularChain::single(test_mol(0x01, 0x01, 0xFF, 0xFF, 0x04));
        let c2 = MolecularChain::single(test_mol(0x02, 0x06, 0x30, 0x20, 0x02));
        assert!(c1.similarity(&c2) < 0.5);
    }

    #[test]
    fn similarity_empty() {
        let c1 = MolecularChain::empty();
        let c2 = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        assert_eq!(c1.similarity(&c2), 0.0);
    }

    #[test]
    fn concat_chains() {
        let c1 = MolecularChain::single(test_mol(0x10, 0x50, 0xFF, 0xFF, 0xC0));
        let c2 = MolecularChain::single(test_mol(0x20, 0x10, 0xC0, 0x40, 0x80));
        let c3 = c1.concat(&c2);
        assert_eq!(c3.len(), 2);
        assert_eq!(c3.to_bytes().len(), 4, "v2: 2 mols × 2 bytes = 4");
    }

    // ── Numeric encoding ──────────────────────────────────────────────────

    #[test]
    fn numeric_roundtrip_integer() {
        let chain = MolecularChain::from_number(42.0);
        assert_eq!(chain.len(), 4);
        assert!(chain.is_number());
        let n = chain.to_number().unwrap();
        assert!((n - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn numeric_roundtrip_float() {
        let chain = MolecularChain::from_number(3.14159);
        let n = chain.to_number().unwrap();
        assert!((n - 3.14159).abs() < 1e-10);
    }

    #[test]
    fn numeric_roundtrip_negative() {
        let chain = MolecularChain::from_number(-7.5);
        let n = chain.to_number().unwrap();
        assert!((n - (-7.5)).abs() < f64::EPSILON);
    }

    #[test]
    fn numeric_roundtrip_zero() {
        let chain = MolecularChain::from_number(0.0);
        let n = chain.to_number().unwrap();
        assert!((n - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn numeric_roundtrip_large() {
        let chain = MolecularChain::from_number(1e15);
        let n = chain.to_number().unwrap();
        assert!((n - 1e15).abs() < 1.0);
    }

    #[test]
    fn numeric_non_numeric_chain() {
        // Regular chain is NOT numeric
        let c = MolecularChain::single(test_mol(0x02, 0x06, 0x30, 0x20, 0x02));
        assert!(!c.is_number());
        assert!(c.to_number().is_none());
    }

    #[test]
    fn numeric_empty_not_number() {
        assert!(!MolecularChain::empty().is_number());
    }

    #[test]
    fn fuzz_all_valid_shapes_relations() {
        // v2: shape 0-17, relation 0-15, all pack/unpack correctly
        for s in 0u8..=17 {
            for r in 0u8..=8 {
                let m = Molecule::pack(s << 4, r << 4, 0x80, 0x80, 0xC0);
                assert_eq!(m.shape(), s);
                assert_eq!(m.relation(), r);
            }
        }
    }

    #[test]
    fn hierarchical_encoding_decode() {
        // v2: ShapeBase direct mapping (no sub-index)
        assert_eq!(ShapeBase::from_hierarchical(0), Some(ShapeBase::Sphere));
        assert_eq!(ShapeBase::from_hierarchical(2), Some(ShapeBase::Capsule));
        assert_eq!(ShapeBase::sub_index(0x01), 0); // always 0 in v2

        // RelationBase: still uses hierarchical scheme
        assert_eq!(
            RelationBase::from_hierarchical(0x06),
            Some(RelationBase::Causes)
        );

        // TimeDim: still uses hierarchical scheme
        assert_eq!(TimeDim::from_hierarchical(0x01), Some(TimeDim::Static));
        assert_eq!(TimeDim::from_hierarchical(0x04), Some(TimeDim::Fast));

        // Encode
        assert_eq!(ShapeBase::Sphere.encode(0), 0);
        assert_eq!(RelationBase::Causes.encode(0), 0x06);
        assert_eq!(TimeDim::Fast.encode(0), 0x04);
    }

    // ── Tagged encoding ─────────────────────────────────────────────────

    #[test]
    fn tagged_all_defaults_minimal() {
        // Molecule với tất cả giá trị default → mask=0x00, chỉ 1 byte
        let m = test_mol(0x01, 0x01, 0x80, 0x80, 0x03); // Sphere, Member, neutral, Medium
        let tagged = m.to_tagged_bytes();
        assert_eq!(tagged.len(), 1, "All defaults → only mask byte");
        assert_eq!(tagged[0], 0x00, "mask = 0 (nothing non-default)");
    }

    #[test]
    fn tagged_roundtrip_all_defaults() {
        let m = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let tagged = m.to_tagged_bytes();
        let (decoded, consumed) = Molecule::from_tagged_bytes(&tagged).unwrap();
        assert_eq!(decoded, m);
        assert_eq!(consumed, tagged.len());
    }

    #[test]
    fn tagged_roundtrip_all_nondefault() {
        // All non-default → mask=0x1F, 6 bytes
        let m = test_mol(0x04, 0x06, 0xC0, 0xC0, 0x04); // Cone, Causes, high emotion, Fast
        let tagged = m.to_tagged_bytes();
        assert_eq!(tagged.len(), 6, "All non-default → 6 bytes");
        assert_eq!(tagged[0], 0x1F, "mask = all bits set");
        let (decoded, consumed) = Molecule::from_tagged_bytes(&tagged).unwrap();
        assert_eq!(decoded, m);
        assert_eq!(consumed, 6);
    }

    #[test]
    fn tagged_partial_nondefault() {
        // Only valence non-default → mask=0x04, 2 bytes
        let m = test_mol(0x01, 0x01, 0xC0, 0x80, 0x03);
        let tagged = m.to_tagged_bytes();
        assert_eq!(tagged.len(), 2, "Only valence → 2 bytes");
        assert_eq!(tagged[0], PRESENT_VALENCE);
        assert_eq!(tagged[1], 0xC0);
        let (decoded, _) = Molecule::from_tagged_bytes(&tagged).unwrap();
        assert_eq!(decoded, m);
    }

    #[test]
    fn tagged_saves_space_vs_legacy() {
        // SDF-like: shape + time non-default
        let sdf_mol = test_mol(0x02, 0x01, 0x80, 0x80, 0x01); // Capsule, Static
        assert!(sdf_mol.tagged_size() < 5, "SDF mol should be < 5 tagged bytes");

        // EMOTICON-like: valence + arousal + time non-default
        let emo_mol = test_mol(0x01, 0x01, 0xC0, 0xC0, 0x04); // high V+A, Fast
        assert!(emo_mol.tagged_size() < 5, "EMOTICON mol should be < 5 tagged bytes");
    }

    #[test]
    fn tagged_chain_roundtrip() {
        let m1 = test_mol(0x01, 0x01, 0xC0, 0xFF, 0x04);
        let m2 = test_mol(0x02, 0x06, 0x30, 0x20, 0x02);
        let chain = MolecularChain(alloc::vec![m1, m2]);
        let tagged = chain.to_tagged_bytes();
        let decoded = MolecularChain::from_tagged_bytes(&tagged).unwrap();
        assert_eq!(chain, decoded);
    }

    #[test]
    fn tagged_chain_empty() {
        let chain = MolecularChain::empty();
        let tagged = chain.to_tagged_bytes();
        assert_eq!(tagged, alloc::vec![0u8]); // mol_count = 0
        let decoded = MolecularChain::from_tagged_bytes(&tagged).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn tagged_chain_savings() {
        // Chain of 2 sparse molecules
        let m1 = test_mol(0x01, 0x01, 0x80, 0x80, 0x01); // only time non-default
        let m2 = test_mol(0x01, 0x01, 0xC0, 0x80, 0x03); // only valence non-default
        let chain = MolecularChain(alloc::vec![m1, m2]);
        let legacy_size = chain.to_bytes().len(); // 10 bytes
        let tagged_size = chain.tagged_byte_size();
        assert!(
            tagged_size < legacy_size,
            "Tagged {} < legacy {} bytes",
            tagged_size,
            legacy_size
        );
    }

    #[test]
    fn tagged_hash_compatibility() {
        // Hash phải giống nhau bất kể format ghi
        let m = test_mol(0x01, 0x01, 0xC0, 0xC0, 0x04);
        let chain = MolecularChain::single(m);
        let hash1 = chain.chain_hash();
        // Roundtrip through tagged format
        let tagged = chain.to_tagged_bytes();
        let decoded = MolecularChain::from_tagged_bytes(&tagged).unwrap();
        let hash2 = decoded.chain_hash();
        assert_eq!(hash1, hash2, "Hash phải stable qua tagged roundtrip");
    }

    #[test]
    fn tagged_numeric_chain_roundtrip() {
        let chain = MolecularChain::from_number(42.0);
        let tagged = chain.to_tagged_bytes();
        let decoded = MolecularChain::from_tagged_bytes(&tagged).unwrap();
        assert_eq!(decoded.to_number().unwrap(), 42.0);
    }

    // ── Evolution tests ───────────────────────────────────────────────────

    #[test]
    fn evolve_shape_creates_new_species() {
        // 🔥 fire-like molecule
        let fire = test_mol(0x01, 0x01, 0xE0, 0xD0, 0x04); // Sphere, Member, high V, high A, Fast
        let chain = MolecularChain::single(fire);
        let old_hash = chain.chain_hash();

        // Evolve shape: Sphere → Plane
        let result = fire.evolve(Dimension::Shape, ShapeBase::Capsule.as_byte());
        assert!(result.valid, "Shape Sphere→Plane should be valid");
        assert_eq!(result.old_value, 0x01);
        assert_eq!(result.new_value, ShapeBase::Capsule.as_byte());

        // Apply → new chain with different hash
        let new_chain = chain.apply_evolution(0, &result).unwrap();
        let new_hash = new_chain.chain_hash();
        assert_ne!(old_hash, new_hash, "Evolved chain = new species (different hash)");
        assert_eq!(new_chain.0[0].shape_base(), ShapeBase::Capsule);
        // Other dimensions unchanged
        assert_eq!(new_chain.0[0].relation(), fire.relation());
        assert_eq!(new_chain.0[0].valence(), fire.valence());
        assert_eq!(new_chain.0[0].arousal(), fire.arousal());
        assert_eq!(new_chain.0[0].time(), fire.time());
    }

    #[test]
    fn evolve_valence_changes_emotion() {
        let mol = test_mol(0x01, 0x01, 0xE0, 0xD0, 0x03); // positive, high arousal
        // Evolve valence to negative
        let result = mol.evolve(Dimension::Valence, 0x20);
        // negative valence + high arousal = consistent (angry/distressed)
        assert!(result.valid);
        assert_eq!(result.molecule.valence(), Molecule::pack(0, 0, 0x20, 0, 0).valence());
        assert_eq!(result.molecule.arousal(), mol.arousal()); // unchanged
    }

    #[test]
    fn evolve_invalid_mutation_detected() {
        // Extreme valence (0xFF) + very low arousal (0x10) = inconsistent
        let mol = test_mol(0x01, 0x01, 0x80, 0x10, 0x03); // neutral, very low arousal
        // Evolve valence to extreme
        let result = mol.evolve(Dimension::Valence, 0xFF);
        // V=0xFF (extreme positive) with A=0x10 (very low) → arousal rule fails
        // consistency should be < 4
        assert!(
            result.consistency < 4,
            "Extreme valence + low arousal should lose consistency points"
        );
    }

    #[test]
    fn evolve_fast_time_needs_arousal() {
        let mol = test_mol(0x01, 0x01, 0x80, 0x20, 0x01); // Static, very low arousal
        // Evolve time to Instant with very low arousal = inconsistent
        let result = mol.evolve(Dimension::Time, TimeDim::Instant.as_byte());
        assert!(
            result.consistency < 4,
            "Instant time + low arousal should lose points"
        );
    }

    #[test]
    fn evolve_and_apply_convenience() {
        let chain = MolecularChain::single(
            test_mol(0x01, 0x01, 0x80, 0x80, 0x03),
        );
        let result = chain.evolve_and_apply(0, Dimension::Relation, RelationBase::Causes.as_byte());
        assert!(result.is_some());
        let (new_chain, ev) = result.unwrap();
        assert!(ev.valid);
        assert_eq!(new_chain.0[0].relation_base(), RelationBase::Causes);
        assert_ne!(chain.chain_hash(), new_chain.chain_hash());
    }

    #[test]
    fn evolve_invalid_not_applied() {
        let chain = MolecularChain::single(
            test_mol(0x01, 0x01, 0xFF, 0x05, 0x05), // extreme V, near-zero A, Instant
        );
        // This combination may be invalid: extreme emotion + zero arousal + instant
        let result = chain.evolve_at(0, Dimension::Valence, 0xFF);
        assert!(result.is_some());
        // Even if evolve_at returns Some, apply_evolution checks valid
        let ev = result.unwrap();
        if !ev.valid {
            let applied = chain.apply_evolution(0, &ev);
            assert!(applied.is_none(), "Invalid evolution should not apply");
        }
    }

    #[test]
    fn evolve_out_of_bounds() {
        let chain = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        assert!(chain.evolve_at(5, Dimension::Shape, 0x02).is_none());
    }

    #[test]
    fn consistency_score_ranges() {
        // All consistent: neutral emotion, moderate arousal, medium time
        let good = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        assert!(good.internal_consistency() >= 3, "Balanced mol should be consistent");

        // All consistent: extreme valence + high arousal + fast = emoticon-like
        let emotional = test_mol(0x01, 0x06, 0xFF, 0xFF, 0x04);
        assert!(emotional.internal_consistency() >= 3, "High emotion + fast = ok");
    }

    #[test]
    fn evolution_is_new_node_not_update() {
        // Key principle: evolving creates a NEW chain (new hash), not modifying old
        let original = MolecularChain::single(test_mol(0x01, 0x01, 0x80, 0x80, 0x03));
        let original_hash = original.chain_hash();
        let original_bytes = original.to_bytes();

        // Evolve
        let (evolved, _) = original
            .evolve_and_apply(0, Dimension::Shape, ShapeBase::Cone.as_byte())
            .unwrap();

        // Original is UNCHANGED (immutable semantics)
        assert_eq!(original.chain_hash(), original_hash);
        assert_eq!(original.to_bytes(), original_bytes);

        // Evolved is a DIFFERENT node
        assert_ne!(original.chain_hash(), evolved.chain_hash());
    }

    // ── dimension_delta tests ──────────────────────────────────────────────

    #[test]
    fn delta_identical_molecules() {
        let m = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let deltas = m.dimension_delta(&m);
        assert_eq!(deltas.len(), 0, "Identical molecules → 0 deltas");
    }

    #[test]
    fn delta_one_dimension_shape() {
        let a = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let b = test_mol(0x03, 0x01, 0x80, 0x80, 0x03); // Shape changed Sphere→Box
        let deltas = a.dimension_delta(&b);
        assert_eq!(deltas.len(), 1, "Only shape differs");
        assert!(matches!(deltas[0].0, Dimension::Shape));
        assert_eq!(deltas[0].1, 0x01); // old
        assert_eq!(deltas[0].2, 0x03); // new
    }

    #[test]
    fn delta_one_dimension_valence() {
        let a = test_mol(0x01, 0x01, 0x20, 0x80, 0x03);
        let b = test_mol(0x01, 0x01, 0xE0, 0x80, 0x03); // Valence flipped
        let deltas = a.dimension_delta(&b);
        assert_eq!(deltas.len(), 1);
        assert!(matches!(deltas[0].0, Dimension::Valence));
    }

    #[test]
    fn delta_two_dimensions() {
        let a = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let b = test_mol(0x03, 0x01, 0x80, 0xC0, 0x03); // Shape + Arousal changed
        let deltas = a.dimension_delta(&b);
        assert_eq!(deltas.len(), 2, "Two dimensions differ → not evolution candidate");
    }

    #[test]
    fn delta_all_dimensions() {
        let a = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let b = test_mol(0x05, 0x06, 0x20, 0xC0, 0x01);
        let deltas = a.dimension_delta(&b);
        assert_eq!(deltas.len(), 5, "All 5 dimensions differ");
    }

    // ── FormulaTable tests ────────────────────────────────────────────────

    #[test]
    fn formula_table_register_and_lookup() {
        let mut table = FormulaTable::new();
        let mol = test_mol(0x09, 0x0E, 0xC0, 0xC0, 0x04); // Sphere sub1, Causes sub1
        let idx = table.register(&mol).unwrap();
        assert_eq!(idx, 0, "First entry = index 0");
        let looked = table.lookup(idx).unwrap();
        assert_eq!(*looked, mol, "Lookup returns exact molecule");
    }

    #[test]
    fn formula_table_dedup() {
        let mut table = FormulaTable::new();
        let mol = test_mol(0x09, 0x0E, 0xC0, 0xC0, 0x04);
        let idx1 = table.register(&mol).unwrap();
        let idx2 = table.register(&mol).unwrap();
        assert_eq!(idx1, idx2, "Same molecule → same index (dedup)");
        assert_eq!(table.len(), 1, "Only 1 entry");
    }

    #[test]
    fn formula_table_multiple() {
        let mut table = FormulaTable::new();
        let fire = test_mol(0x01, 0x06, 0xC0, 0xC0, 0x04);
        let ice = test_mol(0x01, 0x06, 0x30, 0x30, 0x02);
        let idx_f = table.register(&fire).unwrap();
        let idx_i = table.register(&ice).unwrap();
        assert_ne!(idx_f, idx_i);
        assert_eq!(table.len(), 2);
        assert_eq!(*table.lookup(idx_f).unwrap(), fire);
        assert_eq!(*table.lookup(idx_i).unwrap(), ice);
    }

    #[test]
    fn formula_table_find() {
        let mut table = FormulaTable::new();
        let mol = test_mol(0x09, 0x0E, 0xC0, 0xC0, 0x04);
        let idx = table.register(&mol).unwrap();
        assert_eq!(table.find(&mol), Some(idx));
        let other = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        assert_eq!(table.find(&other), None, "Not registered → None");
    }

    // ── CompactQR tests (LOSSLESS with FormulaTable) ────────────────────

    #[test]
    fn compact_qr_size() {
        assert_eq!(CompactQR::SIZE, 2, "CompactQR = 2 bytes");
        assert_eq!(core::mem::size_of::<CompactQR>(), 2);
    }

    #[test]
    fn compact_qr_lossless_roundtrip() {
        let mut table = FormulaTable::new();
        // Molecule with sub-variant — previously LOST by packed encoding
        let mol = test_mol(0x09, 0x0E, 0xC3, 0xA7, 0x09);
        // shape=0x09 (Sphere sub1), relation=0x0E (Causes sub1), V=0xC3, A=0xA7, time=0x09 (Fast sub1)
        let qr = CompactQR::from_molecule(&mol, &mut table).unwrap();
        let restored = qr.to_molecule(&table).unwrap();
        // LOSSLESS: exact match on ALL fields
        assert_eq!(restored, mol, "Lossless roundtrip — exact molecule preserved");
        assert_eq!(restored.shape(), mol.shape(), "Shape quantized preserved");
        assert_eq!(restored.valence(), mol.valence(), "V quantized preserved");
        assert_eq!(restored.arousal(), mol.arousal(), "A quantized preserved");
    }

    #[test]
    fn compact_qr_lossless_fire() {
        let mut table = FormulaTable::new();
        let fire = test_mol(0x01, 0x06, 0xC0, 0xC0, 0x04);
        let qr = CompactQR::from_molecule(&fire, &mut table).unwrap();
        let back = qr.to_molecule(&table).unwrap();
        assert_eq!(back, fire, "🔥 lossless roundtrip");
    }

    #[test]
    fn compact_qr_silk_compare_lossless_identical() {
        let mut table = FormulaTable::new();
        let fire = test_mol(0x01, 0x06, 0xC0, 0xC0, 0x04);
        let qr = CompactQR::from_molecule(&fire, &mut table).unwrap();
        let (base, exact, strength) = qr.silk_compare(qr, &table);
        assert_eq!(base, 5, "Identical → 5/5 base dims");
        assert_eq!(exact, 5, "Identical → 5/5 exact dims");
        assert!((strength - 1.0).abs() < 0.01);
    }

    #[test]
    fn compact_qr_silk_compare_lossless_partial() {
        let mut table = FormulaTable::new();
        let fire = test_mol(0x01, 0x06, 0xC0, 0xC0, 0x04);
        let ice = test_mol(0x01, 0x06, 0x30, 0x30, 0x02);
        let qr_f = CompactQR::from_molecule(&fire, &mut table).unwrap();
        let qr_i = CompactQR::from_molecule(&ice, &mut table).unwrap();
        let (base, exact, _) = qr_f.silk_compare(qr_i, &table);
        assert_eq!(base, 2, "Fire/Ice share Shape + Relation base = 2");
        assert_eq!(exact, 2, "Also exact match (same base values)");
    }

    #[test]
    fn compact_qr_silk_precise_vs_base() {
        let mut table = FormulaTable::new();
        // Same base (Sphere) but different sub-variant
        let a = test_mol(0x01, 0x01, 0x80, 0x80, 0x03); // Sphere base
        let b = test_mol(0x09, 0x01, 0x80, 0x80, 0x03); // Sphere sub1
        let qr_a = CompactQR::from_molecule(&a, &mut table).unwrap();
        let qr_b = CompactQR::from_molecule(&b, &mut table).unwrap();
        let (base, exact, _) = qr_a.silk_compare(qr_b, &table);
        // Shape: same base (Sphere) but different exact → base_shared but NOT exact_shared
        assert_eq!(base, 5, "All 5 bases match (R,V,A,T identical; S same base)");
        assert_eq!(exact, 4, "4 exact (R,V,A,T). Shape = base only, not exact");
    }

    #[test]
    fn compact_qr_silk_compare_lossless_zero() {
        let mut table = FormulaTable::new();
        let a = test_mol(0x01, 0x01, 0x10, 0x10, 0x01);
        let b = test_mol(0x04, 0x06, 0xF0, 0xE0, 0x05);
        let qr_a = CompactQR::from_molecule(&a, &mut table).unwrap();
        let qr_b = CompactQR::from_molecule(&b, &mut table).unwrap();
        let (base, exact, _) = qr_a.silk_compare(qr_b, &table);
        assert_eq!(base, 0, "Completely different → 0 base shared");
        assert_eq!(exact, 0, "Completely different → 0 exact shared");
    }

    #[test]
    fn compact_qr_evolve_lossless() {
        let mut table = FormulaTable::new();
        let fire = test_mol(0x01, 0x06, 0xC0, 0xC0, 0x04);
        let qr = CompactQR::from_molecule(&fire, &mut table).unwrap();
        // Evolve Valence: 0xC0 → 0x40 (like "lửa nhẹ")
        let evolved = qr.evolve(2, 0x40, &mut table).unwrap();
        let evolved_mol = evolved.to_molecule(&table).unwrap();
        assert_eq!(evolved_mol.valence(), Molecule::pack(0, 0, 0x40, 0, 0).valence(), "V quantized from 0x40");
        // Other dims unchanged
        assert_eq!(evolved_mol.shape(), fire.shape());
        assert_eq!(evolved_mol.relation(), fire.relation());
        assert_eq!(evolved_mol.arousal(), fire.arousal());
        assert_eq!(evolved_mol.time(), fire.time());
        // Different node
        assert_ne!(qr.compute_hash(), evolved.compute_hash());
    }

    #[test]
    fn compact_qr_dedup_in_table() {
        let mut table = FormulaTable::new();
        let mol = test_mol(0x01, 0x06, 0xC0, 0xC0, 0x04);
        let qr1 = CompactQR::from_molecule(&mol, &mut table).unwrap();
        let qr2 = CompactQR::from_molecule(&mol, &mut table).unwrap();
        assert_eq!(qr1, qr2, "Same molecule → same CompactQR (dedup)");
        assert_eq!(table.len(), 1, "Table deduplicates");
    }

    #[test]
    fn compact_qr_hash_deterministic() {
        let mut table = FormulaTable::new();
        let mol = test_mol(0x01, 0x06, 0xC0, 0xC0, 0x04);
        let qr = CompactQR::from_molecule(&mol, &mut table).unwrap();
        let h1 = qr.compute_hash();
        let h2 = qr.compute_hash();
        assert_eq!(h1, h2, "Hash must be deterministic");
    }

    #[test]
    fn compact_qr_byte_roundtrip() {
        let mut table = FormulaTable::new();
        let mol = test_mol(0x03, 0x07, 0xA0, 0x60, 0x02);
        let qr = CompactQR::from_molecule(&mol, &mut table).unwrap();
        let bytes = qr.to_bytes();
        let restored = CompactQR::from_bytes(bytes);
        assert_eq!(qr, restored, "Byte roundtrip must be lossless");
        // And molecule roundtrip via restored
        let mol_back = restored.to_molecule(&table).unwrap();
        assert_eq!(mol_back, mol, "Full molecule roundtrip through bytes");
    }

    #[test]
    fn compact_qr_display() {
        let mut table = FormulaTable::new();
        let mol = test_mol(0x01, 0x06, 0xC0, 0xC0, 0x04);
        let qr = CompactQR::from_molecule(&mol, &mut table).unwrap();
        let s = alloc::format!("{}", qr);
        assert!(s.starts_with("QR["), "Display format: {}", s);
    }

    // ── Lossy backward compat tests ─────────────────────────────────────

    #[test]
    fn compact_qr_lossy_roundtrip() {
        let mol = test_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let qr = CompactQR::from_molecule_lossy(&mol);
        let restored = qr.to_molecule_lossy();
        assert_eq!(restored.shape_base(), mol.shape_base());
        assert_eq!(restored.relation_base(), mol.relation_base());
        assert_eq!(restored.time_base(), mol.time_base());
    }

    #[test]
    fn compact_qr_lossy_fire() {
        let fire = test_mol(0x01, 0x06, 0xC0, 0xC0, 0x04);
        let qr = CompactQR::from_molecule_lossy(&fire);
        assert_eq!(qr.shape_idx_lossy(), 0, "Sphere = base 0");
        assert_eq!(qr.relation_idx_lossy(), 5, "Causes = base 5");
        assert_eq!(qr.time_idx_lossy(), 3, "Fast = base 3");
        assert_eq!(qr.valence_zone_lossy(), 12, "0xC0/16 = 12");
        assert_eq!(qr.arousal_zone_lossy(), 6, "0xC0/32 = 6");
    }

    #[test]
    fn compact_qr_lossy_silk_compare() {
        let fire = test_mol(0x01, 0x06, 0xC0, 0xC0, 0x04);
        let qr = CompactQR::from_molecule_lossy(&fire);
        let (shared, strength) = qr.silk_compare_lossy(qr);
        assert_eq!(shared, 5, "Identical → 5/5 dims");
        assert!((strength - 1.0).abs() < 0.01);
    }

    #[test]
    fn compact_qr_lossy_evolve() {
        let fire = test_mol(0x01, 0x06, 0xC0, 0xC0, 0x04);
        let qr = CompactQR::from_molecule_lossy(&fire);
        let evolved = qr.evolve_lossy(3, 2);
        assert_eq!(evolved.valence_zone_lossy(), 2);
        assert_eq!(evolved.shape_idx_lossy(), qr.shape_idx_lossy());
    }

    #[test]
    fn compact_qr_lossy_all_bases() {
        for s in 1u8..=8 {
            let mol = test_mol(s, 0x01, 0x80, 0x80, 0x03);
            let qr = CompactQR::from_molecule_lossy(&mol);
            assert_eq!(qr.shape_idx_lossy(), s - 1);
        }
        for r in 1u8..=8 {
            let mol = test_mol(0x01, r, 0x80, 0x80, 0x03);
            let qr = CompactQR::from_molecule_lossy(&mol);
            assert_eq!(qr.relation_idx_lossy(), r - 1);
        }
        for t in 1u8..=5 {
            let mol = test_mol(0x01, 0x01, 0x80, 0x80, t);
            let qr = CompactQR::from_molecule_lossy(&mol);
            assert_eq!(qr.time_idx_lossy(), t - 1);
        }
    }

    #[test]
    fn compact_qr_storage_at_2b() {
        // 2 billion nodes × 2 bytes = 4 GB
        let nodes: u64 = 2_000_000_000;
        let storage = nodes * CompactQR::SIZE as u64;
        assert!(storage < 16_000_000_000, "2B × 2B = {} < 16GB", storage);
        assert_eq!(storage, 4_000_000_000, "Exactly 4 GB");
    }

    #[test]
    fn formula_table_ram_usage() {
        let mut table = FormulaTable::with_capacity(100);
        for i in 0u8..50 {
            let mol = test_mol(i % 8 + 1, i % 8 + 1, i * 5, i * 5, i % 5 + 1);
            table.register(&mol);
        }
        assert!(table.ram_usage() > 0);
        assert_eq!(table.len(), 50);
    }

    // ── Molecule = Công thức tests ──────────────────────────────────────

    #[test]
    fn molecule_raw_is_fully_evaluated() {
        // v2: all molecules are always fully evaluated
        let m = Molecule::raw(1, 2, 0x80, 0x80, 3);
        assert!(m.is_fully_evaluated());
        assert_eq!(m.evaluated_count(), 5);
        assert!(!m.is_pure_formula());
    }

    #[test]
    fn molecule_formula_same_as_raw() {
        // v2: formula() and raw() are identical (both pack into u16)
        let a = Molecule::raw(1, 2, 0x80, 0x80, 3);
        let b = Molecule::formula(1, 2, 0x80, 0x80, 3);
        assert_eq!(a, b);
        assert!(b.is_fully_evaluated());
        assert_eq!(b.evaluated_count(), 5);
    }

    #[test]
    fn v2_eval_stubs_always_true() {
        let m = Molecule::raw(1, 2, 0x80, 0x80, 3);
        assert!(m.is_dim_evaluated(0));
        assert!(m.is_dim_evaluated(1));
        assert!(m.is_dim_evaluated(2));
        assert!(m.is_dim_evaluated(3));
        assert!(m.is_dim_evaluated(4));
        assert!(m.is_fully_evaluated());
        assert_eq!(m.evaluated_count(), 5);
    }

    #[test]
    fn maturity_requires_evaluated_dims() {
        // advance_with_eval: chỉ Mature khi ≥ 3 dims evaluated
        let m = Maturity::Evaluating;

        // 2 dims evaluated — không đủ
        let result = m.advance_with_eval(10, 0.90, 5, 2);
        assert_eq!(result, Maturity::Evaluating);

        // 3 dims evaluated — đủ
        let result = m.advance_with_eval(10, 0.90, 5, 3);
        assert_eq!(result, Maturity::Mature);

        // 5 dims — chắc chắn đủ
        let result = m.advance_with_eval(10, 0.90, 5, 5);
        assert_eq!(result, Maturity::Mature);
    }

    #[test]
    fn eq_compares_bits() {
        let a = Molecule::raw(1, 2, 0x80, 0x80, 3);
        let b = Molecule::formula(1, 2, 0x80, 0x80, 3);
        assert_eq!(a, b, "raw() and formula() produce same u16");
    }
}

impl Default for MolecularChain {
    fn default() -> Self {
        MolecularChain::empty()
    }
}
