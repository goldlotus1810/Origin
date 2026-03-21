//! build.rs — đọc json/udc.json lúc compile → sinh bảng tĩnh (v2)
//!
//! Output trong OUT_DIR/ucd_generated.rs:
//!   UCD_TABLE         — forward lookup (cp → UcdEntry with packed u16 P_weight)
//!   HASH_TO_CP        — reverse index (chain_hash → cp), O(log n) decode
//!   CP_BUCKET          — bucket index (shape,relation → [cp]), top-n decode
//!   SDF_PRIMITIVES    — 18 SDF primitive mappings (v2)
//!   RELATION_PRIMITIVES — 8 Relation primitive mappings
//!
//! Source of truth: json/udc.json (8,284 characters, 53 blocks, 4 groups)
//! KHÔNG heuristic — P_weight trực tiếp từ udc.json.

use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::PathBuf;

// ─────────────────────────────────────────────────────────────────────────────
// JSON schema — chỉ deserialize các trường cần thiết
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct UdcJson {
    characters: Vec<UdcCharacter>,
    #[serde(default)]
    blocks: Vec<UdcBlock>,
}

#[derive(Deserialize)]
struct UdcBlock {
    name: String,
    group: String,
    range: String,        // e.g. "2190..21FF"
    #[serde(default)]
    #[allow(dead_code)]
    id: String,
}

#[derive(Deserialize)]
struct UdcCharacter {
    codepoint: u32,
    name: String,
    group: String,
    #[serde(default)]
    #[allow(dead_code)]
    hex: String,
    physics_logic: PhysicsLogic,
}

#[derive(Deserialize)]
struct PhysicsLogic {
    #[serde(rename = "P_weight")]
    p_weight: Vec<u16>,
}

// ─────────────────────────────────────────────────────────────────────────────
// 18 SDF Primitives (v2 spec)
// ─────────────────────────────────────────────────────────────────────────────

static SDF_PRIMS: &[(u32, u8, &str)] = &[
    (0x25CF, 0, "BLACK CIRCLE"),               // ● Sphere
    (0x25A0, 1, "BLACK SQUARE"),               // ■ Box
    (0x25AC, 2, "BLACK RECTANGLE"),            // ▬ Capsule
    (0x25BD, 3, "WHITE DOWN-POINTING TRIANGLE"), // ▽ Plane
    (0x25CB, 4, "WHITE CIRCLE"),               // ○ Torus
    (0x2B2E, 5, "BLACK VERTICAL ELLIPSE"),     // ⬮ Ellipsoid
    (0x25B2, 6, "BLACK UP-POINTING TRIANGLE"), // ▲ Cone
    (0x25AD, 7, "WHITE RECTANGLE"),            // ▭ Cylinder
    (0x25C6, 8, "BLACK DIAMOND"),              // ◆ Octahedron
    (0x25B3, 9, "WHITE UP-POINTING TRIANGLE"), // △ Pyramid
    (0x2B21, 10, "WHITE HEXAGON"),             // ⬡ HexPrism
    (0x25B1, 11, "WHITE PARALLELOGRAM"),       // ▱ Prism
    (0x25A2, 12, "WHITE SQUARE WITH ROUNDED CORNERS"), // ▢ RoundBox
    (0x221E, 13, "INFINITY"),                  // ∞ Link
    (0x21BB, 14, "CLOCKWISE OPEN CIRCLE ARROW"), // ↻ Revolve
    (0x21E7, 15, "UPWARDS WHITE ARROW"),       // ⇧ Extrude
    (0x25D0, 16, "CIRCLE WITH LEFT HALF BLACK"), // ◐ CutSphere
    (0x2606, 17, "WHITE STAR"),                // ☆ DeathStar
];

// ─────────────────────────────────────────────────────────────────────────────
// 8 RELATION Primitives — từ Mathematical Operators 2200..22FF
// ─────────────────────────────────────────────────────────────────────────────

