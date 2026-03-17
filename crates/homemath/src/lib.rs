//! homemath — HomeOS native math library (no_std, zero dependencies)
//!
//! Replaces libm with pure-Rust implementations of all math functions
//! used across the HomeOS workspace.

#![no_std]

// ──────────────────── f64 functions ────────────────────

/// Absolute value (f64).
#[inline]
pub fn fabs(x: f64) -> f64 {
    if x < 0.0 { -x } else { x }
}

/// Floor: largest integer ≤ x (f64).
#[inline]
pub fn floor(x: f64) -> f64 {
    let i = x as i64;
    let f = i as f64;
    if x < f { f - 1.0 } else { f }
}

/// Ceil: smallest integer ≥ x (f64).
#[inline]
pub fn ceil(x: f64) -> f64 {
    let i = x as i64;
    let f = i as f64;
    if x > f { f + 1.0 } else { f }
}

/// Round to nearest integer (f64).
#[inline]
pub fn round(x: f64) -> f64 {
    if x >= 0.0 {
        (x + 0.5) as i64 as f64
    } else {
        (x - 0.5) as i64 as f64
    }
}

/// Square root via bit-level seed + Newton-Raphson (f64).
pub fn sqrt(x: f64) -> f64 {
    if x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 || x == 1.0 {
        return x;
    }
    if x.is_nan() {
        return x;
    }
    if x == f64::INFINITY {
        return x;
    }
    // Bit-level initial guess: halve the exponent
    let bits = x.to_bits();
    let guess_bits = 0x1FF7A3BEA91D9CF1_u64.wrapping_add(bits >> 1);
    let mut y = f64::from_bits(guess_bits);
    // 5 Newton-Raphson iterations for full f64 precision
    y = 0.5 * (y + x / y);
    y = 0.5 * (y + x / y);
    y = 0.5 * (y + x / y);
    y = 0.5 * (y + x / y);
    y = 0.5 * (y + x / y);
    y
}

/// Natural logarithm (f64) — argument reduction + polynomial.
pub fn log(x: f64) -> f64 {
    if x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x.is_nan() {
        return x;
    }
    if x == f64::INFINITY {
        return x;
    }
    if x == 1.0 {
        return 0.0;
    }

    // Decompose x = m * 2^e where 1 <= m < 2
    let bits = x.to_bits();
    let e = ((bits >> 52) & 0x7FF) as i64 - 1023;
    let m_bits = (bits & 0x000FFFFFFFFFFFFF) | 0x3FF0000000000000;
    let m = f64::from_bits(m_bits);

    // ln(x) = e * ln(2) + ln(m)
    // For ln(m) where m in [1, 2), use: let t = (m-1)/(m+1), ln(m) = 2*(t + t^3/3 + t^5/5 + ...)
    let t = (m - 1.0) / (m + 1.0);
    let t2 = t * t;
    let ln_m = 2.0 * t * (1.0
        + t2 * (1.0 / 3.0
        + t2 * (1.0 / 5.0
        + t2 * (1.0 / 7.0
        + t2 * (1.0 / 9.0
        + t2 * (1.0 / 11.0
        + t2 * (1.0 / 13.0
        + t2 * (1.0 / 15.0
        + t2 * (1.0 / 17.0
        + t2 * (1.0 / 19.0
        + t2 * (1.0 / 21.0)))))))))));

    e as f64 * core::f64::consts::LN_2 + ln_m
}

/// Power function (f64): x^y = exp(y * ln(x)).
pub fn pow(x: f64, y: f64) -> f64 {
    if y == 0.0 {
        return 1.0;
    }
    if x == 1.0 {
        return 1.0;
    }
    if y == 1.0 {
        return x;
    }
    if x == 0.0 {
        if y > 0.0 { return 0.0; }
        return f64::INFINITY;
    }

    // Check for integer exponent
    let yi = y as i64;
    if (y - yi as f64).abs() < 1e-15 && yi.abs() <= 300 {
        return pow_int(x, yi);
    }

    // General case: x^y = exp(y * ln(x))
    if x < 0.0 {
        return f64::NAN;
    }
    exp(y * log(x))
}

