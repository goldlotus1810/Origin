// internal/gene/vec3.go
// Vec3 — vector 3D nền tảng của toàn bộ HomeOS
// Mọi vị trí, hướng, pháp tuyến, gradient đều là Vec3

package gene

import "math"

// Vec3 là vector 3 chiều float64
type Vec3 struct{ X, Y, Z float64 }

// Các hằng số hay dùng
var (
	V0      = Vec3{0, 0, 0}
	VUp     = Vec3{0, 1, 0}
	VRight  = Vec3{1, 0, 0}
	VFwd    = Vec3{0, 0, 1}
)

func (a Vec3) Add(b Vec3) Vec3  { return Vec3{a.X + b.X, a.Y + b.Y, a.Z + b.Z} }
func (a Vec3) Sub(b Vec3) Vec3  { return Vec3{a.X - b.X, a.Y - b.Y, a.Z - b.Z} }
func (a Vec3) Scale(f float64) Vec3 { return Vec3{a.X * f, a.Y * f, a.Z * f} }
func (a Vec3) Neg() Vec3        { return Vec3{-a.X, -a.Y, -a.Z} }

func (a Vec3) Dot(b Vec3) float64 {
	return a.X*b.X + a.Y*b.Y + a.Z*b.Z
}

func (a Vec3) Cross(b Vec3) Vec3 {
	return Vec3{
		a.Y*b.Z - a.Z*b.Y,
		a.Z*b.X - a.X*b.Z,
		a.X*b.Y - a.Y*b.X,
	}
}

func (a Vec3) Len() float64 {
	return math.Sqrt(a.X*a.X + a.Y*a.Y + a.Z*a.Z)
}

func (a Vec3) Len2() float64 { return a.X*a.X + a.Y*a.Y + a.Z*a.Z }

func (a Vec3) Norm() Vec3 {
	l := a.Len()
	if l < 1e-12 {
		return VUp // fallback: trả về hướng Y
	}
	return a.Scale(1 / l)
}

// Lerp nội suy tuyến tính: a*(1-t) + b*t
func (a Vec3) Lerp(b Vec3, t float64) Vec3 {
	return Vec3{
		a.X + (b.X-a.X)*t,
		a.Y + (b.Y-a.Y)*t,
		a.Z + (b.Z-a.Z)*t,
	}
}

// Abs giá trị tuyệt đối từng thành phần
func (a Vec3) Abs() Vec3 { return Vec3{math.Abs(a.X), math.Abs(a.Y), math.Abs(a.Z)} }

// MaxComp trả về thành phần lớn nhất
func (a Vec3) MaxComp() float64 { return math.Max(a.X, math.Max(a.Y, a.Z)) }

// MinComp trả về thành phần nhỏ nhất
func (a Vec3) MinComp() float64 { return math.Min(a.X, math.Min(a.Y, a.Z)) }

// Max thành phần per-component
func (a Vec3) Max(b Vec3) Vec3 {
	return Vec3{math.Max(a.X, b.X), math.Max(a.Y, b.Y), math.Max(a.Z, b.Z)}
}

// Clamp từng thành phần vào [lo, hi]
func (a Vec3) Clamp(lo, hi float64) Vec3 {
	return Vec3{
		math.Max(lo, math.Min(hi, a.X)),
		math.Max(lo, math.Min(hi, a.Y)),
		math.Max(lo, math.Min(hi, a.Z)),
	}
}

// Dist khoảng cách Euclid giữa 2 điểm
func Dist(a, b Vec3) float64 { return a.Sub(b).Len() }
