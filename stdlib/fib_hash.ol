// stdlib/fib_hash.ol — Fibonacci Hash Table for KnowTree
// Fixed-size 65,536 slots. O(1) insert/lookup via golden ratio.
//
// φ⁻¹ = 0.6180339887... → integer: 40503 (for 16-bit)
// hash(P) = (P × 40503) & 0xFFFF → bijective (gcd(40503,65536)=1)
// No collision. Every P maps to exactly one unique slot.
//
// Usage:
//   let t = fh_new();
//   fh_put(t, p_weight, value);
//   let v = fh_get(t, p_weight);

let PHI_INV_16 = 40503;
let FH_SIZE = 65536;
let FH_MASK = 65535;

// ── Hash function ──────────────────────────────────────────
pub fn fh_hash(p) {
  return __bit_and(p * PHI_INV_16, FH_MASK);
}

// ── Inverse hash (recover P from index) ────────────────────
// Since hash is bijective: inv(idx) = (idx × 40503⁻¹ mod 65536) & 0xFFFF
// 40503⁻¹ mod 65536 = 30599 (precomputed: 40503 × 30599 ≡ 1 mod 65536)
let PHI_INV_16_INV = 30599;

pub fn fh_unhash(idx) {
  return __bit_and(idx * PHI_INV_16_INV, FH_MASK);
}

// ── Create table ───────────────────────────────────────────
pub fn fh_new() {
  let slots = __array_range(FH_SIZE);
  let i = 0;
  while i < FH_SIZE {
    let _ = __set_at(slots, i, 0);
    let i = i + 1;
  };
  return slots;
}

// ── Put: store value at hash(P) ────────────────────────────
pub fn fh_put(table, p, value) {
  let idx = fh_hash(p);
  let _ = __set_at(table, idx, value);
  return table;
}

// ── Get: read value at hash(P) ─────────────────────────────
pub fn fh_get(table, p) {
  let idx = fh_hash(p);
  return __array_get(table, idx);
}

// ── Check if slot is occupied (non-zero) ───────────────────
pub fn fh_has(table, p) {
  return fh_get(table, p) != 0;
}

// ── Delete: reset slot to 0 ───────────────────────────────
pub fn fh_del(table, p) {
  let idx = fh_hash(p);
  let _ = __set_at(table, idx, 0);
  return table;
}
