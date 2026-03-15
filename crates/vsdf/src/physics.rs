//! # physics — Physics từ ∇SDF
//!
//! ∇f(P) = normal analytical tại điểm P.
//! Normal + VectorField forces → chuyển động vật lý.
//!
//! Từ spec MASTER.md:
//!   P_final = P + wind×wind_spline(t)
//!           + gravity×g_spline
//!           + heat×heat_spline(t)
//!   d = sdf(P_final)
//!   n = ∇sdf(P_final)        ← analytical, không numerical
//!
//! Mỗi SDF primitive có ∇f riêng — O(1), không sample.
//! ∇f → normal → diffuse shading → collision response → forces.

extern crate alloc;
use libm::sqrtf;
use crate::sdf::{SdfKind, SdfParams, Vec3, sdf};
use crate::vector::VectorField;
use crate::spline::VectorSpline;

// ─────────────────────────────────────────────────────────────────────────────
// Gradient analytical — ∇f per primitive
// ─────────────────────────────────────────────────────────────────────────────

/// Tính gradient analytical ∇f(P) cho SDF primitive.
///
/// Mỗi primitive có công thức riêng — O(1), không numerical diff.
/// Kết quả là unit normal tại bề mặt gần nhất với P.
pub fn gradient(kind: SdfKind, p: Vec3, params: &SdfParams) -> Vec3 {
    match kind {
        SdfKind::Sphere => grad_sphere(p),
        SdfKind::Plane  => grad_plane(),
        SdfKind::Box    => grad_box(p, params),
        SdfKind::Capsule=> grad_capsule(p, params),
        SdfKind::Torus  => grad_torus(p, params),
        SdfKind::Cylinder => grad_cylinder(p, params),
        _               => grad_numerical(kind, p, params),
    }
}

/// ∇sphere(P) = normalize(P)
fn grad_sphere(p: Vec3) -> Vec3 {
    let len = p.len();
    if len < 1e-6 { Vec3::new(0.0, 1.0, 0.0) } else { p.scale(1.0 / len) }
}

/// ∇plane(P) = (0, 1, 0) — phẳng, normal cố định
fn grad_plane() -> Vec3 { Vec3::new(0.0, 1.0, 0.0) }

/// ∇box(P, b) — sign(P) × step(|P| > b)
fn grad_box(p: Vec3, params: &SdfParams) -> Vec3 {
    let bx = params.b.x.max(1e-6);
    let by = params.b.y.max(1e-6);
    let bz = params.b.z.max(1e-6);
    let qx = p.x.abs() - bx;
    let qy = p.y.abs() - by;
    let qz = p.z.abs() - bz;
    let mx = qx.max(qy).max(qz);
    if mx > 0.0 {
        // Outside: gradient từ closest point trên bề mặt
        let ox = qx.max(0.0);
        let oy = qy.max(0.0);
        let oz = qz.max(0.0);
        let len = sqrtf(ox*ox + oy*oy + oz*oz).max(1e-6);
        Vec3::new(
            p.x.signum() * ox / len,
            p.y.signum() * oy / len,
            p.z.signum() * oz / len,
        )
    } else {
        // Inside: gradient từ face gần nhất
        let ax = qx.abs();
        let ay = qy.abs();
        let az = qz.abs();
        if ax < ay && ax < az      { Vec3::new(p.x.signum(), 0.0, 0.0) }
        else if ay < az             { Vec3::new(0.0, p.y.signum(), 0.0) }
        else                        { Vec3::new(0.0, 0.0, p.z.signum()) }
    }
}

/// ∇capsule(P, r, h) — normalize(P - clamp(y, 0, h)·ĵ)
fn grad_capsule(p: Vec3, params: &SdfParams) -> Vec3 {
    let h = params.h;
    let clamped_y = p.y.clamp(0.0, h);
    let diff = Vec3::new(p.x, p.y - clamped_y, p.z);
    let len  = diff.len();
    if len < 1e-6 { Vec3::new(0.0, 1.0, 0.0) } else { diff.scale(1.0 / len) }
}

/// ∇torus(P, R, r) — analytical qua chain rule
fn grad_torus(p: Vec3, params: &SdfParams) -> Vec3 {
    let big_r  = params.r;
    let small_r = params.r2;
    let xz_len = sqrtf(p.x*p.x + p.z*p.z).max(1e-6);
    let qx = xz_len - big_r;
    let qy = p.y;
    let q_len = sqrtf(qx*qx + qy*qy).max(1e-6);
    // ∂f/∂x = (qx/q_len)·(x/xz_len), etc.
    Vec3::new(
        (qx / q_len) * (p.x / xz_len),
        qy / q_len,
        (qx / q_len) * (p.z / xz_len),
    )
}

