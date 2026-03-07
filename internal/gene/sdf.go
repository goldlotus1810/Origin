// internal/gene/sdf.go
// SDF — Signed Distance Functions
// Đây là "vật lý học" của HomeOS — mọi thứ là hàm khoảng cách
// d < 0 = bên trong · d = 0 = bề mặt · d > 0 = bên ngoài
//
// opTable mapping (để dùng trong Olang):
//   '●' → SDFSphere
//   '⌀' → SDFCapsule
//   '□' → SDFBox
//   '◌' → SDFVoid
//   '∪' → SmoothUnion
//   '∖' → SmoothSub
//   '∇' → SurfaceNormal
//   '·' → Vec3.Dot  (built-in)
//   '☀' → SunLight
//   '👁' → RayMarch

package gene

import "math"

// ─────────────────────────────────────────────────────────────────
// SDF PRIMITIVES
// ─────────────────────────────────────────────────────────────────

// SDFSphere — '●'
// Khoảng cách từ p đến mặt cầu tâm c bán kính r
func SDFSphere(p, c Vec3, r float64) float64 {
	return Dist(p, c) - r
}

// SDFCapsule — '⌀'
// Khoảng cách từ p đến viên nang từ a đến b bán kính r
func SDFCapsule(p, a, b Vec3, r float64) float64 {
	ab := b.Sub(a)
	ap := p.Sub(a)
	h := math.Max(0, math.Min(1, ap.Dot(ab)/ab.Dot(ab)))
	return Dist(p, a.Add(ab.Scale(h))) - r
}

// SDFBox — '□'
// Khoảng cách từ p đến hộp axis-aligned tâm gốc tọa độ kích thước b
func SDFBox(p, b Vec3) float64 {
	q := p.Abs().Sub(b)
	return q.Max(V0).Len() + math.Min(q.MaxComp(), 0)
}

// SDFTorus — hình xuyến tâm gốc, bán kính lớn R, bán kính nhỏ r
func SDFTorus(p Vec3, R, r float64) float64 {
	q := Vec3{math.Hypot(p.X, p.Z) - R, p.Y, 0}
	return q.Len() - r
}

// SDFVoid — '◌' — khoảng trống = tiềm năng = vô cực
func SDFVoid() float64 { return math.Inf(1) }

// ─────────────────────────────────────────────────────────────────
// SDF OPERATIONS
// ─────────────────────────────────────────────────────────────────

// SmoothUnion — '∪' — hòa tan 2 SDF, k là hệ số blend
// k = 0 → union cứng (không blend)
// k = 0.3 → blend mịn
func SmoothUnion(d1, d2, k float64) float64 {
	if k < 1e-9 {
		return math.Min(d1, d2)
	}
	h := math.Max(k-math.Abs(d1-d2), 0) / k
	return math.Min(d1, d2) - h*h*k*0.25
}

// SmoothSub — '∖' — đục lỗ d2 khỏi d1
func SmoothSub(d1, d2, k float64) float64 {
	if k < 1e-9 {
		return math.Max(d1, -d2)
	}
	h := math.Max(k-math.Abs(-d2-d1), 0) / k
	return math.Max(d1, -d2) + h*h*k*0.25
}

// HardUnion — '⊕' — union cứng không blend
func HardUnion(d1, d2 float64) float64 { return math.Min(d1, d2) }

// ─────────────────────────────────────────────────────────────────
// LIGHTING & NORMAL
// ─────────────────────────────────────────────────────────────────

// LightInfo kết quả của SunLight(t)
type LightInfo struct {
	Dir       Vec3    // hướng ánh sáng (từ bề mặt đến nguồn)
	Intensity float64 // cường độ [0..1]
	Ambient   float64 // ánh sáng môi trường
}

// SunLight — '☀' — ánh sáng mặt trời theo giờ t ∈ [0,24]
// Quỹ đạo spline: mặt trời mọc lúc 6h, lặn lúc 18h
func SunLight(t float64) LightInfo {
	a := (t - 6) / 24 * 2 * math.Pi
	lx := -math.Cos(a) * 0.6
	ly := math.Sin(a)*0.5 + 0.5
	lz := -0.4
	dir := Vec3{lx, ly, lz}.Norm()
	intensity := math.Max(0, math.Sin((t-6)/12*math.Pi))
	return LightInfo{Dir: dir, Intensity: intensity, Ambient: 0.25}
}