/// Integer power via squaring.
fn pow_int(mut base: f64, mut exp_val: i64) -> f64 {
    let neg = exp_val < 0;
    if neg { exp_val = -exp_val; }
    let mut result = 1.0;
    while exp_val > 0 {
        if exp_val & 1 == 1 {
            result *= base;
        }
        base *= base;
        exp_val >>= 1;
    }
    if neg { 1.0 / result } else { result }
}

/// Exponential e^x (f64) — range reduction + polynomial.
pub fn exp(x: f64) -> f64 {
    if x.is_nan() { return x; }
    if x == f64::INFINITY { return x; }
    if x == f64::NEG_INFINITY { return 0.0; }
    if fabs(x) < 1e-15 { return 1.0; }

    // Range reduction: e^x = 2^k * e^r where r = x - k*ln(2), |r| <= ln(2)/2
    let k = (x * core::f64::consts::LOG2_E + 0.5_f64.copysign(x)) as i64;
    let r = x - k as f64 * core::f64::consts::LN_2;

    // e^r via Taylor: 1 + r + r^2/2! + ... + r^13/13!
    let mut sum = 1.0;
    let mut term = 1.0;
    for i in 1..=13 {
        term *= r / i as f64;
        sum += term;
    }

    // Multiply by 2^k via bit manipulation
    if k > 1023 { return f64::INFINITY; }
    if k < -1074 { return 0.0; }
    let bits = ((k + 1023) as u64) << 52;
    sum * f64::from_bits(bits)
}

/// Sine (f64) — range reduction + Taylor.
pub fn sin(x: f64) -> f64 {
    if x.is_nan() { return x; }
    if x.is_infinite() { return f64::NAN; }
    sin_taylor_reduced(x, false)
}

/// Cosine (f64) — range reduction + Taylor.
pub fn cos(x: f64) -> f64 {
    if x.is_nan() { return x; }
    if x.is_infinite() { return f64::NAN; }
    sin_taylor_reduced(x, true)
}

/// Clean sin/cos via range reduction and Taylor series.
fn sin_taylor_reduced(x: f64, is_cos: bool) -> f64 {
    let mut t = if is_cos {
        x + core::f64::consts::FRAC_PI_2
    } else {
        x
    };

    // Reduce to [0, 2π)
    let two_pi = 2.0 * core::f64::consts::PI;
    let n = (t / two_pi) as i64;
    t -= n as f64 * two_pi;
    if t < 0.0 { t += two_pi; }

    // Map to [-π/2, π/2] for Taylor
    let mut sign = 1.0;
    if t > core::f64::consts::PI {
        t -= core::f64::consts::PI;
        sign = -1.0;
    }
    if t > core::f64::consts::FRAC_PI_2 {
        t = core::f64::consts::PI - t;
    }

    // Taylor series for sin(t), |t| <= π/2
    let t2 = t * t;
    let s = t * (1.0
        - t2 / 6.0 * (1.0
        - t2 / 20.0 * (1.0
        - t2 / 42.0 * (1.0
        - t2 / 72.0 * (1.0
        - t2 / 110.0 * (1.0
        - t2 / 156.0 * (1.0
        - t2 / 210.0 * (1.0
        - t2 / 272.0))))))));

    sign * s
}

// ──────────────────── f32 functions ────────────────────

