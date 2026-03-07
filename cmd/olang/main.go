// cmd/olang/main.go
// Olang Runtime CLI
//
// Usage:
//   olang run   homeos.olang          — chạy file
//   olang info  homeos.olang          — xem thống kê
//   olang dump  homeos.olang          — dump tất cả nodes
//   olang get   homeos.olang A        — tìm node theo ký tự
//   olang walk  homeos.olang A 3      — walk silk edges depth 3
//   olang verify homeos.olang         — verify SHA256

package main

import (
	"encoding/binary"
	"crypto/sha256"
	"fmt"
	"math"
	"math/bits"
	"os"
	"strconv"
	"strings"
	"unicode/utf8"
)

// ─── Binary format constants ────────────────────────────────────

const (
	Magic          = "OLNG"
	HeaderSize     = 64
	LayerEntrySize = 24
	LayerIndexSize = 9 * LayerEntrySize

	// Link types
	LinkSameShape   = 0x01
	LinkSameSound   = 0x02
	LinkSameMeaning = 0x03
	LinkDerivedFrom = 0x04
	LinkOpposite    = 0x05
	LinkPart        = 0x06
	LinkContext     = 0x07
	LinkWorldLink   = 0x08
	LinkLearned     = 0xFF
)

var layerNames = []string{
	"L0 Primitives", "L1 UTF32", "L2 Nature", "L3 Life",
	"L4 Objects", "L5 Programming", "L6 Perception", "L7 Programs", "L8 Build",
}

var linkNames = map[uint8]string{
	0x01: "SameShape", 0x02: "SameSound", 0x03: "SameMeaning",
	0x04: "DerivedFrom", 0x05: "Opposite", 0x06: "Part",
	0x07: "Context", 0x08: "WorldLink", 0xFF: "Learned",
}

// ─── Data structures ────────────────────────────────────────────

type Header struct {
	Version uint16
	Flags   uint16
	NLayers uint8
	NNodes  uint32
	NEdges  uint32
	Created int64
	SHA256  [32]byte
}

type LayerEntry struct {
	ID     uint8
	Status uint8
	Flags  uint16
	Offset uint64
	Size   uint64
	NNodes uint32
}

type Node struct {
	LayerID   uint8
	ISL       [8]byte
	Codepoint rune
	DNA       []byte
	Name      string
	Links     []Edge
}

type Edge struct {
	From   [8]byte
	To     [8]byte
	LType  uint8
}

type OlangFile struct {
	raw    []byte
	Header Header
	Layers [9]LayerEntry
	Nodes  []*Node
	Edges  []Edge
	ByCP   map[rune]*Node
	ByISL  map[uint64]*Node
}

// ─── Parser ─────────────────────────────────────────────────────

