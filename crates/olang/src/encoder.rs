//! # encoder — codepoint → MolecularChain từ UCD
//!
//! Đây là cách DUY NHẤT tạo MolecularChain trong production code.
//! Không hardcode bất kỳ Molecule nào.

extern crate alloc;
use alloc::vec::Vec;

use crate::molecular::{
    Molecule, MolecularChain,
    ShapeBase, RelationBase, EmotionDim, TimeDim,
};

/// Encode một codepoint Unicode → MolecularChain (1 molecule).
///
/// Tất cả 5 chiều đến từ UCD lookup — không hardcode.
/// Đây là hàm gốc của mọi chain trong HomeOS.
pub fn encode_codepoint(cp: u32) -> MolecularChain {
    let shape    = shape_byte(ucd::shape_of(cp));
    let relation = relation_byte(ucd::relation_of(cp));
    let valence  = ucd::valence_of(cp);
    let arousal  = ucd::arousal_of(cp);
    let time     = time_byte(ucd::time_of(cp));

    MolecularChain::single(Molecule {
        shape,
        relation,
        emotion: EmotionDim { valence, arousal },
        time,
    })
}

/// Encode ZWJ sequence → MolecularChain (N molecules).
///
/// Quy tắc:
///   mol[0..N-2].relation = ∘ (Compose — còn tiếp)
///   mol[N-1].relation    = ∈ (Member  — kết thúc)
///
/// Ví dụ: 👨‍👩‍👦 → [mol(👨,∘), mol(👩,∘), mol(👦,∈)]
pub fn encode_zwj_sequence(codepoints: &[u32]) -> MolecularChain {
    if codepoints.is_empty() { return MolecularChain::empty(); }
    if codepoints.len() == 1 { return encode_codepoint(codepoints[0]); }

    let last = codepoints.len() - 1;
    let molecules: Vec<Molecule> = codepoints.iter().enumerate().map(|(i, &cp)| {
        let mut mol = encode_codepoint(cp).0.remove(0);
        mol.relation = if i < last {
            RelationBase::Compose // ∘ — còn tiếp
        } else {
            RelationBase::Member  // ∈ — kết thúc
        };
        mol
    }).collect();

    MolecularChain(molecules)
}

/// Encode flag sequence (Regional Indicator pair) → 2 molecules.
///
/// Ví dụ: 🇻🇳 = U+1F1FB (V) + U+1F1F3 (N)
pub fn encode_flag(ri1: u32, ri2: u32) -> MolecularChain {
    let letter1 = ri1.saturating_sub(0x1F1E6) as u8;
    let letter2 = ri2.saturating_sub(0x1F1E6) as u8;

    let m1 = Molecule {
        shape:    ShapeBase::Box,
        relation: RelationBase::Compose,
        emotion:  EmotionDim {
            valence: 0x80u8.saturating_add(letter1.saturating_mul(2)),
            arousal: 0x60,
        },
        time: TimeDim::Static,
    };
    let m2 = Molecule {
        shape:    ShapeBase::Box,
        relation: RelationBase::Member,
        emotion:  EmotionDim {
            valence: 0x80u8.saturating_add(letter2.saturating_mul(2)),
            arousal: 0x60,
        },
        time: TimeDim::Static,
    };
    MolecularChain(alloc::vec![m1, m2])
}

// ─────────────────────────────────────────────────────────────────────────────
// Byte → Enum
// ─────────────────────────────────────────────────────────────────────────────

fn shape_byte(b: u8) -> ShapeBase {
    ShapeBase::from_byte(b).unwrap_or(ShapeBase::Sphere)
}

fn relation_byte(b: u8) -> RelationBase {
    RelationBase::from_byte(b).unwrap_or(RelationBase::Member)
}

