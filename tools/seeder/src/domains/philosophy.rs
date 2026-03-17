//! # philosophy — Core philosophical concepts
//!
//! Ontology, epistemology, ethics, logic, aesthetics.

use super::{SeedEdge, SeedNode};

pub static PHILOSOPHY_NODES: &[SeedNode] = &[
    // ─── Ontology (what exists) ─────────────────────────────────────────────
    SeedNode { name: "existence", codepoint: 0x2203, aliases: &[
        "ton-tai", "tồn tại", "existence", "being", "sein",
    ]},
    SeedNode { name: "reality", codepoint: 0x1F30D, aliases: &[
        "thuc-tai", "thực tại", "reality", "réalité",
    ]},
    SeedNode { name: "consciousness", codepoint: 0x1F9E0, aliases: &[
        "y-thuc", "ý thức", "consciousness", "conscience",
        "tu-duy", "tư duy", "cogito",
    ]},
    SeedNode { name: "free_will", codepoint: 0x1F3C3, aliases: &[
        "tu-do-y-chi", "tự do ý chí", "free will", "libre arbitre",
    ]},
    SeedNode { name: "determinism", codepoint: 0x1F517, aliases: &[
        "tat-dinh", "tất định", "determinism", "déterminisme",
    ]},
    SeedNode { name: "identity", codepoint: 0x2261, aliases: &[
        "ban-the", "bản thể", "identity", "identité", "toi-la-ai",
    ]},
    SeedNode { name: "substance", codepoint: 0x25CF, aliases: &[
        "ban-chat", "bản chất", "substance", "essence",
    ]},

    // ─── Epistemology (what we know) ────────────────────────────────────────
    SeedNode { name: "truth", codepoint: 0x2705, aliases: &[
        "chan-ly", "chân lý", "truth", "vérité", "su-that", "sự thật",
    ]},
    SeedNode { name: "knowledge", codepoint: 0x1F4DA, aliases: &[
        "tri-thuc", "tri thức", "knowledge", "connaissance",
    ]},
    SeedNode { name: "belief", codepoint: 0x1F64F, aliases: &[
        "niem-tin", "niềm tin", "belief", "croyance",
    ]},
    SeedNode { name: "reason", codepoint: 0x1F9E0, aliases: &[
        "ly-tri", "lý trí", "reason", "raison", "rationality",
    ]},
    SeedNode { name: "doubt", codepoint: 0x2753, aliases: &[
        "nghi-ngo", "nghi ngờ", "doubt", "doute", "skepticism",
    ]},
    SeedNode { name: "perception_phil", codepoint: 0x1F441, aliases: &[
        "nhan-thuc", "nhận thức", "perception", "cam-nhan",
    ]},

    // ─── Ethics (what is right) ─────────────────────────────────────────────
    SeedNode { name: "ethics", codepoint: 0x2696, aliases: &[
        "dao-duc", "đạo đức", "ethics", "éthique", "morality",
    ]},
    SeedNode { name: "justice", codepoint: 0x2696, aliases: &[
        "cong-bang", "công bằng", "justice",
    ]},
    SeedNode { name: "virtue", codepoint: 0x2B50, aliases: &[
        "duc-hanh", "đức hạnh", "virtue", "vertu",
    ]},
    SeedNode { name: "happiness", codepoint: 0x1F60C, aliases: &[
        "hanh-phuc", "hạnh phúc", "happiness", "eudaimonia", "bonheur",
    ]},
    SeedNode { name: "suffering", codepoint: 0x1F915, aliases: &[
        "kho-dau", "khổ đau", "suffering", "souffrance", "dukkha",
    ]},
    SeedNode { name: "responsibility", codepoint: 0x1F91D, aliases: &[
        "trach-nhiem", "trách nhiệm", "responsibility", "responsabilité",
    ]},

    // ─── Logic & Reasoning ──────────────────────────────────────────────────
    SeedNode { name: "logic", codepoint: 0x2227, aliases: &[
        "logic", "luan-ly", "luận lý", "logique",
    ]},
    SeedNode { name: "paradox", codepoint: 0x221E, aliases: &[
        "nghich-ly", "nghịch lý", "paradox", "paradoxe",
    ]},
    SeedNode { name: "dialectic", codepoint: 0x21D4, aliases: &[
        "bien-chung", "biện chứng", "dialectic", "dialectique",
        "thesis-antithesis-synthesis",
    ]},
    SeedNode { name: "induction", codepoint: 0x21D2, aliases: &[
        "quy-nap", "quy nạp", "induction",
    ]},
    SeedNode { name: "deduction", codepoint: 0x21D2, aliases: &[
        "dien-dich", "diễn dịch", "deduction", "déduction",
    ]},

    // ─── Aesthetics ─────────────────────────────────────────────────────────
    SeedNode { name: "beauty", codepoint: 0x2728, aliases: &[
        "cai-dep", "cái đẹp", "beauty", "beauté",
    ]},
    SeedNode { name: "art", codepoint: 0x1F3A8, aliases: &[
        "nghe-thuat", "nghệ thuật", "art",
    ]},
];

