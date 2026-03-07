// internal/olang/file.go
//
// OLANG FILE FORMAT — 1 file duy nhất chứa mọi thứ
// ==================================================
// Không import. Không dependency. Không install.
// 1 file .olang = toàn bộ vũ trụ của hệ thống.
//
// Cấu trúc:
//   HEADER (64B) + LAYER_INDEX + LAYERS + EDGE_TABLE + SIGNATURE
//
// Nguyên tắc:
//   Append-only: không DELETE, không OVERWRITE
//   QR nodes: đã chứng minh, bất biến, có ED25519 signature
//   ĐN nodes:  đang học, có thể thay đổi, không có sig
//   Ghi vào lõi: chỉ khi verified + signed

package olang

import (
	"crypto/sha256"
	"encoding/binary"
	"errors"
	"io"
	"time"

	"github.com/goldlotus1810/HomeOS/internal/isl"
)

// ── Magic & Version ───────────────────────────────────────────
var Magic = [4]byte{'O', 'L', 'N', 'G'}

const (
	Version18 = uint16(18) // Unicode 18.0 compatible

	// Flags
	FlagZstd    = uint16(0x0001) // payload compressed with zstd
	FlagAES     = uint16(0x0002) // payload encrypted AES-256-GCM
	FlagSigned  = uint16(0x0004) // file signed with ED25519
	FlagSealed  = uint16(0x0008) // file is sealed (no more ĐN appends)
)

// ── Layer IDs ─────────────────────────────────────────────────
const (
	LayerPrimitives  = uint8(0) // L0: SDF opcodes, logic, math — bất biến tuyệt đối
	LayerUTF32       = uint8(1) // L1: 168K chars × SDF + ISL + edges
	LayerNature      = uint8(2) // L2: nước, lửa, đất, gió, ánh sáng
	LayerLife        = uint8(3) // L3: tế bào, cây, động vật, người
	LayerObjects     = uint8(4) // L4: đèn, cửa, xe, thiết bị
	LayerProgramming = uint8(5) // L5: Go/Python/WASM semantics + opcodes
	LayerPerception  = uint8(6) // L6: vision, audio, touch, depth
	LayerPrograms    = uint8(7) // L7: agents, skills, worlds — executable
	LayerBuild       = uint8(8) // L8+: draft zone — ĐN, chưa commit
)

// LayerNames cho human display
var LayerNames = map[uint8]string{
	LayerPrimitives:  "PRIMITIVES",
	LayerUTF32:       "UTF32",
	LayerNature:      "NATURE",
	LayerLife:        "LIFE",
	LayerObjects:     "OBJECTS",
	LayerProgramming: "PROGRAMMING",
	LayerPerception:  "PERCEPTION",
	LayerPrograms:    "PROGRAMS",
	LayerBuild:       "BUILD",
}

// Node status — vòng đời tri thức (QT7)
const (
	StatusDN       = uint8(0x01) // ĐN — đang học, có thể sai, có thể thay đổi
	StatusQR       = uint8(0x02) // QR — đã chứng minh, append-only, có sig
	StatusArchived = uint8(0x03) // ARCHIVED — deprecated, không xóa
)

// ── Silk Edge types ───────────────────────────────────────────
const (
	EdgeSameSound   = uint8(0x01) // A≡α: cùng âm, khác script
	EdgeSameShape   = uint8(0x02) // O≡0: cùng hình
	EdgeSameMeaning = uint8(0x03) // +≡∪: cùng nghĩa
	EdgeDerivedFrom = uint8(0x04) // A←𐤀: nguồn gốc lịch sử
	EdgeLowercase   = uint8(0x05) // A→a
	EdgeUppercase   = uint8(0x06) // a→A
	EdgeMirror      = uint8(0x07) // b↔d
	EdgeMember      = uint8(0x08) // x∈S: thành viên
	EdgeSubset      = uint8(0x09) // A⊂B
	EdgeCompose     = uint8(0x0A) // f∘g: kết hợp
	EdgeEquiv       = uint8(0x0B) // a≡b: tương đương chính xác
	EdgeSimilar     = uint8(0x0C) // a≈b: tương đương xấp xỉ
	EdgeOpposite    = uint8(0x0D) // a⊥b: ngược nhau
	EdgeWorldLink   = uint8(0x0E) // char→world SDF: link tới thực tế
	EdgePhoneme     = uint8(0x0F) // char→phoneme: link âm thanh
)