/// ∇cylinder(P, r, h) — radial hoặc cap normal
fn grad_cylinder(p: Vec3, params: &SdfParams) -> Vec3 {
    let r = params.r;
    let h = params.h;
    let xz_len = sqrtf(p.x*p.x + p.z*p.z).max(1e-6);
    let d_radial = xz_len - r;
    let d_cap    = p.y.abs() - h;
    if d_radial > d_cap {
        Vec3::new(p.x / xz_len, 0.0, p.z / xz_len)
    } else {
        Vec3::new(0.0, p.y.signum(), 0.0)
    }
}

/// Fallback: numerical gradient (dùng cho primitives chưa có analytical)
/// Central differences với ε nhỏ — chỉ dùng khi không có analytical.
fn grad_numerical(kind: SdfKind, p: Vec3, params: &SdfParams) -> Vec3 {
    const EPS: f32 = 0.001;
    let dx = Vec3::new(EPS, 0.0, 0.0);
    let dy = Vec3::new(0.0, EPS, 0.0);
    let dz = Vec3::new(0.0, 0.0, EPS);
    let gx = sdf(kind, p.add(dx), params) - sdf(kind, p.sub(dx), params);
    let gy = sdf(kind, p.add(dy), params) - sdf(kind, p.sub(dy), params);
    let gz = sdf(kind, p.add(dz), params) - sdf(kind, p.sub(dz), params);
    let len = sqrtf(gx*gx + gy*gy + gz*gz).max(1e-6);
    Vec3::new(gx/len, gy/len, gz/len)
}

// ─────────────────────────────────────────────────────────────────────────────
// Particle — điểm vật lý chịu lực
// ─────────────────────────────────────────────────────────────────────────────

/// Particle chịu tác động của VectorFields + SDF collisions.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Particle {
    pub pos:  Vec3,
    pub vel:  Vec3,
    pub mass: f32,
    pub radius: f32,
}

impl Particle {
    pub fn new(pos: Vec3, mass: f32) -> Self {
        Self { pos, vel: Vec3::new(0.0, 0.0, 0.0), mass, radius: 0.1 }
    }

    /// Integrate một timestep dt.
    pub fn integrate(&mut self, force: Vec3, dt: f32) {
        // F = ma → a = F/m
        let ax = force.x / self.mass;
        let ay = force.y / self.mass;
        let az = force.z / self.mass;
        // Semi-implicit Euler
        self.vel.x += ax * dt;
        self.vel.y += ay * dt;
        self.vel.z += az * dt;
        self.pos.x += self.vel.x * dt;
        self.pos.y += self.vel.y * dt;
        self.pos.z += self.vel.z * dt;
    }

