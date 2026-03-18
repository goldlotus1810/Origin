//! Ed25519 digital signature implementation (RFC 8032)
//!
//! HomeOS native — zero external dependencies.
//! Uses SHA-512 from sha512.rs for key derivation and hashing.

extern crate alloc;

use crate::sha512::Sha512;

// ─────────────────────────────────────────────────────────────────────────────
// Field arithmetic mod p = 2^255 - 19
// ─────────────────────────────────────────────────────────────────────────────

/// Element of GF(2^255 - 19), represented as 5 × 51-bit limbs.
#[derive(Clone, Copy)]
struct Fe([u64; 5]);

const MASK51: u64 = (1u64 << 51) - 1;

impl Fe {
    const ZERO: Fe = Fe([0; 5]);
    const ONE: Fe = Fe([1, 0, 0, 0, 0]);

    fn from_bytes(s: &[u8; 32]) -> Fe {
        let mut h = [0u64; 5];
        // Load 256 bits into 5 × 51-bit limbs (little-endian)
        h[0] = load_le_u64(&s[0..]) & MASK51;
        h[1] = (load_le_u64(&s[6..]) >> 3) & MASK51;
        h[2] = (load_le_u64(&s[12..]) >> 6) & MASK51;
        h[3] = (load_le_u64(&s[19..]) >> 1) & MASK51;
        h[4] = (load_le_u64(&s[24..]) >> 12) & MASK51;
        Fe(h)
    }

    fn to_bytes(self) -> [u8; 32] {
        let mut h = self.reduce();
        // Final reduction: ensure h < p
        let mut q = (h.0[0] + 19) >> 51;
        q = (h.0[1] + q) >> 51;
        q = (h.0[2] + q) >> 51;
        q = (h.0[3] + q) >> 51;
        q = (h.0[4] + q) >> 51;

        h.0[0] += 19 * q;
        let carry = h.0[0] >> 51;
        h.0[0] &= MASK51;
        h.0[1] += carry;
        let carry = h.0[1] >> 51;
        h.0[1] &= MASK51;
        h.0[2] += carry;
        let carry = h.0[2] >> 51;
        h.0[2] &= MASK51;
        h.0[3] += carry;
        let carry = h.0[3] >> 51;
        h.0[3] &= MASK51;
        h.0[4] += carry;
        h.0[4] &= MASK51;

        // Pack 5 × 51-bit limbs into 32 bytes (255 bits, little-endian)
        // Use a 320-bit buffer to avoid shift overflow
        let mut buf = [0u64; 5]; // 320 bits
        buf[0] = h.0[0] | (h.0[1] << 51);         // bits 0..101
        buf[1] = (h.0[1] >> 13) | (h.0[2] << 38);  // bits 64..165
        buf[2] = (h.0[2] >> 26) | (h.0[3] << 25);  // bits 128..229
        buf[3] = (h.0[3] >> 39) | (h.0[4] << 12);  // bits 192..255
        buf[4] = h.0[4] >> 52;                      // bits 256+ (at most 3 bits)

        let mut out = [0u8; 32];
        for i in 0..4 {
            out[i*8..(i+1)*8].copy_from_slice(&buf[i].to_le_bytes());
        }
        out
    }

    fn reduce(&self) -> Fe {
        let mut h = *self;
        let mut carry;
        carry = h.0[0] >> 51; h.0[0] &= MASK51; h.0[1] += carry;
        carry = h.0[1] >> 51; h.0[1] &= MASK51; h.0[2] += carry;
        carry = h.0[2] >> 51; h.0[2] &= MASK51; h.0[3] += carry;
        carry = h.0[3] >> 51; h.0[3] &= MASK51; h.0[4] += carry;
        carry = h.0[4] >> 51; h.0[4] &= MASK51; h.0[0] += carry * 19;
        Fe(h.0)
    }

