//! # biology — Cell biology, genetics, ecology
//!
//! DNA, proteins, cells, photosynthesis, evolution.

use super::{SeedEdge, SeedNode};

pub static BIOLOGY_NODES: &[SeedNode] = &[
    // ─── Cell Biology ───────────────────────────────────────────────────────
    SeedNode { name: "cell", codepoint: 0x1F9EC, aliases: &[
        "te-bao", "tế bào", "cell", "cellule",
    ]},
    SeedNode { name: "nucleus", codepoint: 0x1F7E1, aliases: &[
        "nhan-te-bao", "nhân tế bào", "nucleus", "noyau",
    ]},
    SeedNode { name: "mitochondria", codepoint: 0x1F7E2, aliases: &[
        "ti-the", "ty thể", "mitochondria", "mitochondrie",
        "powerhouse of the cell",
    ]},
    SeedNode { name: "ribosome", codepoint: 0x1F7E3, aliases: &[
        "ribosome", "riboxom",
    ]},
    SeedNode { name: "membrane", codepoint: 0x1F7E0, aliases: &[
        "mang-te-bao", "màng tế bào", "cell membrane", "membrane cellulaire",
    ]},

    // ─── Genetics ───────────────────────────────────────────────────────────
    SeedNode { name: "dna", codepoint: 0x1F9EC, aliases: &[
        "DNA", "ADN", "axit deoxyribonucleic", "deoxyribonucleic acid",
    ]},
    SeedNode { name: "rna", codepoint: 0x1F9EC, aliases: &[
        "RNA", "ARN", "axit ribonucleic", "ribonucleic acid",
    ]},
    SeedNode { name: "gene", codepoint: 0x1F9EC, aliases: &[
        "gen", "gene", "gène",
    ]},
    SeedNode { name: "chromosome", codepoint: 0x1F9EC, aliases: &[
        "nhiem-sac-the", "nhiễm sắc thể", "chromosome",
    ]},
    SeedNode { name: "protein", codepoint: 0x1F356, aliases: &[
        "protein", "đạm", "protéine",
    ]},
    SeedNode { name: "enzyme", codepoint: 0x1F9EA, aliases: &[
        "enzyme", "enzym", "men",
    ]},
    SeedNode { name: "amino_acid", codepoint: 0x1F9EC, aliases: &[
        "axit-amin", "axit amin", "amino acid", "acide aminé",
    ]},
    SeedNode { name: "mutation", codepoint: 0x1F9EC, aliases: &[
        "dot-bien", "đột biến", "mutation",
    ]},

    // ─── Processes ──────────────────────────────────────────────────────────
    SeedNode { name: "photosynthesis", codepoint: 0x1F33F, aliases: &[
        "quang-hop", "quang hợp", "photosynthesis", "photosynthèse",
    ]},
    SeedNode { name: "respiration", codepoint: 0x1F4A8, aliases: &[
        "ho-hap", "hô hấp", "cellular respiration", "respiration cellulaire",
    ]},
    SeedNode { name: "mitosis", codepoint: 0x1F9EC, aliases: &[
        "nguyen-phan", "nguyên phân", "mitosis",
    ]},
    SeedNode { name: "meiosis", codepoint: 0x1F9EC, aliases: &[
        "giam-phan", "giảm phân", "meiosis", "méiose",
    ]},
    SeedNode { name: "evolution", codepoint: 0x1F9EC, aliases: &[
        "tien-hoa", "tiến hóa", "evolution", "évolution", "Darwin",
    ]},
    SeedNode { name: "natural_selection", codepoint: 0x1F9EC, aliases: &[
        "chon-loc-tu-nhien", "chọn lọc tự nhiên", "natural selection",
        "sélection naturelle",
    ]},

    // ─── Ecology ────────────────────────────────────────────────────────────
    SeedNode { name: "ecosystem", codepoint: 0x1F333, aliases: &[
        "he-sinh-thai", "hệ sinh thái", "ecosystem", "écosystème",
    ]},
    SeedNode { name: "biodiversity", codepoint: 0x1F33F, aliases: &[
        "da-dang-sinh-hoc", "đa dạng sinh học", "biodiversity", "biodiversité",
    ]},
    SeedNode { name: "food_chain", codepoint: 0x1F517, aliases: &[
        "chuoi-thuc-an", "chuỗi thức ăn", "food chain", "chaîne alimentaire",
    ]},
    SeedNode { name: "organism", codepoint: 0x1F9A0, aliases: &[
        "sinh-vat", "sinh vật", "organism", "organisme",
    ]},
    SeedNode { name: "bacteria", codepoint: 0x1F9A0, aliases: &[
        "vi-khuan", "vi khuẩn", "bacteria", "bactérie",
    ]},
    SeedNode { name: "virus_bio", codepoint: 0x1F9A0, aliases: &[
        "virus", "vi-rut",
    ]},
    SeedNode { name: "neuron", codepoint: 0x1F9E0, aliases: &[
        "te-bao-than-kinh", "tế bào thần kinh", "neuron", "neurone",
    ]},
    SeedNode { name: "synapse", codepoint: 0x1F9E0, aliases: &[
        "khop-than-kinh", "khớp thần kinh", "synapse",
    ]},
];

