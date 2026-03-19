// stdlib/mol.ol — Molecule helpers for Olang
// Molecule [S][R][V][A][T] = 5 bytes = tọa độ 5D

// Constructors
pub fn mol_new(s, r, v, a, t) {
  return { s: s, r: r, v: v, a: a, t: t };
}

pub fn mol_default() {
  // Defaults: S=Sphere, R=Member, V=0x80, A=0x80, T=Medium
  return mol_new(1, 1, 128, 128, 3);
}

// Accessors
pub fn shape(mol) { return mol.s; }
pub fn relation(mol) { return mol.r; }
pub fn valence(mol) { return mol.v; }
pub fn arousal(mol) { return mol.a; }
pub fn time(mol) { return mol.t; }

// Shape constants (8 primitives)
let SPHERE = 1;
let LINE = 2;
let BOX = 3;
let TRIANGLE = 4;
let CIRCLE = 5;
let CUP = 6;
let CAP = 7;
let SLASH = 8;

// Relation constants (8 relations)
let MEMBER = 1;
let SUBSET = 2;
let EQUIVALENT = 3;
let ORTHOGONAL = 4;
let COMPOSE = 5;
let CAUSES = 6;
let APPROXIMATES = 7;
let CAUSED_BY = 8;

// Time constants
let STATIC = 1;
let SLOW = 2;
let MEDIUM = 3;
let FAST = 4;
let INSTANT = 5;

// Evolution: mutate 1 dimension → new concept
pub fn evolve(mol, dim, new_val) {
  if dim == "s" || dim == "shape" {
    return mol_new(new_val, mol.r, mol.v, mol.a, mol.t);
  }
  if dim == "r" || dim == "relation" {
    return mol_new(mol.s, new_val, mol.v, mol.a, mol.t);
  }
  if dim == "v" || dim == "valence" {
    return mol_new(mol.s, mol.r, new_val, mol.a, mol.t);
  }
  if dim == "a" || dim == "arousal" {
    return mol_new(mol.s, mol.r, mol.v, new_val, mol.t);
  }
  if dim == "t" || dim == "time" {
    return mol_new(mol.s, mol.r, mol.v, mol.a, new_val);
  }
  return mol;
}

// Find which dimension differs most
pub fn dimension_delta(a, b) {
  let ds = abs(a.s - b.s);
  let dr = abs(a.r - b.r);
  let dv = abs(a.v - b.v);
  let da = abs(a.a - b.a);
  let dt = abs(a.t - b.t);

  let max_d = ds;
  let max_dim = "shape";

  if dr > max_d { max_d = dr; max_dim = "relation"; }
  if dv > max_d { max_d = dv; max_dim = "valence"; }
  if da > max_d { max_d = da; max_dim = "arousal"; }
  if dt > max_d { max_d = dt; max_dim = "time"; }

  return { dim: max_dim, delta: max_d };
}

// LCA of two molecules (per-dimension average)
pub fn mol_lca(a, b) {
  return mol_new(
    (a.s + b.s) / 2,
    (a.r + b.r) / 2,
    (a.v + b.v) / 2,
    (a.a + b.a) / 2,
    (a.t + b.t) / 2
  );
}

// Check consistency: ≥3/4 semantic rules must hold
pub fn is_consistent(mol) {
  let score = 0;
  // Rule 1: shape in valid range (1-8)
  if mol.s >= 1 && mol.s <= 8 { score = score + 1; }
  // Rule 2: relation in valid range (1-8)
  if mol.r >= 1 && mol.r <= 8 { score = score + 1; }
  // Rule 3: valence in byte range (0-255)
  if mol.v >= 0 && mol.v <= 255 { score = score + 1; }
  // Rule 4: time in valid range (1-5)
  if mol.t >= 1 && mol.t <= 5 { score = score + 1; }
  return score >= 3;
}

// Display
pub fn mol_to_str(mol) {
  return "{ S=" + to_string(mol.s) + " R=" + to_string(mol.r) +
         " V=" + to_string(mol.v) + " A=" + to_string(mol.a) +
         " T=" + to_string(mol.t) + " }";
}
