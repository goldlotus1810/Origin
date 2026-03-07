// internal/gene/alphabet/strokes.go
//
// PRIMITIVE STROKES — vật lý cơ bản của chữ viết
// ================================================
// Mọi chữ cái trong mọi ngôn ngữ = tổ hợp ~10 stroke nguyên tố.
// Không lưu pixel. Không lưu path. Lưu CÔNG THỨC SDF.
// Từ công thức sinh ra hình dạng ở mọi kích thước, mọi độ phân giải.
//
// Hệ tọa độ:
//   Ký tự nằm trong [-0.5, +0.5] × [-0.5, +0.5]
//   (0,0) = tâm   Y+ = lên   X+ = phải

package alphabet

import (
	"math"

	"github.com/goldlotus1810/HomeOS/internal/gene"
)

const (
	T  = 0.055 // thickness chuẩn
	TH = 0.075 // nét đậm
	TL = 0.035 // nét mảnh

	YTop  =  0.44
	YCap  =  0.38
	YXH   =  0.18
	YMid  =  0.00
	YBase = -0.38
	YBot  = -0.44

	XLeft   = -0.28
	XMidL   = -0.14
	XCenter =  0.00
	XMidR   =  0.14
	XRight  =  0.28
)

func p3(x, y float64) gene.Vec3 { return gene.Vec3{X: x, Y: y, Z: 0} }

// ── capsule shorthand ─────────────────────────────────────────
func cap2(ax, ay, bx, by, r float64) func(gene.Vec3) float64 {
	a, b := p3(ax, ay), p3(bx, by)
	return func(p gene.Vec3) float64 { return gene.SDFCapsule(p, a, b, r) }
}

// ── arc: xấp xỉ cung tròn bằng capsule chain ─────────────────
func arcSDF(cx, cy, r, thick, a0, a1 float64) func(gene.Vec3) float64 {
	const steps = 24
	return func(p gene.Vec3) float64 {
		d := math.MaxFloat64
		da := (a1 - a0) / steps
		for i := 0; i < steps; i++ {
			aa := a0 + float64(i)*da
			ab := aa + da
			pa := p3(cx+r*math.Cos(aa), cy+r*math.Sin(aa))
			pb := p3(cx+r*math.Cos(ab), cy+r*math.Sin(ab))
			if s := gene.SDFCapsule(p, pa, pb, thick/2); s < d {
				d = s
			}
		}
		return d
	}
}

// ── union helpers ─────────────────────────────────────────────
func union(fs ...func(gene.Vec3) float64) func(gene.Vec3) float64 {
	return func(p gene.Vec3) float64 {
		d := math.MaxFloat64
		for _, f := range fs {
			if s := f(p); s < d { d = s }
		}
		return d
	}
}

func sUnion(k float64, fs ...func(gene.Vec3) float64) func(gene.Vec3) float64 {
	return func(p gene.Vec3) float64 {
		d := math.MaxFloat64
		for _, f := range fs {
			s := f(p)
			if d == math.MaxFloat64 { d = s; continue }
			h := math.Max(k-math.Abs(d-s), 0) / k
			d = math.Min(d, s) - h*h*k*0.25
		}
		return d
	}
}

// ══════════════════════════════════════════════════════════════
// 10 PRIMITIVE STROKES — bảng nguyên tố của chữ viết
// ══════════════════════════════════════════════════════════════

// SV — Vertical stroke
func SV(x, y0, y1 float64) func(gene.Vec3) float64 { return cap2(x, y0, x, y1, T) }

// SH — Horizontal stroke
func SH(x0, x1, y float64) func(gene.Vec3) float64 { return cap2(x0, y, x1, y, T) }

// SD — Diagonal stroke
func SD(x0, y0, x1, y1 float64) func(gene.Vec3) float64 { return cap2(x0, y0, x1, y1, T) }

// SArc — Arc stroke (a0→a1 radian, 0=right, Pi/2=up)
func SArc(cx, cy, r, a0, a1 float64) func(gene.Vec3) float64 {
	return arcSDF(cx, cy, r, T, a0, a1)
}

// SCircle — full circle stroke
func SCircle(cx, cy, r float64) func(gene.Vec3) float64 {
	return arcSDF(cx, cy, r, T, 0, 2*math.Pi)
}

