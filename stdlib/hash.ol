// stdlib/hash.ol — Hash functions for Olang (v2)
// FNV-1a, similarity scoring, distance metrics.
// Molecules are now packed u16: [S:4][R:4][V:3][A:3][T:2]

// FNV-1a constants
let FNV_OFFSET = 14695981039346656037;
let FNV_PRIME = 1099511628211;

pub fn fnv1a(data) {
  // Hash bytes → u64 (VM builtin)
  return __fnv1a(data);
}

pub fn hash_str(s) {
  return __fnv1a(__str_bytes(s));
}

pub fn hash_combine(a, b) {
  return a * FNV_PRIME + b;
}

// ── 5D distance between two u16 molecules ───────────────────
// Extracts dims via bit shifts, normalizes by max value per dimension:
//   S max=15 (4 bits), R max=15 (4 bits),
//   V max=7 (3 bits), A max=7 (3 bits), T max=3 (2 bits)
pub fn distance_5d(mol_a, mol_b) {
  let ds = ((mol_a >> 12) & 0xF) - ((mol_b >> 12) & 0xF);
  let dr = ((mol_a >> 8)  & 0xF) - ((mol_b >> 8)  & 0xF);
  let dv = ((mol_a >> 5)  & 0x7) - ((mol_b >> 5)  & 0x7);
  let da = ((mol_a >> 2)  & 0x7) - ((mol_b >> 2)  & 0x7);
  let dt = (mol_a & 0x3)         - (mol_b & 0x3);

  // Normalize each dimension to 0..1 range before computing distance
  let ns = ds / 15.0;
  let nr = dr / 15.0;
  let nv = dv / 7.0;
  let na = da / 7.0;
  let nt = dt / 3.0;

  return sqrt(ns*ns + nr*nr + nv*nv + na*na + nt*nt);
}

// ── Similarity score (0.0 = unrelated, 1.0 = identical) ─────
pub fn similarity(mol_a, mol_b) {
  let d = distance_5d(mol_a, mol_b);
  if d < 0.001 { return 1.0; }
  if d > 1.0 { return 0.0; }
  return 1.0 - d;
}

// ── Nearest similarity in a list of u16 molecules ───────────
pub fn nearest_similarity(mol, mol_list) {
  let best = 0.0;
  let i = 0;
  let n = len(mol_list);
  while i < n {
    let s = similarity(mol, mol_list[i]);
    if s > best { best = s; }
    i = i + 1;
  }
  return best;
}
