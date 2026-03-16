//! # edge — Silk Edge Types
//!
//! Mỗi edge mang:
//!   - EdgeKind: loại quan hệ (từ RELATION group)
//!   - EmotionTag V/A: màu cảm xúc lúc co-activation
//!   - weight: Hebbian strength ∈ [0.0, 1.0]
//!   - fire_count: số lần co-activate

// ─────────────────────────────────────────────────────────────────────────────
// EmotionTag — màu cảm xúc của edge
// ─────────────────────────────────────────────────────────────────────────────

/// EmotionTag 4 chiều của một khoảnh khắc.
///
/// V = Valence   ∈ [-1.0, +1.0]  (tiêu cực → tích cực)
/// A = Arousal   ∈ [ 0.0,  1.0]  (bình thản → kích động)
/// D = Dominance ∈ [ 0.0,  1.0]  (phụ thuộc → kiểm soát)
/// I = Intensity ∈ [ 0.0,  1.0]  (nhẹ → mạnh)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmotionTag {
    pub valence: f32,
    pub arousal: f32,
    pub dominance: f32,
    pub intensity: f32,
}

impl EmotionTag {
    /// Trung lập.
    pub const NEUTRAL: Self = Self {
        valence: 0.0,
        arousal: 0.3,
        dominance: 0.5,
        intensity: 0.2,
    };

    /// Tạo mới.
    pub const fn new(v: f32, a: f32, d: f32, i: f32) -> Self {
        Self {
            valence: v,
            arousal: a,
            dominance: d,
            intensity: i,
        }
    }

    /// Blend 2 EmotionTags với tỷ lệ alpha.
    pub fn blend(self, other: Self, alpha: f32) -> Self {
        let b = 1.0 - alpha;
        Self {
            valence: self.valence * alpha + other.valence * b,
            arousal: self.arousal * alpha + other.arousal * b,
            dominance: self.dominance * alpha + other.dominance * b,
            intensity: self.intensity * alpha + other.intensity * b,
        }
    }

    /// Khoảng cách Euclidean V/A.
    pub fn distance_va(&self, other: &Self) -> f32 {
        let dv = self.valence - other.valence;
        let da = self.arousal - other.arousal;
        libm::sqrtf(dv * dv + da * da)
    }

    /// Từ bytes UCD (valence_byte, arousal_byte).
    ///
    /// Valence: byte/128.0 - 1.0 → [-1.0, +0.9921875]
    /// 128.0 = power of 2 → exact in IEEE 754 (trước đây 127.5 gây rounding error).
    /// Byte 128 → 0.0 (neutral) — exact midpoint.
    pub fn from_ucd_bytes(valence_b: u8, arousal_b: u8) -> Self {
        Self {
            valence: (valence_b as f32 / 128.0) - 1.0,
            arousal: arousal_b as f32 / 255.0,
            dominance: 0.5,
            intensity: arousal_b as f32 / 255.0,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EdgeKind — loại quan hệ
// ─────────────────────────────────────────────────────────────────────────────

/// Loại Silk edge — từ RELATION group Unicode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EdgeKind {
    // Structural (L0 bất biến)
    Member = 0x01,      // ∈
    Subset = 0x02,      // ⊂
    Equiv = 0x03,       // ≡
    Orthogonal = 0x04,  // ⊥
    Compose = 0x05,     // ∘
    Causes = 0x06,      // →
    Similar = 0x07,     // ≈
    DerivedFrom = 0x08, // ←
    // Space
    Contains = 0x09,   // ∪
    Intersects = 0x0A, // ∩
    Subtracts = 0x0B,  // ∖
    Mirror = 0x0C,     // ↔
    // Time
    Flows = 0x0D,     // ⟶
    Repeats = 0x0E,   // ⟳
    Resolves = 0x0F,  // ↑
    Activates = 0x10, // ⚡
    Sync = 0x11,      // ∥
    // Language
    Translates = 0x12, // f(L) alias
    // Associative learned (Hebbian + EmotionTag)
    Assoc = 0xFF,      // ~ co-activation (generic)
    EdgeAssoc = 0xA0,  // ~ liên tưởng học được (với EmotionTag + source)
    EdgeCausal = 0xA1, // →→ nhân quả học được (với confidence)
    // QR Supersession
    Supersedes = 0xF0, // B supersedes A
}

impl EdgeKind {
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
            0x09 => Some(Self::Contains),
            0x0A => Some(Self::Intersects),
            0x0B => Some(Self::Subtracts),
            0x0C => Some(Self::Mirror),
            0x0D => Some(Self::Flows),
            0x0E => Some(Self::Repeats),
            0x0F => Some(Self::Resolves),
            0x10 => Some(Self::Activates),
            0x11 => Some(Self::Sync),
            0x12 => Some(Self::Translates),
            0xF0 => Some(Self::Supersedes),
            0xFF => Some(Self::Assoc),
            0xA0 => Some(Self::EdgeAssoc),
            0xA1 => Some(Self::EdgeCausal),
            _ => None,
        }
    }

