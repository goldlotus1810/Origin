// internal/silk/silk.go
// Silk Web — cấu trúc lưu trữ tri thức của HomeOS
// OlangAtom: 12B/symbol (4B codepoint + 8B ISL)
// SilkEdge: silk link giữa 2 nodes
// SilkGraph: đồ thị tri thức append-only

package silk

import (
	"crypto/ed25519"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"os"
	"sync"
	"time"

	"github.com/goldlotus1810/HomeOS/internal/isl"
)

// ─────────────────────────────────────────────────────────────────
// OLANG ATOM — 12 bytes/symbol
// ─────────────────────────────────────────────────────────────────

// OlangAtom là đơn vị cơ bản của ngôn ngữ Olang
// Codepoint: 4B UTF-32 (ký tự gốc)
// ISL:       8B address (nghĩa trong hệ thống)
type OlangAtom struct {
	Codepoint rune        // 4B — ký tự UTF-32
	ISL       isl.Address // 8B — địa chỉ ISL
} // = 12B/symbol như spec

// ─────────────────────────────────────────────────────────────────
// SILK EDGE — liên kết ngữ nghĩa
// ─────────────────────────────────────────────────────────────────

// EdgeOp là loại quan hệ của silk edge
type EdgeOp rune

const (
	OpMember    EdgeOp = '∈' // thuộc về (parent-child)
	OpEquiv     EdgeOp = '≡' // tương đương ngữ nghĩa (A≡α≡а)
	OpPhonetic  EdgeOp = '♫' // cùng âm qua IPA
	OpCompose   EdgeOp = '∘' // tổ hợp (math∘olang)
	OpSimilar   EdgeOp = '≈' // tương tự
)

// SilkEdge là một liên kết ngữ nghĩa giữa 2 nodes
type SilkEdge struct {
	To isl.Address // địa chỉ node đích
	Op EdgeOp      // loại quan hệ
}

// ─────────────────────────────────────────────────────────────────
// NODE STATUS
// ─────────────────────────────────────────────────────────────────

type NodeStatus byte

const (
	StatusDN       NodeStatus = 0x01 // ΔΨ — đang học, chưa chứng minh
	StatusQR       NodeStatus = 0x02 // QR — đã chứng minh, append-only
	StatusArchived NodeStatus = 0x03 // ARCHIVED — không xóa, chỉ ẩn
)

// ─────────────────────────────────────────────────────────────────
// OLANG NODE
// ─────────────────────────────────────────────────────────────────

// OlangNode là node trong SilkGraph
type OlangNode struct {
	Addr   isl.Address  // địa chỉ ISL của node
	Atom   *OlangAtom   // atom gốc (nếu là character node)
	Edges  []SilkEdge   // silk links đi ra
	Status NodeStatus
	Weight int          // số lần xác nhận
	Sig    []byte       // ED25519 signature (chỉ QR nodes)
	// Metadata
	Name   string       // tên human-readable
	Glyph  string       // ký tự hiển thị
	Cat    string       // category (scripts/math/geo/...)
	Layer  int          // 0=origin, 1=supergroup, 2=family, 3=leaf
	// Olang encoding
	DNA    string       // SDF formula trong Olang notation — lưu công thức, không lưu hình
	// Ví dụ: "∪(⌀(-.28,.44,0,-.44,0), ⌀(.28,.44,0,-.44,0), □(0,.05,.36,.04))"
}

// ─────────────────────────────────────────────────────────────────
// SILK GRAPH — đồ thị tri thức
// ─────────────────────────────────────────────────────────────────

// SilkGraph là đồ thị tri thức append-only
// Tìm kiếm: đến đúng node → Walk theo silk edges → không scan toàn bộ
type SilkGraph struct {
	mu    sync.RWMutex
	nodes map[uint64]*OlangNode // key = isl.Address.Uint64()
	byID  map[string]*OlangNode // key = tên/id string (cho prototype)
}

func NewSilkGraph() *SilkGraph {
	return &SilkGraph{
		nodes: make(map[uint64]*OlangNode),
		byID:  make(map[string]*OlangNode),
	}
}

// AddNode thêm node — nếu đã tồn tại thì update (ΔΨ) hoặc bỏ qua (QR)
func (g *SilkGraph) AddNode(n *OlangNode) {
	g.mu.Lock(); defer g.mu.Unlock()
	key := n.Addr.Uint64()
	existing, ok := g.nodes[key]
	if ok && existing.Status == StatusQR {
		return // QR = append-only, không ghi đè
	}
	g.nodes[key] = n
	if n.Name != "" {
		g.byID[n.Name] = n
	}
}

// AddEdge thêm silk edge từ fromAddr đến toAddr
func (g *SilkGraph) AddEdge(from, to isl.Address, op EdgeOp) {
	g.mu.Lock(); defer g.mu.Unlock()
	n := g.nodes[from.Uint64()]
	if n == nil {
		return
	}
	// Không thêm duplicate
	for _, e := range n.Edges {
		if e.To.Uint64() == to.Uint64() && e.Op == op {
			return
		}
	}
	n.Edges = append(n.Edges, SilkEdge{To: to, Op: op})
}

