// internal/ws/ws.go
package ws

import (
	"encoding/json"
	"fmt"
	"log"
	"math"
	"net/http"
	"sync"
	"time"

	"github.com/goldlotus1810/HomeOS/internal/gene"
	"github.com/goldlotus1810/HomeOS/internal/isl"
	"github.com/goldlotus1810/HomeOS/internal/silk"
)

type WorldSnapshot struct {
	Time    float64     `json:"time"`
	Nodes   []NodeState `json:"nodes"`
	Signals []SigState  `json:"signals"`
	Stats   StatsState  `json:"stats"`
}
type NodeState struct {
	ID   string   `json:"id"`
	Glow float64  `json:"glow,omitempty"`
	Sig  float64  `json:"sig,omitempty"`
	X    *float64 `json:"x,omitempty"`
	Y    *float64 `json:"y,omitempty"`
	Z    *float64 `json:"z,omitempty"`
}
type SigState struct {
	From string  `json:"from"`
	To   string  `json:"to"`
	P    float64 `json:"p"`
}
type StatsState struct {
	QR int `json:"qr"`
	DN int `json:"dn"`
	NT int `json:"nt"`
}

type World struct {
	mu      sync.RWMutex
	Seed    int64
	Time    float64
	Genes   []gene.Gene
	Light   gene.LightInfo
	Graph   *silk.SilkGraph
	glowing map[string]float64
	signals []SigState
	nt      int
}

func NewWorld(seed int64, graph *silk.SilkGraph) *World {
	w := &World{Seed: seed, Time: 10.0, Graph: graph, glowing: make(map[string]float64)}
	w.Light = gene.SunLight(w.Time)
	return w
}

func (w *World) Tick(dt float64) {
	w.mu.Lock(); defer w.mu.Unlock()
	w.Time = modF(w.Time+dt, 24.0)
	w.Light = gene.SunLight(w.Time)
	w.nt++
	for _, g := range w.Genes { g.Animate(w.Time) }
	for k, v := range w.glowing {
		v -= 0.015
		if v <= 0 { delete(w.glowing, k) } else { w.glowing[k] = v }
	}
	active := w.signals[:0]
	for _, s := range w.signals { s.P += 0.018; if s.P < 1.0 { active = append(active, s) } }
	w.signals = active
}

func (w *World) Glow(id string) { w.mu.Lock(); w.glowing[id] = 1.0; w.mu.Unlock() }

func (w *World) FireSignal(from, to string) {
	w.mu.Lock(); w.signals = append(w.signals, SigState{From: from, To: to}); w.mu.Unlock()
}

func (w *World) Snapshot(qr, dn int) WorldSnapshot {
	w.mu.RLock(); defer w.mu.RUnlock()
	var nodes []NodeState
	for id, glow := range w.glowing { nodes = append(nodes, NodeState{ID: id, Glow: glow}) }
	sigs := make([]SigState, len(w.signals)); copy(sigs, w.signals)
	return WorldSnapshot{Time: w.Time, Nodes: nodes, Signals: sigs, Stats: StatsState{QR: qr, DN: dn, NT: w.nt}}
}

func (w *World) SDF(p gene.Vec3) float64 {
	w.mu.RLock(); defer w.mu.RUnlock()
	d := math.Inf(1)
	for _, g := range w.Genes { d = gene.SmoothUnion(d, g.SDF(p), 0.1) }
	return d
}

func modF(x, m float64) float64 { r := math.Mod(x, m); if r < 0 { r += m }; return r }

type Server struct {
	world      *World
	mu         sync.RWMutex
	clients    map[string]chan []byte
	qr, dn     *int
	staticPath string
}

func NewServer(world *World, qr, dn *int, staticPath string) *Server {
	return &Server{world: world, clients: make(map[string]chan []byte), qr: qr, dn: dn, staticPath: staticPath}
}

func (s *Server) Serve(addr string) error {
	mux := http.NewServeMux()
	cors := func(h http.HandlerFunc) http.HandlerFunc {
		return func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Access-Control-Allow-Origin", "*")
			if r.Method == "OPTIONS" { return }
			h(w, r)
		}
	}
	mux.HandleFunc("/health", cors(s.handleHealth))
	mux.HandleFunc("/api/tree", cors(s.handleTree))
	mux.HandleFunc("/api/edges", cors(s.handleEdges))
	mux.HandleFunc("/ws/sse", cors(s.handleSSE))
	if s.staticPath != "" { mux.Handle("/", http.FileServer(http.Dir(s.staticPath))) }
	go s.broadcastLoop()
	log.Printf("HomeOS: %s", addr)
	return http.ListenAndServe(addr, mux)
}

func (s *Server) handleHealth(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	fmt.Fprintf(w, `{"status":"ok","time":%.2f}`, s.world.Time)
}

func (s *Server) handleTree(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	type nr struct {
		ID    string `json:"id"`
		Name  string `json:"name"`
		Glyph string `json:"glyph"`
		Cat   string `json:"cat"`
		Layer int    `json:"layer"`
	}
	nodes := s.world.Graph.AllNodes()
	resp := make([]nr, 0, len(nodes))
	for _, n := range nodes { resp = append(resp, nr{ID: n.Name, Name: n.Name, Glyph: n.Glyph, Cat: n.Cat, Layer: n.Layer}) }
	json.NewEncoder(w).Encode(resp)
}

func (s *Server) handleEdges(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	type er struct {
		From string `json:"from"`
		To   string `json:"to"`
		Op   string `json:"op"`
	}
	raw := s.world.Graph.AllEdges()
	resp := make([]er, 0, len(raw))
	for _, e := range raw {
		from := e[0].(isl.Address)
		to   := e[1].(isl.Address)
		op   := string(rune(e[2].(silk.EdgeOp)))
		fn, to_n := from.String(), to.String()
		if n, ok := s.world.Graph.Get(from); ok && n.Name != "" { fn = n.Name }
		if n, ok := s.world.Graph.Get(to);   ok && n.Name != "" { to_n = n.Name }
		resp = append(resp, er{From: fn, To: to_n, Op: op})
	}
	json.NewEncoder(w).Encode(resp)
}

func (s *Server) handleSSE(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	flusher, ok := w.(http.Flusher)
	if !ok { http.Error(w, "streaming unsupported", 500); return }
	id := fmt.Sprintf("%d", time.Now().UnixNano())
	ch := make(chan []byte, 32)
	s.mu.Lock(); s.clients[id] = ch; s.mu.Unlock()
	defer func() { s.mu.Lock(); delete(s.clients, id); s.mu.Unlock() }()
	for {
		select {
		case <-r.Context().Done(): return
		case data := <-ch:
			fmt.Fprintf(w, "data: %s\n\n", data)
			flusher.Flush()
		}
	}
}

func (s *Server) broadcastLoop() {
	t := time.NewTicker(50 * time.Millisecond)
	defer t.Stop()
	for range t.C {
		s.world.Tick(0.004)
		snap := s.world.Snapshot(*s.qr, *s.dn)
		data, err := json.Marshal(snap)
		if err != nil { continue }
		s.mu.RLock()
		for _, ch := range s.clients { select { case ch <- data: default: } }
		s.mu.RUnlock()
	}
}
