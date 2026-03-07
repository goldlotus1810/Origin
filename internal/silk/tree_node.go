// internal/tree/tree.go
// Silk Web Knowledge Tree — cây tri thức tự tổ chức
// Thân (UTF-32 bất biến) → Cành → Nhánh → Lá (học được)
// Lá tương đồng kết nối bằng "sợi tơ" ngữ nghĩa

package tree

import (
	"crypto/rand"
	"crypto/ed25519"
	"encoding/json"
	"fmt"
	"math"
	"sync"
	"time"

	"github.com/goldlotus1810/HomeOS/internal/isl"
)

// NodeType phân loại node trong cây
type NodeType byte

const (
	NodeTrunk     NodeType = 0x01 // Thân — UTF-32 bất biến tuyệt đối
	NodeBranch    NodeType = 0x02 // Cành — bất biến sau khi promote
	NodeLeaf      NodeType = 0x03 // Lá — học được, thay đổi được
	NodeImmutable NodeType = 0x04 // Bất biến — ký số bởi LeoAI, ưu tiên tuyệt đối
)

// Ngưỡng tự động thay đổi cấu trúc cây
const (
	BranchThreshold    = 60  // Lá đủ nhiều → promote thành Nhánh
	ImmutableThreshold = 200 // Nhánh đủ lâu → đề xuất bất biến
	SilkLinkThreshold  = 10  // Dưới ngưỡng này → kết nối sợi tơ thay vì tự đứng
)

// Node là đơn vị cơ bản của cây tri thức
type Node struct {
	mu sync.RWMutex

	// Định danh
	ID      string      // UUID
	ISLAddr isl.Address // Địa chỉ ISL của node này

	// Phân loại
	Type      NodeType
	Immutable bool // true = không thể ghi đè
	Signature []byte // ED25519 signature của LeoAI (chỉ khi Immutable=true)

	// Cấu trúc cây
	Parent   *Node
	Children []*Node

	// Sợi tơ — semantic links đến các lá tương đồng ở nhánh khác
	SilkLinks []string // ISL address strings của các node liên kết

	// Metadata học tập
	Weight    int       // Số lần được xác nhận/truy cập
	CreatedAt time.Time
	UpdatedAt time.Time

	// Dữ liệu thực của lá
	Payload []byte // Dữ liệu thô (JSON, binary...)

	// Vector 3D cho visualization
	Vec3D [3]float32
}

// CanOverwrite kiểm tra node có thể bị ghi đè không
func (n *Node) CanOverwrite(incoming *Node) bool {
	n.mu.RLock()
	defer n.mu.RUnlock()
	if n.Immutable {
		return false // Bất biến = không bao giờ bị ghi đè
	}
	return true
}

// AddChild thêm node con
func (n *Node) AddChild(child *Node) {
	n.mu.Lock()
	defer n.mu.Unlock()
	child.Parent = n
	n.Children = append(n.Children, child)
}

// AddSilkLink thêm sợi tơ đến node khác
func (n *Node) AddSilkLink(targetAddr isl.Address) {
	n.mu.Lock()
	defer n.mu.Unlock()
	addrStr := targetAddr.String()
	for _, existing := range n.SilkLinks {
		if existing == addrStr {
			return // Đã có rồi
		}
	}
	n.SilkLinks = append(n.SilkLinks, addrStr)
}

// ─────────────────────────────────────────────────────────────────
// TREE — cây tri thức tổng thể
// ─────────────────────────────────────────────────────────────────

// Tree là Silk Web Knowledge Tree đầy đủ
type Tree struct {
	mu      sync.RWMutex
	root    *Node              // Gốc ảo
	index   map[string]*Node   // ISL address → Node (O(1) lookup)
	sigKey  ed25519.PrivateKey // LeoAI signing key
}

// NewTree khởi tạo cây mới
func NewTree(sigKey ed25519.PrivateKey) *Tree {
	root := &Node{
		ID:        "root",
		Type:      NodeTrunk,
		Immutable: true,
		CreatedAt: time.Now(),
	}
	t := &Tree{
		root:   root,
		index:  make(map[string]*Node),
		sigKey: sigKey,
	}
	return t
}

// ─────────────────────────────────────────────────────────────────
// INGEST — nạp dữ liệu vào cây
// ─────────────────────────────────────────────────────────────────

// DataPoint là dữ liệu đầu vào để nạp vào cây
type DataPoint struct {
	ISLAddr    isl.Address
	Payload    []byte
	Immutable  bool   // Người dùng đánh dấu bất biến
	SourceLang string // "vi", "en", "zh"...
}

