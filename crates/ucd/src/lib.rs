//! # ucd — Unicode Character Database (v2)
//!
//! Lookup codepoint → UcdEntry (5D P_weight) từ bảng tĩnh generated từ json/udc.json.
//! Không cần file UCD lúc runtime. Chạy no_std.
//!
//! ## P_weight packed u16: [S:4][R:4][V:3][A:3][T:2]
//!
//! ## Source of truth: json/udc.json (8,284 characters, 53 blocks, 4 groups)
//! Không heuristic — mọi giá trị trực tiếp từ udc.json.
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
/// O(log n). Returns None nếu cp không thuộc 4 nhóm.
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
/// Requires feature `reverse-index` (default). Tắt trên embedded để tiết kiệm.
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
///
/// Requires feature `reverse-index` (default).
#[cfg(feature = "reverse-index")]
pub fn bucket_cps(shape: u8, relation: u8) -> &'static [u32] {
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

/// Shape byte của codepoint. Default 0x01 (Sphere) nếu không tìm thấy.
#[inline]
pub fn shape_of(cp: u32) -> u8 {
    lookup(cp).map(|e| e.shape).unwrap_or(0x01)
}

/// Relation byte của codepoint. Default 0x01 (Member) nếu không tìm thấy.
#[inline]
pub fn relation_of(cp: u32) -> u8 {
    lookup(cp).map(|e| e.relation).unwrap_or(0x01)
}

/// Valence byte của codepoint. Default 0x80 (neutral).
#[inline]
pub fn valence_of(cp: u32) -> u8 {
    lookup(cp).map(|e| e.valence).unwrap_or(0x80)
}

/// Arousal byte của codepoint. Default 0x80 (moderate).
#[inline]
pub fn arousal_of(cp: u32) -> u8 {
    lookup(cp).map(|e| e.arousal).unwrap_or(0x80)
}

/// Time byte của codepoint. Default 0x03 (Medium).
#[inline]
pub fn time_of(cp: u32) -> u8 {
    lookup(cp).map(|e| e.time).unwrap_or(0x03)
}

/// Group byte của codepoint.
#[inline]
pub fn group_of(cp: u32) -> u8 {
    lookup(cp).map(|e| e.group).unwrap_or(0x00)
}

/// Packed P_weight u16 của codepoint.
///
/// Layout: [S:4][R:4][V:3][A:3][T:2]
#[inline]
pub fn p_weight_of(cp: u32) -> u16 {
    lookup(cp).map(|e| e.p_weight).unwrap_or(0)
}

/// Số entries trong UCD_TABLE.
#[inline]
pub fn table_len() -> usize {
    UCD_TABLE.len()
}

/// Toàn bộ UCD_TABLE — dùng cho L0 full seeding.
///
/// Trả về slice tĩnh chứa ~8,284 entries, sorted by codepoint.
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

    // ── Forward lookup ──────────────────────────────────────────────────────

    #[test]
    fn lookup_fire() {
        let e = lookup(0x1F525).expect("🔥 FIRE phải có trong UCD");
        assert_eq!(e.cp, 0x1F525);
        assert_eq!(e.group, 0x03, "FIRE thuộc EMOTICON group");
    }

    #[test]
    fn lookup_sphere_sdf() {
        let e = lookup(0x25CF).expect("● BLACK CIRCLE phải có");
        assert_eq!(e.group, 0x01, "Geometric Shapes = SDF group");
    }

    #[test]
    fn lookup_nonexistent() {
        // Latin 'A' không thuộc 4 nhóm
        assert!(lookup(0x0041).is_none(), "'A' không thuộc 4 nhóm");
    }

    #[test]
    fn table_not_empty() {
        assert!(
            table_len() > 1000,
            "UCD table phải có >1000 entries, got {}",
            table_len()
        );
    }

    #[test]
    fn table_sorted_by_cp() {
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

    // ── P_weight packed ─────────────────────────────────────────────────────

    #[test]
    fn p_weight_nonzero_for_known_cp() {
        // FIRE should have nonzero p_weight
        let pw = p_weight_of(0x1F525);
        assert!(pw != 0, "FIRE p_weight should be nonzero");
    }

    #[test]
    fn p_weight_layout() {
        // Check that p_weight is correctly packed [S:4][R:4][V:3][A:3][T:2]
        if let Some(e) = lookup(0x1F525) {
            let pw = e.p_weight;
            let s = (pw >> 12) & 0xF;
            let r = (pw >> 8) & 0xF;
            let v = (pw >> 5) & 0x7;
            let a = (pw >> 2) & 0x7;
            let t = pw & 0x3;
            // Verify these match quantized raw values
            assert_eq!(s, (e.shape >> 4) as u16, "S mismatch");
            assert_eq!(r, (e.relation >> 4) as u16, "R mismatch");
            assert_eq!(v, (e.valence >> 5) as u16, "V mismatch");
            assert_eq!(a, (e.arousal >> 5) as u16, "A mismatch");
            assert_eq!(t, (e.time >> 6) as u16, "T mismatch");
        }
    }

    // ── Reverse lookup (requires feature "reverse-index") ──────────────────

    #[test]
    #[cfg(feature = "reverse-index")]
    fn decode_hash_fire() {
        let e = lookup(0x1F525).unwrap();
        let decoded = decode_hash(e.hash);
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
    fn bucket_nonexistent_empty() {
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
    fn sdf_primitives_derived_from_udc() {
        // SDF_PRIMITIVES now contains ALL characters with group=SDF from udc.json
        // (not just 18 hardcoded ones). Each carries its UDC shape position.
        assert!(
            SDF_PRIMITIVES.len() > 0,
            "SDF primitives must be derived from udc.json (group=SDF)"
        );
    }

    #[test]
    fn relation_primitives_derived_from_udc() {
        // RELATION_PRIMITIVES now contains ALL characters with group=MATH from udc.json
        assert!(
            RELATION_PRIMITIVES.len() > 0,
            "RELATION primitives must be derived from udc.json (group=MATH)"
        );
    }

    #[test]
    fn is_sdf_primitive_correct() {
        // UTF-32 codepoints are aliases mapping into UDC positions
        assert!(is_sdf_primitive(0x25CF), "● = SDF alias (UDC SPHERE)");
        assert!(is_sdf_primitive(0x25CB), "○ = SDF alias (UDC TORUS)");
        assert!(!is_sdf_primitive(0x0041), "'A' không phải SDF primitive");
    }

    #[test]
    fn is_relation_primitive_correct() {
        // UTF-32 math symbols are aliases mapping into UDC relation positions
        // RELATION_PRIMITIVES = all characters with group=MATH in udc.json
        assert!(is_relation_primitive(0x2208), "∈ = MATH group (UDC MEMBER)");
        // Note: 0x2192 (→) is in Arrows block = SDF group, NOT MATH group
        assert!(
            !is_relation_primitive(0x2192),
            "→ belongs to SDF group (Arrows), not MATH"
        );
        assert!(
            !is_relation_primitive(0x25CF),
            "● không phải RELATION primitive"
        );
    }

    // ── Convenience functions ───────────────────────────────────────────────

    #[test]
    fn convenience_fns_unknown_default() {
        assert_eq!(shape_of(0x0041), 0x01); // Sphere default
        assert_eq!(relation_of(0x0041), 0x01); // Member default
        assert_eq!(valence_of(0x0041), 0x80); // neutral
        assert_eq!(arousal_of(0x0041), 0x80); // moderate
        assert_eq!(time_of(0x0041), 0x03); // Medium
        assert_eq!(group_of(0x0041), 0x00); // no group
    }
}
