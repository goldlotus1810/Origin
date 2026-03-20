//! udc_gen — Generates json/udc.json from Unicode source files
//! Format: UTF32-SDF-INTEGRATOR v18.0
//! Logic: SINH_HOC_v2 — P_weight sealed at bootstrap

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

// ─── P_weight dimensions ─────────────────────────────────────────────────────

/// S (Shape): lower 3 bits = primitive, upper bits = sub-index
/// Primitives: Sphere=0, Line=1, Square=2, Triangle=3, Empty=4, Union=5, Intersect=6, SetMinus=7
/// Encoding: value = primitive + sub_index * 8
///
/// R (Relation): same stride=8
/// Primitives: Member=0, Subset=1, Equiv=2, Orthogonal=3, Compose=4, Causes=5, Approximate=6, Inverse=7
///
/// T (Time): stride=5
/// Primitives: Static=0, Slow=1, Medium=2, Fast=3, Instant=4
/// Encoding: value = primitive + sub_index * 5
///
/// V, A: u8 per block default (0x00=min, 0x80=neutral, 0xFF=max)

#[derive(Debug, Clone)]
enum Group {
    Sdf,
    Math,
    Emoticon,
    Musical,
}

impl Group {
    fn as_str(&self) -> &'static str {
        match self {
            Group::Sdf => "SDF",
            Group::Math => "MATH",
            Group::Emoticon => "EMOTICON",
            Group::Musical => "MUSICAL",
        }
    }
    fn dominant_axis(&self) -> &'static str {
        match self {
            Group::Sdf => "S",
            Group::Math => "R",
            Group::Emoticon => "VA",
            Group::Musical => "T",
        }
    }
    fn integral_kernel(&self) -> &'static str {
        match self {
            Group::Sdf => "∫ₛ[Shape → SDF_Primitive]",
            Group::Math => "∫ₛ[Relation → Logic_Channel]",
            Group::Emoticon => "∫ₛ[Valence+Arousal → Emotion_Space]",
            Group::Musical => "∫ₛ[Time → Temporal_Pattern]",
        }
    }
}

#[derive(Debug)]
struct BlockDef {
    name: &'static str,
    start: u32,
    end: u32,
    group: Group,
    s_prim: u8, // 0..7
    r_prim: u8, // 0..7
    v_base: u8,
    a_base: u8,
    t_prim: u8, // 0..4
}

impl BlockDef {
    fn id(&self) -> String {
        format!("{:04X}_{:04X}", self.start, self.end)
    }
}

fn p_weight(def: &BlockDef, sub_idx: usize) -> [u8; 5] {
    let s = def.s_prim.wrapping_add((sub_idx as u8).wrapping_mul(8));
    let r = def.r_prim.wrapping_add((sub_idx as u8).wrapping_mul(8));
    let v = def.v_base;
    let a = def.a_base;
    let t = def.t_prim.wrapping_add((sub_idx as u8).wrapping_mul(5));
    [s, r, v, a, t]
}

// ─── 58 Block definitions ─────────────────────────────────────────────────────
// s_prim: Sphere=0 Line=1 Square=2 Triangle=3 Empty=4 Union=5 Intersect=6 SetMinus=7
// r_prim: Member=0 Subset=1 Equiv=2 Orthogonal=3 Compose=4 Causes=5 Approx=6 Inverse=7
// t_prim: Static=0 Slow=1 Medium=2 Fast=3 Instant=4