    pub fn as_byte(self) -> u8 {
        self as u8
    }

    /// Structural edge — bất biến, không thay đổi weight.
    pub fn is_structural(self) -> bool {
        matches!(
            self,
            Self::Member
                | Self::Subset
                | Self::Equiv
                | Self::Orthogonal
                | Self::Compose
                | Self::Causes
                | Self::Similar
                | Self::DerivedFrom
        )
    }

    /// Associative edge — weight thay đổi theo Hebbian.
    pub fn is_associative(self) -> bool {
        matches!(self, Self::Assoc | Self::EdgeAssoc | Self::EdgeCausal)
    }

    /// Symbol cho edge kind.
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Member => "∈",
            Self::Subset => "⊂",
            Self::Equiv => "≡",
            Self::Similar => "≈",
            Self::Compose => "∘",
            Self::Causes => "→",
            Self::Orthogonal => "⊥",
            Self::DerivedFrom => "←",
            Self::Assoc => "~",
            Self::EdgeAssoc => "~~",  // liên tưởng
            Self::EdgeCausal => "→→", // nhân quả
            _ => "?",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ModalitySource — nguồn gốc học
// ─────────────────────────────────────────────────────────────────────────────

/// Nguồn gốc khi tạo EdgeAssoc/EdgeCausal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[derive(Default)]
pub enum ModalitySource {
    /// Từ text/ngôn ngữ
    #[default]
    Text = 0x01,
    /// Từ audio/giọng nói
    Audio = 0x02,
    /// Từ hình ảnh/video
    Image = 0x03,
    /// Từ cảm biến sinh học (nhịp tim, nhiệt độ...)
    Bio = 0x04,
    /// Từ nhiều nguồn kết hợp
    Fused = 0x05,
}

impl ModalitySource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Audio => "audio",
            Self::Image => "image",
            Self::Bio => "bio",
            Self::Fused => "fused",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SilkEdge — một edge trong graph
// ─────────────────────────────────────────────────────────────────────────────

/// Một Silk edge giữa 2 nodes.
///
/// Mang EmotionTag của khoảnh khắc co-activation.
/// weight ∈ [0.0, 1.0] — Hebbian strength.
#[derive(Debug, Clone)]
pub struct SilkEdge {
    /// Hash của node nguồn
    pub from_hash: u64,
    /// Hash của node đích
    pub to_hash: u64,
    /// Loại quan hệ
    pub kind: EdgeKind,
    /// Màu cảm xúc lúc edge hình thành
    pub emotion: EmotionTag,
    /// Hebbian weight ∈ [0.0, 1.0]
    pub weight: f32,
    /// Số lần co-activate
    pub fire_count: u32,
    /// Timestamp tạo edge (ns)
    pub created_at: i64,
    /// Timestamp cập nhật cuối (ns)
    pub updated_at: i64,
    /// Nguồn gốc học (text/audio/image/bio)
    pub source: ModalitySource,
    /// Confidence cho EdgeCausal ∈ [0.0, 1.0]
    pub confidence: f32,
}

impl SilkEdge {
    /// Tạo structural edge (weight = 1.0, bất biến).
    pub fn structural(from_hash: u64, to_hash: u64, kind: EdgeKind, ts: i64) -> Self {
        Self {
            from_hash,
            to_hash,
            kind,
            emotion: EmotionTag::NEUTRAL,
            weight: 1.0,
            fire_count: 1,
            created_at: ts,
            updated_at: ts,
            source: ModalitySource::Text,
            confidence: 1.0,
        }
    }

    /// Tạo associative edge với EmotionTag.
    pub fn associative(from_hash: u64, to_hash: u64, emotion: EmotionTag, ts: i64) -> Self {
        Self {
            from_hash,
            to_hash,
            kind: EdgeKind::Assoc,
            emotion,
            source: ModalitySource::Text,
            confidence: 0.0,
            weight: 0.1, // khởi đầu yếu
            fire_count: 1,
            created_at: ts,
            updated_at: ts,
        }
    }