// SDot — filled dot
func SDot(cx, cy, r float64) func(gene.Vec3) float64 {
	c := p3(cx, cy)
	return func(p gene.Vec3) float64 { return gene.SDFSphere(p, c, r) }
}

// SCurveR — right-side curve (bụng D, P, B, R)
// nét từ (x,ytop) cong ra phải rồi trở về (x,ybot)
func SCurveR(x, ytop, ybot float64) func(gene.Vec3) float64 {
	cy := (ytop + ybot) / 2
	r  := math.Abs(ytop-ybot) * 0.28
	return arcSDF(x, cy, r, T, -math.Pi/2, math.Pi/2)
}

// SCurveL — left-side curve (C, G, (, [)
func SCurveL(x, ytop, ybot float64) func(gene.Vec3) float64 {
	cy := (ytop + ybot) / 2
	r  := math.Abs(ytop-ybot) * 0.28
	return arcSDF(x, cy, r, T, math.Pi/2, 3*math.Pi/2)
}

// SHook — nét móc xuống-trái (J, j, hook f)
func SHook(x, ytop, ybot float64) func(gene.Vec3) float64 {
	r := 0.09
	return union(
		cap2(x, ytop, x, ybot+r, T),
		arcSDF(x-r, ybot+r, r, T, 0, -math.Pi/2),
	)
}

// ══════════════════════════════════════════════════════════════
// OlangChar — đơn vị tri thức cơ bản
// ══════════════════════════════════════════════════════════════

// OlangChar = mã hóa đầy đủ một ký tự Unicode trong Olang
// Không lưu data — lưu CÔNG THỨC sinh ra mọi thuộc tính
type OlangChar struct {
	// Identity
	Codepoint uint32
	Glyph     string // UTF-8
	Name      string // tên Unicode chính thức
	Script    string // latin/greek/cyrillic/arabic/indic/cjk/math/ipa
	Category  string // letter/digit/math/punct/symbol

	// Hình dạng — SDF
	// < 0: bên trong   = 0: viền   > 0: bên ngoài
	SDF func(gene.Vec3) float64

	// DNA string — Olang notation của SDF
	// Lưu vào SilkTree, agent đọc được mà không cần Go runtime
	// Ví dụ chữ A: "∪(⌀(-.25,.44,0,-.44,0), ⌀(.25,.44,0,-.44,0), □(0,.05,.36,.04))"
	DNA string

	// Âm thanh — IPA
	Phonemes []string // ["/eɪ/", "/æ/", "/ɑː/"]

	// Nét vẽ — thứ tự stroke khi viết ký tự
	// Agent dùng để nhận dạng chữ viết tay và tạo animation
	StrokeOrder []*gene.Spline

	// ISL Address
	ISLLayer byte
	ISLGroup byte
	ISLType  byte
	ISLID    byte

	// Quan hệ ngữ nghĩa (silk edges)
	SameSound   []uint32 // A≡α≡А≡अ≡あ — cùng âm /a/
	SameShape   []uint32 // O≡0≡○ — cùng hình
	SameMeaning []uint32 // +≡∪≡⊕ — cùng nghĩa
	DerivedFrom uint32   // A←𐤀 (Phoenician Aleph)
	LowercaseOf uint32   // A→a
	UppercaseOf uint32   // a→A
	MirrorOf    uint32   // b↔d
}

func (c *OlangChar) Eval(p gene.Vec3) float64 {
	if c.SDF == nil { return 1 }
	return c.SDF(p)
}

func (c *OlangChar) IsInside(x, y float64) bool {
	return c.Eval(p3(x, y)) < 0
}

// StrokeSpline — tạo Spline từ danh sách điểm (x,y)
func StrokeSpline(pts [][2]float64) *gene.Spline {
	sp := &gene.Spline{}
	n := len(pts)
	for i, pt := range pts {
		t := 0.0
		if n > 1 { t = float64(i) / float64(n-1) }
		sp.Keys = append(sp.Keys, gene.Keyframe{
			T: t,
			V: gene.Vec3{X: pt[0], Y: pt[1]},
		})
	}
	return sp
}
