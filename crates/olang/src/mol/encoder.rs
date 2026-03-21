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
    // v2: P_weight trực tiếp từ UCD_TABLE (packed [S:4][R:4][V:3][A:3][T:2]).
    // KHÔNG re-pack từ raw u8 — tránh mất precision do quantization.
    let p_weight = ucd::p_weight_of(cp);
    if p_weight != 0 {
        MolecularChain::single(Molecule::from_u16(p_weight))
    } else {
        // Fallback: cp không trong UDC_TABLE (Latin, CJK, etc.)
        // Thử alias table trước, rồi raw values
        let p_alias = ucd::p_weight_full(cp);
        if p_alias != 0 {
            MolecularChain::single(Molecule::from_u16(p_alias))
        } else {
            // Last resort: raw values (có mất precision nhưng vẫn tạo molecule)
            let mol = Molecule::raw(
                ucd::shape_of(cp), ucd::relation_of(cp),
                ucd::valence_of(cp), ucd::arousal_of(cp), ucd::time_of(cp),
            );
            MolecularChain::single(mol)
        }
    }
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
            // Rebuild molecule with new relation, keeping other dimensions.
            // v2: pack() quantizes relation by >>4, so pre-scale: rel<<4.
            // This puts the relation index in the correct 4-bit position.
            Molecule::pack(mol.shape_u8(), new_rel << 4, mol.valence_u8(), mol.arousal_u8(), mol.time_u8()).bits
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
        // Verify encoder produces same result as direct UCD lookup
        let expected = Molecule::from_u16(ucd::p_weight_of(0x1F525));
        assert_eq!(m.shape(), expected.shape(), "FIRE shape from UCD");
        assert_eq!(m.relation(), expected.relation(), "FIRE relation from UCD");
        assert_eq!(m.valence(), expected.valence(), "FIRE valence from UCD");
        assert_eq!(m.arousal(), expected.arousal(), "FIRE arousal from UCD");
        assert_eq!(m.time(), expected.time(), "FIRE time from UCD");
    }

    #[test]
    fn encode_droplet() {
        let chain = encode_codepoint(0x1F4A7); // 💧
        assert_eq!(chain.len(), 1);
        let m = chain.mol_at(0).unwrap();
        // Verify against UCD data
        let expected = Molecule::from_u16(ucd::p_weight_of(0x1F4A7));
        assert_eq!(m.shape(), expected.shape(), "DROPLET shape from UCD");
        assert_eq!(m.valence(), expected.valence(), "DROPLET valence from UCD");
        assert_eq!(m.arousal(), expected.arousal(), "DROPLET arousal from UCD");
    }

    #[test]
    fn encode_sphere_sdf() {
        let chain = encode_codepoint(0x25CF); // ●
        let m = chain.mol_at(0).unwrap();
        // Verify against UCD data
        let expected = Molecule::from_u16(ucd::p_weight_of(0x25CF));
        assert_eq!(m.shape(), expected.shape(), "SDF shape from UCD");
        assert_eq!(m.time(), expected.time(), "SDF time from UCD");
    }

    #[test]
    fn encode_arrow_causes() {
        let chain = encode_codepoint(0x2192); // →
        let m = chain.mol_at(0).unwrap();
        // Verify against UCD data
        let expected = Molecule::from_u16(ucd::p_weight_of(0x2192));
        assert_eq!(m.relation(), expected.relation(), "Arrow relation from UCD");
        assert_eq!(m.time(), expected.time(), "Arrow time from UCD");
    }

    #[test]
    fn encode_member_relation() {
        let chain = encode_codepoint(0x2208); // ∈
        let m = chain.mol_at(0).unwrap();
        // Verify against UCD data
        let expected = Molecule::from_u16(ucd::p_weight_of(0x2208));
        assert_eq!(m.relation(), expected.relation(), "Member relation from UCD");
        assert_eq!(m.time(), expected.time(), "Member time from UCD");
    }

    #[test]
    fn encode_zwj_family() {
        // 👨‍👩‍👦 = U+1F468 ZWJ U+1F469 ZWJ U+1F466
        let chain = encode_zwj_sequence(&[0x1F468, 0x1F469, 0x1F466]);
        assert_eq!(chain.len(), 3);
        // ZWJ: first N-1 get Compose relation, last gets Member
        assert_eq!(
            chain.mol_at(0).unwrap().relation(),
            RelationBase::Compose.as_byte(),
            "mol[0] relation = Compose (quantized index)"
        );
        assert_eq!(
            chain.mol_at(1).unwrap().relation(),
            RelationBase::Compose.as_byte(),
            "mol[1] relation = Compose (quantized index)"
        );
        assert_eq!(
            chain.mol_at(2).unwrap().relation(),
            RelationBase::Member.as_byte(),
            "mol[2] relation = Member (quantized index)"
        );
    }

    #[test]
    fn encode_zwj_single() {
        let chain = encode_zwj_sequence(&[0x1F525]);
        assert_eq!(chain.len(), 1);
        // Single element → no ZWJ processing, returned as-is from encode_codepoint
        // Relation comes from UCD data, not forced
        let expected = encode_codepoint(0x1F525);
        assert_eq!(chain.mol_at(0).unwrap().relation(), expected.mol_at(0).unwrap().relation());
    }

    #[test]
    fn encode_flag_vietnam() {
        // 🇻🇳 = U+1F1FB (V) + U+1F1F3 (N)
        // QT4: encode_flag delegates to encode_zwj_sequence — no hardcoded Molecule
        let chain = encode_flag(0x1F1FB, 0x1F1F3);
        assert_eq!(chain.len(), 2);
        // ZWJ-like: first mol.relation = Compose, last = Member
        assert_eq!(
            chain.mol_at(0).unwrap().relation(),
            RelationBase::Compose.as_byte(),
            "first RI = Compose"
        );
        assert_eq!(
            chain.mol_at(1).unwrap().relation(),
            RelationBase::Member.as_byte(),
            "last RI = Member"
        );
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