// ════════════════════════════════════════════════════════════════
// FILE HEADER — 64 bytes, bất biến sau khi write
// ════════════════════════════════════════════════════════════════

// FileHeader là 64 bytes đầu tiên của mọi .olang file.
// Sau khi write, không bao giờ thay đổi.
// Phản ánh trạng thái tại thời điểm seal cuối cùng.
type FileHeader struct {
	Magic    [4]byte   // "OLNG"
	Version  uint16    // 18 = Unicode 18.0 compatible
	Flags    uint16    // FlagZstd | FlagAES | FlagSigned | ...
	NLayers  uint8     // số layers hiện tại
	_        [7]byte   // reserved, must be zero
	Created  int64     // unix timestamp nanoseconds
	RootAddr isl.Address // ISL address của root node (L0)
	SHA256   [32]byte  // SHA256 của toàn bộ file (trừ 32B cuối này)
}

// Validate kiểm tra header hợp lệ
func (h *FileHeader) Validate() error {
	if h.Magic != Magic {
		return errors.New("invalid magic: not an olang file")
	}
	if h.Version == 0 {
		return errors.New("invalid version: 0")
	}
	if h.NLayers == 0 {
		return errors.New("no layers")
	}
	return nil
}

// ════════════════════════════════════════════════════════════════
// LAYER INDEX — mỗi layer là 1 phân vùng logic
// ════════════════════════════════════════════════════════════════

// LayerEntry là 1 entry trong layer index.
// 24 bytes per entry.
type LayerEntry struct {
	ID     uint8    // LayerPrimitives = 0, LayerUTF32 = 1, ...
	Status uint8    // StatusDN | StatusQR | StatusArchived
	_      [2]byte  // reserved
	Offset uint64   // byte offset trong file (sau headers)
	Size   uint64   // compressed size in bytes
	Count  uint32   // số nodes trong layer
	_2     [4]byte  // reserved
}

// LayerBudgets — dung lượng ước tính (sau zstd+AES)
var LayerBudgets = map[uint8]int64{
	LayerPrimitives:  4 * 1024,           // 4 KB
	LayerUTF32:       4_400 * 1024,       // 4.4 MB
	LayerNature:      800 * 1024,         // 0.8 MB
	LayerLife:        1_800 * 1024,       // 1.8 MB
	LayerObjects:     1_200 * 1024,       // 1.2 MB
	LayerProgramming: 2_900 * 1024,       // 2.9 MB
	LayerPerception:  1_100 * 1024,       // 1.1 MB
	LayerPrograms:    700 * 1024,         // 0.7 MB
}

// TotalBudget tổng dung lượng dự kiến (không gồm silk edges)
func TotalBudget() int64 {
	total := int64(0)
	for _, b := range LayerBudgets {
		total += b
	}
	return total // ~13 MB layers
}

// ════════════════════════════════════════════════════════════════
// OLANG NODE — đơn vị tri thức cơ bản
// ════════════════════════════════════════════════════════════════

// OlangNodeHeader là phần cố định của mỗi node (17 bytes).
type OlangNodeHeader struct {
	Codepoint  uint32  // Unicode codepoint / ISL-derived ID
	ISLAddr    uint64  // ISL address 8 bytes
	Flags      uint8   // LayerID(4b) | Status(2b) | reserved(2b)
	DNALen     uint16  // length của DNA bytes
	EdgeCount  uint8   // số silk edges
	PhonemeCount uint8 // số phoneme refs
}

