//! build.rs — đọc UnicodeData.txt lúc compile → sinh bảng tĩnh
//!
//! Output trong OUT_DIR/ucd_generated.rs:
//!   UCD_TABLE         — forward lookup (cp → Molecule bytes + formula rules)
//!   HASH_TO_CP        — reverse index (chain_hash → cp), O(log n) decode
//!   CP_BUCKET         — bucket index (shape,relation → [cp]), top-n decode
//!   SDF_PRIMITIVES    — 8 SDF primitive mappings
//!   RELATION_PRIMITIVES — 8 Relation primitive mappings
//!   FORMULA_NAMES_S/R/V/A/T — formula descriptions per dimension
//!
//! KHÔNG hardcode bất kỳ Molecule nào.
//! Mọi giá trị đến từ UnicodeData.txt.
//!
//! Mỗi entry mang 5 công thức (f_s, f_r, f_v, f_a, f_t):
//!   Shape    = f_s(cp, name, block) → u8
//!   Relation = f_r(cp, name, block) → u8
//!   Valence  = f_v(cp, name, block) → u8
//!   Arousal  = f_a(cp, name, block) → u8
//!   Time     = f_t(cp, name, block) → u8
//!
//! Formula rule_id ghi lại CÁCH suy ra mỗi chiều — molecule tự giải thích được.

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
// Formula rule definitions — mỗi chiều có danh sách rules
// Rule ID = index trong danh sách. Molecule mang rule_id → tự giải thích.
// ─────────────────────────────────────────────────────────────────────────────

// Shape formula rules: f_s(cp, name, block) → (value, rule_id)
const FS: &[(u8, &str)] = &[
    // 0: SDF primitive match
    (0, "f_s: cp ∈ SDF_PRIMS → primitive.shape"),
    // 1: Geometric Shapes sub-ranges
    (1, "f_s: cp ∈ [25CB,25CF] → Sphere (○● round)"),
    (2, "f_s: cp ∈ [25A0,25AB] → Box (■ rectangular)"),
    (3, "f_s: cp ∈ [25AC,25AF] → Capsule (▬ oblong)"),
    (4, "f_s: cp ∈ [25B2,25C5] → Cone (▲▼◀▶ directional)"),
    (5, "f_s: cp ∈ [25C6,25CA] → Intersect (◆ diamond)"),
    (6, "f_s: cp ∈ [25D0,25FF] → Union (◐◑ partial fill)"),
    (7, "f_s: cp ∈ [25A0,25FF] → Sphere (geom default)"),
    // 8: Block Elements
    (8, "f_s: cp ∈ [2580,259F] → Box (░▒▓█ fill levels)"),
    // 9: Arrows
    (9, "f_s: cp ∈ [2190,21FF] → Cone (directed arrow)"),
    // 10-13: Math Operators by name
    (10, "f_s: cp ∈ [2200,22FF] ∧ name⊃\"UNION\" → Union"),
    (11, "f_s: cp ∈ [2200,22FF] ∧ name⊃\"INTERSECTION\" → Intersect"),
    (12, "f_s: cp ∈ [2200,22FF] ∧ name⊃\"MINUS\" → Subtract"),
    (13, "f_s: cp ∈ [2200,22FF] → Torus (math loop)"),
    // 14: Letterlike
    (14, "f_s: cp ∈ [2100,214F] → Torus (abstract loop)"),
    // 15-16: EMOTICON
    (15, "f_s: cp ∈ [2600,26FF] → Sphere (entity round)"),
    (16, "f_s: cp ∈ [1F300,1FAFF] → Sphere (pictograph round)"),
    // 17-18: Musical/Yijing
    (17, "f_s: cp ∈ [1D100,1D1FF] → Torus (musical cycle)"),
    (18, "f_s: cp ∈ [4DC0,4DFF] → Torus (yijing cycle)"),
    // 19: Default
    (19, "f_s: default → Sphere"),
];

// Relation formula rules: f_r(cp, name, block) → (value, rule_id)
const FR: &[(u8, &str)] = &[
    (0, "f_r: cp ∈ REL_PRIMS → primitive.relation"),
    // 1-9: Math Operators by name
    (1, "f_r: cp ∈ [2200,22FF] ∧ name⊃\"ELEMENT\" → Member"),
    (2, "f_r: cp ∈ [2200,22FF] ∧ name⊃\"SUBSET\" → Subset"),
    (3, "f_r: cp ∈ [2200,22FF] ∧ name⊃\"IDENTICAL\" → Equiv"),
    (4, "f_r: cp ∈ [2200,22FF] ∧ name⊃\"EQUAL\" → Equiv"),
    (5, "f_r: cp ∈ [2200,22FF] ∧ name⊃\"TACK\" → Orthogonal"),
    (6, "f_r: cp ∈ [2200,22FF] ∧ name⊃\"RING\" → Compose"),
    (7, "f_r: cp ∈ [2200,22FF] ∧ name⊃\"ARROW\" → Causes"),
    (8, "f_r: cp ∈ [2200,22FF] ∧ name⊃\"ALMOST\" → Similar"),
    (9, "f_r: cp ∈ [2200,22FF] ∧ name⊃\"UNION\" → Member"),
    (10, "f_r: cp ∈ [2200,22FF] → Equiv (math default)"),
    // 11-15: Arrows by direction
    (11, "f_r: cp ∈ [2190,21FF] ∧ name⊃\"LEFT\" → DerivedFrom"),
    (12, "f_r: cp ∈ [2190,21FF] ∧ name⊃\"RIGHT\" → Causes"),
    (13, "f_r: cp ∈ [2190,21FF] ∧ name⊃\"UP\" → Causes"),
    (14, "f_r: cp ∈ [2190,21FF] ∧ name⊃\"DOWN\" → DerivedFrom"),
    (15, "f_r: cp ∈ [2190,21FF] → Causes (arrow default)"),
    // 16-19: Block/EMOTICON/Musical
    (16, "f_r: cp ∈ [2580,259F] → Member (fill level)"),
    (17, "f_r: cp ∈ [2600,26FF] → Member (symbol group)"),
    (18, "f_r: cp ∈ [1F300,1FAFF] → Member (pictograph group)"),
    (19, "f_r: cp ∈ [1D100,1D1FF] → Similar (notes relate)"),
    // 20: Default
    (20, "f_r: default → Member"),
];

