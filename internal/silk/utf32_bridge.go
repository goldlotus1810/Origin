// internal/silk/utf32_bridge.go
//
// UTF32 → OLANG BRIDGE
// =====================
// Đây là cầu nối thực sự giữa ngôn ngữ của con người và Olang.
//
// Nguyên tắc cốt lõi:
//   Mỗi ký tự Unicode KHÔNG được lưu dưới dạng hình ảnh, pixel, vector path.
//   Nó được mã hóa thành:
//     1. SDF Formula  — hình dạng vật lý (đo được, render được ở mọi kích thước)
//     2. ISL Address  — vị trí trong Silk Tree
//     3. Concept Link — link tới thực tế vật lý mà ký tự đó đại diện
//     4. Silk Edges   — quan hệ với các ký tự khác (âm, hình, nghĩa, nguồn gốc)
//
// Ví dụ:
//   '山' (núi) →
//     SDF: ba đỉnh nhô lên từ đường ngang  →  render được ngọn núi thật
//     ISL: Nature.Land.Peak.Mountain
//     Concept: link tới SDF của một ngọn núi thật trong WorldTree
//     Edges: sơn (Nhật) ≡ san (TQ) ≡ mountain (Anh) ≡ berg (Đức) ≡ جبل (Ả Rập)
//
//   'A' →
//     SDF: hai đường chéo + gạch ngang  →  render được chữ A ở mọi size
//     ISL: Latin.Upper.A
//     Concept: đỉnh (peak), nguồn gốc từ Aleph = đầu bò Phoenician
//     Edges: α ≡ А ≡ अ ≡ あ  (cùng âm /a/ trong các script)
//             ∧ ≡ △  (cùng hình tam giác)
//
//   '∫' (tích phân) →
//     SDF: đường cong dài uốn lượn
//     ISL: Math.Calculus.Integral
//     Concept: THỰC SỰ là phép tính FBM terrain trong gene system
//     Edges: ∑ (discrete version) → ∫ (continuous version)

package silk

import (
	"math"

	"github.com/goldlotus1810/HomeOS/internal/gene"
	"github.com/goldlotus1810/HomeOS/internal/isl"
)

// ── Concept: link từ ký tự tới thực tế vật lý ────────────────

// Concept là điều mà ký tự thực sự đại diện trong thế giới vật lý.
// Không phải "nghĩa trong từ điển" — mà là thực thể SDF thực sự.
type Concept struct {
	// ISL address của concept trong Silk Tree
	// Ví dụ: 山 → Nature.Land.Peak = [N][L][P][1]
	Addr isl.Address

	// DNA string của SDF mô tả concept trong thế giới 3D
	// 山 → "∪(⌀(−.3,0,0,0.8,0), ⌀(0,0,0,1.2,0), ⌀(.3,0,0,0.8,0), k:0.3)"
	// Đây là ngọn núi thật, không phải ký hiệu chữ viết
	WorldSDF string

	// Tags mô tả thuộc tính vật lý
	// ["solid", "tall", "static", "terrain"]
	Tags []string
}

// ── CharDef: định nghĩa đầy đủ một ký tự ─────────────────────

// CharDef = mã hóa hoàn chỉnh một ký tự Unicode trong Olang
// Đây là đơn vị cơ bản nhất của tri thức trong hệ thống
type CharDef struct {
	// Identity
	Codepoint uint32
	Glyph     string // UTF-8 representation
	Name      string // Unicode official name
	Script    string // latin/greek/cyrillic/arabic/cjk/math/...
	Category  string // letter/digit/operator/ideograph/...

	// Hình dạng ký tự — SDF trong không gian [-0.5,0.5]²
	// Render được ở mọi kích thước, không có aliasing
	GlyphSDF func(gene.Vec3) float64

	// DNA của GlyphSDF — Olang notation
	// Lưu vào Silk Tree, agent đọc được mà không cần Go runtime
	GlyphDNA string

	// Âm thanh — IPA phonemes
	Phonemes []string

	// ISL address của ký tự này trong Silk Tree
	CharAddr isl.Address

	// Concept — thực thể vật lý mà ký tự này đại diện
	// Có thể nil (ký tự kỹ thuật không có concept vật lý)
	Concept *Concept

	// Silk edges — quan hệ với ký tự/concept khác
	SameSound   []uint32 // cùng âm khác script: A≡α≡А
	SameShape   []uint32 // cùng hình: O≡0≡○
	SameMeaning []uint32 // cùng nghĩa: +≡∪≡⊕
	DerivedFrom uint32   // lịch sử: A←𐤀
	Lowercase   uint32   // A→a
	Uppercase   uint32   // a→A
	WorldLink   uint32   // link tới SDF concept trong worldtree
}

