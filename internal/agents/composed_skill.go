// internal/agents/composed_skill.go
//
// ComposedSkill — Skill tự sinh theo QT7
// =========================================
// Pattern: ĐN (quan sát lặp lại) → SkillProposal → QR (ComposedSkill bất biến)
//
// Luồng:
//   Agent chạy → ExecLog ghi mỗi skill execution
//   PatternMiner đọc ExecLog → tìm chuỗi lặp ≥ threshold
//   → SkillProposal { name, sequence []string, confidence float64 }
//   → SkillRegistry.Propose() → chờ confirm hoặc auto-confirm nếu conf > 0.95
//   → ComposedSkill được tạo → AddSkill → lưu Registry
//
// QT4: Agent == tập hợp Skill của nó
// QT7: pattern ĐN lặp lại → QR (ComposedSkill bất biến)

package agents

import (
	"context"
	"encoding/json"
	"fmt"
	"sort"
	"strings"
	"sync"
	"time"

	"github.com/goldlotus1810/HomeOS/internal/isl"
)

// ══════════════════════════════════════════════════════════════
// EXEC LOG — ghi lại mỗi skill execution
// ══════════════════════════════════════════════════════════════

// ExecRecord là một lần skill được gọi thành công
type ExecRecord struct {
	AgentID   string
	SkillName string
	MsgType   isl.MsgType
	At        time.Time
	Latency   time.Duration
	Success   bool
}

// ExecLog ring buffer — ghi append-only, đọc để mine patterns
type ExecLog struct {
	mu      sync.RWMutex
	records []ExecRecord
	cap     int
	head    int // write position
	size    int
}

func NewExecLog(capacity int) *ExecLog {
	return &ExecLog{
		records: make([]ExecRecord, capacity),
		cap:     capacity,
	}
}

func (l *ExecLog) Push(r ExecRecord) {
	l.mu.Lock()
	defer l.mu.Unlock()
	l.records[l.head] = r
	l.head = (l.head + 1) % l.cap
	if l.size < l.cap {
		l.size++
	}
}

// Snapshot trả về bản copy theo thứ tự thời gian (oldest → newest)
func (l *ExecLog) Snapshot() []ExecRecord {
	l.mu.RLock()
	defer l.mu.RUnlock()
	if l.size == 0 {
		return nil
	}
	out := make([]ExecRecord, l.size)
	start := (l.head - l.size + l.cap) % l.cap
	for i := 0; i < l.size; i++ {
		out[i] = l.records[(start+i)%l.cap]
	}
	return out
}

// ══════════════════════════════════════════════════════════════
// PATTERN MINER — tìm chuỗi skill lặp lại
// ══════════════════════════════════════════════════════════════

// SkillSequence là một chuỗi skill names quan sát được
type SkillSequence []string

func (s SkillSequence) Key() string { return strings.Join(s, "→") }

// PatternCount đếm số lần xuất hiện của một sequence
type PatternCount struct {
	Seq   SkillSequence
	Count int
	// window: max gap giữa các step (giây)
	AvgGapSec float64
}

// PatternMiner tìm N-gram patterns trong ExecLog
type PatternMiner struct {
	MinCount  int     // tối thiểu bao nhiêu lần để coi là pattern
	MinLen    int     // độ dài chuỗi tối thiểu
	MaxLen    int     // độ dài chuỗi tối đa
	MaxGapSec float64 // nếu gap > MaxGapSec thì cắt chuỗi
}

func NewPatternMiner() *PatternMiner {
	return &PatternMiner{
		MinCount:  3,    // xuất hiện ≥ 3 lần
		MinLen:    2,    // chuỗi ≥ 2 skill
		MaxLen:    6,    // tối đa 6 skill trong 1 composed skill
		MaxGapSec: 30.0, // gap > 30s → không cùng sequence
	}
}

