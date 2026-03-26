# PLAN T3 — Response Wire: Chi Tiet Trien Khai 12.1–12.5

**Nguon:** PLAN_12_RESPONSE_INTELLIGENCE.md
**Ngay tao:** 2026-03-22
**Muc tieu:** HomeOS cam dung VA noi dung — response phan anh context, topic, instincts

---

## Trang Thai Hien Tai (DOC TRUOC)

```
PHAT HIEN: Mot phan cong viec Plan 12 DA DUOC IMPLEMENT.

✅ walk_emotion() — DA IMPLEMENT (KHONG con tra None)
   File: crates/runtime/src/core/origin.rs:2875-2918
   Walk qua Silk graph, dung sentence_affect(), tra composite EmotionTag
   Ket qua duoc gan vao walk_tag va truyen vao resp_ctx.walk_valence (dong 3410)

✅ ResponseContext struct — DA TAO
   File: crates/runtime/src/output/response_template.rs:153-166
   Co: topics, repetition_count, causality, contradiction, novelty, walk_valence

✅ ResponseContext duoc populate
   File: crates/runtime/src/core/origin.rs:3383-3439
   Topics tu stop-word filter, repetition tu STM fire_count,
   walk_valence tu walk_tag, novelty tu fire_count, instinct results

✅ compose_response() — DA TAO
   File: crates/runtime/src/output/response_template.rs:172-249
   3-part: acknowledgment + topic_phrase + follow_up
   DA duoc goi tu process_input (dong 3618)

✅ detect_language() — DA CO vi_nodiacritics
   File: crates/runtime/src/output/response_template.rs:58-97
   Co: "xin chao", "cam on", "tam biet", "buon", "vui", etc.

✅ Context-aware intent override — DA IMPLEMENT
   File: crates/runtime/src/core/origin.rs:3441-3458
   AddClarify + causality → EmpathizeFirst
   AddClarify + repetition > 2 → EmpathizeFirst
   Observe + topic + V < -0.3 → EmpathizeFirst

⚠️ CON THIEU / CAN TANG CUONG:
   1. compose_response() chi duoc goi cho Observe — cac action khac van dung render()
   2. estimate_intent() van keyword-only (context/analysis/intent.rs:398)
   3. Instinct wire chua day du — honesty confidence chua anh huong response
   4. related_concepts chua co trong ResponseContext
   5. estimate_intent_v2() chua duoc tao (van dung keyword-based)
```

---

## So Do Luong Du Lieu (Wire Diagram)

```
HIEN TAI:
                                    ┌─ walk_emotion() ─→ walk_tag ─→ resp_ctx.walk_valence ✅
                                    │
  text ─→ sentence_affect() ─→ raw_tag
       ─→ infer_context()          │
       ─→ estimate_intent()  ─────────→ est (keyword-only) ─→ decide_action() ─→ action
                                    │
                                    ├─ learning.process_one() ─→ proc_result
                                    │
                                    ├─ run_instincts() ─→ instinct_ctx ─→ resp_ctx.contradiction ✅
                                    │                                  ─→ resp_ctx.causality ✅
                                    │
                                    ├─ BUILD resp_ctx ✅ (topics, repetition, novelty, walk_valence)
                                    │
                                    ├─ context-aware override ✅ (AddClarify → EmpathizeFirst)
                                    │
                                    └─ T7: action ─→ render() hoac compose_response()
                                                      │
                                         Observe ─→ compose_response(p, ctx) ✅
                                         Khac   ─→ render(p) ← VAN DUNG TEMPLATE CU ⚠️

MUC TIEU:
  text ─→ [unchanged T1-T6] ─→ action
       ─→ resp_ctx (enriched: + related_concepts, + honesty)
       ─→ estimate_intent_v2(text, v, a, ctx)  ← THAY estimate_intent()
       ─→ MOI action ─→ compose_response(p, ctx)  ← THAY render()
       ─→ honesty < 0.40 → "[Chua chac chan]" suffix
```

---

## 5 Tasks Chi Tiet

### T3.1 — Kiem Tra & Tang Cuong walk_emotion() (~20-30 LOC)

**Trang thai:** DA IMPLEMENT — can kiem tra chat luong va them tests

**File:** `crates/runtime/src/core/origin.rs:2875-2918`