fn time_byte(b: u8) -> TimeDim {
    TimeDim::from_byte(b).unwrap_or(TimeDim::Medium)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_fire() {
        if ucd::table_len() == 0 { return; }
        let chain = encode_codepoint(0x1F525); // 🔥
        assert_eq!(chain.len(), 1);
        let m = &chain.0[0];
        assert_eq!(m.shape, ShapeBase::Sphere,    "FIRE shape = Sphere");
        assert_eq!(m.relation, RelationBase::Member, "FIRE relation = Member");
        assert!(m.emotion.valence >= 0xC0, "FIRE valence cao");
        assert!(m.emotion.arousal >= 0xC0, "FIRE arousal cao");
        assert_eq!(m.time, TimeDim::Fast, "FIRE time = Fast");
    }

    #[test]
    fn encode_droplet() {
        if ucd::table_len() == 0 { return; }
        let chain = encode_codepoint(0x1F4A7); // 💧
        assert_eq!(chain.len(), 1);
        let m = &chain.0[0];
        assert!(m.emotion.valence >= 0x80, "DROPLET valence moderate");
        assert!(m.emotion.arousal <= 0x80, "DROPLET arousal thấp");
        assert_eq!(m.time, TimeDim::Slow, "DROPLET time = Slow");
    }

    #[test]
    fn encode_sphere_sdf() {
        if ucd::table_len() == 0 { return; }
        let chain = encode_codepoint(0x25CF); // ●
        assert_eq!(chain.0[0].shape, ShapeBase::Sphere);
        assert_eq!(chain.0[0].time, TimeDim::Static, "SDF shapes = Static");
    }

    #[test]
    fn encode_arrow_causes() {
        if ucd::table_len() == 0 { return; }
        let chain = encode_codepoint(0x2192); // →
        assert_eq!(chain.0[0].relation, RelationBase::Causes);
        assert_eq!(chain.0[0].time, TimeDim::Instant, "Arrow = Instant");
    }

    #[test]
    fn encode_member_relation() {
        if ucd::table_len() == 0 { return; }
        let chain = encode_codepoint(0x2208); // ∈
        assert_eq!(chain.0[0].relation, RelationBase::Member);
        assert_eq!(chain.0[0].time, TimeDim::Static, "Math = Static");
    }

    #[test]
    fn encode_zwj_family() {
        if ucd::table_len() == 0 { return; }
        // 👨‍👩‍👦 = U+1F468 ZWJ U+1F469 ZWJ U+1F466
        let chain = encode_zwj_sequence(&[0x1F468, 0x1F469, 0x1F466]);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain.0[0].relation, RelationBase::Compose, "mol[0] = Compose");
        assert_eq!(chain.0[1].relation, RelationBase::Compose, "mol[1] = Compose");
        assert_eq!(chain.0[2].relation, RelationBase::Member,  "mol[2] = Member");
    }

    #[test]
    fn encode_zwj_single() {
        if ucd::table_len() == 0 { return; }
        let chain = encode_zwj_sequence(&[0x1F525]);
        assert_eq!(chain.len(), 1);
        // Single = Member (kết thúc ngay)
        assert_eq!(chain.0[0].relation, RelationBase::Member);
    }

    #[test]
    fn encode_flag_vietnam() {
        // 🇻🇳 = U+1F1FB (V) + U+1F1F3 (N)
        let chain = encode_flag(0x1F1FB, 0x1F1F3);
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.0[0].shape, ShapeBase::Box);
        assert_eq!(chain.0[0].relation, RelationBase::Compose);
        assert_eq!(chain.0[1].relation, RelationBase::Member);
    }

    #[test]
    fn encode_different_cps_different_chains() {
        if ucd::table_len() == 0 { return; }
        let fire  = encode_codepoint(0x1F525);
        let water = encode_codepoint(0x1F4A7);
        // Phải khác nhau ít nhất ở emotion
        assert!(fire.similarity_full(&water) < 1.0,
            "🔥 và 💧 phải có similarity < 1.0");
    }

    #[test]
    fn encode_no_hardcode_verify() {
        if ucd::table_len() == 0 { return; }
        // Verify chain đến từ UCD — so sánh với UCD trực tiếp
        let cp = 0x1F525u32;
        let chain = encode_codepoint(cp);
        let m = &chain.0[0];
        assert_eq!(m.shape.as_byte(),    ucd::shape_of(cp));
        assert_eq!(m.relation.as_byte(), ucd::relation_of(cp));
        assert_eq!(m.emotion.valence,    ucd::valence_of(cp));
        assert_eq!(m.emotion.arousal,    ucd::arousal_of(cp));
        assert_eq!(m.time.as_byte(),     ucd::time_of(cp));
    }
}