// Mine tìm tất cả patterns trong records
func (m *PatternMiner) Mine(records []ExecRecord) []PatternCount {
	if len(records) < m.MinLen {
		return nil
	}

	// Tách thành sessions dựa trên gap
	sessions := m.splitIntoSessions(records)

	// Đếm N-grams
	counts := make(map[string]*PatternCount)
	for _, session := range sessions {
		names := make([]string, len(session))
		for i, r := range session {
			names[i] = r.SkillName
		}
		// Generate all N-grams từ MinLen đến MaxLen
		for n := m.MinLen; n <= m.MaxLen && n <= len(names); n++ {
			for i := 0; i+n <= len(names); i++ {
				seq := SkillSequence(names[i : i+n])
				key := seq.Key()
				if _, ok := counts[key]; !ok {
					cp := make(SkillSequence, n)
					copy(cp, seq)
					counts[key] = &PatternCount{Seq: cp}
				}
				counts[key].Count++
			}
		}
	}

	// Filter theo MinCount
	var result []PatternCount
	for _, pc := range counts {
		if pc.Count >= m.MinCount {
			result = append(result, *pc)
		}
	}

	// Sort: longer sequences first, then by count
	sort.Slice(result, func(i, j int) bool {
		if len(result[i].Seq) != len(result[j].Seq) {
			return len(result[i].Seq) > len(result[j].Seq)
		}
		return result[i].Count > result[j].Count
	})

	// Deduplicate: nếu ABC đã có thì không cần AB và BC riêng
	return m.dedup(result)
}

func (m *PatternMiner) splitIntoSessions(records []ExecRecord) [][]ExecRecord {
	var sessions [][]ExecRecord
	var cur []ExecRecord
	for i, r := range records {
		if !r.Success {
			continue
		}
		if i > 0 && len(cur) > 0 {
			prev := cur[len(cur)-1]
			gap := r.At.Sub(prev.At).Seconds()
			if gap > m.MaxGapSec {
				if len(cur) >= m.MinLen {
					sessions = append(sessions, cur)
				}
				cur = nil
			}
		}
		cur = append(cur, r)
	}
	if len(cur) >= m.MinLen {
		sessions = append(sessions, cur)
	}
	return sessions
}

func (m *PatternMiner) dedup(patterns []PatternCount) []PatternCount {
	// Loại bỏ sub-sequences của patterns dài hơn với cùng count
	result := make([]PatternCount, 0, len(patterns))
	for _, p := range patterns {
		absorbed := false
		for _, q := range patterns {
			if len(q.Seq) > len(p.Seq) && q.Count >= p.Count {
				// kiểm tra p.Seq là sub-sequence của q.Seq
				if isSubSeq(p.Seq, q.Seq) {
					absorbed = true
					break
				}
			}
		}
		if !absorbed {
			result = append(result, p)
		}
	}
	return result
}

func isSubSeq(sub, full SkillSequence) bool {
	subKey := sub.Key()
	fullKey := full.Key()
	return strings.Contains(fullKey, subKey)
}

// ══════════════════════════════════════════════════════════════
// SKILL PROPOSAL — đề xuất tạo ComposedSkill mới
// ══════════════════════════════════════════════════════════════

// ProposalStatus trạng thái của một proposal
type ProposalStatus int

const (
	ProposalPending   ProposalStatus = iota
	ProposalConfirmed                // người dùng / AAM xác nhận
	ProposalRejected                 // bị từ chối → xóa
	ProposalActive                   // đã build thành ComposedSkill
)

// SkillProposal là đề xuất tạo một ComposedSkill mới
type SkillProposal struct {
	ID         string
	Name       string         // tên đề xuất: "vision_recognize", "light_sequence"
	Sequence   []string       // skill names cần compose
	Count      int            // số lần pattern xuất hiện
	Confidence float64        // 0.0–1.0
	Status     ProposalStatus
	CreatedAt  time.Time
	ApprovedAt time.Time
}

func (p *SkillProposal) String() string {
	return fmt.Sprintf("Proposal[%s] %s conf=%.2f status=%d",
		p.ID, strings.Join(p.Sequence, "→"), p.Confidence, p.Status)
}

