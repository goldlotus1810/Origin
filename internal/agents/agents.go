// internal/agents/agents.go
// Agent + Skill system
// QT4: "Agent = tập hợp Skill của nó"
// Silent by default: CPU≈0 khi không có message

package agents

import (
	"context"
	"log"
	"time"

	"github.com/goldlotus1810/HomeOS/internal/isl"
	"github.com/goldlotus1810/HomeOS/internal/nca"
)

// ─────────────────────────────────────────────────────────────────
// SKILL INTERFACE
// 5 quy tắc thiết kế Skill (bất biến):
//   ① 1 Skill = 1 trách nhiệm
//   ② Skill không biết Agent là gì
//   ③ Skill không biết Skill khác tồn tại
//   ④ Skill giao tiếp qua ExecContext.State
//   ⑤ Skill không giữ state — state nằm trong Agent
// ─────────────────────────────────────────────────────────────────

// ExecContext là context thực thi được truyền vào Skill
type ExecContext struct {
	State  map[string]interface{} // shared state từ Agent
	Inbox  <-chan *isl.ISLMessage
	Outbox chan<- *isl.ISLMessage
}

// SkillResult là kết quả trả về từ Skill
type SkillResult struct {
	Data            map[string]interface{} // merge vào Agent.state
	Message         *isl.ISLMessage        // gửi ra outbox (nil = không gửi)
	ProposeNewSkill string                 // tên skill mới để LeoAI xem xét
	Err             error
}

// Skill là interface mà mọi skill phải implement
type Skill interface {
	// CanHandle kiểm tra nhanh trước Execute — không tốn CPU
	CanHandle(msg *isl.ISLMessage) bool
	// Execute thực hiện skill — chỉ gọi khi CanHandle = true
	Execute(ctx *ExecContext, msg *isl.ISLMessage) SkillResult
	// Name trả về tên skill để debug/logging
	Name() string
}

// ─────────────────────────────────────────────────────────────────
// AGENT
// ─────────────────────────────────────────────────────────────────

// Agent là đơn vị thực thi trong HomeOS
// Silent by default: goroutine ngủ khi không có message
type Agent struct {
	id       string
	codec    *isl.ISLCodec
	inbox    chan *isl.ISLMessage
	outbox   chan *isl.ISLMessage
	skills   []Skill
	state    map[string]interface{}
	gate     *nca.SecurityGate
	registry *SkillRegistry
	learner  *SkillLearner
}

// New tạo Agent mới
func New(id string, codec *isl.ISLCodec,
	inbox, outbox chan *isl.ISLMessage) *Agent {
	reg := NewSkillRegistry()
	return &Agent{
		id:       id,
		codec:    codec,
		inbox:    inbox,
		outbox:   outbox,
		skills:   make([]Skill, 0),
		state:    make(map[string]interface{}),
		gate:     &nca.SecurityGate{},
		registry: reg,
		learner:  NewSkillLearner(id, reg),
	}
}

// Learner trả về SkillLearner để set callbacks từ ngoài
func (a *Agent) Learner() *SkillLearner { return a.learner }

// AddSkill thêm skill vào Agent — fluent API
// Đồng thời đăng ký vào SkillRegistry để SkillLearner có thể compose
func (a *Agent) AddSkill(s Skill) *Agent {
	a.skills = append(a.skills, s)
	a.registry.Register(s)
	return a
}

// RunAsync chạy Agent trong goroutine riêng — fluent API
func (a *Agent) RunAsync(ctx context.Context) *Agent {
	go a.Run(ctx)
	return a
}

// Run là vòng lặp chính — silent by default
// Chỉ wake up khi có message trong inbox
func (a *Agent) Run(ctx context.Context) {
	log.Printf("Agent[%s]: online · %d skills · silent", a.id, len(a.skills))
	// Start SkillLearner background
	go a.learner.Run(ctx)
	for {
		select {
		case <-ctx.Done():
			log.Printf("Agent[%s]: shutdown", a.id)
			return
		case msg := <-a.inbox:
			a.handle(msg)
		}
	}
}