    fn add(&self, rhs: &Fe) -> Fe {
        Fe([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3],
            self.0[4] + rhs.0[4],
        ]).reduce()
    }

    fn sub(&self, rhs: &Fe) -> Fe {
        let a_bytes = self.to_bytes();
        let b_bytes = rhs.to_bytes();
        let mut result = [0u64; 4];
        let mut borrow = 0i128;
        for i in 0..4 {
            let av = u64::from_le_bytes(a_bytes[i*8..(i+1)*8].try_into().unwrap());
            let bv = u64::from_le_bytes(b_bytes[i*8..(i+1)*8].try_into().unwrap());
            let diff = av as i128 - bv as i128 - borrow;
            if diff < 0 {
                result[i] = (diff + (1i128 << 64)) as u64;
                borrow = 1;
            } else {
                result[i] = diff as u64;
                borrow = 0;
            }
        }
        if borrow != 0 {
            const P: [u64; 4] = [
                0xFFFFFFFFFFFFFFED, 0xFFFFFFFFFFFFFFFF,
                0xFFFFFFFFFFFFFFFF, 0x7FFFFFFFFFFFFFFF,
            ];
            let mut carry = 0u128;
            for i in 0..4 {
                let sum = result[i] as u128 + P[i] as u128 + carry;
                result[i] = sum as u64;
                carry = sum >> 64;
            }
        }
        let mut out = [0u8; 32];
        for i in 0..4 {
            out[i*8..(i+1)*8].copy_from_slice(&result[i].to_le_bytes());
        }
        Fe::from_bytes(&out)
    }

    fn mul(&self, rhs: &Fe) -> Fe {
        let a = &self.0;
        let b = &rhs.0;

        // Schoolbook multiplication with 128-bit intermediates
        let a0 = a[0] as u128;
        let a1 = a[1] as u128;
        let a2 = a[2] as u128;
        let a3 = a[3] as u128;
        let a4 = a[4] as u128;

        let b0 = b[0] as u128;
        let b1 = b[1] as u128;
        let b2 = b[2] as u128;
        let b3 = b[3] as u128;
        let b4 = b[4] as u128;

        // Multiply with reduction: x * 2^255 ≡ 19x (mod p)
        // So overflow from limb[4] wraps back with factor 19
        let b1_19 = (b[1] * 19) as u128;
        let b2_19 = (b[2] * 19) as u128;
        let b3_19 = (b[3] * 19) as u128;
        let b4_19 = (b[4] * 19) as u128;

        let t0 = a0*b0 + a1*b4_19 + a2*b3_19 + a3*b2_19 + a4*b1_19;
        let t1 = a0*b1 + a1*b0 + a2*b4_19 + a3*b3_19 + a4*b2_19;
        let t2 = a0*b2 + a1*b1 + a2*b0 + a3*b4_19 + a4*b3_19;
        let t3 = a0*b3 + a1*b2 + a2*b1 + a3*b0 + a4*b4_19;
        let t4 = a0*b4 + a1*b3 + a2*b2 + a3*b1 + a4*b0;

        // Carry propagation
        let mut r = [0u64; 5];
        r[0] = (t0 & MASK51 as u128) as u64;
        let c = t0 >> 51;
        let t1 = t1 + c;
        r[1] = (t1 & MASK51 as u128) as u64;
        let c = t1 >> 51;
        let t2 = t2 + c;
        r[2] = (t2 & MASK51 as u128) as u64;
        let c = t2 >> 51;
        let t3 = t3 + c;
        r[3] = (t3 & MASK51 as u128) as u64;
        let c = t3 >> 51;
        let t4 = t4 + c;
        r[4] = (t4 & MASK51 as u128) as u64;
        let c = (t4 >> 51) as u64;
        r[0] += c * 19;

        Fe(r).reduce()
    }

    fn square(&self) -> Fe {
        self.mul(self)
    }

    fn neg(&self) -> Fe {
        Fe::ZERO.sub(self)
    }

    /// Compute self^(2^n) by repeated squaring.
    fn pow2k(&self, k: u32) -> Fe {
        let mut r = *self;
        for _ in 0..k {
            r = r.square();
        }
        r
    }

    /// Modular inverse via Fermat's little theorem: a^(p-2) mod p
    fn invert(&self) -> Fe {
        // p - 2 = 2^255 - 21
        // Using addition chain from ref10
        let z2 = self.square();         // z^2
        let z9 = z2.pow2k(2).mul(self); // z^9 (= z^8 * z)
        let z11 = z9.mul(&z2);          // z^11
        let z_5_0 = z11.square().mul(&z9); // z^(2^5 - 1)
        let z_10_0 = z_5_0.pow2k(5).mul(&z_5_0);
        let z_20_0 = z_10_0.pow2k(10).mul(&z_10_0);
        let z_40_0 = z_20_0.pow2k(20).mul(&z_20_0);
        let z_50_0 = z_40_0.pow2k(10).mul(&z_10_0);
        let z_100_0 = z_50_0.pow2k(50).mul(&z_50_0);
        let z_200_0 = z_100_0.pow2k(100).mul(&z_100_0);
        let z_250_0 = z_200_0.pow2k(50).mul(&z_50_0);
        z_250_0.pow2k(5).mul(&z11)
    }

    /// Compute sqrt(u/v) or return None if no square root exists.
    /// Returns sqrt if u/v is a square, using the formula for p ≡ 5 (mod 8).
    fn sqrt_ratio(u: &Fe, v: &Fe) -> Option<Fe> {
        let v3 = v.square().mul(v);
        let v7 = v3.square().mul(v);
        // candidate = u * v^3 * (u * v^7)^((p-5)/8)
        let uv7 = u.mul(&v7);
        let exp = uv7.pow_p_minus_5_div_8();
        let mut r = u.mul(&v3).mul(&exp);

        // Check: v * r^2 == u or v * r^2 == -u
        let check = v.mul(&r.square());
        if fe_eq(&check, u) {
            return Some(r);
        }
        // Multiply by sqrt(-1)
        r = r.mul(&SQRT_M1);
        let check = v.mul(&r.square());
        if fe_eq(&check, u) {
            return Some(r);
        }
        None
    }

    /// Compute self^((p-5)/8) = self^(2^252 - 3)
    fn pow_p_minus_5_div_8(&self) -> Fe {
        let z2 = self.square();
        let z9 = z2.pow2k(2).mul(self);
        let z11 = z9.mul(&z2);
        let z_5_0 = z11.square().mul(&z9);
        let z_10_0 = z_5_0.pow2k(5).mul(&z_5_0);
        let z_20_0 = z_10_0.pow2k(10).mul(&z_10_0);
        let z_40_0 = z_20_0.pow2k(20).mul(&z_20_0);
        let z_50_0 = z_40_0.pow2k(10).mul(&z_10_0);
        let z_100_0 = z_50_0.pow2k(50).mul(&z_50_0);
        let z_200_0 = z_100_0.pow2k(100).mul(&z_100_0);
        let z_250_0 = z_200_0.pow2k(50).mul(&z_50_0);
        z_250_0.pow2k(2).mul(self)
    }
}