// Confidence tính từ count và độ dài sequence
func calcConfidence(count, seqLen, totalRecords int) float64 {
	if totalRecords == 0 {
		return 0
	}
	// Base: count / total (tần suất xuất hiện)
	freq := float64(count) / float64(totalRecords)
	// Bonus: sequence dài hơn → confidence cao hơn (có ý nghĩa hơn)
	lenBonus := float64(seqLen-1) * 0.05
	c := freq + lenBonus
	if c > 1.0 {
		c = 1.0
	}
	return c
}

// ══════════════════════════════════════════════════════════════
// COMPOSED SKILL — Skill được build từ sequence of Skills
// ══════════════════════════════════════════════════════════════

// ComposedSkill thực thi một chuỗi skills theo thứ tự
// Đây là QR — bất biến sau khi được xác nhận
type ComposedSkill struct {
	proposal *SkillProposal
	skills   []Skill // resolved skill instances
	name     string
}

// NewComposedSkill tạo ComposedSkill từ proposal và skill registry
func NewComposedSkill(p *SkillProposal, registry *SkillRegistry) (*ComposedSkill, error) {
	skills := make([]Skill, 0, len(p.Sequence))
	for _, name := range p.Sequence {
		s, ok := registry.Get(name)
		if !ok {
			return nil, fmt.Errorf("skill %q not found in registry", name)
		}
		skills = append(skills, s)
	}
	return &ComposedSkill{
		proposal: p,
		skills:   skills,
		name:     p.Name,
	}, nil
}

func (c *ComposedSkill) Name() string { return c.name }

func (c *ComposedSkill) CanHandle(msg *isl.ISLMessage) bool {
	if msg == nil {
		return false
	}
	// ComposedSkill handle nếu skill đầu tiên có thể handle
	if len(c.skills) == 0 {
		return false
	}
	return c.skills[0].CanHandle(msg)
}

func (c *ComposedSkill) Execute(ctx *ExecContext, msg *isl.ISLMessage) SkillResult {
	if len(c.skills) == 0 {
		return SkillResult{Err: fmt.Errorf("composed skill %q has no sub-skills", c.name)}
	}

	var lastResult SkillResult
	for i, s := range c.skills {
		// Mỗi skill nhận ctx với state từ skill trước
		result := s.Execute(ctx, msg)
		if result.Err != nil {
			return SkillResult{
				Err: fmt.Errorf("composed[%s] step %d (%s): %w",
					c.name, i, s.Name(), result.Err),
			}
		}
		// Merge state từ step này vào ctx cho step tiếp theo
		if result.Data != nil {
			for k, v := range result.Data {
				ctx.State[k] = v
			}
		}
		lastResult = result
	}
	if lastResult.Data == nil {
		lastResult.Data = make(map[string]interface{})
	}
	lastResult.Data["composed_skill"] = c.name
	lastResult.Data["steps_executed"] = len(c.skills)
	return lastResult
}

// ══════════════════════════════════════════════════════════════
// SKILL REGISTRY — lưu trữ tất cả skills
// ══════════════════════════════════════════════════════════════

// SkillRegistry lưu tất cả skill instances (cả primitive và composed)
type SkillRegistry struct {
	mu    sync.RWMutex
	store map[string]Skill // name → Skill
}

func NewSkillRegistry() *SkillRegistry {
	return &SkillRegistry{store: make(map[string]Skill)}
}

// Register đăng ký một skill (primitive)
func (r *SkillRegistry) Register(s Skill) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.store[s.Name()] = s
}

// Get lấy skill theo tên
func (r *SkillRegistry) Get(name string) (Skill, bool) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	s, ok := r.store[name]
	return s, ok
}

// All trả về tất cả registered skill names
func (r *SkillRegistry) All() []string {
	r.mu.RLock()
	defer r.mu.RUnlock()
	names := make([]string, 0, len(r.store))
	for k := range r.store {
		names = append(names, k)
	}
	sort.Strings(names)
	return names
}

