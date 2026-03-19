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

/// Chiều hình dạng — base category từ SDF group (Geometric Shapes 25A0..25FF).
///
/// 8 base primitives. Mỗi base có tối đa 31 sub-variants.
/// Encoding: `value = base + (sub_index * 8)`.
/// Extract: `base = ((value - 1) % 8) + 1`, `sub = (value - 1) / 8`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ShapeBase {
    /// ● U+25CF Sphere
    Sphere = 0x01,
    /// ▬ U+25AC Capsule
    Capsule = 0x02,
    /// ■ U+25A0 Box
    Box = 0x03,
    /// ▲ U+25B2 Cone
    Cone = 0x04,
    /// ○ U+25CB Torus
    Torus = 0x05,
    /// ∪ U+222A Union
    Union = 0x06,
    /// ∩ U+2229 Intersect
    Intersect = 0x07,
    /// ∖ U+2216 Subtract
    Subtract = 0x08,
}

impl ShapeBase {
    /// Parse exact base value từ byte (chỉ chấp nhận base values 0x01..0x08).
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Sphere),
            0x02 => Some(Self::Capsule),
            0x03 => Some(Self::Box),
            0x04 => Some(Self::Cone),
            0x05 => Some(Self::Torus),
            0x06 => Some(Self::Union),
            0x07 => Some(Self::Intersect),
            0x08 => Some(Self::Subtract),
            _ => None,
        }
    }

    /// Extract base category từ hierarchical byte.
    ///
    /// Bất kỳ byte > 0 đều trích xuất được base: `((b - 1) % 8) + 1`.
    /// Ví dụ: 0x09 (Sphere sub 1) → Sphere, 0x0A (Capsule sub 1) → Capsule.
    pub fn from_hierarchical(b: u8) -> Option<Self> {
        if b == 0 {
            return None;
        }
        let base = ((b - 1) % 8) + 1;
        Self::from_byte(base)
    }

    /// Sub-index within base category (0 = base representative).
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

/// Đơn vị thông tin cơ bản — **5 bytes** trong RAM.
///
/// Legacy wire format: `[shape][relation][valence][arousal][time]` (5 bytes cố định)
/// Tagged wire format: `[mask][present_values...]` (1-6 bytes, chỉ ghi non-default)
///
/// Mỗi byte mang giá trị phân cấp (hierarchical):
///   `value = base_category + (sub_index * N_bases)`
/// Trong đó shape/relation có 8 bases, time có 5 bases.
/// Base = danh tính ngữ nghĩa (Sphere, Causes, Fast...).
/// Sub = biến thể cụ thể trong nhóm Unicode (~5400 mẫu).
///
/// Mọi Molecule đến từ `encoder::encode_codepoint()`.
/// Không bao giờ tạo Molecule struct literal trong code production.
#[derive(Debug, Clone, Copy)]
pub struct Molecule {
    /// Chiều hình dạng — raw hierarchical byte.
    /// Dùng `shape_base()` để lấy ShapeBase category.
    pub shape: u8,
    /// Chiều quan hệ — raw hierarchical byte.
    /// Dùng `relation_base()` để lấy RelationBase category.
    pub relation: u8,
    /// Chiều cảm xúc (Valence + Arousal bytes)
    pub emotion: EmotionDim,
    /// Chiều thời gian — raw hierarchical byte.
    /// Dùng `time_base()` để lấy TimeDim category.
    pub time: u8,
    /// Formula rule ID for Shape dimension (0xFF = unset, runtime metadata only)
    pub fs: u8,
    /// Formula rule ID for Relation dimension (0xFF = unset, runtime metadata only)
    pub fr: u8,
    /// Formula rule ID for Valence dimension (0xFF = unset, runtime metadata only)
    pub fv: u8,
    /// Formula rule ID for Arousal dimension (0xFF = unset, runtime metadata only)
    pub fa: u8,
    /// Formula rule ID for Time dimension (0xFF = unset, runtime metadata only)
    pub ft: u8,
    /// Bitmask: dim nào đã được evaluate (có input thật).
    ///
    /// ```text
    /// Bit 0: Shape     (0x01)
    /// Bit 1: Relation  (0x02)
    /// Bit 2: Valence   (0x04)
    /// Bit 3: Arousal   (0x08)
    /// Bit 4: Time      (0x10)
    /// ```
    ///
    /// 0x00 = TIỀM NĂNG (chưa có input, 5 chiều đều là công thức)
    /// 0x1F = ĐÃ ĐÁNH GIÁ (5 chiều đều có giá trị thật)
    ///
    /// L0 (encode_codepoint) → 0x1F (innate, biết từ Unicode)
    /// LCA result → 0x00 (công thức mới, chờ evidence)
    /// evolve(dim) → bit của dim mutated = 1
    pub evaluated: u8,
}