// Valence formula rules: f_v(cp, name) → (value, rule_id)
const FV: &[(u8, &str)] = &[
    // 0-4: Block Elements fill level
    (0, "f_v: cp=2588(█) → 0xFF (full fill)"),
    (1, "f_v: cp=2593(▓) → 0xC0 (dark fill)"),
    (2, "f_v: cp=2592(▒) → 0x80 (medium fill)"),
    (3, "f_v: cp=2591(░) → 0x40 (light fill)"),
    (4, "f_v: cp ∈ [2580,259F] → 0x00 (empty fill)"),
    // 5-7: Emoticon faces
    (5, "f_v: cp ∈ [1F600,1F60F] → 0xFF (positive face)"),
    (6, "f_v: cp ∈ [1F620,1F62F] → 0x00 (negative face)"),
    (7, "f_v: cp ∈ [1F610,1F61F] → 0x80 (neutral face)"),
    // 8-31: Name patterns (semantic derivation)
    (8, "f_v: name⊃\"FIRE\"|\"FLAME\" → 0xFF (heat+energy=positive)"),
    (9, "f_v: name⊃\"HEART\"|\"LOVE\" → 0xFF (affection=positive)"),
    (10, "f_v: name⊃\"STAR\"|\"SPARKL\" → 0xE0 (brilliance)"),
    (11, "f_v: name⊃\"SUN WITH FACE\" → 0xE8 (warm personality)"),
    (12, "f_v: name⊃\"SUN\"|\"BRIGHT\" → 0xE0 (light=positive)"),
    (13, "f_v: name⊃\"WARNING\"|\"DANGE\" → 0x20 (threat=negative)"),
    (14, "f_v: name⊃\"SKULL\"|\"DEATH\" → 0x00 (mortality=negative)"),
    (15, "f_v: name⊃\"DROPLET\"|\"WATER\" → 0xC0 (life=positive)"),
    (16, "f_v: name⊃\"SNOWFLAKE\"|\"COLD\" → 0x30 (isolation=negative)"),
    (17, "f_v: name⊃\"BRAIN\" → 0xC0 (intelligence=positive)"),
    (18, "f_v: name⊃\"CHECK\" → 0xFF (success=positive)"),
    (19, "f_v: name⊃\"CROSS\" → 0x10 (rejection=negative)"),
    (20, "f_v: name⊃\"LIGHT BULB\"|\"ELECTRIC\" → 0xB0 (idea=positive)"),
    (21, "f_v: name⊃\"HIGH VOLTAGE\"|\"LIGHTNING\" → 0xA0 (power)"),
    (22, "f_v: name⊃\"BLOWING\"|\"WIND\" → 0x90 (movement)"),
    (23, "f_v: name⊃\"SPEAKER\"|\"SOUND\" → 0x98 (communication)"),
    (24, "f_v: name⊃\"BANDAGE\"|\"INJURED\" → 0x18 (pain=negative)"),
    (25, "f_v: name⊃\"FORK\"|\"KNIFE\" → 0x88 (neutral tool)"),
    (26, "f_v: name⊃\"POLICE\"|\"REVOLVING\" → 0x38 (authority=negative)"),
    (27, "f_v: name⊃\"GARDEN\" → 0xBC (nature=positive)"),
    (28, "f_v: name⊃\"HOUSE\"|\"HOME\"|\"BUILDING\" → 0xA8 (shelter=positive)"),
    (29, "f_v: name⊃\"TREE\"|\"DECIDUOUS\" → 0xA4 (growth=positive)"),
    (30, "f_v: name⊃\"WAVE\" → 0xB8 (energy=positive)"),
    (31, "f_v: name⊃\"GLOBE\"|\"EARTH\" → 0x94 (nature)"),
    (32, "f_v: name⊃\"SILHOUETTE\"|\"BUST\" → 0x78 (person=neutral)"),
    (33, "f_v: name⊃\"EYE\" → 0x84 (perception=neutral)"),
    (34, "f_v: name⊃\"PERMANENT\"|\"INFINITY\" → 0x7C (eternal=neutral)"),
    (35, "f_v: name⊃\"OCTAGONAL\" → 0x28 (stop=negative)"),
    (36, "f_v: name⊃\"ALARM\"∧\"CLOCK\" → 0x48 (urgency=negative)"),
    (37, "f_v: name⊃\"OPEN LOCK\" → 0x8C (freedom=neutral+)"),
    (38, "f_v: name⊃\"LOCK\" → 0x58 (confinement=negative)"),
    // 39-41: Block ranges
    (39, "f_v: cp ∈ [2200,22FF] → 0x80 (math=neutral)"),
    (40, "f_v: cp ∈ [2100,214F] → 0x80 (letterlike=neutral)"),
    (41, "f_v: cp ∈ [1D100,1D1FF] → 0xA0 (music=positive)"),
    // 42: Default
    (42, "f_v: default → 0x80 (neutral)"),
];

