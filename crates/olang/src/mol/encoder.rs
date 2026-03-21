//! # encoder — codepoint → MolecularChain từ UCD
//!
//! Đây là cách DUY NHẤT tạo MolecularChain trong production code.
//! Không hardcode bất kỳ Molecule nào.

extern crate alloc;
use alloc::vec::Vec;

use crate::molecular::{MolecularChain, Molecule, RelationBase};

/// Encode một codepoint Unicode → MolecularChain (1 molecule).
///
/// Tất cả 5 chiều đến từ UCD lookup — không hardcode.
/// Raw hierarchical bytes giữ nguyên từ UCD → phân biệt ~5400 mẫu.
/// Đây là hàm gốc của mọi chain trong HomeOS.
pub fn encode_codepoint(cp: u32) -> MolecularChain {
    let shape = ucd::shape_of(cp);
    let relation = ucd::relation_of(cp);
    let valence = ucd::valence_of(cp);
    let arousal = ucd::arousal_of(cp);
    let time = ucd::time_of(cp);

    let mol = Molecule::raw(shape, relation, valence, arousal, time);
    // v2: P_weight trực tiếp từ udc.json, không cần formula rule IDs.
    // Molecule.fs/fr/fv/fa/ft giữ 0xFF (UNSET) — metadata runtime only.

    MolecularChain::single(mol)
}

/// Encode ZWJ sequence → MolecularChain (N molecules).
///
/// Quy tắc:
///   mol[0..N-2].relation = ∘ (Compose — còn tiếp)
///   mol[N-1].relation    = ∈ (Member  — kết thúc)
///
/// Ví dụ: 👨‍👩‍👦 → [mol(👨,∘), mol(👩,∘), mol(👦,∈)]
pub fn encode_zwj_sequence(codepoints: &[u32]) -> MolecularChain {
    if codepoints.is_empty() {
        return MolecularChain::empty();
    }
    if codepoints.len() == 1 {
        return encode_codepoint(codepoints[0]);
    }

    let last = codepoints.len() - 1;
    let bits_vec: Vec<u16> = codepoints
        .iter()
        .enumerate()
        .map(|(i, &cp)| {
            let mol = encode_codepoint(cp).first().unwrap();
            let new_rel = if i < last {
                RelationBase::Compose.as_byte() // ∘ — còn tiếp
            } else {
                RelationBase::Member.as_byte() // ∈ — kết thúc
            };
            // Rebuild molecule with new relation, keeping other dimensions
            Molecule::pack(mol.shape_u8(), new_rel, mol.valence_u8(), mol.arousal_u8(), mol.time_u8()).bits
        })
        .collect();

    MolecularChain(bits_vec)
}

