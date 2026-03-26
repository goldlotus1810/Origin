# PLAN 12 — Response Intelligence: Từ "tra bảng" → "hiểu và nói"

**Vấn đề gốc:** HomeOS cảm đúng nhưng nói sai.
Emotion scoring chính xác (V=-0.65 cho "buồn"), tone đúng (Gentle/Supportive),
nhưng output là template cứng — "Bạn đang tìm hiểu để làm gì" cho mọi input.

**Root cause:** 5 mắt xích bị đứt trong pipeline:

```
1. walk_emotion() — code tồn tại nhưng KHÔNG implement (trả None)
2. STM recall — tính nhưng KHÔNG dùng trong response
3. Intent → Action — quá thô (90% input → AddClarify/WhatPurpose)
4. Template — chỉ nhìn valence number, bỏ qua context/recall/instinct
5. Instincts — 7 bản năng chạy nhưng kết quả KHÔNG đến response
```

**Mục tiêu:** Cùng input "tôi buồn vì mất việc":
- Trước: "Mình hiểu. Bạn muốn kể thêm không?"
- Sau: "Mình nghe bạn — mất việc là chuyện nặng nề. Bạn mất việc gần đây hay đã lâu?"

---

## Kiến trúc hiện tại vs mục tiêu

```
HIỆN TẠI (đứt 5 chỗ):
  text → emotion(✓) → tone(✓) → intent(✗) → template(✗) → generic output
                                    ↑ không dùng         ↑ chỉ nhìn V number
              walk_emotion(✗)  STM recall(✗)  instincts(✗)

MỤC TIÊU (nối lại):
  text → emotion → Silk walk → enriched emotion
                 → STM recall → context (lần đầu? lặp? liên quan gì?)
                 → intent (dùng context, không chỉ keywords)
                 → instincts (Causality, Analogy, Honesty)
                 → response composer (context + tone + instinct → text)
```

---

## Phụ thuộc

```
Không phụ thuộc Phase 8 (parser upgrade) — toàn bộ là Rust code.
Không cần Olang mới — chỉ wire Rust code đã có.
Chạy SONG SONG với Phase 8-11.
```

---

## 5 Tasks (theo thứ tự)

### 12.1 — Wire walk_emotion() vào response pipeline (~100 LOC)

**Vấn đề:** `origin.rs:2958` gọi `walk_emotion()` nhưng hàm trả None.

**Fix:**
```rust
fn walk_emotion(&self, text: &str) -> Option<EmotionTag> {
    // 1. Tách từ → lấy chain_hash cho mỗi từ
    // 2. Với mỗi từ: silk.neighbors(hash) → lấy edges có emotion
    // 3. walk_weighted: amplify emotion qua Silk edges (KHÔNG trung bình)
    // 4. Trả composite EmotionTag
}
```

**Kết quả:** "buồn" + Silk edge tới "mất việc" → V amplify từ -0.65 → -0.78

**DoD:**
- [ ] walk_emotion() trả EmotionTag thật (không None)
- [ ] "buồn vì mất việc" amplify mạnh hơn "buồn" đơn
- [ ] Test: 2 câu cùng "buồn" nhưng context khác → V khác

---

### 12.2 — Context recall trong response (~80 LOC)

**Vấn đề:** `recall_context()` được gọi, kết quả được pass nhưng template IGNORE nó.

**Fix:** Thêm `ResponseContext` struct chứa:
```rust
pub struct ResponseContext {
    /// Từ khóa chính user nhắc đến (extracted từ input)
    pub topics: Vec<String>,
    /// Số lần user nhắc topic tương tự (từ STM fire_count)
    pub repetition_count: u32,
    /// Silk neighbors mạnh nhất (concepts liên quan)
    pub related_concepts: Vec<(String, f32)>,
    /// Instinct results
    pub causality: Option<String>,   // "mất việc" causes "buồn"
    pub contradiction: bool,         // user nói trái ngược turn trước?
    pub novelty: f32,                // 0.0 = đã nói nhiều, 1.0 = hoàn toàn mới
}
```

**Wire:**
1. `process_input()` build ResponseContext từ STM + Silk + Instincts
2. Pass vào `render()` thay vì chỉ pass `original: Option<String>`

**DoD:**
- [ ] ResponseContext có topics từ input
- [ ] repetition_count > 0 khi user lặp topic
- [ ] related_concepts có Silk neighbors