// SurfaceNormal — '∇' — pháp tuyến bề mặt SDF tại điểm p
// Dùng finite difference gradient: ∇f(p) = normalize([∂f/∂x, ∂f/∂y, ∂f/∂z])
func SurfaceNormal(p Vec3, sdf func(Vec3) float64, eps float64) Vec3 {
	e := eps
	nx := sdf(Vec3{p.X + e, p.Y, p.Z}) - sdf(Vec3{p.X - e, p.Y, p.Z})
	ny := sdf(Vec3{p.X, p.Y + e, p.Z}) - sdf(Vec3{p.X, p.Y - e, p.Z})
	nz := sdf(Vec3{p.X, p.Y, p.Z + e}) - sdf(Vec3{p.X, p.Y, p.Z - e})
	return Vec3{nx, ny, nz}.Norm()
}

// Shade tính ánh sáng Lambertian: diffuse = max(0, normal · lightDir)
func Shade(normal Vec3, light LightInfo) float64 {
	diffuse := math.Max(0, normal.Dot(light.Dir))
	return light.Ambient + diffuse*light.Intensity
}

// ─────────────────────────────────────────────────────────────────
// RAY MARCHING — '👁' — 256 bước sphere-trace
// ─────────────────────────────────────────────────────────────────

const RayMaxSteps = 256
const RayHitDist = 0.0005
const RayMaxDist = 500.0

// RayHit kết quả của RayMarch
type RayHit struct {
	Hit    bool
	T      float64 // khoảng cách từ origin đến điểm hit
	Pos    Vec3    // vị trí hit trong world space
	Normal Vec3    // pháp tuyến tại điểm hit
	Steps  int     // số bước đã thực hiện (debug)
}

// RayMarch — '👁' — sphere tracing 256 bước
// origin: vị trí camera, dir: hướng ray (đã normalize)
// sdf: hàm khoảng cách của toàn bộ scene
func RayMarch(origin, dir Vec3, sdf func(Vec3) float64) RayHit {
	t := 0.0
	for i := 0; i < RayMaxSteps; i++ {
		pos := origin.Add(dir.Scale(t))
		d := sdf(pos)
		if d < RayHitDist {
			// Hit — tính normal
			normal := SurfaceNormal(pos, sdf, RayHitDist*2)
			return RayHit{Hit: true, T: t, Pos: pos, Normal: normal, Steps: i}
		}
		t += d
		if t > RayMaxDist {
			break
		}
	}
	return RayHit{Hit: false, T: RayMaxDist, Steps: RayMaxSteps}
}

// ─────────────────────────────────────────────────────────────────
// FBM TERRAIN — '∫' — fractal Brownian motion
// ─────────────────────────────────────────────────────────────────

// FBM tính chiều cao terrain tại (x,z) với oct octaves
func FBM(x, z float64, oct int, seed float64) float64 {
	v, amp, freq := 0.0, 0.5, 1.0
	for i := 0; i < oct; i++ {
		nx := x*0.08*freq + seed
		nz := z*0.08*freq + seed
		ix, iz := math.Floor(nx), math.Floor(nz)
		fx, fz := nx-ix, nz-iz
		ux := fx * fx * (3 - 2*fx)
		uz := fz * fz * (3 - 2*fz)
		h := func(a, b float64) float64 {
			s := math.Sin(a*127.1+b*311.7+seed*419.3) * 43758.5453
			return s - math.Floor(s)
		}
		v += (h(ix, iz) + (h(ix+1, iz)-h(ix, iz))*ux +
			(h(ix, iz+1)-h(ix, iz))*uz +
			(h(ix+1, iz+1)-h(ix+1, iz)-h(ix, iz+1)+h(ix, iz))*ux*uz) * amp
		amp *= 0.5
		freq *= 2.1
	}
	return v
}
