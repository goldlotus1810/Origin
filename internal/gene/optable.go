// internal/gene/optable.go
// opTable — ADN của vũ trụ HomeOS
// Mỗi rune UTF-32 = 1 quy luật vật lý
// 48 entries. Không có file nào khác.
//
// Đây là bản Go của 𝔻 trong HomeOS-DNA.html

package gene

import "math"

// OpFunc là kiểu generic cho operator functions
type OpFunc = interface{}

// OpTable ánh xạ rune → function
// Dùng để Olang compiler resolve symbols
var OpTable = map[rune]OpFunc{

	// ━━━ SDF PRIMITIVES ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
	'●': func(p, c Vec3, r float64) float64 { return SDFSphere(p, c, r) },
	'⌀': func(p, a, b Vec3, r float64) float64 { return SDFCapsule(p, a, b, r) },
	'□': func(p, b Vec3) float64 { return SDFBox(p, b) },
	'◌': func() float64 { return math.Inf(1) },

	// ━━━ SDF OPERATIONS ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
	'∪': func(d1, d2, k float64) float64 { return SmoothUnion(d1, d2, k) },
	'∖': func(d1, d2, k float64) float64 { return SmoothSub(d1, d2, k) },
	'⊕': func(d1, d2 float64) float64 { return HardUnion(d1, d2) },
	'⊗': func(x, z, s float64) (float64, float64) { // repeat
		return x - s*math.Round(x/s), z - s*math.Round(z/s)
	},
	'↻': func(x, z, a float64) (float64, float64) { // rotate Y
		return math.Cos(a)*x + math.Sin(a)*z, -math.Sin(a)*x + math.Cos(a)*z
	},

	// ━━━ PHYSICS ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
	'∇': func(p Vec3, sdf func(Vec3) float64, eps float64) Vec3 {
		return SurfaceNormal(p, sdf, eps)
	},
	'·': func(a, b Vec3) float64 { return a.Dot(b) },
	'☀': func(t float64) LightInfo { return SunLight(t) },
	'⚡': func(E, t, λ float64) float64 { return E * math.Exp(-λ*t) }, // decay
	'∆': func(old, nw float64) float64 { return nw - old },

	// ━━━ RAY / VISIBILITY ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
	'👁': func(origin, dir Vec3, sdf func(Vec3) float64) RayHit {
		return RayMarch(origin, dir, sdf)
	},

	// ━━━ TERRAIN ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
	'∫': func(x, z float64, oct int, seed float64) float64 {
		return FBM(x, z, oct, seed)
	},

	// ━━━ MATH CONSTANTS ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
	'π': func() float64 { return math.Pi },
	'∞': func() float64 { return math.Inf(1) },
	'φ': func() float64 { return (1 + math.Sqrt(5)) / 2 }, // golden ratio
	'≈': func(a, b, ε float64) bool { return math.Abs(a-b) < ε },

	// ━━━ AGGREGATE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
	'∑': func(vals []float64) float64 { // sum
		s := 0.0
		for _, v := range vals {
			s += v
		}
		return s
	},
	'∀': func(arr []interface{}, fn func(interface{}) bool) bool { // all
		for _, v := range arr {
			if !fn(v) {
				return false
			}
		}
		return true
	},
	'∃': func(arr []interface{}, fn func(interface{}) bool) bool { // exists
		for _, v := range arr {
			if fn(v) {
				return true
			}
		}
		return false
	},
	'∈': func(p Vec3, gene Gene) bool { return gene.SDF(p) < 0 }, // membership

	// ━━━ BIOLOGY ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
	'🌱': func(age, rate, dt float64) float64 { // grow
		v := age + rate*dt
		if v > 1 {
			return 1
		}
		return v
	},
	'♻': func(t, period float64) float64 { // cycle [0..1]
		return (math.Sin(t/period*2*math.Pi) + 1) * 0.5
	},
	'⚖': func(a, b float64) float64 { return (a + b) * 0.5 }, // balance
	'🦠': func(v, rate, seed float64) float64 {                  // mutate
		n := math.Sin(seed*127.1)*43758.5453
		n = n - math.Floor(n)
		return v + (n-0.5)*2*rate
	},

	// ━━━ AUDIO / SIGNAL ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
	'♫': func(freq, amp, t float64) float64 { // oscillate
		return math.Sin(t*freq*2*math.Pi) * amp
	},

	// ━━━ SYSTEM ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
	'🌍': func() string { return "WorldRoot" },  // handled by runtime
	'🧬': func() string { return "GeneRegistry" }, // handled by runtime
	'🛡': func() string { return "SecurityGate" }, // bất biến — không bao giờ bypass
	'💾': func() string { return "Ledger" },     // handled by silk package
	'🔄': func() string { return "Render" },     // handled by ws package

	// ━━━ SPATIAL ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
	'√': func(v float64) float64 { return math.Sqrt(math.Abs(v)) },
	'×': func(a, b Vec3) Vec3 { return a.Cross(b) }, // cross product
}

// OpName trả về tên mô tả của operator
var OpName = map[rune]string{
	'●': "SDFSphere",
	'⌀': "SDFCapsule",
	'□': "SDFBox",
	'◌': "SDFVoid",
	'∪': "SmoothUnion",
	'∖': "SmoothSub",
	'⊕': "HardUnion",
	'∇': "SurfaceNormal",
	'·': "DotProduct",
	'☀': "SunLight",
	'👁': "RayMarch",
	'∫': "FBMTerrain",
	'π': "Pi",
	'∞': "Infinity",
	'φ': "GoldenRatio",
	// 🛡 already defined above
	'🌍': func() string { return "WorldRoot" },
	'🧬': func() string { return "GeneRegistry" },
}