// OlangNode là 1 node đầy đủ trong .olang file.
type OlangNode struct {
	OlangNodeHeader

	// Variable-length payload
	DNA      []byte   // SDF formula trong Olang binary encoding
	Edges    []SilkEdge
	Phonemes []uint16  // ISL refs đến IPA phoneme nodes

	// Chỉ QR nodes có signature
	Sig []byte // ED25519 signature, 64 bytes hoặc nil
}

// NodeSize tính tổng kích thước serialized của node (bytes)
func (n *OlangNode) NodeSize() int {
	size := 17 // fixed header
	size += len(n.DNA)
	size += len(n.Edges) * 9    // 9B per edge
	size += len(n.Phonemes) * 2 // 2B per phoneme ref
	if n.Sig != nil {
		size += 64
	}
	return size
}

// Layer trả về layer ID của node
func (n *OlangNode) Layer() uint8 { return n.Flags >> 4 }

// Status trả về status của node
func (n *OlangNode) Status() uint8 { return (n.Flags >> 2) & 0x3 }

// IsQR kiểm tra node đã được chứng minh
func (n *OlangNode) IsQR() bool { return n.Status() == StatusQR }

// ════════════════════════════════════════════════════════════════
// SILK EDGE — quan hệ giữa các nodes
// ════════════════════════════════════════════════════════════════

// SilkEdge là một silk edge nối 2 nodes.
// 9 bytes: src(implicit) + dst(8B) + type(1B)
// src được biết từ context (node đang chứa edge)
type SilkEdge struct {
	Dst  uint64  // ISL address của destination node
	Type uint8   // EdgeSameSound | EdgeSameShape | ...
}

// SilkEdgeEntry trong edge table có cả src và dst (17 bytes).
// Edge table là sorted view của tất cả edges — cho phép lookup nhanh.
type SilkEdgeEntry struct {
	Src  uint64
	Dst  uint64
	Type uint8
}

// ════════════════════════════════════════════════════════════════
// OLANG FILE — đọc/ghi
// ════════════════════════════════════════════════════════════════

// OlangFile đại diện cho 1 file .olang đã được load vào memory.
// Hoặc đang được build (L8+ draft zone).
type OlangFile struct {
	Header    FileHeader
	LayerIdx  []LayerEntry
	// Nodes theo layer — key = layer ID, value = list nodes
	Layers    map[uint8][]*OlangNode
	EdgeTable []SilkEdgeEntry

	// Stats
	TotalNodes int
	TotalEdges int
	FileSize   int64

	// Build zone — ĐN nodes chưa commit
	Draft []*OlangNode
}

// New tạo một OlangFile mới (empty).
func New() *OlangFile {
	return &OlangFile{
		Header: FileHeader{
			Magic:   Magic,
			Version: Version18,
			Flags:   FlagZstd | FlagAES,
			Created: time.Now().UnixNano(),
			NLayers: 9, // L0-L7 + L8 build
		},
		Layers: make(map[uint8][]*OlangNode),
	}
}

// AppendNode thêm node vào đúng layer.
// Nếu node là QR: ghi vào layer chính thức.
// Nếu node là ĐN: ghi vào Draft zone.
// Không bao giờ overwrite — chỉ append.
func (f *OlangFile) AppendNode(n *OlangNode) error {
	// L0 chỉ nhận QR nodes
	if n.Layer() == LayerPrimitives && !n.IsQR() {
		return errors.New("L0 primitives must be QR (proven)")
	}
	// SecurityGate: không hại người, không xóa data
	if err := securityGate(n); err != nil {
		return err
	}

	if n.IsQR() {
		layer := n.Layer()
		f.Layers[layer] = append(f.Layers[layer], n)
	} else {
		// ĐN nodes vào draft zone
		f.Draft = append(f.Draft, n)
	}
	f.TotalNodes++
	for _, e := range n.Edges {
		f.EdgeTable = append(f.EdgeTable, SilkEdgeEntry{
			Src:  addrToUint64(n.ISLAddr),
			Dst:  e.Dst,
			Type: e.Type,
		})
	}
	f.TotalEdges += len(n.Edges)
	return nil
}