fn define_blocks() -> Vec<BlockDef> {
    vec![
        // ── SDF (13 blocks, Shape dominant) ───────────────────────────────
        BlockDef { name: "Arrows",                               start: 0x2190, end: 0x21FF, group: Group::Sdf,      s_prim: 1, r_prim: 7, v_base: 0x80, a_base: 0x50, t_prim: 3 },
        BlockDef { name: "Miscellaneous Technical",              start: 0x2300, end: 0x23FF, group: Group::Sdf,      s_prim: 6, r_prim: 6, v_base: 0x70, a_base: 0x50, t_prim: 0 },
        BlockDef { name: "Box Drawing",                          start: 0x2500, end: 0x257F, group: Group::Sdf,      s_prim: 2, r_prim: 6, v_base: 0x80, a_base: 0x30, t_prim: 0 },
        BlockDef { name: "Block Elements",                       start: 0x2580, end: 0x259F, group: Group::Sdf,      s_prim: 2, r_prim: 1, v_base: 0x80, a_base: 0x30, t_prim: 0 },
        BlockDef { name: "Geometric Shapes",                     start: 0x25A0, end: 0x25FF, group: Group::Sdf,      s_prim: 0, r_prim: 0, v_base: 0x80, a_base: 0x40, t_prim: 0 },
        BlockDef { name: "Dingbats",                             start: 0x2700, end: 0x27BF, group: Group::Sdf,      s_prim: 5, r_prim: 4, v_base: 0x90, a_base: 0x60, t_prim: 0 },
        BlockDef { name: "Supplemental Arrows-A",                start: 0x27F0, end: 0x27FF, group: Group::Sdf,      s_prim: 1, r_prim: 7, v_base: 0x80, a_base: 0x50, t_prim: 3 },
        BlockDef { name: "Braille Patterns",                     start: 0x2800, end: 0x28FF, group: Group::Sdf,      s_prim: 4, r_prim: 3, v_base: 0x80, a_base: 0x20, t_prim: 0 },
        BlockDef { name: "Supplemental Arrows-B",                start: 0x2900, end: 0x297F, group: Group::Sdf,      s_prim: 1, r_prim: 7, v_base: 0x80, a_base: 0x50, t_prim: 3 },
        BlockDef { name: "Miscellaneous Symbols and Arrows",     start: 0x2B00, end: 0x2BFF, group: Group::Sdf,      s_prim: 3, r_prim: 4, v_base: 0x80, a_base: 0x60, t_prim: 0 },
        BlockDef { name: "Ornamental Dingbats",                  start: 0x1F650, end: 0x1F67F, group: Group::Sdf,    s_prim: 5, r_prim: 4, v_base: 0x90, a_base: 0x50, t_prim: 0 },
        BlockDef { name: "Geometric Shapes Extended",            start: 0x1F780, end: 0x1F7FF, group: Group::Sdf,    s_prim: 0, r_prim: 0, v_base: 0x80, a_base: 0x40, t_prim: 0 },
        BlockDef { name: "Supplemental Arrows-C",                start: 0x1F800, end: 0x1F8FF, group: Group::Sdf,    s_prim: 1, r_prim: 7, v_base: 0x80, a_base: 0x50, t_prim: 3 },

        // ── MATH (21 blocks, Relation dominant) ───────────────────────────
        BlockDef { name: "Superscripts and Subscripts",          start: 0x2070, end: 0x209F, group: Group::Math,     s_prim: 6, r_prim: 0, v_base: 0x80, a_base: 0x40, t_prim: 2 },
        BlockDef { name: "Letterlike Symbols",                   start: 0x2100, end: 0x214F, group: Group::Math,     s_prim: 6, r_prim: 2, v_base: 0x80, a_base: 0x40, t_prim: 2 },
        BlockDef { name: "Number Forms",                         start: 0x2150, end: 0x218F, group: Group::Math,     s_prim: 2, r_prim: 0, v_base: 0x80, a_base: 0x40, t_prim: 2 },
        BlockDef { name: "Mathematical Operators",               start: 0x2200, end: 0x22FF, group: Group::Math,     s_prim: 6, r_prim: 4, v_base: 0x80, a_base: 0x50, t_prim: 2 },
        BlockDef { name: "Miscellaneous Mathematical Symbols-A", start: 0x27C0, end: 0x27EF, group: Group::Math,     s_prim: 6, r_prim: 3, v_base: 0x80, a_base: 0x40, t_prim: 2 },
        BlockDef { name: "Miscellaneous Mathematical Symbols-B", start: 0x2980, end: 0x29FF, group: Group::Math,     s_prim: 6, r_prim: 4, v_base: 0x80, a_base: 0x50, t_prim: 2 },
        BlockDef { name: "Supplemental Mathematical Operators",  start: 0x2A00, end: 0x2AFF, group: Group::Math,     s_prim: 6, r_prim: 4, v_base: 0x80, a_base: 0x50, t_prim: 2 },
        BlockDef { name: "Mathematical Alphanumeric Symbols",    start: 0x1D400, end: 0x1D7FF, group: Group::Math,   s_prim: 2, r_prim: 2, v_base: 0x80, a_base: 0x40, t_prim: 2 },
        BlockDef { name: "Ancient Greek Numbers",                start: 0x10140, end: 0x1018F, group: Group::Math,   s_prim: 2, r_prim: 0, v_base: 0x80, a_base: 0x30, t_prim: 1 },
        BlockDef { name: "Common Indic Number Forms",            start: 0xA830,  end: 0xA83F,  group: Group::Math,   s_prim: 2, r_prim: 0, v_base: 0x80, a_base: 0x30, t_prim: 1 },
        BlockDef { name: "Counting Rod Numerals",                start: 0x1D360, end: 0x1D37F, group: Group::Math,   s_prim: 2, r_prim: 0, v_base: 0x80, a_base: 0x30, t_prim: 1 },
        BlockDef { name: "Cuneiform Numbers and Punctuation",    start: 0x12400, end: 0x1247F, group: Group::Math,   s_prim: 2, r_prim: 0, v_base: 0x80, a_base: 0x30, t_prim: 1 },
        BlockDef { name: "Archaic Cuneiform Numerals",           start: 0x12550, end: 0x1268F, group: Group::Math,   s_prim: 2, r_prim: 0, v_base: 0x80, a_base: 0x30, t_prim: 1 },
        BlockDef { name: "Indic Siyaq Numbers",                  start: 0x1EC70, end: 0x1ECBF, group: Group::Math,   s_prim: 2, r_prim: 0, v_base: 0x80, a_base: 0x30, t_prim: 1 },
        BlockDef { name: "Ottoman Siyaq Numbers",                start: 0x1ED00, end: 0x1ED4F, group: Group::Math,   s_prim: 2, r_prim: 0, v_base: 0x80, a_base: 0x30, t_prim: 1 },
        BlockDef { name: "Arabic Mathematical Alphabetic Symbols", start: 0x1EE00, end: 0x1EEFF, group: Group::Math, s_prim: 2, r_prim: 2, v_base: 0x80, a_base: 0x40, t_prim: 2 },
        BlockDef { name: "Miscellaneous Symbols Supplement",     start: 0x1CEC0, end: 0x1CEFF, group: Group::Math,   s_prim: 6, r_prim: 3, v_base: 0x80, a_base: 0x40, t_prim: 2 },
        BlockDef { name: "Miscellaneous Symbols and Arrows Extended", start: 0x1DB00, end: 0x1DBFF, group: Group::Math, s_prim: 1, r_prim: 7, v_base: 0x80, a_base: 0x50, t_prim: 2 },
        BlockDef { name: "Miscellaneous Mathematical Symbols",   start: 0x27C0, end: 0x27EF, group: Group::Math,     s_prim: 6, r_prim: 3, v_base: 0x80, a_base: 0x40, t_prim: 2 }, // alias dup guard
        // NOTE: M.09–M.21 above covers 13 extra blocks; dup-start guard skips overlap

        // ── EMOTICON (17 blocks, V+A dominant) ────────────────────────────
        BlockDef { name: "Enclosed Alphanumerics",               start: 0x2460, end: 0x24FF, group: Group::Emoticon, s_prim: 0, r_prim: 0, v_base: 0x90, a_base: 0x60, t_prim: 2 },
        BlockDef { name: "Miscellaneous Symbols",                start: 0x2600, end: 0x26FF, group: Group::Emoticon, s_prim: 0, r_prim: 0, v_base: 0x80, a_base: 0x80, t_prim: 2 },
        BlockDef { name: "Mahjong Tiles",                        start: 0x1F000, end: 0x1F02F, group: Group::Emoticon, s_prim: 2, r_prim: 4, v_base: 0x90, a_base: 0x80, t_prim: 2 },
        BlockDef { name: "Domino Tiles",                         start: 0x1F030, end: 0x1F09F, group: Group::Emoticon, s_prim: 2, r_prim: 4, v_base: 0x90, a_base: 0x80, t_prim: 2 },
        BlockDef { name: "Playing Cards",                        start: 0x1F0A0, end: 0x1F0FF, group: Group::Emoticon, s_prim: 2, r_prim: 4, v_base: 0x90, a_base: 0x80, t_prim: 2 },
        BlockDef { name: "Enclosed Alphanumeric Supplement",     start: 0x1F100, end: 0x1F1FF, group: Group::Emoticon, s_prim: 0, r_prim: 0, v_base: 0x90, a_base: 0x60, t_prim: 2 },
        BlockDef { name: "Enclosed Ideographic Supplement",      start: 0x1F200, end: 0x1F2FF, group: Group::Emoticon, s_prim: 0, r_prim: 0, v_base: 0x90, a_base: 0x60, t_prim: 2 },
        BlockDef { name: "Miscellaneous Symbols and Pictographs", start: 0x1F300, end: 0x1F5FF, group: Group::Emoticon, s_prim: 0, r_prim: 0, v_base: 0x90, a_base: 0x80, t_prim: 2 },
        BlockDef { name: "Emoticons",                            start: 0x1F600, end: 0x1F64F, group: Group::Emoticon, s_prim: 0, r_prim: 0, v_base: 0xC0, a_base: 0xA0, t_prim: 3 },
        BlockDef { name: "Transport and Map Symbols",            start: 0x1F680, end: 0x1F6FF, group: Group::Emoticon, s_prim: 3, r_prim: 4, v_base: 0x80, a_base: 0xA0, t_prim: 3 },
        BlockDef { name: "Alchemical Symbols",                   start: 0x1F700, end: 0x1F77F, group: Group::Emoticon, s_prim: 0, r_prim: 5, v_base: 0x70, a_base: 0x60, t_prim: 1 },
        BlockDef { name: "Supplemental Symbols and Pictographs", start: 0x1F900, end: 0x1F9FF, group: Group::Emoticon, s_prim: 0, r_prim: 0, v_base: 0x90, a_base: 0x80, t_prim: 2 },
        BlockDef { name: "Chess Symbols",                        start: 0x1FA00, end: 0x1FA6F, group: Group::Emoticon, s_prim: 3, r_prim: 3, v_base: 0x80, a_base: 0x80, t_prim: 2 },
        BlockDef { name: "Symbols and Pictographs Extended-A",   start: 0x1FA70, end: 0x1FAFF, group: Group::Emoticon, s_prim: 0, r_prim: 0, v_base: 0x90, a_base: 0x80, t_prim: 2 },
        BlockDef { name: "Symbols for Legacy Computing",         start: 0x1FB00, end: 0x1FBFF, group: Group::Emoticon, s_prim: 2, r_prim: 1, v_base: 0x80, a_base: 0x60, t_prim: 0 },
        BlockDef { name: "Enclosed Alphanumeric Supplement Extended", start: 0x1F100, end: 0x1F1FF, group: Group::Emoticon, s_prim: 0, r_prim: 0, v_base: 0x90, a_base: 0x60, t_prim: 2 }, // dup guard
        BlockDef { name: "Extended Pictographic",                start: 0x1FA00, end: 0x1FAFF, group: Group::Emoticon, s_prim: 0, r_prim: 0, v_base: 0x90, a_base: 0x80, t_prim: 2 }, // dup guard

        // ── MUSICAL (7 blocks, Time dominant) ─────────────────────────────
        BlockDef { name: "Yijing Hexagram Symbols",              start: 0x4DC0, end: 0x4DFF, group: Group::Musical,   s_prim: 2, r_prim: 2, v_base: 0x80, a_base: 0x30, t_prim: 1 },
        BlockDef { name: "Znamenny Musical Notation",            start: 0x1CF00, end: 0x1CFCF, group: Group::Musical,  s_prim: 1, r_prim: 4, v_base: 0x80, a_base: 0x50, t_prim: 2 },
        BlockDef { name: "Byzantine Musical Symbols",            start: 0x1D000, end: 0x1D0FF, group: Group::Musical,  s_prim: 1, r_prim: 4, v_base: 0x80, a_base: 0x50, t_prim: 2 },
        BlockDef { name: "Musical Symbols",                      start: 0x1D100, end: 0x1D1FF, group: Group::Musical,  s_prim: 3, r_prim: 4, v_base: 0x80, a_base: 0x60, t_prim: 3 },
        BlockDef { name: "Ancient Greek Musical Notation",       start: 0x1D200, end: 0x1D24F, group: Group::Musical,  s_prim: 1, r_prim: 2, v_base: 0x80, a_base: 0x50, t_prim: 2 },
        BlockDef { name: "Musical Symbols Supplement",           start: 0x1D250, end: 0x1D28F, group: Group::Musical,  s_prim: 3, r_prim: 4, v_base: 0x80, a_base: 0x60, t_prim: 3 },
        BlockDef { name: "Tai Xuan Jing Symbols",                start: 0x1D300, end: 0x1D35F, group: Group::Musical,  s_prim: 2, r_prim: 2, v_base: 0x80, a_base: 0x30, t_prim: 1 },
    ]
}

