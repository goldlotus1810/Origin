// internal/olang/runtime.go
// Olang Runtime — đọc và thực thi file .olang
//
// File format:
//   [HEADER 64B][LAYER_INDEX 9×24B][L0..L8 data][SILK_EDGES]
//
// Mỗi node:
//   magic[2]="ND" layer[1] isl[8] codepoint[4] dna_len[2] name_len[1] dna[N] name[M]
//
// Mỗi edge:
//   from_isl[8] to_isl[8] link_type[1]  = 17B

package olang

import (
	"crypto/sha256"
	"encoding/binary"
	"fmt"
	"io"
	"os"
	"sync"
	"time"
)

// ─────────────────────────────────────────────────────────────────
// CONSTANTS
// ─────────────────────────────────────────────────────────────────

const (
	Magic   = "OLNG"
	Version = 18 // Unicode 18.0

	// Layer IDs
	LayerPrimitives = 0
	LayerUTF32      = 1
	LayerNature     = 2
	LayerLife       = 3
	LayerObjects    = 4
	LayerProgramming = 5
	LayerPerception = 6
	LayerPrograms   = 7
	LayerBuild      = 8 // ĐN zone — draft, chưa verify

	// Node status
	StatusEmpty    = 0
	StatusDN       = 1 // Đang học — có thể thay đổi
	StatusQR       = 2 // Đã chứng minh — bất biến
	StatusArchived = 3 // Không dùng nữa nhưng không xóa

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

	// File offsets
	HeaderSize     = 64
	LayerEntrySize = 24
	LayerIndexSize = 9 * LayerEntrySize // 216B

	// Node magic
	NodeMagic = 0x4E44 // "ND"
)

// ─────────────────────────────────────────────────────────────────
// DATA STRUCTURES
// ─────────────────────────────────────────────────────────────────

// ISLAddr là địa chỉ 8 byte của một node hoặc liên kết
type ISLAddr [8]byte

func (a ISLAddr) Layer() byte  { return a[0] }
func (a ISLAddr) Group() byte  { return a[1] }
func (a ISLAddr) Type() byte   { return a[2] }
func (a ISLAddr) ID() byte     { return a[3] }
func (a ISLAddr) Attr() uint32 { return binary.BigEndian.Uint32(a[4:]) }

func (a ISLAddr) ShapeHash() byte    { return byte(a.Attr() >> 24) }
func (a ISLAddr) PhonemeClass() byte { return byte(a.Attr() >> 16) }
func (a ISLAddr) ConceptGroup() byte { return byte(a.Attr() >> 8) }
func (a ISLAddr) Derivation() byte   { return byte(a.Attr()) }

func (a ISLAddr) String() string {
	return fmt.Sprintf("%c%c%c%d", a[0], a[1], a[2], a[3])
}

// Similarity tính độ tương đồng với addr khác (0.0–1.0)
func (a ISLAddr) Similarity(b ISLAddr) float32 {
	score := float32(0)
	weight := float32(0)

	if a[0] == b[0] { score += 3; weight += 3 }
	if a[1] == b[1] { score += 2; weight += 2 }
	if a[2] == b[2] { score += 1; weight += 1 }

	// Shape
	shapeDiff := popcount32(uint32(a.ShapeHash()) ^ uint32(b.ShapeHash()))
	score += float32(8-shapeDiff) / 8.0 * 4; weight += 4

	// Phoneme
	phoneShared := popcount32(uint32(a.PhonemeClass()) & uint32(b.PhonemeClass()))
	score += float32(phoneShared) / 8.0 * 3; weight += 3

	// Concept
	cDiff := absDiff8(a.ConceptGroup(), b.ConceptGroup())
	score += float32(32-min8(cDiff, 32)) / 32.0 * 4; weight += 4

	// Derivation: cùng script family
	if a.Derivation()>>4 == b.Derivation()>>4 { score += 1; weight += 1 }

	if weight == 0 { return 0 }
	return score / weight
}

// Node là một đơn vị tri thức trong Olang
type Node struct {
	LayerID   uint8
	ISL       ISLAddr
	Codepoint rune   // Unicode codepoint (0 nếu không phải UTF32)
	DNA       []byte // SDF/logic bytecode
	Name      string // Human-readable name

	// Runtime state (không lưu vào file)
	Links []Link
}

// Link là liên kết giữa 2 node
// ISL của liên kết = địa chỉ trong không gian ngữ nghĩa
type Link struct {
	From     ISLAddr
	To       ISLAddr
	Type     uint8
	Weight   float32 // tăng khi được dùng nhiều
	Learned  bool    // true = NCA tự học
}

// LayerEntry là metadata của 1 layer trong file
type LayerEntry struct {
	ID     uint8
	Status uint8
	Flags  uint16
	Offset uint64
	Size   uint64
	NNodes uint32
}

// FileHeader là header 64B của file .olang
type FileHeader struct {
	Magic   [4]byte
	Version uint16
	Flags   uint16
	NLayers uint8
	_       [3]byte // reserved
	NNodes  uint32
	NEdges  uint32
	Created int64
	SHA256  [32]byte
}

