//! # scene — 3D Scene Graph from vSDF Chains
//!
//! SceneNode = 1 vật thể (SDF primitive + transform + material).
//! SceneGraph = cây chứa mọi SceneNode → render pipeline.
//!
//! Pipeline:
//!   MolecularChain → decode shape → SceneNode
//!   SceneGraph.render_list() → sorted front-to-back → FFR sample
//!
//! Export: SceneGraph → JSON (for browser/WebGL) or binary (for native)

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::physics::gradient;
use crate::sdf::{sdf, SdfKind, SdfParams, Vec3};

// ─────────────────────────────────────────────────────────────────────────────
// Transform — vị trí + scale + rotation
// ─────────────────────────────────────────────────────────────────────────────

/// Transform 3D đơn giản (position + scale + Y-rotation).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// World position
    pub position: Vec3,
    /// Uniform scale factor
    pub scale: f32,
    /// Y-axis rotation (radians)
    pub rotation_y: f32,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        position: Vec3::ZERO,
        scale: 1.0,
        rotation_y: 0.0,
    };

    pub fn at(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vec3::new(x, y, z),
            scale: 1.0,
            rotation_y: 0.0,
        }
    }

    pub fn with_scale(mut self, s: f32) -> Self {
        self.scale = s;
        self
    }

    pub fn with_rotation(mut self, r: f32) -> Self {
        self.rotation_y = r;
        self
    }

    /// Transform world point → local SDF space.
    pub fn world_to_local(&self, p: Vec3) -> Vec3 {
        let dx = p.x - self.position.x;
        let dy = p.y - self.position.y;
        let dz = p.z - self.position.z;
        // Inverse Y rotation
        let cos = libm::cosf(-self.rotation_y);
        let sin = libm::sinf(-self.rotation_y);
        let lx = (dx * cos - dz * sin) / self.scale;
        let ly = dy / self.scale;
        let lz = (dx * sin + dz * cos) / self.scale;
        Vec3::new(lx, ly, lz)
    }

    /// Transform local SDF point → world space.
    pub fn local_to_world(&self, p: Vec3) -> Vec3 {
        let sx = p.x * self.scale;
        let sy = p.y * self.scale;
        let sz = p.z * self.scale;
        let cos = libm::cosf(self.rotation_y);
        let sin = libm::sinf(self.rotation_y);
        Vec3::new(
            sx * cos - sz * sin + self.position.x,
            sy + self.position.y,
            sx * sin + sz * cos + self.position.z,
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Material — visual properties
// ─────────────────────────────────────────────────────────────────────────────

/// Material đơn giản — color + roughness.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material {
    /// RGB (0.0..1.0)
    pub r: f32,
    pub g: f32,
    pub b: f32,
    /// Alpha (0.0 = transparent, 1.0 = opaque)
    pub alpha: f32,
    /// Roughness (0.0 = mirror, 1.0 = matte)
    pub roughness: f32,
    /// Emission intensity (0.0 = none, >0 = glowing)
    pub emission: f32,
}

impl Material {
    pub const DEFAULT: Self = Self {
        r: 0.8,
        g: 0.8,
        b: 0.8,
        alpha: 1.0,
        roughness: 0.5,
        emission: 0.0,
    };
    pub const RED: Self = Self {
        r: 0.9,
        g: 0.2,
        b: 0.2,
        alpha: 1.0,
        roughness: 0.4,
        emission: 0.0,
    };
    pub const GREEN: Self = Self {
        r: 0.2,
        g: 0.9,
        b: 0.2,
        alpha: 1.0,
        roughness: 0.4,
        emission: 0.0,
    };
    pub const BLUE: Self = Self {
        r: 0.2,
        g: 0.4,
        b: 0.9,
        alpha: 1.0,
        roughness: 0.3,
        emission: 0.0,
    };
    pub const GOLD: Self = Self {
        r: 0.9,
        g: 0.7,
        b: 0.2,
        alpha: 1.0,
        roughness: 0.2,
        emission: 0.1,
    };

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self {
            r,
            g,
            b,
            alpha: 1.0,
            roughness: 0.5,
            emission: 0.0,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::DEFAULT
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SceneNode — 1 vật thể trong scene
// ─────────────────────────────────────────────────────────────────────────────

/// Unique node ID trong scene.
pub type NodeId = u32;

/// 1 vật thể trong scene graph.
#[derive(Debug, Clone)]
pub struct SceneNode {
    /// Unique ID
    pub id: NodeId,
    /// SDF primitive type
    pub kind: SdfKind,
    /// SDF parameters (radius, height, etc.)
    pub params: SdfParams,
    /// World transform
    pub transform: Transform,
    /// Visual material
    pub material: Material,
    /// Parent node ID (0 = root)
    pub parent: NodeId,
    /// Chain hash (từ MolecularChain nếu có)
    pub chain_hash: u64,
    /// Active/visible
    pub visible: bool,
    /// Label (debug)
    pub label: String,
}

impl SceneNode {
    /// Tính SDF distance tại world point.
    pub fn sdf_at(&self, world_p: Vec3) -> f32 {
        let local = self.transform.world_to_local(world_p);
        sdf(self.kind, local, &self.params) * self.transform.scale
    }

    /// Tính world-space normal tại world point.
    pub fn normal_at(&self, world_p: Vec3) -> Vec3 {
        let local = self.transform.world_to_local(world_p);
        let local_n = gradient(self.kind, local, &self.params);
        // Rotate normal back to world space (scale doesn't affect normal direction)
        let cos = libm::cosf(self.transform.rotation_y);
        let sin = libm::sinf(self.transform.rotation_y);
        Vec3::new(
            local_n.x * cos - local_n.z * sin,
            local_n.y,
            local_n.x * sin + local_n.z * cos,
        )
    }

    /// Bounding sphere radius (rough estimate).
    pub fn bounding_radius(&self) -> f32 {
        let base = self.params.r.max(self.params.b.len()).max(self.params.h);
        base * self.transform.scale
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SceneGraph — cây chứa tất cả SceneNodes
// ─────────────────────────────────────────────────────────────────────────────

/// Scene graph — quản lý tất cả vật thể trong thế giới 3D.
///
/// Render pipeline:
/// 1. Collect visible nodes
/// 2. Sort front-to-back (minimize overdraw)
/// 3. For each node: SDF sample → shade → output pixel
pub struct SceneGraph {
    /// Tất cả nodes
    nodes: Vec<SceneNode>,
    /// Next ID
    next_id: NodeId,
    /// Camera position (for sorting)
    pub camera_pos: Vec3,
    /// Light direction (normalized)
    pub light_dir: Vec3,
    /// Ambient light
    pub ambient: f32,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            next_id: 1,
            camera_pos: Vec3::new(0.0, 2.0, 5.0),
            light_dir: Vec3::new(0.577, 0.577, -0.577), // normalized (1,1,-1)
            ambient: 0.25,
        }
    }

    /// Add node → returns ID.
    pub fn add(
        &mut self,
        kind: SdfKind,
        params: SdfParams,
        transform: Transform,
        material: Material,
    ) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(SceneNode {
            id,
            kind,
            params,
            transform,
            material,
            parent: 0,
            chain_hash: 0,
            visible: true,
            label: String::new(),
        });
        id
    }

    /// Add node with chain hash (from MolecularChain).
    pub fn add_from_chain(
        &mut self,
        kind: SdfKind,
        params: SdfParams,
        transform: Transform,
        material: Material,
        chain_hash: u64,
        label: String,
    ) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(SceneNode {
            id,
            kind,
            params,
            transform,
            material,
            parent: 0,
            chain_hash,
            visible: true,
            label,
        });
        id
    }

    /// Set parent (hierarchy).
    pub fn set_parent(&mut self, child: NodeId, parent: NodeId) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == child) {
            node.parent = parent;
        }
    }

    /// Get node by ID.
    pub fn get(&self, id: NodeId) -> Option<&SceneNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Remove node.
    pub fn remove(&mut self, id: NodeId) {
        self.nodes.retain(|n| n.id != id);
    }

    /// Total nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Empty?
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Visible nodes sorted front-to-back for rendering.
    pub fn render_list(&self) -> Vec<&SceneNode> {
        let mut visible: Vec<&SceneNode> = self.nodes.iter().filter(|n| n.visible).collect();

        // Sort by distance to camera (front-to-back → minimize overdraw)
        let cam = self.camera_pos;
        visible.sort_by(|a, b| {
            let da = distance_sq(cam, a.transform.position);
            let db = distance_sq(cam, b.transform.position);
            da.partial_cmp(&db).unwrap_or(core::cmp::Ordering::Equal)
        });

        visible
    }

    /// Find closest node to a ray from camera (ray casting).
    pub fn ray_hit(&self, ray_origin: Vec3, ray_dir: Vec3, max_dist: f32) -> Option<(NodeId, f32)> {
        let mut closest: Option<(NodeId, f32)> = None;

        // Sphere-march along ray
        let mut t = 0.0f32;
        let steps = 64u32;

        for _ in 0..steps {
            if t > max_dist {
                break;
            }

            let p = Vec3::new(
                ray_origin.x + ray_dir.x * t,
                ray_origin.y + ray_dir.y * t,
                ray_origin.z + ray_dir.z * t,
            );

            // Find closest SDF
            let mut min_d = f32::MAX;
            let mut min_id = 0u32;

            for node in &self.nodes {
                if !node.visible {
                    continue;
                }
                let d = node.sdf_at(p);
                if d < min_d {
                    min_d = d;
                    min_id = node.id;
                }
            }

            if min_d < 0.001 {
                closest = Some((min_id, t));
                break;
            }

            t += min_d.max(0.01); // step by SDF distance (sphere tracing)
        }

        closest
    }

    // ── Export to JSON (for browser/WebGL) ─────────────────────────────────

    /// Export scene to JSON string (for browser rendering).
    ///
    /// Format:
    /// ```json
    /// { "nodes": [...], "camera": {...}, "light": {...} }
    /// ```
    pub fn to_json(&self) -> String {
        let mut json = String::from("{\"nodes\":[");

        for (i, node) in self.nodes.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&node_to_json(node));
        }

        json.push_str(&format!(
            "],\"camera\":{{\"x\":{:.3},\"y\":{:.3},\"z\":{:.3}}},\
             \"light\":{{\"x\":{:.3},\"y\":{:.3},\"z\":{:.3}}},\
             \"ambient\":{:.3}}}",
            self.camera_pos.x,
            self.camera_pos.y,
            self.camera_pos.z,
            self.light_dir.x,
            self.light_dir.y,
            self.light_dir.z,
            self.ambient,
        ));

        json
    }

    // ── Export to binary (compact, for ISL transport) ──────────────────────

    /// Export scene to binary bytes (compact, for ISL).
    ///
    /// Format per node: [kind:1][px:4][py:4][pz:4][scale:4][rot:4][r:4][g:4][b:4] = 33 bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"SCNE"); // magic
        buf.extend_from_slice(&(self.nodes.len() as u32).to_be_bytes());

        for node in &self.nodes {
            buf.push(node.kind.as_byte());
            buf.extend_from_slice(&node.transform.position.x.to_be_bytes());
            buf.extend_from_slice(&node.transform.position.y.to_be_bytes());
            buf.extend_from_slice(&node.transform.position.z.to_be_bytes());
            buf.extend_from_slice(&node.transform.scale.to_be_bytes());
            buf.extend_from_slice(&node.transform.rotation_y.to_be_bytes());
            buf.push((node.material.r * 255.0) as u8);
            buf.push((node.material.g * 255.0) as u8);
            buf.push((node.material.b * 255.0) as u8);
        }

        buf
    }

    /// Summary.
    pub fn summary(&self) -> String {
        format!(
            "Scene: {} nodes ({} visible) | camera ({:.1},{:.1},{:.1})",
            self.nodes.len(),
            self.nodes.iter().filter(|n| n.visible).count(),
            self.camera_pos.x,
            self.camera_pos.y,
            self.camera_pos.z,
        )
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn distance_sq(a: Vec3, b: Vec3) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    dx * dx + dy * dy + dz * dz
}

