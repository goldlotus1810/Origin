//! # physics — Physics concepts, laws, and units
//!
//! Forces, energy, particles, waves, relativity.
//! Relationships encode physical laws (F=ma, E=mc²).

use super::{SeedEdge, SeedNode};

pub static PHYSICS_NODES: &[SeedNode] = &[
    // ─── Mechanics ──────────────────────────────────────────────────────────
    SeedNode { name: "force", codepoint: 0x2192, aliases: &[
        "luc", "lực", "force", "kraft", "F",
    ]},
    SeedNode { name: "mass", codepoint: 0x1D440, aliases: &[
        "khoi-luong", "khối lượng", "mass", "masse", "m",
    ]},
    SeedNode { name: "acceleration", codepoint: 0x1D44E, aliases: &[
        "gia-toc", "gia tốc", "acceleration", "a",
    ]},
    SeedNode { name: "velocity", codepoint: 0x1D463, aliases: &[
        "van-toc", "vận tốc", "velocity", "vitesse", "v",
    ]},
    SeedNode { name: "momentum", codepoint: 0x1D45D, aliases: &[
        "dong-luong", "động lượng", "momentum", "p",
    ]},
    SeedNode { name: "gravity", codepoint: 0x1D454, aliases: &[
        "trong-luc", "trọng lực", "gravity", "gravitation", "g", "9.81",
    ]},
    SeedNode { name: "friction", codepoint: 0x1D453, aliases: &[
        "ma-sat", "ma sát", "friction", "frottement",
    ]},
    SeedNode { name: "pressure", codepoint: 0x1D443, aliases: &[
        "ap-suat", "áp suất", "pressure", "pression", "P", "pascal",
    ]},

    // ─── Energy & Work ──────────────────────────────────────────────────────
    SeedNode { name: "energy", codepoint: 0x26A1, aliases: &[
        "nang-luong", "năng lượng", "energy", "énergie", "E",
    ]},
    SeedNode { name: "kinetic_energy", codepoint: 0x1D43E, aliases: &[
        "dong-nang", "động năng", "kinetic energy", "KE",
    ]},
    SeedNode { name: "potential_energy", codepoint: 0x1D448, aliases: &[
        "the-nang", "thế năng", "potential energy", "PE",
    ]},
    SeedNode { name: "work", codepoint: 0x1D44A, aliases: &[
        "cong", "công", "work", "travail", "W",
    ]},
    SeedNode { name: "power_phys", codepoint: 0x1D44B, aliases: &[
        "cong-suat", "công suất", "power", "puissance",
    ]},

    // ─── Waves & Light ──────────────────────────────────────────────────────
    SeedNode { name: "wave", codepoint: 0x1F30A, aliases: &[
        "song", "sóng", "wave", "onde",
    ]},
    SeedNode { name: "frequency", codepoint: 0x1D453, aliases: &[
        "tan-so", "tần số", "frequency", "fréquence", "Hz", "hertz",
    ]},
    SeedNode { name: "wavelength", codepoint: 0x03BB, aliases: &[
        "buoc-song", "bước sóng", "wavelength", "λ",
    ]},
    SeedNode { name: "speed_of_light", codepoint: 0x1D450, aliases: &[
        "toc-do-anh-sang", "tốc độ ánh sáng", "speed of light",
        "c", "299792458", "3e8",
    ]},

    // ─── Electromagnetism ───────────────────────────────────────────────────
    SeedNode { name: "electric_charge", codepoint: 0x1D444, aliases: &[
        "dien-tich", "điện tích", "electric charge", "charge", "Q", "coulomb",
    ]},
    SeedNode { name: "electric_field", codepoint: 0x1D404, aliases: &[
        "truong-dien", "trường điện", "electric field",
    ]},
    SeedNode { name: "magnetic_field", codepoint: 0x1D405, aliases: &[
        "truong-tu", "trường từ", "magnetic field", "B", "tesla",
    ]},
    SeedNode { name: "current", codepoint: 0x1D43C, aliases: &[
        "dong-dien", "dòng điện", "current", "courant", "I", "ampere",
    ]},
    SeedNode { name: "voltage", codepoint: 0x1D449, aliases: &[
        "hieu-dien-the", "hiệu điện thế", "voltage", "V", "volt",
    ]},
    SeedNode { name: "resistance", codepoint: 0x1D445, aliases: &[
        "dien-tro", "điện trở", "resistance", "R", "ohm", "Ω",
    ]},

    // ─── Thermodynamics ─────────────────────────────────────────────────────
    SeedNode { name: "temperature", codepoint: 0x1F321, aliases: &[
        "nhiet-do", "nhiệt độ", "temperature", "T", "kelvin", "celsius",
    ]},
    SeedNode { name: "entropy", codepoint: 0x1D446, aliases: &[
        "entropy", "S", "do-hon-loan", "độ hỗn loạn",
    ]},
    SeedNode { name: "heat", codepoint: 0x1F525, aliases: &[
        "nhiet", "nhiệt", "heat", "chaleur",
    ]},

    // ─── Relativity & Quantum ───────────────────────────────────────────────
    SeedNode { name: "spacetime", codepoint: 0x1F30C, aliases: &[
        "khong-thoi-gian", "không-thời gian", "spacetime",
    ]},
    SeedNode { name: "photon", codepoint: 0x1F4A1, aliases: &[
        "photon", "hat-anh-sang", "hạt ánh sáng",
    ]},
    SeedNode { name: "electron", codepoint: 0x1D452, aliases: &[
        "electron", "dien-tu", "điện tử",
    ]},
    SeedNode { name: "neutron", codepoint: 0x1D45B, aliases: &[
        "neutron", "hat-trung-hoa", "hạt trung hòa",
    ]},
    SeedNode { name: "proton", codepoint: 0x1D45D, aliases: &[
        "proton", "hat-proton",
    ]},
    SeedNode { name: "atom", codepoint: 0x269B, aliases: &[
        "nguyen-tu", "nguyên tử", "atom", "atome",
    ]},
];