static REL_PRIMS: &[(u32, u8, &str)] = &[
    (0x2208, 0x01, "ELEMENT OF"),       // ∈ Member
    (0x2282, 0x02, "SUBSET OF"),        // ⊂ Subset
    (0x2261, 0x03, "IDENTICAL TO"),     // ≡ Equiv
    (0x22A5, 0x04, "UP TACK"),          // ⊥ Orthogonal
    (0x2218, 0x05, "RING OPERATOR"),    // ∘ Compose
    (0x2192, 0x06, "RIGHTWARDS ARROW"), // → Causes
    (0x2248, 0x07, "ALMOST EQUAL TO"),  // ≈ Similar
    (0x2190, 0x08, "LEFTWARDS ARROW"),  // ← DerivedFrom
];

// ─────────────────────────────────────────────────────────────────────────────
// Group → byte mapping
// ─────────────────────────────────────────────────────────────────────────────

fn group_byte(group: &str) -> u8 {
    match group {
        "SDF" => 0x01,
        "MATH" => 0x02,
        "EMOTICON" => 0x03,
        "MUSICAL" => 0x04,
        _ => 0x00,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry struct — intermediate representation for codegen
// ─────────────────────────────────────────────────────────────────────────────

struct Entry {
    cp: u32,
    group: u8,
    shape: u8,
    relation: u8,
    valence: u8,
    arousal: u8,
    time: u8,
    p_weight: u16,
    hash: u64,
    name: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// FNV-1a hash — giống với chain_hash() trong molecular.rs
// ─────────────────────────────────────────────────────────────────────────────

fn fnv1a_hash(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

fn chain_hash(shape: u8, relation: u8, valence: u8, arousal: u8, time: u8) -> u64 {
    fnv1a_hash(&[shape, relation, valence, arousal, time])
}

/// Pack 5 u8 values into u16: [S:4][R:4][V:3][A:3][T:2]
///
/// S quantize: 0-255 → 0-15 (>> 4)
/// R quantize: 0-255 → 0-15 (>> 4)
/// V quantize: 0-255 → 0-7  (>> 5)
/// A quantize: 0-255 → 0-7  (>> 5)
/// T quantize: 0-255 → 0-3  (>> 6)
fn pack_p_weight(s: u8, r: u8, v: u8, a: u8, t: u8) -> u16 {
    let s4 = (s >> 4) as u16;     // 4 bits
    let r4 = (r >> 4) as u16;     // 4 bits
    let v3 = (v >> 5) as u16;     // 3 bits
    let a3 = (a >> 5) as u16;     // 3 bits
    let t2 = (t >> 6) as u16;     // 2 bits
    (s4 << 12) | (r4 << 8) | (v3 << 5) | (a3 << 2) | t2
}

// ─────────────────────────────────────────────────────────────────────────────
// main
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace = manifest.parent().unwrap().parent().unwrap();
    let udc_json_path = workspace.join("json/udc.json");

    println!("cargo:rerun-if-changed={}", udc_json_path.display());

    if !udc_json_path.exists() {
        eprintln!("cargo:warning=json/udc.json not found — generating empty tables");
        write_empty();
        return;
    }

    // ── Parse udc.json ───────────────────────────────────────────────────
    let content = fs::read_to_string(&udc_json_path).expect("read json/udc.json");
    let udc: UdcJson = serde_json::from_str(&content).expect("parse json/udc.json");

    eprintln!("cargo:warning=UDC: parsed {} characters from udc.json", udc.characters.len());

    // ── Build entries ────────────────────────────────────────────────────

    let mut entries: Vec<Entry> = Vec::with_capacity(udc.characters.len());

    for ch in &udc.characters {
        let pw = &ch.physics_logic.p_weight;
        if pw.len() < 5 {
            eprintln!("cargo:warning=UDC: skip cp=0x{:04X} ({}) — P_weight len={}", ch.codepoint, ch.name, pw.len());
            continue;
        }

        let s = pw[0] as u8;
        let r = pw[1] as u8;
        let v = pw[2] as u8;
        let a = pw[3] as u8;
        let t = pw[4] as u8;
        let hash = chain_hash(s, r, v, a, t);
        let packed = pack_p_weight(s, r, v, a, t);
        let group = group_byte(&ch.group);

        entries.push(Entry {
            cp: ch.codepoint,
            group,
            shape: s,
            relation: r,
            valence: v,
            arousal: a,
            time: t,
            p_weight: packed,
            hash,
            name: ch.name.clone(),
        });
    }

    // Sort by cp cho binary_search
    entries.sort_by_key(|e| e.cp);
    // Deduplicate by codepoint (keep first occurrence)
    entries.dedup_by_key(|e| e.cp);

    eprintln!("cargo:warning=UDC entries: {} (after dedup)", entries.len());

    // ── Build HASH_TO_CP (reverse index) ─────────────────────────────────
    let mut hash_to_cp: Vec<(u64, u32)> = entries.iter().map(|e| (e.hash, e.cp)).collect();
    hash_to_cp.sort_by_key(|&(h, _)| h);
    hash_to_cp.dedup_by_key(|&mut (h, _)| h);

    // ── Build CP_BUCKET (shape,relation → [cp]) ──────────────────────────
    let mut buckets: HashMap<(u8, u8), Vec<u32>> = HashMap::new();
    for e in &entries {
        buckets.entry((e.shape, e.relation)).or_default().push(e.cp);
    }
    let mut bucket_list: Vec<((u8, u8), Vec<u32>)> = buckets.into_iter().collect();
    bucket_list.sort_by_key(|&((s, r), _)| (s, r));

    // ── Generate Rust source ──────────────────────────────────────────────
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = out_dir.join("ucd_generated.rs");

    let mut src = String::new();

    writeln!(src, "// AUTO-GENERATED by ucd/build.rs (v2)").unwrap();
    writeln!(src, "// Source: json/udc.json").unwrap();
    writeln!(src, "// {} entries · DO NOT EDIT", entries.len()).unwrap();
    writeln!(src, "// P_weight packed u16: [S:4][R:4][V:3][A:3][T:2]").unwrap();
    writeln!(src).unwrap();

    // UcdEntry struct — v2 with p_weight u16
    writeln!(src, "#[derive(Clone, Copy, Debug)]").unwrap();
    writeln!(src, "#[allow(missing_docs)]").unwrap();
    writeln!(src, "pub struct UcdEntry {{").unwrap();
    writeln!(src, "    pub cp:       u32,").unwrap();
    writeln!(src, "    pub group:    u8,   // 0x01=SDF 0x02=MATH 0x03=EMOTICON 0x04=MUSICAL").unwrap();
    writeln!(src, "    pub shape:    u8,   // S dimension (raw u8 from udc.json)").unwrap();
    writeln!(src, "    pub relation: u8,   // R dimension (raw u8 from udc.json)").unwrap();
    writeln!(src, "    pub valence:  u8,   // V dimension 0x00..0xFF").unwrap();
    writeln!(src, "    pub arousal:  u8,   // A dimension 0x00..0xFF").unwrap();
    writeln!(src, "    pub time:     u8,   // T dimension (raw u8 from udc.json)").unwrap();
    writeln!(src, "    pub p_weight: u16,  // packed [S:4][R:4][V:3][A:3][T:2]").unwrap();
    writeln!(src, "    pub hash:     u64,  // FNV-1a of [shape,rel,val,aro,time]").unwrap();
    writeln!(src, "    pub name:     &'static str,").unwrap();
    writeln!(src, "}}").unwrap();
    writeln!(src).unwrap();

    // UCD_TABLE — forward lookup sorted by cp
    writeln!(src, "/// Forward lookup: cp → UcdEntry (binary search by cp)").unwrap();
    writeln!(src, "/// {} entries from json/udc.json (53 blocks, 4 groups)", entries.len()).unwrap();
    writeln!(src, "pub static UCD_TABLE: &[UcdEntry] = &[").unwrap();
    for e in &entries {
        let safe_name = e.name.replace('\\', "\\\\").replace('"', "\\\"");
        writeln!(src,
            "    UcdEntry {{ cp: 0x{:05X}, group: 0x{:02X}, shape: 0x{:02X}, relation: 0x{:02X}, valence: 0x{:02X}, arousal: 0x{:02X}, time: 0x{:02X}, p_weight: 0x{:04X}, hash: 0x{:016X}u64, name: \"{}\" }},",
            e.cp, e.group, e.shape, e.relation, e.valence, e.arousal, e.time, e.p_weight, e.hash, safe_name
        ).unwrap();
    }
    writeln!(src, "];").unwrap();
    writeln!(src).unwrap();

    // HASH_TO_CP — reverse index sorted by hash
    writeln!(src, "/// Reverse index: chain_hash → cp (binary search by hash) O(log n)").unwrap();
    writeln!(src, "#[cfg(feature = \"reverse-index\")]").unwrap();
    writeln!(src, "pub static HASH_TO_CP: &[(u64, u32)] = &[").unwrap();
    for (hash, cp) in &hash_to_cp {
        writeln!(src, "    (0x{:016X}u64, 0x{:05X}),", hash, cp).unwrap();
    }
    writeln!(src, "];").unwrap();
    writeln!(src).unwrap();

    // CP_BUCKET
    writeln!(src, "/// Bucket index: (shape, relation) → [cp] for top-n decode").unwrap();
    writeln!(src, "#[cfg(feature = \"reverse-index\")]").unwrap();
    writeln!(src, "pub static CP_BUCKET_DATA: &[u32] = &[").unwrap();
    let mut bucket_offsets: Vec<((u8, u8), u32, u32)> = Vec::new();
    let mut offset: u32 = 0;
    let mut all_cps: Vec<u32> = Vec::new();
    for ((s, r), cps) in &bucket_list {
        bucket_offsets.push(((*s, *r), offset, cps.len() as u32));
        for &cp in cps {
            all_cps.push(cp);
        }
        offset += cps.len() as u32;
    }
    for cp in &all_cps {
        writeln!(src, "    0x{:05X},", cp).unwrap();
    }
    writeln!(src, "];").unwrap();
    writeln!(src).unwrap();

    writeln!(src, "/// Bucket lookup: (shape, relation) → (offset, count) into CP_BUCKET_DATA").unwrap();
    writeln!(src, "#[cfg(feature = \"reverse-index\")]").unwrap();
    writeln!(src, "pub static CP_BUCKET_INDEX: &[(u8, u8, u32, u32)] = &[").unwrap();
    for ((s, r), off, cnt) in &bucket_offsets {
        writeln!(src, "    (0x{:02X}, 0x{:02X}, {}, {}),", s, r, off, cnt).unwrap();
    }
    writeln!(src, "];").unwrap();
    writeln!(src).unwrap();

    // SDF_PRIMITIVES — 18 SDF (v2)
    writeln!(src, "/// 18 SDF primitives (v2): (codepoint, shape_index)").unwrap();
    writeln!(src, "pub static SDF_PRIMITIVES: &[(u32, u8)] = &[").unwrap();
    for &(cp, byte, _name) in SDF_PRIMS {
        writeln!(src, "    (0x{:04X}, 0x{:02X}),", cp, byte).unwrap();
    }
    writeln!(src, "];").unwrap();
    writeln!(src).unwrap();

    // RELATION_PRIMITIVES
    writeln!(src, "/// 8 RELATION primitives: (codepoint, relation_byte)").unwrap();
    writeln!(src, "pub static RELATION_PRIMITIVES: &[(u32, u8)] = &[").unwrap();
    for &(cp, byte, _name) in REL_PRIMS {
        writeln!(src, "    (0x{:04X}, 0x{:02X}),", cp, byte).unwrap();
    }
    writeln!(src, "];").unwrap();
    writeln!(src).unwrap();

    // ── KnowTree topology: Groups (L0) and Blocks (L1) ─────────────────
    // Build block→char mapping from udc.json blocks + character block fields
    generate_knowtree_topology(&mut src, &udc, &entries);

    fs::write(&out_file, &src).expect("write ucd_generated.rs");
    eprintln!(
        "cargo:warning=Generated: {} entries, {} hash entries, {} buckets, 18 SDF prims",
        entries.len(),
        hash_to_cp.len(),
        bucket_list.len()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// KnowTree topology generation
// ─────────────────────────────────────────────────────────────────────────────

/// Compute aggregate P_weight via per-dimension mode (most frequent value).
/// P_weight layout: [S:4][R:4][V:3][A:3][T:2]
fn mode_p_weight(p_weights: &[u16]) -> u16 {
    if p_weights.is_empty() {
        return 0;
    }

    // Extract each dimension, count frequencies, pick mode
    let s_vals: Vec<u16> = p_weights.iter().map(|pw| (pw >> 12) & 0xF).collect();
    let r_vals: Vec<u16> = p_weights.iter().map(|pw| (pw >> 8) & 0xF).collect();
    let v_vals: Vec<u16> = p_weights.iter().map(|pw| (pw >> 5) & 0x7).collect();
    let a_vals: Vec<u16> = p_weights.iter().map(|pw| (pw >> 2) & 0x7).collect();
    let t_vals: Vec<u16> = p_weights.iter().map(|pw| pw & 0x3).collect();

    fn dim_mode_n(values: &[u16], max: usize) -> u16 {
        let mut freq = vec![0u32; max];
        for &v in values {
            freq[v as usize] += 1;
        }
        freq.iter()
            .enumerate()
            .max_by_key(|(_, &c)| c)
            .map(|(i, _)| i as u16)
            .unwrap_or(0)
    }

    let s = dim_mode_n(&s_vals, 16);
    let r = dim_mode_n(&r_vals, 16);
    let v = dim_mode_n(&v_vals, 8);
    let a = dim_mode_n(&a_vals, 8);
    let t = dim_mode_n(&t_vals, 4);

    (s << 12) | (r << 8) | (v << 5) | (a << 2) | t
}

/// Parse block range string "2190..21FF" → (start, end) as u32
fn parse_range(range: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = range.split("..").collect();
    if parts.len() != 2 {
        return None;
    }
    let start = u32::from_str_radix(parts[0], 16).ok()?;
    let end = u32::from_str_radix(parts[1], 16).ok()?;
    Some((start, end))
}

#[allow(clippy::needless_range_loop)]
fn generate_knowtree_topology(src: &mut String, udc: &UdcJson, entries: &[Entry]) {
    // Group order (deterministic)
    let group_order = ["SDF", "MATH", "EMOTICON", "MUSICAL"];

    // Collect blocks sorted by group then by range start
    struct BlockInfo {
        name: String,
        group: String,
        range_start: u32,
        range_end: u32,
    }

    let mut blocks: Vec<BlockInfo> = Vec::new();
    for b in &udc.blocks {
        if let Some((start, end)) = parse_range(&b.range) {
            blocks.push(BlockInfo {
                name: b.name.clone(),
                group: b.group.clone(),
                range_start: start,
                range_end: end,
            });
        }
    }

    // Sort: group order first, then range start
    blocks.sort_by(|a, b| {
        let ga = group_order.iter().position(|&g| g == a.group).unwrap_or(99);
        let gb = group_order.iter().position(|&g| g == b.group).unwrap_or(99);
        ga.cmp(&gb).then(a.range_start.cmp(&b.range_start))
    });

    // Map characters to blocks
    // For each block, find characters whose codepoint falls in [range_start, range_end]
    // entries is sorted by cp
    struct BlockWithChars {
        name: String,
        group_idx: usize,
        chars_start: u16,
        chars_count: u16,
        char_p_weights: Vec<u16>,
    }

    let mut block_entries: Vec<BlockWithChars> = Vec::new();

    for blk in &blocks {
        let group_idx = group_order.iter().position(|&g| g == blk.group).unwrap_or(0);

        // Find chars in this block's range via binary search on sorted entries
        // entries is sorted by cp, same order as UCD_TABLE
        let start_pos = entries.partition_point(|e| e.cp < blk.range_start);
        let end_pos = entries.partition_point(|e| e.cp <= blk.range_end);
        let count = (end_pos - start_pos) as u16;

        let char_pws: Vec<u16> = entries[start_pos..end_pos]
            .iter()
            .map(|e| e.p_weight)
            .collect();

        block_entries.push(BlockWithChars {
            name: blk.name.clone(),
            group_idx,
            chars_start: start_pos as u16, // index into UCD_TABLE
            chars_count: count,
            char_p_weights: char_pws,
        });
    }

    // Build group → block indices
    let mut group_block_indices: Vec<Vec<u16>> = vec![Vec::new(); group_order.len()];
    for (bi, be) in block_entries.iter().enumerate() {
        group_block_indices[be.group_idx].push(bi as u16);
    }

    // Compute block aggregate P_weights
    let block_p_weights: Vec<u16> = block_entries
        .iter()
        .map(|be| mode_p_weight(&be.char_p_weights))
        .collect();

    // Compute group aggregate P_weights (mode over all block P_weights in group)
    let group_p_weights: Vec<u16> = group_block_indices
        .iter()
        .map(|block_idxs| {
            let pws: Vec<u16> = block_idxs.iter().map(|&bi| block_p_weights[bi as usize]).collect();
            mode_p_weight(&pws)
        })
        .collect();

    // ── Generate KNOWTREE_GROUPS ────────────────────────────────────────
    writeln!(src, "/// KnowTree group definitions (L0). (name, aggregate_p_weight, &[block_indices])").unwrap();
    writeln!(src, "pub static KNOWTREE_GROUPS: &[(&str, u16, &[u16])] = &[").unwrap();
    for (gi, gname) in group_order.iter().enumerate() {
        let pw = group_p_weights[gi];
        let idxs = &group_block_indices[gi];
        let idx_str: Vec<String> = idxs.iter().map(|i| i.to_string()).collect();
        writeln!(
            src,
            "    (\"{}\", 0x{:04X}, &[{}]),",
            gname,
            pw,
            idx_str.join(", ")
        )
        .unwrap();
    }
    writeln!(src, "];").unwrap();
    writeln!(src).unwrap();

    // ── Generate KNOWTREE_BLOCKS ────────────────────────────────────────
    writeln!(src, "/// KnowTree block definitions (L1). (name, aggregate_p_weight, chars_start_idx, chars_count)").unwrap();
    writeln!(src, "pub static KNOWTREE_BLOCKS: &[(&str, u16, u16, u16)] = &[").unwrap();
    for (bi, be) in block_entries.iter().enumerate() {
        let safe_name = be.name.replace('\\', "\\\\").replace('"', "\\\"");
        writeln!(
            src,
            "    (\"{}\", 0x{:04X}, {}, {}),",
            safe_name,
            block_p_weights[bi],
            be.chars_start,
            be.chars_count,
        )
        .unwrap();
    }
    writeln!(src, "];").unwrap();
    writeln!(src).unwrap();

    // ── Generate KNOWTREE_GROUP_COUNT / KNOWTREE_BLOCK_COUNT ────────────
    writeln!(
        src,
        "/// Number of groups in KnowTree hierarchy."
    )
    .unwrap();
    writeln!(src, "pub const KNOWTREE_GROUP_COUNT: usize = {};", group_order.len()).unwrap();
    writeln!(src).unwrap();
    writeln!(
        src,
        "/// Number of blocks in KnowTree hierarchy."
    )
    .unwrap();
    writeln!(src, "pub const KNOWTREE_BLOCK_COUNT: usize = {};", block_entries.len()).unwrap();
    writeln!(src).unwrap();

    let total_chars_mapped: u16 = block_entries.iter().map(|b| b.chars_count).sum();
    eprintln!(
        "cargo:warning=KnowTree topology: {} groups, {} blocks, {} chars mapped",
        group_order.len(),
        block_entries.len(),
        total_chars_mapped,
    );
}

fn write_empty() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = out_dir.join("ucd_generated.rs");
    fs::write(
        out_file,
        r#"
#[derive(Clone,Copy,Debug)]
#[allow(missing_docs)]
pub struct UcdEntry {
    pub cp:u32, pub group:u8, pub shape:u8, pub relation:u8,
    pub valence:u8, pub arousal:u8, pub time:u8,
    pub p_weight:u16, pub hash:u64, pub name:&'static str,
}
pub static UCD_TABLE: &[UcdEntry] = &[];
#[cfg(feature = "reverse-index")]
pub static HASH_TO_CP: &[(u64,u32)] = &[];
#[cfg(feature = "reverse-index")]
pub static CP_BUCKET_DATA: &[u32] = &[];
#[cfg(feature = "reverse-index")]
pub static CP_BUCKET_INDEX: &[(u8,u8,u32,u32)] = &[];
pub static SDF_PRIMITIVES: &[(u32,u8)] = &[];
pub static RELATION_PRIMITIVES: &[(u32,u8)] = &[];
pub static KNOWTREE_GROUPS: &[(&str, u16, &[u16])] = &[];
pub static KNOWTREE_BLOCKS: &[(&str, u16, u16, u16)] = &[];
pub const KNOWTREE_GROUP_COUNT: usize = 0;
pub const KNOWTREE_BLOCK_COUNT: usize = 0;
"#,
    )
    .unwrap();
}