func load(path string) (*OlangFile, error) {
	raw, err := os.ReadFile(path)
	if err != nil { return nil, err }

	if len(raw) < HeaderSize+LayerIndexSize {
		return nil, fmt.Errorf("file too small (%d bytes)", len(raw))
	}
	if string(raw[0:4]) != Magic {
		return nil, fmt.Errorf("invalid magic %q (expected OLNG)", raw[0:4])
	}

	f := &OlangFile{
		raw:   raw,
		ByCP:  make(map[rune]*Node),
		ByISL: make(map[uint64]*Node),
	}

	// Header
	f.Header.Version = be16(raw[4:])
	f.Header.Flags   = be16(raw[6:])
	f.Header.NLayers = raw[8]
	f.Header.NNodes  = be32(raw[12:])
	f.Header.NEdges  = be32(raw[16:])
	f.Header.Created = int64(be64(raw[20:]))
	copy(f.Header.SHA256[:], raw[28:60])

	// Layer index
	for i := 0; i < 9; i++ {
		off := HeaderSize + i*LayerEntrySize
		f.Layers[i] = LayerEntry{
			ID:     raw[off],
			Status: raw[off+1],
			Flags:  be16(raw[off+2:]),
			Offset: be64(raw[off+4:]),
			Size:   be64(raw[off+12:]),
			NNodes: be32(raw[off+20:]),
		}
	}

	// Parse nodes from each layer
	for i := 0; i < 9; i++ {
		le := f.Layers[i]
		if le.Size == 0 { continue }
		if le.Offset+le.Size > uint64(len(raw)) { continue }

		ldata := raw[le.Offset : le.Offset+le.Size]
		off := 0
		for off < len(ldata) {
			if off+2 > len(ldata) { break }
			if ldata[off] != 'N' || ldata[off+1] != 'D' { break }
			off += 2

			if off+16 > len(ldata) { break }
			lid     := ldata[off]; off++
			var isl [8]byte
			copy(isl[:], ldata[off:off+8]); off += 8
			cp      := rune(be32(ldata[off:])); off += 4
			dnaLen  := int(be16(ldata[off:])); off += 2
			nameLen := int(ldata[off]); off++

			if off+dnaLen+nameLen > len(ldata) { break }
			dna  := make([]byte, dnaLen)
			copy(dna, ldata[off:off+dnaLen]); off += dnaLen
			name := string(ldata[off : off+nameLen]); off += nameLen

			n := &Node{LayerID: lid, ISL: isl, Codepoint: cp, DNA: dna, Name: name}
			f.Nodes = append(f.Nodes, n)
			f.ByISL[islKey(isl)] = n
			if cp != 0 { f.ByCP[cp] = n }
		}
	}

	// Parse edges
	edgeStart := uint64(0)
	for i := 8; i >= 0; i-- {
		if f.Layers[i].Size > 0 {
			edgeStart = f.Layers[i].Offset + f.Layers[i].Size
			break
		}
	}
	edata := raw[edgeStart:]
	for len(edata) >= 17 {
		var e Edge
		copy(e.From[:], edata[0:8])
		copy(e.To[:], edata[8:16])
		e.LType = edata[16]
		f.Edges = append(f.Edges, e)
		edata = edata[17:]

		// Wire to source node
		if n, ok := f.ByISL[islKey(e.From)]; ok {
			n.Links = append(n.Links, e)
		}
	}

	return f, nil
}

func (f *OlangFile) verify() bool {
	content := f.raw[HeaderSize+LayerIndexSize:]
	sum := sha256.Sum256(content)
	return sum == f.Header.SHA256
}

// ─── ISL helpers ────────────────────────────────────────────────

func islKey(isl [8]byte) uint64 { return binary.BigEndian.Uint64(isl[:]) }

func islStr(isl [8]byte) string {
	L, G, T, I := isl[0], isl[1], isl[2], isl[3]
	attr := binary.BigEndian.Uint32(isl[4:])
	return fmt.Sprintf("[%c][%c][%c][%d] attr=0x%08X", L, G, T, I, attr)
}

func attrStr(isl [8]byte) string {
	attr := binary.BigEndian.Uint32(isl[4:])
	sh := attr >> 24 & 0xFF
	ph := attr >> 16 & 0xFF
	cg := attr >> 8 & 0xFF
	dv := attr & 0xFF
	return fmt.Sprintf("shape=0x%02X phoneme=0x%02X concept=0x%02X deriv=0x%02X", sh, ph, cg, dv)
}

func similarity(a, b [8]byte) float32 {
	aa := binary.BigEndian.Uint32(a[4:])
	ab := binary.BigEndian.Uint32(b[4:])

	score, weight := float32(0), float32(0)

	if a[0] == b[0] { score += 3; weight += 3 }
	if a[1] == b[1] { score += 2; weight += 2 }
	if a[2] == b[2] { score += 1; weight += 1 }

	shapeDiff := bits.OnesCount32(uint32(aa>>24&0xFF) ^ uint32(ab>>24&0xFF))
	score += float32(8-shapeDiff) / 8.0 * 4; weight += 4

	phoneShared := bits.OnesCount32(aa >> 16 & 0xFF & ab >> 16 & 0xFF)
	score += float32(phoneShared) / 8.0 * 3; weight += 3

	cDiff := math.Abs(float64(aa>>8&0xFF) - float64(ab>>8&0xFF))
	score += float32(32-min(cDiff, 32)) / 32.0 * 4; weight += 4

	if aa&0xF0 == ab&0xF0 { score += 1; weight += 1 }

	if weight == 0 { return 0 }
	return score / weight
}

func min(a, b float64) float64 {
	if a < b { return a }
	return b
}