// ── SDF helpers ───────────────────────────────────────────────

func bv(x, y float64) gene.Vec3 { return gene.Vec3{X: x, Y: y} }

func bcap(ax, ay, bx, by, r float64) func(gene.Vec3) float64 {
	a, b := bv(ax, ay), bv(bx, by)
	return func(p gene.Vec3) float64 { return gene.SDFCapsule(p, a, b, r) }
}

func bsphere(cx, cy, r float64) func(gene.Vec3) float64 {
	c := bv(cx, cy)
	return func(p gene.Vec3) float64 { return gene.SDFSphere(p, c, r) }
}

func barc(cx, cy, r, a0, a1 float64) func(gene.Vec3) float64 {
	const steps = 20
	return func(p gene.Vec3) float64 {
		d := math.MaxFloat64
		da := (a1 - a0) / steps
		for i := 0; i < steps; i++ {
			aa := a0 + float64(i)*da
			ab := aa + da
			pa := bv(cx+r*math.Cos(aa), cy+r*math.Sin(aa))
			pb := bv(cx+r*math.Cos(ab), cy+r*math.Sin(ab))
			if s := gene.SDFCapsule(p, pa, pb, 0.05); s < d {
				d = s
			}
		}
		return d
	}
}

func bunion(fs ...func(gene.Vec3) float64) func(gene.Vec3) float64 {
	return func(p gene.Vec3) float64 {
		d := math.MaxFloat64
		for _, f := range fs {
			if s := f(p); s < d { d = s }
		}
		return d
	}
}