/// sqrt(-1) mod p
const SQRT_M1: Fe = Fe([
    0x00061b274a0ea0b0,
    0x0000d5a5fc8f189d,
    0x0007ef5e9cbd0c60,
    0x00078595a6804c9e,
    0x0002b8324804fc1d,
]);

fn load_le_u64(s: &[u8]) -> u64 {
    let mut buf = [0u8; 8];
    let n = if s.len() < 8 { s.len() } else { 8 };
    buf[..n].copy_from_slice(&s[..n]);
    u64::from_le_bytes(buf)
}

fn fe_eq(a: &Fe, b: &Fe) -> bool {
    a.to_bytes() == b.to_bytes()
}

// ─────────────────────────────────────────────────────────────────────────────
// Extended Edwards point (x, y, z, t) where x*y = z*t
// Curve: -x^2 + y^2 = 1 + d*x^2*y^2  where d = -121665/121666
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct EdPoint {
    x: Fe,
    y: Fe,
    z: Fe,
    t: Fe,
}

/// d = -121665/121666 mod p
const D: Fe = Fe([
    0x00034dca135978a3,
    0x0001a8283b156ebd,
    0x0005e7a26001c029,
    0x000739c663a03cbb,
    0x00052036cee2b6ff,
]);

/// 2*d (kept for potential future use in dedicated doubling formula)
const _D2: Fe = Fe([
    0x00069b9426b2f159,
    0x00035050762add7a,
    0x0003cf44c0038052,
    0x0006738cc7407977,
    0x0002406d9dc56dff,
]);

impl EdPoint {
    /// Neutral element (identity).
    fn identity() -> Self {
        EdPoint {
            x: Fe::ZERO,
            y: Fe::ONE,
            z: Fe::ONE,
            t: Fe::ZERO,
        }
    }

    /// Decode a compressed Edwards point (32 bytes, y-coordinate + sign bit).
    fn from_bytes(s: &[u8; 32]) -> Option<Self> {
        let mut y_bytes = *s;
        let sign = (y_bytes[31] >> 7) & 1;
        y_bytes[31] &= 0x7F; // Clear sign bit

        let y = Fe::from_bytes(&y_bytes);

        // x^2 = (y^2 - 1) / (d*y^2 + 1)
        let y2 = y.square();
        let u = y2.sub(&Fe::ONE);       // y^2 - 1
        let v = D.mul(&y2).add(&Fe::ONE); // d*y^2 + 1

        let x = Fe::sqrt_ratio(&u, &v)?;

        // Check sign
        let x = if (x.to_bytes()[0] & 1) != sign {
            x.neg()
        } else {
            x
        };

        let t = x.mul(&y);
        Some(EdPoint { x, y, z: Fe::ONE, t })
    }

    /// Compress to 32 bytes.
    fn to_bytes(self) -> [u8; 32] {
        let zi = self.z.invert();
        let x = self.x.mul(&zi);
        let y = self.y.mul(&zi);
        let mut s = y.to_bytes();
        s[31] ^= (x.to_bytes()[0] & 1) << 7;
        s
    }

    /// Point addition (extended coordinates).
    /// Formula: add-2008-hwcd from hyperelliptic.org for a=-1
    /// A=X1*X2, B=Y1*Y2, C=T1*d*T2, D=Z1*Z2
    /// E=(X1+Y1)*(X2+Y2)-A-B, F=D-C, G=D+C, H=B+A  (H=B-aA=B+A since a=-1)
    /// X3=E*F, Y3=G*H, T3=E*H, Z3=F*G
    fn add(&self, rhs: &EdPoint) -> EdPoint {
        let a = self.x.mul(&rhs.x);
        let b = self.y.mul(&rhs.y);
        let c = self.t.mul(&rhs.t).mul(&D);
        let dd = self.z.mul(&rhs.z);
        let e = (self.x.add(&self.y)).mul(&rhs.x.add(&rhs.y)).sub(&a).sub(&b);
        let f = dd.sub(&c);
        let g = dd.add(&c);
        let h = b.add(&a);

        EdPoint {
            x: e.mul(&f),
            y: g.mul(&h),
            z: f.mul(&g),
            t: e.mul(&h),
        }
    }