// Get lấy node theo địa chỉ ISL
func (g *SilkGraph) Get(addr isl.Address) (*OlangNode, bool) {
	g.mu.RLock(); defer g.mu.RUnlock()
	n, ok := g.nodes[addr.Uint64()]
	return n, ok
}

// GetByName lấy node theo tên (dùng cho prototype/debug)
func (g *SilkGraph) GetByName(name string) (*OlangNode, bool) {
	g.mu.RLock(); defer g.mu.RUnlock()
	n, ok := g.byID[name]
	return n, ok
}

// Walk đi theo silk edges từ node, lọc theo op (OpMember = theo cây)
// Đây là cách tìm kiếm chính: không scan toàn bộ graph
func (g *SilkGraph) Walk(from isl.Address, op EdgeOp) []*OlangNode {
	g.mu.RLock(); defer g.mu.RUnlock()
	n := g.nodes[from.Uint64()]
	if n == nil {
		return nil
	}
	var result []*OlangNode
	for _, e := range n.Edges {
		if e.Op == op {
			if child, ok := g.nodes[e.To.Uint64()]; ok {
				result = append(result, child)
			}
		}
	}
	return result
}

// NodeCount trả về số nodes hiện tại
func (g *SilkGraph) NodeCount() int {
	g.mu.RLock(); defer g.mu.RUnlock()
	return len(g.nodes)
}

// EdgeCount trả về tổng số silk edges
func (g *SilkGraph) EdgeCount() int {
	g.mu.RLock(); defer g.mu.RUnlock()
	total := 0
	for _, n := range g.nodes {
		total += len(n.Edges)
	}
	return total
}

// AllNodes trả về tất cả nodes (dùng cho renderer)
func (g *SilkGraph) AllNodes() []*OlangNode {
	g.mu.RLock(); defer g.mu.RUnlock()
	out := make([]*OlangNode, 0, len(g.nodes))
	for _, n := range g.nodes {
		out = append(out, n)
	}
	return out
}

// AllEdges trả về tất cả edges dạng (from, to, op)
func (g *SilkGraph) AllEdges() [][3]interface{} {
	g.mu.RLock(); defer g.mu.RUnlock()
	var out [][3]interface{}
	for _, n := range g.nodes {
		for _, e := range n.Edges {
			out = append(out, [3]interface{}{n.Addr, e.To, e.Op})
		}
	}
	return out
}

// ─────────────────────────────────────────────────────────────────
// LEDGER — append-only với ED25519
// ─────────────────────────────────────────────────────────────────

// LedgerEntry là một entry trong ledger
type LedgerEntry struct {
	Seq       uint64    `json:"seq"`
	Timestamp time.Time `json:"ts"`
	NodeAddr  string    `json:"addr"`  // isl.Address.String()
	Data      []byte    `json:"data"`
	Sig       []byte    `json:"sig"`   // ED25519 signature
}

// Ledger là append-only log của QR nodes
type Ledger struct {
	mu      sync.Mutex
	path    string
	entries []LedgerEntry
	seq     uint64
	privKey ed25519.PrivateKey // nil = unsigned (dev mode)
}

func NewLedger(path string, privKey ed25519.PrivateKey) *Ledger {
	return &Ledger{path: path, privKey: privKey}
}

// Append thêm entry vào ledger
// entry.Sig được tự động tạo nếu privKey != nil
func (l *Ledger) Append(node *OlangNode) error {
	l.mu.Lock(); defer l.mu.Unlock()

	// Marshal only safe fields (no pointer cycles)
	type safeNode struct {
		Addr   string `json:"addr"`
		Name   string `json:"name"`
		Status byte   `json:"status"`
		Weight int    `json:"weight"`
	}
	safe := safeNode{Addr: node.Addr.String(), Name: node.Name, Status: byte(node.Status), Weight: node.Weight}
	data, err := json.Marshal(safe)
	if err != nil {
		return fmt.Errorf("ledger: marshal: %w", err)
	}

	l.seq++
	entry := LedgerEntry{
		Seq:      l.seq,
		Timestamp: time.Now(),
		NodeAddr: node.Addr.String(),
		Data:     data,
	}

	if l.privKey != nil {
		// Sign: seq(8B) + timestamp(8B) + data
		msg := make([]byte, 16+len(data))
		binary.BigEndian.PutUint64(msg[:8], entry.Seq)
		binary.BigEndian.PutUint64(msg[8:16], uint64(entry.Timestamp.Unix()))
		copy(msg[16:], data)
		entry.Sig = ed25519.Sign(l.privKey, msg)
	}

	l.entries = append(l.entries, entry)

	// Persist nếu có path
	if l.path != "" {
		return l.persistEntry(entry)
	}
	return nil
}

func (l *Ledger) persistEntry(e LedgerEntry) error {
	f, err := os.OpenFile(l.path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return err
	}
	defer f.Close()
	enc := json.NewEncoder(f)
	return enc.Encode(e)
}

// Count trả về số entries
func (l *Ledger) Count() int {
	l.mu.Lock(); defer l.mu.Unlock()
	return len(l.entries)
}