// Arousal formula rules: f_a(cp, name) → (value, rule_id)
const FA: &[(u8, &str)] = &[
    // 0-5: Musical dynamics
    (0, "f_a: name⊃\"FORTISSIMO\"|\"FF\" → 0xFF (fortissimo)"),
    (1, "f_a: name⊃\"FORTE\" → 0xC0 (forte)"),
    (2, "f_a: name⊃\"MEZZO\" → 0x80 (mezzo)"),
    (3, "f_a: name⊃\"PIANO\"×2 → 0x10 (pianissimo)"),
    (4, "f_a: name⊃\"PIANO\" → 0x40 (piano)"),
    (5, "f_a: cp ∈ [1D100,1D1FF] → 0x80 (musical default)"),
    // 6-8: Emoticon faces
    (6, "f_a: cp ∈ [1F600,1F60F] → 0xFF (excited positive)"),
    (7, "f_a: cp ∈ [1F620,1F62F] → 0xE0 (excited negative)"),
    (8, "f_a: cp ∈ [1F610,1F61F] → 0x20 (calm neutral)"),
    // 9-32: Name patterns
    (9, "f_a: name⊃\"FIRE\"|\"LIGHTNING\" → 0xFF (intense energy)"),
    (10, "f_a: name⊃\"WARNING\"|\"ALARM\" → 0xE0 (alert state)"),
    (11, "f_a: name⊃\"SNOWFLAKE\"|\"SLEEP\" → 0x10 (dormant)"),
    (12, "f_a: name⊃\"DROPLET\" → 0x40 (gentle flow)"),
    (13, "f_a: name⊃\"BRAIN\" → 0x60 (focused thought)"),
    (14, "f_a: name⊃\"RUNNER\" → 0xD0 (high activity)"),
    (15, "f_a: name⊃\"STOP\" → 0x20 (cessation)"),
    (16, "f_a: name⊃\"LIGHT BULB\"|\"ELECTRIC\" → 0x60 (steady glow)"),
    (17, "f_a: name⊃\"HIGH VOLTAGE\" → 0xF0 (electric surge)"),
    (18, "f_a: name⊃\"BLOWING\"|\"WIND\" → 0x58 (breeze)"),
    (19, "f_a: name⊃\"SPEAKER\"|\"SOUND\" → 0xA0 (audible)"),
    (20, "f_a: name⊃\"BANDAGE\"|\"INJURED\" → 0x48 (pain response)"),
    (21, "f_a: name⊃\"FORK\"|\"KNIFE\" → 0x50 (tool use)"),
    (22, "f_a: name⊃\"POLICE\"|\"REVOLVING\" → 0xC8 (siren urgency)"),
    (23, "f_a: name⊃\"GARDEN\" → 0x44 (peaceful)"),
    (24, "f_a: name⊃\"HOUSE\"|\"HOME\"|\"BUILDING\" → 0x28 (stable shelter)"),
    (25, "f_a: name⊃\"TREE\"|\"DECIDUOUS\" → 0x30 (slow growth)"),
    (26, "f_a: name⊃\"WAVE\" → 0xA8 (dynamic motion)"),
    (27, "f_a: name⊃\"GLOBE\"|\"EARTH\" → 0x48 (massive calm)"),
    (28, "f_a: name⊃\"SILHOUETTE\"|\"BUST\" → 0x40 (still)"),
    (29, "f_a: name⊃\"EYE\" → 0x90 (alert perception)"),
    (30, "f_a: name⊃\"PERMANENT\"|\"INFINITY\" → 0x18 (timeless calm)"),
    (31, "f_a: name⊃\"OCTAGONAL\" → 0xB0 (urgent halt)"),
    (32, "f_a: name⊃\"ALARM\"∧\"CLOCK\" → 0xD8 (alarm ringing)"),
    (33, "f_a: name⊃\"OPEN LOCK\" → 0x48 (release)"),
    (34, "f_a: name⊃\"LOCK\" → 0x28 (secured still)"),
    (35, "f_a: name⊃\"SUN WITH FACE\" → 0x90 (radiant warmth)"),
    (36, "f_a: name⊃\"SPARKL\" → 0xB8 (glitter)"),
    (37, "f_a: name⊃\"HEART\" → 0xC0 (heartbeat)"),
    (38, "f_a: name⊃\"CHECK\" → 0x58 (confirmation)"),
    // 39-41: Block ranges
    (39, "f_a: cp ∈ [2200,22FF] → 0x20 (math=calm)"),
    (40, "f_a: cp ∈ [2100,214F] → 0x10 (letterlike=still)"),
    (41, "f_a: cp ∈ [2190,21FF] → 0xC0 (arrow=directed action)"),
    // 42: Default
    (42, "f_a: default → 0x80 (moderate)"),
];

// Time formula rules: f_t(cp, name) → (value, rule_id)
const FT: &[(u8, &str)] = &[
    // 0-5: Musical note duration
    (0, "f_t: name⊃\"WHOLE\" → Static (largo, sustained)"),
    (1, "f_t: name⊃\"HALF\" → Slow (adagio)"),
    (2, "f_t: name⊃\"QUARTER\" → Medium (andante)"),
    (3, "f_t: name⊃\"EIGHTH\" → Fast (allegro)"),
    (4, "f_t: name⊃\"SIXTEENTH\" → Instant (presto)"),
    (5, "f_t: cp ∈ [1D100,1D1FF] → Medium (musical default)"),
    // 6-10: Block ranges
    (6, "f_t: cp ∈ [2200,22FF] → Static (math truth=eternal)"),
    (7, "f_t: cp ∈ [2100,214F] → Static (letterlike=constant)"),
    (8, "f_t: cp ∈ [2150,218F] → Static (numbers=invariant)"),
    (9, "f_t: cp ∈ [25A0,25FF] → Static (geometry=fixed)"),
    (10, "f_t: cp ∈ [2580,259F] → Static (block=fixed)"),
    (11, "f_t: cp ∈ [2190,21FF] → Instant (arrow=immediate)"),
    // 12-26: Name patterns
    (12, "f_t: name⊃\"FIRE\"|\"LIGHTNING\" → Fast (combustion)"),
    (13, "f_t: name⊃\"SNOWFLAKE\"|\"MOON\" → Slow (frozen/orbit)"),
    (14, "f_t: name⊃\"DROPLET\" → Slow (drip)"),
    (15, "f_t: name⊃\"RUNNER\" → Fast (sprint)"),
    (16, "f_t: name⊃\"STOP\" → Instant (halt)"),
    (17, "f_t: name⊃\"BRAIN\" → Medium (thought pace)"),
    (18, "f_t: name⊃\"CLOCK\" → Instant (time marker)"),
    (19, "f_t: name⊃\"HIGH VOLTAGE\" → Instant (discharge)"),
    (20, "f_t: name⊃\"HOUSE\"|\"HOME\"|\"BUILDING\" → Static (permanent)"),
    (21, "f_t: name⊃\"LOCK\" → Static (secured)"),
    (22, "f_t: name⊃\"TREE\"|\"DECIDUOUS\" → Slow (growth)"),
    (23, "f_t: name⊃\"GLOBE\"|\"EARTH\" → Static (planet)"),
    (24, "f_t: name⊃\"PERMANENT\"|\"INFINITY\" → Static (eternal)"),
    (25, "f_t: name⊃\"EYE\" → Fast (perception)"),
    (26, "f_t: name⊃\"WAVE\" → Fast (dynamic)"),
    (27, "f_t: name⊃\"POLICE\"|\"REVOLVING\" → Fast (urgent)"),
    (28, "f_t: name⊃\"SPEAKER\"|\"SOUND\" → Fast (transient)"),
    // 29-30: General EMOTICON
    (29, "f_t: cp ∈ [2600,26FF] → Medium (entity motion)"),
    (30, "f_t: cp ∈ [1F300,1FAFF] → Medium (pictograph motion)"),
    // 31: Yijing
    (31, "f_t: cp ∈ [4DC0,4DFF] → Slow (yijing cycle)"),
    // 32: Default
    (32, "f_t: default → Medium"),
];