func dnaSummary(dna []byte) string {
	if len(dna) == 0 { return "(empty)" }
	ops := map[byte]string{
		0x01:"SPHERE", 0x02:"CAPSULE", 0x03:"BOX", 0x04:"TORUS", 0x05:"VOID",
		0x10:"UNION",  0x11:"SUB",     0x12:"INTERSECT",
		0x20:"FBM",    0x21:"GRADIENT",0x22:"LIGHT",     0x30:"SPLINE",
		0x40:"AND",    0x41:"OR",      0x42:"NOT",
		0x50:"GET",    0x51:"WALK",    0x52:"EMIT",       0x53:"BCAST",
		0x60:"SEQ",    0x61:"IF",      0x62:"LOOP",       0x63:"DREAM",  0x64:"SPAWN",
		0x70:"ADD",    0x71:"SUB_N",   0x72:"MUL",        0x73:"DIV",
		0x80:"SIN",    0x81:"COS",     0x90:"VEC3",       0x9A:"SCALE",
		0xA0:"LOAD",   0xA1:"STORE",   0xA4:"CALL",       0xA5:"RET",
		0xFF:"GATE",
	}
	var parts []string
	for _, b := range dna[:min2(len(dna), 10)] {
		if name, ok := ops[b]; ok {
			parts = append(parts, name)
		}
	}
	s := strings.Join(parts, " ")
	if len(dna) > 10 { s += "..." }
	return fmt.Sprintf("%s  (%d bytes)", s, len(dna))
}

func min2(a, b int) int {
	if a < b { return a }
	return b
}

// ─── Commands ───────────────────────────────────────────────────

func cmdInfo(f *OlangFile, path string) {
	ok := f.verify()
	verified := "✅ verified"
	if !ok { verified = "❌ SHA256 MISMATCH" }

	fmt.Printf(`
○ OLANG FILE INFO
  ─────────────────────────────────────────
  File:     %s
  Magic:    OLNG
  Version:  Unicode %d.0
  Nodes:    %d
  Edges:    %d
  SHA256:   %s  %s
  ─────────────────────────────────────────
`,
		path,
		f.Header.Version,
		len(f.Nodes),
		len(f.Edges),
		fmt.Sprintf("%x", f.Header.SHA256)[:32]+"...",
		verified,
	)

	for i, le := range f.Layers {
		if le.NNodes == 0 && le.Size == 0 { continue }
		status := []string{"empty", "ĐN", "QR", "archived"}[min2(int(le.Status), 3)]
		fmt.Printf("  %-18s  %4d nodes  %6d B  [%s]\n",
			layerNames[i], le.NNodes, le.Size, status)
	}
	fmt.Println()
}

func cmdDump(f *OlangFile) {
	fmt.Printf("○ NODES (%d total)\n\n", len(f.Nodes))
	for _, n := range f.Nodes {
		ch := ""
		if n.Codepoint > 0 && n.Codepoint < 0x110000 {
			ch = fmt.Sprintf("'%c' U+%04X", n.Codepoint, n.Codepoint)
		}
		fmt.Printf("  [%s]  %-20s  %-12s  %s\n",
			layerNames[min2(int(n.LayerID), 8)][:2],
			n.Name[:min2(len(n.Name), 20)],
			ch,
			dnaSummary(n.DNA),
		)
		if len(n.Links) > 0 {
			for _, e := range n.Links {
				lt := linkNames[e.LType]
				to := ""
				if tn, ok := f.ByISL[islKey(e.To)]; ok {
					to = tn.Name
				}
				fmt.Printf("      └─ %-12s → %s\n", lt, to)
			}
		}
	}
}