**Code hien tai:**
```rust
fn walk_emotion(&self, query: &str) -> Option<silk::edge::EmotionTag> {
    let words: Vec<&str> = query.split_whitespace()
        .filter(|w| w.chars().count() > 1)
        .take(8).collect();
    if words.is_empty() { return None; }

    let mut word_hashes = Vec::new();
    let mut word_emotions = Vec::new();
    for w in &words {
        let low = w.to_lowercase();
        let h = olang::hash::fnv1a_str(&low);
        word_hashes.push(h);
        let emo = if let Some(obs) = self.learning.stm().find_by_hash(h) {
            obs.emotion
        } else {
            let raw = context::emotion::word_affect(&low);
            silk::edge::EmotionTag::new(raw.valence, raw.arousal, 0.5, raw.arousal.abs().max(0.3))
        };
        word_emotions.push(emo);
    }
    let result = silk::walk::sentence_affect(
        self.learning.graph(), &word_hashes, &word_emotions, 8,
    );
    if result.total_weight < 0.001 { return None; }
    Some(result.composite)
}
```

**Hanh vi hien tai vs mong muon:**
```
Hien tai:
  "buon" → word_affect("buon") → V ≈ -0.65 → walk qua Silk
  "buon vi mat viec" → 3 tu → walk → composite (amplified)
  NEU Silk graph khong co edges → total_weight < 0.001 → None

Mong muon:
  "buon vi mat viec" → V < -0.70 (amplified manh hon "buon" don)
  Luon tra ket qua (khong None) neu co >= 1 tu co emotion
```

**De xuat fix:**
```rust
// Thay doi threshold: fallback ve word_emotions trung binh NEU walk khong du manh
if result.total_weight < 0.001 {
    // Fallback: tra emotion trung binh tu word-level (khong walk)
    // AMPLIFY, khong trung binh (theo spec)
    let mut max_intensity = silk::edge::EmotionTag::NEUTRAL;
    for emo in &word_emotions {
        if emo.intensity > max_intensity.intensity {
            max_intensity = *emo;
        }
    }
    if max_intensity.intensity > 0.1 {
        return Some(max_intensity);
    }
    return None;
}
```

**Tests can them:**
```rust
#[test]
fn walk_emotion_single_sad_word() {
    // Setup runtime voi STM co "buon"
    let mut rt = test_runtime();
    let tag = rt.walk_emotion("buon");
    assert!(tag.is_some());
    assert!(tag.unwrap().valence < -0.30);
}

#[test]
fn walk_emotion_amplifies_context() {
    let mut rt = test_runtime();
    let tag1 = rt.walk_emotion("buon");
    let tag2 = rt.walk_emotion("buon vi mat viec");
    // Voi context, valence phai manh hon
    assert!(tag2.unwrap().valence <= tag1.unwrap().valence);
}
```

**Lenh kiem tra:**
```bash
cargo test -p runtime -- walk_emotion
cargo test -p runtime -- process_input
```

**Uoc tinh LOC:** 20-30
**Rui ro:** THAP — chi tang cuong fallback
**Dinh nghia hoan thanh:**
- [ ] walk_emotion("buon") tra Some(...) voi V < -0.30
- [ ] walk_emotion("buon vi mat viec") tra V manh hon walk_emotion("buon")
- [ ] walk_emotion("xin chao") tra Some hoac None (khong panic)
- [ ] 2+ tests moi PASS

---

### T3.2 — Mo Rong ResponseContext + related_concepts (~40-60 LOC)

**Trang thai:** Struct co nhung thieu related_concepts va honesty_confidence

**File 1:** `crates/runtime/src/output/response_template.rs:153-166`
**File 2:** `crates/runtime/src/core/origin.rs:3383-3439`

**Code hien tai ResponseContext:**
```rust
pub struct ResponseContext {
    pub topics: Vec<String>,
    pub repetition_count: u32,
    pub causality: Option<String>,
    pub contradiction: bool,
    pub novelty: f32,
    pub walk_valence: Option<f32>,
}
```

**De xuat them truong:**
```rust
pub struct ResponseContext {
    pub topics: Vec<String>,
    pub repetition_count: u32,
    pub causality: Option<String>,
    pub contradiction: bool,
    pub novelty: f32,
    pub walk_valence: Option<f32>,
    // MOI:
    /// Silk neighbors manh nhat (concepts lien quan)
    pub related_concepts: Vec<(String, f32)>,  // (concept_name, weight)
    /// Honesty confidence tu instinct (0.0-1.0)
    pub honesty_confidence: f32,
}
```