// ─────────────────────────────────────────────────────────────────────────────
// Derive Molecule bytes + formula từ codepoint + UCD data
// Mỗi hàm trả về (value, rule_id) — giá trị VÀ công thức suy ra nó
// ─────────────────────────────────────────────────────────────────────────────

/// f_s(cp, name) → (shape_value, rule_id)
fn shape_of(cp: u32, name: &str) -> (u8, u8) {
    // Rule 0: SDF primitive match
    for &(pcp, pbyte, _) in SDF_PRIMS {
        if cp == pcp {
            return (pbyte, 0);
        }
    }
    // Rules 1-7: Geometric Shapes 25A0..25FF sub-ranges
    if (0x25A0..=0x25FF).contains(&cp) {
        return match cp {
            0x25CB..=0x25CF => (0x01, 1),  // ○● → Sphere/Torus
            0x25A0..=0x25AB => (0x03, 2),  // ■ → Box
            0x25AC..=0x25AF => (0x02, 3),  // ▬ → Capsule
            0x25B2..=0x25C5 => (0x04, 4),  // ▲▼◀▶ → Cone
            0x25C6..=0x25CA => (0x07, 5),  // ◆ → Intersect
            0x25D0..=0x25FF => (0x06, 6),  // ◐◑ → Union
            _ => (0x01, 7),                // Sphere default
        };
    }
    // Rule 8: Block Elements → Box
    if (0x2580..=0x259F).contains(&cp) {
        return (0x03, 8);
    }
    // Rule 9: Arrows → Cone
    if (0x2190..=0x21FF).contains(&cp) {
        return (0x04, 9);
    }
    // Rules 10-13: Math Operators by name
    if (0x2200..=0x22FF).contains(&cp) {
        if name.contains("UNION") {
            return (0x06, 10);
        }
        if name.contains("INTERSECTION") {
            return (0x07, 11);
        }
        if name.contains("MINUS") {
            return (0x08, 12);
        }
        return (0x05, 13); // Torus default
    }
    // Rule 14: Letterlike → Torus
    if (0x2100..=0x214F).contains(&cp) {
        return (0x05, 14);
    }
    // Rules 15-16: EMOTICON → Sphere
    if (0x2600..=0x26FF).contains(&cp) {
        return (0x01, 15);
    }
    if (0x1F300..=0x1FAFF).contains(&cp) {
        return (0x01, 16);
    }
    // Rules 17-18: Musical/Yijing → Torus
    if (0x1D100..=0x1D1FF).contains(&cp) {
        return (0x05, 17);
    }
    if (0x4DC0..=0x4DFF).contains(&cp) {
        return (0x05, 18);
    }
    // Rule 19: Default → Sphere
    (0x01, 19)
}

/// f_r(cp, name) → (relation_value, rule_id)
fn relation_of(cp: u32, name: &str) -> (u8, u8) {
    // Rule 0: RELATION primitive match
    for &(pcp, pbyte, _) in REL_PRIMS {
        if cp == pcp {
            return (pbyte, 0);
        }
    }
    // Rules 1-10: Math Operators by name
    if (0x2200..=0x22FF).contains(&cp) {
        if name.contains("ELEMENT") {
            return (0x01, 1);
        }
        if name.contains("SUBSET") {
            return (0x02, 2);
        }
        if name.contains("IDENTICAL") {
            return (0x03, 3);
        }
        if name.contains("EQUAL") {
            return (0x03, 4);
        }
        if name.contains("TACK") {
            return (0x04, 5);
        }
        if name.contains("RING") {
            return (0x05, 6);
        }
        if name.contains("ARROW") {
            return (0x06, 7);
        }
        if name.contains("ALMOST") {
            return (0x07, 8);
        }
        if name.contains("UNION") {
            return (0x01, 9);
        }
        return (0x03, 10); // Equiv default
    }
    // Rules 11-15: Arrows by direction
    if (0x2190..=0x21FF).contains(&cp) {
        if name.contains("LEFT") {
            return (0x08, 11);
        }
        if name.contains("RIGHT") {
            return (0x06, 12);
        }
        if name.contains("UP") {
            return (0x06, 13);
        }
        if name.contains("DOWN") {
            return (0x08, 14);
        }
        return (0x06, 15); // Causes default
    }
    // Rules 16-19: Block/EMOTICON/Musical
    if (0x2580..=0x259F).contains(&cp) {
        return (0x01, 16);
    }
    if (0x2600..=0x26FF).contains(&cp) {
        return (0x01, 17);
    }
    if (0x1F300..=0x1FAFF).contains(&cp) {
        return (0x01, 18);
    }
    if (0x1D100..=0x1D1FF).contains(&cp) {
        return (0x07, 19);
    }
    // Rule 20: Default → Member
    (0x01, 20)
}