fn node_to_json(node: &SceneNode) -> String {
    format!(
        "{{\"id\":{},\"kind\":{},\"pos\":[{:.3},{:.3},{:.3}],\
         \"scale\":{:.3},\"rot\":{:.3},\
         \"color\":[{:.2},{:.2},{:.2}],\"alpha\":{:.2},\
         \"visible\":{},\"hash\":\"0x{:016X}\"}}",
        node.id,
        node.kind.as_byte(),
        node.transform.position.x,
        node.transform.position.y,
        node.transform.position.z,
        node.transform.scale,
        node.transform.rotation_y,
        node.material.r,
        node.material.g,
        node.material.b,
        node.material.alpha,
        node.visible,
        node.chain_hash,
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_identity() {
        let t = Transform::IDENTITY;
        let p = Vec3::new(1.0, 2.0, 3.0);
        let local = t.world_to_local(p);
        assert!((local.x - 1.0).abs() < 0.001);
        assert!((local.y - 2.0).abs() < 0.001);
    }

    #[test]
    fn transform_translate() {
        let t = Transform::at(10.0, 0.0, 0.0);
        let p = Vec3::new(12.0, 0.0, 0.0);
        let local = t.world_to_local(p);
        assert!((local.x - 2.0).abs() < 0.001, "Translated: {}", local.x);
    }

    #[test]
    fn transform_scale() {
        let t = Transform::at(0.0, 0.0, 0.0).with_scale(2.0);
        let p = Vec3::new(4.0, 0.0, 0.0);
        let local = t.world_to_local(p);
        assert!((local.x - 2.0).abs() < 0.001, "Scaled: {}", local.x);
    }

    #[test]
    fn transform_roundtrip() {
        let t = Transform::at(1.0, 2.0, 3.0)
            .with_scale(1.5)
            .with_rotation(0.5);
        let p = Vec3::new(5.0, 3.0, 1.0);
        let local = t.world_to_local(p);
        let world = t.local_to_world(local);
        assert!(
            (world.x - p.x).abs() < 0.01,
            "Roundtrip X: {} vs {}",
            world.x,
            p.x
        );
        assert!((world.y - p.y).abs() < 0.01, "Roundtrip Y");
        assert!((world.z - p.z).abs() < 0.01, "Roundtrip Z");
    }

    #[test]
    fn scene_add_and_get() {
        let mut scene = SceneGraph::new();
        let id = scene.add(
            SdfKind::Sphere,
            SdfParams::sphere(1.0),
            Transform::IDENTITY,
            Material::RED,
        );
        assert_eq!(scene.len(), 1);
        assert!(scene.get(id).is_some());
        assert_eq!(scene.get(id).unwrap().kind, SdfKind::Sphere);
    }

    #[test]
    fn scene_remove() {
        let mut scene = SceneGraph::new();
        let id = scene.add(
            SdfKind::Box,
            SdfParams::sphere(1.0),
            Transform::IDENTITY,
            Material::DEFAULT,
        );
        scene.remove(id);
        assert_eq!(scene.len(), 0);
    }

    #[test]
    fn scene_render_list_sorted() {
        let mut scene = SceneGraph::new();
        scene.camera_pos = Vec3::new(0.0, 0.0, 10.0);
        // Far node
        scene.add(
            SdfKind::Sphere,
            SdfParams::sphere(1.0),
            Transform::at(0.0, 0.0, -5.0),
            Material::RED,
        );
        // Near node
        scene.add(
            SdfKind::Box,
            SdfParams::sphere(1.0),
            Transform::at(0.0, 0.0, 8.0),
            Material::BLUE,
        );

        let list = scene.render_list();
        assert_eq!(list.len(), 2);
        // Near should be first (front-to-back)
        let d0 = distance_sq(scene.camera_pos, list[0].transform.position);
        let d1 = distance_sq(scene.camera_pos, list[1].transform.position);
        assert!(d0 <= d1, "Front-to-back: {} <= {}", d0, d1);
    }

    #[test]
    fn scene_node_sdf() {
        let mut scene = SceneGraph::new();
        let id = scene.add(
            SdfKind::Sphere,
            SdfParams::sphere(1.0),
            Transform::at(0.0, 0.0, 0.0),
            Material::DEFAULT,
        );
        let node = scene.get(id).unwrap();
        // Point on surface → sdf ≈ 0
        let d = node.sdf_at(Vec3::new(1.0, 0.0, 0.0));
        assert!(d.abs() < 0.1, "Surface point: sdf={}", d);
        // Point inside → sdf < 0
        let d_in = node.sdf_at(Vec3::ZERO);
        assert!(d_in < 0.0, "Inside: sdf={}", d_in);
    }

    #[test]
    fn scene_to_json() {
        let mut scene = SceneGraph::new();
        scene.add(
            SdfKind::Sphere,
            SdfParams::sphere(1.0),
            Transform::IDENTITY,
            Material::RED,
        );
        let json = scene.to_json();
        assert!(json.contains("\"nodes\""), "JSON has nodes");
        assert!(json.contains("\"camera\""), "JSON has camera");
        assert!(json.contains("\"kind\":1"), "Sphere kind=1");
    }

    #[test]
    fn scene_to_bytes() {
        let mut scene = SceneGraph::new();
        scene.add(
            SdfKind::Sphere,
            SdfParams::sphere(1.0),
            Transform::IDENTITY,
            Material::DEFAULT,
        );
        let bytes = scene.to_bytes();
        assert_eq!(&bytes[0..4], b"SCNE", "Magic");
        assert_eq!(
            u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
            1,
            "1 node"
        );
    }

    #[test]
    fn scene_ray_hit_sphere() {
        let mut scene = SceneGraph::new();
        scene.add(
            SdfKind::Sphere,
            SdfParams::sphere(1.0),
            Transform::at(0.0, 0.0, 0.0),
            Material::RED,
        );

        // Ray from (0,0,5) pointing towards origin
        let hit = scene.ray_hit(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0), 10.0);
        assert!(hit.is_some(), "Should hit sphere");
        let (id, t) = hit.unwrap();
        assert_eq!(id, 1, "Hit node 1");
        assert!(t > 3.0 && t < 5.0, "Hit distance ~4.0: {}", t);
    }

    #[test]
    fn scene_ray_miss() {
        let mut scene = SceneGraph::new();
        scene.add(
            SdfKind::Sphere,
            SdfParams::sphere(1.0),
            Transform::at(0.0, 0.0, 0.0),
            Material::RED,
        );

        // Ray going away from sphere
        let hit = scene.ray_hit(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, 1.0), 10.0);
        assert!(hit.is_none(), "Should miss");
    }

    #[test]
    fn scene_hierarchy() {
        let mut scene = SceneGraph::new();
        let parent = scene.add(
            SdfKind::Sphere,
            SdfParams::sphere(1.0),
            Transform::IDENTITY,
            Material::DEFAULT,
        );
        let child = scene.add(
            SdfKind::Box,
            SdfParams::sphere(0.5),
            Transform::at(2.0, 0.0, 0.0),
            Material::RED,
        );
        scene.set_parent(child, parent);
        assert_eq!(scene.get(child).unwrap().parent, parent);
    }

    #[test]
    fn scene_summary() {
        let mut scene = SceneGraph::new();
        scene.add(
            SdfKind::Sphere,
            SdfParams::sphere(1.0),
            Transform::IDENTITY,
            Material::DEFAULT,
        );
        let s = scene.summary();
        assert!(s.contains("1 nodes"), "{}", s);
    }

    #[test]
    fn material_defaults() {
        assert_eq!(Material::RED.r, 0.9);
        assert_eq!(Material::BLUE.b, 0.9);
        assert_eq!(Material::DEFAULT.roughness, 0.5);
    }
}
