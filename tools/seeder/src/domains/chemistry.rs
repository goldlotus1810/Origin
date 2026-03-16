//! # chemistry — Elements, molecules, reactions
//!
//! Periodic table elements, common molecules, chemical bonds.
//! Uses atom symbol ⚛ (U+269B) and supplemental math operators for reactions.

use super::{SeedEdge, SeedNode};

pub static CHEMISTRY_NODES: &[SeedNode] = &[
    // ─── Elements ───────────────────────────────────────────────────────────
    SeedNode { name: "hydrogen", codepoint: 0x1D407, aliases: &[
        "H", "hydro", "hiđrô", "hydrogen", "hydrogène", "Z=1",
    ]},
    SeedNode { name: "helium", codepoint: 0x1D407, aliases: &[
        "He", "heli", "helium", "hélium", "Z=2",
    ]},
    SeedNode { name: "carbon", codepoint: 0x1D402, aliases: &[
        "C", "cacbon", "carbon", "carbone", "Z=6",
    ]},
    SeedNode { name: "nitrogen", codepoint: 0x1D40D, aliases: &[
        "N", "nito", "nitơ", "nitrogen", "azote", "Z=7",
    ]},
    SeedNode { name: "oxygen", codepoint: 0x1D40E, aliases: &[
        "O", "oxi", "oxy", "oxygen", "oxygène", "Z=8",
    ]},
    SeedNode { name: "sodium", codepoint: 0x1D40D, aliases: &[
        "Na", "natri", "sodium", "Z=11",
    ]},
    SeedNode { name: "chlorine", codepoint: 0x1D402, aliases: &[
        "Cl", "clo", "chlorine", "chlore", "Z=17",
    ]},
    SeedNode { name: "iron_element", codepoint: 0x1D405, aliases: &[
        "Fe", "sat", "sắt", "iron", "fer", "Z=26",
    ]},
    SeedNode { name: "copper", codepoint: 0x1D402, aliases: &[
        "Cu", "dong", "đồng", "copper", "cuivre", "Z=29",
    ]},
    SeedNode { name: "gold_element", codepoint: 0x1D400, aliases: &[
        "Au", "vang", "vàng", "gold", "or", "Z=79",
    ]},
    SeedNode { name: "silicon", codepoint: 0x1D412, aliases: &[
        "Si", "silic", "silicon", "silicium", "Z=14",
    ]},
    SeedNode { name: "phosphorus", codepoint: 0x1D40F, aliases: &[
        "P", "photpho", "phosphorus", "phosphore", "Z=15",
    ]},
    SeedNode { name: "sulfur", codepoint: 0x1D412, aliases: &[
        "S", "luu-huynh", "lưu huỳnh", "sulfur", "soufre", "Z=16",
    ]},
    SeedNode { name: "calcium", codepoint: 0x1D402, aliases: &[
        "Ca", "canxi", "calcium", "Z=20",
    ]},
    SeedNode { name: "potassium", codepoint: 0x1D40A, aliases: &[
        "K", "kali", "potassium", "Z=19",
    ]},

    // ─── Molecules ──────────────────────────────────────────────────────────
    SeedNode { name: "water_molecule", codepoint: 0x1F4A7, aliases: &[
        "H2O", "H₂O", "nuoc", "nước", "water", "eau",
    ]},
    SeedNode { name: "carbon_dioxide", codepoint: 0x1F32B, aliases: &[
        "CO2", "CO₂", "khi-cacbonic", "khí cacbonic", "carbon dioxide",
    ]},
    SeedNode { name: "salt", codepoint: 0x1F9C2, aliases: &[
        "NaCl", "muoi", "muối", "salt", "sel", "sodium chloride",
    ]},
    SeedNode { name: "glucose", codepoint: 0x1F36C, aliases: &[
        "C6H12O6", "C₆H₁₂O₆", "duong", "đường", "glucose",
    ]},
    SeedNode { name: "oxygen_gas", codepoint: 0x1F4A8, aliases: &[
        "O2", "O₂", "khi-oxi", "khí oxy", "oxygen gas",
    ]},
    SeedNode { name: "methane", codepoint: 0x1F4A8, aliases: &[
        "CH4", "CH₄", "metan", "methane", "méthane",
    ]},
    SeedNode { name: "ethanol", codepoint: 0x1F37A, aliases: &[
        "C2H5OH", "C₂H₅OH", "ruou-etylic", "rượu etylic", "ethanol",
    ]},
    SeedNode { name: "ammonia", codepoint: 0x1F4A8, aliases: &[
        "NH3", "NH₃", "amoniac", "ammonia", "ammoniac",
    ]},

    // ─── Concepts ───────────────────────────────────────────────────────────
    SeedNode { name: "chemical_bond", codepoint: 0x1F517, aliases: &[
        "lien-ket-hoa-hoc", "liên kết hóa học", "chemical bond", "liaison chimique",
    ]},
    SeedNode { name: "covalent_bond", codepoint: 0x1F517, aliases: &[
        "lien-ket-cong-hoa-tri", "liên kết cộng hóa trị", "covalent bond",
    ]},
    SeedNode { name: "ionic_bond", codepoint: 0x1F517, aliases: &[
        "lien-ket-ion", "liên kết ion", "ionic bond",
    ]},
    SeedNode { name: "chemical_reaction", codepoint: 0x2192, aliases: &[
        "phan-ung-hoa-hoc", "phản ứng hóa học", "chemical reaction",
        "réaction chimique",
    ]},
    SeedNode { name: "acid", codepoint: 0x1F9EA, aliases: &[
        "axit", "acid", "acide", "pH<7",
    ]},
    SeedNode { name: "base_chem", codepoint: 0x1F9EA, aliases: &[
        "bazo", "bazơ", "base", "pH>7", "alkaline",
    ]},
    SeedNode { name: "catalyst", codepoint: 0x26A1, aliases: &[
        "xuc-tac", "chất xúc tác", "catalyst", "catalyseur",
    ]},
    SeedNode { name: "oxidation", codepoint: 0x1F525, aliases: &[
        "oxi-hoa", "oxy hóa", "oxidation", "oxydation",
    ]},
    SeedNode { name: "reduction", codepoint: 0x1F4A7, aliases: &[
        "khu", "khử", "reduction", "réduction",
    ]},
    SeedNode { name: "ph_scale", codepoint: 0x1F9EA, aliases: &[
        "pH", "do-pH", "độ pH", "pH scale",
    ]},
    SeedNode { name: "mole", codepoint: 0x1D440, aliases: &[
        "mol", "so-mol", "mole", "6.022e23", "avogadro",
    ]},
    SeedNode { name: "periodic_table", codepoint: 0x1F9EA, aliases: &[
        "bang-tuan-hoan", "bảng tuần hoàn", "periodic table",
        "tableau périodique", "Mendeleev",
    ]},
];