// Ingest nạp một DataPoint vào cây
// Đây là thuật toán self-organizing clustering cốt lõi
func (t *Tree) Ingest(dp DataPoint) error {
	t.mu.Lock()
	defer t.mu.Unlock()

	addrStr := dp.ISLAddr.String()

	// Nếu đã tồn tại node này
	if existing, ok := t.index[addrStr]; ok {
		existing.mu.Lock()
		if !existing.CanOverwrite(nil) {
			existing.mu.Unlock()
			return fmt.Errorf("tree: node %s is immutable, cannot overwrite", addrStr)
		}
		existing.Weight++
		existing.Payload = dp.Payload
		existing.UpdatedAt = time.Now()
		weight := existing.Weight
		existing.mu.Unlock()

		// Kiểm tra ngưỡng promote
		t.checkPromote(existing, weight)
		return nil
	}

	// Tạo node lá mới
	leaf := &Node{
		ID:        generateID(),
		ISLAddr:   dp.ISLAddr,
		Type:      NodeLeaf,
		Immutable: false,
		Weight:    1,
		Payload:   dp.Payload,
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
	}

	// Nếu dữ liệu bất biến — xử lý đặc biệt
	if dp.Immutable {
		return t.ingestImmutable(leaf)
	}

	// Tìm nhánh cha phù hợp
	parent := t.findOrCreateBranch(dp.ISLAddr)
	parent.AddChild(leaf)
	t.index[addrStr] = leaf

	// Nếu weight quá thấp → kết nối sợi tơ thay vì đứng một mình
	if leaf.Weight < SilkLinkThreshold {
		if nearest := t.findNearestBranch(leaf); nearest != nil {
			leaf.AddSilkLink(nearest.ISLAddr)
		}
	}

	// Cập nhật vector 3D cho visualization
	leaf.Vec3D = t.projectTo3D(dp.ISLAddr)

	return nil
}

// ingestImmutable xử lý dữ liệu bất biến — ưu tiên tuyệt đối
func (t *Tree) ingestImmutable(node *Node) error {
	// Ký số bằng LeoAI private key
	sig := ed25519.Sign(t.sigKey, node.Payload)
	node.Signature = sig
	node.Immutable = true
	node.Type = NodeImmutable
	node.Weight = ImmutableThreshold // Đặt thẳng vào ngưỡng bất biến

	// Đặt ở đầu nhánh tương ứng — ưu tiên tuyệt đối
	parent := t.findOrCreateBranch(node.ISLAddr)
	// Chèn vào đầu danh sách children
	parent.mu.Lock()
	parent.Children = append([]*Node{node}, parent.Children...)
	parent.mu.Unlock()

	node.Parent = parent
	t.index[node.ISLAddr.String()] = node
	return nil
}

// ─────────────────────────────────────────────────────────────────
// CLUSTERING — tự động phân cụm
// ─────────────────────────────────────────────────────────────────

// checkPromote kiểm tra và promote node nếu đủ ngưỡng
func (t *Tree) checkPromote(node *Node, weight int) {
	if weight < BranchThreshold {
		return
	}
	if node.Type >= NodeBranch {
		return // Đã là Nhánh hoặc cao hơn rồi
	}

	// Promote Lá → Nhánh
	node.mu.Lock()
	node.Type = NodeBranch
	node.mu.Unlock()

	// Thông báo LeoAI về node mới được promote
	// (LeoAI sẽ quyết định có đề xuất bất biến không nếu weight >= ImmutableThreshold)
}

// findNearestBranch tìm nhánh gần nhất theo ngữ nghĩa ISL
func (t *Tree) findNearestBranch(node *Node) *Node {
	// Tìm node có cùng Layer và Group nhưng là Branch
	targetLayer := node.ISLAddr.Layer
	targetGroup := node.ISLAddr.Group

	var best *Node
	var bestScore int

	for _, n := range t.index {
		if n.Type < NodeBranch || n == node {
			continue
		}
		score := 0
		if n.ISLAddr.Layer == targetLayer { score += 2 }
		if n.ISLAddr.Group == targetGroup { score++ }
		if score > bestScore {
			bestScore = score
			best = n
		}
	}
	return best
}

// findOrCreateBranch tìm hoặc tạo nhánh cha cho một địa chỉ ISL
func (t *Tree) findOrCreateBranch(addr isl.Address) *Node {
	// Tìm nhánh theo Layer+Group
	branchKey := fmt.Sprintf("%c%c", addr.Layer, addr.Group)

	if branch, ok := t.index[branchKey]; ok {
		return branch
	}

	// Tạo nhánh mới
	branch := &Node{
		ID:        generateID(),
		ISLAddr:   isl.Address{Layer: addr.Layer, Group: addr.Group},
		Type:      NodeBranch,
		Immutable: true, // Nhánh bất biến sau khi được tạo
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
	}
	t.root.AddChild(branch)
	t.index[branchKey] = branch
	return branch
}

// ─────────────────────────────────────────────────────────────────
// QUERY — tìm kiếm trong cây
// ─────────────────────────────────────────────────────────────────

