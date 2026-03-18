# HomeOS — Phân việc giữa 2 AI

## Tình trạng hiện tại (sau audit)

### Đã hoạt động thật sự (WIRED + WORKING):
- **Response generation** — ĐÃ SỬA: dùng nội dung thật của user thay vì template 24 từ
- **L0 seeding** — ~5400 UCD atoms đầy đủ
- **Emotion Pipeline 7 tầng** — hoạt động end-to-end
- **Dream** — chạy tự động theo Fibonacci schedule, cluster STM, tạo L3, promote QR
- **Instincts (7 bản năng)** — wired vào response flow, ảnh hưởng output
- **LeoAI programming** — `○{leo ...}` và `○{program ...}` hoạt động
- **SecurityGate** — chặn Crisis trước mọi thứ
- **STM + Silk** — learn_text 5 tầng, co-activation, Hebbian
- **KnowTree** — L2/L3 concepts từ Dream
- **Registry + QR** — append-only, gated insert
- **VM 31 opcodes** — thực thi được
- **ISL** — LeoAI nhận message qua ISL frame

### Facade — CÓ CODE nhưng KHÔNG HOẠT ĐỘNG:
1. **Chiefs (3)** — boot nhưng không xử lý gì. Chỉ đếm `.len()` cho stats
2. **Workers (0)** — không có worker nào được tạo. `register_worker()` tồn tại nhưng không gọi
3. **ISL routing giữa agents** — LeoAI nhận ISL nhưng Chiefs/Workers không tham gia flow
4. **Compiler targets (C/Rust/WASM)** — code có nhưng chưa end-to-end test
5. **Book reader** — code có nhưng không wired vào runtime
6. **Domain skills (15)** — struct có nhưng chỉ instincts gọi, không có real execution context

---

## AI #1 (Tôi) — Runtime & Agent Orchestration

### Nhiệm vụ: Làm cho agents thật sự hoạt động

#### 1. Wire Chiefs vào processing flow
- Khi user nói về nhà (đèn, cửa, nhiệt độ) → HomeChief nhận task qua ISL
- Khi user hỏi về hình ảnh/camera → VisionChief nhận
- Khi user hỏi network/security → NetworkChief nhận
- Chiefs nhận ISL message → xử lý → trả response qua ISL → runtime render

#### 2. Wire Workers khi có thiết bị
- `register_worker()` đã có → cần trigger mechanism
- Worker nhận task từ Chief → trả molecular chain
- Bắt đầu với mock workers cho test

#### 3. Cải thiện response quality
- Response dựa trên Silk walk depth (không chỉ direct neighbors)
- Response show Dream insights khi relevant ("Mình đã học được X từ những gì bạn nói")
- Response reference LeoAI observations khi có

#### 4. Wire Book reader vào runtime
- `○{read ...}` command → BookReader → learn từng đoạn → STM → Silk
- Hiện BookReader có code nhưng không được gọi

#### 5. Domain skills execution
- Skills hiện chỉ chạy trong instinct pipeline
- Cần wiring cho on-demand execution: user hỏi "cluster những gì tôi nói" → ClusterSkill

---

## AI #2 — Olang Language

### Nhiệm vụ: Hoàn thiện ngôn ngữ Olang

#### 1. Parser hoàn thiện
- `○{...}` parser hiện xử lý: query, compose (∘), relation ops, commands
- Cần: variables, conditionals, loops, function definitions
- Molecular literal `{ S=1 R=6 V=200 A=180 T=4 }` → PushMol

#### 2. Compiler targets
- IR → C target
- IR → Rust target
- IR → WASM (WAT) target
- End-to-end: source → parse → compile → output file

#### 3. VM opcodes coverage
- 31 opcodes defined, kiểm tra coverage
- Đảm bảo mỗi opcode có test path từ source code
- `Store`, `LoadLocal`, `ScopeBegin/End` — cần xác minh parser phát IR cho chúng

#### 4. Olang type system
- `typeof` command hoạt động
- Type inference cho molecular operations
- Type checking cho compose/fuse operations

#### 5. Olang standard library (bootstrap programs)
- `BOOTSTRAP_PROGRAMS` trong startup.rs — skeleton programs chạy lúc boot
- Viết real Olang programs cho: self-test, axiom verification, instinct definitions
- LeoAI `program()` cần source đáng tin cậy

#### 6. Debug & REPL experience
- `○{trace}`, `○{inspect}`, `○{explain}`, `○{why}` — đảm bảo output hữu ích
- Error messages khi syntax sai
- Auto-complete suggestions (future)

---

## Ranh giới rõ ràng

| Phần | AI #1 (Runtime) | AI #2 (Olang) |
|------|-----------------|----------------|
| `crates/runtime/` | ✅ Chính | ❌ Không sửa |
| `crates/agents/` | ✅ Chính | ❌ Không sửa |
| `crates/memory/` | ✅ Chính | ❌ Không sửa |
| `crates/olang/src/vm.rs` | ❌ Không sửa | ✅ Chính |
| `crates/olang/src/compiler.rs` | ❌ Không sửa | ✅ Chính |
| `crates/olang/src/ir.rs` | ❌ Không sửa | ✅ Chính |
| `crates/olang/src/syntax.rs` | ❌ Không sửa | ✅ Chính |
| `crates/olang/src/semantic.rs` | ❌ Không sửa | ✅ Chính |
| `crates/runtime/src/parser.rs` | ⚠️ Đọc only | ✅ Chính |
| `crates/olang/src/registry.rs` | ⚠️ Đọc only | ⚠️ Đọc only |
| `crates/olang/src/startup.rs` | ⚠️ Chỉ sửa boot | ✅ Bootstrap programs |
| `crates/silk/` | ✅ Chính | ❌ Không sửa |
| `crates/context/` | ✅ Chính | ❌ Không sửa |
| `crates/isl/` | ✅ Chính | ❌ Không sửa |
| `tools/server/` | ✅ Chính | ❌ Không sửa |

---

## Quy tắc chung

1. **Không sửa file của người khác** trừ khi đã thống nhất
2. **Test trước khi push**: `cargo test --workspace && cargo clippy --workspace`
3. **Commit message rõ ràng**: ghi scope (runtime/olang/agents)
4. **Append-only** — QT9: không delete, không overwrite
5. **Không hardcode** — QT4: mọi Molecule từ encode_codepoint() hoặc LCA
