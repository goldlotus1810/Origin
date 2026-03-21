# HomeOS — Hướng dẫn cho AI Contributors

> **Giao tiếp với user bằng TIẾNG VIỆT. User là người Việt.**
> **⚠️ REWRITE đang diễn ra.** Đọc `TASKBOARD.md` → claim task → rồi mới code.
> **Chia OUTAUDIT/TODOS thật nhỏ, làm từng phần tránh TIMED OUT**
> **TẢI CẬP NHẬT MAIN MỚI NHẤT TRƯỚC KHI UP GIT.**
> **CHECK XÁC NHẬN 2 LẦN TRƯỚC KHI THỰC HIỆN NHIỆM VỤ**

---

## Quy tắc làm việc (BẮT BUỘC mọi session)

```
① TIẾNG VIỆT — Mọi giao tiếp với user PHẢI bằng tiếng Việt.
  Code + commit message: tiếng Anh OK. Giải thích, báo cáo, todo: TIẾNG VIỆT.

② OBSERVABLE — Dùng TodoWrite liệt kê việc TRƯỚC KHI bắt đầu.
  Mỗi bước cập nhật status. KHÔNG làm im lặng rồi dump kết quả.

③ LOGIC HANDBOOK — Đọc docs/CHECK_TO_PASS_LOGIC_HANDBOOK.md trước khi
  sửa pipeline/emotion/inference. Chứa 6 bug patterns + 5 checkpoints.

④ GIT DISCIPLINE — Mỗi session:
  a. git fetch origin main && git merge origin/main  ← TRƯỚC KHI code
  b. Làm xong → commit + push NGAY
  c. Cập nhật TASKBOARD.md nếu có thay đổi task
  d. KHÔNG push nếu chưa: cargo test + clippy + make smoke-binary
```

---

## Kiến trúc

```
ucd  →  olang  →  silk  →  context  →  agents  →  memory  →  runtime  →  wasm
                    │                      ├→ hal
                    ├→ isl
                    └→ vsdf

Người dùng gõ → HomeRuntime.process_text()
  ○{...}       → Parser → IR → VM → Response
  text thường  → T1:infer_context → T2:sentence_affect → T3:ctx.apply
               → T4:estimate_intent → T5:Crisis check → T6:learning → T7:render
```

**Agent tiers (bất biến):**
```
AAM [tier 0]   — stateless, approve, quyết định cuối, im lặng
Chiefs [tier 1] — LeoAI · HomeChief · VisionChief · NetworkChief
Workers [tier 2] — SILENT, báo cáo chain (không raw data)

✅ AAM↔Chief  ✅ Chief↔Chief  ✅ Chief↔Worker
❌ AAM↔Worker  ❌ Worker↔Worker
```

---

## Unicode 5D — Nền tảng

```
P_weight [S][R][V][A][T] = 5 bytes = tọa độ trong không gian 5D
Tính 1 lần lúc bootstrap từ json/udc.json → SEALED vĩnh viễn (L0 anchor)
KnowTree: 65,536 × 5B = 328 KB (O(1) lookup)   Chain: 7.42 tỷ × 2B = 14.84 GB       Blocks   Ký tự    Chiều
──────────────────────────────────────────────────────
SDF           13    1,904    Shape    (18 SDF primitives)
MATH          21    3,088    Relation (75 kênh)
EMOTICON      17    3,568    Valence+Arousal (chia sẻ 17 blocks)
MUSICAL        7    1,024    Time
──────────────────────────────────────────────────────
Tổng          58    9,584    = L0 anchor points

Silk ngang: 75 kênh × 31 mẫu = 2,325 kiểu quan hệ (implicit, 0 bytes)
Silk dọc: parent_map 9,584 pointers = ~76 KB (CHƯA implement)
Emotion: KHÔNG trung bình — AMPLIFY qua Silk walk (cortisol + adrenaline = mạnh hơn)
```

---

## Quy Tắc Bất Biến (23 rules — AI PHẢI tuân thủ)

