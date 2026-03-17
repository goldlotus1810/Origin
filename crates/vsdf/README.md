# vsdf

> Volumetric SDF (Signed Distance Fields) with 18 primitives, Fibonacci Fractal Representation (FFR), vector fields, spline-based physics, and gradient-driven collision.

## Dependencies
- ucd
- olang
- libm

## Files
| File | Purpose |
|------|---------|
| lib.rs | Crate root; re-exports `sdf`, `ffr`, `fit`, `spline`, `vector`, `delta`, `physics` modules (`#![no_std]`) |
| sdf.rs | 18 SDF primitives (`SdfKind`), `Vec3`, `SdfParams`, boolean ops (union/subtract/intersect/smooth_union) |
| ffr.rs | Fibonacci Fractal Representation — 5D spiral addressing (`FfrPoint`), `ffr_chain`, `ffr_nearest` search |
| physics.rs | Analytical gradient per SDF primitive, `Particle` integration, `PhysicsWorld` simulation, diffuse shading |
| vector.rs | `VectorField` (direction + intensity spline) for light/wind/heat/gravity; `EmotionField` (4-spline V/A/D/I) |
| spline.rs | `VectorSpline` and `BezierSegment` for time-varying intensity curves |
| fit.rs | SDF fitting utilities |
| delta.rs | Delta encoding for SDF parameters |

## Key API
```rust
// Evaluate any of the 18 SDF primitives
pub fn sdf(kind: SdfKind, p: Vec3, params: &SdfParams) -> f32;
pub fn SdfKind::from_byte(b: u8) -> Option<Self>;

// Analytical gradient (normal) at point P
pub fn gradient(kind: SdfKind, p: Vec3, params: &SdfParams) -> Vec3;

// Fibonacci spiral 5D addressing
pub fn FfrPoint::at(n: u64) -> FfrPoint;
pub fn ffr_chain(start: u64, n: usize) -> MolecularChain;
pub fn ffr_nearest(target: &Molecule, max_index: u64) -> (u64, f32);

// Physics simulation step with SDF collision
pub fn PhysicsWorld::step(&mut self, particles: &mut [Particle], dt: f32);

// Vector field evaluation over time
pub fn VectorField::evaluate(&self, t: f32) -> Vec3;
pub fn EmotionField::sample(&self, t: f32) -> EmotionSample;
```

## Rules
- `#![no_std]` — uses `alloc` and `libm` only
- SDF primitives are indexed 0x01..0x12 (18 total), mapping to ShapeBase in UCD
- Gradient is analytical (O(1)) for Sphere, Box, Plane, Capsule, Torus, Cylinder; numerical fallback for others
- FFR dimensions: shape (mod 7), relation (mod 8), valence (mod 256), arousal (mod 256), time (mod 5)
- PhysicsWorld pipeline per step: collect VectorField forces, integrate particle, resolve SDF collision via gradient
- Boolean SDF ops: `union`, `subtract`, `intersect`, `smooth_union`

## Test
```bash
cargo test -p vsdf
```