---

### 12.3 — Intent estimation dùng context (~120 LOC)

**Vấn đề:** `estimate_intent()` chỉ dùng keywords → 90% input = AddClarify.

**Fix:** Thêm context-aware logic:
```rust
pub fn estimate_intent_v2(
    text: &str,
    cur_v: f32,
    arousal: f32,
    ctx: &ResponseContext,  // MỚI: context từ 12.2
) -> IntentEstimate {
    // 1. Nếu ctx.repetition_count > 3 → user đang xoay quanh 1 chủ đề
    //    → IntentKind::Heal (cần empathize sâu hơn, không hỏi clarify)
    // 2. Nếu ctx.causality.is_some() → user đã nêu nguyên nhân
    //    → Không cần hỏi "tìm hiểu để làm gì"
    // 3. Nếu ctx.novelty > 0.8 → chủ đề mới hoàn toàn
    //    → Observe (lắng nghe trước, chưa vội phản hồi)
    // 4. Nếu ctx.contradiction → mâu thuẫn
    //    → AskContext (nhưng hỏi về mâu thuẫn, không generic)
    // 5. Nếu V < -0.3 && topics không rỗng → cảm xúc + topic rõ
    //    → EmpathizeFirst (không AddClarify)

    // Fallback: existing keyword-based estimate
}
```

**DoD:**
- [ ] "tôi buồn vì mất việc" → EmpathizeFirst (không AddClarify)
- [ ] "tôi buồn" lần 4 → Heal sâu (không lặp template)
- [ ] "xin chào" → Chat (không AddClarify)
- [ ] "hôm nay trời đẹp" → Engaged (không hỏi "tìm hiểu gì")

---

### 12.4 — Response composer thay template (~200 LOC)

**Vấn đề:** Template chỉ nhìn V number → mọi câu "buồn" cùng output.

**Fix:** Thay `render()` bằng `compose_response()`:
```rust
pub fn compose_response(p: &ResponseParams, ctx: &ResponseContext) -> String {
    let mut parts: Vec<String> = Vec::new();

    // 1. Acknowledgment (từ tone + V, như cũ nhưng ngắn hơn)
    parts.push(acknowledgment(p.tone, p.valence));

    // 2. Context-specific (MỚI — từ topics + related_concepts)
    if let Some(topic) = ctx.topics.first() {
        parts.push(topic_response(topic, p.valence, ctx));
    }

    // 3. Follow-up (MỚI — dựa trên instincts)
    if ctx.repetition_count > 2 {
        // User lặp → hỏi cụ thể hơn
        parts.push(deepen_question(ctx));
    } else if ctx.novelty > 0.7 {
        // Chủ đề mới → mở rộng
        parts.push(open_question(ctx));
    } else if ctx.causality.is_some() {
        // Đã biết nguyên nhân → acknowledge cụ thể
        parts.push(cause_response(ctx));
    }

    parts.join(" ")
}

fn topic_response(topic: &str, v: f32, ctx: &ResponseContext) -> String {
    // Dùng topic thật trong câu trả lời
    // "mất việc" + V=-0.7 → "mất việc là chuyện nặng nề"
    // "thi rớt" + V=-0.5 → "thi rớt chắc bạn thất vọng lắm"
    // Không phải template — mà là GHÉP từ topic + emotion descriptor
}
```

**Emotion descriptors (tự sinh từ V):**
```
V < -0.7: "nặng nề", "đau", "khó khăn"
V < -0.4: "không dễ dàng", "áp lực", "lo lắng"
V < -0.1: "bận tâm", "suy nghĩ"
V > 0.3:  "vui", "tốt", "đáng mừng"
V > 0.6:  "tuyệt vời", "hạnh phúc"
```

**Follow-up questions (dựa vào context, không generic):**
```
repetition > 2: "Bạn đã nhắc đến {topic} nhiều lần — có điều gì cụ thể bạn muốn chia sẻ?"
novelty > 0.7:  "Kể cho mình nghe thêm về {topic}?"
has_cause:      "{cause} ảnh hưởng đến bạn thế nào?"
contradiction:  "Trước đó bạn nói {prev} — bây giờ khác rồi sao?"
```