    /// Point doubling — uses add(self, self) for correctness.
    fn double(&self) -> EdPoint {
        self.add(self)
    }

    /// Scalar multiplication: self * scalar (256-bit little-endian).
    fn scalar_mul(&self, scalar: &[u8; 32]) -> EdPoint {
        let mut result = EdPoint::identity();
        let mut temp = *self;

        for i in 0..256 {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            if (scalar[byte_idx] >> bit_idx) & 1 == 1 {
                result = result.add(&temp);
            }
            temp = temp.double();
        }
        result
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Ed25519 base point B
// ─────────────────────────────────────────────────────────────────────────────

/// Base point y-coordinate: 4/5 mod p
/// In bytes (compressed): the standard Ed25519 base point
const BASE_POINT_BYTES: [u8; 32] = [
    0x58, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
    0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
    0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
    0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
];

fn base_point() -> EdPoint {
    EdPoint::from_bytes(&BASE_POINT_BYTES).expect("base point must decode")
}

// ─────────────────────────────────────────────────────────────────────────────
// Scalar arithmetic mod L (group order)
// L = 2^252 + 27742317777372353535851937790883648493
// ─────────────────────────────────────────────────────────────────────────────

/// Reduce a 64-byte scalar (from SHA-512) mod L.
fn sc_reduce(s: &[u8; 64]) -> [u8; 32] {
    sc_reduce_simple(s)
}

/// Simple scalar reduction mod L using repeated subtraction / schoolbook.
fn sc_reduce_simple(s: &[u8; 64]) -> [u8; 32] {
    // L as bytes (little-endian)
    const L: [u8; 32] = [
        0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58,
        0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
    ];

    // We need to compute s mod L where s is 512 bits.
    // Use the method: s = s_hi * 2^256 + s_lo
    // Then s mod L = (s_hi * (2^256 mod L) + s_lo) mod L
    // But this requires multiprecision multiplication.

    // Let's do this with a multiprecision approach using u64 limbs.
    // Load s as 8 × u64 limbs (little-endian)
    let mut n = [0u64; 8];
    for i in 0..8 {
        n[i] = u64::from_le_bytes(s[i*8..(i+1)*8].try_into().unwrap());
    }

    // L as 4 × u64 limbs
    let l = [
        u64::from_le_bytes(L[0..8].try_into().unwrap()),
        u64::from_le_bytes(L[8..16].try_into().unwrap()),
        u64::from_le_bytes(L[16..24].try_into().unwrap()),
        u64::from_le_bytes(L[24..32].try_into().unwrap()),
    ];

    // Barrett-like reduction: we'll repeatedly subtract L * 2^(64*i) from n
    // Process from the top limb down
    // Actually, let's use the standard Barrett reduction approach.

    // For correctness, let me use a simple divide-and-reduce approach.
    // The group order L is ~2^252.5, so we need ~4 u64 limbs.

    // Simple approach: compute s mod L using the identity:
    // 2^252 ≡ -27742317777372353535851937790883648493 (mod L)
    // This constant fits in ~125 bits.

    // Actually, let me just do schoolbook long division.
    // Or better: use the approach from ed25519-donna / ref10.

    // For simplicity and correctness, implement modular reduction via
    // repeated conditional subtraction from the MSB.

    // Convert to a big integer representation and reduce
    reduce_512_mod_l(&n, &l)
}

/// Reduce 512-bit number (8 × u64 LE limbs) mod L (4 × u64 LE limbs).
fn reduce_512_mod_l(n: &[u64; 8], _l: &[u64; 4]) -> [u8; 32] {
    // Convert n to bytes, then process MSB-first: acc = (acc * 256 + byte) mod L
    let mut bytes = [0u8; 64];
    for i in 0..8 {
        bytes[i*8..(i+1)*8].copy_from_slice(&n[i].to_le_bytes());
    }

    let mut acc = [0u64; 5];

    for i in (0..64).rev() {
        // acc = acc * 256
        let mut carry = 0u128;
        for limb in &mut acc {
            let v = (*limb as u128) * 256 + carry;
            *limb = v as u64;
            carry = v >> 64;
        }

        // acc += bytes[i]
        let v = acc[0] as u128 + bytes[i] as u128;
        acc[0] = v as u64;
        let mut carry = v >> 64;
        for limb in &mut acc[1..] {
            let v = *limb as u128 + carry;
            *limb = v as u64;
            carry = v >> 64;
        }

        // Reduce: while acc >= L, acc -= L
        while ge_l(&acc) {
            sub_l(&mut acc);
        }
    }

    let mut out = [0u8; 32];
    for i in 0..4 {
        out[i*8..(i+1)*8].copy_from_slice(&acc[i].to_le_bytes());
    }
    out
}

/// Is acc >= L?
fn ge_l(acc: &[u64; 5]) -> bool {
    if acc[4] != 0 { return true; }

    const L: [u64; 4] = [
        0x5812631a5cf5d3ed,
        0x14def9dea2f79cd6,
        0x0000000000000000,
        0x1000000000000000,
    ];

    for i in (0..4).rev() {
        if acc[i] > L[i] { return true; }
        if acc[i] < L[i] { return false; }
    }
    true // equal
}

/// acc -= L
fn sub_l(acc: &mut [u64; 5]) {
    const L: [u64; 4] = [
        0x5812631a5cf5d3ed,
        0x14def9dea2f79cd6,
        0x0000000000000000,
        0x1000000000000000,
    ];

    let mut borrow = 0i128;
    for i in 0..4 {
        let diff = acc[i] as i128 - L[i] as i128 - borrow;
        if diff < 0 {
            acc[i] = (diff + (1i128 << 64)) as u64;
            borrow = 1;
        } else {
            acc[i] = diff as u64;
            borrow = 0;
        }
    }
    if borrow != 0 && acc[4] > 0 {
        acc[4] -= 1;
    }
}

/// Add two 256-bit scalars mod L.
fn sc_add(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u64; 5];
    let mut carry = 0u128;
    for i in 0..4 {
        let va = u64::from_le_bytes(a[i*8..(i+1)*8].try_into().unwrap());
        let vb = u64::from_le_bytes(b[i*8..(i+1)*8].try_into().unwrap());
        let sum = va as u128 + vb as u128 + carry;
        result[i] = sum as u64;
        carry = sum >> 64;
    }
    result[4] = carry as u64;

    // Reduce mod L
    while ge_l(&result) {
        sub_l(&mut result);
    }

    let mut out = [0u8; 32];
    for i in 0..4 {
        out[i*8..(i+1)*8].copy_from_slice(&result[i].to_le_bytes());
    }
    out
}

/// Multiply two 256-bit scalars mod L, producing a 256-bit result.
fn sc_mul(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    // Load as 4 × u64
    let mut al = [0u64; 4];
    let mut bl = [0u64; 4];
    for i in 0..4 {
        al[i] = u64::from_le_bytes(a[i*8..(i+1)*8].try_into().unwrap());
        bl[i] = u64::from_le_bytes(b[i*8..(i+1)*8].try_into().unwrap());
    }

    // Schoolbook multiply → 8 × u64
    let mut product = [0u64; 8];
    for i in 0..4 {
        let mut carry = 0u128;
        for j in 0..4 {
            let v = al[i] as u128 * bl[j] as u128 + product[i + j] as u128 + carry;
            product[i + j] = v as u64;
            carry = v >> 64;
        }
        product[i + 4] = carry as u64;
    }

    // Convert to bytes and use sc_reduce
    let mut bytes = [0u8; 64];
    for i in 0..8 {
        bytes[i*8..(i+1)*8].copy_from_slice(&product[i].to_le_bytes());
    }
    sc_reduce(&bytes)
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API — drop-in replacement for ed25519-dalek
// ─────────────────────────────────────────────────────────────────────────────

/// ED25519 signing key (32-byte seed).
pub struct SigningKey {
    _seed: [u8; 32],
    /// Expanded secret scalar (clamped lower 32 bytes of SHA-512(seed))
    scalar: [u8; 32],
    /// Upper 32 bytes of SHA-512(seed), used as nonce prefix
    nonce_prefix: [u8; 32],
    /// Public key point
    public: VerifyingKey,
}

/// ED25519 verifying (public) key.
#[derive(Clone)]
pub struct VerifyingKey {
    bytes: [u8; 32],
}

/// ED25519 signature (64 bytes: R || S).
pub struct Signature {
    bytes: [u8; 64],
}

impl SigningKey {
    /// Create signing key from a 32-byte seed.
    pub fn from_bytes(seed: &[u8; 32]) -> Self {
        let mut h = Sha512::new();
        h.update(seed);
        let expanded = h.finalize();

        let mut scalar = [0u8; 32];
        scalar.copy_from_slice(&expanded[0..32]);
        // Clamp
        scalar[0] &= 248;
        scalar[31] &= 127;
        scalar[31] |= 64;

        let mut nonce_prefix = [0u8; 32];
        nonce_prefix.copy_from_slice(&expanded[32..64]);

        // Public key = scalar * B
        let public_point = base_point().scalar_mul(&scalar);
        let public_bytes = public_point.to_bytes();

        Self {
            _seed: *seed,
            scalar,
            nonce_prefix,
            public: VerifyingKey { bytes: public_bytes },
        }
    }

    /// Get the verifying (public) key.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.public.clone()
    }

    /// Sign a message.
    pub fn sign(&self, message: &[u8]) -> Signature {
        // r = SHA-512(nonce_prefix || message)
        let mut h = Sha512::new();
        h.update(self.nonce_prefix);
        h.update(message);
        let r_hash = h.finalize();
        let r = sc_reduce(&r_hash);

        // R = r * B
        let big_r = base_point().scalar_mul(&r);
        let big_r_bytes = big_r.to_bytes();

        // k = SHA-512(R || public_key || message)
        let mut h = Sha512::new();
        h.update(big_r_bytes);
        h.update(self.public.bytes);
        h.update(message);
        let k_hash = h.finalize();
        let k = sc_reduce(&k_hash);

        // S = r + k * scalar (mod L)
        let k_a = sc_mul(&k, &self.scalar);
        let s = sc_add(&r, &k_a);

        let mut sig_bytes = [0u8; 64];
        sig_bytes[0..32].copy_from_slice(&big_r_bytes);
        sig_bytes[32..64].copy_from_slice(&s);
        Signature { bytes: sig_bytes }
    }
}

impl VerifyingKey {
    /// Verify a signature.
    #[allow(clippy::result_unit_err)]
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), ()> {
        let r_bytes: [u8; 32] = signature.bytes[0..32].try_into().unwrap();
        let s_bytes: [u8; 32] = signature.bytes[32..64].try_into().unwrap();

        // Decode R
        let big_r = EdPoint::from_bytes(&r_bytes).ok_or(())?;

        // Decode public key A
        let big_a = EdPoint::from_bytes(&self.bytes).ok_or(())?;

        // k = SHA-512(R || A || message)
        let mut h = Sha512::new();
        h.update(r_bytes);
        h.update(self.bytes);
        h.update(message);
        let k_hash = h.finalize();
        let k = sc_reduce(&k_hash);

        // Check: S * B == R + k * A
        let sb = base_point().scalar_mul(&s_bytes);
        let ka = big_a.scalar_mul(&k);
        let rhs = big_r.add(&ka);

        if sb.to_bytes() == rhs.to_bytes() {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl Signature {
    /// Create from 64 bytes.
    pub fn from_bytes(bytes: &[u8; 64]) -> Self {
        Self { bytes: *bytes }
    }

    /// Convert to 64 bytes.
    pub fn to_bytes(&self) -> [u8; 64] {
        self.bytes
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_encode(bytes: &[u8]) -> alloc::string::String {
        let mut s = alloc::string::String::new();
        for b in bytes {
            s.push_str(&alloc::format!("{:02x}", b));
        }
        s
    }

    #[test]
    fn test_fe_roundtrip() {
        // Test that Fe::from_bytes → Fe::to_bytes is identity for the base point y
        let y = Fe::from_bytes(&BASE_POINT_BYTES);
        let out = y.to_bytes();
        // Base point bytes with sign bit cleared
        let mut expected = BASE_POINT_BYTES;
        expected[31] &= 0x7F;
        assert_eq!(out, expected, "Fe roundtrip must preserve bytes");
    }

    #[test]
    fn test_fe_mul_one() {
        let y = Fe::from_bytes(&BASE_POINT_BYTES);
        let result = y.mul(&Fe::ONE);
        assert_eq!(result.to_bytes(), y.to_bytes(), "x * 1 = x");
    }

    #[test]
    fn test_fe_add_sub() {
        // Test with small values first
        let a = Fe::from_bytes(&[42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let b = Fe::from_bytes(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let c = a.add(&b);
        let expected_add = [43, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(c.to_bytes(), expected_add, "42 + 1 = 43");

        let d = c.sub(&b);
        assert_eq!(d.to_bytes(), a.to_bytes(), "43 - 1 = 42");

        // Test with base point
        let a2 = Fe::from_bytes(&BASE_POINT_BYTES);
        let c2 = a2.add(&b);
        let d2 = c2.sub(&b);
        assert_eq!(d2.to_bytes(), a2.to_bytes(), "bp + 1 - 1 = bp");
    }

    #[test]
    fn test_fe_invert() {
        let a = Fe::from_bytes(&BASE_POINT_BYTES);
        let inv = a.invert();
        let product = a.mul(&inv);
        assert_eq!(product.to_bytes(), Fe::ONE.to_bytes(), "a * a^-1 = 1");
    }

    #[test]
    fn test_base_point_decodes() {
        let bp = base_point();
        let encoded = bp.to_bytes();
        assert_eq!(encoded, BASE_POINT_BYTES, "base point round-trip");
    }

    #[test]
    fn test_identity() {
        let id = EdPoint::identity();
        let bytes = id.to_bytes();
        // Identity in Edwards: (0, 1) → y-coordinate = 1, x=0, sign bit = 0
        assert_eq!(bytes[0], 1);
        for i in 1..31 {
            assert_eq!(bytes[i], 0);
        }
        assert_eq!(bytes[31], 0);
    }

    #[test]
    fn test_sign_verify_basic() {
        let seed = [0x42u8; 32];
        let sk = SigningKey::from_bytes(&seed);
        let msg = b"Hello, HomeOS!";
        let sig = sk.sign(msg);

        let vk = sk.verifying_key();
        assert!(vk.verify(msg, &sig).is_ok(), "signature must verify");
    }

    #[test]
    fn test_sign_verify_empty_message() {
        let seed = [0x01u8; 32];
        let sk = SigningKey::from_bytes(&seed);
        let sig = sk.sign(b"");
        assert!(sk.verifying_key().verify(b"", &sig).is_ok());
    }

    #[test]
    fn test_wrong_message_fails() {
        let seed = [0x42u8; 32];
        let sk = SigningKey::from_bytes(&seed);
        let sig = sk.sign(b"correct message");
        let vk = sk.verifying_key();
        assert!(vk.verify(b"wrong message", &sig).is_err());
    }

    #[test]
    fn test_wrong_key_fails() {
        let sk1 = SigningKey::from_bytes(&[0x01u8; 32]);
        let sk2 = SigningKey::from_bytes(&[0x02u8; 32]);
        let sig = sk1.sign(b"message");
        assert!(sk2.verifying_key().verify(b"message", &sig).is_err());
    }

    #[test]
    fn test_tamper_signature_fails() {
        let sk = SigningKey::from_bytes(&[0x42u8; 32]);
        let mut sig = sk.sign(b"message");
        sig.bytes[0] ^= 1; // flip one bit
        assert!(sk.verifying_key().verify(b"message", &sig).is_err());
    }

    #[test]
    fn test_deterministic() {
        let sk = SigningKey::from_bytes(&[0x42u8; 32]);
        let sig1 = sk.sign(b"test");
        let sig2 = sk.sign(b"test");
        assert_eq!(sig1.bytes, sig2.bytes, "Ed25519 must be deterministic");
    }

    #[test]
    fn test_add_identity() {
        let bp = base_point();
        let id = EdPoint::identity();
        let result = bp.add(&id);
        assert_eq!(result.to_bytes(), BASE_POINT_BYTES, "B + O = B");
    }

    #[test]
    fn test_scalar_mul_three() {
        let bp = base_point();
        let mut three = [0u8; 32];
        three[0] = 3;
        let result = bp.scalar_mul(&three);
        let expected = bp.add(&bp).add(&bp);
        assert_eq!(result.to_bytes(), expected.to_bytes(), "3*B");
    }

    #[test]
    fn test_scalar_mul_256() {
        let bp = base_point();
        let mut scalar = [0u8; 32];
        scalar[1] = 1; // scalar = 256
        let result = bp.scalar_mul(&scalar);
        // 256*B = 2^8 * B = 8 doublings
        let mut expected = bp;
        for _ in 0..8 {
            expected = expected.add(&expected); // double
        }
        assert_eq!(result.to_bytes(), expected.to_bytes(), "256*B");
    }

    #[test]
    fn test_scalar_mul_large() {
        let bp = base_point();
        // Test with scalar that has high bits set (like real Ed25519 scalars)
        let mut scalar = [0u8; 32];
        scalar[31] = 0x40; // bit 254 set (like clamped scalars)
        scalar[0] = 0x08;  // also bit 3
        let result = bp.scalar_mul(&scalar);
        let result2 = bp.scalar_mul(&scalar);
        assert_eq!(result.to_bytes(), result2.to_bytes(), "deterministic");

        // Verify by computing same thing differently:
        // scalar = 2^254 + 8
        // result should equal (2^254 * B) + (8 * B)
        let mut eight = [0u8; 32];
        eight[0] = 8;
        let eight_b = bp.scalar_mul(&eight);

        // 2^254 * B = 254 doublings
        let mut pow254_b = bp;
        for _ in 0..254 {
            pow254_b = pow254_b.add(&pow254_b);
        }
        let expected = pow254_b.add(&eight_b);
        assert_eq!(result.to_bytes(), expected.to_bytes(), "2^254*B + 8*B");
    }

    #[test]
    fn test_scalar_mul_one() {
        let bp = base_point();
        let mut one = [0u8; 32];
        one[0] = 1;
        let result = bp.scalar_mul(&one);
        assert_eq!(result.to_bytes(), BASE_POINT_BYTES, "1*B = B");
    }

    #[test]
    fn test_double_vs_add() {
        let bp = base_point();
        let doubled = bp.double();
        let added = bp.add(&bp);
        assert_eq!(doubled.to_bytes(), added.to_bytes(), "2B via double = 2B via add");
    }

    #[test]
    fn test_scalar_mul_two() {
        let bp = base_point();
        let mut two = [0u8; 32];
        two[0] = 2;
        let result = bp.scalar_mul(&two);
        let expected = bp.double();
        assert_eq!(result.to_bytes(), expected.to_bytes(), "2*B = double(B)");
    }

    #[test]
    fn test_pubkey_from_scalar() {
        // Direct test: take the known clamped scalar and compute pubkey
        let scalar_hex = "307c83864f2833cb427a2ef1c00a013cfdff2768d980c0a3a520f006904de94f";
        let mut scalar = [0u8; 32];
        hex_decode(scalar_hex, &mut scalar);
        let pk = base_point().scalar_mul(&scalar);
        let pk_hex = hex_encode(&pk.to_bytes());
        // RFC 8032 test vector 1 public key
        assert_eq!(pk_hex, "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", "pubkey from scalar");
    }

    #[test]
    fn test_sha512_for_ed25519() {
        let seed = [0x9d, 0x61, 0xb1, 0x9d, 0xef, 0xfd, 0x5a, 0x60,
                    0xba, 0x84, 0x4a, 0xf4, 0x92, 0xec, 0x2c, 0xc4,
                    0x44, 0x49, 0xc5, 0x69, 0x7b, 0x32, 0x69, 0x19,
                    0x70, 0x3b, 0xac, 0x03, 0x1c, 0xae, 0x7f, 0x60];
        let mut h = Sha512::new();
        h.update(&seed);
        let out = h.finalize();
        let expected = "357c83864f2833cb427a2ef1c00a013cfdff2768d980c0a3a520f006904de90f9b4f0afe280b746a778684e75442502057b7473a03f08f96f5a38e9287e01f8f";
        assert_eq!(hex_encode(&out), expected, "SHA-512 of RFC8032 seed");
    }

    #[test]
    fn test_scalar_clamping() {
        let seed_hex = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
        let mut seed = [0u8; 32];
        hex_decode(seed_hex, &mut seed);

        let sk = SigningKey::from_bytes(&seed);
        let expected_scalar = "307c83864f2833cb427a2ef1c00a013cfdff2768d980c0a3a520f006904de94f";
        assert_eq!(hex_encode(&sk.scalar), expected_scalar, "clamped scalar");
    }

    // RFC 8032 test vector 1
    #[test]
    fn test_rfc8032_vector1() {
        let seed_hex = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
        let mut seed = [0u8; 32];
        hex_decode(seed_hex, &mut seed);

        let sk = SigningKey::from_bytes(&seed);

        // Expected public key (RFC 8032 Section 7.1 Test 1)
        let expected_pk = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
        assert_eq!(hex_encode(&sk.verifying_key().bytes), expected_pk);

        // Sign empty message
        let sig = sk.sign(b"");
        let expected_sig = "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b";
        assert_eq!(hex_encode(&sig.bytes), expected_sig);

        // Verify
        assert!(sk.verifying_key().verify(b"", &sig).is_ok());
    }

    fn hex_decode(hex: &str, out: &mut [u8]) {
        for i in 0..out.len() {
            out[i] = u8::from_str_radix(&hex[i*2..i*2+2], 16).unwrap();
        }
    }

    #[test]
    fn test_base_point_on_curve() {
        // Verify base point x satisfies x^2 = (y^2-1)/(d*y^2+1)
        let bp = base_point();
        let zi = bp.z.invert();
        let x = bp.x.mul(&zi);
        let y = bp.y.mul(&zi);
        let x2 = x.square();
        let y2 = y.square();
        let u = y2.sub(&Fe::ONE);
        let v = D.mul(&y2).add(&Fe::ONE);
        let lhs = x2.mul(&v);
        assert_eq!(
            hex_encode(&lhs.to_bytes()),
            hex_encode(&u.to_bytes()),
            "base point must satisfy curve equation"
        );
    }

    #[test]
    fn test_d_constant() {
        // d = -121665/121666 mod p: verify 121666*d ≡ -121665 (mod p)
        let mut factor_bytes = [0u8; 32];
        factor_bytes[0] = 0x42; // 121666 = 0x01DB42
        factor_bytes[1] = 0xdb;
        factor_bytes[2] = 0x01;
        let factor = Fe::from_bytes(&factor_bytes);
        let product = D.mul(&factor);

        // p - 121665: 0xED - 0x41 = 0xAC, 0xFF - 0xDB = 0x24, 0xFF - 0x01 = 0xFE
        let mut neg_121665 = [0u8; 32];
        neg_121665[0] = 0xac;
        neg_121665[1] = 0x24;
        neg_121665[2] = 0xfe;
        for i in 3..31 { neg_121665[i] = 0xff; }
        neg_121665[31] = 0x7f;
        let expected = Fe::from_bytes(&neg_121665);
        assert_eq!(hex_encode(&product.to_bytes()), hex_encode(&expected.to_bytes()), "121666*d = -121665 mod p");
    }
}
