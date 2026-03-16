//! # math — Mathematical symbols, LaTeX mapping, and concepts
//!
//! Unicode MATH block (U+2200–22FF, U+2A00–2AFF) + Greek letters.
//! Mỗi symbol có LaTeX alias: ∫ → "\int", "tích phân", "integral"
//!
//! Relation bytes (EdgeKind):
//!   0x01 = Member, 0x02 = Subset, 0x03 = Equiv, 0x05 = Compose,
//!   0x06 = Causes, 0x07 = Similar, 0x08 = DerivedFrom, 0x09 = Contains

use super::{SeedEdge, SeedNode};

// ─── Calculus ───────────────────────────────────────────────────────────────

pub static CALCULUS_NODES: &[SeedNode] = &[
    SeedNode { name: "integral", codepoint: 0x222B, aliases: &[
        "\\int", "tich-phan", "tích phân", "integral", "intégrale",
        "∫", "nguyen-ham", "nguyên hàm", "antiderivative",
    ]},
    SeedNode { name: "double_integral", codepoint: 0x222C, aliases: &[
        "\\iint", "tich-phan-kep", "tích phân kép", "double integral", "∬",
    ]},
    SeedNode { name: "triple_integral", codepoint: 0x222D, aliases: &[
        "\\iiint", "tich-phan-ba", "tích phân ba", "triple integral", "∭",
    ]},
    SeedNode { name: "contour_integral", codepoint: 0x222E, aliases: &[
        "\\oint", "tich-phan-duong", "tích phân đường", "contour integral",
        "line integral", "∮",
    ]},
    SeedNode { name: "partial", codepoint: 0x2202, aliases: &[
        "\\partial", "dao-ham-rieng", "đạo hàm riêng", "partial derivative", "∂",
    ]},
    SeedNode { name: "nabla", codepoint: 0x2207, aliases: &[
        "\\nabla", "gradient", "del", "∇", "grad",
        "divergence-op", "curl-op",
    ]},
    SeedNode { name: "summation", codepoint: 0x2211, aliases: &[
        "\\sum", "tong", "tổng", "summation", "sigma", "∑",
    ]},
    SeedNode { name: "product_nary", codepoint: 0x220F, aliases: &[
        "\\prod", "tich", "tích", "product", "n-ary product", "∏",
    ]},
    SeedNode { name: "coproduct", codepoint: 0x2210, aliases: &[
        "\\coprod", "doi-tich", "coproduct", "∐",
    ]},
    SeedNode { name: "limit", codepoint: 0x2A00, aliases: &[
        "\\lim", "gioi-han", "giới hạn", "limit", "limite",
    ]},
];

// ─── Operators & Relations ──────────────────────────────────────────────────

pub static OPERATOR_NODES: &[SeedNode] = &[
    SeedNode { name: "plus_minus", codepoint: 0x00B1, aliases: &[
        "\\pm", "cong-tru", "plus-minus", "±",
    ]},
    SeedNode { name: "minus_plus", codepoint: 0x2213, aliases: &[
        "\\mp", "tru-cong", "minus-plus", "∓",
    ]},
    SeedNode { name: "times", codepoint: 0x00D7, aliases: &[
        "\\times", "nhan", "nhân", "multiply", "×",
    ]},
    SeedNode { name: "divide", codepoint: 0x00F7, aliases: &[
        "\\div", "chia", "divide", "÷",
    ]},
    SeedNode { name: "sqrt", codepoint: 0x221A, aliases: &[
        "\\sqrt", "can-bac-hai", "căn bậc hai", "square root", "√",
    ]},
    SeedNode { name: "cbrt", codepoint: 0x221B, aliases: &[
        "\\sqrt[3]", "can-bac-ba", "căn bậc ba", "cube root", "∛",
    ]},
    SeedNode { name: "infinity", codepoint: 0x221E, aliases: &[
        "\\infty", "vo-cuc", "vô cực", "infinity", "∞",
    ]},
    SeedNode { name: "proportional", codepoint: 0x221D, aliases: &[
        "\\propto", "ti-le", "tỉ lệ", "proportional", "∝",
    ]},
    SeedNode { name: "dot_operator", codepoint: 0x22C5, aliases: &[
        "\\cdot", "nhan-cham", "dot product", "⋅",
    ]},
    SeedNode { name: "cross_product", codepoint: 0x2A2F, aliases: &[
        "\\times", "tich-co-huong", "tích có hướng", "cross product", "⨯",
    ]},
];