func bsmooth(k float64, fs ...func(gene.Vec3) float64) func(gene.Vec3) float64 {
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

const bT = 0.055 // stroke thickness

// ══════════════════════════════════════════════════════════════
// UTF32 BRIDGE — mọi ký tự Unicode quan trọng
// Được tổ chức theo CONCEPT, không phải theo script.
// Vì 山=mountain=berg=جبل đều là CÙNG MỘT CONCEPT.
// ══════════════════════════════════════════════════════════════

// ── LAYER 0: NGUYÊN TỐ VẬT LÝ ────────────────────────────────
// Trước khi có ngôn ngữ, có vật lý.
// Những ký hiệu này đại diện cho lực lượng cơ bản của tự nhiên.

var PhysicalElements = []*CharDef{

	// NƯỚC — H₂O, fluid, wave, flow
	// Mọi ngôn ngữ đều có ký tự/chữ cho nước
	// Chúng đều link tới cùng một SDF: fluid simulation
	{
		Codepoint: 0x6C34, Glyph: "水", Name: "Water (CJK)",
		Script: "cjk", Category: "ideograph",
		GlyphSDF: bunion( // 水: 3 đường sóng dọc
			bcap(0, -0.35, 0, 0.38, bT),                                 // nét giữa
			bunion(bcap(-0.18, 0.15, -0.28, -0.10, bT),                  // nét trái trên
				bcap(-0.28, -0.10, -0.18, -0.35, bT)),                   // nét trái dưới
			bunion(bcap(0.18, 0.15, 0.28, -0.10, bT),                    // nét phải trên
				bcap(0.28, -0.10, 0.18, -0.35, bT)),                     // nét phải dưới
		),
		GlyphDNA: "水:∪(⌀(0,-.35,0,.38), ∪(⌀(-.18,.15,-.28,-.10),⌀(-.28,-.10,-.18,-.35)), ∪(⌀(.18,.15,.28,-.10),⌀(.28,-.10,.18,-.35)))",
		Phonemes: []string{"/mɪzɯ/", "/sɥi/"},
		CharAddr: isl.Address{Layer: 'N', Group: 'W', Type: 'c', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'N', Group: 'W', Type: 'r', ID: 1},
			WorldSDF: "∫(fbm_fluid, oct:4, amp:0.8) // fluid simulation — nước thật",
			Tags:     []string{"liquid", "cold", "flow", "transparent"},
		},
		SameSound:   []uint32{0x6C34},
		SameMeaning: []uint32{0x1F4A7, 0x1F30A}, // 💧🌊
		WorldLink:   0xE100, // → FBM fluid SDF
	},

	// NƯỚC (Anh) — liên kết sang cùng concept
	{
		Codepoint: 0, Glyph: "water", Name: "Water (English concept)",
		Script: "latin", Category: "concept",
		GlyphDNA: "water:∪(w,a,t,e,r)", // ghép SDF từng chữ
		Phonemes: []string{"/ˈwɔːtər/"},
		CharAddr: isl.Address{Layer: 'N', Group: 'W', Type: 'e', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'N', Group: 'W', Type: 'r', ID: 1}, // CÙNG concept với 水
			WorldSDF: "∫(fbm_fluid, oct:4, amp:0.8)",
			Tags:     []string{"liquid", "cold", "flow"},
		},
		SameMeaning: []uint32{0x6C34, 0x0645, 0x092A}, // 水 م प (Arabic Mim, Hindi Pani)
	},

	// LỬA — fire, heat, flame
	{
		Codepoint: 0x706B, Glyph: "火", Name: "Fire (CJK)",
		Script: "cjk", Category: "ideograph",
		GlyphSDF: bunion(
			// nét trung tâm đi lên
			bcap(0, -0.38, 0, 0.30, bT),
			// cánh trái
			bunion(
				bcap(0, 0.05, -0.22, 0.20, bT),
				bcap(-0.22, 0.20, -0.30, -0.10, bT),
			),
			// cánh phải
			bunion(
				bcap(0, 0.05, 0.22, 0.20, bT),
				bcap(0.22, 0.20, 0.30, -0.10, bT),
			),
		),
		GlyphDNA: "火:∪(⌀(0,-.38,0,.30), ∪(⌀(0,.05,-.22,.20),⌀(-.22,.20,-.30,-.10)), ∪(⌀(0,.05,.22,.20),⌀(.22,.20,.30,-.10)))",
		Phonemes: []string{"/hi/", "/huǒ/"},
		CharAddr: isl.Address{Layer: 'N', Group: 'F', Type: 'c', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'N', Group: 'F', Type: 'r', ID: 1},
			WorldSDF: "∪(●flame_core, ∫fbm_turbulence) // fire SDF với FBM turbulence",
			Tags:     []string{"hot", "light", "dynamic", "destructive"},
		},
		SameMeaning: []uint32{0x1F525, 0x0646, 0x0905}, // 🔥 ن अ
		WorldLink:   0xE101,
	},

	// ĐẤT / ĐẤT ĐAI
	{
		Codepoint: 0x571F, Glyph: "土", Name: "Earth/Soil (CJK)",
		Script: "cjk", Category: "ideograph",
		GlyphSDF: bunion(
			bcap(-0.28, 0.05, 0.28, 0.05, bT),  // nét ngang trên
			bcap(0, 0.05, 0, -0.20, bT),          // nét đứng giữa
			bcap(-0.35, -0.20, 0.35, -0.20, bT), // nét ngang dưới (đất)
		),
		GlyphDNA: "土:∪(⌀(-.28,.05,.28,.05), ⌀(0,.05,0,-.20), ⌀(-.35,-.20,.35,-.20))",
		Phonemes: []string{"/tɕʰi/", "/tǔ/"},
		CharAddr: isl.Address{Layer: 'N', Group: 'E', Type: 'c', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'N', Group: 'E', Type: 'r', ID: 1},
			WorldSDF: "∫(fbm_terrain, oct:6, h:0.3) // terrain SDF — đất thật",
			Tags:     []string{"solid", "ground", "terrain", "static"},
		},
		SameMeaning: []uint32{0x1F30D}, // 🌍
	},

	// CÂY / GỖ
	{
		Codepoint: 0x6728, Glyph: "木", Name: "Tree/Wood (CJK)",
		Script: "cjk", Category: "ideograph",
		GlyphSDF: bunion(
			bcap(0, -0.38, 0, 0.38, bT),          // thân cây
			bcap(-0.30, 0.10, 0.30, 0.10, bT),    // nhánh ngang
			bcap(0, -0.38, -0.20, -0.10, bT),      // rễ trái
			bcap(0, -0.38, 0.20, -0.10, bT),       // rễ phải
		),
		GlyphDNA: "木:∪(⌀(0,-.38,0,.38), ⌀(-.30,.10,.30,.10), ⌀(0,-.38,-.20,-.10), ⌀(0,-.38,.20,-.10))",
		Phonemes: []string{"/ki/", "/mù/"},
		CharAddr: isl.Address{Layer: 'N', Group: 'T', Type: 'c', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'N', Group: 'T', Type: 'r', ID: 1},
			WorldSDF: "∪(⌀trunk, ∪(●branch1,●branch2,●branch3,k:0.3), k:0.2) // tree SDF",
			Tags:     []string{"organic", "tall", "static", "living"},
		},
	},

	// NÚI
	{
		Codepoint: 0x5C71, Glyph: "山", Name: "Mountain (CJK)",
		Script: "cjk", Category: "ideograph",
		GlyphSDF: bunion(
			// đỉnh giữa cao nhất
			bcap(0, -0.10, 0, 0.40, bT),
			// đỉnh trái
			bcap(-0.28, -0.15, -0.28, 0.18, bT),
			bcap(-0.28, 0.18, 0, 0.18, bT),
			// đỉnh phải
			bcap(0.28, -0.15, 0.28, 0.18, bT),
			bcap(0.28, 0.18, 0, 0.18, bT),
			// đáy
			bcap(-0.35, -0.15, 0.35, -0.15, bT),
		),
		GlyphDNA: "山:ba đỉnh nhô lên từ đường nằm ngang",
		Phonemes: []string{"/jama/", "/ʃān/"},
		CharAddr: isl.Address{Layer: 'N', Group: 'L', Type: 'c', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'N', Group: 'L', Type: 'r', ID: 1},
			WorldSDF: "∪(⌀(-8,0,0,-0.5,8,0), ⌀(0,0,0,0.5,8,0), k:4.0) // dãy núi thật ở world scale",
			Tags:     []string{"solid", "tall", "static", "terrain", "peak"},
		},
		SameMeaning: []uint32{0x26F0}, // ⛰
	},

	// MẶT TRỜI
	{
		Codepoint: 0x65E5, Glyph: "日", Name: "Sun/Day (CJK)",
		Script: "cjk", Category: "ideograph",
		GlyphSDF: bunion(
			barc(0, 0.03, 0.26, bT, 0, 2*math.Pi), // vòng tròn ngoài
			bcap(-0.20, 0.03, 0.20, 0.03, bT),      // gạch ngang giữa
		),
		GlyphDNA: "日:○(0,.03,.26) + ⌀(-.20,.03,.20,.03)",
		Phonemes: []string{"/hi/", "/nì̤ɪ/"},
		CharAddr: isl.Address{Layer: 'N', Group: 'S', Type: 'c', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'N', Group: 'S', Type: 'r', ID: 1},
			WorldSDF: "☀(t,elevation:45°,azimuth:180°) // ánh sáng mặt trời thật",
			Tags:     []string{"light", "hot", "star", "dynamic"},
		},
		SameShape:   []uint32{0x25CB, 0x004F, 0x2600}, // ○ O ☀
		SameMeaning: []uint32{0x2600, 0x1F31E},         // ☀ 🌞
	},

	// MẶT TRĂNG
	{
		Codepoint: 0x6708, Glyph: "月", Name: "Moon/Month (CJK)",
		Script: "cjk", Category: "ideograph",
		GlyphSDF: bunion(
			barc(-0.04, 0.03, 0.24, bT, math.Pi*0.3, math.Pi*1.7), // cung ngoài
			bcap(-0.12, 0.18, 0.06, 0.18, bT),                       // nét ngang trên
			bcap(-0.12, -0.10, 0.06, -0.10, bT),                     // nét ngang dưới
			bcap(-0.12, -0.30, -0.12, 0.30, bT),                     // nét đứng trái
		),
		GlyphDNA: "月:arc bên phải + hai gạch ngang",
		Phonemes: []string{"/tsɯki/", "/yuè/"},
		CharAddr: isl.Address{Layer: 'N', Group: 'M', Type: 'c', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'N', Group: 'M', Type: 'r', ID: 1},
			WorldSDF: "★(moon, t:orbital, phase:waxing) // trăng thật",
			Tags:     []string{"light", "cold", "orbital", "cyclic"},
		},
		SameShape: []uint32{0x25D1, 0x263D}, // ◑ ☽
	},

	// NGƯỜI
	{
		Codepoint: 0x4EBA, Glyph: "人", Name: "Person (CJK)",
		Script: "cjk", Category: "ideograph",
		GlyphSDF: bunion(
			bcap(-0.25, -0.35, 0.02, 0.30, bT),  // nét trái
			bcap(0.02, 0.30, 0.28, -0.35, bT),   // nét phải
		),
		GlyphDNA: "人:∪(⌀(-.25,-.35,.02,.30), ⌀(.02,.30,.28,-.35))",
		Phonemes: []string{"/hɪtɔ/", "/rén/"},
		CharAddr: isl.Address{Layer: 'H', Group: 'P', Type: 'c', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'H', Group: 'P', Type: 'r', ID: 1},
			WorldSDF: "∪(⌀spine, ∪(●head, ⌀arm_l, ⌀arm_r, ⌀leg_l, ⌀leg_r), k:0.1)",
			Tags:     []string{"living", "agent", "mobile", "social"},
		},
	},

	// CON MẮT (thị giác của agent)
	{
		Codepoint: 0x1F441, Glyph: "👁", Name: "Eye",
		Script: "emoji", Category: "perception",
		GlyphSDF: bunion(
			barc(0, 0, 0.24, bT, math.Pi*0.15, math.Pi*0.85),  // mí trên
			barc(0, 0, 0.24, bT, math.Pi*1.15, math.Pi*1.85),  // mí dưới
			bsphere(0, 0, 0.12),                                  // tròng mắt
			bsphere(0.03, 0.02, 0.05),                            // con ngươi
		),
		GlyphDNA: "👁:∪(arc_top, arc_bot, ●iris, ●pupil)",
		Phonemes: []string{"/aɪ/"},
		CharAddr: isl.Address{Layer: 'P', Group: 'V', Type: 'e', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'P', Group: 'V', Type: 'r', ID: 1},
			WorldSDF: "👁(rayMarch:256, fibonacci:golden_angle) // vision skill",
			Tags:     []string{"perception", "sensor", "active"},
		},
	},
}