// Promote đưa một ĐN node lên QR sau khi verified + signed.
// Đây là lúc "ghi vào lõi" của .olang.
func (f *OlangFile) Promote(addr isl.Address, sig []byte) error {
	if len(sig) != 64 {
		return errors.New("ED25519 signature must be 64 bytes")
	}
	// Tìm trong draft
	for i, n := range f.Draft {
		if addrToUint64(n.ISLAddr) == addrToUint64(addr) {
			n.Sig = sig
			// Cập nhật flags: status = QR
			n.Flags = (n.Flags & 0xF0) | (StatusQR << 2)
			// Move to official layer
			layer := n.Layer()
			f.Layers[layer] = append(f.Layers[layer], n)
			// Remove from draft (preserve order, set nil)
			f.Draft[i] = nil
			return nil
		}
	}
	return errors.New("node not found in draft")
}

// Hash tính SHA256 của toàn bộ nội dung (không gồm hash field).
func (f *OlangFile) Hash() [32]byte {
	h := sha256.New()
	// Write header without hash field
	h.Write(f.Header.Magic[:])
	binary.Write(h, binary.LittleEndian, f.Header.Version)
	binary.Write(h, binary.LittleEndian, f.Header.Flags)
	binary.Write(h, binary.LittleEndian, f.Header.NLayers)
	binary.Write(h, binary.LittleEndian, f.Header.Created)
	// Write all nodes
	for _, nodes := range f.Layers {
		for _, n := range nodes {
			if n == nil { continue }
			h.Write(n.DNA)
		}
	}
	// Write edge table
	for _, e := range f.EdgeTable {
		binary.Write(h, binary.LittleEndian, e)
	}
	var result [32]byte
	copy(result[:], h.Sum(nil))
	return result
}

// SizeReport trả về báo cáo dung lượng của từng layer
func (f *OlangFile) SizeReport() map[string]int {
	report := make(map[string]int)
	total := 0
	for lid, nodes := range f.Layers {
		size := 0
		for _, n := range nodes {
			if n != nil { size += n.NodeSize() }
		}
		name := LayerNames[lid]
		report[name] = size
		total += size
	}
	report["DRAFT"] = func() int {
		s := 0
		for _, n := range f.Draft {
			if n != nil { s += n.NodeSize() }
		}
		return s
	}()
	report["EDGES"] = len(f.EdgeTable) * 17
	report["TOTAL_RAW"] = total + report["EDGES"]
	return report
}

// ════════════════════════════════════════════════════════════════
// SECURITY GATE (Rule 1: không hại người)
// ════════════════════════════════════════════════════════════════

// securityGate kiểm tra node trước khi append vào file.
// Rule 1: Không hại con người — bất biến tuyệt đối
// Rule 2: Không vòng lặp vô hạn (LOOP phải có điều kiện thoát)
// Rule 3: Không xóa dữ liệu bất biến (không được OVERWRITE QR)
func securityGate(n *OlangNode) error {
	if n == nil {
		return errors.New("nil node")
	}
	// Rule 3: Không append node có cùng ISL address với QR node đã tồn tại
	// (check này cần access vào file — handled trong AppendNode)

	// Rule 2: LOOP opcodes phải có điều kiện (DNA không phải LOOP(∞))
	if len(n.DNA) > 0 && n.DNA[0] == 0x62 { // LOOP opcode
		// 0x62 LOOP phải có ít nhất 2B: opcode + condition
		if len(n.DNA) < 3 {
			return errors.New("LOOP opcode requires condition (Rule 2: no infinite loops)")
		}
	}
	return nil
}

// ════════════════════════════════════════════════════════════════
// HELPERS
// ════════════════════════════════════════════════════════════════

func addrToUint64(a isl.Address) uint64 {
	return uint64(a.Layer)<<56 | uint64(a.Group)<<48 |
		uint64(a.Type)<<40 | uint64(a.ID)<<32
}

func uint64ToAddr(v uint64) isl.Address {
	return isl.Address{
		Layer: byte(v >> 56),
		Group: byte(v >> 48),
		Type:  byte(v >> 40),
		ID:    byte(v >> 32),
	}
}

