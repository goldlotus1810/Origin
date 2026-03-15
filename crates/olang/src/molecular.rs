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

/// Chiều hình dạng — từ SDF group (Geometric Shapes 25A0..25FF).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ShapeBase {
    /// ● U+25CF Sphere
    Sphere    = 0x01,
    /// ▬ U+25AC Capsule
    Capsule   = 0x02,
    /// ■ U+25A0 Box
    Box       = 0x03,
    /// ▲ U+25B2 Cone
    Cone      = 0x04,
    /// ○ U+25CB Torus
    Torus     = 0x05,
    /// ∪ U+222A Union
    Union     = 0x06,
    /// ∩ U+2229 Intersect
    Intersect = 0x07,
    /// ∖ U+2216 Subtract
    Subtract  = 0x08,
}

impl ShapeBase {
    /// Parse từ byte.
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
            _    => None,
        }
    }

    /// Byte value.
    pub fn as_byte(self) -> u8 { self as u8 }
}

/// Chiều quan hệ — từ RELATION group (Math Operators 2200..22FF).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RelationBase {
    /// ∈ U+2208 Member
    Member      = 0x01,
    /// ⊂ U+2282 Subset
    Subset      = 0x02,
    /// ≡ U+2261 Equiv
    Equiv       = 0x03,
    /// ⊥ U+22A5 Orthogonal
    Orthogonal  = 0x04,
    /// ∘ U+2218 Compose
    Compose     = 0x05,
    /// → U+2192 Causes
    Causes      = 0x06,
    /// ≈ U+2248 Similar
    Similar     = 0x07,
    /// ← U+2190 DerivedFrom
    DerivedFrom = 0x08,
}

impl RelationBase {
    /// Parse từ byte.
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
            _    => None,
        }
    }

    /// Byte value.
    pub fn as_byte(self) -> u8 { self as u8 }
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
    pub const NEUTRAL: Self = Self { valence: 0x7F, arousal: 0x80 };
}

/// Chiều thời gian — từ MUSICAL group (note duration).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TimeDim {
    /// 𝅝 Whole note — Largo
    Static  = 0x01,
    /// 𝅗 Half note — Adagio
    Slow    = 0x02,
    /// ♩ Quarter note — Andante
    Medium  = 0x03,
    /// ♪ Eighth note — Allegro
    Fast    = 0x04,
    /// 16th note — Presto
    Instant = 0x05,
}

impl TimeDim {
    /// Parse từ byte.
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Static),
            0x02 => Some(Self::Slow),
            0x03 => Some(Self::Medium),
            0x04 => Some(Self::Fast),
            0x05 => Some(Self::Instant),
            _    => None,
        }
    }

    /// Byte value.
    pub fn as_byte(self) -> u8 { self as u8 }
}

// ─────────────────────────────────────────────────────────────────────────────
// Molecule — 5 bytes
// ─────────────────────────────────────────────────────────────────────────────

/// Đơn vị thông tin cơ bản — **5 bytes**.
///
/// Wire format: `[shape][relation][valence][arousal][time]`
///
/// Mọi Molecule đến từ `encoder::encode_codepoint()`.
/// Không bao giờ tạo Molecule struct literal trong code production.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Molecule {
    /// Chiều hình dạng (Shape byte)
    pub shape:    ShapeBase,
    /// Chiều quan hệ (Relation byte)
    pub relation: RelationBase,
    /// Chiều cảm xúc (Valence + Arousal bytes)
    pub emotion:  EmotionDim,
    /// Chiều thời gian (Time byte)
    pub time:     TimeDim,
}

impl Molecule {
    /// Serialize → 5 bytes.
    pub fn to_bytes(self) -> [u8; 5] {
        [
            self.shape.as_byte(),
            self.relation.as_byte(),
            self.emotion.valence,
            self.emotion.arousal,
            self.time.as_byte(),
        ]
    }

    /// Deserialize từ 5 bytes.
    pub fn from_bytes(b: &[u8; 5]) -> Option<Self> {
        Some(Self {
            shape:    ShapeBase::from_byte(b[0])?,
            relation: RelationBase::from_byte(b[1])?,
            emotion:  EmotionDim { valence: b[2], arousal: b[3] },
            time:     TimeDim::from_byte(b[4])?,
        })
    }

