// internal/silk/ipa_edges.go
// IPA phoneme edges — Rule 4
// 44 IPA phonemes × 10 scripts → silk ♫
// Ví dụ: /a/ → Latin A, Greek α, Cyrillic а, Arabic ا, Hiragana あ

package silk

// ipaGroup là một phoneme và các scripts thể hiện nó
type ipaGroup struct {
	IPA     string   // IPA symbol
	Scripts []string // script family names
	Glyphs  []string // glyph đại diện của mỗi script
}

// ipaGroups: 44 IPA phonemes chính
var ipaGroups = []ipaGroup{
	// Vowels
	{"/a/", []string{"Latin", "Greek", "Cyrillic", "Arabic", "Hiragana", "Devanagari"}, []string{"a", "α", "а", "ا", "あ", "अ"}},
	{"/e/", []string{"Latin", "Greek", "Cyrillic", "Hiragana", "Devanagari"}, []string{"e", "ε", "е", "え", "ए"}},
	{"/i/", []string{"Latin", "Greek", "Cyrillic", "Arabic", "Hiragana", "Devanagari"}, []string{"i", "ι", "и", "ي", "い", "इ"}},
	{"/o/", []string{"Latin", "Greek", "Cyrillic", "Hiragana", "Devanagari"}, []string{"o", "ο", "о", "お", "ओ"}},
	{"/u/", []string{"Latin", "Greek", "Cyrillic", "Arabic", "Hiragana"}, []string{"u", "υ", "у", "و", "う"}},
	{"/ə/", []string{"Latin", "Cyrillic"}, []string{"ə", "ə"}},
	{"/æ/", []string{"Latin"}, []string{"æ"}},
	{"/ø/", []string{"Latin", "Greek"}, []string{"ø", "θ"}},

	// Consonants — stops
	{"/p/", []string{"Latin", "Greek", "Cyrillic", "Arabic", "Hiragana"}, []string{"p", "π", "п", "ب", "ぱ"}},
	{"/b/", []string{"Latin", "Greek", "Cyrillic", "Arabic"}, []string{"b", "β", "б", "ب"}},
	{"/t/", []string{"Latin", "Greek", "Cyrillic", "Arabic", "Hiragana"}, []string{"t", "τ", "т", "ت", "た"}},
	{"/d/", []string{"Latin", "Greek", "Cyrillic", "Arabic"}, []string{"d", "δ", "д", "د"}},
	{"/k/", []string{"Latin", "Greek", "Cyrillic", "Arabic", "Hiragana"}, []string{"k", "κ", "к", "ك", "か"}},
	{"/g/", []string{"Latin", "Greek", "Cyrillic"}, []string{"g", "γ", "г"}},

	// Consonants — fricatives
	{"/f/", []string{"Latin", "Greek", "Cyrillic"}, []string{"f", "φ", "ф"}},
	{"/v/", []string{"Latin", "Cyrillic"}, []string{"v", "в"}},
	{"/s/", []string{"Latin", "Greek", "Cyrillic", "Arabic", "Hiragana"}, []string{"s", "σ", "с", "س", "さ"}},
	{"/z/", []string{"Latin", "Cyrillic", "Arabic"}, []string{"z", "з", "ز"}},
	{"/h/", []string{"Latin", "Greek", "Cyrillic", "Arabic", "Hiragana"}, []string{"h", "η", "х", "ه", "は"}},
	{"/ʃ/", []string{"Latin", "Arabic"}, []string{"ʃ", "ش"}},
	{"/x/", []string{"Latin", "Greek", "Cyrillic", "Arabic"}, []string{"x", "χ", "х", "خ"}},

	// Consonants — nasals
	{"/m/", []string{"Latin", "Greek", "Cyrillic", "Arabic", "Hiragana"}, []string{"m", "μ", "м", "م", "ま"}},
	{"/n/", []string{"Latin", "Greek", "Cyrillic", "Arabic", "Hiragana", "Devanagari"}, []string{"n", "ν", "н", "ن", "な", "न"}},
	{"/ŋ/", []string{"Latin", "Hiragana"}, []string{"ŋ", "ん"}},

	// Consonants — liquids
	{"/l/", []string{"Latin", "Greek", "Cyrillic", "Arabic", "Hiragana"}, []string{"l", "λ", "л", "ل", "ら"}},
	{"/r/", []string{"Latin", "Greek", "Cyrillic", "Arabic", "Hiragana"}, []string{"r", "ρ", "р", "ر", "ら"}},

	// Math "phonemes" — symbols với âm đọc giống nhau
	{"/sigma/", []string{"Greek", "MathLetters"}, []string{"σ", "𝜎"}},
	{"/pi/", []string{"Greek", "MathLetters", "OlangOps"}, []string{"π", "𝜋", "π"}},
	{"/phi/", []string{"Greek", "MathLetters", "OlangOps"}, []string{"φ", "𝜙", "φ"}},
	{"/delta/", []string{"Greek", "MathLetters", "OlangOps"}, []string{"δ", "𝛿", "∆"}},
	{"/omega/", []string{"Greek", "MathLetters"}, []string{"ω", "𝜔"}},
	{"/alpha/", []string{"Greek", "MathLetters"}, []string{"α", "𝛼"}},
	{"/lambda/", []string{"Greek", "MathLetters"}, []string{"λ", "𝜆"}},
}

// ApplyIPAEdges thêm silk ♫ edges giữa các ký tự cùng âm
func ApplyIPAEdges(g *SilkGraph) int {
	added := 0

	// Index nodes theo glyph
	glyphIndex := make(map[string][]*OlangNode)
	for _, n := range g.AllNodes() {
		if n.Glyph != "" {
			glyphIndex[n.Glyph] = append(glyphIndex[n.Glyph], n)
		}
		// Cũng index theo script family name
		if n.Layer == 2 {
			glyphIndex["@"+n.Name] = append(glyphIndex["@"+n.Name], n)
		}
	}

	// Với mỗi IPA group → link tất cả nodes trong group via ♫
	for _, grp := range ipaGroups {
		var groupNodes []*OlangNode
		for _, glyph := range grp.Glyphs {
			if ns, ok := glyphIndex[glyph]; ok {
				groupNodes = append(groupNodes, ns...)
			}
		}
		// Thêm script-level nodes (L2) nếu tìm thấy
		for _, scriptName := range grp.Scripts {
			if ns, ok := glyphIndex["@"+scriptName]; ok {
				_ = ns // script nodes đã được link qua tree
			}
		}
		// Link pairs trong group via ♫
		for i := 0; i < len(groupNodes); i++ {
			for j := i + 1; j < len(groupNodes); j++ {
				a, b := groupNodes[i], groupNodes[j]
				if a.Addr.Uint64() != b.Addr.Uint64() {
					g.AddEdge(a.Addr, b.Addr, OpPhonetic)
					g.AddEdge(b.Addr, a.Addr, OpPhonetic)
					added += 2
				}
			}
		}
	}
	return added
}
