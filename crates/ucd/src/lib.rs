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

// ─────────────────────────────────────────────────────────────────────────────
// KnowTree hierarchy API
// ─────────────────────────────────────────────────────────────────────────────

/// Number of groups (L0) in the KnowTree hierarchy.
#[inline]
pub fn group_count() -> usize {
    KNOWTREE_GROUP_COUNT
}

/// Number of blocks (L1) in the KnowTree hierarchy.
#[inline]
pub fn block_count() -> usize {
    KNOWTREE_BLOCK_COUNT
}

/// Group definitions: &[(name, aggregate_p_weight, &[block_indices])].
#[inline]
pub fn groups() -> &'static [(&'static str, u16, &'static [u16])] {
    KNOWTREE_GROUPS
}

/// Block definitions: &[(name, aggregate_p_weight, chars_start_idx, chars_count)].
#[inline]
pub fn blocks() -> &'static [(&'static str, u16, u16, u16)] {
    KNOWTREE_BLOCKS
}

/// Block indices belonging to a given group.
///
/// Returns empty slice if group_idx out of range.
#[inline]
pub fn group_blocks(group_idx: usize) -> &'static [u16] {
    if group_idx < KNOWTREE_GROUPS.len() {
        KNOWTREE_GROUPS[group_idx].2
    } else {
        &[]
    }
}

/// UCD entries belonging to a given block.
///
/// Returns empty slice if block_idx out of range.
#[inline]
pub fn block_chars(block_idx: usize) -> &'static [UcdEntry] {
    if block_idx < KNOWTREE_BLOCKS.len() {
        let (_, _, start, count) = KNOWTREE_BLOCKS[block_idx];
        let s = start as usize;
        let e = s + count as usize;
        if e <= UCD_TABLE.len() {
            &UCD_TABLE[s..e]
        } else {
            &[]
        }
    } else {
        &[]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SDF + RELATION primitives
// ─────────────────────────────────────────────────────────────────────────────

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
    fn sdf_primitives_count() {
        assert_eq!(SDF_PRIMITIVES.len(), 18, "Phải có đúng 18 SDF primitives (v2)");
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
        assert!(!is_sdf_primitive(0x0041), "'A' không phải SDF primitive");
    }

    #[test]
    fn is_relation_primitive_correct() {
        assert!(is_relation_primitive(0x2208), "∈ = RELATION primitive");
        assert!(is_relation_primitive(0x2192), "→ = RELATION primitive");
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

    // ── KnowTree hierarchy API ──────────────────────────────────────────

    #[test]
    fn knowtree_group_count() {
        assert_eq!(group_count(), 4, "Must have 4 groups (SDF, MATH, EMOTICON, MUSICAL)");
    }

    #[test]
    fn knowtree_block_count() {
        assert!(block_count() > 0, "Must have blocks");
    }

    #[test]
    fn knowtree_groups_data() {
        let g = groups();
        assert_eq!(g.len(), 4);
        // Check group names
        assert_eq!(g[0].0, "SDF");
        assert_eq!(g[1].0, "MATH");
        assert_eq!(g[2].0, "EMOTICON");
        assert_eq!(g[3].0, "MUSICAL");
        // Each group should have block indices
        for (name, _pw, block_idxs) in g {
            assert!(!block_idxs.is_empty(), "Group {} should have blocks", name);
        }
    }

    #[test]
    fn knowtree_blocks_data() {
        let b = blocks();
        assert!(!b.is_empty());
        // First block should have a name and valid counts
        let (name, _pw, _start, count) = b[0];
        assert!(!name.is_empty());
        assert!(count > 0, "First block should have chars");
    }

    #[test]
    fn knowtree_group_blocks_api() {
        for gi in 0..group_count() {
            let block_idxs = group_blocks(gi);
            assert!(!block_idxs.is_empty(), "Group {} should have blocks", gi);
            for &bi in block_idxs {
                assert!((bi as usize) < block_count(),
                    "Block index {} out of range for group {}", bi, gi);
            }
        }
        // Out of range
        assert!(group_blocks(999).is_empty());
    }

    #[test]
    fn knowtree_block_chars_api() {
        for bi in 0..block_count() {
            let chars = block_chars(bi);
            // Block should have characters (or be empty for sparse blocks)
            let (_, _, _, count) = blocks()[bi];
            assert_eq!(chars.len(), count as usize,
                "Block {} char count mismatch", bi);
        }
        // Out of range
        assert!(block_chars(999).is_empty());
    }
}
