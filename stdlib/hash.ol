// stdlib/hash.ol — Hash functions for Olang
// FNV-1a, similarity scoring, distance metrics.

// FNV-1a constants
let FNV_OFFSET = 14695981039346656037;
let FNV_PRIME = 1099511628211;

pub fn fnv1a(data) {
  // Hash bytes → u64
  // Uses VM builtin for efficiency
  return __fnv1a(data);
}

pub fn hash_str(s) {
  // Hash a string
  return __fnv1a(__str_bytes(s));
}

pub fn hash_combine(a, b) {
  // Combine two hashes
  return a * FNV_PRIME + b;
}

// 5D distance between two molecules
pub fn distance_5d(mol_a, mol_b) {
  let ds = mol_a.s - mol_b.s;
  let dr = mol_a.r - mol_b.r;
  let dv = mol_a.v - mol_b.v;
  let da = mol_a.a - mol_b.a;
  let dt = mol_a.t - mol_b.t;
  return sqrt(ds*ds + dr*dr + dv*dv + da*da + dt*dt) / 255.0;
}

// Similarity score (0.0 = unrelated, 1.0 = identical)
pub fn similarity(mol_a, mol_b) {
  let d = distance_5d(mol_a, mol_b);
  if d < 0.001 { return 1.0; }
  if d > 1.0 { return 0.0; }
  return 1.0 - d;
}

// Nearest similarity in a list
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
