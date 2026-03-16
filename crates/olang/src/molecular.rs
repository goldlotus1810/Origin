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
            _ => None,
        }
    }

    /// Byte value.
    pub fn as_byte(self) -> u8 {
        self as u8
    }
}

/// Chiều quan hệ — từ RELATION group (Math Operators 2200..22FF).
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
            _ => None,
        }
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

/// Chiều thời gian — từ MUSICAL group (note duration).
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
    /// Parse từ byte.
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
/// Mọi Molecule đến từ `encoder::encode_codepoint()`.
/// Không bao giờ tạo Molecule struct literal trong code production.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Molecule {
    /// Chiều hình dạng (Shape byte)
    pub shape: ShapeBase,
    /// Chiều quan hệ (Relation byte)
    pub relation: RelationBase,
    /// Chiều cảm xúc (Valence + Arousal bytes)
    pub emotion: EmotionDim,
    /// Chiều thời gian (Time byte)
    pub time: TimeDim,
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
            shape: ShapeBase::from_byte(b[0])?,
            relation: RelationBase::from_byte(b[1])?,
            emotion: EmotionDim {
                valence: b[2],
                arousal: b[3],
            },
            time: TimeDim::from_byte(b[4])?,
        })
    }

    /// Presence mask — bit nào bật = dimension đó ≠ default.
    ///
    /// Dùng bởi tagged encoding để biết fields nào cần ghi.
    pub fn presence_mask(&self) -> u8 {
        let mut mask = 0u8;
        if self.shape.as_byte() != TAGGED_DEFAULT_SHAPE {
            mask |= PRESENT_SHAPE;
        }
        if self.relation.as_byte() != TAGGED_DEFAULT_RELATION {
            mask |= PRESENT_RELATION;
        }
        if self.emotion.valence != TAGGED_DEFAULT_VALENCE {
            mask |= PRESENT_VALENCE;
        }
        if self.emotion.arousal != TAGGED_DEFAULT_AROUSAL {
            mask |= PRESENT_AROUSAL;
        }
        if self.time.as_byte() != TAGGED_DEFAULT_TIME {
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
            out.push(self.shape.as_byte());
        }
        if mask & PRESENT_RELATION != 0 {
            out.push(self.relation.as_byte());
        }
        if mask & PRESENT_VALENCE != 0 {
            out.push(self.emotion.valence);
        }
        if mask & PRESENT_AROUSAL != 0 {
            out.push(self.emotion.arousal);
        }
        if mask & PRESENT_TIME != 0 {
            out.push(self.time.as_byte());
        }
        out
    }

    /// Deserialize từ tagged bytes.
    ///
    /// Returns `(Molecule, bytes_consumed)`. Absent fields → defaults.
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
            let s = ShapeBase::from_byte(b[idx])?;
            idx += 1;
            s
        } else {
            ShapeBase::from_byte(TAGGED_DEFAULT_SHAPE)?
        };
        let relation = if mask & PRESENT_RELATION != 0 {
            let r = RelationBase::from_byte(b[idx])?;
            idx += 1;
            r
        } else {
            RelationBase::from_byte(TAGGED_DEFAULT_RELATION)?
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
            let t = TimeDim::from_byte(b[idx])?;
            idx += 1;
            t
        } else {
            TimeDim::from_byte(TAGGED_DEFAULT_TIME)?
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
    /// Dựa trên structural overlap (shape + relation match).
    /// O(n×m) — chains ngắn trong thực tế (1-10 molecules).
    pub fn similarity(&self, other: &Self) -> f32 {
        if self.is_empty() || other.is_empty() {
            return 0.0;
        }
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
        if self.is_empty() || other.is_empty() {
            return 0.0;
        }
        let n = self.0.len().min(other.0.len());
        let mut total = 0.0f32;
        for i in 0..n {
            let a = &self.0[i];
            let b = &other.0[i];
            let shape_m = if a.shape == b.shape { 1.0f32 } else { 0.0 };
            let rel_m = if a.relation == b.relation {
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

    // ── Numeric encoding ─────────────────────────────────────────────────

    /// Encode f64 → 4-molecule chain.
    ///
    /// Marker: shape=Sphere, relation=Equiv, time=Static (signals "number").
    /// 8 bytes of f64 stored in valence+arousal of 4 molecules (2 bytes each).
    pub fn from_number(n: f64) -> Self {
        let bits = n.to_bits().to_le_bytes();
        let mut mols = Vec::with_capacity(4);
        for chunk in bits.chunks(2) {
            mols.push(Molecule {
                shape: ShapeBase::Sphere,
                relation: RelationBase::Equiv,
                emotion: EmotionDim {
                    valence: chunk[0],
                    arousal: chunk[1],
                },
                time: TimeDim::Static,
            });
        }
        Self(mols)
    }

    /// Decode chain → f64 if it's a numeric chain.
    ///
    /// Returns Some(f64) if chain is exactly 4 molecules with
    /// shape=Sphere, relation=Equiv, time=Static (numeric marker).
    pub fn to_number(&self) -> Option<f64> {
        if self.0.len() != 4 {
            return None;
        }
        // Check all molecules have numeric marker
        for m in &self.0 {
            if m.shape != ShapeBase::Sphere
                || m.relation != RelationBase::Equiv
                || m.time != TimeDim::Static
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
            shape: ShapeBase::from_byte(shape).unwrap(),
            relation: RelationBase::from_byte(relation).unwrap(),
            emotion: EmotionDim {
                valence: v,
                arousal: a,
            },
            time: TimeDim::from_byte(t).unwrap(),
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
        for s in 0x01u8..=0x08 {
            for r in 0x01u8..=0x08 {
                let bytes = [s, r, 0x7F, 0x80, 0x03u8];
                let m = Molecule::from_bytes(&bytes).unwrap();
                assert_eq!(m.to_bytes()[0], s);
                assert_eq!(m.to_bytes()[1], r);
            }
        }
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
}

impl Default for MolecularChain {
    fn default() -> Self {
        MolecularChain::empty()
    }
}