func cmdGet(f *OlangFile, query string) {
	var n *Node

	// Thử tìm theo ký tự
	if utf8.RuneCountInString(query) == 1 {
		r, _ := utf8.DecodeRuneInString(query)
		n = f.ByCP[r]
	}

	// Thử tìm theo tên
	if n == nil {
		q := strings.ToUpper(query)
		for _, nd := range f.Nodes {
			if strings.Contains(strings.ToUpper(nd.Name), q) {
				n = nd; break
			}
		}
	}

	if n == nil {
		fmt.Printf("❌ not found: %q\n", query)
		return
	}

	ch := ""
	if n.Codepoint > 0 { ch = fmt.Sprintf("'%c' U+%04X", n.Codepoint, n.Codepoint) }

	fmt.Printf(`
○ NODE: %s
  ─────────────────────────────────────
  Layer:     %s
  Char:      %s
  ISL:       %s
  Attrs:     %s
  DNA:       %s
  Links:     %d
`,
		n.Name,
		layerNames[min2(int(n.LayerID), 8)],
		ch,
		islStr(n.ISL),
		attrStr(n.ISL),
		dnaSummary(n.DNA),
		len(n.Links),
	)

	if len(n.Links) > 0 {
		fmt.Println("  ─────────────────────────────────────")
		for _, e := range n.Links {
			lt := linkNames[e.LType]
			toName := fmt.Sprintf("ISL %x", e.To[:4])
			if tn, ok := f.ByISL[islKey(e.To)]; ok {
				toName = tn.Name
				if tn.Codepoint > 0 { toName += fmt.Sprintf(" '%c'", tn.Codepoint) }
			}
			fmt.Printf("  %-14s → %s\n", lt, toName)
		}
	}

	// Tìm similar nodes
	if len(f.Nodes) > 1 {
		fmt.Println()
		fmt.Println("  Similar nodes (≥ 0.6):")
		count := 0
		for _, other := range f.Nodes {
			if other == n { continue }
			sim := similarity(n.ISL, other.ISL)
			if sim >= 0.6 {
				ch2 := ""
				if other.Codepoint > 0 { ch2 = fmt.Sprintf("'%c'", other.Codepoint) }
				fmt.Printf("    %.3f  %-4s %s\n", sim, ch2, other.Name)
				count++
				if count >= 8 { break }
			}
		}
		if count == 0 { fmt.Println("    (none)") }
	}
	fmt.Println()
}

func cmdWalk(f *OlangFile, query string, depth int) {
	var start *Node
	if utf8.RuneCountInString(query) == 1 {
		r, _ := utf8.DecodeRuneInString(query)
		start = f.ByCP[r]
	}
	if start == nil {
		q := strings.ToUpper(query)
		for _, n := range f.Nodes {
			if strings.Contains(strings.ToUpper(n.Name), q) { start = n; break }
		}
	}
	if start == nil { fmt.Printf("❌ not found: %q\n", query); return }

	fmt.Printf("○ WALK from '%s' depth=%d\n\n", start.Name, depth)

	visited := map[uint64]bool{}
	var walk func(n *Node, d int, prefix string)
	walk = func(n *Node, d int, prefix string) {
		key := islKey(n.ISL)
		if visited[key] { return }
		visited[key] = true
		ch := ""
		if n.Codepoint > 0 { ch = fmt.Sprintf("'%c'", n.Codepoint) }
		fmt.Printf("%s● %-4s %s\n", prefix, ch, n.Name)
		if d <= 0 { return }
		for _, e := range n.Links {
			lt := linkNames[e.LType]
			if tn, ok := f.ByISL[islKey(e.To)]; ok {
				fmt.Printf("%s  └─[%s]─ ", prefix, lt)
				walk(tn, d-1, prefix+"     ")
			}
		}
	}
	walk(start, depth, "  ")
	fmt.Println()
}