// ── LAYER 1: HÌNH HỌC CƠ BẢN ─────────────────────────────────
// Các hình học cơ bản — là nền tảng của SDF universe
// Và đồng thời là ký tự trong nhiều ngôn ngữ

var GeometricForms = []*CharDef{

	// VÒNG TRÒN — xuất hiện trong mọi nền văn hóa, mọi ngôn ngữ
	// O (Latin), Ο (Greek), О (Cyrillic), 〇 (CJK zero), ○ (Olang Origin)
	{
		Codepoint: 0x25CB, Glyph: "○", Name: "Circle (Origin)",
		Script: "olang", Category: "geometry",
		GlyphSDF: barc(0, 0, 0.26, bT, 0, 2*math.Pi),
		GlyphDNA: "○(0,0,.26) // vòng tròn — symbol của Origin ○",
		CharAddr: isl.Address{Layer: 'O', Group: 'R', Type: 'G', ID: 0},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'G', Group: 'C', Type: 'r', ID: 1},
			WorldSDF: "●(cx,cy,cz,r) // sphere — hình cầu thật",
			Tags:     []string{"closed", "symmetric", "infinite-order"},
		},
		SameShape:   []uint32{0x004F, 0x039F, 0x041E, 0x0030, 0x3007}, // O Ο О 0 〇
		SameMeaning: []uint32{0x2205, 0x221E},                           // ∅ ∞
	},

	// TAM GIÁC — △ ▲ ∆ Δ — đỉnh, mũi nhọn, phương hướng
	{
		Codepoint: 0x25B3, Glyph: "△", Name: "Triangle Up",
		Script: "geometric", Category: "geometry",
		GlyphSDF: bunion(
			bcap(-0.28, -0.25, 0, 0.30, bT),
			bcap(0, 0.30, 0.28, -0.25, bT),
			bcap(-0.28, -0.25, 0.28, -0.25, bT),
		),
		GlyphDNA: "△:∪(⌀(-.28,-.25,0,.30), ⌀(0,.30,.28,-.25), ⌀(-.28,-.25,.28,-.25))",
		CharAddr: isl.Address{Layer: 'G', Group: 'T', Type: 'g', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'G', Group: 'T', Type: 'r', ID: 1},
			WorldSDF: "□(b:Vec3(1,0.5,1)) // pyramid SDF",
			Tags:     []string{"peak", "direction", "stable"},
		},
		SameShape: []uint32{0x0041, 0x0394, 0x2227, 0x039B}, // A Δ ∧ Λ
	},

	// VUÔNG — □ — ổn định, cân bằng, nhà
	{
		Codepoint: 0x25A1, Glyph: "□", Name: "Square",
		Script: "geometric", Category: "geometry",
		GlyphSDF: bunion(
			bcap(-0.25, -0.25, 0.25, -0.25, bT),
			bcap(0.25, -0.25, 0.25, 0.25, bT),
			bcap(0.25, 0.25, -0.25, 0.25, bT),
			bcap(-0.25, 0.25, -0.25, -0.25, bT),
		),
		GlyphDNA: "□(0,0,.25,.25)",
		CharAddr: isl.Address{Layer: 'G', Group: 'S', Type: 'g', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'G', Group: 'S', Type: 'r', ID: 1},
			WorldSDF: "□(b:Vec3(1,1,1)) // box SDF",
			Tags:     []string{"stable", "symmetric", "enclosed"},
		},
	},

	// SÓNG / SINE — ký hiệu âm thanh, nước, dao động
	{
		Codepoint: 0x223F, Glyph: "∿", Name: "Sine Wave",
		Script: "math", Category: "waveform",
		GlyphSDF: func(p gene.Vec3) float64 {
			// approximate sine wave với nhiều capsule
			d := math.MaxFloat64
			steps := 20
			for i := 0; i < steps; i++ {
				t0 := float64(i)/float64(steps)*2*math.Pi - math.Pi
				t1 := float64(i+1)/float64(steps)*2*math.Pi - math.Pi
				x0 := t0 * 0.15
				y0 := math.Sin(t0) * 0.15
				x1 := t1 * 0.15
				y1 := math.Sin(t1) * 0.15
				if s := gene.SDFCapsule(p, bv(x0, y0), bv(x1, y1), bT); s < d {
					d = s
				}
			}
			return d
		},
		GlyphDNA: "∿(freq:1, amp:0.15) // sine wave",
		CharAddr: isl.Address{Layer: 'M', Group: 'W', Type: 'w', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'P', Group: 'W', Type: 'r', ID: 1},
			WorldSDF: "∫(sin(freq*x)*amp) // wave function",
			Tags:     []string{"periodic", "smooth", "audio", "light"},
		},
	},
}