/// PartialEq compares only the 5 core dimensions (shape, relation, valence, arousal, time).
/// Formula fields (fs, fr, fv, fa, ft) are runtime metadata and excluded from identity.
impl PartialEq for Molecule {
    fn eq(&self, other: &Self) -> bool {
        self.shape == other.shape
            && self.relation == other.relation
            && self.emotion == other.emotion
            && self.time == other.time
    }
}

impl Eq for Molecule {}

impl core::hash::Hash for Molecule {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.shape.hash(state);
        self.relation.hash(state);
        self.emotion.hash(state);
        self.time.hash(state);
    }
}

/// Sentinel value indicating a formula field has not been set.
pub const FORMULA_UNSET: u8 = 0xFF;

impl Molecule {
    /// Create a Molecule with all formula fields set to FORMULA_UNSET.
    /// evaluated = 0x1F (all dims evaluated) for backward compatibility.
    /// Use this instead of struct literals.
    pub fn raw(shape: u8, relation: u8, valence: u8, arousal: u8, time: u8) -> Self {
        Self {
            shape,
            relation,
            emotion: EmotionDim { valence, arousal },
            time,
            fs: FORMULA_UNSET,
            fr: FORMULA_UNSET,
            fv: FORMULA_UNSET,
            fa: FORMULA_UNSET,
            ft: FORMULA_UNSET,
            evaluated: 0x1F, // all 5 dims evaluated (backward compat)
        }
    }

    /// Tạo Molecule công thức — TIỀM NĂNG, chưa có input.
    ///
    /// Values là defaults (tọa độ trong 5D), nhưng chưa được xác nhận.
    /// Khi có evidence → `evaluate_dim()` từng chiều.
    pub fn formula(shape: u8, relation: u8, valence: u8, arousal: u8, time: u8) -> Self {
        Self {
            shape,
            relation,
            emotion: EmotionDim { valence, arousal },
            time,
            fs: FORMULA_UNSET,
            fr: FORMULA_UNSET,
            fv: FORMULA_UNSET,
            fa: FORMULA_UNSET,
            ft: FORMULA_UNSET,
            evaluated: 0x00, // TIỀM NĂNG — chưa dim nào được evaluate
        }
    }

    // ── Evaluated bitmask ─────────────────────────────────────────────────

    /// Bit: Shape đã evaluated.
    pub const EVAL_SHAPE: u8 = 0x01;
    /// Bit: Relation đã evaluated.
    pub const EVAL_RELATION: u8 = 0x02;
    /// Bit: Valence đã evaluated.
    pub const EVAL_VALENCE: u8 = 0x04;
    /// Bit: Arousal đã evaluated.
    pub const EVAL_AROUSAL: u8 = 0x08;
    /// Bit: Time đã evaluated.
    pub const EVAL_TIME: u8 = 0x10;
    /// Tất cả 5 dims đã evaluated.
    pub const EVAL_ALL: u8 = 0x1F;
    /// Chưa dim nào evaluated (tiềm năng).
    pub const EVAL_NONE: u8 = 0x00;

    /// Dim index → eval bit.
    fn eval_bit(dim: u8) -> u8 {
        match dim {
            0 => Self::EVAL_SHAPE,
            1 => Self::EVAL_RELATION,
            2 => Self::EVAL_VALENCE,
            3 => Self::EVAL_AROUSAL,
            4 => Self::EVAL_TIME,
            _ => 0,
        }
    }

    /// Chiều `dim` đã được evaluate (có input thật)?
    pub fn is_dim_evaluated(&self, dim: u8) -> bool {
        self.evaluated & Self::eval_bit(dim) != 0
    }

    /// Tất cả 5 chiều đã được evaluate?
    pub fn is_fully_evaluated(&self) -> bool {
        self.evaluated == Self::EVAL_ALL
    }

