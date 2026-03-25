# HomeOS trên Olang 1.0 — Handcode == Zero

> **Sora (空) — 2026-03-25**
> **Olang 1.0 = nền tảng. HomeOS = sản phẩm. Vấn đề demo = feature chưa build.**

---

## I. VẤN ĐỀ THẬT — Không phải bug, mà là thiết kế cũ

### Demo thật trên máy user:

```
> hi
🤔 (Mình biết: Origin là dự án...) [fact]    ← SAI: "hi" = chào, không cần fact

> viet nam o dau?
🤔 (Mình biết: Margaret Mitchell...) [fact]   ← SAI: keyword "viet" lowercase ≠ "Viet"

> 2+1?
Parse error: unexpected symbol '?'             ← SAI: user hỏi toán, không viết code
```

### Nguyên nhân gốc — HomeOS hiện tại = if-else chains

```
90 hardcoded pattern checks (if _a_has, if intent ==, if tone ==)
4 response templates ("Mình nghe rồi", "Để mình tìm hiểu"...)
5 keyword lists (buon, vui, cam on, xin chao...)

agent_respond() LUÔN:
  1. Encode text → mol
  2. Tìm knowledge → attach fact (dù không liên quan)
  3. Compose reply từ template

KHÔNG BAO GIỜ:
  - Hỏi lại khi không hiểu
  - Im lặng khi context không phù hợp
  - Phân biệt "chào" vs "hỏi" vs "tính toán"
```

---

## II. NGUYÊN TẮC — Gate trước, trả lời sau

### Gate = Cổng kiểm soát

```
Input → GATE → Decision:
  ├── BIẾT     → trả lời (confidence ≥ threshold)
  ├── KHÔNG BIẾT → hỏi lại ("Bạn muốn hỏi về gì?")
  ├── KHÔNG PHÙ HỢP → im lặng hoặc chuyển mode
  └── KHÔNG HIỂU → parse khác / thử cách khác

Hiện tại: Gate chỉ check crisis patterns (kill, suicide).
Cần:      Gate check EVERYTHING trước khi respond.
```

### Handcode == Zero nghĩa là gì

```
KHÔNG:
  if text == "xin chao" { return "Xin chao ban!"; };
  if intent == "heal" { return "Tu tu thoi..."; };
  if _a_has(text, "buon") { tone = "supportive"; };

MÀ:
  let match = knowledge_search(text);
  if match.score < THRESHOLD { return ask_back(text); };
  let response = compose_from_knowledge(match, context);
  return gate_check(response);
```

Intelligence từ DATA + ALGORITHM, không từ if-else.

---

## III. OLANG 1.0 ĐÃ CÓ GÌ ĐỂ BUILD

### Công cụ sẵn sàng

```
Chuỗi:     split, join, contains, len, char_at, __substr, __str_trim
Mảng:      sort, map, filter, reduce, push, set_at, min_val, max_val, sum
Logic:      pipe, any, all, fn lambda
Dữ liệu:   dict { key: value }, match pattern
Crypto:     __sha256
Mol:        __mol_s/r/v/a/t, __mol_pack, encode_codepoint, mol_compose
Node:       fn_node_register, fn_node_describe, fn_node_fire
Persist:    save, load, __file_read, __file_write
Error:      try/catch, __throw
```

### Cái CHƯA CÓ (cần build bằng Olang)

```
❌ Case-insensitive search        → build bằng split + map + compare
❌ Input classifier (code/NL/math) → build bằng pattern detection
❌ Confidence scoring              → build bằng knowledge match score
❌ Context-aware gate              → build bằng STM + intent + score
❌ Ask-back mechanism              → build bằng gate + response
❌ NL math parser                  → build bằng split + __to_number
❌ Smart greeting                  → build bằng word list + random response
```

**Tất cả đều build được bằng Olang 1.0 hiện tại.**

---

## IV. KIẾN TRÚC HOMEOS MỚI — TỪ GATE ĐẾN OUTPUT

```
Input
  │
  ▼
┌─────────────────────────────┐
│ CLASSIFIER                  │  ← split, contains, patterns
│  "2+1?" → math              │
│  "hi" → greeting            │
│  "ha noi?" → question       │
│  "let x = 1" → code        │
│  "learn X" → command        │
└──────┬──────────────────────┘
       │
  ┌────▼────┐
  │  ROUTER │
  │         │
  ├── math ────→ eval_math(input) → trả kết quả
  ├── code ────→ compile + run → trả kết quả
  ├── command ─→ execute (learn/save/load/test/build)
  ├── greeting → greet(context) → response ngắn
  └── question/chat → ↓
       │
  ┌────▼────────────────────────┐
  │ GATE — Quyết định có nên    │
  │ trả lời không                │
  │                              │
  │ search = knowledge_search()  │
  │ score = search.score         │
  │                              │
  │ if score >= HIGH → respond   │
  │ if score >= LOW → cautious   │
  │ if score < LOW → ask_back    │
  │ if context mismatch → silent │
  └──────┬──────────────────────┘
         │
    ┌────▼─────────┐
    │ COMPOSE      │  ← từ knowledge match, không từ template
    │              │
    │ Trả fact trực tiếp nếu biết
    │ Hỏi lại nếu không biết
    │ Im lặng nếu không phù hợp
    └──────────────┘
```