**Populate related_concepts (them vao origin.rs sau dong 3411):**
```rust
// Related concepts tu Silk graph — lay neighbors co weight cao nhat
if let ProcessResult::Ok { ref chain, .. } = proc_result {
    let hash = chain.chain_hash();
    let graph = self.learning.graph();
    let neighbors = graph.edges_from(hash);
    ctx.related_concepts = neighbors.iter()
        .filter(|e| e.weight >= 0.30)
        .take(3)
        .filter_map(|e| {
            // Try lay ten tu registry
            self.registry.lookup_hash(e.to_hash)
                .map(|name| (name.to_string(), e.weight))
        })
        .collect();
}
```

**Populate honesty_confidence (them vao origin.rs sau dong 3421):**
```rust
if let Some(ref insight) = instinct_ctx {
    ctx.contradiction = insight.has_contradiction;
    ctx.honesty_confidence = insight.honesty_confidence.unwrap_or(1.0);
    // ... causality ...
}
```

**Lenh kiem tra:**
```bash
cargo test -p runtime -- response_context
cargo test -p runtime -- process_input
```

**Uoc tinh LOC:** 40-60
**Rui ro:** THAP — them truong, khong sua logic cu
**Dinh nghia hoan thanh:**
- [ ] ResponseContext co related_concepts va honesty_confidence
- [ ] related_concepts duoc populate tu Silk graph
- [ ] honesty_confidence duoc populate tu instinct
- [ ] cargo test --workspace PASS

---

### T3.3 — estimate_intent_v2() Dung Context (~100-120 LOC)

**Trang thai:** CHUA IMPLEMENT — hien tai chi co keyword-based estimate_intent()

**File:** `crates/context/src/analysis/intent.rs:398`

**Code hien tai:**
```rust
pub fn estimate_intent(text: &str, cur_v: f32, cur_a: f32) -> IntentEstimate {
    // Chi dung keywords — khong co context
    // 90% input → AddClarify voi WhatPurpose
}
```

**Hanh vi hien tai vs mong muon:**
```
Hien tai:
  "toi buon vi mat viec" → scan keywords → Heal (vi "buon")
                         → nhung decide_action() van co the → AddClarify
  "xin chao"             → khong match keyword nao → Learn baseline → AddClarify
  "toi buon" (lan thu 4) → van Heal → van cung response

Mong muon:
  "toi buon vi mat viec" → EmpathizeFirst (da co causality, khong hoi "tim hieu gi")
  "xin chao"             → Chat (greeting → tra loi greeting)
  "toi buon" (lan thu 4) → Heal (sau hon, khong lap template)
  "hom nay troi dep"     → Engaged (binh thuong, khong hoi clarify)
```

**De xuat — ham moi estimate_intent_v2():**
```rust
// File: crates/context/src/analysis/intent.rs

use crate::ResponseContext;  // hoac path tuong ung

/// Context-aware intent estimation — thay keyword-only.
///
/// Dung ResponseContext de dieu chinh intent phu hop hon.
/// Fallback ve estimate_intent() neu khong co context.
pub fn estimate_intent_v2(
    text: &str,
    cur_v: f32,
    cur_a: f32,
    ctx: Option<&ResponseContext>,
) -> IntentEstimate {
    // Base: keyword estimation (giu nguyen logic cu)
    let mut est = estimate_intent(text, cur_v, cur_a);

    // Neu khong co context → tra base
    let ctx = match ctx {
        Some(c) => c,
        None => return est,
    };

    // Override rules (theo thu tu uu tien):

    // 1. User lap topic > 3 lan → chuyen sang Heal (can empathize sau)
    if ctx.repetition_count > 3 && cur_v < -0.20 {
        est.primary = IntentKind::Heal;
        est.confidence = est.confidence.max(0.75);
    }

    // 2. Da co causality → khong can hoi "tim hieu gi"
    //    (origin.rs da co override nay o T7a2, nhung lam o day chinh xac hon)
    if ctx.causality.is_some() && est.primary == IntentKind::Learn {
        if cur_v < -0.30 {
            est.primary = IntentKind::Heal;
        } else {
            est.primary = IntentKind::Inform;
        }
    }

    // 3. Novelty cao (chuyen de moi hoan toan) → Explore
    if ctx.novelty > 0.80 && est.primary == IntentKind::Learn {
        est.primary = IntentKind::Explore;
    }

    // 4. Contradiction → chuyen sang Research (tim hieu tai sao mau thuan)
    if ctx.contradiction {
        est.primary = IntentKind::Research;
        est.confidence = est.confidence.max(0.70);
    }

    // 5. Greeting detection — "xin chao", "hello"
    let lo = text.to_lowercase();
    let greeting_words = ["xin chao", "chao ban", "hello", "hi ", "hey "];
    if greeting_words.iter().any(|g| lo.contains(g)) {
        est.primary = IntentKind::Chat;
        est.confidence = 0.90;
    }

    // 6. Emotion manh + topic ro → EmpathizeFirst (thong qua Heal)
    if cur_v < -0.50 && !ctx.topics.is_empty() {
        est.primary = IntentKind::Heal;
        est.confidence = est.confidence.max(0.80);
    }

    est
}
```