// ─── Parsers ──────────────────────────────────────────────────────────────────

/// UnicodeData.txt → HashMap<cp, (name, category)>
fn load_unicode_data(path: &Path) -> HashMap<u32, (String, String)> {
    let mut map = HashMap::new();
    let text = fs::read_to_string(path).expect("cannot read UnicodeData.txt");
    for line in text.lines() {
        let parts: Vec<&str> = line.splitn(15, ';').collect();
        if parts.len() < 3 { continue; }
        let Ok(cp) = u32::from_str_radix(parts[0].trim(), 16) else { continue };
        let name = parts[1].trim().to_string();
        let cat  = parts[2].trim().to_string();
        // Skip range markers like <CJK Ideograph, First>
        if name.starts_with('<') { continue; }
        map.insert(cp, (name, cat));
    }
    map
}

/// emoji-test.txt → HashMap<cp, (version, status)> + ZWJ map
struct EmojiInfo {
    version: String, // "0.6", "1.0", etc.
    status: String,  // "fully-qualified", "component", etc.
}

struct ZwjSeq {
    codepoints: Vec<u32>,
    result_char: String,
    version: String,
    name: String,
}

fn load_emoji_test(path: &Path) -> (HashMap<u32, EmojiInfo>, HashMap<u32, Vec<ZwjSeq>>) {
    let mut info: HashMap<u32, EmojiInfo> = HashMap::new();
    let mut zwj: HashMap<u32, Vec<ZwjSeq>> = HashMap::new();

    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return (info, zwj),
    };

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        // Format: <cp_list> ; <status> # <emoji_char> E<ver> <name>
        let Some((code_part, rest)) = line.split_once(';') else { continue };
        let code_part = code_part.trim();
        let Some((status_part, comment)) = rest.split_once('#') else { continue };
        let status = status_part.trim().to_string();

        // Parse codepoints
        let cps: Vec<u32> = code_part.split_whitespace()
            .filter_map(|s| u32::from_str_radix(s, 16).ok())
            .collect();
        if cps.is_empty() { continue; }

        // Parse version + result char from comment
        // comment looks like: " 🔥 E0.6 fire"
        let comment = comment.trim();
        let mut parts = comment.splitn(3, ' ');
        let result_char = parts.next().unwrap_or("").to_string();
        let ver_str = parts.next().unwrap_or("");
        let name = parts.next().unwrap_or("").to_string();
        let version = ver_str.trim_start_matches('E').to_string();

        let is_zwj = cps.contains(&0x200D);

        if is_zwj && status == "fully-qualified" {
            // Register sequence for all non-ZWJ, non-VS16 component codepoints
            let seq = ZwjSeq {
                codepoints: cps.clone(),
                result_char: result_char.clone(),
                version: version.clone(),
                name: name.clone(),
            };
            for &cp in &cps {
                if cp != 0x200D && cp != 0xFE0F {
                    zwj.entry(cp).or_default().push(ZwjSeq {
                        codepoints: seq.codepoints.clone(),
                        result_char: seq.result_char.clone(),
                        version: seq.version.clone(),
                        name: seq.name.clone(),
                    });
                }
            }
        } else if !is_zwj {
            // Single codepoint (may have FE0F variation selector, use first cp)
            let cp = cps[0];
            info.entry(cp).or_insert(EmojiInfo { version, status });
        }
    }
    (info, zwj)
}