// handle xử lý một message
func (a *Agent) handle(msg *isl.ISLMessage) {
	// 🛡 SecurityGate TRƯỚC TIÊN
	if err := a.gate.Check(msg); err != nil {
		log.Printf("Agent[%s]: 🛡 BLOCKED: %v", a.id, err)
		return
	}

	// Duyệt skills theo thứ tự
	execCtx := &ExecContext{
		State:  a.state,
		Outbox: a.outbox,
	}

	handled := false
	for _, skill := range a.skills {
		if skill.CanHandle(msg) {
			start := time.Now()
			result := skill.Execute(execCtx, msg)
			latency := time.Since(start)
			// Merge state
			for k, v := range result.Data {
				a.state[k] = v
			}
			// Gửi message nếu có
			if result.Message != nil && a.outbox != nil {
				select {
				case a.outbox <- result.Message:
				default:
					log.Printf("Agent[%s]: outbox full, dropped msg", a.id)
				}
			}
			success := result.Err == nil
			if !success {
				log.Printf("Agent[%s]: skill[%s] error: %v", a.id, skill.Name(), result.Err)
			}
			// Ghi vào ExecLog để SkillLearner mine patterns
			a.learner.RecordExec(a.id, skill.Name(), msg.MsgType, latency, success)
			handled = true
			break // chỉ một skill xử lý mỗi message
		}
	}

	if !handled {
		// Silent — không xử lý được thì im lặng
	}
}

// ─────────────────────────────────────────────────────────────────
// BUILT-IN SKILLS
// ─────────────────────────────────────────────────────────────────

// BroadcastSkill phát message ra outbox khi nhận MsgActivate
type BroadcastSkill struct{}

func (s *BroadcastSkill) Name() string { return "broadcast" }
func (s *BroadcastSkill) CanHandle(msg *isl.ISLMessage) bool {
	return msg.MsgType == isl.MsgActivate && msg.TargetID == 0 // broadcast to all
}
func (s *BroadcastSkill) Execute(ctx *ExecContext, msg *isl.ISLMessage) SkillResult {
	return SkillResult{Message: msg}
}

// HeartbeatSkill trả lời MsgHeartbeat
type HeartbeatSkill struct{}

func (s *HeartbeatSkill) Name() string { return "heartbeat" }
func (s *HeartbeatSkill) CanHandle(msg *isl.ISLMessage) bool {
	return msg.MsgType == isl.MsgHeartbeat
}
func (s *HeartbeatSkill) Execute(ctx *ExecContext, msg *isl.ISLMessage) SkillResult {
	resp := &isl.ISLMessage{
		MsgType:  isl.MsgResponse,
		TargetID: msg.SenderID,
		Payload:  []byte("pong"),
	}
	return SkillResult{Message: resp}
}

// ActuatorLightSkill điều khiển đèn
type ActuatorLightSkill struct {
	Location string // "living_room", "bedroom"...
	State    bool   // true=on, false=off
}

func NewActuatorLight(location string) *ActuatorLightSkill {
	return &ActuatorLightSkill{Location: location}
}

func (s *ActuatorLightSkill) Name() string { return "actuator.light." + s.Location }
func (s *ActuatorLightSkill) CanHandle(msg *isl.ISLMessage) bool {
	// Xử lý MsgActivate/MsgDeactivate có PrimaryAddr.Group = 'A' (Actuator)
	return (msg.MsgType == isl.MsgActivate || msg.MsgType == isl.MsgDeactivate) &&
		msg.PrimaryAddr.Group == 'A'
}
func (s *ActuatorLightSkill) Execute(ctx *ExecContext, msg *isl.ISLMessage) SkillResult {
	on := msg.MsgType == isl.MsgActivate
	s.State = on
	state := "ON"
	if !on {
		state = "OFF"
	}
	log.Printf("💡 Light[%s]: %s", s.Location, state)
	// TODO: gpio.Write(pin, on)
	return SkillResult{
		Data: map[string]interface{}{"light_" + s.Location: on},
		Message: &isl.ISLMessage{
			MsgType: isl.MsgResponse,
			Payload: []byte("light." + s.Location + "=" + state),
		},
	}
}