// ─── Set Theory ─────────────────────────────────────────────────────────────

pub static SET_NODES: &[SeedNode] = &[
    SeedNode { name: "element_of", codepoint: 0x2208, aliases: &[
        "\\in", "thuoc", "thuộc", "element of", "∈",
    ]},
    SeedNode { name: "not_element", codepoint: 0x2209, aliases: &[
        "\\notin", "khong-thuoc", "không thuộc", "not element of", "∉",
    ]},
    SeedNode { name: "subset", codepoint: 0x2282, aliases: &[
        "\\subset", "tap-con", "tập con", "subset", "⊂",
    ]},
    SeedNode { name: "superset", codepoint: 0x2283, aliases: &[
        "\\supset", "tap-cha", "tập cha", "superset", "⊃",
    ]},
    SeedNode { name: "union", codepoint: 0x222A, aliases: &[
        "\\cup", "hop", "hợp", "union", "∪",
    ]},
    SeedNode { name: "intersection", codepoint: 0x2229, aliases: &[
        "\\cap", "giao", "giao nhau", "intersection", "∩",
    ]},
    SeedNode { name: "empty_set", codepoint: 0x2205, aliases: &[
        "\\emptyset", "\\varnothing", "tap-rong", "tập rỗng", "empty set", "∅",
    ]},
    SeedNode { name: "for_all", codepoint: 0x2200, aliases: &[
        "\\forall", "voi-moi", "với mọi", "for all", "∀",
    ]},
    SeedNode { name: "exists", codepoint: 0x2203, aliases: &[
        "\\exists", "ton-tai", "tồn tại", "exists", "∃",
    ]},
    SeedNode { name: "not_exists", codepoint: 0x2204, aliases: &[
        "\\nexists", "khong-ton-tai", "không tồn tại", "not exists", "∄",
    ]},
];

// ─── Logic ──────────────────────────────────────────────────────────────────

pub static LOGIC_NODES: &[SeedNode] = &[
    SeedNode { name: "logical_and", codepoint: 0x2227, aliases: &[
        "\\land", "\\wedge", "va", "và", "logical and", "conjunction", "∧",
    ]},
    SeedNode { name: "logical_or", codepoint: 0x2228, aliases: &[
        "\\lor", "\\vee", "hoac", "hoặc", "logical or", "disjunction", "∨",
    ]},
    SeedNode { name: "logical_not", codepoint: 0x00AC, aliases: &[
        "\\neg", "\\lnot", "phu-dinh", "phủ định", "logical not", "negation", "¬",
    ]},
    SeedNode { name: "implies", codepoint: 0x21D2, aliases: &[
        "\\Rightarrow", "\\implies", "suy-ra", "suy ra", "implies", "⇒",
    ]},
    SeedNode { name: "iff", codepoint: 0x21D4, aliases: &[
        "\\Leftrightarrow", "\\iff", "khi-va-chi-khi", "if and only if", "⇔",
    ]},
    SeedNode { name: "therefore", codepoint: 0x2234, aliases: &[
        "\\therefore", "vi-vay", "vì vậy", "therefore", "∴",
    ]},
    SeedNode { name: "because", codepoint: 0x2235, aliases: &[
        "\\because", "boi-vi", "bởi vì", "because", "∵",
    ]},
];

// ─── Comparison & Equivalence ───────────────────────────────────────────────

pub static COMPARISON_NODES: &[SeedNode] = &[
    SeedNode { name: "not_equal", codepoint: 0x2260, aliases: &[
        "\\neq", "\\ne", "khac", "khác", "not equal", "≠",
    ]},
    SeedNode { name: "approx", codepoint: 0x2248, aliases: &[
        "\\approx", "xap-xi", "xấp xỉ", "approximately", "≈",
    ]},
    SeedNode { name: "equiv", codepoint: 0x2261, aliases: &[
        "\\equiv", "tuong-duong", "tương đương", "equivalent", "identical", "≡",
    ]},
    SeedNode { name: "leq", codepoint: 0x2264, aliases: &[
        "\\leq", "\\le", "nho-hon-bang", "nhỏ hơn bằng", "less or equal", "≤",
    ]},
    SeedNode { name: "geq", codepoint: 0x2265, aliases: &[
        "\\geq", "\\ge", "lon-hon-bang", "lớn hơn bằng", "greater or equal", "≥",
    ]},
    SeedNode { name: "much_less", codepoint: 0x226A, aliases: &[
        "\\ll", "nho-hon-nhieu", "much less than", "≪",
    ]},
    SeedNode { name: "much_greater", codepoint: 0x226B, aliases: &[
        "\\gg", "lon-hon-nhieu", "much greater than", "≫",
    ]},
];