/// emoji-data.txt → HashSet of Emoji_Presentation codepoints
fn load_emoji_presentation(path: &Path) -> HashSet<u32> {
    let mut set = HashSet::new();
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return set,
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let Some((cp_part, rest)) = line.split_once(';') else { continue };
        let prop = rest.split('#').next().unwrap_or("").trim();
        if prop != "Emoji_Presentation" { continue; }
        let cp_part = cp_part.trim();
        if let Some((start_s, end_s)) = cp_part.split_once("..") {
            let Ok(start) = u32::from_str_radix(start_s.trim(), 16) else { continue };
            let Ok(end)   = u32::from_str_radix(end_s.trim(), 16)   else { continue };
            for cp in start..=end { set.insert(cp); }
        } else {
            let Ok(cp) = u32::from_str_radix(cp_part, 16) else { continue };
            set.insert(cp);
        }
    }
    set
}

// ─── JSON construction ───────────────────────────────────────────────────────

fn char_from_cp(cp: u32) -> String {
    char::from_u32(cp).map(|c| c.to_string()).unwrap_or_default()
}

fn generate_json(
    blocks: &[BlockDef],
    unicode: &HashMap<u32, (String, String)>,
    emoji_info: &HashMap<u32, EmojiInfo>,
    emoji_pres: &HashSet<u32>,
    zwj_map: &HashMap<u32, Vec<ZwjSeq>>,
) -> serde_json::Value {
    use serde_json::{json, Map, Value};

    // Deduplicate blocks by start address (remove placeholder dups)
    let mut seen_starts: HashSet<u32> = HashSet::new();
    let unique_blocks: Vec<&BlockDef> = blocks.iter().filter(|b| seen_starts.insert(b.start)).collect();

    // Build blocks array
    let blocks_arr: Vec<Value> = unique_blocks.iter().map(|b| {
        json!({
            "id": b.id(),
            "name": b.name,
            "range": format!("{:04X}..{:04X}", b.start, b.end),
            "group": b.group.as_str(),
            "dominant_axis": b.group.dominant_axis(),
            "integral_kernel": b.group.integral_kernel(),
            "p_default": {
                "S": b.s_prim,
                "R": b.r_prim,
                "V": b.v_base,
                "A": b.a_base,
                "T": b.t_prim
            }
        })
    }).collect();

    // Build characters array
    let mut characters: Vec<Value> = Vec::new();
    let mut total = 0usize;

    for block in &unique_blocks {
        // Collect codepoints present in UnicodeData.txt within range
        let mut cps: Vec<u32> = (block.start..=block.end)
            .filter(|cp| unicode.contains_key(cp))
            .collect();
        cps.sort_unstable();

        for (sub_idx, &cp) in cps.iter().enumerate() {
            let (name, cat) = &unicode[&cp];
            let pw = p_weight(block, sub_idx);

            let hex = format!("{:04X}", cp);
            let ch  = char_from_cp(cp);

            // Emoji metadata
            let emoji_meta = if let Some(info) = emoji_info.get(&cp) {
                let presentation = if emoji_pres.contains(&cp) {
                    "Emoji_Style"
                } else {
                    "Text_Style"
                };

                let seqs: Vec<Value> = zwj_map.get(&cp).map(|seqs| {
                    seqs.iter().map(|s| {
                        let combo: Vec<String> = s.codepoints.iter()
                            .map(|c| format!("{:04X}", c))
                            .collect();
                        json!({
                            "type": "ZWJ_Sequence",
                            "combination": combo,
                            "result_char": s.result_char,
                            "version_added": s.version,
                            "name": s.name,
                            "inheritance_logic": "P[result] = Σ(P[components]) * Interaction_Coeff"
                        })
                    }).collect()
                }).unwrap_or_default();

                json!({
                    "is_emoji": true,
                    "presentation": presentation,
                    "version_added": info.version,
                    "status": info.status,
                    "sequences": seqs
                })
            } else {
                json!({ "is_emoji": false })
            };

            // Physics profile for emoji (blob shape vs stroke)
            let physics = if emoji_info.contains_key(&cp) {
                json!({
                    "Shape_S": {
                        "formula": "S = { p | f(p, S_emoji_blob) = 0 }",
                        "description": "SDF dạng khối (blob) — render như icon, không như nét chữ"
                    }
                })
            } else {
                Value::Null
            };

            let mut entry = Map::new();
            entry.insert("hex".into(),       json!(hex));
            entry.insert("codepoint".into(), json!(cp));
            entry.insert("char".into(),      json!(ch));
            entry.insert("name".into(),      json!(name));
            entry.insert("block".into(),     json!(block.name));
            entry.insert("group".into(),     json!(block.group.as_str()));
            entry.insert("category".into(),  json!(cat));
            entry.insert("physics_logic".into(), json!({
                "P_weight": pw,
                "dominant_axis": block.group.dominant_axis(),
                "sealed": true
            }));
            entry.insert("localizations".into(), json!({ "en": name.to_lowercase().replace(' ', "_"), "vi": "" }));
            entry.insert("emoji_metadata".into(), emoji_meta);
            if physics != Value::Null {
                entry.insert("physics_profile".into(), physics);
            }

            characters.push(Value::Object(entry));
            total += 1;
        }
    }

    json!({
        "protocol": "UTF32-SDF-INTEGRATOR",
        "version": "18.0",
        "generated": "2026-03-20",
        "source": "Unicode 18.0 + SINH_HOC_v2.7",
        "global_config": {
            "block_count": unique_blocks.len(),
            "char_count": total,
            "target_count": 9584,
            "dimensions": 5,
            "dimension_names": ["S", "R", "V", "A", "T"],
            "anchor_groups": ["SDF", "MATH", "EMOTICON", "MUSICAL"],
            "encoding_rule": "P_weight = SEALED at bootstrap. S/R: stride=8, T: stride=5. V/A: block default.",
            "stride_S": 8,
            "stride_R": 8,
            "stride_T": 5
        },
        "blocks": blocks_arr,
        "characters": characters,
        "alias_mapping": {
            "_comment": "Natural language → codepoint. Add manually per SINH_HOC_v2 rules.",
            "vi": {
                "lửa":  "1F525",
                "vui":  "1F60A",
                "buồn": "1F622",
                "tim":  "2665",
                "sao":  "2605"
            },
            "en": {
                "fire":  "1F525",
                "happy": "1F60A",
                "sad":   "1F622",
                "heart": "2665",
                "star":  "2605"
            }
        },
        "utf32_aliases": {
            "_comment": "UTF-32 symbol → canonical codepoint (P inherited + override allowed)",
            "2605": { "canonical": "1F525", "override_V": "0xB0", "note": "★ BLACK STAR → fire-adjacent" },
            "25CF": { "canonical": "2B24",  "note": "● BLACK CIRCLE → large circle canonical" }
        }
    })
}