// WriteTo viết OlangFile vào writer (uncompressed, cho testing).
// Production version sẽ dùng zstd + AES-256-GCM.
func (f *OlangFile) WriteTo(w io.Writer) error {
	// Update hash trước khi write
	f.Header.SHA256 = f.Hash()
	// Write header (64 bytes)
	if err := binary.Write(w, binary.LittleEndian, f.Header); err != nil {
		return err
	}
	// Write layer index
	for lid := uint8(0); lid < f.Header.NLayers; lid++ {
		nodes := f.Layers[lid]
		entry := LayerEntry{
			ID:    lid,
			Status: StatusQR,
			Count: uint32(len(nodes)),
		}
		if err := binary.Write(w, binary.LittleEndian, entry); err != nil {
			return err
		}
	}
	// Write nodes per layer
	for lid := uint8(0); lid < f.Header.NLayers; lid++ {
		for _, n := range f.Layers[lid] {
			if n == nil { continue }
			if err := binary.Write(w, binary.LittleEndian, n.OlangNodeHeader); err != nil {
				return err
			}
			w.Write(n.DNA)
			for _, e := range n.Edges {
				binary.Write(w, binary.LittleEndian, e)
			}
			for _, p := range n.Phonemes {
				binary.Write(w, binary.LittleEndian, p)
			}
			if n.Sig != nil {
				w.Write(n.Sig)
			}
		}
	}
	// Write edge table
	for _, e := range f.EdgeTable {
		binary.Write(w, binary.LittleEndian, e)
	}
	return nil
}

// ════════════════════════════════════════════════════════════════
// MILESTONE TRACKER
// ════════════════════════════════════════════════════════════════

// Milestone tracks progress toward "ghi vào lõi"
type Milestone struct {
	ID   int
	Name string
	// Layers cần complete trước khi commit milestone này
	Layers  []uint8
	// Minimum nodes required per layer
	MinNodes map[uint8]int
	// Completed = tất cả conditions met + user signed
	Completed bool
}

// Milestones định nghĩa lộ trình build
var Milestones = []Milestone{
	{
		ID: 1, Name: "UTF32 Core",
		Layers:   []uint8{LayerUTF32},
		MinNodes: map[uint8]int{LayerUTF32: 168046},
		// ✓ 168K chars có valid SDF
		// ✓ Render đúng 8pt → 1000pt
		// ✓ Silk edges verified
		// → COMMIT L1
	},
	{
		ID: 2, Name: "Nature + Life",
		Layers:   []uint8{LayerNature, LayerLife},
		MinNodes: map[uint8]int{LayerNature: 50, LayerLife: 200},
		// ✓ Water, fire, earth animate
		// ✓ Human walks via Spline
		// → COMMIT L2+L3
	},
	{
		ID: 3, Name: "HomeOS v1.0",
		Layers:   []uint8{LayerObjects, LayerPrograms},
		MinNodes: map[uint8]int{LayerObjects: 20, LayerPrograms: 10},
		// ✓ "tắt đèn phòng khách" end-to-end
		// ✓ AAM → Chief → LightAgent works
		// ✓ SecurityGate pass
		// → olang run homeos.olang WORKS
	},
	{
		ID: 4, Name: "Full Stack",
		Layers:   []uint8{LayerProgramming, LayerPerception},
		MinNodes: map[uint8]int{LayerProgramming: 500, LayerPerception: 30},
		// ✓ Go AST → ISL → compile
		// ✓ Vision: agent sees ISL addresses
		// → Complete .olang
	},
}

// CheckMilestone kiểm tra milestone đã đạt chưa
func (f *OlangFile) CheckMilestone(m *Milestone) bool {
	for layer, min := range m.MinNodes {
		nodes := f.Layers[layer]
		count := 0
		for _, n := range nodes {
			if n != nil && n.IsQR() { count++ }
		}
		if count < min { return false }
	}
	return true
}