pub static PHILOSOPHY_EDGES: &[SeedEdge] = &[
    // Ontology
    SeedEdge { from: "consciousness", to: "existence", relation: 0x06 },   // consciousness → existence
    SeedEdge { from: "free_will", to: "determinism", relation: 0x04 },     // free will ⊥ determinism
    SeedEdge { from: "identity", to: "consciousness", relation: 0x08 },    // identity ← consciousness
    SeedEdge { from: "reality", to: "existence", relation: 0x09 },         // reality ∋ existence
    SeedEdge { from: "substance", to: "existence", relation: 0x08 },       // substance ← existence
    // Epistemology
    SeedEdge { from: "knowledge", to: "truth", relation: 0x06 },         // knowledge → truth
    SeedEdge { from: "knowledge", to: "belief", relation: 0x09 },        // knowledge ∋ belief
    SeedEdge { from: "belief", to: "truth", relation: 0x06 },            // belief → truth
    SeedEdge { from: "reason", to: "knowledge", relation: 0x06 },        // reason → knowledge
    SeedEdge { from: "doubt", to: "belief", relation: 0x04 },            // doubt ⊥ belief
    SeedEdge { from: "perception_phil", to: "knowledge", relation: 0x06 }, // perception → knowledge
    // Ethics
    SeedEdge { from: "ethics", to: "justice", relation: 0x09 },        // ethics ∋ justice
    SeedEdge { from: "ethics", to: "virtue", relation: 0x09 },         // ethics ∋ virtue
    SeedEdge { from: "virtue", to: "happiness", relation: 0x06 },      // virtue → happiness
    SeedEdge { from: "happiness", to: "suffering", relation: 0x04 },   // happiness ⊥ suffering
    SeedEdge { from: "responsibility", to: "ethics", relation: 0x01 }, // responsibility ∈ ethics
    SeedEdge { from: "free_will", to: "responsibility", relation: 0x06 }, // free will → responsibility
    // Logic
    SeedEdge { from: "logic", to: "reason", relation: 0x02 },        // logic ⊂ reason
    SeedEdge { from: "paradox", to: "logic", relation: 0x04 },       // paradox ⊥ logic
    SeedEdge { from: "dialectic", to: "logic", relation: 0x02 },     // dialectic ⊂ logic
    SeedEdge { from: "induction", to: "deduction", relation: 0x04 }, // induction ⊥ deduction
    // Aesthetics
    SeedEdge { from: "beauty", to: "art", relation: 0x06 },          // beauty → art
    SeedEdge { from: "art", to: "consciousness", relation: 0x06 },   // art → consciousness
];

pub fn all_nodes() -> Vec<&'static SeedNode> {
    PHILOSOPHY_NODES.iter().collect()
}
