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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

impl Molecule {
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
        Some(Self {
            shape: b[0],
            relation: b[1],
            emotion: EmotionDim {
                valence: b[2],
                arousal: b[3],
            },
            time: b[4],
        })
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
            Self {
                shape,
                relation,
                emotion: EmotionDim { valence, arousal },
                time,
            },
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

        let consistency = evolved.internal_consistency();
        EvolveResult {
            molecule: evolved,
            dimension: dim,
            old_value,
            new_value,
            consistency,
            valid: consistency >= 3,
        }
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
            let arr: [u8; 5] = chunk.try_into().unwrap();
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
            mols.push(Molecule {
                shape: ShapeBase::Sphere.as_byte(),
                relation: RelationBase::Equiv.as_byte(),
                emotion: EmotionDim {
                    valence: chunk[0],
                    arousal: chunk[1],
                },
                time: TimeDim::Static.as_byte(),
            });
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
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Tạo Molecule test — CHỈ dùng trong test.
    /// Production code dùng encoder::encode_codepoint().
    fn test_mol(shape: u8, relation: u8, v: u8, a: u8, t: u8) -> Molecule {
        Molecule {
            shape,
            relation,
            emotion: EmotionDim {
                valence: v,
                arousal: a,
            },
            time: t,
        }
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
}

impl Default for MolecularChain {
    fn default() -> Self {
        MolecularChain::empty()
    }
}
