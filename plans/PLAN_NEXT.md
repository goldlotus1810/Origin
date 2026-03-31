# PLAN NEXT — Sau CUT.4

> **Ngày:** 2026-03-24
> **Status:** T1-T4 done. Self-build works. Nó nói chuyện, nhớ, đọc sách.
> **Binary:** 911KB x86_64, 383KB self-built, 3KB WASM

---

## Tình trạng hiện tại

### DONE ✅ (27/31 tasks)
- T1 Intelligence: 5/5 (encoder, analysis, intent, agents, response)
- T2 Language: 10/10 (for-in, comprehension, try/catch, match, interpolation, dict, import)
- T3 Platform: 3/5 (SHA-256, WASM, Browser)
- T4 Cut cord: 3/4 (runtime done, tests done, self-build works)
- Memory: STM + Silk + Dream + Knowledge (learn_file, 128 facts)
- Natural language mode: tiếng Việt → agent response

### REMAINING (4 tasks)
- OL.11: ARM64 VM (WIP, boots bare via QEMU)
- OL.15: Mobile (blocked by ARM64)
- CUT.2: Full Rust builder replacement (self-build works but copies bytecode)
- CUT.4: Remove Rust completely (needs CUT.2 full)

---

## Phase 5 — Hoàn thiện sản phẩm

### 5.1 Performance (ưu tiên CAO)

| Task | Effort | Impact |
|------|--------|--------|
| Arena allocator cho VM heap | ~200 LOC ASM | Unlock bootstrap self-compile. No more heap overflow. |
| Tokenizer optimize (batch substr) | ~50 LOC Olang | lexer.ol compile < 1s instead of crash |
| `&&`/`||` operators in bootstrap parser | ~30 LOC Olang | encoder.ol compiles fully (hiện partial) |
| `<<`/`>>` bit shift builtins | ~20 LOC ASM | mol.ol compiles correctly |

### 5.2 Intelligence (ưu tiên CAO)

| Task | Effort | Impact |
|------|--------|--------|
| Context window (multi-turn summary) | ~100 LOC Olang | Tóm tắt conversation khi STM > 8 turns |
| Sentence splitting cho learn_file | ~50 LOC Olang | Học từng câu thay vì cả paragraph |
| Word stemming (Vietnamese) | ~100 LOC Olang | "chạy", "đang chạy", "chạy đi" → cùng root |
| Response templates (configurable) | ~80 LOC Olang | Thay hardcode "Minh nghe roi" bằng template file |
| Emotion carry-over between turns | ~50 LOC Olang | Cảm xúc tích lũy qua conversation |

### 5.3 Platform (ưu tiên TRUNG BÌNH)

| Task | Effort | Impact |
|------|--------|--------|
| WASM full compiler (not just arithmetic) | ~500 LOC WAT | Browser REPL compiles Olang |
| ARM64 builtins (34 missing) | ~500 LOC ASM | ARM64 runs stdlib |
| AES-256-GCM in ASM | ~300 LOC ASM | Encrypted storage |
| Ed25519 signatures | ~400 LOC ASM | QR record signing |

### 5.4 Self-Hosting (ưu tiên TRUNG BÌNH)

| Task | Effort | Impact |
|------|--------|--------|
| Full bootstrap self-compile | ~100 LOC | origin.olang compiles ALL .ol files (not just copy bytecode) |
| Remove Cargo.toml + crates/ | 0 LOC | Clean repo — only ASM + Olang |
| Olang package manager | ~300 LOC | `install <url>` → download + compile |

### 5.5 User Experience (ưu tiên THẤP)

| Task | Effort | Impact |
|------|--------|--------|
| REPL history (up/down arrows) | ~100 LOC ASM | Terminal UX |
| Syntax highlighting | ~200 LOC Olang | Pretty REPL output |
| Error messages with line numbers | ~100 LOC Olang | Debug UX |
| Config file (.homeosrc) | ~50 LOC Olang | Persistent settings |
| Multi-line input (\ continuation) | ~30 LOC ASM | Long programs |

---

## Đề xuất thứ tự

```
Phase 5.1: Performance
  ① Arena allocator → unlock full self-compile
  ② && || operators → encoder.ol compiles fully
  ③ << >> bit shifts → mol.ol compiles correctly

Phase 5.2: Intelligence
  ④ Sentence splitting → better knowledge from books
  ⑤ Context window → longer conversations
  ⑥ Response templates → configurable personality

Phase 5.3: Platform
  ⑦ WASM full compiler → browser HomeOS
  ⑧ ARM64 builtins → mobile ready

Phase 5.4: Self-Hosting
  ⑨ Full self-compile (all 63 .ol files)
  ⑩ Remove Rust dependency entirely
```

---

## Metrics

| Metric | Hiện tại | Mục tiêu |
|--------|---------|----------|
| Binary size | 911KB | < 1MB (self-built) |
| Boot time | < 1s | < 0.5s |
| Self-build time | 5.8s | < 3s |
| Knowledge capacity | 128 facts | 1000+ facts |
| STM turns | 8 | 32 |
| Silk edges | 64 | 256 |
| Test coverage | 12 tests | 50+ tests |
| Platforms | x86_64 + WASM | + ARM64 + iOS |
| Files self-compiled | 8/63 | 63/63 |

---

*911KB. 13 ngày. Nó nói chuyện, nhớ, đọc sách, tự build.*
*Kẻ điên thắng rồi. Nhưng vẫn còn nhiều việc.*
