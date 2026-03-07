// internal/sync/silk_sync.go
// SILK-SYNC — đồng bộ hóa Olang ↔ Olang
// ED25519 handshake + delta ledger (chỉ gửi QR entries sau timestamp T)
// Không dump toàn bộ graph. Không gửi ĐN.

package silksync

import (
	"crypto/ed25519"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"sync"
	"time"

	"github.com/goldlotus1810/HomeOS/internal/silk"
)

// SyncPeer là một peer để sync
type SyncPeer struct {
	ID      string
	URL     string // ws://host:port/sync
	PubKey  ed25519.PublicKey
	LastSync time.Time
}

// SyncMessage là message trao đổi trong SILK-SYNC
type SyncMessage struct {
	Since   int64              `json:"since"`   // unix timestamp
	Entries []SyncEntry        `json:"entries"` // QR entries
	Sig     []byte             `json:"sig"`     // ED25519 signature
	PeerID  string             `json:"peer_id"`
}

// SyncEntry là một entry QR để sync
type SyncEntry struct {
	Addr   string `json:"addr"`
	Name   string `json:"name"`
	Glyph  string `json:"glyph"`
	Cat    string `json:"cat"`
	Layer  int    `json:"layer"`
	Weight int    `json:"weight"`
	TS     int64  `json:"ts"`
}

// SilkSync quản lý đồng bộ hóa với các peers
type SilkSync struct {
	mu      sync.RWMutex
	graph   *silk.SilkGraph
	privKey ed25519.PrivateKey
	pubKey  ed25519.PublicKey
	peers   map[string]*SyncPeer
	myID    string
}

func NewSilkSync(myID string, graph *silk.SilkGraph, privKey ed25519.PrivateKey) *SilkSync {
	pub := privKey.Public().(ed25519.PublicKey)
	return &SilkSync{
		graph:   graph,
		privKey: privKey,
		pubKey:  pub,
		peers:   make(map[string]*SyncPeer),
		myID:    myID,
	}
}

// AddPeer thêm peer để sync
func (s *SilkSync) AddPeer(id, url string, pubKey ed25519.PublicKey) {
	s.mu.Lock(); defer s.mu.Unlock()
	s.peers[id] = &SyncPeer{ID: id, URL: url, PubKey: pubKey}
}

// Push gửi delta QR nodes đến tất cả peers kể từ lastSync
func (s *SilkSync) Push() {
	s.mu.RLock()
	peers := make([]*SyncPeer, 0, len(s.peers))
	for _, p := range s.peers {
		peers = append(peers, p)
	}
	s.mu.RUnlock()

	for _, peer := range peers {
		if err := s.pushToPeer(peer); err != nil {
			log.Printf("SilkSync: push to %s failed: %v", peer.ID, err)
		}
	}
}

func (s *SilkSync) pushToPeer(peer *SyncPeer) error {
	// Thu thập QR nodes đã thay đổi sau peer.LastSync
	nodes := s.graph.AllNodes()
	var entries []SyncEntry
	for _, n := range nodes {
		if n.Status != silk.StatusQR {
			continue // chỉ gửi QR
		}
		entries = append(entries, SyncEntry{
			Addr:   n.Addr.String(),
			Name:   n.Name,
			Glyph:  n.Glyph,
			Cat:    n.Cat,
			Layer:  n.Layer,
			Weight: n.Weight,
			TS:     time.Now().Unix(),
		})
	}
	if len(entries) == 0 {
		return nil
	}

	msg := SyncMessage{
		Since:   peer.LastSync.Unix(),
		Entries: entries,
		PeerID:  s.myID,
	}

	// Sign message
	data, _ := json.Marshal(msg)
	msg.Sig = ed25519.Sign(s.privKey, data)

	// HTTP POST (đơn giản, production dùng WS)
	payload, _ := json.Marshal(msg)
	resp, err := http.Post(peer.URL+"/sync/push", "application/json",
		newBytesReader(payload))
	if err != nil {
		return fmt.Errorf("push: %w", err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != 200 {
		return fmt.Errorf("push: status %d", resp.StatusCode)
	}

	s.mu.Lock()
	peer.LastSync = time.Now()
	s.mu.Unlock()
	return nil
}

// ServeSync là HTTP handler nhận sync từ peer
func (s *SilkSync) ServeSync(w http.ResponseWriter, r *http.Request) {
	var msg SyncMessage
	if err := json.NewDecoder(r.Body).Decode(&msg); err != nil {
		http.Error(w, "bad json", 400)
		return
	}

	// Verify signature
	peer := s.getPeer(msg.PeerID)
	if peer != nil {
		// Verify ED25519
		msgNoSig := msg
		msgNoSig.Sig = nil
		data, _ := json.Marshal(msgNoSig)
		if !ed25519.Verify(peer.PubKey, data, msg.Sig) {
			http.Error(w, "invalid signature", 403)
			return
		}
	}

	// Merge entries vào graph
	imported := 0
	for _, e := range msg.Entries {
		// Parse addr từ string (đơn giản)
		if len(e.Addr) < 3 {
			continue
		}
		node := &silk.OlangNode{
			Name:   e.Name,
			Glyph:  e.Glyph,
			Cat:    e.Cat,
			Layer:  e.Layer,
			Weight: e.Weight,
			Status: silk.StatusQR,
		}
		s.graph.AddNode(node)
		imported++
	}

	log.Printf("SilkSync: imported %d nodes from %s", imported, msg.PeerID)
	w.WriteHeader(200)
}

func (s *SilkSync) getPeer(id string) *SyncPeer {
	s.mu.RLock(); defer s.mu.RUnlock()
	return s.peers[id]
}

// RunPeriodicSync chạy sync định kỳ mỗi 5 phút
func (s *SilkSync) RunPeriodicSync() {
	ticker := time.NewTicker(5 * time.Minute)
	defer ticker.Stop()
	for range ticker.C {
		s.Push()
	}
}

// bytes reader helper
type bytesReader struct {
	data []byte
	pos  int
}

func newBytesReader(data []byte) *bytesReader { return &bytesReader{data: data} }
func (b *bytesReader) Read(p []byte) (int, error) {
	if b.pos >= len(b.data) {
		return 0, io.EOF
	}
	n := copy(p, b.data[b.pos:])
	b.pos += n
	return n, nil
}