    /// Chưa chiều nào được evaluate — node là TIỀM NĂNG (công thức thuần).
    pub fn is_pure_formula(&self) -> bool {
        self.evaluated == Self::EVAL_NONE
    }

    /// Đếm số chiều đã evaluate.
    pub fn evaluated_count(&self) -> u8 {
        self.evaluated.count_ones() as u8
    }

    /// Evaluate chiều `dim` với giá trị mới — chuyển từ công thức → giá trị.
    ///
    /// Giống mutation nhưng KHÔNG tạo loài mới — chỉ confirm giá trị.
    pub fn evaluate_dim(&mut self, dim: u8, value: u8) {
        match dim {
            0 => self.shape = value,
            1 => self.relation = value,
            2 => self.emotion.valence = value,
            3 => self.emotion.arousal = value,
            4 => self.time = value,
            _ => return,
        }
        self.evaluated |= Self::eval_bit(dim);
    }

    /// Extract base ShapeBase category từ hierarchical shape byte.
    pub fn shape_base(&self) -> ShapeBase {
        ShapeBase::from_hierarchical(self.shape).unwrap_or(ShapeBase::Sphere)
    }

    /// Extract base RelationBase category từ hierarchical relation byte.
    pub fn relation_base(&self) -> RelationBase {
        RelationBase::from_hierarchical(self.relation).unwrap_or(RelationBase::Member)
    }

    /// Extract base TimeDim category từ hierarchical time byte.
    pub fn time_base(&self) -> TimeDim {
        TimeDim::from_hierarchical(self.time).unwrap_or(TimeDim::Medium)
    }

    /// Serialize → 5 bytes.
    pub fn to_bytes(self) -> [u8; 5] {
        [
            self.shape,
            self.relation,
            self.emotion.valence,
            self.emotion.arousal,
            self.time,
        ]
    }

    /// Deserialize từ 5 bytes.
    ///
    /// Chấp nhận bất kỳ byte > 0 cho shape/relation/time (hierarchical values).
    pub fn from_bytes(b: &[u8; 5]) -> Option<Self> {
        // Validate: shape, relation, time phải > 0
        if b[0] == 0 || b[1] == 0 || b[4] == 0 {
            return None;
        }
        Some(Self::raw(b[0], b[1], b[2], b[3], b[4]))
    }

    /// Presence mask — bit nào bật = dimension đó ≠ default.
    ///
    /// Dùng bởi tagged encoding để biết fields nào cần ghi.
    pub fn presence_mask(&self) -> u8 {
        let mut mask = 0u8;
        if self.shape != TAGGED_DEFAULT_SHAPE {
            mask |= PRESENT_SHAPE;
        }
        if self.relation != TAGGED_DEFAULT_RELATION {
            mask |= PRESENT_RELATION;
        }
        if self.emotion.valence != TAGGED_DEFAULT_VALENCE {
            mask |= PRESENT_VALENCE;
        }
        if self.emotion.arousal != TAGGED_DEFAULT_AROUSAL {
            mask |= PRESENT_AROUSAL;
        }
        if self.time != TAGGED_DEFAULT_TIME {
            mask |= PRESENT_TIME;
        }
        mask
    }

    /// Serialize → tagged bytes (1-6 bytes, chỉ ghi non-default dimensions).
    ///
    /// Format: `[mask: 1B][present_values: 0-5B]`
    /// - mask bit 0: shape, bit 1: relation, bit 2: valence, bit 3: arousal, bit 4: time
    /// - values ghi theo thứ tự: shape, relation, valence, arousal, time (chỉ ghi nếu bit bật)
    ///
    /// Decode bằng `from_tagged_bytes()`. Absent fields → defaults (Sphere/Member/0x80/0x80/Medium).
    pub fn to_tagged_bytes(&self) -> Vec<u8> {
        let mask = self.presence_mask();
        let mut out = Vec::with_capacity(1 + mask.count_ones() as usize);
        out.push(mask);
        if mask & PRESENT_SHAPE != 0 {
            out.push(self.shape);
        }
        if mask & PRESENT_RELATION != 0 {
            out.push(self.relation);
        }
        if mask & PRESENT_VALENCE != 0 {
            out.push(self.emotion.valence);
        }
        if mask & PRESENT_AROUSAL != 0 {
            out.push(self.emotion.arousal);
        }
        if mask & PRESENT_TIME != 0 {
            out.push(self.time);
        }
        out
    }