**Wire vao origin.rs — thay estimate_intent() (dong 3035):**
```rust
// TRUOC:
let est = estimate_intent(text_for_intent, cur_v, raw_tag.arousal);

// SAU (sau khi build resp_ctx — can doi thu tu code):
// Option 1: Van dung estimate_intent() roi override o T7a2 (DA LAM)
// Option 2: Build resp_ctx TRUOC, roi dung estimate_intent_v2()
//   → Can refactor thu tu: build ctx truoc T4
```

**LUU Y QUAN TRONG:**
Hien tai resp_ctx duoc build SAU estimate_intent() (dong 3383).
De dung estimate_intent_v2(ctx), can:
- Option A: Build resp_ctx BASIC (chi topics, repetition) TRUOC estimate_intent
- Option B: Giu 2-pass: estimate_intent → build ctx → override (DA LAM o T7a2)

**Khuyen nghi:** Giu Option B (2-pass) vi da hoat dong. Chuyen logic T7a2 vao
estimate_intent_v2() de gon gang hon, nhung KHONG thay doi thu tu pipeline.

**Lenh kiem tra:**
```bash
cargo test -p context -- estimate_intent
cargo test -p runtime -- process_input
```

**Uoc tinh LOC:** 100-120
**Rui ro:** TRUNG BINH — can xu ly circular dependency (ctx can intent, intent can ctx)
**Dinh nghia hoan thanh:**
- [ ] "toi buon vi mat viec" → KHONG phai AddClarify
- [ ] "xin chao" → Chat intent
- [ ] "toi buon" lan 4 → Heal (khong lap)
- [ ] estimate_intent_v2() co tests rieng
- [ ] Khong break tests cu

---

### T3.4 — compose_response() Thay The render() Toan Bo (~80-120 LOC)

**Trang thai:** compose_response() DA CO nhung chi dung cho Observe

**File 1:** `crates/runtime/src/output/response_template.rs:172-249`
**File 2:** `crates/runtime/src/core/origin.rs:3618` (chi Observe goi compose_response)

**Hanh vi hien tai vs mong muon:**
```
Hien tai:
  Observe    → compose_response(p, ctx) ✅
  AddClarify → render(p) ← template cu: "Ban dang tim hieu de lam gi?"
  Empathize  → render(p) ← template cu: "Minh nghe ban. Ban muon ke them khong?"
  Proceed    → render(p) ← template cu hoac original

Mong muon:
  MOI action → compose_response(p, ctx)
  compose_response() xu ly:
    - AddClarify: ack + topic_phrase + clarify_question (CU THE hon, dung topic)
    - EmpathizeFirst: ack + topic_phrase + follow_up (TU NHIEN hon)
    - Proceed: ack + topic_phrase + original (NEU co recall)
```