// ─── Greek Letters (math usage) ─────────────────────────────────────────────

pub static GREEK_NODES: &[SeedNode] = &[
    SeedNode { name: "alpha", codepoint: 0x03B1, aliases: &["\\alpha", "α"] },
    SeedNode { name: "beta", codepoint: 0x03B2, aliases: &["\\beta", "β"] },
    SeedNode { name: "gamma", codepoint: 0x03B3, aliases: &["\\gamma", "γ"] },
    SeedNode { name: "delta_letter", codepoint: 0x03B4, aliases: &["\\delta", "δ"] },
    SeedNode { name: "epsilon", codepoint: 0x03B5, aliases: &["\\epsilon", "ε"] },
    SeedNode { name: "zeta", codepoint: 0x03B6, aliases: &["\\zeta", "ζ"] },
    SeedNode { name: "eta", codepoint: 0x03B7, aliases: &["\\eta", "η"] },
    SeedNode { name: "theta", codepoint: 0x03B8, aliases: &["\\theta", "θ"] },
    SeedNode { name: "lambda_letter", codepoint: 0x03BB, aliases: &["\\lambda", "λ"] },
    SeedNode { name: "mu_letter", codepoint: 0x03BC, aliases: &["\\mu", "μ"] },
    SeedNode { name: "pi_const", codepoint: 0x03C0, aliases: &[
        "\\pi", "pi", "số pi", "pi=16·arctan(1/5)-4·arctan(1/239)",
    ]},
    SeedNode { name: "sigma_letter", codepoint: 0x03C3, aliases: &["\\sigma", "σ"] },
    SeedNode { name: "tau", codepoint: 0x03C4, aliases: &["\\tau", "τ", "tau=2pi"] },
    SeedNode { name: "phi_letter", codepoint: 0x03C6, aliases: &["\\phi", "\\varphi", "φ", "phi=(1+sqrt(5))/2"] },
    SeedNode { name: "omega_letter", codepoint: 0x03C9, aliases: &["\\omega", "ω"] },
    SeedNode { name: "capital_sigma", codepoint: 0x03A3, aliases: &["\\Sigma", "Σ"] },
    SeedNode { name: "capital_pi", codepoint: 0x03A0, aliases: &["\\Pi", "Π"] },
    SeedNode { name: "capital_delta", codepoint: 0x0394, aliases: &["\\Delta", "Δ", "bien-thien", "biến thiên"] },
    SeedNode { name: "capital_omega", codepoint: 0x03A9, aliases: &["\\Omega", "Ω", "ohm"] },
];

// ─── Constants & Concepts ───────────────────────────────────────────────────