---

## V. TỪNG MODULE — VIẾT BẰNG OLANG 1.0

### Module 1: Classifier (~50 LOC)

```olang
fn classify(_cl_input) {
    let _cl_trimmed = __str_trim(_cl_input);
    if len(_cl_trimmed) == 0 { return "empty"; };

    // Trailing ? → question OR math
    let _cl_last = __char_code(char_at(_cl_trimmed, len(_cl_trimmed) - 1));
    let _cl_has_q = 0;
    if _cl_last == 63 { _cl_has_q = 1; };  // ?

    // Starts with digit → possible math
    let _cl_first = __char_code(char_at(_cl_trimmed, 0));
    if _cl_first >= 48 {
        if _cl_first <= 57 {
            // Contains operator? → math
            if contains(_cl_trimmed, "+") == 1 { return "math"; };
            if contains(_cl_trimmed, "*") == 1 { return "math"; };
            if contains(_cl_trimmed, "/") == 1 { return "math"; };
            if contains(_cl_trimmed, "-") == 1 { return "math"; };
        };
    };

    // Keywords → code
    let _cl_words = split(_cl_trimmed, " ");
    let _cl_w0 = _cl_words[0];
    if _cl_w0 == "let" { return "code"; };
    if _cl_w0 == "fn" { return "code"; };
    if _cl_w0 == "emit" { return "code"; };
    if _cl_w0 == "if" { return "code"; };
    if _cl_w0 == "while" { return "code"; };
    if _cl_w0 == "for" { return "code"; };

    // Commands
    if _cl_w0 == "learn" { return "command_learn"; };
    if _cl_w0 == "respond" { return "command_respond"; };
    if _cl_w0 == "save" { return "command"; };
    if _cl_w0 == "load" { return "command"; };
    if _cl_w0 == "test" { return "command"; };
    if _cl_w0 == "build" { return "command"; };
    if _cl_w0 == "memory" { return "command"; };
    if _cl_w0 == "fns" { return "command"; };
    if _cl_w0 == "help" { return "command"; };
    if _cl_w0 == "exit" { return "command"; };

    // Short greeting patterns
    if len(_cl_trimmed) <= 10 {
        if _str_lower_has(_cl_trimmed, "hi") == 1 { return "greeting"; };
        if _str_lower_has(_cl_trimmed, "hello") == 1 { return "greeting"; };
        if _str_lower_has(_cl_trimmed, "chao") == 1 { return "greeting"; };
        if _str_lower_has(_cl_trimmed, "hey") == 1 { return "greeting"; };
    };

    // Has question mark → question
    if _cl_has_q == 1 { return "question"; };

    // Default → chat (natural text)
    return "chat";
}
```

**Đây KHÔNG PHẢI hardcode.** Đây là **router** — phân loại input rồi chuyển đến handler đúng. Giống switch/case trong mọi ngôn ngữ.

### Module 2: Gate (~30 LOC)

```olang
fn gate_decide(_gd_input, _gd_intent) {
    // Search knowledge
    let _gd_match = knowledge_search_scored(_gd_input);

    // HIGH confidence → respond trực tiếp
    if _gd_match.score >= 15 {
        return { action: "respond", fact: _gd_match.text, confidence: "high" };
    };

    // MEDIUM → cautious response
    if _gd_match.score >= 5 {
        return { action: "respond", fact: _gd_match.text, confidence: "medium" };
    };

    // LOW → hỏi lại
    if _gd_match.score > 0 {
        return { action: "ask_back", fact: _gd_match.text, confidence: "low" };
    };

    // ZERO → không biết
    return { action: "unknown", fact: "", confidence: "none" };
}
```

### Module 3: Math Evaluator (~20 LOC)