// QueryResult là kết quả truy vấn
type QueryResult struct {
	Node       *Node
	Confidence float32
	Path       []string // Đường đi từ gốc đến node
	SilkPath   []string // Các node liên quan qua sợi tơ
}

// Query tìm kiếm node theo ISL address, theo dõi sợi tơ
// depth: độ sâu tìm kiếm tối đa; maxNodes: giới hạn số node trả về
func (t *Tree) Query(addr isl.Address, depth int, maxNodes int) []QueryResult {
	t.mu.RLock()
	defer t.mu.RUnlock()

	addrStr := addr.String()
	node, ok := t.index[addrStr]
	if !ok {
		return nil
	}

	var results []QueryResult
	visited := make(map[string]bool)

	// BFS từ node tìm được
	queue := []struct {
		node  *Node
		depth int
		conf  float32
	}{{node, 0, 1.0}}

	for len(queue) > 0 && len(results) < maxNodes {
		item := queue[0]
		queue = queue[1:]

		if visited[item.node.ID] || item.depth > depth {
			continue
		}
		visited[item.node.ID] = true

		results = append(results, QueryResult{
			Node:       item.node,
			Confidence: item.conf,
		})

		// Theo sợi tơ — độ tin cậy giảm dần
		item.node.mu.RLock()
		for _, silkAddr := range item.node.SilkLinks {
			if linked, ok := t.index[silkAddr]; ok && !visited[linked.ID] {
				queue = append(queue, struct {
					node  *Node
					depth int
					conf  float32
				}{linked, item.depth + 1, item.conf * 0.85})
			}
		}
		item.node.mu.RUnlock()
	}

	return results
}

// ─────────────────────────────────────────────────────────────────
// VECTOR 3D — cho visualization
// ─────────────────────────────────────────────────────────────────

// projectTo3D ánh xạ ISL address vào không gian 3D
// Vị trí = tổng hợp 4 chiều ngữ nghĩa của Attributes
// Cùng hình dạng → gần nhau trên trục X
// Cùng âm thanh  → gần nhau trên trục Y
// Cùng ý nghĩa   → gần nhau trên trục Z
// Nguồn gốc      → góc quay quanh trục Y (script family)
//
// Kết quả: 'A' và '△' gần nhau (cùng ShapeHash "đỉnh nhọn")
//           'A' và 'α' gần nhau (cùng PhonemeClass /a/)
//           'A' và '1' gần nhau (cùng ConceptGroup "đầu tiên")
func (t *Tree) projectTo3D(addr isl.Address) [3]float32 {
	// X: hình dạng SDF — curved vs angular vs compound
	shapeX := float64(addr.ShapeHash()) / 255.0 * 4.0 - 2.0

	// Y: âm thanh — IPA feature space
	phonemeY := float64(addr.PhonemeClass()) / 255.0 * 4.0 - 2.0

	// Z: ý nghĩa — concept cluster
	conceptZ := float64(addr.ConceptGroup()) / 255.0 * 4.0 - 2.0

	// Góc quay theo script family (Derivation bits 7-4)
	scriptFamily := float64(addr.Derivation()>>4) / 16.0 * 2 * math.Pi
	r := 0.5 // bán kính nhỏ, tránh overlap

	return [3]float32{
		float32(shapeX + r*math.Cos(scriptFamily)),
		float32(phonemeY),
		float32(conceptZ + r*math.Sin(scriptFamily)),
	}
}

// ExportFor3D xuất toàn bộ cây dạng JSON cho web visualization
func (t *Tree) ExportFor3D() []byte {
	t.mu.RLock()
	defer t.mu.RUnlock()

	type ExportNode struct {
		ID        string    `json:"id"`
		ISLAddr   string    `json:"isl_addr"`
		Type      string    `json:"type"`
		Immutable bool      `json:"immutable"`
		Vec3D     [3]float32 `json:"vec3d"`
		Weight    int       `json:"weight"`
		SilkLinks []string  `json:"silk_links"`
	}

	typeNames := map[NodeType]string{
		NodeTrunk:     "trunk",
		NodeBranch:    "branch",
		NodeLeaf:      "leaf",
		NodeImmutable: "immutable",
	}

	var nodes []ExportNode
	for _, n := range t.index {
		n.mu.RLock()
		nodes = append(nodes, ExportNode{
			ID:        n.ID,
			ISLAddr:   n.ISLAddr.String(),
			Type:      typeNames[n.Type],
			Immutable: n.Immutable,
			Vec3D:     n.Vec3D,
			Weight:    n.Weight,
			SilkLinks: n.SilkLinks,
		})
		n.mu.RUnlock()
	}

	data, _ := json.Marshal(nodes)
	return data
}

// ─────────────────────────────────────────────────────────────────
// HELPERS
// ─────────────────────────────────────────────────────────────────

func generateID() string {
	b := make([]byte, 8)
	_, _ = rand.Read(b)
	return fmt.Sprintf("%x", b)
}
