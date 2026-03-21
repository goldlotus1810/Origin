// stdlib/mol.ol — Molecule helpers for Olang (v2)
// Molecule = packed u16: [S:4][R:4][V:3][A:3][T:2] = 16 bits

// ── Constructor ──────────────────────────────────────────────
// Packs 5 dimensions into a single u16.
//   S occupies bits 15..12, R bits 11..8, V bits 7..5, A bits 4..2, T bits 1..0
pub fn mol_new(s, r, v, a, t) {
  return (s << 12) | (r << 8) | (v << 5) | (a << 2) | t;
}

// Default molecule: Sphere(0), Member(0), V=4(neutral), A=4(neutral), Medium(2)
pub fn mol_default() {
  return mol_new(0, 0, 4, 4, 2);
}

// ── Accessors (extract via bit shifts) ───────────────────────
pub fn shape(mol)    { return (mol >> 12) & 0xF; }
pub fn relation(mol) { return (mol >> 8)  & 0xF; }
pub fn valence(mol)  { return (mol >> 5)  & 0x7; }
pub fn arousal(mol)  { return (mol >> 2)  & 0x7; }
pub fn time(mol)     { return mol & 0x3; }

// ── Shape constants (18 SDF primitives, 0-17) ────────────────
let SPHERE      = 0;
let BOX         = 1;
let CAPSULE     = 2;
let PLANE       = 3;
let TORUS       = 4;
let ELLIPSOID   = 5;
let CONE        = 6;
let CYLINDER    = 7;
let OCTAHEDRON  = 8;
let PYRAMID     = 9;
let HEX_PRISM   = 10;
let PRISM       = 11;
let ROUND_BOX   = 12;
let LINK        = 13;
let REVOLVE     = 14;
let EXTRUDE     = 15;
let CUT_SPHERE  = 16;
let DEATH_STAR  = 17;

// ── Relation constants (8 relations, 0-7) ────────────────────
let MEMBER       = 0;
let SUBSET       = 1;
let EQUIVALENT   = 2;
let ORTHOGONAL   = 3;
let COMPOSE      = 4;
let CAUSES       = 5;
let APPROXIMATES = 6;
let CAUSED_BY    = 7;

// ── Time constants (4 levels, 0-3) ───────────────────────────
let STATIC = 0;
let SLOW   = 1;
let MEDIUM = 2;
let FAST   = 3;

// ── Evolution: replace one dimension → new packed u16 ────────
pub fn evolve(mol, dim, new_val) {
  let s = shape(mol);
  let r = relation(mol);
  let v = valence(mol);
  let a = arousal(mol);
  let t = time(mol);

  if dim == "s" || dim == "shape"    { s = new_val; }
  if dim == "r" || dim == "relation" { r = new_val; }
  if dim == "v" || dim == "valence"  { v = new_val; }
  if dim == "a" || dim == "arousal"  { a = new_val; }
  if dim == "t" || dim == "time"     { t = new_val; }

  return mol_new(s, r, v, a, t);
}

// ── Dimension delta: find which dimension differs most ───────
pub fn dimension_delta(a, b) {
  let ds = abs(shape(a)    - shape(b));
  let dr = abs(relation(a) - relation(b));
  let dv = abs(valence(a)  - valence(b));
  let da = abs(arousal(a)  - arousal(b));
  let dt = abs(time(a)     - time(b));

  let max_d = ds;
  let max_dim = "shape";

  if dr > max_d { max_d = dr; max_dim = "relation"; }
  if dv > max_d { max_d = dv; max_dim = "valence"; }
  if da > max_d { max_d = da; max_dim = "arousal"; }
  if dt > max_d { max_d = dt; max_dim = "time"; }

  return { dim: max_dim, delta: max_d };
}

// ── LCA (deprecated — v2 uses amplify in Rust LCA) ──────────
// @deprecated: Use Rust-side LCA with amplify instead.
pub fn mol_lca(a, b) {
  return mol_new(
    (shape(a)    + shape(b))    / 2,
    (relation(a) + relation(b)) / 2,
    (valence(a)  + valence(b))  / 2,
    (arousal(a)  + arousal(b))  / 2,
    (time(a)     + time(b))     / 2
  );
}

// ── Consistency check (v2 ranges) ────────────────────────────
// S: 0-17, R: 0-7, V: 0-7, A: 0-7, T: 0-3
pub fn is_consistent(mol) {
  let score = 0;
  let s = shape(mol);
  let r = relation(mol);
  let v = valence(mol);
  let a = arousal(mol);
  let t = time(mol);

  if s >= 0 && s <= 17 { score = score + 1; }
  if r >= 0 && r <= 7  { score = score + 1; }
  if v >= 0 && v <= 7  { score = score + 1; }
  if a >= 0 && a <= 7  { score = score + 1; }
  if t >= 0 && t <= 3  { score = score + 1; }

  return score >= 4;
}

// ── Display ──────────────────────────────────────────────────
pub fn mol_to_str(mol) {
  return "{ S=" + to_string(shape(mol)) +
         " R="  + to_string(relation(mol)) +
         " V="  + to_string(valence(mol)) +
         " A="  + to_string(arousal(mol)) +
         " T="  + to_string(time(mol)) + " }";
}
