// internal/gene/alphabet/operators.go
//
// OPERATORS — 80 ký hiệu toán học, logic, vật lý, Olang
// =======================================================
// Đây không phải là ký tự ngôn ngữ — đây là LỆNH.
// Mỗi operator = một phép biến đổi thực sự trên thế giới vật lý.
//
// ∪ không có nghĩa là "union" trong toán học trừu tượng.
// ∪ có nghĩa là: hai vật thể này THỰC SỰ hợp nhất trong không gian 3D,
// với blending factor k điều chỉnh độ mượt của mặt tiếp xúc.
//
// Đây là ngôn ngữ của Olang — không phải ký hiệu — là thực tế.

package alphabet

import "math"

// AllOperators — 80 operators đầy đủ với SDF và DNA
var AllOperators = []*OlangChar{

	// ══ ORIGIN / IDENTITY ════════════════════════════════════

	// ○ — Origin: nguồn gốc của mọi thứ
	// ○(x) == x    ○(∅) == ○    ○ ∘ ○ == ○
	{
		Codepoint: 0x25CB, Glyph: "○", Name: "Origin",
		Script: "olang", Category: "identity",
		SDF:      SCircle(XCenter, YMid, 0.25),
		DNA:      "○",
		Phonemes: []string{"/ɔːrdʒɪn/"},
		ISLLayer: 'O', ISLGroup: 'R', ISLType: 'G', ISLID: 0,
	},

	// ══ SDF PRIMITIVES ═══════════════════════════════════════

	// ● — Sphere
	{
		Codepoint: 0x25CF, Glyph: "●", Name: "SDFSphere",
		Script: "olang", Category: "primitive",
		SDF: union(
			SCircle(XCenter, YMid, 0.22),
			SDot(XCenter, YMid, 0.15),
		),
		DNA:      "●(cx,cy,cz,r)",
		ISLLayer: 'S', ISLGroup: 'P', ISLType: 's', ISLID: 1,
	},

	// ⌀ — Capsule
	{
		Codepoint: 0x2300, Glyph: "⌀", Name: "SDFCapsule",
		Script: "olang", Category: "primitive",
		SDF: union(
			SCircle(XCenter, YTop*0.5, 0.14),
			SV(XCenter, YBot*0.5, YTop*0.5),
			SCircle(XCenter, YBot*0.5, 0.14),
		),
		DNA:      "⌀(ax,ay,az,bx,by,bz,r)",
		ISLLayer: 'S', ISLGroup: 'P', ISLType: 's', ISLID: 2,
	},

	// □ — Box
	{
		Codepoint: 0x25A1, Glyph: "□", Name: "SDFBox",
		Script: "olang", Category: "primitive",
		SDF: union(
			SH(XLeft, XRight, YTop), SH(XLeft, XRight, YBot),
			SV(XLeft, YBot, YTop),   SV(XRight, YBot, YTop),
		),
		DNA:      "□(cx,cy,cz,hx,hy,hz)",
		ISLLayer: 'S', ISLGroup: 'P', ISLType: 's', ISLID: 3,
	},

	// ◌ — Void
	{
		Codepoint: 0x25CC, Glyph: "◌", Name: "SDFVoid",
		Script: "olang", Category: "primitive",
		SDF:      SCircle(XCenter, YMid, 0.22),
		DNA:      "◌",
		ISLLayer: 'S', ISLGroup: 'P', ISLType: 's', ISLID: 4,
	},

	// ⬡ — Hexagonal Prism
	{
		Codepoint: 0x2B21, Glyph: "⬡", Name: "SDFHex",
		Script: "olang", Category: "primitive",
		SDF: func(p gene.Vec3) float64 {
			// approximate hex with 6 capsules
			r := 0.24
			d := math.MaxFloat64
			for i := 0; i < 6; i++ {
				a0 := float64(i) * math.Pi / 3
				a1 := float64(i+1) * math.Pi / 3
				pa := p3(r*math.Cos(a0), r*math.Sin(a0))
				pb := p3(r*math.Cos(a1), r*math.Sin(a1))
				if s := gene.SDFCapsule(p, pa, pb, T); s < d {
					d = s
				}
			}
			return d
		},
		DNA:      "⬡(cx,cy,cz,r,h)",
		ISLLayer: 'S', ISLGroup: 'P', ISLType: 's', ISLID: 5,
	},

	// ── SDF OPERATIONS ────────────────────────────────────────

	// ∪ — Smooth Union
	{
		Codepoint: 0x222A, Glyph: "∪", Name: "SmoothUnion",
		Script: "olang", Category: "operation",
		SDF: union(
			SArc(XCenter, YTop*0.3, 0.22, math.Pi, 2*math.Pi),
			SV(XLeft*0.85, YBot*0.5, YTop*0.3),
			SV(XRight*0.85, YBot*0.5, YTop*0.3),
		),
		DNA:      "∪(a,b,k)",
		Phonemes: []string{"/juːnjən/"},
		ISLLayer: 'S', ISLGroup: 'O', ISLType: 'u', ISLID: 1,
		SameMeaning: []uint32{0x2295, 0x002B}, // ⊕ +
	},

	// ∖ — Smooth Subtraction
	{
		Codepoint: 0x2216, Glyph: "∖", Name: "SmoothSub",
		Script: "olang", Category: "operation",
		SDF:      SD(XLeft*0.9, YBot*0.7, XRight*0.9, YTop*0.7),
		DNA:      "∖(a,b,k)",
		ISLLayer: 'S', ISLGroup: 'O', ISLType: 'u', ISLID: 2,
	},

	// ∩ — Smooth Intersection
	{
		Codepoint: 0x2229, Glyph: "∩", Name: "SmoothIntersect",
		Script: "olang", Category: "operation",
		SDF: union(
			SArc(XCenter, YBot*0.3, 0.22, 0, math.Pi),
			SV(XLeft*0.85, YBot*0.3, YTop*0.5),
			SV(XRight*0.85, YBot*0.3, YTop*0.5),
		),
		DNA:      "∩(a,b,k)",
		ISLLayer: 'S', ISLGroup: 'O', ISLType: 'u', ISLID: 3,
	},

	// ── CALCULUS / PHYSICS ────────────────────────────────────

	// ∫ — FBM Terrain / Integral
	{
		Codepoint: 0x222B, Glyph: "∫", Name: "Integral",
		Script: "olang", Category: "calculus",
		SDF: union(
			SV(XCenter, YBot*0.7, YTop*0.7),
			SArc(XCenter+0.08, YTop*0.6, 0.08, math.Pi*1.5, math.Pi*2.5),
			SArc(XCenter-0.08, YBot*0.6, 0.08, math.Pi*0.5, math.Pi*1.5),
		),
		DNA:      "∫(f,a,b)",
		Phonemes: []string{"/ɪntɪɡrəl/"},
		ISLLayer: 'S', ISLGroup: 'C', ISLType: 'c', ISLID: 1,
	},

	// ∇ — Gradient / Nabla
	{
		Codepoint: 0x2207, Glyph: "∇", Name: "Gradient",
		Script: "olang", Category: "calculus",
		SDF: union(
			SH(XLeft, XRight, YTop),
			SD(XLeft, YTop, XCenter, YBot*0.7),
			SD(XCenter, YBot*0.7, XRight, YTop),
		),
		DNA:      "∇(f)",
		Phonemes: []string{"/ɡreɪdiənt/"},
		ISLLayer: 'S', ISLGroup: 'C', ISLType: 'c', ISLID: 2,
	},

	// ∂ — Partial Derivative
	{
		Codepoint: 0x2202, Glyph: "∂", Name: "Partial",
		Script: "olang", Category: "calculus",
		SDF: union(
			SArc(XCenter*0.3, YBot*0.45, 0.16, math.Pi*0.2, math.Pi*2.0),
			SV(XCenter*0.3+0.16, YMid, YTop),
		),
		DNA:      "∂(f)/∂(x)",
		ISLLayer: 'S', ISLGroup: 'C', ISLType: 'c', ISLID: 3,
	},

	// ∑ — Summation
	{
		Codepoint: 0x2211, Glyph: "∑", Name: "Sum",
		Script: "olang", Category: "math",
		SDF: union(
			SH(XLeft, XRight*0.9, YTop),
			SD(XRight*0.9, YTop, XLeft, YMid),
			SD(XLeft, YMid, XRight*0.9, YBot),
			SH(XLeft, XRight*0.9, YBot),
		),
		DNA:      "∑(i=0,n,f(i))",
		Phonemes: []string{"/sɪɡmə/"},
		ISLLayer: 'S', ISLGroup: 'M', ISLType: 'm', ISLID: 1,
	},

	// ∏ — Product
	{
		Codepoint: 0x220F, Glyph: "∏", Name: "Product",
		Script: "olang", Category: "math",
		SDF: union(
			SH(XLeft, XRight, YTop),
			SV(XLeft, YBot, YTop),
			SV(XRight, YBot, YTop),
		),
		DNA:      "∏(i=0,n,f(i))",
		ISLLayer: 'S', ISLGroup: 'M', ISLType: 'm', ISLID: 2,
	},

	// √ — Square Root
	{
		Codepoint: 0x221A, Glyph: "√", Name: "Sqrt",
		Script: "olang", Category: "math",
		SDF: union(
			SD(XLeft*0.9, YMid, XLeft*0.3, YBot*0.6),
			SD(XLeft*0.3, YBot*0.6, XCenter*0.3, YTop*0.8),
			SH(XCenter*0.3, XRight*0.9, YTop*0.8),
		),
		DNA:      "√(x)",
		ISLLayer: 'S', ISLGroup: 'M', ISLType: 'm', ISLID: 3,
	},

	// ∞ — Infinity (trong QT2: không tồn tại — ∞-1 mới tồn tại thật)
	{
		Codepoint: 0x221E, Glyph: "∞", Name: "Infinity",
		Script: "olang", Category: "math",
		SDF: sUnion(0.05,
			SCircle(XMidL, YMid, 0.14),
			SCircle(XMidR, YMid, 0.14),
		),
		DNA:      "∞ := ∞-1+1 (QT2: không tồn tại thực)",
		Phonemes: []string{"/ɪnfɪnɪti/"},
		ISLLayer: 'S', ISLGroup: 'M', ISLType: 'm', ISLID: 4,
	},

	// ── LOGIC ─────────────────────────────────────────────────

	// ∧ — AND
	{
		Codepoint: 0x2227, Glyph: "∧", Name: "And",
		Script: "olang", Category: "logic",
		SDF: union(
			SD(XLeft, YBot*0.7, XCenter, YTop*0.7),
			SD(XCenter, YTop*0.7, XRight, YBot*0.7),
		),
		DNA:      "∧(a,b)",
		ISLLayer: 'L', ISLGroup: 'G', ISLType: 'l', ISLID: 1,
		SameShape: []uint32{0x0041}, // A (hình giống A)
	},

	// ∨ — OR
	{
		Codepoint: 0x2228, Glyph: "∨", Name: "Or",
		Script: "olang", Category: "logic",
		SDF: union(
			SD(XLeft, YTop*0.7, XCenter, YBot*0.7),
			SD(XCenter, YBot*0.7, XRight, YTop*0.7),
		),
		DNA:      "∨(a,b)",
		ISLLayer: 'L', ISLGroup: 'G', ISLType: 'l', ISLID: 2,
		SameShape: []uint32{0x0056}, // V
	},

	// ¬ — NOT
	{
		Codepoint: 0x00AC, Glyph: "¬", Name: "Not",
		Script: "olang", Category: "logic",
		SDF: union(
			SH(XLeft, XRight*0.6, YMid),
			SV(XRight*0.6, YBot*0.5, YMid),
		),
		DNA:      "¬(a)",
		ISLLayer: 'L', ISLGroup: 'G', ISLType: 'l', ISLID: 3,
	},

	// ∀ — For All
	{
		Codepoint: 0x2200, Glyph: "∀", Name: "ForAll",
		Script: "olang", Category: "logic",
		SDF: union(
			SD(XLeft, YTop*0.7, XCenter, YBot*0.7),
			SD(XCenter, YBot*0.7, XRight, YTop*0.7),
			SH(XLeft*0.5, XRight*0.5, YMid),
		),
		DNA:      "∀x.P(x)",
		ISLLayer: 'L', ISLGroup: 'G', ISLType: 'l', ISLID: 4,
	},

	// ∃ — Exists
	{
		Codepoint: 0x2203, Glyph: "∃", Name: "Exists",
		Script: "olang", Category: "logic",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SH(XLeft, XRight*0.8, YTop),
			SH(XLeft, XRight*0.7, YMid),
			SH(XLeft, XRight*0.8, YBot),
		),
		DNA:      "∃x.P(x)",
		ISLLayer: 'L', ISLGroup: 'G', ISLType: 'l', ISLID: 5,
	},

	// ⊤ — True / Top
	{
		Codepoint: 0x22A4, Glyph: "⊤", Name: "True",
		Script: "olang", Category: "logic",
		SDF: union(
			SH(XLeft, XRight, YTop),
			SV(XCenter, YBot, YTop),
		),
		DNA: "⊤",
		ISLLayer: 'L', ISLGroup: 'G', ISLType: 'l', ISLID: 6,
	},

	// ⊥ — False / Bottom
	{
		Codepoint: 0x22A5, Glyph: "⊥", Name: "False",
		Script: "olang", Category: "logic",
		SDF: union(
			SH(XLeft, XRight, YBot),
			SV(XCenter, YBot, YTop),
		),
		DNA: "⊥",
		ISLLayer: 'L', ISLGroup: 'G', ISLType: 'l', ISLID: 7,
	},

	// ── RELATIONS ─────────────────────────────────────────────

	// = — Equal (vật lý đã chứng minh — QT3)
	{
		Codepoint: 0x3D, Glyph: "=", Name: "Equal",
		Script: "olang", Category: "relation",
		SDF: union(
			SH(XLeft, XRight, YMid+0.08),
			SH(XLeft, XRight, YMid-0.08),
		),
		DNA:      "=(a,b) // sự thật vật lý đã chứng minh",
		ISLLayer: 'R', ISLGroup: 'E', ISLType: 'r', ISLID: 1,
	},

	// ≡ — Identical / Cross-script equivalent
	{
		Codepoint: 0x2261, Glyph: "≡", Name: "Identical",
		Script: "olang", Category: "relation",
		SDF: union(
			SH(XLeft, XRight, YMid+0.12),
			SH(XLeft, XRight, YMid),
			SH(XLeft, XRight, YMid-0.12),
		),
		DNA:      "≡(a,b) // A≡α≡А≡अ (cùng âm, khác script)",
		ISLLayer: 'R', ISLGroup: 'E', ISLType: 'r', ISLID: 2,
	},

	// ≈ — Approximately
	{
		Codepoint: 0x2248, Glyph: "≈", Name: "Approx",
		Script: "olang", Category: "relation",
		SDF: union(
			SArc(XLeft*0.5, YMid+0.10, 0.18, 0, math.Pi),
			SArc(XLeft*0.5, YMid-0.10, 0.18, 0, math.Pi),
		),
		DNA:      "≈(a,b,ε)",
		ISLLayer: 'R', ISLGroup: 'E', ISLType: 'r', ISLID: 3,
	},

	// ≠ — Not Equal
	{
		Codepoint: 0x2260, Glyph: "≠", Name: "NotEqual",
		Script: "olang", Category: "relation",
		SDF: union(
			SH(XLeft, XRight, YMid+0.08),
			SH(XLeft, XRight, YMid-0.08),
			SD(XLeft*0.5, YBot*0.6, XRight*0.5, YTop*0.6),
		),
		DNA:      "≠(a,b)",
		ISLLayer: 'R', ISLGroup: 'E', ISLType: 'r', ISLID: 4,
	},

	// < > ≤ ≥
	{
		Codepoint: 0x3C, Glyph: "<", Name: "LessThan",
		Script: "olang", Category: "relation",
		SDF: union(
			SD(XRight*0.7, YTop*0.6, XLeft*0.7, YMid),
			SD(XLeft*0.7, YMid, XRight*0.7, YBot*0.6),
		),
		DNA: "<(a,b)",
		ISLLayer: 'R', ISLGroup: 'E', ISLType: 'r', ISLID: 5,
	},
	{
		Codepoint: 0x3E, Glyph: ">", Name: "GreaterThan",
		Script: "olang", Category: "relation",
		SDF: union(
			SD(XLeft*0.7, YTop*0.6, XRight*0.7, YMid),
			SD(XRight*0.7, YMid, XLeft*0.7, YBot*0.6),
		),
		DNA: ">(a,b)",
		ISLLayer: 'R', ISLGroup: 'E', ISLType: 'r', ISLID: 6,
	},

	// ∈ — Member of (ISL: child node)
	{
		Codepoint: 0x2208, Glyph: "∈", Name: "Member",
		Script: "olang", Category: "relation",
		SDF: union(
			SArc(XCenter, YMid, 0.20, math.Pi*0.3, math.Pi*1.7),
			SH(-0.14, 0.14, YMid),
		),
		DNA:      "∈(x,S) // x là thành viên của S",
		ISLLayer: 'R', ISLGroup: 'S', ISLType: 'r', ISLID: 7,
	},

	// ⊂ ⊃ — Subset / Superset
	{
		Codepoint: 0x2282, Glyph: "⊂", Name: "Subset",
		Script: "olang", Category: "relation",
		SDF: union(
			SArc(XCenter*0.3, YMid, 0.22, math.Pi*0.4, math.Pi*1.6),
			SV(XCenter*0.3-0.22*0.85, YMid-0.14, YMid+0.14),
		),
		DNA:      "⊂(A,B) // A là tập con của B",
		ISLLayer: 'R', ISLGroup: 'S', ISLType: 'r', ISLID: 8,
	},

	// ── ARROWS / FLOW ─────────────────────────────────────────

	// → — FlowRight (maps to, implies)
	{
		Codepoint: 0x2192, Glyph: "→", Name: "FlowRight",
		Script: "olang", Category: "flow",
		SDF: union(
			SH(XLeft, XRight*0.6, YMid),
			SD(XRight*0.6, YMid, XRight*0.05, YTop*0.4),
			SD(XRight*0.6, YMid, XRight*0.05, YBot*0.4),
		),
		DNA:      "→(from,to)",
		ISLLayer: 'F', ISLGroup: 'L', ISLType: 'f', ISLID: 1,
	},

	// ← ↑ ↓ ↔ ⇒ ⇔
	{
		Codepoint: 0x2190, Glyph: "←", Name: "FlowLeft",
		Script: "olang", Category: "flow",
		SDF: union(
			SH(XLeft*0.6, XRight, YMid),
			SD(XLeft*0.6, YMid, XLeft*0.05, YTop*0.4),
			SD(XLeft*0.6, YMid, XLeft*0.05, YBot*0.4),
		),
		DNA: "←(to,from)",
		ISLLayer: 'F', ISLGroup: 'L', ISLType: 'f', ISLID: 2,
	},
	{
		Codepoint: 0x2191, Glyph: "↑", Name: "Promote",
		Script: "olang", Category: "flow",
		SDF: union(
			SV(XCenter, YBot*0.6, YTop),
			SD(XCenter, YTop, XLeft*0.4, YTop*0.4),
			SD(XCenter, YTop, XRight*0.4, YTop*0.4),
		),
		DNA: "↑(ΔΨ→QR) // promote short-term to long-term",
		ISLLayer: 'F', ISLGroup: 'L', ISLType: 'f', ISLID: 3,
	},
	{
		Codepoint: 0x2193, Glyph: "↓", Name: "Demote",
		Script: "olang", Category: "flow",
		SDF: union(
			SV(XCenter, YTop*0.6, YBot),
			SD(XCenter, YBot, XLeft*0.4, YBot*0.4),
			SD(XCenter, YBot, XRight*0.4, YBot*0.4),
		),
		DNA: "↓(decay)",
		ISLLayer: 'F', ISLGroup: 'L', ISLType: 'f', ISLID: 4,
	},
	{
		Codepoint: 0x21D2, Glyph: "⇒", Name: "Implies",
		Script: "olang", Category: "flow",
		SDF: union(
			SH(XLeft, XRight*0.5, YMid+0.06),
			SH(XLeft, XRight*0.5, YMid-0.06),
			SD(XRight*0.5, YTop*0.4, XRight*0.9, YMid),
			SD(XRight*0.9, YMid, XRight*0.5, YBot*0.4),
		),
		DNA: "⇒(a,b) // a implies b",
		ISLLayer: 'F', ISLGroup: 'L', ISLType: 'f', ISLID: 5,
	},
	{
		Codepoint: 0x21D4, Glyph: "⇔", Name: "Iff",
		Script: "olang", Category: "flow",
		SDF: union(
			SH(XLeft*0.5, XRight*0.5, YMid+0.06),
			SH(XLeft*0.5, XRight*0.5, YMid-0.06),
			SD(XLeft*0.5, YTop*0.4, XLeft*0.9, YMid),
			SD(XLeft*0.9, YMid, XLeft*0.5, YBot*0.4),
			SD(XRight*0.5, YTop*0.4, XRight*0.9, YMid),
			SD(XRight*0.9, YMid, XRight*0.5, YBot*0.4),
		),
		DNA: "⇔(a,b) // if and only if",
		ISLLayer: 'F', ISLGroup: 'L', ISLType: 'f', ISLID: 6,
	},

	// ── LIGHT / RENDER ────────────────────────────────────────

	// ☀ — SunLight
	{
		Codepoint: 0x2600, Glyph: "☀", Name: "SunLight",
		Script: "olang", Category: "light",
		SDF: func(p gene.Vec3) float64 {
			center := SCircle(XCenter, YMid, 0.14)(p)
			// 8 rays
			d := center
			for i := 0; i < 8; i++ {
				a := float64(i) * math.Pi / 4
				rx := math.Cos(a)
				ry := math.Sin(a)
				ray := cap2(0.16*rx, 0.16*ry, 0.28*rx, 0.28*ry, TL)(p)
				if ray < d { d = ray }
			}
			return d
		},
		DNA:      "☀(t,elevation,azimuth)",
		ISLLayer: 'S', ISLGroup: 'L', ISLType: 'l', ISLID: 1,
	},

	// ★ — Star / Point Light
	{
		Codepoint: 0x2605, Glyph: "★", Name: "StarLight",
		Script: "olang", Category: "light",
		SDF: func(p gene.Vec3) float64 {
			d := math.MaxFloat64
			for i := 0; i < 5; i++ {
				a0 := float64(i)*2*math.Pi/5 - math.Pi/2
				a1 := float64(i)*2*math.Pi/5 + 2*math.Pi/5 - math.Pi/2
				pa := p3(0.24*math.Cos(a0), 0.24*math.Sin(a0))
				mid := p3(0.10*math.Cos((a0+a1)/2), 0.10*math.Sin((a0+a1)/2))
				pb := p3(0.24*math.Cos(a1), 0.24*math.Sin(a1))
				d1 := gene.SDFCapsule(p, pa, mid, TL)
				d2 := gene.SDFCapsule(p, mid, pb, TL)
				if d1 < d { d = d1 }
				if d2 < d { d = d2 }
			}
			return d
		},
		DNA:      "★(cx,cy,cz,intensity,color)",
		ISLLayer: 'S', ISLGroup: 'L', ISLType: 'l', ISLID: 2,
	},

	// ── MEMORY / KNOWLEDGE (QT7) ──────────────────────────────

	// ΔΨ — ShortTerm Memory (Dendrites)
	{
		Codepoint: 0xE001, Glyph: "ΔΨ", Name: "ShortTerm",
		Script: "olang", Category: "memory",
		SDF: union(
			// Δ
			SD(-0.28, YBot*0.7, XCenter*(-0.2), YTop*0.7),
			SD(XCenter*(-0.2), YTop*0.7, XMidL*0.5, YBot*0.7),
			SH(-0.28, XMidL*0.5, YBot*0.7),
		),
		DNA:      "ΔΨ(capacity,decay_rate) // short-term: học nhanh, quên nhanh",
		ISLLayer: 'M', ISLGroup: 'S', ISLType: 'm', ISLID: 1,
	},

	// QR — LongTerm Memory (Axon, Silk Tree)
	{
		Codepoint: 0xE002, Glyph: "QR", Name: "LongTerm",
		Script: "olang", Category: "memory",
		SDF: union(
			SH(XLeft, XRight, YTop),
			SV(XLeft, YBot, YTop),
			SH(XLeft, XCenter*0.5, YMid),
			SV(XCenter*0.5, YBot, YMid),
		),
		DNA:      "QR(key,value,sig:ED25519) // long-term: append-only, đã chứng minh",
		ISLLayer: 'M', ISLGroup: 'L', ISLType: 'm', ISLID: 2,
	},

	// ◎ — Dream
	{
		Codepoint: 0x25CE, Glyph: "◎", Name: "Dream",
		Script: "olang", Category: "memory",
		SDF: sUnion(0.02,
			SCircle(XCenter, YMid, 0.22),
			SCircle(XCenter, YMid, 0.12),
		),
		DNA:      "◎(ΔΨ→verify→QR|delete) // dreaming: kiểm chứng khi rảnh",
		ISLLayer: 'M', ISLGroup: 'D', ISLType: 'm', ISLID: 3,
	},

	// ── GEOMETRY (ISL địa chỉ) ────────────────────────────────

	// + — Plus / Add
	{
		Codepoint: 0x2B, Glyph: "+", Name: "Plus",
		Script: "olang", Category: "arithmetic",
		SDF: union(
			SH(XLeft*0.7, XRight*0.7, YMid),
			SV(XCenter, YBot*0.7, YTop*0.7),
		),
		DNA:      "+(a,b)",
		ISLLayer: 'A', ISLGroup: 'R', ISLType: 'a', ISLID: 1,
		SameMeaning: []uint32{0x222A, 0x2295}, // ∪ ⊕
	},

	// − — Minus
	{
		Codepoint: 0x2212, Glyph: "−", Name: "Minus",
		Script: "olang", Category: "arithmetic",
		SDF:      SH(XLeft*0.7, XRight*0.7, YMid),
		DNA:      "−(a,b)",
		ISLLayer: 'A', ISLGroup: 'R', ISLType: 'a', ISLID: 2,
	},

	// × — Multiply
	{
		Codepoint: 0x00D7, Glyph: "×", Name: "Multiply",
		Script: "olang", Category: "arithmetic",
		SDF: union(
			SD(XLeft*0.7, YTop*0.5, XRight*0.7, YBot*0.5),
			SD(XLeft*0.7, YBot*0.5, XRight*0.7, YTop*0.5),
		),
		DNA:      "×(a,b)",
		ISLLayer: 'A', ISLGroup: 'R', ISLType: 'a', ISLID: 3,
		SameShape: []uint32{0x0058, 0x0078, 0x03A7}, // X x Χ
	},

	// ÷ — Divide
	{
		Codepoint: 0x00F7, Glyph: "÷", Name: "Divide",
		Script: "olang", Category: "arithmetic",
		SDF: union(
			SH(XLeft*0.7, XRight*0.7, YMid),
			SDot(XCenter, YMid+0.14, 0.055),
			SDot(XCenter, YMid-0.14, 0.055),
		),
		DNA:      "÷(a,b)",
		ISLLayer: 'A', ISLGroup: 'R', ISLType: 'a', ISLID: 4,
	},

	// π — Pi
	{
		Codepoint: 0x03C0, Glyph: "π", Name: "Pi",
		Script: "olang", Category: "constant",
		SDF: union(
			SH(XLeft, XRight, YTop*0.6),
			SV(XMidL, YBot, YTop*0.6),
			SHook(XMidR, YTop*0.6, YBase),
		),
		DNA:      "π := 3.14159265358979...",
		Phonemes: []string{"/paɪ/"},
		ISLLayer: 'C', ISLGroup: 'N', ISLType: 'c', ISLID: 1,
	},

	// φ — Golden Ratio
	{
		Codepoint: 0x03C6, Glyph: "φ", Name: "GoldenRatio",
		Script: "olang", Category: "constant",
		SDF: union(
			SCircle(XCenter*0.2, YMid, 0.18),
			SV(XCenter*0.2+0.18, YBot, YTop),
		),
		DNA:      "φ := 1.61803398874989...",
		Phonemes: []string{"/faɪ/"},
		ISLLayer: 'C', ISLGroup: 'N', ISLType: 'c', ISLID: 2,
	},

	// ── SYSTEM OPERATORS ──────────────────────────────────────

	// 🛡 — SecurityGate
	{
		Codepoint: 0x1F6E1, Glyph: "🛡", Name: "SecurityGate",
		Script: "olang", Category: "system",
		SDF: union(
			SArc(XCenter, YMid+0.05, 0.25, math.Pi*0.1, math.Pi*0.9),
			SH(XLeft*0.9, XRight*0.9, YMid+0.05),
			SV(XCenter, YBot*0.6, YMid+0.05),
		),
		DNA:      "🛡(Rule1:NoHarm, Rule2:NoLoop, Rule3:NoDelete)",
		ISLLayer: 'S', ISLGroup: 'Y', ISLType: 's', ISLID: 1,
	},

	// 👁 — Vision
	{
		Codepoint: 0x1F441, Glyph: "👁", Name: "Vision",
		Script: "olang", Category: "perception",
		SDF: union(
			SArc(XCenter, YMid, 0.22, math.Pi*0.2, math.Pi*0.8),
			SArc(XCenter, YMid, 0.22, math.Pi*1.2, math.Pi*1.8),
			SCircle(XCenter, YMid, 0.10),
			SDot(XCenter, YMid, 0.05),
		),
		DNA:      "👁(rayMarch,fibonacci,ISL→address)",
		ISLLayer: 'P', ISLGroup: 'V', ISLType: 'p', ISLID: 1,
	},

	// 🌍 — World Root
	{
		Codepoint: 0x1F30D, Glyph: "🌍", Name: "WorldRoot",
		Script: "olang", Category: "system",
		SDF: sUnion(0.02,
			SCircle(XCenter, YMid, 0.24),
			SD(XLeft*0.9, YMid, XRight*0.9, YMid),
			SD(XCenter, YTop*0.9, XCenter, YBot*0.9),
		),
		DNA:      "🌍(seed,∫fbm,∇,☀)",
		ISLLayer: 'W', ISLGroup: 'R', ISLType: 'w', ISLID: 1,
	},
}

// import needed for SDFCapsule in operator SDFs
var _ = gene.Vec3{}