**De xuat sua compose_response() (response_template.rs):**
```rust
pub fn compose_response(p: &ResponseParams, ctx: &ResponseContext) -> String {
    // Action dac biet → van dung render() (Crisis, SoftRefusal, etc.)
    match &p.action {
        IntentAction::CrisisOverride
        | IntentAction::SoftRefusal
        | IntentAction::UserConfirm
        | IntentAction::UserDeny
        | IntentAction::ForceLearnQR
        | IntentAction::ConfirmLearnQR
        | IntentAction::SilentAck
        | IntentAction::HomeControl => return render(p),
        _ => {}
    }

    // THEM: AskContext — dung ctx de hoi cu the hon
    if let IntentAction::AskContext { angry } = &p.action {
        if let Some(topic) = ctx.topics.first() {
            let lang = p.language;
            return match lang {
                Lang::Vi => {
                    if *angry || p.valence < -0.50 {
                        format!("Minh thay ban dang co cam xuc manh ve {}. Ke cho minh nghe?", topic)
                    } else {
                        format!("Ve {} — ban dang nghi den tinh huong nao cu the?", topic)
                    }
                }
                Lang::En => {
                    if *angry || p.valence < -0.50 {
                        format!("I can see you feel strongly about {}. Tell me more?", topic)
                    } else {
                        format!("About {} — what specific situation?", topic)
                    }
                }
            };
        }
        return render(p);  // fallback neu khong co topic
    }

    // THEM: AddClarify — dung topic thay vi generic question
    if let IntentAction::AddClarify { kind } = &p.action {
        if let Some(topic) = ctx.topics.first() {
            let lang = p.language;
            let effective_v = ctx.walk_valence.unwrap_or(p.valence);
            let ack = match lang {
                Lang::Vi => acknowledgment_vi(p.tone, effective_v),
                Lang::En => acknowledgment_en(p.tone, effective_v),
            };
            let clarify = match lang {
                Lang::Vi => format!("Ve {} — ban co the noi them khong?", topic),
                Lang::En => format!("About {} — could you tell me more?", topic),
            };
            if ack.is_empty() {
                return clarify;
            }
            return format!("{} {}", ack, clarify);
        }
        return render(p);
    }

    // ... phan con lai giu nguyen (EmpathizeFirst, Proceed, Observe da xu ly) ...
    // [CODE HIEN TAI tu dong 188-249]
}
```

**Wire vao origin.rs — thay render() bang compose_response():**
```rust
// File: crates/runtime/src/core/origin.rs
// Tim tat ca cho goi render() trong T7 section va thay bang compose_response()

// Vi du dong ~3637 (normal flow):
// TRUOC:
let text = render(&ResponseParams { ... });
// SAU:
let text = compose_response(&ResponseParams { ... }, &resp_ctx);
```

**Cac cho can thay doi trong origin.rs (search "render(&"):**
```
Dong ~3618: compose_response — DA DUNG cho Observe ✅
Dong ~3649: contextual_reply / natural_reply — can wire qua compose_response
Dong ~3660+: render(&ResponseParams{...}) — THAY bang compose_response()
```

**Lenh kiem tra:**
```bash
cargo test -p runtime -- response
cargo test -p runtime -- compose_response
cargo test -p runtime -- process_input
```

**Test can them:**
```rust
#[test]
fn compose_response_sad_with_topic() {
    let p = ResponseParams {
        tone: ResponseTone::Supportive,
        action: IntentAction::EmpathizeFirst,
        valence: -0.65,
        fx: -0.5,
        context: None,
        original: None,
        language: Lang::Vi,
    };
    let ctx = ResponseContext {
        topics: vec!["mat viec".to_string()],
        repetition_count: 1,
        causality: Some("mat viec".to_string()),
        novelty: 0.85,
        walk_valence: Some(-0.72),
        ..Default::default()
    };
    let r = compose_response(&p, &ctx);
    assert!(r.contains("mat viec"), "Response phai nhac den topic: {}", r);
    assert!(!r.contains("tim hieu"), "Khong duoc hoi 'tim hieu gi': {}", r);
}

#[test]
fn compose_response_happy() {
    let p = ResponseParams {
        tone: ResponseTone::Celebratory,
        action: IntentAction::Proceed,
        valence: 0.65,
        fx: 0.5,
        context: None,
        original: None,
        language: Lang::Vi,
    };
    let ctx = ResponseContext {
        topics: vec!["thanh cong".to_string()],
        novelty: 0.85,
        ..Default::default()
    };
    let r = compose_response(&p, &ctx);
    assert!(!r.is_empty());
}

#[test]
fn compose_response_greeting() {
    let p = ResponseParams {
        tone: ResponseTone::Engaged,
        action: IntentAction::Proceed,
        valence: 0.10,
        fx: 0.0,
        context: None,
        original: Some("Chao ban!".to_string()),
        language: Lang::Vi,
    };
    let ctx = ResponseContext::default();
    let r = compose_response(&p, &ctx);
    assert!(!r.contains("tim hieu"), "Greeting khong duoc hoi 'tim hieu': {}", r);
}
```