    /// Key để index edge (from, to, kind).
    pub fn key(&self) -> (u64, u64, u8) {
        (self.from_hash, self.to_hash, self.kind.as_byte())
    }

    /// Serialize → 42 bytes.
    ///
    /// [from:8][to:8][kind:1][V:4][A:4][D:4][I:4][weight:4][fire:4][ts:4] = 46 bytes
    pub fn to_bytes(&self) -> [u8; 46] {
        let mut out = [0u8; 46];
        out[0..8].copy_from_slice(&self.from_hash.to_le_bytes());
        out[8..16].copy_from_slice(&self.to_hash.to_le_bytes());
        out[16] = self.kind.as_byte();
        out[17..21].copy_from_slice(&self.emotion.valence.to_le_bytes());
        out[21..25].copy_from_slice(&self.emotion.arousal.to_le_bytes());
        out[25..29].copy_from_slice(&self.emotion.dominance.to_le_bytes());
        out[29..33].copy_from_slice(&self.emotion.intensity.to_le_bytes());
        out[33..37].copy_from_slice(&self.weight.to_le_bytes());
        out[37..41].copy_from_slice(&self.fire_count.to_le_bytes());
        out[41..45].copy_from_slice(&(self.created_at as i32).to_le_bytes());
        out[45] = 0; // padding
        out
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emotion_tag_neutral() {
        let e = EmotionTag::NEUTRAL;
        assert_eq!(e.valence, 0.0);
        assert!(e.arousal > 0.0);
    }

    #[test]
    fn emotion_tag_blend() {
        let a = EmotionTag::new(1.0, 0.8, 0.7, 0.9);
        let b = EmotionTag::new(-1.0, 0.2, 0.3, 0.1);
        let mid = a.blend(b, 0.5);
        assert!((mid.valence - 0.0).abs() < 0.01);
        assert!((mid.arousal - 0.5).abs() < 0.01);
    }

    #[test]
    fn emotion_from_ucd_bytes() {
        // valence=0xFF → +1.0, arousal=0xFF → 1.0
        let e = EmotionTag::from_ucd_bytes(0xFF, 0xFF);
        assert!(e.valence > 0.9, "High valence byte → positive valence");
        assert!(e.arousal > 0.9, "High arousal byte → high arousal");

        // valence=0x00 → -1.0
        let e2 = EmotionTag::from_ucd_bytes(0x00, 0x80);
        assert!(e2.valence < -0.9, "Low valence byte → negative valence");
    }

    #[test]
    fn emotion_distance() {
        let a = EmotionTag::new(1.0, 1.0, 0.5, 0.5);
        let b = EmotionTag::new(-1.0, 0.0, 0.5, 0.5);
        let dist = a.distance_va(&b);
        assert!(dist > 2.0, "Distance (1,-1) to (-1,0) > 2");
        assert_eq!(a.distance_va(&a), 0.0, "Self distance = 0");
    }

    #[test]
    fn edge_kind_roundtrip() {
        for b in [0x01u8, 0x06, 0x0E, 0xFF, 0xF0] {
            let k = EdgeKind::from_byte(b).unwrap();
            assert_eq!(k.as_byte(), b);
        }
    }

    #[test]
    fn edge_kind_structural() {
        assert!(EdgeKind::Member.is_structural());
        assert!(EdgeKind::Causes.is_structural());
        assert!(!EdgeKind::Assoc.is_structural());
        assert!(!EdgeKind::Flows.is_structural());
    }

    #[test]
    fn edge_kind_associative() {
        assert!(EdgeKind::Assoc.is_associative());
        assert!(!EdgeKind::Member.is_associative());
    }

    #[test]
    fn structural_edge_weight_one() {
        let e = SilkEdge::structural(0xA, 0xB, EdgeKind::Member, 1000);
        assert_eq!(e.weight, 1.0, "Structural edge = weight 1.0");
        assert_eq!(e.kind, EdgeKind::Member);
    }

    #[test]
    fn associative_edge_weak_start() {
        let emotion = EmotionTag::new(-0.6, 0.8, 0.3, 0.7);
        let e = SilkEdge::associative(0xA, 0xB, emotion, 1000);
        assert!(e.weight < 0.2, "Associative edge bắt đầu yếu");
        assert_eq!(e.kind, EdgeKind::Assoc);
        assert!((e.emotion.valence - (-0.6)).abs() < 0.001);
    }

    #[test]
    fn edge_serialization_size() {
        let e = SilkEdge::structural(0xDEAD, 0xBEEF, EdgeKind::Causes, 0);
        assert_eq!(e.to_bytes().len(), 46);
    }
}