pub static CONCEPT_NODES: &[SeedNode] = &[
    // Use emoticon-range emojis for abstract concepts
    SeedNode { name: "function", codepoint: 0x1D453, aliases: &[
        "ham-so", "hàm số", "function", "fonction", "f(x)",
    ]},
    SeedNode { name: "equation", codepoint: 0x1D452, aliases: &[
        "phuong-trinh", "phương trình", "equation", "équation",
    ]},
    SeedNode { name: "theorem", codepoint: 0x1D447, aliases: &[
        "dinh-ly", "định lý", "theorem", "théorème",
    ]},
    SeedNode { name: "proof", codepoint: 0x220E, aliases: &[
        "\\blacksquare", "chung-minh", "chứng minh", "proof", "QED", "∎",
    ]},
    SeedNode { name: "matrix", codepoint: 0x1D440, aliases: &[
        "ma-tran", "ma trận", "matrix", "matrice",
    ]},
    SeedNode { name: "vector", codepoint: 0x1D42B, aliases: &[
        "\\vec", "vecto", "véc-tơ", "vector", "vecteur",
    ]},
    SeedNode { name: "determinant", codepoint: 0x1D437, aliases: &[
        "\\det", "dinh-thuc", "định thức", "determinant",
    ]},
    SeedNode { name: "euler_e", codepoint: 0x1D452, aliases: &[
        "e", "so-e", "số e", "euler number", "e=sum(1/n!,n=0..inf)",
    ]},
    SeedNode { name: "golden_ratio", codepoint: 0x03C6, aliases: &[
        "\\varphi", "ti-le-vang", "tỉ lệ vàng", "golden ratio", "φ",
        "phi=(1+sqrt(5))/2", "phi^2=phi+1",
    ]},
    SeedNode { name: "imaginary_i", codepoint: 0x1D456, aliases: &[
        "don-vi-ao", "đơn vị ảo", "imaginary unit", "i",
    ]},
    SeedNode { name: "euler_gamma", codepoint: 0x03B3, aliases: &[
        "\\gamma", "euler-mascheroni", "hằng số Euler-Mascheroni",
        "gamma=lim(sum(1/k,k=1..n)-ln(n))",
    ]},
    SeedNode { name: "ln2_const", codepoint: 0x2113, aliases: &[
        "ln2", "ln(2)", "log-tu-nhien-2",
        "ln2=sum(1/(n*2^n),n=1..inf)",
    ]},
    SeedNode { name: "sqrt2_const", codepoint: 0x221A, aliases: &[
        "\\sqrt{2}", "sqrt2", "căn 2", "căn bậc hai của 2",
        "sqrt2=Newton(x^2-2=0)",
    ]},
    // Number theory
    SeedNode { name: "prime_number", codepoint: 0x2119, aliases: &[
        "\\mathbb{P}", "so-nguyen-to", "số nguyên tố", "prime", "ℙ",
    ]},
    SeedNode { name: "natural_numbers", codepoint: 0x2115, aliases: &[
        "\\mathbb{N}", "so-tu-nhien", "số tự nhiên", "natural numbers", "ℕ",
    ]},
    SeedNode { name: "integers", codepoint: 0x2124, aliases: &[
        "\\mathbb{Z}", "so-nguyen", "số nguyên", "integers", "ℤ",
    ]},
    SeedNode { name: "rationals", codepoint: 0x211A, aliases: &[
        "\\mathbb{Q}", "so-huu-ti", "số hữu tỉ", "rationals", "ℚ",
    ]},
    SeedNode { name: "reals", codepoint: 0x211D, aliases: &[
        "\\mathbb{R}", "so-thuc", "số thực", "real numbers", "ℝ",
    ]},
    SeedNode { name: "complex_numbers", codepoint: 0x2102, aliases: &[
        "\\mathbb{C}", "so-phuc", "số phức", "complex numbers", "ℂ",
    ]},
];

// ─── Trigonometry (use supplemental math operators) ─────────────────────────

pub static TRIG_NODES: &[SeedNode] = &[
    SeedNode { name: "sine", codepoint: 0x2A0E, aliases: &[
        "\\sin", "sin", "sine",
    ]},
    SeedNode { name: "cosine", codepoint: 0x2A0F, aliases: &[
        "\\cos", "cos", "cosine",
    ]},
    SeedNode { name: "tangent", codepoint: 0x2A10, aliases: &[
        "\\tan", "tan", "tangent",
    ]},
    SeedNode { name: "logarithm", codepoint: 0x2A11, aliases: &[
        "\\log", "log", "logarithm", "logarit",
    ]},
    SeedNode { name: "natural_log", codepoint: 0x2A12, aliases: &[
        "\\ln", "ln", "natural logarithm", "logarit tự nhiên",
    ]},
];

// ─── Edges (relationships between math concepts) ────────────────────────────