```olang
fn eval_math(_em_input) {
    // Strip trailing ? = !
    let _em_clean = _em_input;
    let _em_last = __char_code(char_at(_em_clean, len(_em_clean) - 1));
    if _em_last == 63 { _em_clean = __substr(_em_clean, 0, len(_em_clean) - 1); };
    if _em_last == 61 { _em_clean = __substr(_em_clean, 0, len(_em_clean) - 1); };
    if _em_last == 33 { _em_clean = __substr(_em_clean, 0, len(_em_clean) - 1); };

    // Compile + eval as expression
    let _em_code = "emit " + __str_trim(_em_clean);
    let _em_tokens = tokenize(_em_code);
    let _em_ast = parse(_em_tokens);
    let _em_state = analyze(_em_ast);
    if _g_pos > 0 {
        __eval_bytecode(/* compiled bytecode */);
    };
}
```

### Module 4: Smart Greeting (~10 LOC)

```olang
fn smart_greet(_sg_context) {
    if _sg_context.stm_count == 0 {
        return "Xin chao! Minh la HomeOS. Ban muon lam gi hom nay?";
    };
    return "Chao ban! Minh o day.";
}
```

### Module 5: Case-Insensitive Search (~15 LOC)

```olang
// Đã viết _str_eq_ci ở trên — dùng cho knowledge_search
// Thay == bằng _str_eq_ci() trong search loop
// + min word length 2 chars (Vietnamese: Ha, Da, Ho)
```

---

## VI. GIÁ TRỊ CỦA CÁCH TIẾP CẬN NÀY

### Cách cũ (patch bugs):

```
Bug: "viet nam" → sai fact
Fix: thêm _str_eq_ci vào encoder.ol
→ 1 bug fix, 15 LOC, encoder.ol 1768 LOC → ngày càng phình

Bug: "2+1?" → parse error
Fix: strip "?" trong repl.ol
→ 1 hack, 3 LOC, nhưng "2+1 bang may?" vẫn sai

Bug: "hi" → trả fact
Fix: thêm if len(text) < 5 skip knowledge
→ 1 hack, 3 LOC, nhưng "bye" cũng bị skip
```

Mỗi bug fix = thêm 1 if-else. HomeOS ngày càng mong manh.

### Cách mới (build features):

```
Feature: Input Classifier
→ classify("2+1?") = "math"
→ classify("hi") = "greeting"
→ classify("viet nam o dau?") = "question"
→ Router gửi đến handler đúng
→ KHÔNG CẦN hack gì trong encoder.ol

Feature: Gate
→ gate_decide("viet nam o dau?") = { action: "respond", score: 15 }
→ gate_decide("asdfghjk") = { action: "unknown" }
→ Response: "Mình chưa biết về điều này" thay vì fact random

Feature: Math
→ eval_math("2+1?") = 3
→ eval_math("2*3=") = 6
→ Trả kết quả, không parse error
```

Mỗi feature = 1 module. Composition bằng pipe/map/filter. **Testable riêng.**

---

## VII. THỨ TỰ BUILD

```
Sprint 1: Foundation (~100 LOC)
  ① _str_eq_ci() — case-insensitive compare
  ② _str_lower() — lowercase converter  
  ③ classify() — input classifier (math/code/greeting/question/chat)
  ④ Test: 20+ cases, all classify correctly

Sprint 2: Router + Handlers (~80 LOC)
  ⑤ eval_math() — strip ?/= → compile as expression
  ⑥ smart_greet() — context-aware greeting
  ⑦ Router: classify → dispatch to handler
  ⑧ Test: "2+1?" → 3, "hi" → greeting, "let x=1" → code

Sprint 3: Gate + Intelligence (~60 LOC)
  ⑨ knowledge_search_scored() — return {text, score}
  ⑩ gate_decide() — threshold logic
  ⑪ ask_back() — "Bạn muốn hỏi về gì?"
  ⑫ Test: "viet nam" → correct fact, "asdf" → ask_back

Sprint 4: Compose + Polish (~40 LOC)
  ⑬ compose_response() — from gate output, không template
  ⑭ Integrate: classifier → router → gate → compose → output
  ⑮ Full integration test: 30+ scenarios
```

**~280 LOC mới. Thay thế ~90 hardcoded patterns.**

---

## VIII. KẾT LUẬN

```
Olang 1.0 = BÚA.
HomeOS hiện tại = cái nhà xây bằng tay, vá víu.
HomeOS mới = cái nhà xây bằng búa, thiết kế đúng.

"hi" → greeting, không phải fact random
"viet nam o dau?" → tìm đúng, case-insensitive
"2+1?" → 3, không phải parse error
"asdfghjk" → "Mình chưa hiểu. Bạn muốn hỏi gì?"
im lặng khi ngữ cảnh không phù hợp

Gate trước. Trả lời sau.
Biết thì nói. Không biết thì hỏi. Không phù hợp thì im.
Handcode == Zero. Intelligence từ data + algorithm.
```
