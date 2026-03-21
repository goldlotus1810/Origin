// stdlib/chain.ol — MolecularChain helpers for Olang (v2)
// Chain = ordered list of u16 packed molecules.
// Each link is a u16 with layout [S:4][R:4][V:3][A:3][T:2].

pub fn chain_new() {
  return { links: [], hash: 0 };
}

// Create chain from a single u16 molecule
pub fn chain_from_mol(mol_u16) {
  return { links: [mol_u16], hash: __fnv1a(mol_bytes(mol_u16)) };
}

// Append a u16 molecule to chain
pub fn chain_append(c, mol_u16) {
  push(c.links, mol_u16);
  c.hash = hash_combine(c.hash, __fnv1a(mol_bytes(mol_u16)));
  return c;
}

pub fn chain_len(c) {
  return len(c.links);
}

// Returns u16 molecule at index
pub fn chain_get(c, idx) {
  if idx < 0 || idx >= len(c.links) { return mol_default(); }
  return c.links[idx];
}

pub fn chain_first(c) {
  if len(c.links) == 0 { return mol_default(); }
  return c.links[0];
}

pub fn chain_last(c) {
  let n = len(c.links);
  if n == 0 { return mol_default(); }
  return c.links[n - 1];
}

// ── LCA of two chains ───────────────────────────────────────
// Per-position mol_lca (or __lca builtin) of corresponding u16 molecules
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

// ── Concatenate two chains ──────────────────────────────────
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

// ── Split chain at position ─────────────────────────────────
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

// ── Compare chains: -1, 0, 1 ────────────────────────────────
// Extracts dims from each u16 molecule via accessors
pub fn chain_compare(a, b) {
  let na = chain_len(a);
  let nb = chain_len(b);
  let n = min(na, nb);
  let i = 0;
  while i < n {
    let ma = chain_get(a, i);
    let mb = chain_get(b, i);
    // Compare by dimension priority: S, R, V, A, T
    let sa = shape(ma);    let sb = shape(mb);
    if sa != sb { if sa < sb { return -1; } return 1; }
    let ra = relation(ma); let rb = relation(mb);
    if ra != rb { if ra < rb { return -1; } return 1; }
    let va = valence(ma);  let vb = valence(mb);
    if va != vb { if va < vb { return -1; } return 1; }
    let aa = arousal(ma);  let ab = arousal(mb);
    if aa != ab { if aa < ab { return -1; } return 1; }
    let ta = time(ma);     let tb = time(mb);
    if ta != tb { if ta < tb { return -1; } return 1; }
    i = i + 1;
  }
  if na < nb { return -1; }
  if na > nb { return 1; }
  return 0;
}

// ── Similarity between two chains ───────────────────────────
// Average molecule similarity across corresponding u16 molecules
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

// ── Helper: u16 molecule to 2-byte array [hi, lo] ──────────
pub fn mol_bytes(mol) {
  let hi = (mol >> 8) & 0xFF;
  let lo = mol & 0xFF;
  return [hi, lo];
}