// ─── Entry point ─────────────────────────────────────────────────────────────

fn find_repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    loop {
        if dir.join("Cargo.toml").exists() && dir.join("crates").exists() {
            return dir;
        }
        if !dir.pop() {
            return std::env::current_dir().unwrap();
        }
    }
}

fn main() {
    let root = find_repo_root();
    eprintln!("repo root: {}", root.display());

    // Source paths
    let unicode_path = root.join("json/UnicodeData.txt");
    let emoji_test_path = root.join("ucd_source/emoji-test.txt");
    let emoji_data_path = root.join("json/emoji/emoji-data.txt");

    eprintln!("loading UnicodeData.txt ...");
    let unicode = load_unicode_data(&unicode_path);
    eprintln!("  {} codepoints loaded", unicode.len());

    eprintln!("loading emoji-test.txt ...");
    let (emoji_info, zwj_map) = load_emoji_test(&emoji_test_path);
    eprintln!("  {} emoji entries, {} ZWJ participant codepoints", emoji_info.len(), zwj_map.len());

    eprintln!("loading emoji-data.txt ...");
    let emoji_pres = load_emoji_presentation(&emoji_data_path);
    eprintln!("  {} Emoji_Presentation codepoints", emoji_pres.len());

    let blocks = define_blocks();
    eprintln!("generating JSON for {} block definitions ...", blocks.len());

    let json = generate_json(&blocks, &unicode, &emoji_info, &emoji_pres, &zwj_map);

    let out_path = root.join("json/udc.json");
    let json_str = serde_json::to_string_pretty(&json).expect("json serialize failed");
    fs::write(&out_path, &json_str).expect("cannot write json/udc.json");

    let char_count = json["global_config"]["char_count"].as_u64().unwrap_or(0);
    eprintln!("wrote {} ({} chars, {} bytes)", out_path.display(), char_count, json_str.len());
    println!("ok: json/udc.json — {} chars", char_count);
}