// Propose đăng ký một ComposedSkill mới từ proposal
func (r *SkillRegistry) Propose(p *SkillProposal) error {
	cs, err := NewComposedSkill(p, r)
	if err != nil {
		return fmt.Errorf("registry: %w", err)
	}
	r.mu.Lock()
	defer r.mu.Unlock()
	r.store[p.Name] = cs
	return nil
}

// ══════════════════════════════════════════════════════════════
// SKILL LEARNER — engine tự sinh skill
// ══════════════════════════════════════════════════════════════

// SkillLearner chạy background, tự phát hiện và tạo ComposedSkill
// Đây là hiện thực của QT7 cho code:
//   ĐN (pattern lặp lại) → SkillProposal → QR (ComposedSkill bất biến)
type SkillLearner struct {
	agentID   string
	execLog   *ExecLog
	miner     *PatternMiner
	registry  *SkillRegistry
	proposals map[string]*SkillProposal // proposalID → proposal
	mu        sync.Mutex
	autoThreshold float64 // conf > autoThreshold → auto confirm
	onPropose     func(*SkillProposal) // callback khi có proposal mới
	onActivate    func(*SkillProposal) // callback khi skill được activate
}

func NewSkillLearner(agentID string, registry *SkillRegistry) *SkillLearner {
	return &SkillLearner{
		agentID:       agentID,
		execLog:       NewExecLog(1024),
		miner:         NewPatternMiner(),
		registry:      registry,
		proposals:     make(map[string]*SkillProposal),
		autoThreshold: 0.95, // tự động tạo skill nếu confidence >= 95%
	}
}

// OnPropose đặt callback khi có proposal mới (để AAM thông báo user)
func (l *SkillLearner) OnPropose(fn func(*SkillProposal)) { l.onPropose = fn }

// OnActivate đặt callback khi skill được activate
func (l *SkillLearner) OnActivate(fn func(*SkillProposal)) { l.onActivate = fn }

// RecordExec ghi lại một lần skill execution (gọi từ Agent.Run)
func (l *SkillLearner) RecordExec(agentID, skillName string, msgType isl.MsgType, latency time.Duration, success bool) {
	l.execLog.Push(ExecRecord{
		AgentID:   agentID,
		SkillName: skillName,
		MsgType:   msgType,
		At:        time.Now(),
		Latency:   latency,
		Success:   success,
	})
}

// Confirm người dùng / AAM xác nhận một proposal
func (l *SkillLearner) Confirm(proposalID string) error {
	l.mu.Lock()
	defer l.mu.Unlock()
	p, ok := l.proposals[proposalID]
	if !ok {
		return fmt.Errorf("proposal %q not found", proposalID)
	}
	if p.Status != ProposalPending {
		return fmt.Errorf("proposal %q not pending (status=%d)", proposalID, p.Status)
	}
	return l.activate(p)
}

// Reject từ chối một proposal
func (l *SkillLearner) Reject(proposalID string) {
	l.mu.Lock()
	defer l.mu.Unlock()
	if p, ok := l.proposals[proposalID]; ok {
		p.Status = ProposalRejected
	}
}

// Proposals trả về tất cả proposals đang pending
func (l *SkillLearner) Proposals() []*SkillProposal {
	l.mu.Lock()
	defer l.mu.Unlock()
	var out []*SkillProposal
	for _, p := range l.proposals {
		if p.Status == ProposalPending {
			out = append(out, p)
		}
	}
	return out
}

// ActiveSkills trả về tất cả composed skills đã active
func (l *SkillLearner) ActiveSkills() []*SkillProposal {
	l.mu.Lock()
	defer l.mu.Unlock()
	var out []*SkillProposal
	for _, p := range l.proposals {
		if p.Status == ProposalActive {
			out = append(out, p)
		}
	}
	return out
}

// Run chạy background learning loop
func (l *SkillLearner) Run(ctx context.Context) {
	ticker := time.NewTicker(60 * time.Second) // mine mỗi 60 giây
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			l.mine()
		}
	}
}