    /// Điểm tương đồng giữa 2 molecules ∈ [0, 5].
    pub fn match_score(&self, other: &Self) -> u8 {
        let mut s = 0u8;
        if self.shape    == other.shape    { s += 1; }
        if self.relation == other.relation { s += 1; }
        if self.time     == other.time     { s += 1; }
        // Valence: gần nhau trong [-32, +32] → điểm
        let vd = self.emotion.valence.abs_diff(other.emotion.valence);
        if vd < 32 { s += 1; }
        // Arousal tương tự
        let ad = self.emotion.arousal.abs_diff(other.emotion.arousal);
        if ad < 32 { s += 1; }
        s
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
    pub fn empty() -> Self { Self(Vec::new()) }

    /// Chain từ 1 molecule.
    pub fn single(m: Molecule) -> Self { Self(alloc::vec![m]) }

    /// Số molecules.
    pub fn len(&self) -> usize { self.0.len() }

    /// Chain có rỗng không.
    pub fn is_empty(&self) -> bool { self.0.is_empty() }

    /// Molecule đầu tiên.
    pub fn first(&self) -> Option<&Molecule> { self.0.first() }

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
        if b.len() % 5 != 0 { return None; }
        let mut ms = Vec::with_capacity(b.len() / 5);
        for chunk in b.chunks_exact(5) {
            let arr: [u8; 5] = chunk.try_into().unwrap();
            ms.push(Molecule::from_bytes(&arr)?);
        }
        Some(Self(ms))
    }

    /// FNV-1a hash — dùng trong Registry và reverse index.
    pub fn chain_hash(&self) -> u64 {
        const OFFSET: u64 = 0xcbf29ce484222325;
        const PRIME:  u64 = 0x100000001b3;
        let mut h = OFFSET;
        for b in self.to_bytes() {
            h ^= b as u64;
            h  = h.wrapping_mul(PRIME);
        }
        h
    }

    /// Similarity với chain khác ∈ [0.0, 1.0].
    ///
    /// Dựa trên structural overlap (shape + relation match).
    /// O(n×m) — chains ngắn trong thực tế (1-10 molecules).
    pub fn similarity(&self, other: &Self) -> f32 {
        if self.is_empty() || other.is_empty() { return 0.0; }
        let mut overlap = 0usize;
        for a in &self.0 {
            for b in &other.0 {
                if a.shape == b.shape && a.relation == b.relation {
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
    pub fn similarity_full(&self, other: &Self) -> f32 {
        if self.is_empty() || other.is_empty() { return 0.0; }
        let n = self.0.len().min(other.0.len());
        let mut total = 0.0f32;
        for i in 0..n {
            let a = &self.0[i];
            let b = &other.0[i];
            let shape_m = if a.shape    == b.shape    { 1.0f32 } else { 0.0 };
            let rel_m   = if a.relation == b.relation { 1.0f32 } else { 0.0 };
            let vd = a.emotion.valence.abs_diff(b.emotion.valence) as f32;
            let ad = a.emotion.arousal.abs_diff(b.emotion.arousal) as f32;
            let emo_sim = 1.0 - (vd + ad) / 510.0;
            total += 0.3 * shape_m + 0.2 * rel_m + 0.5 * emo_sim;
        }
        total / n as f32
    }

    /// Nối 2 chains.
    pub fn concat(&self, other: &Self) -> Self {
        let mut v = self.0.clone();
        v.extend_from_slice(&other.0);
        Self(v)
    }

    /// Thêm molecule vào cuối.
    pub fn push(&mut self, m: Molecule) { self.0.push(m); }
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
            shape:    ShapeBase::from_byte(shape).unwrap(),
            relation: RelationBase::from_byte(relation).unwrap(),
            emotion:  EmotionDim { valence: v, arousal: a },
            time:     TimeDim::from_byte(t).unwrap(),
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
    fn molecule_invalid_shape() {
        let bytes = [0x00, 0x01, 0x80, 0x80, 0x03]; // shape=0x00 invalid
        assert!(Molecule::from_bytes(&bytes).is_none());
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

    #[test]
    fn fuzz_all_valid_shapes_relations() {
        for s in 0x01u8..=0x08 {
            for r in 0x01u8..=0x08 {
                let bytes = [s, r, 0x7F, 0x80, 0x03u8];
                let m = Molecule::from_bytes(&bytes).unwrap();
                assert_eq!(m.to_bytes()[0], s);
                assert_eq!(m.to_bytes()[1], r);
            }
        }
    }
}

impl Default for MolecularChain {
    fn default() -> Self { MolecularChain::empty() }
}