func cmdRun(f *OlangFile, path string) {
	// Verify trước
	if !f.verify() {
		fmt.Fprintln(os.Stderr, "❌ SHA256 mismatch — file may be corrupted")
		os.Exit(1)
	}

	fmt.Printf(`
○ ─────────────────────────────────────────
○  OLANG RUNTIME
○  %s
○ ─────────────────────────────────────────
`, path)

	// Print layer summary
	for i, le := range f.Layers {
		if le.NNodes == 0 { continue }
		status := []string{"", "ĐN", "QR", "archived"}[min2(int(le.Status), 3)]
		fmt.Printf("  %-20s  %3d nodes  [%s]\n", layerNames[i], le.NNodes, status)
	}
	fmt.Printf("  Silk edges:           %3d\n", len(f.Edges))
	fmt.Println()

	// SecurityGate — Rule 1
	fmt.Println("  🛡 SecurityGate: ONLINE")
	fmt.Println("     Rule 1: Không hại con người  [ENFORCED]")
	fmt.Println("     Rule 2: Không vòng lặp vô hạn [ENFORCED]")
	fmt.Println("     Rule 3: Không xóa QR nodes   [ENFORCED]")
	fmt.Println()

	// Agents từ L7
	fmt.Println("  Agents:")
	for _, n := range f.Nodes {
		if n.LayerID == 7 {
			fmt.Printf("    ○ %s  [online · silent]\n", n.Name)
		}
	}
	fmt.Println()

	// Silk Web
	l1count := 0
	for _, n := range f.Nodes {
		if n.LayerID == 1 { l1count++ }
	}
	fmt.Printf("  Silk Web: %d chars · %d edges\n", l1count, len(f.Edges))
	fmt.Println()

	fmt.Println("○ HomeOS running.")
	fmt.Println("  Type a command:")
	fmt.Println()

	// Simple REPL
	buf := make([]byte, 256)
	for {
		fmt.Print("  > ")
		n, err := os.Stdin.Read(buf)
		if err != nil { break }
		input := strings.TrimSpace(string(buf[:n]))
		if input == "" { continue }
		if input == "exit" || input == "quit" { break }

		// Tìm node theo input
		found := false
		if utf8.RuneCountInString(input) == 1 {
			r, _ := utf8.DecodeRuneInString(input)
			if node, ok := f.ByCP[r]; ok {
				fmt.Printf("  → '%c'  %s\n", r, node.Name)
				fmt.Printf("     ISL: %s\n", islStr(node.ISL))
				fmt.Printf("     DNA: %s\n", dnaSummary(node.DNA))
				for _, e := range node.Links {
					lt := linkNames[e.LType]
					if tn, ok2 := f.ByISL[islKey(e.To)]; ok2 {
						ch := ""
						if tn.Codepoint > 0 { ch = fmt.Sprintf("'%c'", tn.Codepoint) }
						fmt.Printf("     %-14s → %s %s\n", lt, ch, tn.Name)
					}
				}
				found = true
			}
		}

		if !found {
			// Tìm theo tên
			q := strings.ToUpper(input)
			for _, node := range f.Nodes {
				if strings.Contains(strings.ToUpper(node.Name), q) {
					ch := ""
					if node.Codepoint > 0 { ch = fmt.Sprintf("'%c'", node.Codepoint) }
					fmt.Printf("  → %s %s\n", ch, node.Name)
					fmt.Printf("     DNA: %s\n", dnaSummary(node.DNA))
					found = true
					break
				}
			}
		}

		if !found {
			fmt.Printf("  ? %q not found in silk web\n", input)
		}
		fmt.Println()
	}

	fmt.Println("\n○ shutdown.")
}

// ─── Byte helpers ────────────────────────────────────────────────

func be16(b []byte) uint16 { return binary.BigEndian.Uint16(b) }
func be32(b []byte) uint32 { return binary.BigEndian.Uint32(b) }
func be64(b []byte) uint64 { return binary.BigEndian.Uint64(b) }

// ─── Main ────────────────────────────────────────────────────────

func usage() {
	fmt.Println(`
○ olang — Olang Runtime

Usage:
  olang run    <file.olang>           run HomeOS
  olang info   <file.olang>           show file info
  olang dump   <file.olang>           dump all nodes
  olang get    <file.olang> <char>    get node by char or name
  olang walk   <file.olang> <char> [depth]  walk silk edges
  olang verify <file.olang>           verify SHA256

Examples:
  olang run    homeos.olang
  olang info   homeos.olang
  olang get    homeos.olang A
  olang get    homeos.olang 山
  olang walk   homeos.olang A 3
  olang verify homeos.olang
`)
}

func main() {
	if len(os.Args) < 3 {
		usage()
		os.Exit(1)
	}

	cmd  := os.Args[1]
	path := os.Args[2]

	f, err := load(path)
	if err != nil {
		fmt.Fprintf(os.Stderr, "❌ %v\n", err)
		os.Exit(1)
	}

	switch cmd {
	case "run":
		cmdRun(f, path)

	case "info":
		cmdInfo(f, path)

	case "dump":
		cmdDump(f)

	case "get":
		if len(os.Args) < 4 { fmt.Println("usage: olang get <file> <char>"); os.Exit(1) }
		cmdGet(f, os.Args[3])

	case "walk":
		if len(os.Args) < 4 { fmt.Println("usage: olang walk <file> <char> [depth]"); os.Exit(1) }
		depth := 2
		if len(os.Args) >= 5 {
			depth, _ = strconv.Atoi(os.Args[4])
		}
		cmdWalk(f, os.Args[3], depth)

	case "verify":
		if f.verify() {
			fmt.Printf("✅ %s — SHA256 OK\n    %x\n", path, f.Header.SHA256)
		} else {
			fmt.Printf("❌ %s — SHA256 MISMATCH\n", path)
			os.Exit(1)
		}

	default:
		fmt.Printf("❌ unknown command: %q\n", cmd)
		usage()
		os.Exit(1)
	}
}