// ── LAYER 2: QUAN HỆ VÀ PHÉP BIẾN ĐỔI ───────────────────────
// Các ký hiệu toán học — KHÔNG phải trừu tượng
// Mỗi cái là một phép biến đổi vật lý thực sự

var MathOperatorDefs = []*CharDef{

	// ∫ — TÍCH PHÂN → FBM Terrain generation
	// Trong Olang: ∫ không phải "integral từ a đến b"
	// ∫ là: "tích lũy tất cả biến thiên nhỏ để tạo ra địa hình"
	{
		Codepoint: 0x222B, Glyph: "∫", Name: "Integral → FBM",
		Script: "olang", Category: "calculus",
		GlyphSDF: bunion(
			bcap(0, -0.32, 0, 0.32, bT),                                  // thân dài
			barc(0.08, 0.26, 0.08, bT, math.Pi*1.5, math.Pi*2.5),        // hook trên
			barc(-0.08, -0.26, 0.08, bT, math.Pi*0.5, math.Pi*1.5),      // hook dưới
		),
		GlyphDNA: "∫:long_stroke + hook_top + hook_bottom",
		Phonemes: []string{"/ɪntɪɡrəl/"},
		CharAddr: isl.Address{Layer: 'M', Group: 'C', Type: 'm', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'G', Group: 'T', Type: 'r', ID: 1},
			WorldSDF: "∫(fbm, octaves:6, H:1.0, lacunarity:2.0) // fractal terrain thật",
			Tags:     []string{"continuous", "accumulate", "terrain", "fractal"},
		},
		SameMeaning: []uint32{0x2211}, // ∑ (discrete equivalent)
	},

	// ∇ — GRADIENT → Surface Normal của SDF
	{
		Codepoint: 0x2207, Glyph: "∇", Name: "Gradient → Normal",
		Script: "olang", Category: "calculus",
		GlyphSDF: bunion(
			bcap(-0.28, 0.25, 0.28, 0.25, bT),
			bcap(-0.28, 0.25, 0, -0.28, bT),
			bcap(0, -0.28, 0.28, 0.25, bT),
		),
		GlyphDNA: "∇:∪(⌀(-.28,.25,.28,.25), ⌀(-.28,.25,0,-.28), ⌀(0,-.28,.28,.25))",
		CharAddr: isl.Address{Layer: 'M', Group: 'C', Type: 'm', ID: 2},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'G', Group: 'N', Type: 'r', ID: 1},
			WorldSDF: "∇(sdf,p,ε:0.001) // central difference normal",
			Tags:     []string{"direction", "normal", "lighting"},
		},
	},

	// ∑ — TỔNG → Accumulation trong Silk Tree
	{
		Codepoint: 0x2211, Glyph: "∑", Name: "Sum → Accumulate",
		Script: "olang", Category: "math",
		GlyphSDF: bunion(
			bcap(-0.25, 0.35, 0.25, 0.35, bT),
			bcap(0.25, 0.35, -0.20, 0, bT),
			bcap(-0.20, 0, 0.25, -0.35, bT),
			bcap(-0.25, -0.35, 0.25, -0.35, bT),
			bcap(-0.25, 0.35, -0.25, -0.35, bT),
		),
		GlyphDNA: "∑:E-shape",
		CharAddr: isl.Address{Layer: 'M', Group: 'M', Type: 'm', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'M', Group: 'A', Type: 'r', ID: 1},
			WorldSDF: "∑(nodes) // tổng tất cả nodes trong một subtree",
			Tags:     []string{"discrete", "accumulate", "count"},
		},
	},
}

