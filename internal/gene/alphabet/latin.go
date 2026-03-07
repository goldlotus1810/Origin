// internal/gene/alphabet/latin.go
//
// LATIN ALPHABET — 52 chữ cái + 10 chữ số
// =========================================
// Mỗi chữ = công thức SDF tạo từ primitive strokes.
// Mỗi chữ có: SDF + DNA string + Phoneme + StrokeOrder + ISL + Relations.
//
// Nguồn gốc lịch sử:
//   Phoenician (3200 BC) → Greek → Etruscan → Latin → tất cả chữ Latin hiện đại
//   A ← 𐤀 Aleph (đầu bò)     B ← 𐤁 Beth (nhà)
//   C ← 𐤂 Gimel (lạc đà)     D ← 𐤃 Dalet (cửa)
//   ... mọi hình dạng đều có nghĩa vật lý từ thế giới thực

package alphabet

import "math"

// latinUpper — 26 chữ hoa A-Z
// Mỗi chữ được định nghĩa bằng SDF công thức từ primitive strokes.
// DNA string = Olang notation để lưu vào SilkTree.
var LatinUpper = []*OlangChar{

	// A — hai nét chéo gặp nhau ở đỉnh + một nét ngang giữa
	// Nguồn: Phoenician Aleph 𐤀 (đầu bò, lộn ngược)
	// Âm: /eɪ/ (name) hoặc /æ/ (cat) hoặc /ɑː/ (father)
	{
		Codepoint: 0x41, Glyph: "A", Name: "Latin Capital A",
		Script: "latin", Category: "letter",
		SDF: union(
			SD(XLeft, YBot, XCenter, YTop),        // nét chéo trái
			SD(XCenter, YTop, XRight, YBot),        // nét chéo phải
			SH(XMidL*0.9, XMidR*0.9, YMid*0.2),   // nét ngang giữa
		),
		DNA:      "∪(⌀(-.28,-.38,.0,.44,0), ⌀(.0,.44,.28,-.38,0), ⌀(-.13,.08,.13,.08,0))",
		Phonemes: []string{"/eɪ/", "/æ/", "/ɑː/"},
		StrokeOrder: []*gene.Spline{
			StrokeSpline([][2]float64{{XLeft, YBot}, {XCenter, YTop}}),
			StrokeSpline([][2]float64{{XCenter, YTop}, {XRight, YBot}}),
			StrokeSpline([][2]float64{{XMidL * 0.9, YMid * 0.2}, {XMidR * 0.9, YMid * 0.2}}),
		},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 1,
		SameSound:   []uint32{0x3B1, 0x0410, 0x0905, 0x3042}, // α А अ あ
		DerivedFrom: 0x10900,                                   // 𐤀 Phoenician Aleph
		LowercaseOf: 0x61,                                      // a
	},

	// B — nét đứng + hai bụng cong phải
	// Nguồn: Phoenician Beth 𐤁 (nhà, mặt tiền)
	{
		Codepoint: 0x42, Glyph: "B", Name: "Latin Capital B",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SCurveR(XLeft, YTop, YMid+0.02),
			SCurveR(XLeft, YMid+0.02, YBot),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.38,0), curveR(-.28,.38,.02,0.18), curveR(-.28,.02,-.38,0.18))",
		Phonemes: []string{"/b/"},
		StrokeOrder: []*gene.Spline{
			StrokeSpline([][2]float64{{XLeft, YTop}, {XLeft, YBot}}),
			StrokeSpline([][2]float64{{XLeft, YTop}, {XRight, YMid + 0.1}, {XLeft, YMid + 0.02}}),
			StrokeSpline([][2]float64{{XLeft, YMid + 0.02}, {XRight, YMid - 0.15}, {XLeft, YBot}}),
		},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 2,
		SameSound:   []uint32{0x3B2, 0x0411},      // β Б
		DerivedFrom: 0x10901,                        // 𐤁 Phoenician Beth
		LowercaseOf: 0x62,
	},

	// C — cung tròn mở phải
	// Nguồn: Phoenician Gimel 𐤂 (lạc đà, cổ)
	{
		Codepoint: 0x43, Glyph: "C", Name: "Latin Capital C",
		Script: "latin", Category: "letter",
		SDF: SArc(XCenter*0.4, YMid, 0.28, math.Pi*0.25, math.Pi*1.75),
		DNA:      "⌀arc(0,.0,.28,45°,315°)",
		Phonemes: []string{"/k/", "/s/"},
		StrokeOrder: []*gene.Spline{
			StrokeSpline([][2]float64{
				{XRight * 0.7, YTop * 0.8},
				{XLeft * 1.1, YMid + 0.1},
				{XLeft * 1.1, YMid - 0.1},
				{XRight * 0.7, YBot * 0.8},
			}),
		},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 3,
		DerivedFrom: 0x10902, // 𐤂
		LowercaseOf: 0x63,
	},

	// D — nét đứng + bụng cong phải to
	{
		Codepoint: 0x44, Glyph: "D", Name: "Latin Capital D",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SArc(XLeft, YMid, 0.30, -math.Pi/2, math.Pi/2),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.38,0), ⌀arc(-.28,.0,.30,-90°,90°))",
		Phonemes: []string{"/d/"},
		StrokeOrder: []*gene.Spline{
			StrokeSpline([][2]float64{{XLeft, YTop}, {XLeft, YBot}}),
			StrokeSpline([][2]float64{{XLeft, YTop}, {XRight * 1.1, YMid}, {XLeft, YBot}}),
		},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 4,
		DerivedFrom: 0x10903, // 𐤃 Dalet
		LowercaseOf: 0x64,
	},

	// E — nét đứng + 3 nét ngang
	// Nguồn: Phoenician He 𐤄 (cửa sổ, hai tay giơ)
	{
		Codepoint: 0x45, Glyph: "E", Name: "Latin Capital E",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SH(XLeft, XRight*0.9, YTop),
			SH(XLeft, XRight*0.7, YMid),
			SH(XLeft, XRight*0.9, YBot),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.38,0), ⌀(-.28,.38,.25,.38,0), ⌀(-.28,0,.20,0,0), ⌀(-.28,-.38,.25,-.38,0))",
		Phonemes: []string{"/iː/", "/ɛ/", "/e/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 5,
		SameSound:   []uint32{0x3B5, 0x0415}, // ε Е
		DerivedFrom: 0x10904,
		LowercaseOf: 0x65,
	},

	// F — nét đứng + 2 nét ngang trên
	{
		Codepoint: 0x46, Glyph: "F", Name: "Latin Capital F",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SH(XLeft, XRight*0.9, YTop),
			SH(XLeft, XRight*0.7, YMid),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.38,0), ⌀(-.28,.38,.25,.38,0), ⌀(-.28,0,.20,0,0))",
		Phonemes: []string{"/f/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 6,
		LowercaseOf: 0x66,
	},

	// G — C + nét ngang vào trong ở giữa
	{
		Codepoint: 0x47, Glyph: "G", Name: "Latin Capital G",
		Script: "latin", Category: "letter",
		SDF: union(
			SArc(XCenter*0.4, YMid, 0.28, math.Pi*0.25, math.Pi*1.75),
			SH(XCenter*0.4, XRight, YMid),
			SV(XRight, YMid, YBot*0.1),
		),
		DNA:      "∪(arc(0,.0,.28), ⌀(.0,.0,.28,.0,0), ⌀(.28,.0,.28,-.10,0))",
		Phonemes: []string{"/ɡ/", "/dʒ/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 7,
		LowercaseOf: 0x67,
	},

	// H — hai nét đứng + nét ngang giữa
	{
		Codepoint: 0x48, Glyph: "H", Name: "Latin Capital H",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SV(XRight, YBot, YTop),
			SH(XLeft, XRight, YMid),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.38,0), ⌀(.28,-.38,.28,.38,0), ⌀(-.28,0,.28,0,0))",
		Phonemes: []string{"/h/"},
		StrokeOrder: []*gene.Spline{
			StrokeSpline([][2]float64{{XLeft, YTop}, {XLeft, YBot}}),
			StrokeSpline([][2]float64{{XRight, YTop}, {XRight, YBot}}),
			StrokeSpline([][2]float64{{XLeft, YMid}, {XRight, YMid}}),
		},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 8,
		SameSound:   []uint32{0x0126, 0x0425}, // Ħ Х
		DerivedFrom: 0x10907,
		LowercaseOf: 0x68,
	},

	// I — nét đứng đơn
	{
		Codepoint: 0x49, Glyph: "I", Name: "Latin Capital I",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XCenter, YBot, YTop),
			SH(XMidL, XMidR, YTop),
			SH(XMidL, XMidR, YBot),
		),
		DNA:      "∪(⌀(0,-.38,0,.38,0), ⌀(-.14,.38,.14,.38,0), ⌀(-.14,-.38,.14,-.38,0))",
		Phonemes: []string{"/aɪ/", "/ɪ/", "/iː/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 9,
		SameSound:   []uint32{0x0399, 0x0418}, // Ι И
		LowercaseOf: 0x69,
	},

	// J — nét đứng + móc trái ở đáy
	{
		Codepoint: 0x4A, Glyph: "J", Name: "Latin Capital J",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XMidR, YMid, YTop),
			SH(XLeft, XMidR, YTop),
			SHook(XMidR, YMid, YBase),
		),
		DNA:      "∪(⌀(.14,.0,.14,.38,0), ⌀(-.28,.38,.14,.38,0), hook(.14,.0,-.38))",
		Phonemes: []string{"/dʒ/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 10,
		LowercaseOf: 0x6A,
	},

	// K — nét đứng + hai nét chéo ra phải
	{
		Codepoint: 0x4B, Glyph: "K", Name: "Latin Capital K",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SD(XLeft, YMid, XRight, YTop),
			SD(XLeft, YMid, XRight, YBot),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.38,0), ⌀(-.28,0,.28,.38,0), ⌀(-.28,0,.28,-.38,0))",
		Phonemes: []string{"/k/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 11,
		SameSound:   []uint32{0x039A, 0x041A}, // Κ К
		LowercaseOf: 0x6B,
	},

	// L — nét đứng + nét ngang đáy
	{
		Codepoint: 0x4C, Glyph: "L", Name: "Latin Capital L",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SH(XLeft, XRight*0.9, YBot),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.38,0), ⌀(-.28,-.38,.25,-.38,0))",
		Phonemes: []string{"/l/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 12,
		SameSound:   []uint32{0x039B}, // Λ (shape similar)
		LowercaseOf: 0x6C,
	},

	// M — hai nét đứng + hai nét chéo vào giữa
	{
		Codepoint: 0x4D, Glyph: "M", Name: "Latin Capital M",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SV(XRight, YBot, YTop),
			SD(XLeft, YTop, XCenter, YMid),
			SD(XCenter, YMid, XRight, YTop),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.38,0), ⌀(.28,-.38,.28,.38,0), ⌀(-.28,.38,0,.0,0), ⌀(0,.0,.28,.38,0))",
		Phonemes: []string{"/m/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 13,
		SameSound:   []uint32{0x039C, 0x041C, 0x092E}, // Μ М म
		DerivedFrom: 0x1090C,
		LowercaseOf: 0x6D,
	},

	// N — hai nét đứng + nét chéo từ trái-trên xuống phải-dưới
	{
		Codepoint: 0x4E, Glyph: "N", Name: "Latin Capital N",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SV(XRight, YBot, YTop),
			SD(XLeft, YTop, XRight, YBot),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.38,0), ⌀(.28,-.38,.28,.38,0), ⌀(-.28,.38,.28,-.38,0))",
		Phonemes: []string{"/n/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 14,
		SameSound:   []uint32{0x039D, 0x041D, 0x0928}, // Ν Н न
		DerivedFrom: 0x1090D,
		LowercaseOf: 0x6E,
	},

	// O — đường tròn
	// Nguồn: Phoenician Ayin 𐤏 (con mắt)
	{
		Codepoint: 0x4F, Glyph: "O", Name: "Latin Capital O",
		Script: "latin", Category: "letter",
		SDF: SCircle(XCenter, YMid+0.03, 0.26),
		DNA:      "○(0,.03,.26)",
		Phonemes: []string{"/oʊ/", "/ɒ/", "/ɔː/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 15,
		SameShape:   []uint32{0x30AA, 0x004F, 0x039F, 0x041E, 0x25CB}, // オ O Ο О ○
		SameSound:   []uint32{0x039F, 0x041E, 0x0913},                  // Ο О ओ
		DerivedFrom: 0x1090F,
		LowercaseOf: 0x6F,
	},

	// P — nét đứng + bụng trên phải
	{
		Codepoint: 0x50, Glyph: "P", Name: "Latin Capital P",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SCurveR(XLeft, YTop, YMid+0.05),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.38,0), curveR(-.28,.38,.05,0.18))",
		Phonemes: []string{"/p/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 16,
		SameSound:   []uint32{0x03A1, 0x0420}, // Ρ Р (shape similar)
		LowercaseOf: 0x70,
	},

	// Q — O + nét chéo nhỏ góc phải dưới
	{
		Codepoint: 0x51, Glyph: "Q", Name: "Latin Capital Q",
		Script: "latin", Category: "letter",
		SDF: union(
			SCircle(XCenter, YMid+0.03, 0.26),
			SD(XCenter*0.3, YMid-0.12, XRight*1.05, YBot*0.8),
		),
		DNA:      "∪(○(0,.03,.26), ⌀(.06,-.12,.30,-.32,0))",
		Phonemes: []string{"/k/", "/kj/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 17,
		LowercaseOf: 0x71,
	},

	// R — P + nét chéo phải xuống
	{
		Codepoint: 0x52, Glyph: "R", Name: "Latin Capital R",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SCurveR(XLeft, YTop, YMid+0.05),
			SD(XLeft+0.03, YMid+0.05, XRight, YBot),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.38,0), curveR(-.28,.38,.05), ⌀(-.25,.05,.28,-.38,0))",
		Phonemes: []string{"/r/", "/ɹ/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 18,
		SameSound:   []uint32{0x03A1, 0x0420}, // Ρ Р
		DerivedFrom: 0x10910,
		LowercaseOf: 0x72,
	},

	// S — hai cung ngược chiều nối nhau
	{
		Codepoint: 0x53, Glyph: "S", Name: "Latin Capital S",
		Script: "latin", Category: "letter",
		SDF: union(
			SArc(XCenter*0.3, YTop*0.55, 0.16, math.Pi*0.2, math.Pi*1.1),
			SArc(XCenter*(-0.3), YBot*0.55, 0.16, math.Pi*1.2, math.Pi*2.0),
		),
		DNA:      "∪(arc(.06,.24,.16,36°,198°), arc(-.06,-.24,.16,216°,360°))",
		Phonemes: []string{"/s/", "/z/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 19,
		SameSound:   []uint32{0x03A3, 0x0421}, // Σ С
		LowercaseOf: 0x73,
	},

	// T — nét đứng + nét ngang đỉnh
	{
		Codepoint: 0x54, Glyph: "T", Name: "Latin Capital T",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XCenter, YBot, YTop),
			SH(XLeft, XRight, YTop),
		),
		DNA:      "∪(⌀(0,-.38,0,.38,0), ⌀(-.28,.38,.28,.38,0))",
		Phonemes: []string{"/t/"},
		StrokeOrder: []*gene.Spline{
			StrokeSpline([][2]float64{{XLeft, YTop}, {XRight, YTop}}),
			StrokeSpline([][2]float64{{XCenter, YTop}, {XCenter, YBot}}),
		},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 20,
		SameSound:   []uint32{0x03A4, 0x0422, 0x0924}, // Τ Т त
		DerivedFrom: 0x10911,
		LowercaseOf: 0x74,
	},

	// U — cung đáy + hai nét đứng
	{
		Codepoint: 0x55, Glyph: "U", Name: "Latin Capital U",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YMid, YTop),
			SV(XRight, YMid, YTop),
			SArc(XCenter, YMid, 0.28, math.Pi, 2*math.Pi),
		),
		DNA:      "∪(⌀(-.28,.0,-.28,.38,0), ⌀(.28,.0,.28,.38,0), arc(0,.0,.28,180°,360°))",
		Phonemes: []string{"/juː/", "/ʌ/", "/ʊ/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 21,
		SameSound:   []uint32{0x03A5, 0x0423}, // Υ У
		LowercaseOf: 0x75,
	},

	// V — hai nét chéo gặp nhau ở đáy
	{
		Codepoint: 0x56, Glyph: "V", Name: "Latin Capital V",
		Script: "latin", Category: "letter",
		SDF: union(
			SD(XLeft, YTop, XCenter, YBot),
			SD(XCenter, YBot, XRight, YTop),
		),
		DNA:      "∪(⌀(-.28,.38,.0,-.38,0), ⌀(.0,-.38,.28,.38,0))",
		Phonemes: []string{"/v/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 22,
		SameShape:   []uint32{0x2228, 0x039B}, // ∨ Λ
		LowercaseOf: 0x76,
	},

	// W — V kép (4 nét chéo)
	{
		Codepoint: 0x57, Glyph: "W", Name: "Latin Capital W",
		Script: "latin", Category: "letter",
		SDF: union(
			SD(-0.35, YTop, -0.10, YBot),
			SD(-0.10, YBot, XCenter, YMid+0.05),
			SD(XCenter, YMid+0.05, 0.10, YBot),
			SD(0.10, YBot, 0.35, YTop),
		),
		DNA:      "∪(⌀(-.35,.38,-.10,-.38,0), ⌀(-.10,-.38,.0,.05,0), ⌀(.0,.05,.10,-.38,0), ⌀(.10,-.38,.35,.38,0))",
		Phonemes: []string{"/w/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 23,
		LowercaseOf: 0x77,
	},

	// X — hai nét chéo cắt nhau
	{
		Codepoint: 0x58, Glyph: "X", Name: "Latin Capital X",
		Script: "latin", Category: "letter",
		SDF: union(
			SD(XLeft, YTop, XRight, YBot),
			SD(XLeft, YBot, XRight, YTop),
		),
		DNA:      "∪(⌀(-.28,.38,.28,-.38,0), ⌀(-.28,-.38,.28,.38,0))",
		Phonemes: []string{"/ks/", "/ɡz/", "/z/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 24,
		SameShape:   []uint32{0x00D7, 0x2715, 0x03A7}, // × ✕ Χ
		SameSound:   []uint32{0x03A7, 0x0425},           // Χ Х
		LowercaseOf: 0x78,
	},

	// Y — hai nét chéo hội tụ giữa + nét đứng xuống
	{
		Codepoint: 0x59, Glyph: "Y", Name: "Latin Capital Y",
		Script: "latin", Category: "letter",
		SDF: union(
			SD(XLeft, YTop, XCenter, YMid),
			SD(XRight, YTop, XCenter, YMid),
			SV(XCenter, YBot, YMid),
		),
		DNA:      "∪(⌀(-.28,.38,.0,.0,0), ⌀(.28,.38,.0,.0,0), ⌀(0,.0,.0,-.38,0))",
		Phonemes: []string{"/waɪ/", "/j/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 25,
		SameShape:   []uint32{0x03A5, 0x0423}, // Υ У (shape)
		LowercaseOf: 0x79,
	},

	// Z — nét ngang trên + nét chéo + nét ngang dưới
	{
		Codepoint: 0x5A, Glyph: "Z", Name: "Latin Capital Z",
		Script: "latin", Category: "letter",
		SDF: union(
			SH(XLeft, XRight, YTop),
			SD(XRight, YTop, XLeft, YBot),
			SH(XLeft, XRight, YBot),
		),
		DNA:      "∪(⌀(-.28,.38,.28,.38,0), ⌀(.28,.38,-.28,-.38,0), ⌀(-.28,-.38,.28,-.38,0))",
		Phonemes: []string{"/z/", "/ts/"},
		ISLLayer: 'L', ISLGroup: 'U', ISLType: 'a', ISLID: 26,
		SameSound:   []uint32{0x0396, 0x0417}, // Ζ З
		LowercaseOf: 0x7A,
	},
}

// latLower — 26 chữ thường a-z
// Phần lớn là biến thể thu nhỏ của chữ hoa
// nhưng một số có hình dạng khác hoàn toàn: a, g, ...
var LatinLower = []*OlangChar{

	// a — hai dạng: one-story (ɑ) và two-story (a)
	// Dùng two-story (chữ in chuẩn): vòng tròn nhỏ + nét đứng phải
	{
		Codepoint: 0x61, Glyph: "a", Name: "Latin Small a",
		Script: "latin", Category: "letter",
		SDF: union(
			SCircle(XMidL*0.2, YXH*0.55, 0.16),
			SV(XMidR*1.3, YBot, YXH),
		),
		DNA:      "∪(○(-.03,.10,.16), ⌀(.18,-.38,.18,.18,0))",
		Phonemes: []string{"/æ/", "/ɑː/", "/eɪ/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 1,
		UppercaseOf: 0x41,
		SameSound:   []uint32{0x03B1, 0x0430, 0x0905, 0x3042}, // α а अ あ
	},

	// b — nét đứng cao + bụng phải dưới
	{
		Codepoint: 0x62, Glyph: "b", Name: "Latin Small b",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SCircle(XMidL*0.1, YXH*0.5, 0.16),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.38,0), ○(-.02,.09,.16))",
		Phonemes: []string{"/b/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 2,
		UppercaseOf: 0x42,
		MirrorOf:    0x64, // b↔d
	},

	// c — cung nhỏ mở phải
	{
		Codepoint: 0x63, Glyph: "c", Name: "Latin Small c",
		Script: "latin", Category: "letter",
		SDF: SArc(XCenter*0.2, YXH*0.5, 0.15, math.Pi*0.25, math.Pi*1.75),
		DNA:      "arc(.04,.09,.15,45°,315°)",
		Phonemes: []string{"/k/", "/s/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 3,
		UppercaseOf: 0x43,
	},

	// d — bụng trái + nét đứng cao bên phải
	{
		Codepoint: 0x64, Glyph: "d", Name: "Latin Small d",
		Script: "latin", Category: "letter",
		SDF: union(
			SCircle(XMidL*0.1, YXH*0.5, 0.16),
			SV(XMidR*1.3, YBot, YTop),
		),
		DNA:      "∪(○(-.02,.09,.16), ⌀(.18,-.38,.18,.38,0))",
		Phonemes: []string{"/d/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 4,
		UppercaseOf: 0x44,
		MirrorOf:    0x62, // d↔b
	},

	// e — cung tròn + nét ngang giữa
	{
		Codepoint: 0x65, Glyph: "e", Name: "Latin Small e",
		Script: "latin", Category: "letter",
		SDF: union(
			SArc(XCenter*0.2, YXH*0.5, 0.15, math.Pi*0.1, math.Pi*1.9),
			SH(-0.13, 0.17, YXH*0.5),
		),
		DNA:      "∪(arc(.04,.09,.15,18°,342°), ⌀(-.13,.09,.17,.09,0))",
		Phonemes: []string{"/iː/", "/ɛ/", "/e/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 5,
		UppercaseOf: 0x45,
		SameSound:   []uint32{0x03B5, 0x0435, 0x090F}, // ε е ए
	},

	// f — nét đứng + hook trên + nét ngang giữa
	{
		Codepoint: 0x66, Glyph: "f", Name: "Latin Small f",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XCenter*0.2, YBot, YTop*0.85),
			SArc(XCenter*0.2+0.12, YTop*0.75, 0.12, math.Pi*0.5, math.Pi*1.5),
			SH(-0.06, 0.22, YXH),
		),
		DNA:      "∪(⌀(.04,-.38,.04,.32,0), arc(.16,.33,.12,90°,270°), ⌀(-.06,.18,.22,.18,0))",
		Phonemes: []string{"/f/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 6,
		UppercaseOf: 0x46,
	},

	// g — two-story: vòng tròn + móc xuống phải
	{
		Codepoint: 0x67, Glyph: "g", Name: "Latin Small g",
		Script: "latin", Category: "letter",
		SDF: union(
			SCircle(XMidL*0.2, YXH*0.5, 0.15),
			SV(XMidR*1.3, YBase+0.05, YXH),
			SArc(XMidR*1.3-0.10, YBase+0.05, 0.10, 0, -math.Pi),
		),
		DNA:      "∪(○(-.03,.09,.15), ⌀(.18,-.32,.18,.18,0), arc(.08,-.32,.10,0°,-180°))",
		Phonemes: []string{"/ɡ/", "/dʒ/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 7,
		UppercaseOf: 0x47,
	},

	// h — nét đứng cao + arch phải
	{
		Codepoint: 0x68, Glyph: "h", Name: "Latin Small h",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SArc(XLeft+0.10, YXH*0.9, 0.10, math.Pi, 0),
			SV(XLeft+0.20, YBot, YXH*0.9),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.38,0), arc(-.18,.17,.10,180°,0°), ⌀(-.08,-.38,-.08,.17,0))",
		Phonemes: []string{"/h/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 8,
		UppercaseOf: 0x48,
	},

	// i — chấm + nét đứng ngắn
	{
		Codepoint: 0x69, Glyph: "i", Name: "Latin Small i",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XCenter, YBot, YXH),
			SDot(XCenter, YTop*0.65, 0.055),
		),
		DNA:      "∪(⌀(0,-.38,0,.18,0), ●(0,.29,.055))",
		Phonemes: []string{"/ɪ/", "/aɪ/", "/iː/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 9,
		UppercaseOf: 0x49,
	},

	// j — nét đứng ngắn + móc trái + chấm
	{
		Codepoint: 0x6A, Glyph: "j", Name: "Latin Small j",
		Script: "latin", Category: "letter",
		SDF: union(
			SHook(XCenter, YXH, YBase),
			SDot(XCenter, YTop*0.65, 0.055),
		),
		DNA:      "∪(hook(0,.18,-.32), ●(0,.29,.055))",
		Phonemes: []string{"/dʒ/", "/j/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 10,
		UppercaseOf: 0x4A,
	},

	// k — nét đứng + hai nét chéo nhỏ
	{
		Codepoint: 0x6B, Glyph: "k", Name: "Latin Small k",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YTop),
			SD(XLeft+0.03, YXH*0.6, XRight*0.9, YXH*1.1),
			SD(XLeft+0.03, YXH*0.6, XRight*0.9, YBot),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.38,0), ⌀(-.25,.11,.25,.20,0), ⌀(-.25,.11,.25,-.38,0))",
		Phonemes: []string{"/k/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 11,
		UppercaseOf: 0x4B,
	},

	// l — nét đứng đơn
	{
		Codepoint: 0x6C, Glyph: "l", Name: "Latin Small l",
		Script: "latin", Category: "letter",
		SDF: SV(XCenter, YBot, YTop),
		DNA:      "⌀(0,-.38,0,.38,0)",
		Phonemes: []string{"/l/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 12,
		UppercaseOf: 0x4C,
	},

	// m — ba nét đứng + hai arch
	{
		Codepoint: 0x6D, Glyph: "m", Name: "Latin Small m",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YXH),
			SArc(XLeft+0.09, YXH*0.9, 0.09, math.Pi, 0),
			SV(XLeft+0.18, YBot, YXH*0.9),
			SArc(XLeft+0.27, YXH*0.9, 0.09, math.Pi, 0),
			SV(XLeft+0.36, YBot, YXH*0.9),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.18,0), arc(-.19,.17,.09,180°,0°), ⌀(-.10,-.38,-.10,.17,0), arc(-.01,.17,.09,180°,0°), ⌀(.08,-.38,.08,.17,0))",
		Phonemes: []string{"/m/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 13,
		UppercaseOf: 0x4D,
	},

	// n — hai nét đứng + arch trên
	{
		Codepoint: 0x6E, Glyph: "n", Name: "Latin Small n",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YXH),
			SArc(XLeft+0.14, YXH*0.9, 0.14, math.Pi, 0),
			SV(XLeft+0.28, YBot, YXH*0.9),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.18,0), arc(-.14,.17,.14,180°,0°), ⌀(.0,-.38,.0,.17,0))",
		Phonemes: []string{"/n/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 14,
		UppercaseOf: 0x4E,
	},

	// o — vòng tròn nhỏ
	{
		Codepoint: 0x6F, Glyph: "o", Name: "Latin Small o",
		Script: "latin", Category: "letter",
		SDF: SCircle(XCenter, YXH*0.5, 0.16),
		DNA:      "○(0,.09,.16)",
		Phonemes: []string{"/ɒ/", "/oʊ/", "/ɔː/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 15,
		UppercaseOf: 0x4F,
		SameShape:   []uint32{0x25CB, 0x30AA, 0x039F}, // ○ オ Ο
	},

	// p — nét đứng xuống + bụng phải trên
	{
		Codepoint: 0x70, Glyph: "p", Name: "Latin Small p",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBase, YXH),
			SCircle(XMidL*0.1, YXH*0.5, 0.16),
		),
		DNA:      "∪(⌀(-.28,-.32,-.28,.18,0), ○(-.02,.09,.16))",
		Phonemes: []string{"/p/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 16,
		UppercaseOf: 0x50,
		MirrorOf:    0x71, // p↔q
	},

	// q — bụng trái + nét đứng xuống phải
	{
		Codepoint: 0x71, Glyph: "q", Name: "Latin Small q",
		Script: "latin", Category: "letter",
		SDF: union(
			SCircle(XMidL*0.1, YXH*0.5, 0.16),
			SV(XMidR*1.3, YBase, YXH),
		),
		DNA:      "∪(○(-.02,.09,.16), ⌀(.18,-.32,.18,.18,0))",
		Phonemes: []string{"/k/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 17,
		UppercaseOf: 0x51,
		MirrorOf:    0x70,
	},

	// r — nét đứng + shoulder nhỏ phải
	{
		Codepoint: 0x72, Glyph: "r", Name: "Latin Small r",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft, YBot, YXH),
			SArc(XLeft+0.08, YXH*0.85, 0.08, math.Pi, math.Pi*0.3),
		),
		DNA:      "∪(⌀(-.28,-.38,-.28,.18,0), arc(-.20,.16,.08,180°,54°))",
		Phonemes: []string{"/r/", "/ɹ/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 18,
		UppercaseOf: 0x52,
	},

	// s — hai cung nhỏ ngược chiều
	{
		Codepoint: 0x73, Glyph: "s", Name: "Latin Small s",
		Script: "latin", Category: "letter",
		SDF: union(
			SArc(XCenter*0.3, YXH*0.72, 0.10, math.Pi*0.25, math.Pi*1.2),
			SArc(XCenter*(-0.3), YXH*0.28, 0.10, math.Pi*1.25, math.Pi*2.1),
		),
		DNA:      "∪(arc(.06,.13,.10,45°,216°), arc(-.06,.05,.10,225°,378°))",
		Phonemes: []string{"/s/", "/z/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 19,
		UppercaseOf: 0x53,
	},

	// t — nét đứng + nét ngang giữa
	{
		Codepoint: 0x74, Glyph: "t", Name: "Latin Small t",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XCenter, YBot, YTop*0.75),
			SH(-0.16, 0.16, YXH*1.1),
		),
		DNA:      "∪(⌀(0,-.38,0,.28,0), ⌀(-.16,.20,.16,.20,0))",
		Phonemes: []string{"/t/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 20,
		UppercaseOf: 0x54,
	},

	// u — hai nét đứng + cung dưới
	{
		Codepoint: 0x75, Glyph: "u", Name: "Latin Small u",
		Script: "latin", Category: "letter",
		SDF: union(
			SV(XLeft+0.08, YBot*0.5, YXH),
			SV(XRight-0.08, YBot*0.5, YXH),
			SArc(XCenter, YBot*0.5, 0.16, math.Pi, 2*math.Pi),
		),
		DNA:      "∪(⌀(-.20,.0,-.20,.18,0), ⌀(.20,.0,.20,.18,0), arc(0,.0,.16,180°,360°))",
		Phonemes: []string{"/ʌ/", "/ʊ/", "/juː/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 21,
		UppercaseOf: 0x55,
	},

	// v — hai nét chéo nhỏ
	{
		Codepoint: 0x76, Glyph: "v", Name: "Latin Small v",
		Script: "latin", Category: "letter",
		SDF: union(
			SD(XLeft*0.8, YXH, XCenter, YBot*0.5),
			SD(XCenter, YBot*0.5, XRight*0.8, YXH),
		),
		DNA:      "∪(⌀(-.22,.18,.0,-.19,0), ⌀(.0,-.19,.22,.18,0))",
		Phonemes: []string{"/v/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 22,
		UppercaseOf: 0x56,
	},

	// w — v kép
	{
		Codepoint: 0x77, Glyph: "w", Name: "Latin Small w",
		Script: "latin", Category: "letter",
		SDF: union(
			SD(-0.28, YXH, -0.14, YBot*0.5),
			SD(-0.14, YBot*0.5, 0, YMid+0.06),
			SD(0, YMid+0.06, 0.14, YBot*0.5),
			SD(0.14, YBot*0.5, 0.28, YXH),
		),
		DNA:      "∪(⌀(-.28,.18,-.14,-.19,0), ⌀(-.14,-.19,0,.06,0), ⌀(0,.06,.14,-.19,0), ⌀(.14,-.19,.28,.18,0))",
		Phonemes: []string{"/w/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 23,
		UppercaseOf: 0x57,
	},

	// x — hai nét chéo cắt nhau
	{
		Codepoint: 0x78, Glyph: "x", Name: "Latin Small x",
		Script: "latin", Category: "letter",
		SDF: union(
			SD(XLeft*0.8, YXH, XRight*0.8, YBot*0.5),
			SD(XLeft*0.8, YBot*0.5, XRight*0.8, YXH),
		),
		DNA:      "∪(⌀(-.22,.18,.22,-.19,0), ⌀(-.22,-.19,.22,.18,0))",
		Phonemes: []string{"/ks/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 24,
		UppercaseOf: 0x58,
	},

	// y — hai nét chéo + đuôi xuống trái
	{
		Codepoint: 0x79, Glyph: "y", Name: "Latin Small y",
		Script: "latin", Category: "letter",
		SDF: union(
			SD(XLeft*0.8, YXH, XCenter, YMid*0.2),
			SD(XRight*0.8, YXH, XLeft*0.5, YBase),
		),
		DNA:      "∪(⌀(-.22,.18,.0,.04,0), ⌀(.22,.18,-.14,-.32,0))",
		Phonemes: []string{"/j/", "/aɪ/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 25,
		UppercaseOf: 0x59,
	},

	// z — nét ngang + chéo + nét ngang
	{
		Codepoint: 0x7A, Glyph: "z", Name: "Latin Small z",
		Script: "latin", Category: "letter",
		SDF: union(
			SH(XLeft*0.8, XRight*0.8, YXH),
			SD(XRight*0.8, YXH, XLeft*0.8, YBot*0.5),
			SH(XLeft*0.8, XRight*0.8, YBot*0.5),
		),
		DNA:      "∪(⌀(-.22,.18,.22,.18,0), ⌀(.22,.18,-.22,-.19,0), ⌀(-.22,-.19,.22,-.19,0))",
		Phonemes: []string{"/z/"},
		ISLLayer: 'L', ISLGroup: 'l', ISLType: 'a', ISLID: 26,
		UppercaseOf: 0x5A,
	},
}

// ── DIGITS 0–9 ────────────────────────────────────────────────
var LatinDigits = []*OlangChar{

	// 0 — hình bầu dục
	{
		Codepoint: 0x30, Glyph: "0", Name: "Digit Zero",
		Script: "latin", Category: "digit",
		SDF:      SCircle(XCenter, YMid+0.03, 0.24),
		DNA:      "○(0,.03,.24)",
		Phonemes: []string{"/zɪərəʊ/"},
		ISLLayer: 'L', ISLGroup: 'D', ISLType: 'd', ISLID: 0,
		SameShape:   []uint32{0x4F, 0x6F, 0x039F, 0x041E, 0x25CB}, // O o Ο О ○
		SameMeaning: []uint32{0x2205, 0x25CB},                       // ∅ ○
	},

	// 1 — nét đứng + tai nhỏ trên trái
	{
		Codepoint: 0x31, Glyph: "1", Name: "Digit One",
		Script: "latin", Category: "digit",
		SDF: union(
			SV(XCenter, YBot, YTop),
			SD(XCenter-0.10, YTop*0.75, XCenter, YTop),
		),
		DNA:      "∪(⌀(0,-.38,0,.38,0), ⌀(-.10,.28,.0,.38,0))",
		Phonemes: []string{"/wʌn/"},
		ISLLayer: 'L', ISLGroup: 'D', ISLType: 'd', ISLID: 1,
	},

	// 2 — cung trên + nét chéo + nét ngang dưới
	{
		Codepoint: 0x32, Glyph: "2", Name: "Digit Two",
		Script: "latin", Category: "digit",
		SDF: union(
			SArc(XCenter*0.3, YTop*0.65, 0.16, math.Pi*0.1, math.Pi*1.5),
			SD(XRight*0.85, YMid*0.3, XLeft*0.85, YBot),
			SH(XLeft*0.85, XRight*0.85, YBot),
		),
		DNA:      "∪(arc(.06,.24,.16,18°,270°), ⌀(.24,.06,-.24,-.38,0), ⌀(-.24,-.38,.24,-.38,0))",
		Phonemes: []string{"/tuː/"},
		ISLLayer: 'L', ISLGroup: 'D', ISLType: 'd', ISLID: 2,
	},

	// 3 — hai cung phải
	{
		Codepoint: 0x33, Glyph: "3", Name: "Digit Three",
		Script: "latin", Category: "digit",
		SDF: union(
			SArc(XCenter*0.1, YTop*0.6, 0.17, math.Pi*0.25, math.Pi*1.2),
			SArc(XCenter*0.1, YBot*0.6, 0.17, math.Pi*0.8, math.Pi*1.75),
		),
		DNA:      "∪(arc(.02,.23,.17,45°,216°), arc(.02,-.23,.17,144°,315°))",
		Phonemes: []string{"/θriː/"},
		ISLLayer: 'L', ISLGroup: 'D', ISLType: 'd', ISLID: 3,
	},

	// 4 — nét đứng + nét ngang + nét thẳng
	{
		Codepoint: 0x34, Glyph: "4", Name: "Digit Four",
		Script: "latin", Category: "digit",
		SDF: union(
			SV(XRight*0.7, YBot, YTop),
			SD(XRight*0.7, YTop, XLeft*0.8, YMid*0.3),
			SH(XLeft*0.8, XRight*0.85, YMid*0.3),
		),
		DNA:      "∪(⌀(.20,-.38,.20,.38,0), ⌀(.20,.38,-.22,.06,0), ⌀(-.22,.06,.24,.06,0))",
		Phonemes: []string{"/fɔː/"},
		ISLLayer: 'L', ISLGroup: 'D', ISLType: 'd', ISLID: 4,
	},

	// 5 — nét ngang trên + nét đứng + cung dưới phải
	{
		Codepoint: 0x35, Glyph: "5", Name: "Digit Five",
		Script: "latin", Category: "digit",
		SDF: union(
			SH(XLeft*0.8, XRight*0.8, YTop),
			SV(XLeft*0.8, YMid*0.3, YTop),
			SArc(XCenter*0.1, YBot*0.45, 0.18, math.Pi*1.7, math.Pi*0.9+2*math.Pi),
		),
		DNA:      "∪(⌀(-.22,.38,.22,.38,0), ⌀(-.22,.38,-.22,.06,0), arc(.02,-.17,.18,306°,450°))",
		Phonemes: []string{"/faɪv/"},
		ISLLayer: 'L', ISLGroup: 'D', ISLType: 'd', ISLID: 5,
	},

	// 6 — cung mở dưới + vòng tròn dưới
	{
		Codepoint: 0x36, Glyph: "6", Name: "Digit Six",
		Script: "latin", Category: "digit",
		SDF: union(
			SArc(XCenter*0.1, YTop*0.4, 0.24, math.Pi*0.1, math.Pi*1.1),
			SCircle(XCenter*0.1, YBot*0.45, 0.16),
		),
		DNA:      "∪(arc(.02,.15,.24,18°,198°), ○(.02,-.17,.16))",
		Phonemes: []string{"/sɪks/"},
		ISLLayer: 'L', ISLGroup: 'D', ISLType: 'd', ISLID: 6,
	},

	// 7 — nét ngang trên + nét chéo
	{
		Codepoint: 0x37, Glyph: "7", Name: "Digit Seven",
		Script: "latin", Category: "digit",
		SDF: union(
			SH(XLeft*0.8, XRight*0.8, YTop),
			SD(XRight*0.8, YTop, XLeft*0.3, YBot),
		),
		DNA:      "∪(⌀(-.22,.38,.22,.38,0), ⌀(.22,.38,-.08,-.38,0))",
		Phonemes: []string{"/sevn/"},
		ISLLayer: 'L', ISLGroup: 'D', ISLType: 'd', ISLID: 7,
	},

	// 8 — hai vòng tròn chồng
	{
		Codepoint: 0x38, Glyph: "8", Name: "Digit Eight",
		Script: "latin", Category: "digit",
		SDF: sUnion(0.04,
			SCircle(XCenter, YTop*0.55, 0.15),
			SCircle(XCenter, YBot*0.45, 0.18),
		),
		DNA:      "∪k.04(○(0,.21,.15), ○(0,-.17,.18))",
		Phonemes: []string{"/eɪt/"},
		ISLLayer: 'L', ISLGroup: 'D', ISLType: 'd', ISLID: 8,
		SameMeaning: []uint32{0x221E}, // ∞ (lộn ngang)
	},

	// 9 — vòng tròn trên + đuôi xuống phải
	{
		Codepoint: 0x39, Glyph: "9", Name: "Digit Nine",
		Script: "latin", Category: "digit",
		SDF: union(
			SCircle(XCenter*0.1, YTop*0.5, 0.16),
			SV(XMidR*1.3, YBot, YTop*0.5),
			SArc(XMidR*1.3, YBot*0.4, 0.10, 0, -math.Pi*0.8),
		),
		DNA:      "∪(○(.02,.19,.16), ⌀(.18,-.38,.18,.19,0), arc(.18,-.15,.10,0°,-144°))",
		Phonemes: []string{"/naɪn/"},
		ISLLayer: 'L', ISLGroup: 'D', ISLType: 'd', ISLID: 9,
	},
}

// AllLatin trả về toàn bộ Latin chars theo thứ tự
// A-Z, a-z, 0-9 = 62 ký tự
func AllLatin() []*OlangChar {
	out := make([]*OlangChar, 0, 62)
	out = append(out, LatinUpper...)
	out = append(out, LatinLower...)
	out = append(out, LatinDigits...)
	return out
}
