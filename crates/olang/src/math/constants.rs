//! # constants — Symbolic mathematical constants with adaptive precision
//!
//! Mỗi hằng số = **công thức sinh** (series/formula), KHÔNG phải giá trị cố định.
//! Precision tùy cấu hình máy:
//!   Tier 1 (Full/Server): 50 digits  → 200 iterations
//!   Tier 2 (Compact):     15 digits  → 50  iterations
//!   Tier 3 (Worker):       6 digits  → 15  iterations
//!   Tier 4 (Sensor):       3 digits  → 5   iterations
//!
//! ```text
//! π  = 16·arctan(1/5) − 4·arctan(1/239)     (Machin's formula)
//! e  = Σ 1/n!                                 (Taylor series)
//! φ  = (1 + √5) / 2                          (algebraic)
//! √2 = Newton's method on x² - 2 = 0
//! ln2 = Σ (-1)^(n+1) / n                     (alternating harmonic)
//! γ  = lim(Σ 1/k − ln(n))                    (Euler–Mascheroni)
//! ```

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// Precision tiers
// ─────────────────────────────────────────────────────────────────────────────

/// Precision level for constant computation.
/// Maps to HAL HardwareTier but decoupled (olang doesn't import hal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precision {
    /// 3-4 significant digits (sensor/MCU)
    Low = 0,
    /// 6-8 significant digits (ESP32/worker)
    Medium = 1,
    /// 15-16 significant digits (f64 limit, compact devices)
    High = 2,
    /// Max f64 precision + extra iterations for convergence (PC/Server)
    Ultra = 3,
}

impl Precision {
    /// Number of series iterations for this precision.
    pub fn iterations(self) -> usize {
        match self {
            Self::Low => 5,
            Self::Medium => 15,
            Self::High => 50,
            Self::Ultra => 200,
        }
    }

    /// Number of significant digits this precision targets.
    pub fn digits(self) -> usize {
        match self {
            Self::Low => 3,
            Self::Medium => 7,
            Self::High => 15,
            Self::Ultra => 50, // beyond f64, for future arbitrary precision
        }
    }

    /// From HAL tier byte (1=Full → Ultra, 2=Compact → High, 3=Worker → Medium, 4=Sensor → Low).
    pub fn from_tier_byte(b: u8) -> Self {
        match b {
            1 => Self::Ultra,
            2 => Self::High,
            3 => Self::Medium,
            _ => Self::Low,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MathConstant — symbolic definition of each constant
// ─────────────────────────────────────────────────────────────────────────────

/// A mathematical constant defined by its generating formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathConstant {
    /// π = 16·arctan(1/5) − 4·arctan(1/239) (Machin's formula)
    Pi,
    /// e = Σ(n=0..∞) 1/n! (Taylor series)
    E,
    /// φ = (1 + √5) / 2 (Golden ratio)
    Phi,
    /// √2 (Newton's method)
    Sqrt2,
    /// ln(2) = Σ(n=1..∞) (-1)^(n+1) / n
    Ln2,
    /// γ ≈ 0.5772... (Euler–Mascheroni constant)
    EulerGamma,
    /// τ = 2π (full turn)
    Tau,
    /// Catalan's constant G = Σ (-1)^n / (2n+1)^2
    Catalan,
    /// Apéry's constant ζ(3) = Σ 1/n^3
    Apery,
}

impl MathConstant {
    /// Compute the constant to the given precision using its defining formula.
    pub fn compute(self, precision: Precision) -> f64 {
        let n = precision.iterations();
        match self {
            Self::Pi => compute_pi_machin(n),
            Self::E => compute_e_taylor(n),
            Self::Phi => compute_phi(),
            Self::Sqrt2 => compute_sqrt2_newton(n),
            Self::Ln2 => compute_ln2_series(n),
            Self::EulerGamma => compute_euler_gamma(n),
            Self::Tau => compute_pi_machin(n) * 2.0,
            Self::Catalan => compute_catalan(n),
            Self::Apery => compute_apery(n),
        }
    }

    /// Name of the constant.
    pub fn name(self) -> &'static str {
        match self {
            Self::Pi => "π",
            Self::E => "e",
            Self::Phi => "φ",
            Self::Sqrt2 => "√2",
            Self::Ln2 => "ln(2)",
            Self::EulerGamma => "γ",
            Self::Tau => "τ",
            Self::Catalan => "G",
            Self::Apery => "ζ(3)",
        }
    }

    /// LaTeX representation.
    pub fn latex(self) -> &'static str {
        match self {
            Self::Pi => "\\pi",
            Self::E => "e",
            Self::Phi => "\\varphi",
            Self::Sqrt2 => "\\sqrt{2}",
            Self::Ln2 => "\\ln 2",
            Self::EulerGamma => "\\gamma",
            Self::Tau => "\\tau",
            Self::Catalan => "G",
            Self::Apery => "\\zeta(3)",
        }
    }

