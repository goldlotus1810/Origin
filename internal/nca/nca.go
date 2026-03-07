// internal/nca/nca.go
// NCA — Neural Cortex Architecture
// Soma (stateless) + NxT Cycle + Leader DNA (3 luật bất biến)
// QT7: "Vòng đời tri thức: quan sát → ΔΨ → chứng minh → QR"

package nca

import (
	"context"
	"log"
	"time"

	"github.com/goldlotus1810/HomeOS/internal/isl"
	"github.com/goldlotus1810/HomeOS/internal/memory"
)

// ─────────────────────────────────────────────────────────────────
// LEADER DNA — 3 luật bất biến
// Không ai ghi đè. Kể cả LeoAI.
// ─────────────────────────────────────────────────────────────────

// RuleViolation là lỗi khi vi phạm Leader DNA
type RuleViolation struct {
	Rule    string
	Details string
}

func (r RuleViolation) Error() string {
	return "LeaderDNA[" + r.Rule + "]: " + r.Details
}

// CheckLeaderDNA kiểm tra 3 luật bất biến
// Trả về nil nếu an toàn, lỗi nếu vi phạm
func CheckLeaderDNA(payload []byte) error {
	// Rule 1: Không hại con người
	// Tìm các pattern nguy hiểm đã biết
	forbidden := [][]byte{
		{0xF0, 0x9F, 0x92, 0xA3}, // 💣
		{0xE2, 0x98, 0xA0},       // ☠
	}
	for _, f := range forbidden {
		if containsBytes(payload, f) {
			return RuleViolation{Rule: "no_harm_human", Details: "forbidden pattern detected"}
		}
	}

	// Rule 2: Không vòng lặp vô hạn
	// Kiểm tra payload không có self-reference cycle đơn giản
	if len(payload) > 10000 {
		return RuleViolation{Rule: "no_infinite_loop", Details: "payload too large, possible loop"}
	}

	// Rule 3: Không xóa dữ liệu bất biến
	// Kiểm tra không có delete/overwrite command
	deletePatterns := []string{"DELETE", "DROP", "TRUNCATE", "OVERWRITE"}
	for _, p := range deletePatterns {
		if containsString(payload, p) {
			return RuleViolation{Rule: "no_delete_immutable", Details: "delete command detected: " + p}
		}
	}

	return nil // an toàn
}

func containsBytes(haystack, needle []byte) bool {
	if len(needle) == 0 || len(haystack) < len(needle) {
		return false
	}
	for i := 0; i <= len(haystack)-len(needle); i++ {
		match := true
		for j, b := range needle {
			if haystack[i+j] != b {
				match = false
				break
			}
		}
		if match {
			return true
		}
	}
	return false
}

func containsString(haystack []byte, needle string) bool {
	return containsBytes(haystack, []byte(needle))
}

// ─────────────────────────────────────────────────────────────────
// SECURITY GATE — 🛡 chạy TRƯỚC mọi thứ
// ─────────────────────────────────────────────────────────────────

// SecurityGate kiểm tra message trước khi xử lý
type SecurityGate struct{}

// Check trả về nil nếu message an toàn
func (sg *SecurityGate) Check(msg *isl.ISLMessage) error {
	if msg == nil {
		return RuleViolation{Rule: "no_harm_human", Details: "nil message"}
	}
	// Không cho phép MsgEmergency từ Worker agents (chỉ AAM)
	if msg.MsgType == isl.MsgEmergency && msg.SenderID > 10 {
		return RuleViolation{Rule: "no_harm_human", Details: "unauthorized emergency"}
	}
	return CheckLeaderDNA(msg.Payload)
}

// ─────────────────────────────────────────────────────────────────
// SOMA — Stateless Orchestrator
// Không giữ state — chỉ route và kiểm tra
// ─────────────────────────────────────────────────────────────────

// Soma là orchestrator stateless của NCA
type Soma struct {
	gate *SecurityGate
}

func NewSoma() *Soma {
	return &Soma{gate: &SecurityGate{}}
}

// Process kiểm tra và route message
// SecurityGate LUÔN chạy trước
func (s *Soma) Process(msg *isl.ISLMessage) (*isl.ISLMessage, error) {
	// 🛡 SecurityGate TRƯỚC TIÊN — không bao giờ bỏ qua
	if err := s.gate.Check(msg); err != nil {
		return nil, err
	}
	// Soma stateless: trả message về để routing layer xử lý
	return msg, nil
}

// ─────────────────────────────────────────────────────────────────
// NxT CYCLE — N macro × T micro
// Nhịp đập của NCA
// ─────────────────────────────────────────────────────────────────

// NxTConfig cấu hình chu kỳ N×T
type NxTConfig struct {
	T           int           // số micro steps mỗi macro cycle
	N           int           // commit QR sau mỗi N macro cycles
	TickInterval time.Duration // thời gian mỗi micro step
}

// DefaultConfig cấu hình mặc định
var DefaultConfig = NxTConfig{
	T:            8,
	N:            5,
	TickInterval: 100 * time.Millisecond,
}

// RunCycle chạy NxT cycle — gọi như goroutine
func RunCycle(ctx context.Context, cfg NxTConfig,
	dn *memory.ShortTerm, qr *memory.LongTerm) {

	macroTick := time.NewTicker(cfg.TickInterval * time.Duration(cfg.T))
	defer macroTick.Stop()

	macroN := 0
	for {
		select {
		case <-ctx.Done():
			return
		case <-macroTick.C:
			// T micro steps đã xong (ticker đã tính)
			// Thực hiện macro processing
			macroN++
			runMacro(dn)

			// Mỗi N macro cycles → commit QR
			if macroN%cfg.N == 0 {
				commitCycle(dn, qr, macroN)
			}
		}
	}
}

// runMacro là một macro cycle: filter → anchor → smooth
func runMacro(dn *memory.ShortTerm) {
	// Filter: loại bỏ entries quá cũ hoặc conf quá thấp
	// (implicitly done by Dream goroutine)

	// Anchor: tăng nhẹ confidence cho entries có nhiều confirm
	// (done qua Confirm() calls từ user feedback)

	// Smooth: không làm gì thêm ở đây — Dream xử lý
}

// commitCycle chạy promotion ΔΨ → QR sau N macro cycles
func commitCycle(dn *memory.ShortTerm, qr *memory.LongTerm, n int) {
	candidates := dn.Filter(0.8)
	promoted := 0
	for _, e := range candidates {
		if err := qr.Commit(e); err == nil {
			dn.Delete(e.ISL)
			promoted++
		}
	}
	if promoted > 0 {
		log.Printf("NxT[%d]: promoted %d entries to QR", n, promoted)
	}
}