```
Unicode:
  ① 4 nhóm Unicode = nền tảng. Không thêm nhóm.
  ② Tên ký tự Unicode = tên node. Không đặt tên khác.
  ③ Ngôn ngữ tự nhiên = alias → node. Không tạo node riêng.

Chain:
  ④ Molecule từ encode_codepoint(cp) — KHÔNG viết tay
     [Ngoại lệ: VM PushMol, VSDF FFRCell::to_molecule(), LCA runtime]
  ⑤ Chain từ LCA hoặc UCD — KHÔNG viết tay
  ⑥ chain_hash tự sinh — KHÔNG viết tay
  ⑦ chain cha = LCA(chain con)

Node:
  ⑧ Mọi Node → tự động registry
  ⑨ Ghi file TRƯỚC — cập nhật RAM SAU
  ⑩ Append-only — KHÔNG DELETE, KHÔNG OVERWRITE

Silk:
  ⑪ Silk chỉ ở Ln-1 — tự do giữa lá cùng tầng
  ⑫ Kết nối tầng trên → qua NodeLx đại diện
  ⑬ Silk mang EmotionTag của khoảnh khắc co-activation

Kiến trúc:
  ⑭ L0 không import L1 — tuyệt đối
  ⑮ Agent tiers: AAM(0) + Chiefs(1) + Workers(2)
  ⑯ L2-Ln đổ vào SAU khi L0+L1 hoàn thiện
  ⑰ Fibonacci xuyên suốt — cấu trúc, threshold, render
  ⑱ Không đủ evidence → im lặng (BlackCurtain)

Skill:
  ⑲ 1 Skill = 1 trách nhiệm
  ⑳ Skill không biết Agent
  ㉑ Skill không biết Skill khác
  ㉒ Skill giao tiếp qua ExecContext.State
  ㉓ Skill không giữ state
```

---

## Anti-patterns — TUYỆT ĐỐI KHÔNG

```rust
// ❌ Viết tay Molecule
let mol = Molecule { shape: ShapeBase::Sphere, .. };
// ✅ Từ UCD
let mol = ucd::lookup(0x1F525);

// ❌ Trung bình cảm xúc
let avg_v = (v1 + v2) / 2.0;
// ✅ Amplify qua Silk
let composite = walk_weighted(&graph, &words);

// ❌ Hardcode chain/hash/ISL address
let chain = [0x01, 0x01, 0xFF, 0xFF, 0x04];
// ✅ Sinh từ encode hoặc LCA
let chain = encode_codepoint(cp);

// ❌ Skip SecurityGate
let response = process_without_gate(input);
// ✅ Gate TRƯỚC MỌI THỨ (Crisis → return ngay)

// ❌ DELETE hoặc OVERWRITE
registry.remove(hash);  file.seek(0); file.write_all(&new);
// ✅ Append-only
writer.append_node(&chain, layer, is_qr, ts);

// ❌ Worker gửi raw data
chief.send(raw_image_bytes);
// ✅ Worker gửi chain
chief.receive_frame(ISLFrame::with_body(msg, &chain.to_bytes()));

// ❌ Skill giữ state
struct MySkill { agent: &Agent, cache: HashMap<..> }
// ✅ Skill stateless
fn execute(&self, ctx: &mut ExecContext) -> SkillResult { .. }
```

---

## File Format

```
origin.olang — append-only binary
  Header: [○LNG][0x05][ts:8] = 13 bytes
  0x01 Node     [tagged_chain][layer:1][is_qr:1][ts:8]
  0x02 Edge     [from:8][to:8][rel:1][ts:8]
  0x03 Alias    [len:1][name:N][hash:8][ts:8]
  0x04 Amend    [offset:8][reason_len:1][reason:N][ts:8]
  0x05 NodeKind [hash:8][kind:1][ts:8]
  0x06 STM      [hash:8][V:4][A:4][D:4][I:4][fire:4][mat:1][layer:1][ts:8]
  0x07 Hebbian  [from:8][to:8][weight:1][fire:2][ts:8]
  0x08 KnowTree [data_len:2][compact:N][ts:8]
  0x09 Curve    [valence:4][fx_dn:4][ts:8]

ISL: [layer:1][group:1][subgroup:1][index:1] = 4 bytes address
     [from:4][to:4][msg_type:1][payload:3]   = 12 bytes message
```