/// Absolute value (f32).
#[inline]
pub fn fabsf(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// Maximum of two f32 values.
#[inline]
pub fn fmaxf(x: f32, y: f32) -> f32 {
    if x >= y { x } else { y }
}

/// Minimum of two f32 values.
#[inline]
pub fn fminf(x: f32, y: f32) -> f32 {
    if x <= y { x } else { y }
}

/// Square root (f32) — bit-level seed + Newton-Raphson.
pub fn sqrtf(x: f32) -> f32 {
    if x < 0.0 { return f32::NAN; }
    if x == 0.0 || x == 1.0 { return x; }
    if x.is_nan() { return x; }
    if x == f32::INFINITY { return x; }
    let bits = x.to_bits();
    let guess_bits = 0x1FBD1DF5_u32.wrapping_add(bits >> 1);
    let mut y = f32::from_bits(guess_bits);
    y = 0.5 * (y + x / y);
    y = 0.5 * (y + x / y);
    y = 0.5 * (y + x / y);
    y = 0.5 * (y + x / y);
    y
}

/// Sine (f32).
pub fn sinf(x: f32) -> f32 {
    sin(x as f64) as f32
}

/// Cosine (f32).
pub fn cosf(x: f32) -> f32 {
    cos(x as f64) as f32
}

/// Arc cosine (f32) — via asin with sqrt identity.
pub fn acosf(x: f32) -> f32 {
    if !(-1.0..=1.0).contains(&x) { return f32::NAN; }
    if x == 1.0 { return 0.0; }
    if x == -1.0 { return core::f32::consts::PI; }

    // acos(x) = π/2 - asin(x), computed in f64 for precision
    let xd = x as f64;
    let result = core::f64::consts::FRAC_PI_2 - asin_f64(xd);
    result as f32
}

/// asin(x) for |x| <= 1 via f64 polynomial.
fn asin_f64(x: f64) -> f64 {
    if fabs(x) > 0.7 {
        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let ax = fabs(x);
        let half = (1.0 - ax) / 2.0;
        let s = sqrt(half);
        return sign * (core::f64::consts::FRAC_PI_2 - 2.0 * asin_small(s));
    }
    asin_small(x)
}

/// asin(x) for |x| <= 0.7 via Taylor-like polynomial.
fn asin_small(x: f64) -> f64 {
    let x2 = x * x;
    x * (1.0
        + x2 * (1.0 / 6.0
        + x2 * (3.0 / 40.0
        + x2 * (15.0 / 336.0
        + x2 * (105.0 / 3456.0
        + x2 * (945.0 / 42240.0
        + x2 * (10395.0 / 599040.0
        + x2 * (135135.0 / 9676800.0))))))))
}

/// Power function (f32).
pub fn powf(x: f32, y: f32) -> f32 {
    pow(x as f64, y as f64) as f32
}

/// Log base 2 (f32).
pub fn log2f(x: f32) -> f32 {
    (log(x as f64) * core::f64::consts::LOG2_E) as f32
}

// ──────────────────── Tests ────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const F64_TOL: f64 = 1e-10;
    const F32_TOL: f32 = 1e-5;

    fn assert_close_f64(a: f64, b: f64, tol: f64) {
        let diff = fabs(a - b);
        assert!(diff < tol, "f64: {} vs {} (diff={})", a, b, diff);
    }

    fn assert_close_f32(a: f32, b: f32, tol: f32) {
        let diff = fabsf(a - b);
        assert!(diff < tol, "f32: {} vs {} (diff={})", a, b, diff);
    }

    #[test]
    fn test_sqrt_basic() {
        assert_close_f64(sqrt(4.0), 2.0, F64_TOL);
        assert_close_f64(sqrt(9.0), 3.0, F64_TOL);
        assert_close_f64(sqrt(2.0), 1.4142135623730951, F64_TOL);
        assert_close_f64(sqrt(0.0), 0.0, F64_TOL);
        assert_close_f64(sqrt(1.0), 1.0, F64_TOL);
    }

    #[test]
    fn test_sqrt_phi() {
        let phi = (1.0 + sqrt(5.0)) / 2.0;
        assert_close_f64(phi, 1.6180339887498949, F64_TOL);
    }

    #[test]
    fn test_sqrtf_basic() {
        assert_close_f32(sqrtf(4.0), 2.0, F32_TOL);
        assert_close_f32(sqrtf(2.0), 1.41421356, F32_TOL);
    }

    #[test]
    fn test_sin_basic() {
        assert_close_f64(sin(0.0), 0.0, F64_TOL);
        assert_close_f64(sin(core::f64::consts::FRAC_PI_2), 1.0, F64_TOL);
        assert_close_f64(sin(core::f64::consts::PI), 0.0, F64_TOL);
        assert_close_f64(sin(-core::f64::consts::FRAC_PI_2), -1.0, F64_TOL);
    }

    #[test]
    fn test_cos_basic() {
        assert_close_f64(cos(0.0), 1.0, F64_TOL);
        assert_close_f64(cos(core::f64::consts::FRAC_PI_2), 0.0, F64_TOL);
        assert_close_f64(cos(core::f64::consts::PI), -1.0, F64_TOL);
    }

    #[test]
    fn test_sinf_cosf() {
        assert_close_f32(sinf(0.0), 0.0, F32_TOL);
        assert_close_f32(cosf(0.0), 1.0, F32_TOL);
        assert_close_f32(sinf(core::f32::consts::FRAC_PI_2), 1.0, F32_TOL);
        assert_close_f32(cosf(core::f32::consts::PI), -1.0, F32_TOL);
    }

    #[test]
    fn test_log_basic() {
        assert_close_f64(log(1.0), 0.0, F64_TOL);
        assert_close_f64(log(core::f64::consts::E), 1.0, F64_TOL);
        assert_close_f64(log(10.0), 2.302585092994046, F64_TOL);
    }

    #[test]
    fn test_pow_basic() {
        assert_close_f64(pow(2.0, 10.0), 1024.0, F64_TOL);
        assert_close_f64(pow(3.0, 0.0), 1.0, F64_TOL);
        assert_close_f64(pow(2.0, 0.5), sqrt(2.0), F64_TOL);
    }

    #[test]
    fn test_powf_basic() {
        assert_close_f32(powf(2.0, 3.0), 8.0, F32_TOL);
    }

    #[test]
    fn test_exp_basic() {
        assert_close_f64(exp(0.0), 1.0, F64_TOL);
        assert_close_f64(exp(1.0), core::f64::consts::E, F64_TOL);
        assert_close_f64(exp(2.0), core::f64::consts::E * core::f64::consts::E, F64_TOL);
    }

    #[test]
    fn test_round() {
        assert_close_f64(round(1.5), 2.0, F64_TOL);
        assert_close_f64(round(1.4), 1.0, F64_TOL);
        assert_close_f64(round(-0.6), -1.0, F64_TOL);
    }

    #[test]
    fn test_fabs() {
        assert_close_f64(fabs(-3.14), 3.14, F64_TOL);
        assert_close_f64(fabs(2.71), 2.71, F64_TOL);
    }

    #[test]
    fn test_acosf() {
        assert_close_f32(acosf(1.0), 0.0, F32_TOL);
        assert_close_f32(acosf(0.0), core::f32::consts::FRAC_PI_2, F32_TOL);
        assert_close_f32(acosf(-1.0), core::f32::consts::PI, F32_TOL);
    }

    #[test]
    fn test_log2f() {
        assert_close_f32(log2f(1.0), 0.0, F32_TOL);
        assert_close_f32(log2f(2.0), 1.0, F32_TOL);
        assert_close_f32(log2f(8.0), 3.0, F32_TOL);
    }

    #[test]
    fn test_fmaxf_fminf() {
        assert_close_f32(fmaxf(1.0, 2.0), 2.0, F32_TOL);
        assert_close_f32(fminf(1.0, 2.0), 1.0, F32_TOL);
    }

    #[test]
    fn test_sin_cos_identity() {
        for i in 0..100 {
            let x = i as f64 * 0.1;
            let s = sin(x);
            let c = cos(x);
            assert_close_f64(s * s + c * c, 1.0, 1e-9);
        }
    }

    #[test]
    fn test_exp_log_inverse() {
        for i in 1..50 {
            let x = i as f64 * 0.5;
            assert_close_f64(exp(log(x)), x, 1e-9);
        }
    }

    #[test]
    fn test_sqrt_large_small() {
        assert_close_f64(sqrt(1e10), 1e5, 1e-4);
        assert_close_f64(sqrt(1e-10), 1e-5, 1e-15);
    }
}
