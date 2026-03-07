// internal/perception/vision.go
// VisionSkill — nhận diện vật thể qua SDF + Fibonacci
// Không pixel matching. Phân tích cấu trúc SDF + tỉ lệ vàng.

package perception

import (
	"math"

	"github.com/goldlotus1810/HomeOS/internal/gene"
	"github.com/goldlotus1810/HomeOS/internal/isl"
)

const phi = 1.6180339887

// VisionResult kết quả nhận diện
type VisionResult struct {
	Addr       isl.Address
	Confidence float64
	Label      string
	FibScore   float64 // tỉ lệ Fibonacci match
	SDF        float64 // SDF distance tại điểm quan sát
}

// VisionSkill phân tích cảnh qua ray marching + Fibonacci
type VisionSkill struct {
	Scene    func(gene.Vec3) float64 // SDF của toàn bộ scene
	CamPos   gene.Vec3
	CamDir   gene.Vec3
	ViewDist float64 // khoảng cách quan sát tối đa
}

func NewVisionSkill(scene func(gene.Vec3) float64, cam, dir gene.Vec3) *VisionSkill {
	return &VisionSkill{
		Scene:    scene,
		CamPos:   cam,
		CamDir:   dir.Norm(),
		ViewDist: 200.0,
	}
}

// Observe bắn ray từ camera, trả về hit info + Fibonacci analysis
func (v *VisionSkill) Observe() (*gene.RayHit, float64) {
	hit := gene.RayMarch(v.CamPos, v.CamDir, v.Scene)
	if !hit.Hit {
		return &hit, 0
	}
	fib := v.analyzeFibonacci(hit.Pos, hit.Normal)
	return &hit, fib
}

// analyzeFibonacci tính tỉ lệ vàng của bề mặt tại điểm p
// Lấy mẫu SDF theo nhiều hướng → so sánh tỉ lệ khoảng cách
func (v *VisionSkill) analyzeFibonacci(p, normal gene.Vec3) float64 {
	// Tạo basis vuông góc với normal
	tangent := perp(normal)
	bitangent := normal.Cross(tangent).Norm()

	scores := make([]float64, 0, 8)

	// Lấy mẫu 8 hướng trên bề mặt
	for i := 0; i < 8; i++ {
		angle := float64(i) / 8 * 2 * math.Pi
		dir := tangent.Scale(math.Cos(angle)).Add(bitangent.Scale(math.Sin(angle)))

		d1 := math.Abs(v.Scene(p.Add(dir.Scale(0.1))))
		d2 := math.Abs(v.Scene(p.Add(dir.Scale(0.1*phi))))

		if d2 > 1e-9 {
			ratio := d1 / d2
			score := isFibRatio(ratio)
			scores = append(scores, score)
		}
	}

	if len(scores) == 0 {
		return 0
	}
	sum := 0.0
	for _, s := range scores {
		sum += s
	}
	return sum / float64(len(scores))
}

// isFibRatio kiểm tra một tỉ lệ có gần φ hoặc 1/φ không
func isFibRatio(r float64) float64 {
	if r <= 0 {
		return 0
	}
	// Kiểm tra r ≈ φ, 1/φ, φ², 1/φ²
	targets := []float64{phi, 1 / phi, phi * phi, 1 / (phi * phi)}
	best := 0.0
	for _, t := range targets {
		diff := math.Abs(r-t) / t
		score := math.Max(0, 1-diff/0.15) // tolerance 15%
		if score > best {
			best = score
		}
	}
	return best
}

// ScanScene bắn nhiều rays, tổng hợp kết quả
// Dùng để nhận dạng vật thể trong scene rộng hơn
func (v *VisionSkill) ScanScene(rays int) []VisionResult {
	results := make([]VisionResult, 0, rays)

	for i := 0; i < rays; i++ {
		// Jitter direction nhẹ
		angle := float64(i) / float64(rays) * 2 * math.Pi
		jitter := 0.05
		jDir := gene.Vec3{
			X: v.CamDir.X + math.Cos(angle)*jitter,
			Y: v.CamDir.Y + math.Sin(angle*0.7)*jitter*0.5,
			Z: v.CamDir.Z + math.Sin(angle)*jitter,
		}.Norm()

		hit := gene.RayMarch(v.CamPos, jDir, v.Scene)
		if !hit.Hit {
			continue
		}

		fib := v.analyzeFibonacci(hit.Pos, hit.Normal)
		results = append(results, VisionResult{
			FibScore:   fib,
			SDF:        hit.T,
			Confidence: fib * (1.0 - hit.T/v.ViewDist),
		})
	}
	return results
}

// CanHandle — Skill interface
func (v *VisionSkill) CanHandle(msg *isl.ISLMessage) bool {
	return msg.MsgType == isl.MsgQuery && msg.PrimaryAddr.Group == 'V'
}

// Name — Skill interface
func (v *VisionSkill) Name() string { return "vision" }

// perp tìm vector vuông góc với v
func perp(v gene.Vec3) gene.Vec3 {
	if math.Abs(v.X) < 0.9 {
		return gene.Vec3{Y: v.Z, Z: -v.Y}.Norm()
	}
	return gene.Vec3{X: -v.Z, Z: v.X}.Norm()
}