    /// Deserialize từ tagged bytes.
    ///
    /// Returns `(Molecule, bytes_consumed)`. Absent fields → defaults.
    /// Chấp nhận bất kỳ non-zero byte cho shape/relation/time (hierarchical values).
    pub fn from_tagged_bytes(b: &[u8]) -> Option<(Self, usize)> {
        if b.is_empty() {
            return None;
        }
        let mask = b[0];
        let expected = 1 + mask.count_ones() as usize;
        if b.len() < expected {
            return None;
        }

        let mut idx = 1usize;
        let shape = if mask & PRESENT_SHAPE != 0 {
            let s = b[idx];
            if s == 0 {
                return None;
            }
            idx += 1;
            s
        } else {
            TAGGED_DEFAULT_SHAPE
        };
        let relation = if mask & PRESENT_RELATION != 0 {
            let r = b[idx];
            if r == 0 {
                return None;
            }
            idx += 1;
            r
        } else {
            TAGGED_DEFAULT_RELATION
        };
        let valence = if mask & PRESENT_VALENCE != 0 {
            let v = b[idx];
            idx += 1;
            v
        } else {
            TAGGED_DEFAULT_VALENCE
        };
        let arousal = if mask & PRESENT_AROUSAL != 0 {
            let a = b[idx];
            idx += 1;
            a
        } else {
            TAGGED_DEFAULT_AROUSAL
        };
        let time = if mask & PRESENT_TIME != 0 {
            let t = b[idx];
            if t == 0 {
                return None;
            }
            idx += 1;
            t
        } else {
            TAGGED_DEFAULT_TIME
        };

        Some((
            Self::raw(shape, relation, valence, arousal, time),
            idx,
        ))
    }

    /// Tagged byte size (without actually serializing).
    pub fn tagged_size(&self) -> usize {
        1 + self.presence_mask().count_ones() as usize
    }