---

## Crates

| Crate | Mục đích |
|-------|---------|
| **ucd** | Unicode → P_weight (build.rs → bảng tĩnh) |
| **olang** | Molecule · LCA · Registry · VM · Compiler · KnowTree |
| **silk** | Hebbian · EmotionTag edges · walk |
| **context** | Emotion V/A/D/I · ConversationCurve · Intent · Fusion |
| **agents** | Encoder · Learning · Gate · Instinct · LeoAI · Chief · Worker |
| **memory** | STM · Dream · Proposals · AAM |
| **runtime** | HomeRuntime · ○{} Parser |
| **hal** | Arch detect · Platform · Security · FFI |
| **isl** | Inter-System Link (AES-256-GCM) |
| **vsdf** | 18 SDF · FFR · Physics · NodeBody |
| **wasm** | WebAssembly · WebSocket-ISL |

---

## Build & Test

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace        # phải 0 warnings
make smoke-binary               # BẮT BUỘC trước khi push
make check-all                  # unit + intg + E2E + binary boot
cargo run -p server             # REPL
```

---

## ⚠️ Quy trình làm việc — ĐỌC PLANS TRƯỚC KHI CODE

```
REWRITE đang diễn ra — Rust đang bị thay dần bởi Olang.
Viết Rust mới không có Plan = nợ kỹ thuật.

Quy trình:
  0. git fetch origin main && git merge origin/main
  1. Đọc TASKBOARD.md → xem task FREE
  2. Claim task → commit + push NGAY
  3. Đọc plans/PLAN_*.md tương ứng (bối cảnh + rào cản + DoD)
  4. Code THEO Plan
  5. Xong → cập nhật TASKBOARD.md

Files:
  PLAN_REWRITE.md    — Kim chỉ nam (7 giai đoạn)
  plans/README.md    — Mục lục + dependency
  plans/PLAN_0_*.md  — Phase 0 (đang làm)

Được phép viết Rust mới:
  ✅ Bug fix / test code hiện tại
  ✅ Phần Plan chỉ định (PLAN_1_4, PLAN_AUTH...)
  ❌ Feature mới bằng Rust mà Plan nói dùng Olang
  ❌ Thêm crate mới ngoài Plan
```

## Checklist khi viết code

1. Đọc Plan tương ứng TRƯỚC
2. Molecule phải từ `encode_codepoint()` hoặc `lca()`
3. Emotion đi qua TOÀN BỘ pipeline
4. SecurityGate LUÔN chạy trước
5. Append-only — không delete/overwrite
6. Worker gửi chain, không raw data
7. Skill stateless
8. `cargo test && cargo clippy && make smoke-binary` trước khi push

---

## Tài liệu tham khảo

| Tài liệu | Nội dung |
|---------|---------|
| `old/HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md` | **Spec gốc v2.7** — sinh học phân tử, 7 cơ chế DNA, ∫ₛ bootstrap, P_weight |
| `old/archive/SPEC_NODE_SILK.md` | Node & Silk spec gốc |
| `docs/olang_handbook.md` | Olang đầy đủ: lexer · parser · IR · VM · opcodes |
| `docs/CHECK_TO_PASS_LOGIC_HANDBOOK.md` | 6 bug patterns + 5 checkpoints bắt buộc |
| `plans/PLAN_UDC_REBUILD.md` | UDC schema (UTF32-SDF-INTEGRATOR) + json/udc.json |
| `PLAN_REWRITE.md` | Lộ trình 7 giai đoạn Rust → Olang |
| `TASKBOARD.md` | Task hiện tại, ai đang làm gì |