// mine là core: đọc execLog → tìm patterns → tạo proposals
func (l *SkillLearner) mine() {
	records := l.execLog.Snapshot()
	if len(records) < l.miner.MinLen*l.miner.MinCount {
		return
	}

	patterns := l.miner.Mine(records)
	if len(patterns) == 0 {
		return
	}

	l.mu.Lock()
	defer l.mu.Unlock()

	for _, pat := range patterns {
		key := pat.Seq.Key()

		// Kiểm tra đã có proposal cho pattern này chưa
		alreadyExists := false
		for _, p := range l.proposals {
			if p.Name == l.seqToName(pat.Seq) ||
				strings.Join(pat.Seq, "→") == strings.Join(p.Sequence, "→") {
				alreadyExists = true
				break
			}
		}
		if alreadyExists {
			continue
		}

		conf := calcConfidence(pat.Count, len(pat.Seq), len(records))
		proposal := &SkillProposal{
			ID:        fmt.Sprintf("prop_%s_%d", l.agentID, time.Now().UnixNano()),
			Name:      l.seqToName(pat.Seq),
			Sequence:  pat.Seq,
			Count:     pat.Count,
			Confidence: conf,
			Status:    ProposalPending,
			CreatedAt: time.Now(),
		}

		l.proposals[proposal.ID] = proposal

		// Notify
		if l.onPropose != nil {
			go l.onPropose(proposal)
		}

		// Auto-confirm nếu confidence đủ cao
		if conf >= l.autoThreshold {
			_ = l.activate(proposal) // ignore error if skills missing
		}

		_ = key // suppress unused warning
	}
}

// activate build và đăng ký ComposedSkill (gọi với mu held)
func (l *SkillLearner) activate(p *SkillProposal) error {
	if err := l.registry.Propose(p); err != nil {
		// Một số skills có thể chưa có trong registry → pending
		return err
	}
	p.Status = ProposalActive
	p.ApprovedAt = time.Now()
	if l.onActivate != nil {
		go l.onActivate(p)
	}
	return nil
}

// seqToName tạo tên skill từ sequence
// ["actuator.light.living_room", "actuator.light.bedroom"] → "light_sequence"
func (l *SkillLearner) seqToName(seq []string) string {
	if len(seq) == 0 {
		return "composed_skill"
	}
	// Lấy phần chung của các skill names
	parts := make([]string, len(seq))
	for i, s := range seq {
		// Lấy prefix cuối (actuator.light.X → light_X)
		chunks := strings.Split(s, ".")
		if len(chunks) > 0 {
			parts[i] = chunks[len(chunks)-1]
		} else {
			parts[i] = s
		}
	}
	// Tìm common prefix
	if len(parts) == 1 {
		return parts[0] + "_composed"
	}
	// Nếu tất cả cùng domain → domain_sequence
	domains := make(map[string]int)
	for _, s := range seq {
		chunks := strings.Split(s, ".")
		if len(chunks) >= 2 {
			domains[chunks[0]]++
		}
	}
	for domain, count := range domains {
		if count == len(seq) {
			return domain + "_sequence_" + fmt.Sprintf("%d", len(seq))
		}
	}
	// Fallback: join first chars
	initials := make([]string, len(parts))
	for i, p := range parts {
		if len(p) > 3 {
			initials[i] = p[:3]
		} else {
			initials[i] = p
		}
	}
	return strings.Join(initials, "_") + "_composed"
}

// ══════════════════════════════════════════════════════════════
// PROPOSAL JSON — để AAM broadcast lên user
// ══════════════════════════════════════════════════════════════

// ProposalJSON trả về JSON representation của một proposal
func (p *SkillProposal) JSON() []byte {
	data := map[string]interface{}{
		"id":         p.ID,
		"name":       p.Name,
		"sequence":   p.Sequence,
		"count":      p.Count,
		"confidence": fmt.Sprintf("%.2f", p.Confidence),
		"status":     p.Status,
		"created_at": p.CreatedAt.Format(time.RFC3339),
	}
	b, _ := json.Marshal(data)
	return b
}