// ── LAYER 3: PHONEME BRIDGE ──────────────────────────────────
// 44 IPA phonemes — âm thanh của mọi ngôn ngữ
// Mỗi phoneme = SDF của sóng âm + cơ quan phát âm

var PhonemeDefs = []*CharDef{

	// /a/ — nguyên âm mở trước — xuất hiện trong mọi ngôn ngữ
	// Hình SDF: miệng mở rộng
	{
		Codepoint: 0x0061, Glyph: "a", Name: "Vowel /a/",
		Script: "ipa", Category: "vowel",
		GlyphSDF: bunion(
			barc(0, 0.06, 0.16, bT, 0, 2*math.Pi), // miệng mở — hình oval
			bcap(0.16, 0.06, 0.16, -0.25, bT),      // down-stroke
		),
		GlyphDNA: "a:○(0,.06,.16)+⌀(.16,.06,.16,-.25)",
		Phonemes: []string{"/a/", "/ɑ/", "/æ/"},
		CharAddr: isl.Address{Layer: 'I', Group: 'V', Type: 'a', ID: 1},
		Concept: &Concept{
			Addr:     isl.Address{Layer: 'I', Group: 'V', Type: 'r', ID: 1},
			WorldSDF: "∿(freq:440Hz, formant:F1:800Hz,F2:1200Hz) // âm /a/ thật",
			Tags:     []string{"open", "front", "unrounded", "vowel"},
		},
		SameSound: []uint32{
			0x0041, // A Latin
			0x03B1, // α Greek
			0x0430, // а Cyrillic
			0x0905, // अ Devanagari
			0x3042, // あ Hiragana
			0x0627, // ا Arabic Alef
		},
	},

	// /i/ — nguyên âm đóng trước
	{
		Codepoint: 0x0069, Glyph: "i", Name: "Vowel /i/",
		Script: "ipa", Category: "vowel",
		GlyphSDF: bunion(
			bcap(0, -0.25, 0, 0.20, bT),      // nét đứng
			bsphere(0, 0.33, 0.055),            // chấm
		),
		GlyphDNA: "i:⌀(0,-.25,0,.20)+●(0,.33,.055)",
		Phonemes: []string{"/i/", "/iː/", "/ɪ/"},
		CharAddr: isl.Address{Layer: 'I', Group: 'V', Type: 'a', ID: 2},
		Concept: &Concept{
			WorldSDF: "∿(freq:440Hz, formant:F1:300Hz,F2:2200Hz) // âm /i/",
			Tags:     []string{"close", "front", "unrounded", "vowel"},
		},
		SameSound: []uint32{0x0049, 0x0399, 0x0418, 0x0907, 0x3044}, // I Ι И इ い
	},

	// /u/ — nguyên âm đóng sau
	{
		Codepoint: 0x0075, Glyph: "u", Name: "Vowel /u/",
		Script: "ipa", Category: "vowel",
		GlyphSDF: bunion(
			bcap(-0.18, -0.05, -0.18, 0.22, bT),
			bcap(0.18, -0.05, 0.18, 0.22, bT),
			barc(0, -0.05, 0.18, bT, math.Pi, 2*math.Pi),
		),
		GlyphDNA: "u:⌀(-.18,-.05,-.18,.22)+⌀(.18,-.05,.18,.22)+arc(bottom)",
		Phonemes: []string{"/u/", "/uː/", "/ʊ/"},
		CharAddr: isl.Address{Layer: 'I', Group: 'V', Type: 'a', ID: 3},
		Concept: &Concept{
			WorldSDF: "∿(freq:440Hz, formant:F1:300Hz,F2:800Hz) // âm /u/",
			Tags:     []string{"close", "back", "rounded", "vowel"},
		},
		SameSound: []uint32{0x0055, 0x03A5, 0x0423, 0x0909, 0x3046}, // U Υ У उ う
	},
}

