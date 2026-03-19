// stdlib/chain.ol — MolecularChain helpers for Olang
// Chain = ordered sequence of Molecules.

pub fn chain_new() {
  return { mols: [], hash: 0 };
}

pub fn chain_from_mol(mol) {
  return { mols: [mol], hash: __fnv1a(mol_bytes(mol)) };
}

pub fn chain_append(c, mol) {
  push(c.mols, mol);
  c.hash = hash_combine(c.hash, __fnv1a(mol_bytes(mol)));
  return c;
}

pub fn chain_len(c) {
  return len(c.mols);
}

pub fn chain_get(c, idx) {
  if idx < 0 || idx >= len(c.mols) { return mol_default(); }
  return c.mols[idx];
}

pub fn chain_first(c) {
  if len(c.mols) == 0 { return mol_default(); }
  return c.mols[0];
}

pub fn chain_last(c) {
  let n = len(c.mols);
  if n == 0 { return mol_default(); }
  return c.mols[n - 1];
}

// LCA of two chains (per-dimension average of corresponding molecules)
pub fn chain_lca(a, b) {
  let result = chain_new();
  let n = min(chain_len(a), chain_len(b));
  let i = 0;
  while i < n {
    let lca_mol = mol_lca(chain_get(a, i), chain_get(b, i));
    chain_append(result, lca_mol);
    i = i + 1;
  }
  return result;
}

// Concatenate two chains
pub fn chain_concat(a, b) {
  let result = chain_new();
  let i = 0;
  while i < chain_len(a) {
    chain_append(result, chain_get(a, i));
    i = i + 1;
  }
  i = 0;
  while i < chain_len(b) {
    chain_append(result, chain_get(b, i));
    i = i + 1;
  }
  return result;
}

// Split chain at position
pub fn chain_split(c, pos) {
  let left = chain_new();
  let right = chain_new();
  let i = 0;
  let n = chain_len(c);
  while i < n {
    if i < pos {
      chain_append(left, chain_get(c, i));
    } else {
      chain_append(right, chain_get(c, i));
    }
    i = i + 1;
  }
  return [left, right];
}

// Compare chains: -1, 0, 1
pub fn chain_compare(a, b) {
  let na = chain_len(a);
  let nb = chain_len(b);
  let n = min(na, nb);
  let i = 0;
  while i < n {
    let ma = chain_get(a, i);
    let mb = chain_get(b, i);
    // Compare by dimension priority: S, R, V, A, T
    if ma.s != mb.s { if ma.s < mb.s { return -1; } return 1; }
    if ma.r != mb.r { if ma.r < mb.r { return -1; } return 1; }
    if ma.v != mb.v { if ma.v < mb.v { return -1; } return 1; }
    if ma.a != mb.a { if ma.a < mb.a { return -1; } return 1; }
    if ma.t != mb.t { if ma.t < mb.t { return -1; } return 1; }
    i = i + 1;
  }
  if na < nb { return -1; }
  if na > nb { return 1; }
  return 0;
}

// Similarity between two chains (average molecule similarity)
pub fn chain_similarity(a, b) {
  let na = chain_len(a);
  let nb = chain_len(b);
  if na == 0 || nb == 0 { return 0.0; }
  let n = min(na, nb);
  let total = 0.0;
  let i = 0;
  while i < n {
    total = total + similarity(chain_get(a, i), chain_get(b, i));
    i = i + 1;
  }
  return total / n;
}

// Helper: molecule to bytes (for hashing)
fn mol_bytes(mol) {
  return [mol.s, mol.r, mol.v, mol.a, mol.t];
}