**Uoc tinh LOC:** 80-120
**Rui ro:** TRUNG BINH — can thay nhieu cho goi render() trong origin.rs
**Dinh nghia hoan thanh:**
- [ ] "toi buon vi mat viec" → response nhac den "mat viec"
- [ ] "toi vui vi Y" → response nhac den Y
- [ ] "xin chao" → KHONG hoi "tim hieu gi"
- [ ] Cung "buon" + context khac → response khac
- [ ] Crisis van dung (regression safe)
- [ ] 3+ tests moi PASS

---

### T3.5 — Language Detection Fix + Honesty Wire (~30-50 LOC)

**Trang thai:** detect_language() DA CO vi_nodiacritics. Honesty chua wire.

**File 1:** `crates/runtime/src/output/response_template.rs:58-97`
**File 2:** `crates/runtime/src/core/origin.rs:3420` (instinct wire)

**detect_language() hien tai:**
```rust
pub fn detect_language(text: &str) -> Lang {
    // DA CO:
    // - Vietnamese diacritics check ✅
    // - Common Vietnamese words (co dau) ✅
    // - vi_nodiac: "xin chao", "cam on", "tam biet", "buon", "vui", ... ✅
    // - Single-word check >= 2 match ✅
}
```

**Hanh vi hien tai vs mong muon:**
```
Hien tai:
  "xin chao"  → check vi_nodiac → "xin chao" match → Vi ✅ (DA FIX)
  "tam biet"   → check vi_nodiac → "tam biet" match → Vi ✅
  "toi"        → single word → chi 1 match → En ⚠️ (can >= 2)
  "toi buon"   → "toi" + "buon" → buon trong vi_nodiac → Vi ✅

Mong muon:
  "toi" (1 tu) → Van la Vi (nhung kho, vi "toi" co the la En)
  → CHAP NHAN HIEN TAI: >= 2 tu match moi la Vi. Hop ly.
```

**De xuat fix cho detect_language() — them tu:**
```rust
// Them vao vi_nodiac (dong 74):
let vi_nodiac = [
    "xin chao", "cam on", "tam biet", "xin", "chao", "buon", "vui",
    "giup", "sao", "nhe", "da", "vang", "roi",
    // THEM:
    "oi", "ay", "nay", "kia", "lam", "qua", "that",
];

// Them vao vi_single (dong 86):
let vi_single = ["toi", "ban", "khong", "duoc", "lam",
    // THEM:
    "minh", "co", "nha", "nhe",
];
```

**Honesty wire — them vao compose_response():**
```rust
// File: crates/runtime/src/output/response_template.rs
// Trong compose_response(), CUOI CUNG truoc khi return:

pub fn compose_response(p: &ResponseParams, ctx: &ResponseContext) -> String {
    // ... (logic hien tai) ...

    let mut result = parts.join(" ");

    // Honesty: confidence thap → them suffix
    if ctx.honesty_confidence < 0.40 && !result.is_empty() {
        let suffix = match p.language {
            Lang::Vi => " [Chua chac chan]",
            Lang::En => " [Not certain]",
        };
        result.push_str(suffix);
    }

    result
}
```

**Wire honesty_confidence tu instinct vao origin.rs:**
```rust
// File: crates/runtime/src/core/origin.rs, dong ~3420
if let Some(ref insight) = instinct_ctx {
    ctx.contradiction = insight.has_contradiction;
    // THEM:
    ctx.honesty_confidence = insight.honesty_confidence.unwrap_or(1.0);
}
```

**LUU Y:** Can kiem tra `InstinctInsight` struct co truong `honesty_confidence` chua.
Neu chua, can them vao `crates/agents/src/instinct/` tuong ung.

**Lenh kiem tra:**
```bash
cargo test -p runtime -- detect_language
cargo test -p runtime -- compose_response
cargo test -p runtime -- honesty
```