    /// Damping (drag)
    pub fn damp(&mut self, factor: f32) {
        self.vel.x *= factor;
        self.vel.y *= factor;
        self.vel.z *= factor;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PhysicsWorld — simulation step
// ─────────────────────────────────────────────────────────────────────────────

/// Môi trường vật lý.
#[allow(missing_docs)]
pub struct PhysicsWorld {
    /// SDF obstacle (kind + params)
    pub obstacle_kind:   SdfKind,
    pub obstacle_params: SdfParams,

    /// VectorFields tác động
    pub gravity:  VectorField,
    pub wind:     Option<VectorField>,
    pub heat:     Option<VectorField>,

    /// Damping per step
    pub damping:  f32,

    /// Thời gian simulation (s)
    pub time:     f32,
}

impl PhysicsWorld {
    /// Tạo world đơn giản với gravity + SDF sphere obstacle.
    pub fn simple(kind: SdfKind, params: SdfParams) -> Self {
        Self {
            obstacle_kind:   kind,
            obstacle_params: params,
            gravity:  VectorField::gravity(9.8),
            wind:     None,
            heat:     None,
            damping:  0.98,
            time:     0.0,
        }
    }

    /// Simulate một bước dt.
    ///
    /// Pipeline:
    ///   1. Thu thập forces từ VectorFields
    ///   2. Integrate particle
    ///   3. Collision resolve từ ∇SDF
    pub fn step(&mut self, particles: &mut [Particle], dt: f32) {
        self.time += dt;
        let t_norm = (self.time % 24.0) / 24.0; // normalized [0,1]

        for p in particles.iter_mut() {
            // 1. Forces từ VectorFields
            let g_force = self.gravity.evaluate(t_norm);
            let mut force = g_force;

            if let Some(w) = &self.wind {
                force = force.add(w.evaluate(t_norm));
            }
            if let Some(h) = &self.heat {
                // Heat: lực đẩy lên (buoyancy) proportional to intensity
                let hf = h.evaluate(t_norm);
                force = Vec3::new(force.x, force.y - hf.len() * 0.3, force.z);
            }

            // Scale force by mass
            let net = Vec3::new(force.x * p.mass, force.y * p.mass, force.z * p.mass);

            // 2. Integrate
            p.integrate(net, dt);
            p.damp(self.damping);

            // 3. Collision resolve — ∇SDF
            let d = sdf(self.obstacle_kind, p.pos, &self.obstacle_params);
            if d < p.radius {
                // Bề mặt: đẩy ra theo normal
                let n = gradient(self.obstacle_kind, p.pos, &self.obstacle_params);
                let penetration = p.radius - d;
                p.pos = Vec3::new(
                    p.pos.x + n.x * penetration,
                    p.pos.y + n.y * penetration,
                    p.pos.z + n.z * penetration,
                );
                // Reflect velocity theo normal (hệ số nảy 0.3)
                let vdotn = p.vel.x*n.x + p.vel.y*n.y + p.vel.z*n.z;
                if vdotn < 0.0 {
                    p.vel.x -= 2.0 * vdotn * n.x * 0.3;
                    p.vel.y -= 2.0 * vdotn * n.y * 0.3;
                    p.vel.z -= 2.0 * vdotn * n.z * 0.3;
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Diffuse shading — dot(n, light)
// ─────────────────────────────────────────────────────────────────────────────

/// Tính diffuse shading tại điểm p trên SDF surface.
///
/// `shade = ambient + max(0, dot(∇f, light)) × intensity`
pub fn diffuse_shade(
    kind: SdfKind, p: Vec3, params: &SdfParams,
    light_dir: Vec3, ambient: f32, intensity: f32,
) -> f32 {
    let n = gradient(kind, p, params);
    let dot = (n.x*light_dir.x + n.y*light_dir.y + n.z*light_dir.z).max(0.0);
    (ambient + dot * intensity).clamp(0.0, 1.0)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere_params() -> SdfParams {
        SdfParams { r: 1.0, h: 0.0, r2: 0.0, b: Vec3::new(1.0,1.0,1.0) }
    }

    fn box_params() -> SdfParams {
        SdfParams { r: 0.5, h: 0.5, r2: 0.0, b: Vec3::new(1.0, 0.5, 0.5) }
    }

    // ── Gradient ──────────────────────────────────────────────────────────────

    #[test]
    fn sphere_gradient_points_outward() {
        // Point on +X axis → gradient = (1, 0, 0)
        let p = Vec3::new(2.0, 0.0, 0.0);
        let g = gradient(SdfKind::Sphere, p, &sphere_params());
        assert!((g.x - 1.0).abs() < 0.01, "Sphere grad X: {}", g.x);
        assert!(g.y.abs() < 0.01, "Sphere grad Y=0: {}", g.y);
    }

    #[test]
    fn sphere_gradient_normalized() {
        let p = Vec3::new(1.5, 0.8, 0.3);
        let g = gradient(SdfKind::Sphere, p, &sphere_params());
        let len = sqrtf(g.x*g.x + g.y*g.y + g.z*g.z);
        assert!((len - 1.0).abs() < 0.01, "Gradient phải normalized: {}", len);
    }

    #[test]
    fn plane_gradient_always_up() {
        // Plane gradient = (0,1,0) bất kể điểm nào
        for p in [Vec3::new(0.0,2.0,0.0), Vec3::new(5.0,-1.0,3.0)] {
            let g = gradient(SdfKind::Plane, p, &sphere_params());
            assert!((g.y - 1.0).abs() < 0.01, "Plane normal up: {}", g.y);
        }
    }

    #[test]
    fn box_gradient_outside_points_away() {
        // Point above box → gradient points up
        let p = Vec3::new(0.0, 2.0, 0.0);
        let g = gradient(SdfKind::Box, p, &box_params());
        assert!(g.y > 0.5, "Box above → normal up: {}", g.y);
    }

    #[test]
    fn capsule_gradient_normalized() {
        let p    = Vec3::new(1.5, 0.3, 0.0);
        let params = SdfParams { r: 0.3, h: 1.0, r2: 0.0, b: Vec3::new(0.3,0.3,0.3) };
        let g = gradient(SdfKind::Capsule, p, &params);
        let len = sqrtf(g.x*g.x + g.y*g.y + g.z*g.z);
        assert!((len - 1.0).abs() < 0.01, "Capsule gradient normalized: {}", len);
    }

    #[test]
    fn numerical_matches_analytical_sphere() {
        // Verify analytical ≈ numerical
        let p = Vec3::new(1.5, 0.8, 0.3);
        let analytical  = grad_sphere(p);
        let numerical   = grad_numerical(SdfKind::Sphere, p, &sphere_params());
        let dx = (analytical.x - numerical.x).abs();
        let dy = (analytical.y - numerical.y).abs();
        let dz = (analytical.z - numerical.z).abs();
        assert!(dx < 0.01 && dy < 0.01 && dz < 0.01,
            "Analytical vs numerical: d=({:.4},{:.4},{:.4})", dx, dy, dz);
    }

    // ── Particle ──────────────────────────────────────────────────────────────

    #[test]
    fn particle_falls_under_gravity() {
        let mut p = Particle::new(Vec3::new(0.0, 5.0, 0.0), 1.0);
        let gravity = Vec3::new(0.0, -9.8, 0.0);
        let dt = 0.016; // ~60fps
        let y0 = p.pos.y;
        p.integrate(gravity, dt);
        assert!(p.pos.y < y0, "Particle falls: {} < {}", p.pos.y, y0);
        assert!(p.vel.y < 0.0, "Velocity down: {}", p.vel.y);
    }

    #[test]
    fn particle_bounces_on_sphere() {
        let params = sphere_params();
        let mut world = PhysicsWorld::simple(SdfKind::Sphere, params);

        // Particle falling from above sphere
        let mut particles = alloc::vec![
            Particle::new(Vec3::new(0.0, 3.0, 0.0), 1.0)
        ];

        // Simulate until hits sphere
        for _ in 0..200 {
            world.step(&mut particles, 0.016);
        }

        // Particle should be above sphere surface (r=1 → y > 0.9)
        assert!(particles[0].pos.y >= 0.9,
            "Particle above sphere: y={}", particles[0].pos.y);
    }

    #[test]
    fn particle_damping_slows_down() {
        let mut p = Particle::new(Vec3::new(0.0, 0.0, 0.0), 1.0);
        p.vel = Vec3::new(10.0, 0.0, 0.0);
        let v0 = p.vel.x;
        p.damp(0.98);
        assert!(p.vel.x < v0, "Damping reduces velocity");
        assert!((p.vel.x - v0 * 0.98).abs() < 1e-5);
    }

    // ── Diffuse shading ───────────────────────────────────────────────────────

    #[test]
    fn diffuse_lit_face() {
        // Point on top of sphere, light from above
        let p = Vec3::new(0.0, 1.5, 0.0); // above sphere
        let light = Vec3::new(0.0, 1.0, 0.0); // pointing up
        let shade = diffuse_shade(SdfKind::Sphere, p, &sphere_params(), light, 0.25, 0.75);
        assert!(shade > 0.8, "Lit face bright: {}", shade);
    }

    #[test]
    fn diffuse_shadow_face() {
        // Point on bottom, light from above
        let p = Vec3::new(0.0, -1.5, 0.0);
        let light = Vec3::new(0.0, 1.0, 0.0);
        let shade = diffuse_shade(SdfKind::Sphere, p, &sphere_params(), light, 0.25, 0.75);
        // Normal points down, light points up → diffuse=0 → only ambient
        assert!((shade - 0.25).abs() < 0.05, "Shadow = ambient only: {}", shade);
    }

    #[test]
    fn diffuse_clamped_0_1() {
        let p = Vec3::new(0.0, 2.0, 0.0);
        let light = Vec3::new(0.0, 1.0, 0.0);
        let shade = diffuse_shade(SdfKind::Sphere, p, &sphere_params(), light, 0.5, 1.0);
        assert!(shade >= 0.0 && shade <= 1.0, "Shade trong [0,1]: {}", shade);
    }

    // ── Physics pipeline ──────────────────────────────────────────────────────

    #[test]
    fn physics_world_step_moves_particle() {
        let mut world = PhysicsWorld::simple(SdfKind::Plane,
            SdfParams { r: 0.0, h: -3.0, r2: 0.0, b: Vec3::new(0.0,0.0,0.0) });
        let mut particles = alloc::vec![
            Particle::new(Vec3::new(0.0, 2.0, 0.0), 1.0)
        ];
        let y0 = particles[0].pos.y;
        world.step(&mut particles, 0.1);
        // Gravity pulls down
        assert!(particles[0].pos.y < y0, "Gravity moves particle down");
    }

    #[test]
    fn gradient_consistent_with_sdf_sign() {
        // Ngoài sphere: sdf > 0, gradient hướng ra
        let p_out = Vec3::new(2.0, 0.0, 0.0);
        let g_out = gradient(SdfKind::Sphere, p_out, &sphere_params());
        let d_out = sdf(SdfKind::Sphere, p_out, &sphere_params());
        assert!(d_out > 0.0, "Ngoài sphere: d > 0");
        // Gradient hướng ra → dot với (p - center) > 0
        let dot = g_out.x * p_out.x + g_out.y * p_out.y + g_out.z * p_out.z;
        assert!(dot > 0.0, "Gradient hướng ra: dot > 0");
    }
}
