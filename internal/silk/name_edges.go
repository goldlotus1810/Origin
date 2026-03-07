// internal/silk/name_edges.go
// 5 Rules tạo silk edges tự động từ tên Unicode nodes
// R1: cùng "LETTER X" → silk ≡ (cross-script equivalence)
// R2: cùng block prefix → silk ∈
// R3: cùng semantic keyword → concept node → silk ≡
// R4: IPA phoneme → silk ♫ (trong ipa_edges.go)
// R5: số học "ONE" → ISL[Num] → silk ≡

package silk

import "strings"

// ApplyNameEdges áp dụng 5 rules lên SilkGraph đã seed
// Gọi sau SeedUnicodeTree()
func ApplyNameEdges(g *SilkGraph) int {
	nodes := g.AllNodes()
	added := 0

	// Nhóm nodes theo tên (lowercase) cho lookup nhanh
	byKeyword := make(map[string][]*OlangNode)
	for _, n := range nodes {
		if n.Name == "" {
			continue
		}
		key := strings.ToLower(n.Name)
		byKeyword[key] = append(byKeyword[key], n)
	}

	// ── Rule 1 — cross-script glyph equivalence ──────────────────
	// Nodes có cùng Glyph nhưng khác script → silk ≡
	byGlyph := make(map[string][]*OlangNode)
	for _, n := range nodes {
		if n.Glyph != "" && n.Layer >= 2 {
			byGlyph[n.Glyph] = append(byGlyph[n.Glyph], n)
		}
	}
	for _, group := range byGlyph {
		if len(group) < 2 {
			continue
		}
		for i := 0; i < len(group); i++ {
			for j := i + 1; j < len(group); j++ {
				if group[i].Cat != group[j].Cat {
					g.AddEdge(group[i].Addr, group[j].Addr, OpEquiv)
					g.AddEdge(group[j].Addr, group[i].Addr, OpEquiv)
					added += 2
				}
			}
		}
	}

	// ── Rule 2 — same block prefix → silk ∈ ─────────────────────
	// Nodes có cùng Cat → đã có qua L1→L2→L3 tree edges
	// Thêm cross-family edges cho cat giống nhau
	byCat := make(map[string][]*OlangNode)
	for _, n := range nodes {
		if n.Layer == 3 {
			byCat[n.Cat] = append(byCat[n.Cat], n)
		}
	}
	for _, group := range byCat {
		if len(group) < 2 {
			continue
		}
		// Không thêm tất cả pairs (quá nhiều) — chỉ link theo chuỗi
		for i := 0; i+1 < len(group); i++ {
			g.AddEdge(group[i].Addr, group[i+1].Addr, OpSimilar)
			added++
		}
	}

	// ── Rule 3 — semantic keyword → concept node ─────────────────
	added += applySemanticConcepts(g, nodes)

	// ── Rule 5 — số học equivalence ──────────────────────────────
	added += applyNumericEquiv(g, nodes)

	// ── Composition edges — opTable pairs ────────────────────────
	added += applyCompositionEdges(g, nodes)

	return added
}

// semanticGroups là các từ khoá → tên concept node
var semanticGroups = map[string][]string{
	"union":      {"union", "smoothunion", "hardunion"},
	"geometry":   {"sphere", "capsule", "box", "void", "torus"},
	"light":      {"sunlight", "decay", "oscillate"},
	"memory":     {"ledger", "gene", "world"},
	"safety":     {"security", "gate"},
	"math_calc":  {"sum", "gradient", "delta", "sqrt", "cross", "dot"},
	"logic":      {"forall", "exists", "member", "equiv", "approx"},
	"bio":        {"grow", "mutate", "cycle", "balance"},
	"render":     {"raycast", "render", "rotate", "repeat"},
}

func applySemanticConcepts(g *SilkGraph, nodes []*OlangNode) int {
	added := 0
	nameIndex := make(map[string]*OlangNode)
	for _, n := range nodes {
		nameIndex[strings.ToLower(n.Name)] = n
	}

	for _, members := range semanticGroups {
		var group []*OlangNode
		for _, m := range members {
			if n, ok := nameIndex[m]; ok {
				group = append(group, n)
			}
		}
		if len(group) < 2 {
			continue
		}
		// Ring: n[0]→n[1]→...→n[last]→n[0] via OpCompose
		for i := 0; i < len(group); i++ {
			next := group[(i+1)%len(group)]
			g.AddEdge(group[i].Addr, next.Addr, OpCompose)
			added++
		}
	}
	return added
}

// applyNumericEquiv: nodes có tên liên quan số → silk ≡
func applyNumericEquiv(g *SilkGraph, nodes []*OlangNode) int {
	added := 0
	numKeywords := []string{"sum", "pi", "infinity", "phi", "sqrt"}
	var mathNodes []*OlangNode
	for _, n := range nodes {
		name := strings.ToLower(n.Name)
		for _, kw := range numKeywords {
			if strings.Contains(name, kw) {
				mathNodes = append(mathNodes, n)
				break
			}
		}
	}
	// Link math constants với silk ≡
	for i := 0; i+1 < len(mathNodes); i++ {
		g.AddEdge(mathNodes[i].Addr, mathNodes[i+1].Addr, OpEquiv)
		added++
	}
	return added
}

// applyCompositionEdges: link SDF ops theo pipeline logic
// sphere + union → composition chain
func applyCompositionEdges(g *SilkGraph, nodes []*OlangNode) int {
	added := 0
	nameIdx := make(map[string]*OlangNode)
	for _, n := range nodes {
		nameIdx[strings.ToLower(n.Name)] = n
	}

	// Pipeline chains
	chains := [][]string{
		{"sphere", "union", "gradient"},          // ●→∪→∇ SDF pipeline
		{"fbm", "union", "sunlight"},              // terrain pipeline
		{"raycast", "gradient", "dot", "sunlight"}, // shading pipeline
		{"gene", "ledger", "render"},              // data pipeline
		{"grow", "cycle", "balance"},              // bio pipeline
	}
	for _, chain := range chains {
		for i := 0; i+1 < len(chain); i++ {
			from, fok := nameIdx[chain[i]]
			to, tok := nameIdx[chain[i+1]]
			if fok && tok {
				g.AddEdge(from.Addr, to.Addr, OpCompose)
				added++
			}
		}
	}
	return added
}
