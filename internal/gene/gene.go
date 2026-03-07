// internal/gene/gene.go
// Gene interface — đơn vị sinh học của HomeOS
// QT4: "Agent = tập hợp Skill của nó"
// Tương tự: Gene = công thức SDF của một thực thể

package gene

// Gene là interface mà mọi thực thể SDF phải implement
// Một Gene = một công thức toán học, không phải dữ liệu hình dạng
type Gene interface {
	// SDF trả về khoảng cách có dấu từ p đến bề mặt Gene
	SDF(p Vec3) float64

	// Normal trả về pháp tuyến bề mặt tại p
	// Mặc định: SurfaceNormal(p, g.SDF, 0.001)
	Normal(p Vec3) Vec3

	// Animate cập nhật trạng thái theo thời gian t (world time [0..24])
	Animate(t float64)

	// DNA trả về chuỗi UTF-32 tái tạo Gene này
	// Ví dụ: "∪(⌀(10,0,5,0.6),∪(●c1,●c2,●c3,0.3),0.3)"
	DNA() string

	// SimpleSDF phiên bản đơn giản cho LOD khi node xa camera
	SimpleSDF(p Vec3) float64
}

// ─────────────────────────────────────────────────────────────────
// SphereGene — Gene đơn giản nhất: một khối cầu
// Dùng cho nodes trong ISDF renderer
// ─────────────────────────────────────────────────────────────────

// SphereGene biểu diễn một khối cầu SDF với animation spline
type SphereGene struct {
	Center Vec3    // vị trí base (không đổi)
	Radius float64 // bán kính
	Motion *Spline // orbit animation
	pos    Vec3    // vị trí hiện tại (sau Animate)
}

// NewSphereGene tạo SphereGene với orbit spline
func NewSphereGene(center Vec3, r float64, orbitAmp, phase float64) *SphereGene {
	g := &SphereGene{
		Center: center,
		Radius: r,
		pos:    center,
	}
	if orbitAmp > 1e-9 {
		g.Motion = NewOrbitSpline(center, orbitAmp, phase)
	}
	return g
}

func (g *SphereGene) SDF(p Vec3) float64 {
	return SDFSphere(p, g.pos, g.Radius)
}

func (g *SphereGene) Normal(p Vec3) Vec3 {
	return SurfaceNormal(p, g.SDF, g.Radius*0.01)
}

func (g *SphereGene) Animate(t float64) {
	if g.Motion != nil {
		// t world time [0..24] → spline phase [0..1)
		g.pos = g.Motion.Eval(t / 24.0)
	}
}

func (g *SphereGene) DNA() string {
	return "●" // chuỗi Olang đơn giản nhất
}

func (g *SphereGene) SimpleSDF(p Vec3) float64 {
	// Dùng vị trí base thay vì animated position — nhanh hơn cho LOD
	return SDFSphere(p, g.Center, g.Radius)
}

// Pos trả về vị trí hiện tại (sau Animate)
func (g *SphereGene) Pos() Vec3 { return g.pos }

// ─────────────────────────────────────────────────────────────────
// TreeGene — ∪(⌀trunk, ∪(●,●,●)) từ HomeOS-DNA.html
// ─────────────────────────────────────────────────────────────────

// TreeGene biểu diễn cây = thân capsule + 3 quả cầu tán lá
type TreeGene struct {
	X, Y, Z float64
	Age     float64 // [0..1]
	k       float64 // blend factor
}

func NewTreeGene(x, y, z, age float64) *TreeGene {
	return &TreeGene{X: x, Y: y, Z: z, Age: age, k: 0.3}
}

func (g *TreeGene) SDF(p Vec3) float64 {
	s := 0.3 + g.Age*0.7
	base := Vec3{g.X, g.Y, g.Z}
	top := Vec3{g.X, g.Y + 0.6*s, g.Z}

	trunk := SDFCapsule(p, base, top, 0.08*s)

	c0 := SDFSphere(p, top, 0.35*s)
	c1 := SDFSphere(p, Vec3{g.X + 0.18*s, g.Y + 0.6*s - 0.1*s, g.Z + 0.12*s}, 0.28*s)
	c2 := SDFSphere(p, Vec3{g.X - 0.15*s, g.Y + 0.6*s - 0.15*s, g.Z - 0.1*s}, 0.22*s)

	crown := SmoothUnion(c0, SmoothUnion(c1, c2, g.k), g.k)
	return SmoothUnion(trunk, crown, g.k*0.5)
}

func (g *TreeGene) Normal(p Vec3) Vec3 {
	return SurfaceNormal(p, g.SDF, 0.01)
}

func (g *TreeGene) Animate(t float64) {
	// Sinh trưởng: age tăng rất chậm
	g.Age = clamp01(g.Age + 0.00008*t)
}

func (g *TreeGene) DNA() string {
	return "∪(⌀,∪(●,●,●,k:0.3),k:0.3)"
}

func (g *TreeGene) SimpleSDF(p Vec3) float64 {
	s := 0.3 + g.Age*0.7
	return SDFSphere(p, Vec3{g.X, g.Y + 0.3*s, g.Z}, 0.5*s)
}

func clamp01(v float64) float64 {
	if v < 0 {
		return 0
	}
	if v > 1 {
		return 1
	}
	return v
}
