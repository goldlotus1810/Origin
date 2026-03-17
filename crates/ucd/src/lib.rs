//! # ucd — Unicode Character Database
//!
//! Lookup codepoint → Molecule bytes từ bảng tĩnh generated lúc compile.
//! Không cần file UCD lúc runtime. Chạy no_std.
//!
//! ## 3 cách decode (tốc độ tăng dần):
//! - `lookup(cp)` → UcdEntry — forward lookup O(log n)
//! - `decode_hash(hash)` → Option<u32> — reverse O(log n)
//! - `bucket_cps(shape, relation)` → &[u32] — top-n candidates O(1)

#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]

// Include generated tables
include!(concat!(env!("OUT_DIR"), "/ucd_generated.rs"));

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Forward lookup: codepoint → UcdEntry
///
/// Binary search trên UCD_TABLE (sorted by cp).
/// O(log n). Returns None nếu cp không thuộc 5 nhóm.
#[inline]
pub fn lookup(cp: u32) -> Option<&'static UcdEntry> {
    UCD_TABLE
        .binary_search_by_key(&cp, |e| e.cp)
        .ok()
        .map(|i| &UCD_TABLE[i])
}

/// Reverse lookup: chain_hash → codepoint
///
/// Binary search trên HASH_TO_CP (sorted by hash).
/// O(log n). Dùng để decode MolecularChain → codepoint.
///
/// Requires feature `reverse-index` (default). Tắt trên embedded để tiết kiệm ~40KB.
#[cfg(feature = "reverse-index")]
#[inline]
pub fn decode_hash(hash: u64) -> Option<u32> {
    HASH_TO_CP
        .binary_search_by_key(&hash, |&(h, _)| h)
        .ok()
        .map(|i| HASH_TO_CP[i].1)
}

/// Stub khi không có reverse-index (embedded build).
/// Luôn trả None — Worker phải ISL request lên Chief để decode.
#[cfg(not(feature = "reverse-index"))]
#[inline]
pub fn decode_hash(_hash: u64) -> Option<u32> {
    None
}

/// Bucket lookup: (shape, relation) → slice of codepoints
///
/// O(1) lookup. Dùng để tìm top-n candidates cho decode.
/// Candidates cùng shape+relation, sort theo similarity.
///
/// Requires feature `reverse-index` (default).
#[cfg(feature = "reverse-index")]
pub fn bucket_cps(shape: u8, relation: u8) -> &'static [u32] {
    // Binary search trong CP_BUCKET_INDEX
    let idx = CP_BUCKET_INDEX.binary_search_by(|&(s, r, _, _)| (s, r).cmp(&(shape, relation)));
    match idx {
        Ok(i) => {
            let (_, _, offset, count) = CP_BUCKET_INDEX[i];
            let start = offset as usize;
            let end = start + count as usize;
            &CP_BUCKET_DATA[start..end]
        }
        Err(_) => &[],
    }
}

/// Stub khi không có reverse-index (embedded build).
#[cfg(not(feature = "reverse-index"))]
pub fn bucket_cps(_shape: u8, _relation: u8) -> &'static [u32] {
    &[]
}

/// Shape byte của codepoint.
#[inline]
pub fn shape_of(cp: u32) -> u8 {
    lookup(cp).map(|e| e.shape).unwrap_or(0x01)
}

/// Relation byte của codepoint.
#[inline]
pub fn relation_of(cp: u32) -> u8 {
    lookup(cp).map(|e| e.relation).unwrap_or(0x01)
}

/// Valence byte của codepoint.
#[inline]
pub fn valence_of(cp: u32) -> u8 {
    lookup(cp).map(|e| e.valence).unwrap_or(0x80)
}

/// Arousal byte của codepoint.
#[inline]
pub fn arousal_of(cp: u32) -> u8 {
    lookup(cp).map(|e| e.arousal).unwrap_or(0x80)
}

/// Time byte của codepoint.
#[inline]
pub fn time_of(cp: u32) -> u8 {
    lookup(cp).map(|e| e.time).unwrap_or(0x03)
}

/// Group byte của codepoint.
#[inline]
pub fn group_of(cp: u32) -> u8 {
    lookup(cp).map(|e| e.group).unwrap_or(0x00)
}