**DoD:**
- [ ] "tôi buồn vì mất việc" → "Mình nghe bạn — mất việc là chuyện nặng nề. Bạn mất việc gần đây không?"
- [ ] "tôi rất vui hôm nay" → "Tuyệt! Có chuyện gì vui vậy?"
- [ ] "tôi giận lắm" → "Mình hiểu bạn đang giận. Chuyện gì xảy ra?"
- [ ] "xin chào" → "Chào bạn!" (KHÔNG hỏi "tìm hiểu gì")
- [ ] "tôi sợ quá" → "Mình ở đây. Bạn đang sợ điều gì?"

---

### 12.5 — Language detection fix + Instinct wire (~60 LOC)

**Vấn đề 1:** "xin chào" → English (không có dấu tiếng Việt)

**Fix:** Thêm common Vietnamese words KHÔNG DẤU vào detect_language():
```rust
let vi_nodiacritics = ["xin", "chao", "cam on", "toi", "ban", "vui", "buon",
                        "xin chao", "tam biet", "da", "vang", "khong"];
```

**Vấn đề 2:** Instincts chạy nhưng không wire vào response.

**Fix:** Trong `process_input()`, sau khi instincts chạy:
```rust
// Honesty instinct → set confidence
if instinct_result.honesty_confidence < 0.40 {
    // Thêm "[Chưa chắc chắn]" suffix — ĐÃ CÓ nhưng chỉ cho Observe
    // Mở rộng cho mọi action
}

// Causality → feed vào ResponseContext
if let Some(cause) = instinct_result.causality {
    ctx.causality = Some(cause);
}

// Contradiction → feed vào ResponseContext
if instinct_result.contradiction {
    ctx.contradiction = true;
}
```

**DoD:**
- [ ] "xin chào" → tiếng Việt
- [ ] "tam biet" → tiếng Việt
- [ ] Honesty < 0.40 → "[Chưa chắc chắn]" trên MỌI response
- [ ] Causality detected → pass vào ResponseContext

---

## Thứ tự thực hiện

```
12.1 (walk_emotion)     ← nền tảng, phải có trước
  ↓
12.2 (ResponseContext)  ← struct chứa context
  ↓
12.3 (intent v2)        ← dùng context để classify đúng
  ↓
12.4 (composer)         ← sinh text từ context, thay template
  ↓
12.5 (lang fix + instincts) ← polish
```

Ước tính: ~560 LOC Rust. Không cần Olang mới. Không break API cũ (thêm hàm mới, không xóa cũ).

---

## Rào cản

```
1. walk_emotion() cần word→chain_hash mapping
   → encoder.rs đã có encode_codepoint() nhưng cần encode_word()
   → Giải pháp: dùng FNV1a hash trực tiếp (giống learning.rs)

2. Topic extraction từ text thô
   → Không có NLP parser → dùng heuristic:
     - Bỏ stopwords (tôi, bạn, là, của, và, với, cho, để)
     - Từ còn lại = topics
     - Silk neighbors của topics = related concepts

3. Response composer không được hardcode
   → Dùng pattern: acknowledgment + topic_phrase + follow_up
   → Mỗi phần tự sinh từ data (V number → descriptor, topic → phrase)
   → KHÔNG thêm template mới — GHÉP từ thành phần

4. Test khó viết vì output là text tự nhiên
   → Test: output PHẢI chứa topic keyword
   → Test: output PHẢI khác nhau cho input khác nhau
   → Test: crisis vẫn hoạt động (regression)
```

---

## Definition of Done

- [ ] 8 test cases trong t16 đều response PHÙ HỢP (người đọc đánh giá)
- [ ] "xin chào" → tiếng Việt, không hỏi "tìm hiểu gì"
- [ ] "tôi buồn vì X" → response nhắc đến X
- [ ] "tôi vui vì Y" → response nhắc đến Y
- [ ] "tôi giận" → hỏi "chuyện gì" (không hỏi "tìm hiểu gì")
- [ ] Cùng "buồn" + context khác → response khác
- [ ] Lặp topic 3+ lần → response acknowledge sự lặp
- [ ] Crisis input → vẫn đúng (regression safe)
- [ ] All existing tests pass (0 failures)

---

## Ghi chú

Đây KHÔNG phải language model. HomeOS không sinh text từ probability distribution.
Đây là **compositional response** — ghép mảnh từ knowledge graph.
Giới hạn: chỉ "nói" được những gì nó "biết" qua Silk + STM + Instincts.
Nhưng tốt hơn NHIỀU so với template lookup.