pub static PHYSICS_EDGES: &[SeedEdge] = &[
    // F = m × a
    SeedEdge { from: "force", to: "mass", relation: 0x08 },         // F ← m (derived)
    SeedEdge { from: "force", to: "acceleration", relation: 0x08 }, // F ← a (derived)
    // p = m × v
    SeedEdge { from: "momentum", to: "mass", relation: 0x08 },
    SeedEdge { from: "momentum", to: "velocity", relation: 0x08 },
    // E = mc²
    SeedEdge { from: "energy", to: "mass", relation: 0x08 },
    SeedEdge { from: "energy", to: "speed_of_light", relation: 0x08 },
    // Energy types
    SeedEdge { from: "kinetic_energy", to: "energy", relation: 0x02 },   // KE ⊂ E
    SeedEdge { from: "potential_energy", to: "energy", relation: 0x02 }, // PE ⊂ E
    SeedEdge { from: "heat", to: "energy", relation: 0x02 },            // Q ⊂ E
    // Work-energy
    SeedEdge { from: "work", to: "force", relation: 0x08 },     // W ← F
    SeedEdge { from: "power_phys", to: "work", relation: 0x08 }, // P ← W
    // Gravity
    SeedEdge { from: "gravity", to: "force", relation: 0x02 },  // gravity ⊂ force
    SeedEdge { from: "gravity", to: "mass", relation: 0x06 },   // gravity → mass
    SeedEdge { from: "friction", to: "force", relation: 0x02 }, // friction ⊂ force
    // Waves
    SeedEdge { from: "wave", to: "frequency", relation: 0x09 },   // wave ∋ frequency
    SeedEdge { from: "wave", to: "wavelength", relation: 0x09 },  // wave ∋ wavelength
    SeedEdge { from: "wavelength", to: "frequency", relation: 0x04 }, // λ ⊥ f (inverse)
    SeedEdge { from: "speed_of_light", to: "photon", relation: 0x06 }, // c → photon
    // EM
    SeedEdge { from: "current", to: "electric_charge", relation: 0x08 }, // I ← Q
    SeedEdge { from: "voltage", to: "current", relation: 0x06 },        // V → I (Ohm)
    SeedEdge { from: "resistance", to: "current", relation: 0x04 },     // R ⊥ I
    SeedEdge { from: "electric_field", to: "magnetic_field", relation: 0x07 }, // E ≈ B
    // Thermo
    SeedEdge { from: "temperature", to: "heat", relation: 0x06 },   // T → Q
    SeedEdge { from: "entropy", to: "temperature", relation: 0x08 }, // S ← T
    // Particles
    SeedEdge { from: "atom", to: "electron", relation: 0x09 },  // atom ∋ electron
    SeedEdge { from: "atom", to: "proton", relation: 0x09 },    // atom ∋ proton
    SeedEdge { from: "atom", to: "neutron", relation: 0x09 },   // atom ∋ neutron
    SeedEdge { from: "photon", to: "wave", relation: 0x07 },    // photon ≈ wave (duality)
    SeedEdge { from: "electron", to: "electric_charge", relation: 0x09 }, // e ∋ charge
];

pub fn all_nodes() -> Vec<&'static SeedNode> {
    PHYSICS_NODES.iter().collect()
}