/// f_v(cp, name) → (valence_value, rule_id)
fn valence_of(cp: u32, name: &str) -> (u8, u8) {
    // Rules 0-4: Block Elements fill level
    if (0x2580..=0x259F).contains(&cp) {
        return match cp {
            0x2588 => (0xFF, 0), // █ full
            0x2593 => (0xC0, 1), // ▓
            0x2592 => (0x80, 2), // ▒
            0x2591 => (0x40, 3), // ░
            _ => (0x00, 4),
        };
    }
    // Rules 5-7: Emoticon faces
    if (0x1F600..=0x1F64F).contains(&cp) {
        if (0x1F600..=0x1F60F).contains(&cp) {
            return (0xFF, 5);
        }
        if (0x1F620..=0x1F62F).contains(&cp) {
            return (0x00, 6);
        }
        if (0x1F610..=0x1F61F).contains(&cp) {
            return (0x80, 7);
        }
    }
    // Rules 8-38: Name patterns
    if name.contains("FIRE") || name.contains("FLAME") {
        return (0xFF, 8);
    }
    if name.contains("HEART") || name.contains("LOVE") {
        return (0xFF, 9);
    }
    if name.contains("STAR") || name.contains("SPARKL") {
        return (0xE0, 10);
    }
    if name.contains("SUN WITH FACE") {
        return (0xE8, 11);
    }
    if name.contains("SUN") || name.contains("BRIGHT") {
        return (0xE0, 12);
    }
    if name.contains("WARNING") || name.contains("DANGE") {
        return (0x20, 13);
    }
    if name.contains("SKULL") || name.contains("DEATH") {
        return (0x00, 14);
    }
    if name.contains("DROPLET") || name.contains("WATER") {
        return (0xC0, 15);
    }
    if name.contains("SNOWFLAKE") || name.contains("COLD") {
        return (0x30, 16);
    }
    if name.contains("BRAIN") {
        return (0xC0, 17);
    }
    if name.contains("CHECK") {
        return (0xFF, 18);
    }
    if name.contains("CROSS") {
        return (0x10, 19);
    }
    if name.contains("LIGHT BULB") || name.contains("ELECTRIC") {
        return (0xB0, 20);
    }
    if name.contains("HIGH VOLTAGE") || name.contains("LIGHTNING") {
        return (0xA0, 21);
    }
    if name.contains("BLOWING") || name.contains("WIND") {
        return (0x90, 22);
    }
    if name.contains("SPEAKER") || name.contains("SOUND") {
        return (0x98, 23);
    }
    if name.contains("BANDAGE") || name.contains("INJURED") {
        return (0x18, 24);
    }
    if name.contains("FORK") || name.contains("KNIFE") {
        return (0x88, 25);
    }
    if name.contains("POLICE") || name.contains("REVOLVING") {
        return (0x38, 26);
    }
    if name.contains("GARDEN") {
        return (0xBC, 27);
    }
    if name.contains("HOUSE") || name.contains("HOME") || name.contains("BUILDING") {
        return (0xA8, 28);
    }
    if name.contains("TREE") || name.contains("DECIDUOUS") {
        return (0xA4, 29);
    }
    if name.contains("WAVE") {
        return (0xB8, 30);
    }
    if name.contains("GLOBE") || name.contains("EARTH") {
        return (0x94, 31);
    }
    if name.contains("SILHOUETTE") || name.contains("BUST") {
        return (0x78, 32);
    }
    if name.contains("EYE") {
        return (0x84, 33);
    }
    if name.contains("PERMANENT") || name.contains("INFINITY") {
        return (0x7C, 34);
    }
    if name.contains("OCTAGONAL") {
        return (0x28, 35);
    }
    if name.contains("ALARM") && name.contains("CLOCK") {
        return (0x48, 36);
    }
    if name.contains("OPEN LOCK") {
        return (0x8C, 37);
    }
    if name.contains("LOCK") {
        return (0x58, 38);
    }
    // Rules 39-41: Block ranges
    if (0x2200..=0x22FF).contains(&cp) {
        return (0x80, 39);
    }
    if (0x2100..=0x214F).contains(&cp) {
        return (0x80, 40);
    }
    if (0x1D100..=0x1D1FF).contains(&cp) {
        return (0xA0, 41);
    }
    // Rule 42: Default neutral
    (0x80, 42)
}

