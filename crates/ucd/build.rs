//! build.rs — đọc UnicodeData.txt lúc compile → sinh bảng tĩnh
//!
//! Output trong OUT_DIR/ucd_generated.rs:
//!   UCD_TABLE         — forward lookup (cp → Molecule bytes)
//!   HASH_TO_CP        — reverse index (chain_hash → cp), O(log n) decode
//!   CP_BUCKET         — bucket index (shape,relation → [cp]), top-n decode
//!   SDF_PRIMITIVES    — 8 SDF primitive mappings
//!   RELATION_PRIMITIVES — 8 Relation primitive mappings
//!
//! KHÔNG hardcode bất kỳ Molecule nào.
//! Mọi giá trị đến từ UnicodeData.txt.

use std::collections::HashMap;
use std::env;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::PathBuf;

// ─────────────────────────────────────────────────────────────────────────────
// 5 nhóm Unicode — ranges
// ─────────────────────────────────────────────────────────────────────────────

struct Group {
    byte: u8,
    _name: &'static str,
    ranges: &'static [(u32, u32)],
}

static GROUPS: &[Group] = &[
    Group {
        byte: 0x01, // SDF
        _name: "SDF",
        ranges: &[
            (0x2190, 0x21FF),   // Arrows
            (0x2500, 0x257F),   // Box Drawing
            (0x2580, 0x259F),   // Block Elements
            (0x25A0, 0x25FF),   // Geometric Shapes ← 8 primitives here
            (0x2700, 0x27BF),   // Dingbats
            (0x27F0, 0x27FF),   // Supplemental Arrows-A
            (0x2900, 0x297F),   // Supplemental Arrows-B
            (0x2B00, 0x2BFF),   // Misc Symbols+Arrows
            (0x1F780, 0x1F7FF), // Geometric Shapes Extended
            (0x1F800, 0x1F8FF), // Supplemental Arrows-C
        ],
    },
    Group {
        byte: 0x02, // MATH
        _name: "MATH",
        ranges: &[
            (0x2070, 0x209F),   // Superscripts and Subscripts
            (0x2100, 0x214F),   // Letterlike Symbols
            (0x2150, 0x218F),   // Number Forms
            (0x2200, 0x22FF),   // Mathematical Operators ← RELATION subset here
            (0x27C0, 0x27EF),   // Misc Math Symbols-A
            (0x2980, 0x29FF),   // Misc Math Symbols-B
            (0x2A00, 0x2AFF),   // Supplemental Math Operators
            (0x1D400, 0x1D7FF), // Math Alphanumeric Symbols
        ],
    },
    Group {
        byte: 0x03, // EMOTICON
        _name: "EMOTICON",
        ranges: &[
            (0x23F0, 0x23FF),   // Clocks + misc technical (⏰⏱⏲⏳)
            (0x2600, 0x26FF),   // Miscellaneous Symbols
            (0x1F300, 0x1F5FF), // Misc Symbols and Pictographs
            (0x1F600, 0x1F64F), // Emoticons (faces)
            (0x1F680, 0x1F6FF), // Transport and Map Symbols
            (0x1F700, 0x1F77F), // Alchemical Symbols
            (0x1F900, 0x1F9FF), // Supplemental Symbols and Pictographs
            (0x1FA70, 0x1FAFF), // Symbols and Pictographs Extended-A
        ],
    },
    Group {
        byte: 0x04, // MUSICAL
        _name: "MUSICAL",
        ranges: &[
            (0x1D100, 0x1D1FF), // Musical Symbols (CORE)
            (0x4DC0, 0x4DFF),   // Yijing Hexagram Symbols (64 states)
            (0x1D300, 0x1D35F), // Tai Xuan Jing Symbols (81 states)
        ],
    },
];

// ─────────────────────────────────────────────────────────────────────────────
// 8 SDF Primitives — từ Geometric Shapes 25A0..25FF
// ─────────────────────────────────────────────────────────────────────────────

