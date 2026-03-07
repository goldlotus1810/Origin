// internal/silk/unicode_tree.go
// Unicode Tree — đọc cấu trúc Unicode vào SilkGraph
// Không tạo lại, không tải UnicodeData.txt
// Chỉ seed cấu trúc block-level → ~140 nodes L1/L2 + opTable L3

package silk

import "github.com/goldlotus1810/HomeOS/internal/isl"

// UnicodeBlock mô tả một Unicode block
type UnicodeBlock struct {
	Name  string
	Cat   string // "scripts","math","geo","emoji","olang","punct"
	Layer int    // 1=supergroup, 2=family
	Grp   byte   // ISL Group byte
	ID    byte   // ISL ID trong group
	Glyph string // ký tự đại diện
}

// SeedUnicodeTree seed SilkGraph với cây Unicode
// Origin → L1 supergroups → L2 script families → L3 opTable chars
func SeedUnicodeTree(g *SilkGraph) {
	// ── Origin ────────────────────────────────────────────────────
	origin := &OlangNode{
		Addr:   addrOf('O', 'R', 'g', 0),
		Name:   "○", Glyph: "○", Cat: "origin", Layer: 0,
		Status: StatusQR,
	}
	g.AddNode(origin)

	// ── L1 Supergroups ────────────────────────────────────────────
	l1 := []struct {
		name, glyph, cat string
		lay, grp         byte
	}{
		{"Scripts", "Aa", "scripts", 'S', 1},
		{"Math", "∑", "math", 'M', 2},
		{"Geometry", "●", "geo", 'G', 3},
		{"Emoji", "🌍", "emoji", 'E', 4},
		{"Olang", "⌀", "olang", 'O', 5},
		{"Punctuation", "·", "punct", 'P', 6},
		{"Numbers", "①", "numbers", 'N', 7},
		{"Musical", "♫", "musical", 'U', 8},
	}
	l1Nodes := make(map[byte]*OlangNode)
	for i, s := range l1 {
		n := &OlangNode{
			Addr:   addrOf('K', s.lay, 'a', byte(i)),
			Name:   s.name, Glyph: s.glyph, Cat: s.cat, Layer: 1,
			Status: StatusQR,
		}
		g.AddNode(n)
		g.AddEdge(origin.Addr, n.Addr, OpMember)
		l1Nodes[s.grp] = n
	}

	// ── L2 Script families ────────────────────────────────────────
	scripts := []struct {
		name, glyph string
		id           byte
		parent       byte // L1 group byte
	}{
		// European
		{"Latin", "A", 0, 'S'}, {"Greek", "α", 1, 'S'}, {"Cyrillic", "А", 2, 'S'},
		{"Gothic", "𝔊", 3, 'S'}, {"Runic", "ᚠ", 4, 'S'},
		// West Asian
		{"Arabic", "ع", 10, 'S'}, {"Hebrew", "א", 11, 'S'}, {"Syriac", "ܐ", 12, 'S'},
		{"Cuneiform", "𒀭", 13, 'S'},
		// South Asian
		{"Devanagari", "अ", 20, 'S'}, {"Tamil", "அ", 21, 'S'}, {"Bengali", "অ", 22, 'S'},
		{"Tibetan", "ཀ", 23, 'S'},
		// East Asian
		{"CJK", "字", 30, 'S'}, {"Hangul", "한", 31, 'S'}, {"Hiragana", "あ", 32, 'S'},
		{"Katakana", "ア", 33, 'S'}, {"Bopomofo", "ㄅ", 34, 'S'},
		// SE Asian
		{"Thai", "ก", 40, 'S'}, {"Khmer", "ក", 41, 'S'}, {"Myanmar", "က", 42, 'S'},
		{"Lao", "ກ", 43, 'S'},
		// Other
		{"Armenian", "Ա", 50, 'S'}, {"Georgian", "Ა", 51, 'S'}, {"Ethiopic", "አ", 52, 'S'},
		{"Cherokee", "Ꭰ", 53, 'S'}, {"Mongolian", "ᠠ", 54, 'S'},
		// Math families
		{"MathOperators", "∑", 0, 'M'}, {"MathArrows", "→", 1, 'M'},
		{"MathLetters", "𝔸", 2, 'M'}, {"MathSets", "∈", 3, 'M'},
		{"MathRelations", "≡", 4, 'M'}, {"MathEnclosed", "①", 5, 'M'},
		// Geo families
		{"GeoPrimitives", "●", 0, 'G'}, {"GeoArrows", "↑", 1, 'G'},
		{"GeoBoxDrawing", "┼", 2, 'G'}, {"GeoBraille", "⠿", 3, 'G'},
		// Emoji families
		{"EmojiNature", "🌿", 0, 'E'}, {"EmojiPeople", "👤", 1, 'E'},
		{"EmojiObjects", "💡", 2, 'E'}, {"EmojiSymbols", "🛡", 3, 'E'},
		{"EmojiFlags", "🏳", 4, 'E'},
		// Olang opTable family
		{"OlangOps", "○", 0, 'O'},
		// Musical
		{"MusicNotes", "♩", 0, 'U'}, {"MusicNotation", "𝄞", 1, 'U'},
	}

	l2Nodes := make(map[string]*OlangNode)
	for _, s := range scripts {
		parent := l1Nodes[s.parent]
		if parent == nil {
			continue
		}
		n := &OlangNode{
			Addr:   addrOf('K', s.parent, 'b', s.id),
			Name:   s.name, Glyph: s.glyph, Cat: parent.Cat, Layer: 2,
			Status: StatusQR,
		}
		g.AddNode(n)
		g.AddEdge(parent.Addr, n.Addr, OpMember)
		l2Nodes[s.name] = n
	}

	// ── L3 — opTable chars (48 operators) ────────────────────────
	type opEntry struct {
		cp         rune
		name, cat  string
		parent     string // L2 family name
	}
	ops := []opEntry{
		// SDF primitives → GeoPrimitives
		{'●', "sphere", "geo", "GeoPrimitives"},
		{'⌀', "capsule", "geo", "GeoPrimitives"},
		{'□', "box", "geo", "GeoPrimitives"},
		{'◌', "void", "geo", "GeoPrimitives"},
		// SDF ops → MathOperators
		{'∪', "union", "math", "MathOperators"},
		{'∖', "subtract", "math", "MathOperators"},
		{'⊕', "hardunion", "math", "MathOperators"},
		{'⊗', "repeat", "math", "MathOperators"},
		{'↻', "rotate", "math", "MathArrows"},
		// Physics → MathOperators
		{'∇', "gradient", "math", "MathOperators"},
		{'·', "dot", "math", "MathOperators"},
		{'∫', "fbm", "math", "MathOperators"},
		{'∑', "sum", "math", "MathOperators"},
		{'∀', "forall", "math", "MathSets"},
		{'∃', "exists", "math", "MathSets"},
		{'∈', "member", "math", "MathSets"},
		{'≡', "equiv", "math", "MathRelations"},
		{'≈', "approx", "math", "MathRelations"},
		{'π', "pi", "math", "MathLetters"},
		{'∞', "infinity", "math", "MathLetters"},
		{'φ', "phi", "math", "MathLetters"},
		{'√', "sqrt", "math", "MathOperators"},
		// Lighting/render → EmojiSymbols
		{'☀', "sunlight", "geo", "EmojiNature"},
		{'👁', "raycast", "olang", "OlangOps"},
		{'🌍', "world", "olang", "OlangOps"},
		{'🧬', "gene", "olang", "OlangOps"},
		{'🛡', "security", "olang", "OlangOps"},
		{'💾', "ledger", "olang", "OlangOps"},
		{'🔄', "render", "olang", "OlangOps"},
		// Biology
		{'🌱', "grow", "olang", "EmojiNature"},
		{'♻', "cycle", "olang", "EmojiSymbols"},
		{'⚖', "balance", "olang", "EmojiSymbols"},
		{'🦠', "mutate", "olang", "EmojiNature"},
		// Audio
		{'♫', "oscillate", "musical", "MusicNotes"},
		{'♩', "note", "musical", "MusicNotes"},
		// System
		{'⚡', "decay", "olang", "EmojiSymbols"},
		{'∆', "delta", "math", "MathOperators"},
		// Cross operators
		{'×', "cross", "math", "MathOperators"},
	}

	for i, op := range ops {
		parent := l2Nodes[op.parent]
		if parent == nil {
			parent = l2Nodes["OlangOps"]
		}
		if parent == nil {
			continue
		}
		n := &OlangNode{
			Addr:   addrOf('K', 'F', 'f', byte(i)),
			Name:   op.name, Glyph: string(op.cp), Cat: op.cat, Layer: 3,
			Status: StatusQR,
			Atom:   &OlangAtom{Codepoint: op.cp, ISL: addrOf('K', 'F', 'f', byte(i))},
		}
		g.AddNode(n)
		g.AddEdge(parent.Addr, n.Addr, OpMember)
	}
}

// addrOf helper tạo ISL address nhanh
func addrOf(layer, group, typ, id byte) isl.Address {
	return isl.Address{Layer: layer, Group: group, Type: typ, ID: id}
}