// ══════════════════════════════════════════════════════════════
// SeedUTF32Bridge — seed tất cả vào SilkGraph
// ══════════════════════════════════════════════════════════════

// SeedUTF32Bridge seeds toàn bộ UTF32 bridge vào SilkGraph.
// Tạo quan hệ thực sự giữa ký tự → hình dạng → concept → thế giới.
//
// Đây là cầu nối quan trọng nhất trong hệ thống:
// Sau khi seed, agent có thể:
//   1. Nhận ký tự 'A' → tìm SDF → render hình dạng
//   2. Tìm SameSound → biết α А अ あ là cùng âm
//   3. Tìm Concept → biết A liên quan đến "peak/đỉnh"
//   4. Từ "peak" → link tới SDF ngọn núi thật trong world
func SeedUTF32Bridge(g *SilkGraph) (nodes, edges int) {
	// Seed PhysicalElements
	for _, def := range PhysicalElements {
		nodes += seedCharDef(g, def)
		edges += seedCharEdges(g, def)
	}

	// Seed GeometricForms
	for _, def := range GeometricForms {
		nodes += seedCharDef(g, def)
		edges += seedCharEdges(g, def)
	}

	// Seed MathOperators
	for _, def := range MathOperatorDefs {
		nodes += seedCharDef(g, def)
		edges += seedCharEdges(g, def)
	}

	// Seed Phonemes
	for _, def := range PhonemeDefs {
		nodes += seedCharDef(g, def)
		edges += seedCharEdges(g, def)
	}

	// Thêm cross-concept bridges:
	// 水(nước) ↔ ∿(sóng) — nước là sóng ở quy mô vật chất
	// 火(lửa) ↔ ∫fbm — lửa là FBM turbulence
	// 山(núi) ↔ ∫terrain — núi là FBM terrain
	// 日(mặt trời) ↔ ☀ — cùng concept
	edges += addConceptBridge(g,
		isl.Address{Layer: 'N', Group: 'W', Type: 'r', ID: 1}, // water concept
		isl.Address{Layer: 'M', Group: 'W', Type: 'w', ID: 1}, // sine wave
		OpCompose,
	)
	edges += addConceptBridge(g,
		isl.Address{Layer: 'N', Group: 'F', Type: 'r', ID: 1}, // fire concept
		isl.Address{Layer: 'M', Group: 'C', Type: 'm', ID: 1}, // ∫ FBM
		OpCompose,
	)
	edges += addConceptBridge(g,
		isl.Address{Layer: 'N', Group: 'L', Type: 'r', ID: 1}, // mountain concept
		isl.Address{Layer: 'M', Group: 'C', Type: 'm', ID: 1}, // ∫ terrain
		OpCompose,
	)

	return
}