**Test can them:**
```rust
#[test]
fn detect_language_xin_chao() {
    assert_eq!(detect_language("xin chao"), Lang::Vi);
}

#[test]
fn detect_language_tam_biet() {
    assert_eq!(detect_language("tam biet"), Lang::Vi);
}

#[test]
fn detect_language_english() {
    assert_eq!(detect_language("hello world"), Lang::En);
}

#[test]
fn compose_response_low_honesty_adds_suffix() {
    let p = ResponseParams {
        tone: ResponseTone::Engaged,
        action: IntentAction::Proceed,
        valence: 0.10,
        fx: 0.0,
        context: None,
        original: Some("Day la thong tin.".to_string()),
        language: Lang::Vi,
    };
    let ctx = ResponseContext {
        honesty_confidence: 0.30,
        ..Default::default()
    };
    let r = compose_response(&p, &ctx);
    assert!(r.contains("[Chua chac chan]"), "Honesty thap phai co suffix: {}", r);
}
```

**Uoc tinh LOC:** 30-50
**Rui ro:** THAP — them tu va suffix, khong thay doi logic lon
**Dinh nghia hoan thanh:**
- [ ] "xin chao" → Lang::Vi ✅ (DA DUNG)
- [ ] "tam biet" → Lang::Vi ✅
- [ ] Honesty < 0.40 → "[Chua chac chan]" tren MOI response
- [ ] 4+ tests moi PASS

---

## Thu Tu Thuc Hien

```
T3.1 (walk_emotion tang cuong)  ← Nhanh, nen tang cho T3.2
  ↓
T3.2 (ResponseContext mo rong)  ← Them related_concepts + honesty_confidence
  ↓
T3.3 (estimate_intent_v2)      ← LON NHAT, dung context
  ↓
T3.4 (compose_response toan bo) ← Thay render() moi noi
  ↓
T3.5 (lang fix + honesty wire)  ← Polish cuoi cung
```

**Tong uoc tinh:** 270-380 LOC Rust, 4-6h
**Khong can Olang moi — chi Rust code.**
**Khong break API cu — them ham moi, khong xoa cu.**

---

## Danh Gia Rui Ro Toan Bo

```
Rui ro                              Muc do    Giam thieu
──────────────────────────────────────────────────────────────
Circular dependency                 TRUNG BINH  Giu 2-pass (estimate → build ctx → override)
(ctx can intent, intent can ctx)                Khong refactor pipeline order

compose_response() break template   TRUNG BINH  Giu render() lam fallback
                                                  Moi action chua handle → fallback render()

Honesty suffix gay confuse user     THAP        Chi hien khi confidence < 0.40
                                                  User co the tat (config)

ResponseContext struct thay doi     THAP        Them truong voi Default → backward compat
                                                  Khong xoa truong cu

walk_emotion fallback                THAP       Max-intensity thay vi trung binh
                                                  Spec noi AMPLIFY → chon manh nhat

Performance (them processing)        RAT THAP   Chi them vai phep so sanh + format
                                                  Khong co vong lap nang
```

---

## Lenh Kiem Tra Toan Dien

```bash
# 1. Tests workspace
cargo test --workspace

# 2. Tests cu the
cargo test -p runtime -- walk_emotion
cargo test -p runtime -- compose_response
cargo test -p runtime -- detect_language
cargo test -p runtime -- process_input
cargo test -p context -- estimate_intent

# 3. Integration
cargo test -p intg

# 4. Clippy
cargo clippy --workspace

# 5. Smoke binary
make smoke-binary
```

---

## Dinh Nghia Hoan Thanh Toan Bo T3

```
✅ walk_emotion() tra ket qua thuc (khong None) cho input co emotion
✅ ResponseContext co related_concepts + honesty_confidence
✅ "toi buon vi mat viec" → response nhac den "mat viec" (khong generic)
✅ "toi vui vi Y" → response nhac den Y
✅ "xin chao" → tieng Viet, khong hoi "tim hieu gi"
✅ "toi gian" → hoi "chuyen gi" (khong hoi "tim hieu gi")
✅ Cung "buon" + context khac → response KHAC
✅ Lap topic 3+ lan → response acknowledge su lap
✅ Honesty < 0.40 → "[Chua chac chan]"
✅ Crisis input → van dung (regression safe)
✅ cargo test --workspace → 0 FAILED
✅ cargo clippy --workspace → 0 warnings
```

---

## Ghi Chu

```
Day KHONG phai language model — HomeOS khong sinh text tu probability distribution.
Day la COMPOSITIONAL RESPONSE — ghep manh tu knowledge graph.
Gioi han: chi "noi" duoc nhung gi no "biet" qua Silk + STM + Instincts.
Nhung tot hon NHIEU so voi template lookup.
```