pub static CHEMISTRY_EDGES: &[SeedEdge] = &[
    // H2O composition
    SeedEdge { from: "water_molecule", to: "hydrogen", relation: 0x08 },  // H2O ← H
    SeedEdge { from: "water_molecule", to: "oxygen", relation: 0x08 },    // H2O ← O
    // CO2 composition
    SeedEdge { from: "carbon_dioxide", to: "carbon", relation: 0x08 },
    SeedEdge { from: "carbon_dioxide", to: "oxygen", relation: 0x08 },
    // NaCl
    SeedEdge { from: "salt", to: "sodium", relation: 0x08 },
    SeedEdge { from: "salt", to: "chlorine", relation: 0x08 },
    // Glucose
    SeedEdge { from: "glucose", to: "carbon", relation: 0x08 },
    SeedEdge { from: "glucose", to: "hydrogen", relation: 0x08 },
    SeedEdge { from: "glucose", to: "oxygen", relation: 0x08 },
    // Bonds
    SeedEdge { from: "covalent_bond", to: "chemical_bond", relation: 0x02 }, // covalent ⊂ bond
    SeedEdge { from: "ionic_bond", to: "chemical_bond", relation: 0x02 },    // ionic ⊂ bond
    SeedEdge { from: "salt", to: "ionic_bond", relation: 0x09 },             // NaCl ∋ ionic
    SeedEdge { from: "water_molecule", to: "covalent_bond", relation: 0x09 }, // H2O ∋ covalent
    // Acid-base
    SeedEdge { from: "acid", to: "base_chem", relation: 0x04 },  // acid ⊥ base
    SeedEdge { from: "ph_scale", to: "acid", relation: 0x09 },   // pH ∋ acid
    SeedEdge { from: "ph_scale", to: "base_chem", relation: 0x09 }, // pH ∋ base
    // Redox
    SeedEdge { from: "oxidation", to: "reduction", relation: 0x04 }, // ox ⊥ red (inverse)
    SeedEdge { from: "chemical_reaction", to: "catalyst", relation: 0x06 }, // reaction → catalyst
    // Elements → periodic table
    SeedEdge { from: "hydrogen", to: "periodic_table", relation: 0x01 },
    SeedEdge { from: "carbon", to: "periodic_table", relation: 0x01 },
    SeedEdge { from: "oxygen", to: "periodic_table", relation: 0x01 },
    SeedEdge { from: "iron_element", to: "periodic_table", relation: 0x01 },
    SeedEdge { from: "gold_element", to: "periodic_table", relation: 0x01 },
    // Mole
    SeedEdge { from: "mole", to: "chemical_reaction", relation: 0x06 }, // mol → reaction
];

pub fn all_nodes() -> Vec<&'static SeedNode> {
    CHEMISTRY_NODES.iter().collect()
}
