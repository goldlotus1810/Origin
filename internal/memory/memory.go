// internal/memory/memory.go
// Memory — ΔΨ ShortTerm (Dendrites) + QR LongTerm (Axon)
// ΔΨ = đang học, chưa chứng minh — tự do thay đổi
// QR = đã chứng minh, ED25519 signed — append-only

package memory

import (
	"context"
	"log"
	"sync"
	"time"

	"github.com/goldlotus1810/HomeOS/internal/isl"
	"github.com/goldlotus1810/HomeOS/internal/silk"
)

// ─────────────────────────────────────────────────────────────────
// MEMORY ENTRY
// ─────────────────────────────────────────────────────────────────

// MemoryEntry là một đơn vị tri thức
type MemoryEntry struct {
	ISL        isl.Address
	Data       []byte
	Confidence float64   // 0.0..1.0
	CreatedAt  time.Time
	UpdatedAt  time.Time
	Confirmed  int       // số lần user xác nhận
	Rejected   int       // số lần user bác bỏ
}

// ─────────────────────────────────────────────────────────────────
// SHORT TERM — ΔΨ Dendrites
// Ring buffer 512 entries, tự do thay đổi
// ─────────────────────────────────────────────────────────────────

// ShortTerm là bộ nhớ ngắn hạn — Dendrites
type ShortTerm struct {
	mu  sync.Mutex
	buf []MemoryEntry
	max int
}

func NewShortTerm(maxSize int) *ShortTerm {
	return &ShortTerm{
		buf: make([]MemoryEntry, 0, maxSize),
		max: maxSize,
	}
}

// Push thêm entry — nếu đầy thì xóa entry cũ nhất
func (st *ShortTerm) Push(e MemoryEntry) {
	st.mu.Lock(); defer st.mu.Unlock()
	e.CreatedAt = time.Now()
	e.UpdatedAt = time.Now()
	// Update nếu đã có addr này
	for i, existing := range st.buf {
		if existing.ISL.Uint64() == e.ISL.Uint64() {
			st.buf[i] = e
			return
		}
	}
	if len(st.buf) >= st.max {
		st.buf = st.buf[1:] // xóa cũ nhất
	}
	st.buf = append(st.buf, e)
}

// Filter lọc entries có confidence >= minConf
func (st *ShortTerm) Filter(minConf float64) []MemoryEntry {
	st.mu.Lock(); defer st.mu.Unlock()
	var out []MemoryEntry
	for _, e := range st.buf {
		if e.Confidence >= minConf {
			out = append(out, e)
		}
	}
	return out
}

// Delete xóa entry theo addr (khi user bác bỏ — QT7 correction)
func (st *ShortTerm) Delete(addr isl.Address) {
	st.mu.Lock(); defer st.mu.Unlock()
	key := addr.Uint64()
	newBuf := st.buf[:0]
	for _, e := range st.buf {
		if e.ISL.Uint64() != key {
			newBuf = append(newBuf, e)
		}
	}
	st.buf = newBuf
}

// Confirm tăng confidence khi user xác nhận
func (st *ShortTerm) Confirm(addr isl.Address) {
	st.mu.Lock(); defer st.mu.Unlock()
	for i, e := range st.buf {
		if e.ISL.Uint64() == addr.Uint64() {
			st.buf[i].Confirmed++
			st.buf[i].UpdatedAt = time.Now()
			// confidence tăng theo số lần confirm
			c := float64(e.Confirmed) / float64(e.Confirmed+e.Rejected+1)
			st.buf[i].Confidence = c
		}
	}
}

// Count trả về số entries hiện tại
func (st *ShortTerm) Count() int {
	st.mu.Lock(); defer st.mu.Unlock()
	return len(st.buf)
}

// ─────────────────────────────────────────────────────────────────
// LONG TERM — QR Axon
// Append-only, ED25519 signed, lưu vào SilkGraph + Ledger
// ─────────────────────────────────────────────────────────────────

// LongTerm là bộ nhớ dài hạn — Axon
type LongTerm struct {
	graph  *silk.SilkGraph
	ledger *silk.Ledger
}

func NewLongTerm(graph *silk.SilkGraph, ledger *silk.Ledger) *LongTerm {
	return &LongTerm{graph: graph, ledger: ledger}
}

// Commit ghi MemoryEntry vào QR — chỉ được gọi khi conf >= 0.8
func (lt *LongTerm) Commit(e MemoryEntry) error {
	if e.Confidence < 0.8 {
		return nil // chưa đủ confidence
	}

	node := &silk.OlangNode{
		Addr:   e.ISL,
		Status: silk.StatusQR,
		Weight: e.Confirmed,
		Name:   e.ISL.String(),
	}

	// Append vào ledger (ED25519 signed)
	if err := lt.ledger.Append(node); err != nil {
		return err
	}

	// Thêm vào SilkGraph
	lt.graph.AddNode(node)
	return nil
}

// Query tìm node trong QR graph
func (lt *LongTerm) Query(addr isl.Address) (*silk.OlangNode, bool) {
	return lt.graph.Get(addr)
}

// ─────────────────────────────────────────────────────────────────
// DREAM — kiểm chứng ΔΨ khi idle
// Chạy khi inbox rảnh > 5 phút
// Kiểm chứng với QR: pass → promote / fail → xóa
// ─────────────────────────────────────────────────────────────────

// Dream là goroutine dreaming — NCA idle processing
type Dream struct {
	dn       *ShortTerm
	qr       *LongTerm
	idleWait time.Duration
	lastMsg  time.Time
	mu       sync.Mutex
}

func NewDream(dn *ShortTerm, qr *LongTerm) *Dream {
	return &Dream{
		dn:       dn,
		qr:       qr,
		idleWait: 5 * time.Minute,
		lastMsg:  time.Now(),
	}
}

// Poke thông báo có message mới (reset idle timer)
func (d *Dream) Poke() {
	d.mu.Lock(); defer d.mu.Unlock()
	d.lastMsg = time.Now()
}

// Run là vòng lặp dreaming — chạy như goroutine
func (d *Dream) Run(ctx context.Context) {
	ticker := time.NewTicker(30 * time.Second)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			d.mu.Lock()
			idle := time.Since(d.lastMsg)
			d.mu.Unlock()
			if idle >= d.idleWait {
				d.process()
			}
		}
	}
}

// process kiểm chứng ΔΨ entries
func (d *Dream) process() {
	candidates := d.dn.Filter(0.5) // chỉ xem xét conf >= 0.5
	promoted, deleted := 0, 0

	for _, e := range candidates {
		// Kiểm chứng: nếu conf >= 0.8 → promote lên QR
		if e.Confidence >= 0.8 {
			if err := d.qr.Commit(e); err == nil {
				d.dn.Delete(e.ISL)
				promoted++
			}
		} else if e.Confidence < 0.3 && e.Confirmed+e.Rejected > 5 {
			// Quá ít xác nhận sau nhiều lần check → xóa
			d.dn.Delete(e.ISL)
			deleted++
		}
	}

	if promoted+deleted > 0 {
		log.Printf("Dream: promoted=%d deleted=%d remaining=%d",
			promoted, deleted, d.dn.Count())
	}
}