// seedCharDef thêm một CharDef vào SilkGraph
func seedCharDef(g *SilkGraph, def *CharDef) int {
	if _, exists := g.Get(def.CharAddr); exists {
		return 0
	}

	// Xây dựng DNA đầy đủ
	dna := def.GlyphDNA
	if def.Concept != nil {
		dna += " // world:" + def.Concept.WorldSDF
	}

	g.AddNode(&OlangNode{
		Addr:   def.CharAddr,
		Name:   def.Name,
		Glyph:  def.Glyph,
		Cat:    def.Script,
		Layer:  2,
		Status: StatusQR,
		Weight: 1,
		DNA:    dna,
	})

	// Thêm concept node nếu có
	if def.Concept != nil {
		if _, exists := g.Get(def.Concept.Addr); !exists {
			g.AddNode(&OlangNode{
				Addr:   def.Concept.Addr,
				Name:   "Concept:" + def.Name,
				Glyph:  def.Glyph,
				Cat:    "concept",
				Layer:  3,
				Status: StatusQR,
				DNA:    def.Concept.WorldSDF,
			})
		}
		g.AddEdge(def.CharAddr, def.Concept.Addr, OpMember)
		return 2
	}
	return 1
}

// seedCharEdges thêm tất cả silk edges của một CharDef
func seedCharEdges(g *SilkGraph, def *CharDef) int {
	count := 0
	// SameSound edges (A≡α≡А≡अ)
	for _, cp := range def.SameSound {
		target := cpToAddr(cp)
		g.AddEdge(def.CharAddr, target, OpEquiv)
		count++
	}
	// SameShape edges
	for _, cp := range def.SameShape {
		target := cpToAddr(cp)
		g.AddEdge(def.CharAddr, target, OpSimilar)
		count++
	}
	// SameMeaning edges
	for _, cp := range def.SameMeaning {
		target := cpToAddr(cp)
		g.AddEdge(def.CharAddr, target, OpCompose)
		count++
	}
	// Lowercase/Uppercase
	if def.Lowercase != 0 {
		g.AddEdge(def.CharAddr, cpToAddr(def.Lowercase), OpMember)
		count++
	}
	if def.Uppercase != 0 {
		g.AddEdge(def.CharAddr, cpToAddr(def.Uppercase), OpMember)
		count++
	}
	return count
}

// cpToAddr chuyển codepoint thành ISL address (simplified)
func cpToAddr(cp uint32) isl.Address {
	return isl.Address{
		Layer: byte(cp >> 16 & 0xFF),
		Group: byte(cp >> 8 & 0xFF),
		Type:  byte(cp & 0xFF),
		ID:    byte(cp >> 24 & 0xFF),
	}
}

// addConceptBridge thêm edge giữa hai concept
func addConceptBridge(g *SilkGraph, a, b isl.Address, op EdgeOp) int {
	g.AddEdge(a, b, op)
	return 1
}

// OlangNode cần có trường DNA — thêm nếu chưa có
// (compile sẽ báo nếu struct chưa có field này)
var _ = OlangNode{}