/// EdgeKind bytes:
///   Member(0x01), Subset(0x02), Equiv(0x03), Compose(0x05),
///   Causes(0x06), Similar(0x07), DerivedFrom(0x08), Contains(0x09)
pub static MATH_EDGES: &[SeedEdge] = &[
    // Calculus hierarchy
    SeedEdge { from: "integral", to: "summation", relation: 0x07 },      // ∫ ≈ ∑ (continuous vs discrete)
    SeedEdge { from: "double_integral", to: "integral", relation: 0x08 }, // ∬ ← ∫
    SeedEdge { from: "triple_integral", to: "integral", relation: 0x08 }, // ∭ ← ∫
    SeedEdge { from: "contour_integral", to: "integral", relation: 0x08 }, // ∮ ← ∫
    SeedEdge { from: "partial", to: "nabla", relation: 0x05 },           // ∂ ∘ ∇ (compose)
    SeedEdge { from: "integral", to: "partial", relation: 0x04 },        // ∫ ⊥ ∂ (inverse ops)
    SeedEdge { from: "limit", to: "integral", relation: 0x06 },          // lim → ∫ (integral defined via limit)
    SeedEdge { from: "limit", to: "partial", relation: 0x06 },           // lim → ∂ (derivative defined via limit)
    SeedEdge { from: "summation", to: "product_nary", relation: 0x07 },  // ∑ ≈ ∏ (similar operations)
    SeedEdge { from: "summation", to: "limit", relation: 0x08 },         // ∑ ← lim (series)

    // Operators
    SeedEdge { from: "sqrt", to: "cbrt", relation: 0x07 },     // √ ≈ ∛
    SeedEdge { from: "times", to: "divide", relation: 0x04 },   // × ⊥ ÷ (inverse)
    SeedEdge { from: "dot_operator", to: "cross_product", relation: 0x07 }, // · ≈ ×

    // Set theory → Logic
    SeedEdge { from: "element_of", to: "subset", relation: 0x06 },    // ∈ → ⊂
    SeedEdge { from: "union", to: "intersection", relation: 0x04 },   // ∪ ⊥ ∩
    SeedEdge { from: "for_all", to: "exists", relation: 0x04 },       // ∀ ⊥ ∃
    SeedEdge { from: "logical_and", to: "logical_or", relation: 0x04 }, // ∧ ⊥ ∨
    SeedEdge { from: "implies", to: "iff", relation: 0x02 },          // ⇒ ⊂ ⇔
    SeedEdge { from: "therefore", to: "because", relation: 0x04 },    // ∴ ⊥ ∵

    // Number sets (subset chain)
    SeedEdge { from: "natural_numbers", to: "integers", relation: 0x02 },      // ℕ ⊂ ℤ
    SeedEdge { from: "integers", to: "rationals", relation: 0x02 },            // ℤ ⊂ ℚ
    SeedEdge { from: "rationals", to: "reals", relation: 0x02 },               // ℚ ⊂ ℝ
    SeedEdge { from: "reals", to: "complex_numbers", relation: 0x02 },         // ℝ ⊂ ℂ
    SeedEdge { from: "prime_number", to: "natural_numbers", relation: 0x02 },  // ℙ ⊂ ℕ

    // Constants
    SeedEdge { from: "pi_const", to: "reals", relation: 0x01 },       // π ∈ ℝ
    SeedEdge { from: "euler_e", to: "reals", relation: 0x01 },        // e ∈ ℝ
    SeedEdge { from: "golden_ratio", to: "reals", relation: 0x01 },   // φ ∈ ℝ
    SeedEdge { from: "imaginary_i", to: "complex_numbers", relation: 0x01 }, // i ∈ ℂ

    // Trig
    SeedEdge { from: "sine", to: "cosine", relation: 0x07 },    // sin ≈ cos
    SeedEdge { from: "tangent", to: "sine", relation: 0x08 },   // tan ← sin (derived)
    SeedEdge { from: "tangent", to: "cosine", relation: 0x08 }, // tan ← cos (derived)
    SeedEdge { from: "natural_log", to: "logarithm", relation: 0x02 }, // ln ⊂ log

    // Algebra
    SeedEdge { from: "function", to: "equation", relation: 0x06 },    // f(x) → equation
    SeedEdge { from: "theorem", to: "proof", relation: 0x06 },        // theorem → proof
    SeedEdge { from: "matrix", to: "determinant", relation: 0x09 },   // matrix ∋ determinant
    SeedEdge { from: "matrix", to: "vector", relation: 0x09 },        // matrix ∋ vector

    // Greek → concepts
    SeedEdge { from: "capital_sigma", to: "summation", relation: 0x03 }, // Σ ≡ ∑
    SeedEdge { from: "capital_pi", to: "product_nary", relation: 0x03 }, // Π ≡ ∏
    SeedEdge { from: "capital_delta", to: "partial", relation: 0x07 },   // Δ ≈ ∂
];

/// All math nodes combined.
pub fn all_nodes() -> Vec<&'static SeedNode> {
    let mut nodes = Vec::new();
    for n in CALCULUS_NODES { nodes.push(n); }
    for n in OPERATOR_NODES { nodes.push(n); }
    for n in SET_NODES { nodes.push(n); }
    for n in LOGIC_NODES { nodes.push(n); }
    for n in COMPARISON_NODES { nodes.push(n); }
    for n in GREEK_NODES { nodes.push(n); }
    for n in CONCEPT_NODES { nodes.push(n); }
    for n in TRIG_NODES { nodes.push(n); }
    nodes
}
