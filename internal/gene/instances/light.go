// internal/gene/instances/light.go
// LightGene — đèn vật lý trong HomeOS
// ActuatorLight điều khiển LightGene qua ISL

package instances

import (
	"fmt"
	"math"

	"github.com/goldlotus1810/HomeOS/internal/gene"
)

// LightGene biểu diễn một nguồn sáng
type LightGene struct {
	Pos     gene.Vec3
	Color   [3]float64 // RGB [0..1]
	On      bool
	Radius  float64    // bán kính ảnh hưởng
	spline  *gene.Spline
}

func NewLightGene(pos gene.Vec3, color [3]float64, r float64) *LightGene {
	return &LightGene{
		Pos: pos, Color: color,
		On: true, Radius: r,
	}
}

// NewSunLightGene tạo đèn mặt trời với orbital spline
func NewSunLightGene(altitude float64) *LightGene {
	// Orbit quanh trục Y ở độ cao altitude
	base := gene.Vec3{X: 0, Y: altitude, Z: 0}
	sp := gene.NewOrbitSpline(base, altitude*1.5, 0)
	lg := &LightGene{
		Pos:    base,
		Color:  [3]float64{1.0, 0.95, 0.85},
		On:     true,
		Radius: 1000,
		spline: sp,
	}
	return lg
}

func (l *LightGene) SDF(p gene.Vec3) float64 {
	if !l.On {
		return gene.SDFVoid()
	}
	return gene.SDFSphere(p, l.Pos, l.Radius*0.05)
}

func (l *LightGene) Normal(p gene.Vec3) gene.Vec3 {
	return p.Sub(l.Pos).Norm()
}

func (l *LightGene) Animate(t float64) {
	if l.spline != nil {
		// t world time [0..24] → spline [0..1)
		phase := t / 24.0
		l.Pos = l.spline.Eval(phase)
		// Màu theo thời gian: bình minh=cam, trưa=trắng, chiều=đỏ
		sun := math.Sin(t / 24.0 * math.Pi)
		l.Color = [3]float64{
			1.0,
			0.75 + sun*0.2,
			0.55 + sun*0.3,
		}
	}
}

func (l *LightGene) Toggle(on bool) { l.On = on }

func (l *LightGene) DNA() string {
	return fmt.Sprintf("☀(pos:%.1f,%.1f,%.1f)", l.Pos.X, l.Pos.Y, l.Pos.Z)
}

func (l *LightGene) SimpleSDF(p gene.Vec3) float64 {
	return l.SDF(p)
}

// Illuminate tính contribution của light tại điểm p với normal n
func (l *LightGene) Illuminate(p, normal gene.Vec3) float64 {
	if !l.On {
		return 0
	}
	dir := l.Pos.Sub(p).Norm()
	diff := math.Max(0, normal.Dot(dir))
	dist := gene.Dist(p, l.Pos)
	atten := 1.0 / (1.0 + dist*dist/(l.Radius*l.Radius))
	return diff * atten
}