    /// Unicode codepoint for this constant (if exists).
    pub fn codepoint(self) -> Option<u32> {
        match self {
            Self::Pi => Some(0x03C0),        // π
            Self::E => None,                  // no dedicated codepoint
            Self::Phi => Some(0x03C6),        // φ
            Self::Sqrt2 => None,              // composite
            Self::Ln2 => None,                // composite
            Self::EulerGamma => Some(0x03B3), // γ
            Self::Tau => Some(0x03C4),        // τ
            Self::Catalan => None,
            Self::Apery => None,
        }
    }

    /// The generating formula as a human-readable string.
    pub fn formula(self) -> &'static str {
        match self {
            Self::Pi => "16·arctan(1/5) - 4·arctan(1/239)",
            Self::E => "sum(1/n!, n=0..inf)",
            Self::Phi => "(1 + sqrt(5)) / 2",
            Self::Sqrt2 => "Newton(x^2 - 2 = 0)",
            Self::Ln2 => "sum((-1)^(n+1)/n, n=1..inf)",
            Self::EulerGamma => "lim(sum(1/k, k=1..n) - ln(n))",
            Self::Tau => "2 * pi",
            Self::Catalan => "sum((-1)^n/(2n+1)^2, n=0..inf)",
            Self::Apery => "sum(1/n^3, n=1..inf)",
        }
    }

    /// All known constants.
    pub fn all() -> &'static [MathConstant] {
        &[
            Self::Pi,
            Self::E,
            Self::Phi,
            Self::Sqrt2,
            Self::Ln2,
            Self::EulerGamma,
            Self::Tau,
            Self::Catalan,
            Self::Apery,
        ]
    }

    /// Lookup by name (supports multiple aliases).
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "pi" | "π" | "PI" => Some(Self::Pi),
            "e" | "E" | "euler" => Some(Self::E),
            "phi" | "φ" | "golden" | "golden_ratio" | "tỷ_lệ_vàng" => Some(Self::Phi),
            "sqrt2" | "√2" | "căn_2" => Some(Self::Sqrt2),
            "ln2" | "ln(2)" => Some(Self::Ln2),
            "gamma" | "γ" | "euler_gamma" | "euler_mascheroni" => Some(Self::EulerGamma),
            "tau" | "τ" => Some(Self::Tau),
            "catalan" | "G" => Some(Self::Catalan),
            "apery" | "ζ(3)" | "zeta3" => Some(Self::Apery),
            _ => None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ConstantRegistry — thread-local cache of computed values per precision
// ─────────────────────────────────────────────────────────────────────────────

/// Registry that caches computed constant values at a given precision.
/// Compute once, use many.
pub struct ConstantRegistry {
    precision: Precision,
    /// Cached: (constant_id, value). Sorted by constant for binary search.
    cache: Vec<(u8, f64)>,
}

impl ConstantRegistry {
    /// Create a new registry with the given precision.
    pub fn new(precision: Precision) -> Self {
        Self {
            precision,
            cache: Vec::new(),
        }
    }

    /// Get or compute a constant value.
    pub fn get(&mut self, constant: MathConstant) -> f64 {
        let id = constant as u8;
        // Linear search (only 9 constants, faster than binary)
        for &(cid, val) in &self.cache {
            if cid == id {
                return val;
            }
        }
        // Compute and cache
        let val = constant.compute(self.precision);
        self.cache.push((id, val));
        val
    }

    /// Current precision.
    pub fn precision(&self) -> Precision {
        self.precision
    }

    /// Change precision (clears cache).
    pub fn set_precision(&mut self, p: Precision) {
        if p != self.precision {
            self.precision = p;
            self.cache.clear();
        }
    }