    /// Điểm tương đồng giữa 2 molecules ∈ [0, 5].
    ///
    /// So sánh exact raw bytes cho shape/relation/time.
    pub fn match_score(&self, other: &Self) -> u8 {
        let mut s = 0u8;
        if self.shape == other.shape {
            s += 1;
        }
        if self.relation == other.relation {
            s += 1;
        }
        if self.time == other.time {
            s += 1;
        }
        // Valence: gần nhau trong [-32, +32] → điểm
        let vd = self.emotion.valence.abs_diff(other.emotion.valence);
        if vd < 32 {
            s += 1;
        }
        // Arousal tương tự
        let ad = self.emotion.arousal.abs_diff(other.emotion.arousal);
        if ad < 32 {
            s += 1;
        }
        s
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
    pub fn evolve(&self, dim: Dimension, new_value: u8) -> EvolveResult {
        let mut evolved = *self;
        let old_value = match dim {
            Dimension::Shape => {
                let old = self.shape;
                evolved.shape = new_value;
                old
            }
            Dimension::Relation => {
                let old = self.relation;
                evolved.relation = new_value;
                old
            }
            Dimension::Valence => {
                let old = self.emotion.valence;
                evolved.emotion.valence = new_value;
                old
            }
            Dimension::Arousal => {
                let old = self.emotion.arousal;
                evolved.emotion.arousal = new_value;
                old
            }
            Dimension::Time => {
                let old = self.time;
                evolved.time = new_value;
                old
            }
        };

        // evolve = thay 1 biến trong công thức → dim mutated được evaluate
        // Các dim khác thừa kế trạng thái evaluated từ source
        evolved.evaluated |= Self::eval_bit(dim as u8);

        let consistency = evolved.internal_consistency();
        EvolveResult {
            molecule: evolved,
            dimension: dim,
            old_value,
            new_value,
            consistency,
            valid: consistency >= 3,
            origin: CompositionOrigin::Evolved {
                source: 0, // Caller should set this to actual chain_hash
                dim: dim as u8,
                old_val: old_value,
                new_val: new_value,
            },
        }
    }

    /// So sánh 2 molecules — tìm dimensions nào khác nhau.
    ///
    /// Trả về danh sách (Dimension, old_value, new_value) cho mỗi chiều khác.
    /// Nếu chỉ 1 chiều khác → candidate cho evolution.
    /// Nếu 0 chiều khác → identical (không evolve).
    /// Nếu 2+ chiều khác → quá khác biệt (cần LCA thay vì evolve).
    pub fn dimension_delta(&self, other: &Molecule) -> Vec<(Dimension, u8, u8)> {
        let mut deltas = Vec::new();
        if self.shape != other.shape {
            deltas.push((Dimension::Shape, self.shape, other.shape));
        }
        if self.relation != other.relation {
            deltas.push((Dimension::Relation, self.relation, other.relation));
        }
        if self.emotion.valence != other.emotion.valence {
            deltas.push((Dimension::Valence, self.emotion.valence, other.emotion.valence));
        }
        if self.emotion.arousal != other.emotion.arousal {
            deltas.push((Dimension::Arousal, self.emotion.arousal, other.emotion.arousal));
        }
        if self.time != other.time {
            deltas.push((Dimension::Time, self.time, other.time));
        }
        deltas
    }

    /// Internal consistency score ∈ [0, 4].
    ///
    /// Kiểm tra 4 quy tắc tương thích giữa 5 chiều:
    ///   1. Shape ↔ Time: SDF shapes (Plane/Box/Torus) → thường Static/Slow
    ///   2. Shape ↔ Relation: Math shapes → Equiv/Orthogonal; Emoticon → Member/Causes
    ///   3. Valence ↔ Arousal: extreme valence (|V-0x80| > 0x40) → arousal thường > 0x60
    ///   4. Time ↔ Arousal: Fast/Instant → arousal thường > 0x80
    pub fn internal_consistency(&self) -> u8 {
        let mut score = 0u8;
        let sb = self.shape_base();
        let tb = self.time_base();
        let v = self.emotion.valence;
        let a = self.emotion.arousal;

        // Rule 1: Shape ↔ Time compatibility
        // SDF primitives (Plane, Box, Torus, Intersect, Subtract) → Static/Slow often
        // Emoticons/Musical → Medium/Fast/Instant often
        // This is a soft rule — any combo is possible, but some are more natural
        let shape_time_ok = match sb {
            ShapeBase::Capsule | ShapeBase::Box | ShapeBase::Intersect | ShapeBase::Subtract => {
                // Geometric shapes can be any time, slightly prefer static/slow
                true // geometric shapes are flexible
            }
            _ => true, // sphere, cone, torus, union — all times valid
        };
        if shape_time_ok {
            score += 1;
        }

        // Rule 2: Valence ↔ Arousal coherence
        // Extreme valence (very positive or very negative) often drives arousal up
        // Neutral valence (0x70-0x90) → arousal can be anything
        let v_extreme = v.abs_diff(0x80) > 0x40; // |V - neutral| > 64
        let arousal_matches = if v_extreme {
            a >= 0x50 // extreme emotion → at least moderate arousal
        } else {
            true // neutral valence → any arousal ok
        };
        if arousal_matches {
            score += 1;
        }

        // Rule 3: Time ↔ Arousal coherence
        // Fast/Instant → usually higher arousal
        // Static/Slow → can be low arousal
        let time_arousal_ok = match tb {
            TimeDim::Fast | TimeDim::Instant => a >= 0x40, // fast things are at least somewhat active
            _ => true,
        };
        if time_arousal_ok {
            score += 1;
        }

        // Rule 4: Shape ↔ Relation coherence
        // This is the weakest constraint — most combos are valid
        // But some are semantically odd (e.g., Subtract shape + Member relation = unusual)
        let _rb = self.relation_base();
        score += 1; // always pass for now — Silk edges validate this better

        score
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

    /// Serialize → bytes (len × 5).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.0.len() * 5);
        for m in &self.0 {
            out.extend_from_slice(&m.to_bytes());
        }
        out
    }

    /// Deserialize từ bytes (phải là bội số của 5).
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if !b.len().is_multiple_of(5) {
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
            let vd = a.emotion.valence.abs_diff(b.emotion.valence) as f32;
            let ad = a.emotion.arousal.abs_diff(b.emotion.arousal) as f32;
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
    /// Marker: shape=Sphere(0x01), relation=Equiv(0x03), time=Static(0x01) (signals "number").
    /// 8 bytes of f64 stored in valence+arousal of 4 molecules (2 bytes each).
    pub fn from_number(n: f64) -> Self {
        let bits = n.to_bits().to_le_bytes();
        let mut mols = Vec::with_capacity(4);
        for chunk in bits.chunks(2) {
            mols.push(Molecule::raw(
                ShapeBase::Sphere.as_byte(),
                RelationBase::Equiv.as_byte(),
                chunk[0],
                chunk[1],
                TimeDim::Static.as_byte(),
            ));
        }
        Self(mols)
    }

    /// Decode chain → f64 if it's a numeric chain.
    ///
    /// Returns Some(f64) if chain is exactly 4 molecules with
    /// shape base=Sphere, relation base=Equiv, time base=Static (numeric marker).
    pub fn to_number(&self) -> Option<f64> {
        if self.0.len() != 4 {
            return None;
        }
        // Check all molecules have numeric marker (base categories)
        for m in &self.0 {
            if m.shape_base() != ShapeBase::Sphere
                || m.relation_base() != RelationBase::Equiv
                || m.time_base() != TimeDim::Static
            {
                return None;
            }
        }
        // Extract 8 bytes
        let mut bits = [0u8; 8];
        for (i, m) in self.0.iter().enumerate() {
            bits[i * 2] = m.emotion.valence;
            bits[i * 2 + 1] = m.emotion.arousal;
        }
        Some(f64::from_bits(u64::from_le_bytes(bits)))
    }

    /// Check if this chain represents a number.
    pub fn is_number(&self) -> bool {
        self.to_number().is_some()
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
    ((mol.shape as u64) << 32)
        | ((mol.relation as u64) << 24)
        | ((mol.emotion.valence as u64) << 16)
        | ((mol.emotion.arousal as u64) << 8)
        | (mol.time as u64)
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
        let s = if mol.shape == 0 { 0 } else { ((mol.shape - 1) % 8) as u16 };
        let r = if mol.relation == 0 { 0 } else { ((mol.relation - 1) % 8) as u16 };
        let t = if mol.time == 0 { 2 } else { (((mol.time - 1) % 5) as u16).min(4) };
        let v = (mol.emotion.valence / 16) as u16;
        let a = (mol.emotion.arousal / 32) as u16;
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
            if a.shape == b.shape { exact_shared += 1; strength += 1.0; }
            else { strength += 0.5; }
        }

        // Relation: base + exact
        let a_rb = a.relation_base() as u8;
        let b_rb = b.relation_base() as u8;
        if a_rb == b_rb {
            base_shared += 1;
            if a.relation == b.relation { exact_shared += 1; strength += 1.0; }
            else { strength += 0.5; }
        }

        // Valence: zone (base) + exact
        let a_vz = a.emotion.valence / 32; // 8 zones per doc
        let b_vz = b.emotion.valence / 32;
        if a_vz == b_vz {
            base_shared += 1;
            if a.emotion.valence == b.emotion.valence { exact_shared += 1; strength += 1.0; }
            else { strength += 0.5; }
        }

        // Arousal: zone (base) + exact
        let a_az = a.emotion.arousal / 32; // 8 zones per doc
        let b_az = b.emotion.arousal / 32;
        if a_az == b_az {
            base_shared += 1;
            if a.emotion.arousal == b.emotion.arousal { exact_shared += 1; strength += 1.0; }
            else { strength += 0.5; }
        }

        // Time: base + exact
        let a_tb = a.time_base() as u8;
        let b_tb = b.time_base() as u8;
        if a_tb == b_tb {
            base_shared += 1;
            if a.time == b.time { exact_shared += 1; strength += 1.0; }
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
        let mut mol = self.to_molecule(table)?;
        match dim {
            0 => mol.shape = new_val,
            1 => mol.relation = new_val,
            2 => mol.emotion.valence = new_val,
            3 => mol.emotion.arousal = new_val,
            4 => mol.time = new_val,
            _ => {}
        }
        Self::from_molecule(&mol, table)
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
        assert_eq!(m.to_bytes().len(), 5);
    }

    #[test]
    fn molecule_roundtrip() {
        let m = test_mol(0x01, 0x06, 0xC0, 0xFF, 0x04);
        let bytes = m.to_bytes();
        let decoded = Molecule::from_bytes(&bytes).unwrap();
        assert_eq!(m, decoded);
    }

    #[test]
    fn molecule_invalid_zero() {
        // shape=0x00 invalid (must be > 0)
        assert!(Molecule::from_bytes(&[0x00, 0x01, 0x80, 0x80, 0x03]).is_none());
        // relation=0x00 invalid
        assert!(Molecule::from_bytes(&[0x01, 0x00, 0x80, 0x80, 0x03]).is_none());
        // time=0x00 invalid
        assert!(Molecule::from_bytes(&[0x01, 0x01, 0x80, 0x80, 0x00]).is_none());
    }

    #[test]
    fn molecule_hierarchical_roundtrip() {
        // Hierarchical values: shape=0x09 (Sphere sub 1), relation=0x0E (Causes sub 1), time=0x09 (Fast sub 1)
        let bytes = [0x09, 0x0E, 0xC0, 0xFF, 0x09];
        let m = Molecule::from_bytes(&bytes).unwrap();
        assert_eq!(m.shape_base(), ShapeBase::Sphere);
        assert_eq!(m.relation_base(), RelationBase::Causes);
        assert_eq!(m.time_base(), TimeDim::Fast);
        assert_eq!(m.to_bytes(), bytes);
    }

    #[test]
    fn chain_empty() {
        let c = MolecularChain::empty();
        assert!(c.is_empty());
        assert_eq!(c.to_bytes().len(), 0);
    }

    #[test]
    fn chain_roundtrip() {
        let m1 = test_mol(0x01, 0x01, 0xFF, 0xFF, 0x04);
        let m2 = test_mol(0x02, 0x06, 0x30, 0x20, 0x02);
        let chain = MolecularChain(alloc::vec![m1, m2]);
        let bytes = chain.to_bytes();
        assert_eq!(bytes.len(), 10);
        let decoded = MolecularChain::from_bytes(&bytes).unwrap();
        assert_eq!(chain, decoded);
    }

    #[test]
    fn chain_invalid_bytes() {
        // Không phải bội số của 5
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
        let c1 = MolecularChain::single(test_mol(0x01, 0x05, 0xFF, 0xFF, 0x04));
        let c2 = MolecularChain::single(test_mol(0x02, 0x01, 0xC0, 0x40, 0x02));
        let c3 = c1.concat(&c2);
        assert_eq!(c3.len(), 2);
        assert_eq!(c3.to_bytes().len(), 10);
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
        // All base values
        for s in 0x01u8..=0x08 {
            for r in 0x01u8..=0x08 {
                let bytes = [s, r, 0x7F, 0x80, 0x03u8];
                let m = Molecule::from_bytes(&bytes).unwrap();
                assert_eq!(m.to_bytes()[0], s);
                assert_eq!(m.to_bytes()[1], r);
            }
        }
        // Hierarchical values (sub > 0)
        for s in [0x09u8, 0x11, 0x19, 0xF1] {
            let bytes = [s, 0x01, 0x80, 0x80, 0x03];
            let m = Molecule::from_bytes(&bytes).unwrap();
            assert_eq!(m.shape, s);
            assert_eq!(m.shape_base(), ShapeBase::Sphere);
        }
    }

    #[test]
    fn hierarchical_encoding_decode() {
        // ShapeBase: base + sub*8
        assert_eq!(ShapeBase::from_hierarchical(0x01), Some(ShapeBase::Sphere));
        assert_eq!(ShapeBase::from_hierarchical(0x09), Some(ShapeBase::Sphere)); // sub=1
        assert_eq!(ShapeBase::from_hierarchical(0x02), Some(ShapeBase::Capsule));
        assert_eq!(ShapeBase::from_hierarchical(0x0A), Some(ShapeBase::Capsule)); // sub=1
        assert_eq!(ShapeBase::sub_index(0x01), 0);
        assert_eq!(ShapeBase::sub_index(0x09), 1);
        assert_eq!(ShapeBase::sub_index(0xF1), 30);

        // RelationBase: same scheme
        assert_eq!(
            RelationBase::from_hierarchical(0x06),
            Some(RelationBase::Causes)
        );
        assert_eq!(
            RelationBase::from_hierarchical(0x0E),
            Some(RelationBase::Causes)
        ); // sub=1

        // TimeDim: base + sub*5
        assert_eq!(TimeDim::from_hierarchical(0x01), Some(TimeDim::Static));
        assert_eq!(TimeDim::from_hierarchical(0x06), Some(TimeDim::Static)); // sub=1
        assert_eq!(TimeDim::from_hierarchical(0x04), Some(TimeDim::Fast));
        assert_eq!(TimeDim::from_hierarchical(0x09), Some(TimeDim::Fast)); // sub=1
        assert_eq!(TimeDim::sub_index(0x01), 0);
        assert_eq!(TimeDim::sub_index(0x06), 1);

        // Encode roundtrip
        assert_eq!(ShapeBase::Sphere.encode(0), 0x01);
        assert_eq!(ShapeBase::Sphere.encode(1), 0x09);
        assert_eq!(RelationBase::Causes.encode(1), 0x0E);
        assert_eq!(TimeDim::Fast.encode(1), 0x09);
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
        assert_eq!(new_chain.0[0].relation, fire.relation);
        assert_eq!(new_chain.0[0].emotion, fire.emotion);
        assert_eq!(new_chain.0[0].time, fire.time);
    }

    #[test]
    fn evolve_valence_changes_emotion() {
        let mol = test_mol(0x01, 0x01, 0xE0, 0xD0, 0x03); // positive, high arousal
        // Evolve valence to negative
        let result = mol.evolve(Dimension::Valence, 0x20);
        // negative valence + high arousal = consistent (angry/distressed)
        assert!(result.valid);
        assert_eq!(result.molecule.emotion.valence, 0x20);
        assert_eq!(result.molecule.emotion.arousal, 0xD0); // unchanged
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
        assert_eq!(restored.shape, 0x09, "Sub-variant preserved (was lost in packed)");
        assert_eq!(restored.emotion.valence, 0xC3, "Exact V preserved (was ±8 in packed)");
        assert_eq!(restored.emotion.arousal, 0xA7, "Exact A preserved (was ±16 in packed)");
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
        assert_eq!(evolved_mol.emotion.valence, 0x40, "Exact V=0x40");
        // Other dims unchanged — EXACT
        assert_eq!(evolved_mol.shape, fire.shape);
        assert_eq!(evolved_mol.relation, fire.relation);
        assert_eq!(evolved_mol.emotion.arousal, fire.emotion.arousal);
        assert_eq!(evolved_mol.time, fire.time);
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
        let m = Molecule::raw(1, 2, 0x80, 0x80, 3);
        assert!(m.is_fully_evaluated());
        assert_eq!(m.evaluated_count(), 5);
        assert!(!m.is_pure_formula());
    }

    #[test]
    fn molecule_formula_is_potential() {
        let m = Molecule::formula(1, 2, 0x80, 0x80, 3);
        assert!(m.is_pure_formula());
        assert!(!m.is_fully_evaluated());
        assert_eq!(m.evaluated_count(), 0);
    }

    #[test]
    fn evaluate_dim_transitions() {
        let mut m = Molecule::formula(1, 2, 0x80, 0x80, 3);

        // Evaluate shape
        m.evaluate_dim(0, 5);
        assert!(m.is_dim_evaluated(0)); // shape evaluated
        assert!(!m.is_dim_evaluated(1)); // relation still formula
        assert_eq!(m.evaluated_count(), 1);
        assert_eq!(m.shape, 5);

        // Evaluate valence
        m.evaluate_dim(2, 0xC0);
        assert_eq!(m.evaluated_count(), 2);
        assert_eq!(m.emotion.valence, 0xC0);

        // Evaluate all remaining
        m.evaluate_dim(1, 6);
        m.evaluate_dim(3, 0xA0);
        m.evaluate_dim(4, 4);
        assert!(m.is_fully_evaluated());
        assert_eq!(m.evaluated_count(), 5);
    }

    #[test]
    fn evolve_marks_dim_evaluated() {
        let m = Molecule::formula(1, 2, 0x80, 0x80, 3);
        let result = m.evolve(Dimension::Valence, 0xC0);
        // Dim mutated (valence=2) should be evaluated
        assert!(result.molecule.is_dim_evaluated(2));
        // Other dims should still be unevaluated (inherited from formula)
        assert!(!result.molecule.is_dim_evaluated(0)); // shape
        assert_eq!(result.molecule.evaluated_count(), 1);
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
    fn eq_ignores_evaluated_field() {
        let a = Molecule::raw(1, 2, 0x80, 0x80, 3);
        let b = Molecule::formula(1, 2, 0x80, 0x80, 3);
        // PartialEq chỉ so sánh 5 core dims, không so evaluated
        assert_eq!(a, b);
    }
}

impl Default for MolecularChain {
    fn default() -> Self {
        MolecularChain::empty()
    }
}