pub static BIOLOGY_EDGES: &[SeedEdge] = &[
    // Cell structure
    SeedEdge { from: "cell", to: "nucleus", relation: 0x09 },       // cell ∋ nucleus
    SeedEdge { from: "cell", to: "mitochondria", relation: 0x09 },  // cell ∋ mitochondria
    SeedEdge { from: "cell", to: "ribosome", relation: 0x09 },      // cell ∋ ribosome
    SeedEdge { from: "cell", to: "membrane", relation: 0x09 },      // cell ∋ membrane
    // Genetics
    SeedEdge { from: "nucleus", to: "dna", relation: 0x09 },           // nucleus ∋ DNA
    SeedEdge { from: "dna", to: "gene", relation: 0x09 },              // DNA ∋ gene
    SeedEdge { from: "chromosome", to: "dna", relation: 0x09 },        // chromosome ∋ DNA
    SeedEdge { from: "gene", to: "protein", relation: 0x06 },          // gene → protein
    SeedEdge { from: "dna", to: "rna", relation: 0x06 },               // DNA → RNA (transcription)
    SeedEdge { from: "rna", to: "protein", relation: 0x06 },           // RNA → protein (translation)
    SeedEdge { from: "protein", to: "amino_acid", relation: 0x08 },    // protein ← amino acid
    SeedEdge { from: "enzyme", to: "protein", relation: 0x02 },        // enzyme ⊂ protein
    SeedEdge { from: "mutation", to: "gene", relation: 0x06 },         // mutation → gene
    // Processes
    SeedEdge { from: "photosynthesis", to: "respiration", relation: 0x04 }, // ⊥ (inverse)
    SeedEdge { from: "mitosis", to: "cell", relation: 0x06 },    // mitosis → cell
    SeedEdge { from: "meiosis", to: "mitosis", relation: 0x07 }, // meiosis ≈ mitosis
    SeedEdge { from: "meiosis", to: "gene", relation: 0x06 },    // meiosis → gene diversity
    // Evolution
    SeedEdge { from: "evolution", to: "natural_selection", relation: 0x06 },  // evo → selection
    SeedEdge { from: "natural_selection", to: "mutation", relation: 0x08 },   // selection ← mutation
    SeedEdge { from: "evolution", to: "biodiversity", relation: 0x06 },       // evo → biodiversity
    // Ecology
    SeedEdge { from: "ecosystem", to: "food_chain", relation: 0x09 },     // ecosystem ∋ food chain
    SeedEdge { from: "ecosystem", to: "biodiversity", relation: 0x09 },   // ecosystem ∋ biodiversity
    SeedEdge { from: "ecosystem", to: "organism", relation: 0x09 },       // ecosystem ∋ organism
    SeedEdge { from: "bacteria", to: "organism", relation: 0x02 },        // bacteria ⊂ organism
    // Neuroscience
    SeedEdge { from: "neuron", to: "cell", relation: 0x02 },    // neuron ⊂ cell
    SeedEdge { from: "neuron", to: "synapse", relation: 0x09 }, // neuron ∋ synapse
];

pub fn all_nodes() -> Vec<&'static SeedNode> {
    BIOLOGY_NODES.iter().collect()
}