    /// Compute all constants and return formatted info.
    pub fn info(&mut self) -> String {
        let mut out = String::from("Mathematical Constants (adaptive precision)\n");
        out.push_str(&format!(
            "Precision: {:?} ({} iterations, ~{} digits)\n\n",
            self.precision,
            self.precision.iterations(),
            self.precision.digits(),
        ));
        for &c in MathConstant::all() {
            let val = self.get(c);
            let digits = self.precision.digits().min(16); // f64 max
            out.push_str(&format!(
                "{:<6} = {:.prec$}  ({})\n",
                c.name(),
                val,
                c.formula(),
                prec = digits,
            ));
        }
        out
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Computation functions — the actual formulas
// ─────────────────────────────────────────────────────────────────────────────

/// π via Machin's formula: π = 16·arctan(1/5) − 4·arctan(1/239)
/// arctan(x) = Σ(n=0..N) (-1)^n · x^(2n+1) / (2n+1)
fn compute_pi_machin(iterations: usize) -> f64 {
    16.0 * arctan_series(1.0 / 5.0, iterations) - 4.0 * arctan_series(1.0 / 239.0, iterations)
}

/// arctan(x) via Taylor series.
fn arctan_series(x: f64, n: usize) -> f64 {
    let mut sum = 0.0;
    let mut x_pow = x; // x^1
    let x2 = x * x;
    for k in 0..n {
        let term = x_pow / (2 * k + 1) as f64;
        if k % 2 == 0 {
            sum += term;
        } else {
            sum -= term;
        }
        x_pow *= x2;
    }
    sum
}

/// e via Taylor series: e = Σ(n=0..N) 1/n!
fn compute_e_taylor(iterations: usize) -> f64 {
    let mut sum = 0.0;
    let mut factorial = 1.0;
    for n in 0..iterations {
        sum += 1.0 / factorial;
        factorial *= (n + 1) as f64;
    }
    sum
}

/// φ = (1 + √5) / 2 — algebraic, exact to f64.
fn compute_phi() -> f64 {
    (1.0 + homemath::sqrt(5.0)) / 2.0
}

/// √2 via Newton's method: x_{n+1} = (x_n + 2/x_n) / 2
fn compute_sqrt2_newton(iterations: usize) -> f64 {
    let mut x = 1.5; // initial guess
    for _ in 0..iterations {
        x = (x + 2.0 / x) / 2.0;
    }
    x
}

/// ln(2) via series: ln(2) = Σ(n=1..N) (-1)^(n+1) / n
/// (Slow convergence — uses accelerated formula for higher precision)
fn compute_ln2_series(iterations: usize) -> f64 {
    // Use the faster series: ln(2) = Σ(n=1..N) 1/(n·2^n)
    let mut sum = 0.0;
    let mut pow2 = 2.0; // 2^n
    for n in 1..=iterations {
        sum += 1.0 / (n as f64 * pow2);
        pow2 *= 2.0;
    }
    sum
}

/// Euler–Mascheroni γ ≈ 0.5772156649...
/// γ = lim(n→∞) (Σ(k=1..n) 1/k − ln(n))
fn compute_euler_gamma(iterations: usize) -> f64 {
    let n = iterations.max(10); // need enough terms
    let mut harmonic = 0.0;
    for k in 1..=n {
        harmonic += 1.0 / k as f64;
    }
    harmonic - homemath::log(n as f64)
}

/// Catalan's constant G = Σ(n=0..N) (-1)^n / (2n+1)^2
fn compute_catalan(iterations: usize) -> f64 {
    let mut sum = 0.0;
    for n in 0..iterations {
        let denom = (2 * n + 1) as f64;
        let term = 1.0 / (denom * denom);
        if n % 2 == 0 {
            sum += term;
        } else {
            sum -= term;
        }
    }
    sum
}

/// Apéry's constant ζ(3) = Σ(n=1..N) 1/n^3
fn compute_apery(iterations: usize) -> f64 {
    let mut sum = 0.0;
    for n in 1..=iterations {
        let nf = n as f64;
        sum += 1.0 / (nf * nf * nf);
    }
    sum
}

// ─────────────────────────────────────────────────────────────────────────────
// Fibonacci sequence — adaptive precision
// ─────────────────────────────────────────────────────────────────────────────

/// Compute Fibonacci(n) exactly for n ≤ 92 (u64), or via Binet's formula for larger n.
pub fn fibonacci_u64(n: u64) -> Option<u64> {
    if n > 92 {
        return None; // overflow
    }
    let mut a: u64 = 0;
    let mut b: u64 = 1;
    for _ in 0..n {
        let t = a + b;
        a = b;
        b = t;
    }
    Some(a)
}

/// Compute Fibonacci(n) as f64. Exact for small n, Binet approximation for large n.
pub fn fibonacci(n: u64) -> f64 {
    if let Some(v) = fibonacci_u64(n) {
        v as f64
    } else {
        // Binet's formula: F(n) = φ^n / √5
        let phi = compute_phi();
        let sqrt5 = homemath::sqrt(5.0);
        homemath::round(homemath::pow(phi, n as f64) / sqrt5)
    }
}

/// Fibonacci ratio F(n+1)/F(n) — converges to φ.
/// With `precision` iterations for convergence check.
pub fn fibonacci_ratio(n: u64) -> f64 {
    if n == 0 {
        return 1.0; // F(1)/F(0) = 1/0 → undefined, return 1
    }
    let fn1 = fibonacci(n + 1);
    let fn0 = fibonacci(n);
    if fn0 == 0.0 {
        return compute_phi();
    }
    fn1 / fn0
}

// ─────────────────────────────────────────────────────────────────────────────
// Command interface
// ─────────────────────────────────────────────────────────────────────────────

/// Process a constant-related command. Returns formatted result.
///
/// Commands:
///   const pi          → compute π at current precision
///   const all         → list all constants
///   const phi formula → show formula for φ
///   fib 10            → Fibonacci(10) = 55
///   fib ratio 20      → F(21)/F(20) ≈ φ
pub fn process_constant_command(input: &str, precision: Precision) -> String {
    let input = input.trim();
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.is_empty() {
        return String::from("Usage: const <name|all> | fib <n|ratio n>");
    }

    match parts[0] {
        "const" | "hang-so" => {
            if parts.len() < 2 {
                return String::from("Usage: const <name|all|compare>");
            }
            match parts[1] {
                "all" => {
                    let mut reg = ConstantRegistry::new(precision);
                    reg.info()
                }
                "compare" => compare_precisions(),
                name => {
                    if let Some(c) = MathConstant::from_name(name) {
                        if parts.len() > 2 && parts[2] == "formula" {
                            format!(
                                "{} = {}\n  Formula: {}\n  LaTeX: {}",
                                c.name(),
                                c.compute(precision),
                                c.formula(),
                                c.latex()
                            )
                        } else {
                            let val = c.compute(precision);
                            format!(
                                "{} = {:.prec$}\n  Formula: {}\n  Precision: {:?} ({} iterations)",
                                c.name(),
                                val,
                                c.formula(),
                                precision,
                                precision.iterations(),
                                prec = precision.digits().min(16),
                            )
                        }
                    } else {
                        format!("Unknown constant: {}. Try: pi, e, phi, sqrt2, ln2, gamma, tau, catalan, apery", name)
                    }
                }
            }
        }

        "fib" | "fibonacci" => {
            if parts.len() < 2 {
                return String::from("Usage: fib <n> | fib ratio <n>");
            }
            if parts[1] == "ratio" {
                let n: u64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
                let r = fibonacci_ratio(n);
                format!(
                    "F({})/F({}) = {:.15}\nφ           = {:.15}\nDifference  = {:.2e}",
                    n + 1,
                    n,
                    r,
                    compute_phi(),
                    homemath::fabs(r - compute_phi()),
                )
            } else {
                let n: u64 = match parts[1].parse() {
                    Ok(v) => v,
                    Err(_) => return format!("Invalid number: {}", parts[1]),
                };
                if let Some(exact) = fibonacci_u64(n) {
                    format!("F({}) = {}", n, exact)
                } else {
                    let approx = fibonacci(n);
                    format!("F({}) ≈ {:.6e} (Binet approximation)", n, approx)
                }
            }
        }

        _ => String::from("Unknown command. Try: const <name|all> | fib <n>"),
    }
}

/// Show the same constant at all 4 precision levels.
fn compare_precisions() -> String {
    let mut out = String::from("Precision comparison:\n\n");
    let precisions = [
        (Precision::Low, "Sensor (Tier 4)"),
        (Precision::Medium, "Worker (Tier 3)"),
        (Precision::High, "Compact (Tier 2)"),
        (Precision::Ultra, "Full   (Tier 1)"),
    ];
    for &c in &[MathConstant::Pi, MathConstant::E, MathConstant::Phi, MathConstant::EulerGamma] {
        out.push_str(&format!("{}:\n", c.name()));
        for &(p, label) in &precisions {
            let val = c.compute(p);
            let digits = p.digits().min(16);
            out.push_str(&format!(
                "  {:<16} ({:>3} iter) = {:.prec$}\n",
                label,
                p.iterations(),
                val,
                prec = digits,
            ));
        }
        out.push('\n');
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── π ────────────────────────────────────────────────────────────────

    #[test]
    fn pi_low_precision() {
        let pi = MathConstant::Pi.compute(Precision::Low);
        assert!((pi - core::f64::consts::PI).abs() < 0.01, "Low precision π = {}", pi);
    }

    #[test]
    fn pi_medium_precision() {
        let pi = MathConstant::Pi.compute(Precision::Medium);
        assert!((pi - core::f64::consts::PI).abs() < 1e-7, "Medium precision π = {}", pi);
    }

    #[test]
    fn pi_high_precision() {
        let pi = MathConstant::Pi.compute(Precision::High);
        assert!((pi - core::f64::consts::PI).abs() < 1e-15, "High precision π = {}", pi);
    }

    #[test]
    fn pi_ultra_precision() {
        let pi = MathConstant::Pi.compute(Precision::Ultra);
        assert!((pi - core::f64::consts::PI).abs() < 1e-15, "Ultra precision π = {}", pi);
    }

    // ── e ────────────────────────────────────────────────────────────────

    #[test]
    fn e_low_precision() {
        let e = MathConstant::E.compute(Precision::Low);
        assert!((e - core::f64::consts::E).abs() < 0.01, "Low precision e = {}", e);
    }

    #[test]
    fn e_high_precision() {
        let e = MathConstant::E.compute(Precision::High);
        assert!((e - core::f64::consts::E).abs() < 1e-15, "High precision e = {}", e);
    }

    // ── φ ────────────────────────────────────────────────────────────────

    #[test]
    fn phi_exact() {
        let phi = MathConstant::Phi.compute(Precision::Low);
        let expected = (1.0 + homemath::sqrt(5.0)) / 2.0;
        assert!((phi - expected).abs() < 1e-15, "φ = {}", phi);
    }

    #[test]
    fn phi_is_golden_ratio() {
        let phi = MathConstant::Phi.compute(Precision::High);
        // φ² = φ + 1
        assert!((phi * phi - phi - 1.0).abs() < 1e-14, "φ² ≠ φ + 1");
    }

    // ── √2 ───────────────────────────────────────────────────────────────

    #[test]
    fn sqrt2_convergence() {
        let s = MathConstant::Sqrt2.compute(Precision::High);
        assert!((s * s - 2.0).abs() < 1e-15, "√2² = {}", s * s);
    }

    // ── ln(2) ────────────────────────────────────────────────────────────

    #[test]
    fn ln2_convergence() {
        let ln2 = MathConstant::Ln2.compute(Precision::High);
        assert!((ln2 - core::f64::consts::LN_2).abs() < 1e-14, "ln2 = {}", ln2);
    }

    // ── γ ────────────────────────────────────────────────────────────────

    #[test]
    fn euler_gamma_approximate() {
        let gamma = MathConstant::EulerGamma.compute(Precision::Ultra);
        // Known value: 0.5772156649015329...
        assert!((gamma - 0.5772156649015329).abs() < 0.01, "γ = {}", gamma);
    }

    // ── τ ────────────────────────────────────────────────────────────────

    #[test]
    fn tau_is_2pi() {
        let tau = MathConstant::Tau.compute(Precision::High);
        let two_pi = 2.0 * core::f64::consts::PI;
        assert!((tau - two_pi).abs() < 1e-14, "τ = {}", tau);
    }

    // ── Catalan ──────────────────────────────────────────────────────────

    #[test]
    fn catalan_approximate() {
        let g = MathConstant::Catalan.compute(Precision::Ultra);
        // Known value: 0.9159655941772190...
        assert!((g - 0.9159655941772190).abs() < 0.01, "G = {}", g);
    }

    // ── Apéry ────────────────────────────────────────────────────────────

    #[test]
    fn apery_approximate() {
        let z3 = MathConstant::Apery.compute(Precision::Ultra);
        // Known value: 1.2020569031595942...
        assert!((z3 - 1.2020569031595942).abs() < 0.01, "ζ(3) = {}", z3);
    }

    // ── Fibonacci ────────────────────────────────────────────────────────

    #[test]
    fn fib_small() {
        assert_eq!(fibonacci_u64(0), Some(0));
        assert_eq!(fibonacci_u64(1), Some(1));
        assert_eq!(fibonacci_u64(10), Some(55));
        assert_eq!(fibonacci_u64(20), Some(6765));
    }

    #[test]
    fn fib_92_exact() {
        // Largest Fibonacci that fits u64
        assert_eq!(fibonacci_u64(92), Some(7540113804746346429));
    }

    #[test]
    fn fib_93_overflow() {
        assert_eq!(fibonacci_u64(93), None);
    }

    #[test]
    fn fib_ratio_converges_to_phi() {
        let r = fibonacci_ratio(30);
        let phi = compute_phi();
        assert!((r - phi).abs() < 1e-10, "F(31)/F(30) = {}, φ = {}", r, phi);
    }

    // ── Registry ─────────────────────────────────────────────────────────

    #[test]
    fn registry_caches() {
        let mut reg = ConstantRegistry::new(Precision::High);
        let v1 = reg.get(MathConstant::Pi);
        let v2 = reg.get(MathConstant::Pi);
        assert_eq!(v1, v2);
    }

    #[test]
    fn registry_precision_change_clears_cache() {
        let mut reg = ConstantRegistry::new(Precision::Low);
        let _ = reg.get(MathConstant::Pi);
        assert_eq!(reg.cache.len(), 1);
        reg.set_precision(Precision::High);
        assert_eq!(reg.cache.len(), 0);
    }

    // ── Precision comparison ─────────────────────────────────────────────

    #[test]
    fn higher_precision_is_more_accurate() {
        let pi_low = MathConstant::Pi.compute(Precision::Low);
        let pi_high = MathConstant::Pi.compute(Precision::High);
        let real_pi = core::f64::consts::PI;
        assert!(
            (pi_high - real_pi).abs() <= (pi_low - real_pi).abs(),
            "High should be closer: low_err={}, high_err={}",
            (pi_low - real_pi).abs(),
            (pi_high - real_pi).abs(),
        );
    }

    // ── Name lookup ──────────────────────────────────────────────────────

    #[test]
    fn lookup_by_name() {
        assert_eq!(MathConstant::from_name("pi"), Some(MathConstant::Pi));
        assert_eq!(MathConstant::from_name("π"), Some(MathConstant::Pi));
        assert_eq!(MathConstant::from_name("phi"), Some(MathConstant::Phi));
        assert_eq!(MathConstant::from_name("golden"), Some(MathConstant::Phi));
        assert_eq!(MathConstant::from_name("gamma"), Some(MathConstant::EulerGamma));
        assert_eq!(MathConstant::from_name("unknown"), None);
    }

    // ── Command interface ────────────────────────────────────────────────

    #[test]
    fn cmd_const_pi() {
        let out = process_constant_command("const pi", Precision::High);
        assert!(out.contains("π"), "output: {}", out);
        assert!(out.contains("3.14"), "output: {}", out);
    }

    #[test]
    fn cmd_const_all() {
        let out = process_constant_command("const all", Precision::High);
        assert!(out.contains("π"));
        assert!(out.contains("e"));
        assert!(out.contains("φ"));
    }

    #[test]
    fn cmd_const_compare() {
        let out = process_constant_command("const compare", Precision::High);
        assert!(out.contains("Sensor"));
        assert!(out.contains("Full"));
    }

    #[test]
    fn cmd_fib() {
        let out = process_constant_command("fib 10", Precision::High);
        assert!(out.contains("55"), "output: {}", out);
    }

    #[test]
    fn cmd_fib_ratio() {
        let out = process_constant_command("fib ratio 20", Precision::High);
        assert!(out.contains("1.618"), "output: {}", out);
    }

    #[test]
    fn cmd_const_formula() {
        let out = process_constant_command("const pi formula", Precision::High);
        assert!(out.contains("arctan"), "output: {}", out);
        assert!(out.contains("\\pi"), "output: {}", out);
    }
}
