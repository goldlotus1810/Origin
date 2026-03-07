// cmd/homeos/main.go — HomeOS bootstrap
package main

import (
	"context"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/goldlotus1810/HomeOS/internal/agents"
	"github.com/goldlotus1810/HomeOS/internal/isl"
	"github.com/goldlotus1810/HomeOS/internal/memory"
	"github.com/goldlotus1810/HomeOS/internal/nca"
	"github.com/goldlotus1810/HomeOS/internal/silk"
	"github.com/goldlotus1810/HomeOS/internal/ws"
)

func main() {
	log.SetFlags(log.Ltime | log.Lshortfile)
	log.Println("○ HomeOS starting...")

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// 🛡 SecurityGate TRƯỚC TIÊN
	soma := nca.NewSoma()
	_ = soma
	log.Println("🛡 SecurityGate: online")

	// SilkGraph + Unicode Tree + UTF32 Full Knowledge (367 blocks)
	graph  := silk.NewSilkGraph()
	ledger := silk.NewLedger("", nil)
	silk.SeedUnicodeTree(graph)
	ne := silk.ApplyNameEdges(graph)
	ie := silk.ApplyIPAEdges(graph)
	un, ue := silk.SeedUTF32(graph)
	log.Printf("○ SilkGraph: %d nodes, %d edges (name:%d ipa:%d utf32:+%d/+%d)",
		graph.NodeCount(), graph.EdgeCount(), ne, ie, un, ue)

	// Memory ΔΨ + QR
	dn    := memory.NewShortTerm(512)
	qr    := memory.NewLongTerm(graph, ledger)
	dream := memory.NewDream(dn, qr)
	go dream.Run(ctx)

	// NxT Cycle
	go nca.RunCycle(ctx, nca.DefaultConfig, dn, qr)
	log.Println("○ NxT: running")

	// Agents
	codec := isl.NewPlainCodec()
	bus   := make(chan *isl.ISLMessage, 128)

	aam := agents.New("aam", codec, bus, bus).
		AddSkill(&agents.BroadcastSkill{}).
		AddSkill(&agents.HeartbeatSkill{})

	// SkillLearner callbacks — QT7: ĐN → SkillProposal → QR
	aam.Learner().OnPropose(func(p *agents.SkillProposal) {
		log.Printf("○ SkillLearner: new proposal [%s] seq=%v conf=%.2f",
			p.Name, p.Sequence, p.Confidence)
		// Auto-confirm nếu confidence >= 0.95 (đã xử lý trong learner)
		// Manual confirm: aam.Learner().Confirm(p.ID)
	})
	aam.Learner().OnActivate(func(p *agents.SkillProposal) {
		log.Printf("○ SkillLearner: activated ComposedSkill [%s] ✅ (count=%d)",
			p.Name, p.Count)
	})
	aam.RunAsync(ctx)

	agents.New("light_living", codec, bus, bus).
		AddSkill(agents.NewActuatorLight("living_room")).
		RunAsync(ctx)

	agents.New("light_bed", codec, bus, bus).
		AddSkill(agents.NewActuatorLight("bedroom")).
		RunAsync(ctx)

	log.Println("○ Agents: online")

	// Milestone 1 test sau 2s
	go func() {
		time.Sleep(2 * time.Second)
		bus <- &isl.ISLMessage{
			Version: 1, MsgType: isl.MsgDeactivate,
			PrimaryAddr: isl.Address{Layer: 'H', Group: 'A', Type: 'a', ID: 1},
			Payload: []byte("living_room"),
		}
		log.Println("✅ M1: 'tắt đèn phòng khách' →")
	}()

	// World + HTTP/SSE server
	qrCount := graph.NodeCount()
	dnCount := 0
	world   := ws.NewWorld(888, graph)
	server  := ws.NewServer(world, &qrCount, &dnCount, envOr("HOMEOS_STATIC", "web/static"))
	go func() {
		if err := server.Serve(envOr("HOMEOS_ADDR", ":8080")); err != nil {
			log.Fatalf("server: %v", err)
		}
	}()
	log.Println("○ http://localhost:8080/  (api/tree  api/edges  ws/sse)")

	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
	<-sig
	log.Println("○ shutdown")
	cancel()
}

func envOr(k, def string) string {
	if v := os.Getenv(k); v != "" { return v }
	return def
}