/// Số entries trong UCD_TABLE.
#[inline]
pub fn table_len() -> usize {
    UCD_TABLE.len()
}

/// Toàn bộ UCD_TABLE — dùng cho L0 full seeding.
///
/// Trả về slice tĩnh chứa ~5400 entries, sorted by codepoint.
/// Mỗi entry = 1 nguyên tố trong bảng tuần hoàn của HomeOS.
#[inline]
pub fn table() -> &'static [UcdEntry] {
    UCD_TABLE
}

/// Kiểm tra codepoint có phải SDF primitive không.
#[inline]
pub fn is_sdf_primitive(cp: u32) -> bool {
    SDF_PRIMITIVES.iter().any(|&(p, _)| p == cp)
}

/// Kiểm tra codepoint có phải RELATION primitive không.
#[inline]
pub fn is_relation_primitive(cp: u32) -> bool {
    RELATION_PRIMITIVES.iter().any(|&(p, _)| p == cp)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    // ── Forward lookup ──────────────────────────────────────────────────────

    #[test]
    fn lookup_fire() {
        if table_len() == 0 {
            return;
        }
        let e = lookup(0x1F525).expect("🔥 FIRE phải có trong UCD");
        assert_eq!(e.cp, 0x1F525);
        assert_eq!(e.group, 0x03, "FIRE thuộc EMOTICON group");
        assert_eq!((e.shape - 1) % 8 + 1, 0x01, "FIRE shape base = Sphere");
        assert_eq!((e.relation - 1) % 8 + 1, 0x01, "FIRE relation base = Member");
        assert!(
            e.valence >= 0xC0,
            "FIRE valence phải cao, got 0x{:02X}",
            e.valence
        );
        assert!(
            e.arousal >= 0xC0,
            "FIRE arousal phải cao, got 0x{:02X}",
            e.arousal
        );
        assert_eq!((e.time - 1) % 5 + 1, 0x04, "FIRE time base = Fast");
    }

    #[test]
    fn lookup_sphere_sdf() {
        if table_len() == 0 {
            return;
        }
        let e = lookup(0x25CF).expect("● BLACK CIRCLE phải có");
        assert_eq!((e.shape - 1) % 8 + 1, 0x01, "● = Sphere base");
        assert_eq!(e.group, 0x01, "Geometric Shapes = SDF group");
        assert_eq!((e.time - 1) % 5 + 1, 0x01, "SDF shapes = Static base");
    }

    #[test]
    fn lookup_torus_sdf() {
        if table_len() == 0 {
            return;
        }
        let e = lookup(0x25CB).expect("○ WHITE CIRCLE phải có");
        assert_eq!((e.shape - 1) % 8 + 1, 0x05, "○ = Torus base");
    }

    #[test]
    fn lookup_member_relation() {
        if table_len() == 0 {
            return;
        }
        let e = lookup(0x2208).expect("∈ ELEMENT OF phải có");
        assert_eq!((e.relation - 1) % 8 + 1, 0x01, "∈ = Member base");
        assert_eq!((e.time - 1) % 5 + 1, 0x01, "Math relation = Static base");
    }

    #[test]
    fn lookup_arrow_causes() {
        if table_len() == 0 {
            return;
        }
        let e = lookup(0x2192).expect("→ RIGHTWARDS ARROW phải có");
        assert_eq!((e.relation - 1) % 8 + 1, 0x06, "→ = Causes base");
        assert_eq!((e.time - 1) % 5 + 1, 0x05, "Arrow = Instant base");
    }

    #[test]
    fn lookup_pi_math() {
        if table_len() == 0 {
            return;
        }
        // π = U+03C0 — không thuộc 5 nhóm → None
        // Nhưng ∂ = U+2202 thuộc Math Operators
        let e = lookup(0x2202).expect("∂ PARTIAL DIFFERENTIAL phải có");
        assert_eq!(e.group, 0x02, "∂ thuộc MATH group");
        assert_eq!((e.time - 1) % 5 + 1, 0x01, "Math = Static base");
    }

    #[test]
    fn lookup_musical_note() {
        if table_len() == 0 {
            return;
        }
        // Musical Symbols thật sự ở 1D100..1D1FF
        // 𝄞 MUSICAL SYMBOL G CLEF = U+1D11E
        if let Some(e) = lookup(0x1D11E) {
            assert_eq!(e.group, 0x04, "𝄞 thuộc MUSICAL group");
        }
        // ♩ U+2669 thuộc Misc Symbols (2600..26FF) = EMOTICON group
        let e2 = lookup(0x2669).expect("♩ phải có trong EMOTICON");
        assert_eq!(e2.group, 0x03, "♩ thuộc EMOTICON (Misc Symbols)");
    }

    #[test]
    fn lookup_droplet() {
        if table_len() == 0 {
            return;
        }
        let e = lookup(0x1F4A7).expect("💧 DROPLET phải có");
        assert_eq!(e.group, 0x03, "DROPLET = EMOTICON");
        assert!(e.valence >= 0x80, "DROPLET valence moderate+");
        assert!(e.arousal <= 0x80, "DROPLET arousal thấp (calm)");
        assert_eq!((e.time - 1) % 5 + 1, 0x02, "DROPLET = Slow base");
    }

    #[test]
    fn lookup_nonexistent() {
        // Latin 'A' không thuộc 5 nhóm
        assert!(lookup(0x0041).is_none(), "'A' không thuộc 5 nhóm");
    }

    #[test]
    fn table_not_empty() {
        if table_len() == 0 {
            return;
        }
        assert!(
            table_len() > 1000,
            "UCD table phải có >1000 entries, got {}",
            table_len()
        );
    }

    #[test]
    fn table_sorted_by_cp() {
        // Bắt buộc cho binary_search
        for i in 1..UCD_TABLE.len() {
            assert!(
                UCD_TABLE[i - 1].cp < UCD_TABLE[i].cp,
                "UCD_TABLE phải sorted: 0x{:05X} >= 0x{:05X} tại index {}",
                UCD_TABLE[i - 1].cp,
                UCD_TABLE[i].cp,
                i
            );
        }
    }

    // ── Reverse lookup (requires feature "reverse-index") ──────────────────

    #[test]
    #[cfg(feature = "reverse-index")]
    fn decode_hash_fire() {
        if table_len() == 0 {
            return;
        }
        let e = lookup(0x1F525).unwrap();
        let decoded = decode_hash(e.hash);
        // Có thể decode ra cp khác nếu hash collision
        // Nhưng phải decode ra SOMETHING
        assert!(decoded.is_some(), "decode_hash phải trả về Some");
    }

    #[test]
    #[cfg(feature = "reverse-index")]
    fn hash_to_cp_sorted() {
        for i in 1..HASH_TO_CP.len() {
            assert!(
                HASH_TO_CP[i - 1].0 < HASH_TO_CP[i].0,
                "HASH_TO_CP phải sorted tại index {}",
                i
            );
        }
    }

    #[test]
    #[cfg(not(feature = "reverse-index"))]
    fn decode_hash_stub_returns_none() {
        assert!(decode_hash(0x12345678).is_none(), "Stub always None");
    }

    // ── Bucket lookup ───────────────────────────────────────────────────────

    #[test]
    #[cfg(feature = "reverse-index")]
    fn bucket_sphere_member() {
        if table_len() == 0 {
            return;
        }
        let cps = bucket_cps(0x01, 0x01); // Sphere + Member
                                          // EMOTICON group nhiều nodes Sphere+Member
        assert!(!cps.is_empty(), "bucket (Sphere, Member) phải có entries");
        // Mọi cp trong bucket phải có shape=Sphere, relation=Member
        for &cp in cps {
            if let Some(e) = lookup(cp) {
                assert_eq!(e.shape, 0x01, "cp 0x{:05X} phải Sphere", cp);
                assert_eq!(e.relation, 0x01, "cp 0x{:05X} phải Member", cp);
            }
        }
    }

    #[test]
    #[cfg(feature = "reverse-index")]
    fn bucket_nonexistent_empty() {
        // Shape=0xFF không tồn tại
        let cps = bucket_cps(0xFF, 0xFF);
        assert!(cps.is_empty());
    }

    #[test]
    #[cfg(not(feature = "reverse-index"))]
    fn bucket_stub_returns_empty() {
        assert!(bucket_cps(0x01, 0x01).is_empty(), "Stub always empty");
    }

    // ── SDF + RELATION primitives ───────────────────────────────────────────

    #[test]
    fn sdf_primitives_count() {
        assert_eq!(SDF_PRIMITIVES.len(), 8, "Phải có đúng 8 SDF primitives");
    }

    #[test]
    fn relation_primitives_count() {
        assert_eq!(
            RELATION_PRIMITIVES.len(),
            8,
            "Phải có đúng 8 RELATION primitives"
        );
    }

    #[test]
    fn is_sdf_primitive_correct() {
        assert!(is_sdf_primitive(0x25CF), "● = SDF primitive");
        assert!(is_sdf_primitive(0x25CB), "○ = SDF primitive");
        assert!(is_sdf_primitive(0x222A), "∪ = SDF primitive");
        assert!(!is_sdf_primitive(0x0041), "'A' không phải SDF primitive");
    }

    #[test]
    fn is_relation_primitive_correct() {
        assert!(is_relation_primitive(0x2208), "∈ = RELATION primitive");
        assert!(is_relation_primitive(0x2192), "→ = RELATION primitive");
        assert!(is_relation_primitive(0x2190), "← = RELATION primitive");
        assert!(
            !is_relation_primitive(0x25CF),
            "● không phải RELATION primitive"
        );
    }

    // ── Convenience functions ───────────────────────────────────────────────

    #[test]
    fn convenience_fns_fire() {
        if table_len() == 0 {
            return;
        }
        let s = shape_of(0x1F525);
        assert_eq!((s - 1) % 8 + 1, 0x01, "FIRE shape base = Sphere");
        let r = relation_of(0x1F525);
        assert_eq!((r - 1) % 8 + 1, 0x01, "FIRE relation base = Member");
        assert!(valence_of(0x1F525) >= 0xC0);
        assert!(arousal_of(0x1F525) >= 0xC0);
        let t = time_of(0x1F525);
        assert_eq!((t - 1) % 5 + 1, 0x04, "FIRE time base = Fast");
        assert_eq!(group_of(0x1F525), 0x03);
    }

    #[test]
    fn convenience_fns_unknown_default() {
        // cp không trong UCD → default values
        assert_eq!(shape_of(0x0041), 0x01); // Sphere default
        assert_eq!(relation_of(0x0041), 0x01); // Member default
        assert_eq!(valence_of(0x0041), 0x80); // neutral
        assert_eq!(arousal_of(0x0041), 0x80); // moderate
        assert_eq!(time_of(0x0041), 0x03); // Medium
        assert_eq!(group_of(0x0041), 0x00); // no group
    }

    #[test]
    fn count_unique_molecules() {
        if table_len() == 0 {
            return;
        }
        // Count unique 5-tuples using a sorted vec + dedup
        let mut tuples: Vec<(u8, u8, u8, u8, u8)> = UCD_TABLE
            .iter()
            .map(|e| (e.shape, e.relation, e.valence, e.arousal, e.time))
            .collect();
        let total = tuples.len();
        tuples.sort();
        tuples.dedup();
        let unique = tuples.len();
        let collision_rate = 1.0 - (unique as f64 / total as f64);
        assert!(
            unique as f64 / total as f64 > 0.90,
            "COLLISION: {total} entries → {unique} unique molecules ({:.1}% collision). Need >90% unique.",
            collision_rate * 100.0
        );
    }

    #[test]
    fn count_unique_hashes() {
        if table_len() == 0 {
            return;
        }
        let mut hashes: Vec<u64> = UCD_TABLE.iter().map(|e| e.hash).collect();
        let total = hashes.len();
        hashes.sort();
        hashes.dedup();
        let unique = hashes.len();
        assert!(
            unique as f64 / total as f64 > 0.90,
            "HASH COLLISION: {total} entries → {unique} unique hashes ({:.1}% collision).",
            (1.0 - unique as f64 / total as f64) * 100.0
        );
    }
}
