// internal/gene/spline.go
// Spline — Catmull-Rom keyframe interpolation
// Mọi chuyển động trong HomeOS = Spline. Không dùng lerp thẳng.
// Mọi sensor = Spline. Không dùng []float64 raw.

package gene

import "math"

// Keyframe là một điểm kiểm soát trong spline
type Keyframe struct {
	T float64 // time trong [0,1], tăng dần
	Vec3      // position
}

// Spline là chuỗi keyframe Catmull-Rom, periodic (lặp lại)
type Spline struct {
	Keys []Keyframe
}

// Eval nội suy vị trí tại thời điểm t (periodic, t có thể > 1)
// Catmull-Rom: smooth C1, đi qua tất cả control points
func (s *Spline) Eval(t float64) Vec3 {
	n := len(s.Keys)
	if n == 0 {
		return V0
	}
	if n == 1 {
		return s.Keys[0].Vec3
	}

	// Periodic: t về [0,1)
	t = math.Mod(t, 1.0)
	if t < 0 {
		t += 1.0
	}

	// Tìm segment
	seg := t * float64(n)
	i := int(seg)
	f := seg - float64(i)

	p0 := s.Keys[(i-1+n)%n].Vec3
	p1 := s.Keys[i%n].Vec3
	p2 := s.Keys[(i+1)%n].Vec3
	p3 := s.Keys[(i+2)%n].Vec3

	return catmullRom(p0, p1, p2, p3, f)
}

// catmullRom nội suy Catmull-Rom giữa p1 và p2
func catmullRom(p0, p1, p2, p3 Vec3, t float64) Vec3 {
	t2, t3 := t*t, t*t*t
	f := func(a, b, c, d float64) float64 {
		return 0.5 * ((-a+3*b-3*c+d)*t3 + (2*a-5*b+4*c-d)*t2 + (-a+c)*t + 2*b)
	}
	return Vec3{
		f(p0.X, p1.X, p2.X, p3.X),
		f(p0.Y, p1.Y, p2.Y, p3.Y),
		f(p0.Z, p1.Z, p2.Z, p3.Z),
	}
}

// ─────────────────────────────────────────────────────────────────
// ORBIT SPLINE — tạo spline orbit tròn quanh base position
// Dùng cho animation nodes trong ISDF renderer
// ─────────────────────────────────────────────────────────────────

// NewOrbitSpline tạo spline orbit hình elipse quanh base
// amp: biên độ orbit (world units)
// phase: offset góc ban đầu (radians) — khác nhau per node
func NewOrbitSpline(base Vec3, amp, phase float64) *Spline {
	keys := make([]Keyframe, 4)
	for k := 0; k < 4; k++ {
		a := phase + float64(k)/4*2*math.Pi
		keys[k] = Keyframe{
			T: float64(k) / 4,
			Vec3: Vec3{
				base.X + amp*math.Cos(a),
				base.Y + amp*0.4*math.Sin(a*1.3+0.5), // sóng dọc nhỏ hơn
				base.Z + amp*math.Sin(a),
			},
		}
	}
	return &Spline{Keys: keys}
}

// PhaseFor tính phase duy nhất cho một node dựa trên ID
// Đảm bảo các nodes không đồng bộ nhau → trông tự nhiên
func PhaseFor(id string) float64 {
	if len(id) == 0 {
		return 0
	}
	p := 0.0
	for i, c := range id {
		p += float64(c) * (0.37 + float64(i)*0.13)
	}
	return p
}
