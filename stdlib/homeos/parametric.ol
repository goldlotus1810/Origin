// stdlib/homeos/parametric.ol — Parametric SDF (FE.8: T×S Integration)
// Ported from Rust: crates/vsdf/src/render/parametric.rs (285 LOC)
//
// KEY INSIGHT: T × S = Infinite Shapes from Finite Primitives
//   S = WHAT shape (18 SDF primitives)
//   T = HOW BIG (amplitude), WHERE (phase), HOW FAST (frequency)
//   18 primitives × T knots = unbounded shape diversity

// ═══ 18 SDF Primitives ═══
// 0=Sphere, 1=Box, 2=Capsule, 3=Cylinder, 4=Cone,
// 5=Torus, 6=Plane, 7=Line, 8=Triangle, 9=Hexagon,
// 10=Octahedron, 11=Pyramid, 12=RoundBox, 13=RoundCone,
// 14=Ellipsoid, 15=HollowSphere, 16=CappedTorus, 17=DeathStar

// SDF distance function for each primitive
// Returns signed distance: <0 = inside, >0 = outside, 0 = on surface
pub fn sdf_sphere(px, py, pz, radius) {
  // d = |p| - r
  let d2 = px * px + py * py + pz * pz;
  let d = __isqrt(d2);
  return d - radius;
}

pub fn sdf_box(px, py, pz, sx, sy, sz) {
  // d = |max(|p| - s, 0)|
  let dx = _abs(px) - sx; if dx < 0 { dx = 0; }
  let dy = _abs(py) - sy; if dy < 0 { dy = 0; }
  let dz = _abs(pz) - sz; if dz < 0 { dz = 0; }
  return __isqrt(dx * dx + dy * dy + dz * dz);
}

pub fn sdf_capsule(px, py, pz, h, r) {
  // Capsule along Y axis
  let py2 = py - _clamp(py, 0, h);
  return __isqrt(px * px + py2 * py2 + pz * pz) - r;
}

pub fn sdf_cylinder(px, py, pz, h, r) {
  let d2d = __isqrt(px * px + pz * pz) - r;
  let dy = _abs(py) - h;
  if d2d < 0 { d2d = 0; }
  if dy < 0 { dy = 0; }
  return __isqrt(d2d * d2d + dy * dy);
}

// ═══ Parametric SDF: T modulates S ═══

// Create parametric shape from S dimension + T knot
pub fn psdf_new(shape_idx, amp, phase, freq) {
  return { s: shape_idx, amp: amp, phase: phase, freq: freq };
}

// Evaluate parametric SDF at point (px, py, pz) and time t
pub fn psdf_eval(psdf, px, py, pz, t) {
  let radius = psdf.amp;
  // Apply time modulation: sinusoidal motion
  let offset_y = psdf.phase + _sin_approx(psdf.freq * t) * radius / 10;
  let py_adj = py - offset_y;

  if psdf.s == 0 { return sdf_sphere(px, py_adj, pz, radius); }
  if psdf.s == 1 { return sdf_box(px, py_adj, pz, radius, radius, radius); }
  if psdf.s == 2 { return sdf_capsule(px, py_adj, pz, radius * 2, radius / 2); }
  if psdf.s == 3 { return sdf_cylinder(px, py_adj, pz, radius, radius / 2); }
  // Default: sphere
  return sdf_sphere(px, py_adj, pz, radius);
}

// ═══ CSG Composition ═══

// Union: minimum SDF distance (smooth merge)
pub fn sdf_union(shapes, px, py, pz, t) {
  let min_d = 999999;
  let i = 0;
  while i < len(shapes) {
    let d = psdf_eval(shapes[i], px, py, pz, t);
    if d < min_d { min_d = d; }
    i = i + 1;
  }
  return min_d;
}

// Smooth union: organic blending (k = smoothness factor)
pub fn sdf_smooth_union(shapes, px, py, pz, t, k) {
  if len(shapes) == 0 { return 999999; }
  let result = psdf_eval(shapes[0], px, py, pz, t);
  let i = 1;
  while i < len(shapes) {
    let d = psdf_eval(shapes[i], px, py, pz, t);
    // Smooth min: h = clamp(0.5 + 0.5*(d-result)/k, 0, 1)
    // result = mix(d, result, h) - k*h*(1-h)
    let h = 500 + (d - result) * 500 / k;
    if h < 0 { h = 0; }
    if h > 1000 { h = 1000; }
    result = d + (result - d) * h / 1000 - k * h * (1000 - h) / 1000000;
    i = i + 1;
  }
  return result;
}

// ═══ Examples ═══

// Snowman: 3 stacked spheres with phase offsets
pub fn snowman() {
  let shapes = [];
  push(shapes, psdf_new(0, 300, 0, 0));      // Bottom: r=3, y=0
  push(shapes, psdf_new(0, 200, 500, 0));     // Middle: r=2, y=5
  push(shapes, psdf_new(0, 150, 850, 0));     // Head: r=1.5, y=8.5
  return shapes;
}

// ═══ Helpers ═══

fn _abs(x) { if x < 0 { return 0 - x; } return x; }
fn _clamp(x, lo, hi) { if x < lo { return lo; } if x > hi { return hi; } return x; }

// Sine approximation (integer math, *1000)
// Taylor series: sin(x) ≈ x - x³/6 + x⁵/120
fn _sin_approx(x) {
  // Normalize x to [-3141, 3141] (*1000 for π)
  let x2 = x % 6283;
  if x2 > 3141 { x2 = x2 - 6283; }
  if x2 < 0 - 3141 { x2 = x2 + 6283; }
  // sin(x) ≈ x - x³/6 (first 2 terms, sufficient for SDF)
  let x3 = x2 * x2 * x2 / 1000000;
  return x2 - x3 / 6;
}