/// f_a(cp, name) → (arousal_value, rule_id)
fn arousal_of(cp: u32, name: &str) -> (u8, u8) {
    // Rules 0-5: Musical dynamics
    if (0x1D100..=0x1D1FF).contains(&cp) {
        if name.contains("FORTISSIMO") || name.contains("FF") {
            return (0xFF, 0);
        }
        if name.contains("FORTE") {
            return (0xC0, 1);
        }
        if name.contains("MEZZO") {
            return (0x80, 2);
        }
        if name.contains("PIANO") && name.contains("PIANO") {
            return (0x10, 3);
        }
        if name.contains("PIANO") {
            return (0x40, 4);
        }
        return (0x80, 5);
    }
    // Rules 6-8: Emoticon faces
    if (0x1F600..=0x1F64F).contains(&cp) {
        if (0x1F600..=0x1F60F).contains(&cp) {
            return (0xFF, 6);
        }
        if (0x1F620..=0x1F62F).contains(&cp) {
            return (0xE0, 7);
        }
        if (0x1F610..=0x1F61F).contains(&cp) {
            return (0x20, 8);
        }
    }
    // Rules 9-38: Name patterns
    if name.contains("FIRE") || name.contains("LIGHTNING") {
        return (0xFF, 9);
    }
    if name.contains("WARNING") || name.contains("ALARM") {
        return (0xE0, 10);
    }
    if name.contains("SNOWFLAKE") || name.contains("SLEEP") {
        return (0x10, 11);
    }
    if name.contains("DROPLET") {
        return (0x40, 12);
    }
    if name.contains("BRAIN") {
        return (0x60, 13);
    }
    if name.contains("RUNNER") {
        return (0xD0, 14);
    }
    if name.contains("STOP") {
        return (0x20, 15);
    }
    if name.contains("LIGHT BULB") || name.contains("ELECTRIC") {
        return (0x60, 16);
    }
    if name.contains("HIGH VOLTAGE") {
        return (0xF0, 17);
    }
    if name.contains("BLOWING") || name.contains("WIND") {
        return (0x58, 18);
    }
    if name.contains("SPEAKER") || name.contains("SOUND") {
        return (0xA0, 19);
    }
    if name.contains("BANDAGE") || name.contains("INJURED") {
        return (0x48, 20);
    }
    if name.contains("FORK") || name.contains("KNIFE") {
        return (0x50, 21);
    }
    if name.contains("POLICE") || name.contains("REVOLVING") {
        return (0xC8, 22);
    }
    if name.contains("GARDEN") {
        return (0x44, 23);
    }
    if name.contains("HOUSE") || name.contains("HOME") || name.contains("BUILDING") {
        return (0x28, 24);
    }
    if name.contains("TREE") || name.contains("DECIDUOUS") {
        return (0x30, 25);
    }
    if name.contains("WAVE") {
        return (0xA8, 26);
    }
    if name.contains("GLOBE") || name.contains("EARTH") {
        return (0x48, 27);
    }
    if name.contains("SILHOUETTE") || name.contains("BUST") {
        return (0x40, 28);
    }
    if name.contains("EYE") {
        return (0x90, 29);
    }
    if name.contains("PERMANENT") || name.contains("INFINITY") {
        return (0x18, 30);
    }
    if name.contains("OCTAGONAL") {
        return (0xB0, 31);
    }
    if name.contains("ALARM") && name.contains("CLOCK") {
        return (0xD8, 32);
    }
    if name.contains("OPEN LOCK") {
        return (0x48, 33);
    }
    if name.contains("LOCK") {
        return (0x28, 34);
    }
    if name.contains("SUN WITH FACE") {
        return (0x90, 35);
    }
    if name.contains("SPARKL") {
        return (0xB8, 36);
    }
    if name.contains("HEART") {
        return (0xC0, 37);
    }
    if name.contains("CHECK") {
        return (0x58, 38);
    }
    // Rules 39-41: Block ranges
    if (0x2200..=0x22FF).contains(&cp) {
        return (0x20, 39);
    }
    if (0x2100..=0x214F).contains(&cp) {
        return (0x10, 40);
    }
    if (0x2190..=0x21FF).contains(&cp) {
        return (0xC0, 41);
    }
    // Rule 42: Default moderate
    (0x80, 42)
}