static SDF_PRIMS: &[(u32, u8, &str)] = &[
    (0x25CF, 0x01, "BLACK CIRCLE"),               // ● Sphere
    (0x25AC, 0x02, "BLACK RECTANGLE"),            // ▬ Capsule
    (0x25A0, 0x03, "BLACK SQUARE"),               // ■ Box
    (0x25B2, 0x04, "BLACK UP-POINTING TRIANGLE"), // ▲ Cone
    (0x25CB, 0x05, "WHITE CIRCLE"),               // ○ Torus
    (0x222A, 0x06, "UNION"),                      // ∪ Union
    (0x2229, 0x07, "INTERSECTION"),               // ∩ Intersect
    (0x2216, 0x08, "SET MINUS"),                  // ∖ Subtract
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
// Derive Molecule bytes từ codepoint + UCD data
// ─────────────────────────────────────────────────────────────────────────────

/// Shape byte từ codepoint.
/// Không hardcode — suy ra từ block và tên.
fn shape_of(cp: u32, name: &str) -> u8 {
    // Kiểm tra SDF primitive trực tiếp
    for &(pcp, pbyte, _) in SDF_PRIMS {
        if cp == pcp {
            return pbyte;
        }
    }
    // Geometric Shapes 25A0..25FF: shape theo sub-range
    if (0x25A0..=0x25FF).contains(&cp) {
        return match cp {
            0x25CB..=0x25CF => 0x01, // ○● → Sphere/Torus
            0x25A0..=0x25AB => 0x03, // ■ → Box
            0x25AC..=0x25AF => 0x02, // ▬ → Capsule
            0x25B2..=0x25C5 => 0x04, // ▲▼◀▶ → Cone (directional)
            0x25C6..=0x25CA => 0x07, // ◆ → Intersect (diamond = intersection)
            0x25D0..=0x25FF => 0x06, // ◐◑ → Union (partially filled)
            _ => 0x01,               // default Sphere
        };
    }
    // Block Elements 2580..259F: fill levels → Box shape
    if (0x2580..=0x259F).contains(&cp) {
        return 0x03;
    } // ░▒▓█ → Box
      // Arrows 2190..21FF → Cone (directional)
    if (0x2190..=0x21FF).contains(&cp) {
        return 0x04;
    }
    // Math Operators 2200..22FF
    if (0x2200..=0x22FF).contains(&cp) {
        // Set operations → Union/Intersect/Subtract
        if name.contains("UNION") {
            return 0x06;
        }
        if name.contains("INTERSECTION") {
            return 0x07;
        }
        if name.contains("MINUS") {
            return 0x08;
        }
        return 0x05; // Torus — vòng lặp toán học
    }
    // Letterlike 2100..214F → Torus (abstract loops)
    if (0x2100..=0x214F).contains(&cp) {
        return 0x05;
    }
    // EMOTICON → Sphere (entities are round)
    if (0x2600..=0x26FF).contains(&cp) {
        return 0x01;
    }
    if (0x1F300..=0x1FAFF).contains(&cp) {
        return 0x01;
    }
    // Musical → Torus (cycles)
    if (0x1D100..=0x1D1FF).contains(&cp) {
        return 0x05;
    }
    if (0x4DC0..=0x4DFF).contains(&cp) {
        return 0x05;
    }
    0x01 // default: Sphere
}

/// Relation byte từ codepoint.
fn relation_of(cp: u32, name: &str) -> u8 {
    // RELATION primitives trực tiếp
    for &(pcp, pbyte, _) in REL_PRIMS {
        if cp == pcp {
            return pbyte;
        }
    }
    // Math Operators 2200..22FF
    if (0x2200..=0x22FF).contains(&cp) {
        if name.contains("ELEMENT") {
            return 0x01;
        } // ∈ Member
        if name.contains("SUBSET") {
            return 0x02;
        } // ⊂ Subset
        if name.contains("IDENTICAL") {
            return 0x03;
        } // ≡ Equiv
        if name.contains("EQUAL") {
            return 0x03;
        }
        if name.contains("TACK") {
            return 0x04;
        } // ⊥ Orthogonal
        if name.contains("RING") {
            return 0x05;
        } // ∘ Compose
        if name.contains("ARROW") {
            return 0x06;
        } // → Causes
        if name.contains("ALMOST") {
            return 0x07;
        } // ≈ Similar
        if name.contains("UNION") {
            return 0x01;
        } // ∪ Member (contains)
        return 0x03; // Equiv default for math
    }
    // Arrows 2190..21FF
    if (0x2190..=0x21FF).contains(&cp) {
        if name.contains("LEFT") {
            return 0x08;
        } // ← DerivedFrom
        if name.contains("RIGHT") {
            return 0x06;
        } // → Causes
        if name.contains("UP") {
            return 0x06;
        } // ↑ Causes (upward)
        if name.contains("DOWN") {
            return 0x08;
        } // ↓ DerivedFrom (downward)
        return 0x06; // Causes default
    }
    // Block Elements → Member (fill = belonging to level)
    if (0x2580..=0x259F).contains(&cp) {
        return 0x01;
    }
    // EMOTICON → Member (entities belong to groups)
    if (0x2600..=0x26FF).contains(&cp) {
        return 0x01;
    }
    if (0x1F300..=0x1FAFF).contains(&cp) {
        return 0x01;
    }
    // Musical → Similar (notes relate to each other)
    if (0x1D100..=0x1D1FF).contains(&cp) {
        return 0x07;
    }
    0x01 // default: Member
}

/// Valence byte từ codepoint và tên Unicode.
fn valence_of(cp: u32, name: &str) -> u8 {
    // Block Elements = fill level → valence
    if (0x2580..=0x259F).contains(&cp) {
        return match cp {
            0x2588 => 0xFF, // █ full
            0x2593 => 0xC0, // ▓
            0x2592 => 0x80, // ▒
            0x2591 => 0x40, // ░
            _ => 0x00,
        };
    }
    // EMOTICON — face emotions
    if (0x1F600..=0x1F64F).contains(&cp) {
        // Positive faces 1F600..1F60F
        if (0x1F600..=0x1F60F).contains(&cp) {
            return 0xFF;
        }
        // Negative faces 1F620..1F62F
        if (0x1F620..=0x1F62F).contains(&cp) {
            return 0x00;
        }
        // Neutral 1F610..1F61F
        if (0x1F610..=0x1F61F).contains(&cp) {
            return 0x80;
        }
    }
    // Symbols by name
    if name.contains("FIRE") || name.contains("FLAME") {
        return 0xFF;
    }
    if name.contains("HEART") || name.contains("LOVE") {
        return 0xFF;
    }
    if name.contains("STAR") || name.contains("SPARKL") {
        return 0xE0;
    }
    if name.contains("SUN WITH FACE") {
        return 0xE8;
    }
    if name.contains("SUN") || name.contains("BRIGHT") {
        return 0xE0;
    }
    if name.contains("WARNING") || name.contains("DANGE") {
        return 0x20;
    }
    if name.contains("SKULL") || name.contains("DEATH") {
        return 0x00;
    }
    if name.contains("DROPLET") || name.contains("WATER") {
        return 0xC0;
    }
    if name.contains("SNOWFLAKE") || name.contains("COLD") {
        return 0x30;
    }
    if name.contains("BRAIN") {
        return 0xC0;
    }
    if name.contains("CHECK") {
        return 0xFF;
    }
    if name.contains("CROSS") {
        return 0x10;
    }
    // ── Codepoints hay collide vào default 0x80 ────────────────────────
    if name.contains("LIGHT BULB") || name.contains("ELECTRIC") {
        return 0xB0;
    }
    if name.contains("HIGH VOLTAGE") || name.contains("LIGHTNING") {
        return 0xA0;
    }
    if name.contains("BLOWING") || name.contains("WIND") {
        return 0x90;
    }
    if name.contains("SPEAKER") || name.contains("SOUND") {
        return 0x98;
    }
    if name.contains("BANDAGE") || name.contains("INJURED") {
        return 0x18;
    }
    if name.contains("FORK") || name.contains("KNIFE") {
        return 0x88;
    }
    if name.contains("POLICE") || name.contains("REVOLVING") {
        return 0x38;
    }
    if name.contains("GARDEN") {
        return 0xBC;
    }
    if name.contains("HOUSE") || name.contains("HOME") || name.contains("BUILDING") {
        return 0xA8;
    }
    if name.contains("TREE") || name.contains("DECIDUOUS") {
        return 0xA4;
    }
    if name.contains("WAVE") {
        return 0xB8;
    }
    if name.contains("GLOBE") || name.contains("EARTH") {
        return 0x94;
    }
    if name.contains("SILHOUETTE") || name.contains("BUST") {
        return 0x78;
    }
    if name.contains("EYE") {
        return 0x84;
    }
    if name.contains("PERMANENT") || name.contains("INFINITY") {
        return 0x7C;
    }
    if name.contains("OCTAGONAL") {
        return 0x28;
    }
    if name.contains("ALARM") && name.contains("CLOCK") {
        return 0x48;
    }
    if name.contains("OPEN LOCK") {
        return 0x8C;
    }
    if name.contains("LOCK") {
        return 0x58;
    }
    // Math → neutral
    if (0x2200..=0x22FF).contains(&cp) {
        return 0x80;
    }
    if (0x2100..=0x214F).contains(&cp) {
        return 0x80;
    }
    // Musical → slightly positive
    if (0x1D100..=0x1D1FF).contains(&cp) {
        return 0xA0;
    }
    0x80 // default neutral
}

/// Arousal byte từ codepoint và tên Unicode.
fn arousal_of(cp: u32, name: &str) -> u8 {
    // Musical dynamics → arousal trực tiếp
    if (0x1D100..=0x1D1FF).contains(&cp) {
        if name.contains("FORTISSIMO") || name.contains("FF") {
            return 0xFF;
        }
        if name.contains("FORTE") {
            return 0xC0;
        }
        if name.contains("MEZZO") {
            return 0x80;
        }
        if name.contains("PIANO") && name.contains("PIANO") {
            return 0x10;
        }
        if name.contains("PIANO") {
            return 0x40;
        }
        return 0x80;
    }
    // EMOTICON energy
    if (0x1F600..=0x1F64F).contains(&cp) {
        if (0x1F600..=0x1F60F).contains(&cp) {
            return 0xFF;
        } // excited positive
        if (0x1F620..=0x1F62F).contains(&cp) {
            return 0xE0;
        } // excited negative
        if (0x1F610..=0x1F61F).contains(&cp) {
            return 0x20;
        } // calm neutral
    }
    // By name
    if name.contains("FIRE") || name.contains("LIGHTNING") {
        return 0xFF;
    }
    if name.contains("WARNING") || name.contains("ALARM") {
        return 0xE0;
    }
    if name.contains("SNOWFLAKE") || name.contains("SLEEP") {
        return 0x10;
    }
    if name.contains("DROPLET") {
        return 0x40;
    }
    if name.contains("BRAIN") {
        return 0x60;
    }
    if name.contains("RUNNER") {
        return 0xD0;
    }
    if name.contains("STOP") {
        return 0x20;
    }
    // ── Codepoints hay collide vào default 0x80 ────────────────────────
    if name.contains("LIGHT BULB") || name.contains("ELECTRIC") {
        return 0x60;
    }
    if name.contains("HIGH VOLTAGE") {
        return 0xF0;
    }
    if name.contains("BLOWING") || name.contains("WIND") {
        return 0x58;
    }
    if name.contains("SPEAKER") || name.contains("SOUND") {
        return 0xA0;
    }
    if name.contains("BANDAGE") || name.contains("INJURED") {
        return 0x48;
    }
    if name.contains("FORK") || name.contains("KNIFE") {
        return 0x50;
    }
    if name.contains("POLICE") || name.contains("REVOLVING") {
        return 0xC8;
    }
    if name.contains("GARDEN") {
        return 0x44;
    }
    if name.contains("HOUSE") || name.contains("HOME") || name.contains("BUILDING") {
        return 0x28;
    }
    if name.contains("TREE") || name.contains("DECIDUOUS") {
        return 0x30;
    }
    if name.contains("WAVE") {
        return 0xA8;
    }
    if name.contains("GLOBE") || name.contains("EARTH") {
        return 0x48;
    }
    if name.contains("SILHOUETTE") || name.contains("BUST") {
        return 0x40;
    }
    if name.contains("EYE") {
        return 0x90;
    }
    if name.contains("PERMANENT") || name.contains("INFINITY") {
        return 0x18;
    }
    if name.contains("OCTAGONAL") {
        return 0xB0;
    }
    if name.contains("ALARM") && name.contains("CLOCK") {
        return 0xD8;
    }
    if name.contains("OPEN LOCK") {
        return 0x48;
    }
    if name.contains("LOCK") {
        return 0x28;
    }
    if name.contains("SUN WITH FACE") {
        return 0x90;
    }
    if name.contains("SPARKL") {
        return 0xB8;
    }
    if name.contains("HEART") {
        return 0xC0;
    }
    if name.contains("CHECK") {
        return 0x58;
    }
    // Math → static (arousal near 0)
    if (0x2200..=0x22FF).contains(&cp) {
        return 0x20;
    }
    if (0x2100..=0x214F).contains(&cp) {
        return 0x10;
    }
    // Arrows → moderate (directed action)
    if (0x2190..=0x21FF).contains(&cp) {
        return 0xC0;
    }
    0x80 // default moderate
}

/// Time byte từ codepoint và tên Unicode.
fn time_of(cp: u32, name: &str) -> u8 {
    // Musical note duration → Time
    if (0x1D100..=0x1D1FF).contains(&cp) {
        if name.contains("WHOLE") {
            return 0x01;
        } // Static (Largo)
        if name.contains("HALF") {
            return 0x02;
        } // Slow (Adagio)
        if name.contains("QUARTER") {
            return 0x03;
        } // Medium (Andante)
        if name.contains("EIGHTH") {
            return 0x04;
        } // Fast (Allegro)
        if name.contains("SIXTEENTH") {
            return 0x05;
        } // Instant (Presto)
        return 0x03; // Medium default
    }
    // Math symbols = static (mathematical truths không thay đổi)
    if (0x2200..=0x22FF).contains(&cp) {
        return 0x01;
    } // Static
    if (0x2100..=0x214F).contains(&cp) {
        return 0x01;
    } // Static
    if (0x2150..=0x218F).contains(&cp) {
        return 0x01;
    } // Static (numbers)
      // Geometric shapes = static
    if (0x25A0..=0x25FF).contains(&cp) {
        return 0x01;
    } // Static
      // Block elements = static
    if (0x2580..=0x259F).contains(&cp) {
        return 0x01;
    } // Static
      // Arrows = instant (directed, immediate)
    if (0x2190..=0x21FF).contains(&cp) {
        return 0x05;
    } // Instant
      // EMOTICON by name
    if name.contains("FIRE") || name.contains("LIGHTNING") {
        return 0x04;
    } // Fast
    if name.contains("SNOWFLAKE") || name.contains("MOON") {
        return 0x02;
    } // Slow
    if name.contains("DROPLET") {
        return 0x02;
    } // Slow
    if name.contains("RUNNER") {
        return 0x04;
    } // Fast
    if name.contains("STOP") {
        return 0x05;
    } // Instant
    if name.contains("BRAIN") {
        return 0x03;
    } // Medium
    if name.contains("CLOCK") {
        return 0x05;
    } // Instant
    if name.contains("HIGH VOLTAGE") {
        return 0x05;
    } // Instant — lightning
    if name.contains("HOUSE") || name.contains("HOME") || name.contains("BUILDING") {
        return 0x01;
    } // Static — permanent structure
    if name.contains("LOCK") {
        return 0x01;
    } // Static — security
    if name.contains("TREE") || name.contains("DECIDUOUS") {
        return 0x02;
    } // Slow — growth
    if name.contains("GLOBE") || name.contains("EARTH") {
        return 0x01;
    } // Static — planet
    if name.contains("PERMANENT") || name.contains("INFINITY") {
        return 0x01;
    } // Static — eternal
    if name.contains("EYE") {
        return 0x04;
    } // Fast — quick perception
    if name.contains("WAVE") {
        return 0x04;
    } // Fast — dynamic
    if name.contains("POLICE") || name.contains("REVOLVING") {
        return 0x04;
    } // Fast — urgent
    if name.contains("SPEAKER") || name.contains("SOUND") {
        return 0x04;
    } // Fast — transient
      // General EMOTICON = medium (thực thể có vận động bình thường)
    if (0x2600..=0x26FF).contains(&cp) {
        return 0x03;
    }
    if (0x1F300..=0x1FAFF).contains(&cp) {
        return 0x03;
    }
    // Yijing = slow cycle
    if (0x4DC0..=0x4DFF).contains(&cp) {
        return 0x02;
    }
    0x03 // default Medium
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

// ─────────────────────────────────────────────────────────────────────────────
// main
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace = manifest.parent().unwrap().parent().unwrap();
    let ucd_dir = workspace.join("ucd_source");
    let unicode_data = ucd_dir.join("UnicodeData.txt");

    println!("cargo:rerun-if-changed={}", unicode_data.display());

    if !unicode_data.exists() {
        eprintln!("cargo:warning=UnicodeData.txt not found — generating empty tables");
        write_empty();
        return;
    }

    // ── Parse UnicodeData.txt ─────────────────────────────────────────────
    let content = fs::read_to_string(&unicode_data).expect("read UnicodeData.txt");
    let mut cp_map: HashMap<u32, (String, String)> = HashMap::new(); // cp → (name, cat)

    for line in content.lines() {
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() < 3 {
            continue;
        }
        let cp = match u32::from_str_radix(parts[0], 16) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let name = parts[1];
        let cat = parts[2];
        if name.starts_with('<') {
            continue;
        } // skip range markers
        cp_map.insert(cp, (name.to_string(), cat.to_string()));
    }

    // ── Build entries từ 5 nhóm ───────────────────────────────────────────
    struct Entry {
        cp: u32,
        group: u8,
        shape: u8,
        relation: u8,
        valence: u8,
        arousal: u8,
        time: u8,
        hash: u64,
        name: String,
    }

    let mut entries: Vec<Entry> = Vec::new();
    let mut seen_cp: HashMap<u32, bool> = HashMap::new();

    for group in GROUPS {
        for &(start, end) in group.ranges {
            for cp in start..=end {
                if seen_cp.contains_key(&cp) {
                    continue;
                }
                let (name, _cat) = match cp_map.get(&cp) {
                    Some(v) => v,
                    None => continue,
                };
                seen_cp.insert(cp, true);

                let shape = shape_of(cp, name);
                let relation = relation_of(cp, name);
                let valence = valence_of(cp, name);
                let arousal = arousal_of(cp, name);
                let time = time_of(cp, name);
                let hash = chain_hash(shape, relation, valence, arousal, time);

                entries.push(Entry {
                    cp,
                    group: group.byte,
                    shape,
                    relation,
                    valence,
                    arousal,
                    time,
                    hash,
                    name: name.clone(),
                });
            }
        }
    }

    // Sort by cp cho binary_search
    entries.sort_by_key(|e| e.cp);

    eprintln!("cargo:warning=UCD entries: {}", entries.len());

    // ── Build HASH_TO_CP (reverse index) ─────────────────────────────────
    // Sort by hash cho binary_search
    let mut hash_to_cp: Vec<(u64, u32)> = entries.iter().map(|e| (e.hash, e.cp)).collect();
    hash_to_cp.sort_by_key(|&(h, _)| h);
    // Deduplicate: nếu 2 cp có cùng hash, chỉ giữ 1
    hash_to_cp.dedup_by_key(|&mut (h, _)| h);

    // ── Build CP_BUCKET (shape,relation → [cp]) ──────────────────────────
    let mut buckets: HashMap<(u8, u8), Vec<u32>> = HashMap::new();
    for e in &entries {
        buckets.entry((e.shape, e.relation)).or_default().push(e.cp);
    }
    // Flatten thành sorted array cho static embedding
    let mut bucket_list: Vec<((u8, u8), Vec<u32>)> = buckets.into_iter().collect();
    bucket_list.sort_by_key(|&((s, r), _)| (s, r));

    // ── Generate Rust source ──────────────────────────────────────────────
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = out_dir.join("ucd_generated.rs");

    let mut src = String::new();

    writeln!(src, "// AUTO-GENERATED by ucd/build.rs").unwrap();
    writeln!(src, "// Source: UnicodeData.txt (Unicode 18.0)").unwrap();
    writeln!(src, "// {} entries · DO NOT EDIT", entries.len()).unwrap();
    writeln!(src).unwrap();

    // UcdEntry struct
    writeln!(src, "#[derive(Clone, Copy, Debug)]").unwrap();
    writeln!(src, "#[allow(missing_docs)]").unwrap();
    writeln!(src, "pub struct UcdEntry {{").unwrap();
    writeln!(src, "    pub cp:       u32,").unwrap();
    writeln!(
        src,
        "    pub group:    u8,   // 0x01=SDF 0x02=MATH 0x03=EMOTICON 0x04=MUSICAL"
    )
    .unwrap();
    writeln!(src, "    pub shape:    u8,   // ShapeBase byte").unwrap();
    writeln!(src, "    pub relation: u8,   // RelationBase byte").unwrap();
    writeln!(src, "    pub valence:  u8,   // 0x00=V− 0x7F=V0 0xFF=V+").unwrap();
    writeln!(src, "    pub arousal:  u8,   // 0x00=calm 0xFF=excited").unwrap();
    writeln!(src, "    pub time:     u8,   // 0x01=Static..0x05=Instant").unwrap();
    writeln!(
        src,
        "    pub hash:     u64,  // FNV-1a của [shape,rel,val,aro,time]"
    )
    .unwrap();
    writeln!(src, "    pub name:     &'static str,").unwrap();
    writeln!(src, "}}").unwrap();
    writeln!(src).unwrap();

    // UCD_TABLE — forward lookup sorted by cp
    writeln!(
        src,
        "/// Forward lookup: cp → UcdEntry (binary search by cp)"
    )
    .unwrap();
    writeln!(src, "pub static UCD_TABLE: &[UcdEntry] = &[").unwrap();
    for e in &entries {
        let safe_name = e.name.replace('\\', "\\\\").replace('"', "\\\"");
        writeln!(src,
            "    UcdEntry {{ cp: 0x{:05X}, group: 0x{:02X}, shape: 0x{:02X}, relation: 0x{:02X}, valence: 0x{:02X}, arousal: 0x{:02X}, time: 0x{:02X}, hash: 0x{:016X}u64, name: \"{}\" }},",
            e.cp, e.group, e.shape, e.relation, e.valence, e.arousal, e.time, e.hash, safe_name
        ).unwrap();
    }
    writeln!(src, "];").unwrap();
    writeln!(src).unwrap();

    // HASH_TO_CP — reverse index sorted by hash
    writeln!(
        src,
        "/// Reverse index: chain_hash → cp (binary search by hash) O(log n)"
    )
    .unwrap();
    writeln!(src, "#[cfg(feature = \"reverse-index\")]").unwrap();
    writeln!(src, "pub static HASH_TO_CP: &[(u64, u32)] = &[").unwrap();
    for (hash, cp) in &hash_to_cp {
        writeln!(src, "    (0x{:016X}u64, 0x{:05X}),", hash, cp).unwrap();
    }
    writeln!(src, "];").unwrap();
    writeln!(src).unwrap();

    // CP_BUCKET — bucket index
    writeln!(
        src,
        "/// Bucket index: (shape, relation) → [cp] for top-n decode"
    )
    .unwrap();
    writeln!(
        src,
        "/// Format: flat array of (shape, relation, cp_count, cp[0], cp[1], ...)"
    )
    .unwrap();
    writeln!(src, "#[cfg(feature = \"reverse-index\")]").unwrap();
    writeln!(src, "pub static CP_BUCKET_DATA: &[u32] = &[").unwrap();
    let mut bucket_offsets: Vec<((u8, u8), u32, u32)> = Vec::new(); // (key, offset, count)
    let mut offset: u32 = 0;
    // First pass: write all cp lists
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

    writeln!(
        src,
        "/// Bucket lookup: (shape, relation) → (offset, count) into CP_BUCKET_DATA"
    )
    .unwrap();
    writeln!(src, "#[cfg(feature = \"reverse-index\")]").unwrap();
    writeln!(
        src,
        "pub static CP_BUCKET_INDEX: &[(u8, u8, u32, u32)] = &["
    )
    .unwrap();
    for ((s, r), off, cnt) in &bucket_offsets {
        writeln!(src, "    (0x{:02X}, 0x{:02X}, {}, {}),", s, r, off, cnt).unwrap();
    }
    writeln!(src, "];").unwrap();
    writeln!(src).unwrap();

    // SDF_PRIMITIVES
    writeln!(src, "/// 8 SDF primitives: (codepoint, shape_byte)").unwrap();
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

    fs::write(&out_file, &src).expect("write ucd_generated.rs");
    eprintln!(
        "cargo:warning=Generated: {} entries, {} hash entries, {} buckets",
        entries.len(),
        hash_to_cp.len(),
        bucket_list.len()
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
    pub valence:u8, pub arousal:u8, pub time:u8, pub hash:u64,
    pub name:&'static str,
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
"#,
    )
    .unwrap();
}