// ─────────────────────────────────────────────────────────────────
// RUNTIME
// ─────────────────────────────────────────────────────────────────

// Runtime là Olang runtime — load và thực thi file .olang
type Runtime struct {
	mu sync.RWMutex

	// Index chính: ISL uint64 → *Node
	index map[uint64]*Node

	// Index theo codepoint
	byCodepoint map[rune]*Node

	// Index theo layer
	byLayer [9][]*Node

	// Silk edges
	edges []Link

	// File header
	header FileHeader
	layers [9]LayerEntry

	// Stats
	loadedAt time.Time
	filePath string
}

// NewRuntime tạo runtime mới
func NewRuntime() *Runtime {
	return &Runtime{
		index:       make(map[uint64]*Node),
		byCodepoint: make(map[rune]*Node),
	}
}

// Load đọc file .olang vào memory
func (r *Runtime) Load(path string) error {
	f, err := os.Open(path)
	if err != nil {
		return fmt.Errorf("olang: open %s: %w", path, err)
	}
	defer f.Close()

	data, err := io.ReadAll(f)
	if err != nil {
		return fmt.Errorf("olang: read: %w", err)
	}

	if len(data) < HeaderSize+LayerIndexSize {
		return fmt.Errorf("olang: file too small (%d bytes)", len(data))
	}

	// Đọc header
	if string(data[0:4]) != Magic {
		return fmt.Errorf("olang: invalid magic %q", data[0:4])
	}
	r.header.Version = binary.BigEndian.Uint16(data[4:6])
	r.header.Flags   = binary.BigEndian.Uint16(data[6:8])
	r.header.NLayers = data[8]
	r.header.NNodes  = binary.BigEndian.Uint32(data[12:16])
	r.header.NEdges  = binary.BigEndian.Uint32(data[16:20])
	r.header.Created = int64(binary.BigEndian.Uint64(data[20:28]))
	copy(r.header.SHA256[:], data[28:60])

	// Verify SHA256
	content := data[HeaderSize+LayerIndexSize:]
	sum := sha256.Sum256(content)
	if sum != r.header.SHA256 {
		return fmt.Errorf("olang: SHA256 mismatch — file corrupted")
	}

	// Đọc layer index
	for i := 0; i < 9; i++ {
		off := HeaderSize + i*LayerEntrySize
		r.layers[i] = LayerEntry{
			ID:     data[off],
			Status: data[off+1],
			Flags:  binary.BigEndian.Uint16(data[off+2 : off+4]),
			Offset: binary.BigEndian.Uint64(data[off+4 : off+12]),
			Size:   binary.BigEndian.Uint64(data[off+12 : off+20]),
			NNodes: binary.BigEndian.Uint32(data[off+20 : off+24]),
		}
	}

	// Đọc tất cả nodes từ các layer
	r.mu.Lock()
	defer r.mu.Unlock()

	for i := 0; i < 9; i++ {
		le := r.layers[i]
		if le.Size == 0 { continue }

		layerData := data[le.Offset : le.Offset+le.Size]
		nodes, err := r.parseNodes(layerData, uint8(i))
		if err != nil {
			return fmt.Errorf("olang: parse layer %d: %w", i, err)
		}

		for _, n := range nodes {
			key := islToUint64(n.ISL)
			r.index[key] = n
			if n.Codepoint != 0 {
				r.byCodepoint[n.Codepoint] = n
			}
			r.byLayer[i] = append(r.byLayer[i], n)
		}
	}

	// Đọc edges từ cuối file
	// Edges nằm sau tất cả layer data
	lastLayer := r.layers[8]
	edgeStart := lastLayer.Offset + lastLayer.Size
	// Tìm layer cuối có data
	for i := 8; i >= 0; i-- {
		if r.layers[i].Size > 0 {
			edgeStart = r.layers[i].Offset + r.layers[i].Size
			break
		}
	}

	edgeData := data[edgeStart:]
	for len(edgeData) >= 17 {
		var from, to ISLAddr
		copy(from[:], edgeData[0:8])
		copy(to[:], edgeData[8:16])
		ltype := edgeData[16]
		r.edges = append(r.edges, Link{
			From:    from,
			To:      to,
			Type:    ltype,
			Weight:  1.0,
			Learned: ltype == LinkLearned,
		})
		edgeData = edgeData[17:]
	}

	// Wire edges vào nodes
	for _, e := range r.edges {
		key := islToUint64(e.From)
		if n, ok := r.index[key]; ok {
			n.Links = append(n.Links, e)
		}
	}

	r.filePath = path
	r.loadedAt = time.Now()
	return nil
}