/// f_t(cp, name) → (time_value, rule_id)
fn time_of(cp: u32, name: &str) -> (u8, u8) {
    // Rules 0-5: Musical note duration
    if (0x1D100..=0x1D1FF).contains(&cp) {
        if name.contains("WHOLE") {
            return (0x01, 0);
        }
        if name.contains("HALF") {
            return (0x02, 1);
        }
        if name.contains("QUARTER") {
            return (0x03, 2);
        }
        if name.contains("EIGHTH") {
            return (0x04, 3);
        }
        if name.contains("SIXTEENTH") {
            return (0x05, 4);
        }
        return (0x03, 5);
    }
    // Rules 6-11: Block ranges
    if (0x2200..=0x22FF).contains(&cp) {
        return (0x01, 6);
    }
    if (0x2100..=0x214F).contains(&cp) {
        return (0x01, 7);
    }
    if (0x2150..=0x218F).contains(&cp) {
        return (0x01, 8);
    }
    if (0x25A0..=0x25FF).contains(&cp) {
        return (0x01, 9);
    }
    if (0x2580..=0x259F).contains(&cp) {
        return (0x01, 10);
    }
    if (0x2190..=0x21FF).contains(&cp) {
        return (0x05, 11);
    }
    // Rules 12-28: Name patterns
    if name.contains("FIRE") || name.contains("LIGHTNING") {
        return (0x04, 12);
    }
    if name.contains("SNOWFLAKE") || name.contains("MOON") {
        return (0x02, 13);
    }
    if name.contains("DROPLET") {
        return (0x02, 14);
    }
    if name.contains("RUNNER") {
        return (0x04, 15);
    }
    if name.contains("STOP") {
        return (0x05, 16);
    }
    if name.contains("BRAIN") {
        return (0x03, 17);
    }
    if name.contains("CLOCK") {
        return (0x05, 18);
    }
    if name.contains("HIGH VOLTAGE") {
        return (0x05, 19);
    }
    if name.contains("HOUSE") || name.contains("HOME") || name.contains("BUILDING") {
        return (0x01, 20);
    }
    if name.contains("LOCK") {
        return (0x01, 21);
    }
    if name.contains("TREE") || name.contains("DECIDUOUS") {
        return (0x02, 22);
    }
    if name.contains("GLOBE") || name.contains("EARTH") {
        return (0x01, 23);
    }
    if name.contains("PERMANENT") || name.contains("INFINITY") {
        return (0x01, 24);
    }
    if name.contains("EYE") {
        return (0x04, 25);
    }
    if name.contains("WAVE") {
        return (0x04, 26);
    }
    if name.contains("POLICE") || name.contains("REVOLVING") {
        return (0x04, 27);
    }
    if name.contains("SPEAKER") || name.contains("SOUND") {
        return (0x04, 28);
    }
    // Rules 29-30: General EMOTICON
    if (0x2600..=0x26FF).contains(&cp) {
        return (0x03, 29);
    }
    if (0x1F300..=0x1FAFF).contains(&cp) {
        return (0x03, 30);
    }
    // Rule 31: Yijing
    if (0x4DC0..=0x4DFF).contains(&cp) {
        return (0x02, 31);
    }
    // Rule 32: Default Medium
    (0x03, 32)
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
// Generate formula name tables
// ─────────────────────────────────────────────────────────────────────────────

fn write_formula_table(src: &mut String, table_name: &str, rules: &[(u8, &str)]) {
    writeln!(src, "/// Formula descriptions for {} dimension", table_name).unwrap();
    writeln!(src, "pub static {}: &[&str] = &[", table_name).unwrap();
    for (_id, desc) in rules {
        let safe = desc.replace('\\', "\\\\").replace('"', "\\\"");
        writeln!(src, "    \"{}\",", safe).unwrap();
    }
    writeln!(src, "];").unwrap();
    writeln!(src).unwrap();
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
        // 5 formula rule IDs — mỗi chiều mang công thức suy ra nó
        fs: u8, // shape formula rule
        fr: u8, // relation formula rule
        fv: u8, // valence formula rule
        fa: u8, // arousal formula rule
        ft: u8, // time formula rule
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

                let (shape, fs) = shape_of(cp, name);
                let (relation, fr) = relation_of(cp, name);
                let (valence, fv) = valence_of(cp, name);
                let (arousal, fa) = arousal_of(cp, name);
                let (time, ft) = time_of(cp, name);
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
                    fs,
                    fr,
                    fv,
                    fa,
                    ft,
                });
            }
        }
    }

    // ── Assign hierarchical sub-indices ────────────────────────────────────
    // Encoding: value = base + (sub_index * N_bases)
    // Shape/relation: N_bases = 8, Time: N_bases = 5
    // Sub-index tăng tuần tự trong mỗi base group → phân biệt ~5400 mẫu.
    //
    // Phase 1: hierarchical encoding trên shape/relation/time
    // Phase 2: collision resolution — nếu 2+ entries cùng 5-tuple,
    //          perturb valence/arousal (±1..±127) dùng codepoint làm seed.
    //          Giữ nguyên semantic base, chỉ thay đổi tối thiểu.
    {
        // Count per base → assign sub_index (NO wrap-around, clamp at max)
        let mut shape_counters: HashMap<u8, u16> = HashMap::new(); // base → next_sub
        let mut relation_counters: HashMap<u8, u16> = HashMap::new();
        let mut time_counters: HashMap<u8, u16> = HashMap::new();

        for entry in &mut entries {
            // Shape: base + sub*8, max sub = (255-base)/8
            let shape_base = entry.shape; // already 0x01-0x08
            let shape_sub = shape_counters.entry(shape_base).or_insert(0);
            let max_shape_sub = (255u16 - shape_base as u16) / 8;
            if *shape_sub > 0 && *shape_sub <= max_shape_sub {
                entry.shape = shape_base + (*shape_sub as u8) * 8;
            } else if *shape_sub > max_shape_sub {
                // Clamp at max — collision will be resolved in phase 2
                entry.shape = shape_base + ((*shape_sub % (max_shape_sub + 1)) as u8) * 8;
            }
            *shape_sub += 1;

            // Relation: base + sub*8
            let rel_base = entry.relation;
            let rel_sub = relation_counters.entry(rel_base).or_insert(0);
            let max_rel_sub = (255u16 - rel_base as u16) / 8;
            if *rel_sub > 0 && *rel_sub <= max_rel_sub {
                entry.relation = rel_base + (*rel_sub as u8) * 8;
            } else if *rel_sub > max_rel_sub {
                entry.relation = rel_base + ((*rel_sub % (max_rel_sub + 1)) as u8) * 8;
            }
            *rel_sub += 1;

            // Time: base + sub*5
            let time_base = entry.time;
            let time_sub = time_counters.entry(time_base).or_insert(0);
            let max_time_sub = (255u16 - time_base as u16) / 5;
            if *time_sub > 0 && *time_sub <= max_time_sub {
                entry.time = time_base + (*time_sub as u8) * 5;
            } else if *time_sub > max_time_sub {
                entry.time = time_base + ((*time_sub % (max_time_sub + 1)) as u8) * 5;
            }
            *time_sub += 1;
        }

        // Phase 2: Collision resolution via valence/arousal perturbation
        // Group entries by their 5-tuple, resolve collisions by perturbing V/A.
        use std::collections::HashSet;
        let mut seen: HashSet<(u8, u8, u8, u8, u8)> = HashSet::with_capacity(entries.len());
        for entry in &mut entries {
            let tuple = (entry.shape, entry.relation, entry.valence, entry.arousal, entry.time);
            if seen.contains(&tuple) {
                // Collision — perturb valence and/or arousal using codepoint as seed.
                // Try small perturbations first (±1, ±2, ...) to stay close to semantic value.
                let mut resolved = false;
                for delta in 1u8..=127 {
                    // Try valence+delta
                    let v_up = entry.valence.wrapping_add(delta);
                    let t1 = (entry.shape, entry.relation, v_up, entry.arousal, entry.time);
                    if !seen.contains(&t1) {
                        entry.valence = v_up;
                        seen.insert(t1);
                        resolved = true;
                        break;
                    }
                    // Try valence-delta
                    let v_down = entry.valence.wrapping_sub(delta);
                    let t2 = (entry.shape, entry.relation, v_down, entry.arousal, entry.time);
                    if !seen.contains(&t2) {
                        entry.valence = v_down;
                        seen.insert(t2);
                        resolved = true;
                        break;
                    }
                    // Try arousal+delta
                    let a_up = entry.arousal.wrapping_add(delta);
                    let t3 = (entry.shape, entry.relation, entry.valence, a_up, entry.time);
                    if !seen.contains(&t3) {
                        entry.arousal = a_up;
                        seen.insert(t3);
                        resolved = true;
                        break;
                    }
                    // Try arousal-delta
                    let a_down = entry.arousal.wrapping_sub(delta);
                    let t4 = (entry.shape, entry.relation, entry.valence, a_down, entry.time);
                    if !seen.contains(&t4) {
                        entry.arousal = a_down;
                        seen.insert(t4);
                        resolved = true;
                        break;
                    }
                }
                if !resolved {
                    // Last resort: perturb both V and A
                    for dv in 1u8..=127 {
                        for da in 1u8..=127 {
                            let v = entry.valence.wrapping_add(dv);
                            let a = entry.arousal.wrapping_add(da);
                            let t = (entry.shape, entry.relation, v, a, entry.time);
                            if !seen.contains(&t) {
                                entry.valence = v;
                                entry.arousal = a;
                                seen.insert(t);
                                break;
                            }
                        }
                    }
                }
            } else {
                seen.insert(tuple);
            }
        }

        // Recompute all hashes after collision resolution
        for entry in &mut entries {
            entry.hash = chain_hash(entry.shape, entry.relation, entry.valence, entry.arousal, entry.time);
        }

        // Verify uniqueness at build time
        let mut verify: HashSet<(u8, u8, u8, u8, u8)> = HashSet::with_capacity(entries.len());
        let mut collisions = 0u32;
        for entry in &entries {
            if !verify.insert((entry.shape, entry.relation, entry.valence, entry.arousal, entry.time)) {
                collisions += 1;
            }
        }
        if collisions > 0 {
            eprintln!("cargo:warning=UCD COLLISION WARNING: {collisions} entries still collide after resolution!");
        } else {
            eprintln!("cargo:warning=UCD: all {} entries have unique molecules ✓", entries.len());
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
    writeln!(src, "// Mỗi entry mang 5 công thức: f_s, f_r, f_v, f_a, f_t").unwrap();
    writeln!(src).unwrap();

    // UcdEntry struct — now with formula rule IDs
    writeln!(src, "#[derive(Clone, Copy, Debug)]").unwrap();
    writeln!(src, "#[allow(missing_docs)]").unwrap();
    writeln!(src, "pub struct UcdEntry {{").unwrap();
    writeln!(src, "    pub cp:       u32,").unwrap();
    writeln!(
        src,
        "    pub group:    u8,   // 0x01=SDF 0x02=MATH 0x03=EMOTICON 0x04=MUSICAL"
    )
    .unwrap();
    writeln!(src, "    pub shape:    u8,   // ShapeBase byte (hierarchical)").unwrap();
    writeln!(src, "    pub relation: u8,   // RelationBase byte (hierarchical)").unwrap();
    writeln!(src, "    pub valence:  u8,   // 0x00=V- 0x7F=V0 0xFF=V+").unwrap();
    writeln!(src, "    pub arousal:  u8,   // 0x00=calm 0xFF=excited").unwrap();
    writeln!(src, "    pub time:     u8,   // 0x01=Static..0x05=Instant (hierarchical)").unwrap();
    writeln!(
        src,
        "    pub hash:     u64,  // FNV-1a of [shape,rel,val,aro,time]"
    )
    .unwrap();
    writeln!(src, "    pub name:     &'static str,").unwrap();
    writeln!(src, "    // ── 5 formulas: WHY each dimension has its value ──").unwrap();
    writeln!(src, "    pub fs: u8,  // f_s rule → index into FORMULA_NAMES_S").unwrap();
    writeln!(src, "    pub fr: u8,  // f_r rule → index into FORMULA_NAMES_R").unwrap();
    writeln!(src, "    pub fv: u8,  // f_v rule → index into FORMULA_NAMES_V").unwrap();
    writeln!(src, "    pub fa: u8,  // f_a rule → index into FORMULA_NAMES_A").unwrap();
    writeln!(src, "    pub ft: u8,  // f_t rule → index into FORMULA_NAMES_T").unwrap();
    writeln!(src, "}}").unwrap();
    writeln!(src).unwrap();

    // UCD_TABLE — forward lookup sorted by cp (now with formulas)
    writeln!(
        src,
        "/// Forward lookup: cp → UcdEntry (binary search by cp)"
    )
    .unwrap();
    writeln!(src, "pub static UCD_TABLE: &[UcdEntry] = &[").unwrap();
    for e in &entries {
        let safe_name = e.name.replace('\\', "\\\\").replace('"', "\\\"");
        writeln!(src,
            "    UcdEntry {{ cp: 0x{:05X}, group: 0x{:02X}, shape: 0x{:02X}, relation: 0x{:02X}, valence: 0x{:02X}, arousal: 0x{:02X}, time: 0x{:02X}, hash: 0x{:016X}u64, name: \"{}\", fs: {}, fr: {}, fv: {}, fa: {}, ft: {} }},",
            e.cp, e.group, e.shape, e.relation, e.valence, e.arousal, e.time, e.hash, safe_name,
            e.fs, e.fr, e.fv, e.fa, e.ft
        ).unwrap();
    }
    writeln!(src, "];").unwrap();
    writeln!(src).unwrap();

    // Formula name tables — human-readable derivation for each dimension
    write_formula_table(&mut src, "FORMULA_NAMES_S", FS);
    write_formula_table(&mut src, "FORMULA_NAMES_R", FR);
    write_formula_table(&mut src, "FORMULA_NAMES_V", FV);
    write_formula_table(&mut src, "FORMULA_NAMES_A", FA);
    write_formula_table(&mut src, "FORMULA_NAMES_T", FT);

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
        "cargo:warning=Generated: {} entries, {} hash entries, {} buckets, 5 formula tables",
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
    pub fs:u8, pub fr:u8, pub fv:u8, pub fa:u8, pub ft:u8,
}
pub static UCD_TABLE: &[UcdEntry] = &[];
pub static FORMULA_NAMES_S: &[&str] = &[];
pub static FORMULA_NAMES_R: &[&str] = &[];
pub static FORMULA_NAMES_V: &[&str] = &[];
pub static FORMULA_NAMES_A: &[&str] = &[];
pub static FORMULA_NAMES_T: &[&str] = &[];
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