/// Encode flag sequence (Regional Indicator pair) → 2 molecules.
///
/// Ví dụ: 🇻🇳 = U+1F1FB (V) + U+1F1F3 (N)
/// Dùng encode_codepoint() cho từng RI — KHÔNG hardcode Molecule.
pub fn encode_flag(ri1: u32, ri2: u32) -> MolecularChain {
    // QT4: mọi Molecule từ encode_codepoint() — ZWJ-like sequence
    encode_zwj_sequence(&[ri1, ri2])
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecular::{ShapeBase, TimeDim};

    #[test]
    fn encode_fire() {
        let chain = encode_codepoint(0x1F525); // 🔥
        assert_eq!(chain.len(), 1);
        let m = chain.mol_at(0).unwrap();
        assert_eq!(m.shape_base(), ShapeBase::Sphere, "FIRE shape = Sphere");
        assert_eq!(
            m.relation_base(),
            RelationBase::Member,
            "FIRE relation = Member"
        );
        assert!(m.valence_u8() >= 0xC0, "FIRE valence cao");
        assert!(m.arousal_u8() >= 0xC0, "FIRE arousal cao");
        assert_eq!(m.time_base(), TimeDim::Fast, "FIRE time = Fast");
    }

    #[test]
    fn encode_droplet() {
        let chain = encode_codepoint(0x1F4A7); // 💧
        assert_eq!(chain.len(), 1);
        let m = chain.mol_at(0).unwrap();
        assert!(m.valence_u8() >= 0x80, "DROPLET valence moderate");
        assert!(m.arousal_u8() <= 0x80, "DROPLET arousal thấp");
        assert_eq!(m.time_base(), TimeDim::Slow, "DROPLET time = Slow");
    }

    #[test]
    fn encode_sphere_sdf() {
        let chain = encode_codepoint(0x25CF); // ●
        assert_eq!(chain.mol_at(0).unwrap().shape_base(), ShapeBase::Sphere);
        assert_eq!(chain.mol_at(0).unwrap().time_base(), TimeDim::Static, "SDF shapes = Static");
    }

    #[test]
    fn encode_arrow_causes() {
        let chain = encode_codepoint(0x2192); // →
        assert_eq!(chain.mol_at(0).unwrap().relation_base(), RelationBase::Causes);
        assert_eq!(chain.mol_at(0).unwrap().time_base(), TimeDim::Instant, "Arrow = Instant");
    }

    #[test]
    fn encode_member_relation() {
        let chain = encode_codepoint(0x2208); // ∈
        assert_eq!(chain.mol_at(0).unwrap().relation_base(), RelationBase::Member);
        assert_eq!(chain.mol_at(0).unwrap().time_base(), TimeDim::Static, "Math = Static");
    }

    #[test]
    fn encode_zwj_family() {
        // 👨‍👩‍👦 = U+1F468 ZWJ U+1F469 ZWJ U+1F466
        let chain = encode_zwj_sequence(&[0x1F468, 0x1F469, 0x1F466]);
        assert_eq!(chain.len(), 3);
        assert_eq!(
            chain.mol_at(0).unwrap().relation_base(),
            RelationBase::Compose,
            "mol[0] = Compose"
        );
        assert_eq!(
            chain.mol_at(1).unwrap().relation_base(),
            RelationBase::Compose,
            "mol[1] = Compose"
        );
        assert_eq!(
            chain.mol_at(2).unwrap().relation_base(),
            RelationBase::Member,
            "mol[2] = Member"
        );
    }

    #[test]
    fn encode_zwj_single() {
        let chain = encode_zwj_sequence(&[0x1F525]);
        assert_eq!(chain.len(), 1);
        // Single = Member (kết thúc ngay)
        assert_eq!(chain.mol_at(0).unwrap().relation_base(), RelationBase::Member);
    }

    #[test]
    fn encode_flag_vietnam() {
        // 🇻🇳 = U+1F1FB (V) + U+1F1F3 (N)
        // QT4: encode_flag delegates to encode_zwj_sequence — no hardcoded Molecule
        let chain = encode_flag(0x1F1FB, 0x1F1F3);
        assert_eq!(chain.len(), 2);
        // ZWJ-like: first mol.relation = Compose, last = Member
        assert_eq!(chain.mol_at(0).unwrap().relation_base(), RelationBase::Compose);
        assert_eq!(chain.mol_at(1).unwrap().relation_base(), RelationBase::Member);
    }

    #[test]
    fn encode_different_cps_different_chains() {
        let fire = encode_codepoint(0x1F525);
        let water = encode_codepoint(0x1F4A7);
        // Phải khác nhau ít nhất ở emotion
        assert!(
            fire.similarity_full(&water) < 1.0,
            "🔥 và 💧 phải có similarity < 1.0"
        );
    }

    #[test]
    fn encode_no_hardcode_verify() {
        // Verify chain đến từ UCD — so sánh với UCD trực tiếp
        // Values are quantized during pack(), so compare packed results
        let cp = 0x1F525u32;
        let chain = encode_codepoint(cp);
        let m = chain.mol_at(0).unwrap();
        let expected = Molecule::pack(
            ucd::shape_of(cp),
            ucd::relation_of(cp),
            ucd::valence_of(cp),
            ucd::arousal_of(cp),
            ucd::time_of(cp),
        );
        assert_eq!(m.shape(), expected.shape());
        assert_eq!(m.relation(), expected.relation());
        assert_eq!(m.valence(), expected.valence());
        assert_eq!(m.arousal(), expected.arousal());
        assert_eq!(m.time(), expected.time());
    }
}