// parseNodes đọc danh sách nodes từ raw bytes
func (r *Runtime) parseNodes(data []byte, layerID uint8) ([]*Node, error) {
	var nodes []*Node
	offset := 0

	for offset < len(data) {
		if offset+2 > len(data) { break }

		// Check magic "ND"
		if data[offset] != 'N' || data[offset+1] != 'D' {
			return nil, fmt.Errorf("expected node magic at offset %d, got %02x%02x",
				offset, data[offset], data[offset+1])
		}
		offset += 2

		if offset+18 > len(data) {
			return nil, fmt.Errorf("node header truncated at %d", offset)
		}

		lid        := data[offset]; offset++
		var isl ISLAddr
		copy(isl[:], data[offset:offset+8]); offset += 8
		cp          := rune(binary.BigEndian.Uint32(data[offset : offset+4])); offset += 4
		dnaLen      := int(binary.BigEndian.Uint16(data[offset : offset+2])); offset += 2
		nameLen     := int(data[offset]); offset++

		if offset+dnaLen+nameLen > len(data) {
			return nil, fmt.Errorf("node data truncated at %d", offset)
		}

		dna  := make([]byte, dnaLen)
		copy(dna, data[offset:offset+dnaLen]); offset += dnaLen

		name := string(data[offset : offset+nameLen]); offset += nameLen

		_ = lid // stored in ISL layer byte already

		nodes = append(nodes, &Node{
			LayerID:   layerID,
			ISL:       isl,
			Codepoint: cp,
			DNA:       dna,
			Name:      name,
		})
	}

	return nodes, nil
}

// ─────────────────────────────────────────────────────────────────
// QUERY API
// ─────────────────────────────────────────────────────────────────

// Get tìm node theo ISL address
func (r *Runtime) Get(isl ISLAddr) (*Node, bool) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	n, ok := r.index[islToUint64(isl)]
	return n, ok
}

// GetByCodepoint tìm node theo Unicode codepoint
func (r *Runtime) GetByCodepoint(cp rune) (*Node, bool) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	n, ok := r.byCodepoint[cp]
	return n, ok
}

// Layer trả về tất cả nodes trong một layer
func (r *Runtime) Layer(id int) []*Node {
	r.mu.RLock()
	defer r.mu.RUnlock()
	if id < 0 || id > 8 { return nil }
	return r.byLayer[id]
}

// Walk đi theo silk edges từ một node, depth hops
func (r *Runtime) Walk(start ISLAddr, depth int) []*Node {
	r.mu.RLock()
	defer r.mu.RUnlock()

	visited := make(map[uint64]bool)
	var result []*Node

	var walk func(isl ISLAddr, d int)
	walk = func(isl ISLAddr, d int) {
		key := islToUint64(isl)
		if visited[key] || d < 0 { return }
		visited[key] = true

		n, ok := r.index[key]
		if !ok { return }
		result = append(result, n)

		for _, link := range n.Links {
			walk(link.To, d-1)
		}
	}

	walk(start, depth)
	return result
}

// Similar tìm các node tương đồng với threshold
func (r *Runtime) Similar(addr ISLAddr, threshold float32) []*Node {
	r.mu.RLock()
	defer r.mu.RUnlock()

	var result []*Node
	for _, n := range r.index {
		if sim := addr.Similarity(n.ISL); sim >= threshold {
			result = append(result, n)
		}
	}
	return result
}

// Append thêm node mới (ĐN zone — chưa verify)
// SecurityGate check trước khi append
func (r *Runtime) Append(n *Node) error {
	r.mu.Lock()
	defer r.mu.Unlock()

	// SecurityGate Rule 1: không bao giờ override QR node
	key := islToUint64(n.ISL)
	if existing, ok := r.index[key]; ok {
		if existing.LayerID <= LayerObjects { // L0-L4 = QR
			return fmt.Errorf("olang: cannot overwrite QR node %s", n.ISL)
		}
	}

	n.LayerID = LayerBuild
	r.index[key] = n
	r.byLayer[LayerBuild] = append(r.byLayer[LayerBuild], n)
	if n.Codepoint != 0 {
		r.byCodepoint[n.Codepoint] = n
	}
	return nil
}

// Stats trả về thống kê runtime
func (r *Runtime) Stats() map[string]interface{} {
	r.mu.RLock()
	defer r.mu.RUnlock()

	layerNames := []string{"Primitives","UTF32","Nature","Life","Objects",
		"Programming","Perception","Programs","Build"}

	stats := map[string]interface{}{
		"file":      r.filePath,
		"loaded_at": r.loadedAt,
		"version":   r.header.Version,
		"total_nodes": len(r.index),
		"total_edges": len(r.edges),
	}

	layers := make(map[string]int)
	for i, name := range layerNames {
		layers[name] = len(r.byLayer[i])
	}
	stats["layers"] = layers
	return stats
}

// ─────────────────────────────────────────────────────────────────
// HELPERS
// ─────────────────────────────────────────────────────────────────

func islToUint64(isl ISLAddr) uint64 {
	return binary.BigEndian.Uint64(isl[:])
}

func popcount32(x uint32) int {
	count := 0
	for x != 0 { count += int(x & 1); x >>= 1 }
	return count
}

func absDiff8(a, b byte) byte {
	if a > b { return a - b }
	return b - a
}

func min8(a, b byte) byte {
	if a < b { return a }
	return b
}
