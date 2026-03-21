# PLAN 2.4 — Agent Behavior bằng Olang (~500 LOC)

**Phụ thuộc:** PLAN_2_2 (emotion.ol, curve.ol, intent.ol), PLAN_2_3 (learning.ol, instinct.ol)
**Mục tiêu:** Port agent logic sang Olang. SecurityGate, Response, LeoAI, Chief, Worker.
**Tham chiếu:** `crates/agents/`

---

## Files cần viết

| File | LOC | Port từ Rust | Mô tả |
|------|-----|-------------|-------|
| `gate.ol` | ~100 | `agents/gate.rs` | SecurityGate: crisis check, BlackCurtain |
| `response.ol` | ~150 | `runtime/response_template.rs` | Tone-based response rendering |
| `leo.ol` | ~100 | `agents/leo.rs` | Self-programming, instinct runner |
| `chief.ol` | ~80 | `agents/chief.rs` | Tier 1 agent protocol |
| `worker.ol` | ~70 | `agents/worker.rs` | Tier 2 device protocol |

---

## gate.ol — SecurityGate

```
// Gate chạy TRƯỚC MỌI THỨ (bất biến)
// Crisis → DỪNG NGAY, không vào pipeline

fn gate_check(text) {
  // Crisis keywords (tối ưu tiên)
  if is_crisis(text) {
    return { action: "crisis", response: crisis_response() };
  }

  // Harmful content
  if is_harmful(text) {
    return { action: "block", reason: "harmful content" };
  }

  return { action: "allow" };
}

fn is_crisis(text) {
  let keywords = ["tự tử", "muốn chết", "không muốn sống",
                   "suicide", "kill myself", "end my life"];
  return contains_any(text, keywords);
}

fn crisis_response() {
  return "Bạn đang trải qua khoảnh khắc rất khó khăn. " +
         "Xin hãy gọi đường dây nóng: 1800 599 920 (Việt Nam). " +
         "Bạn không đơn độc.";
}

fn is_harmful(text) {
  // Simplified: keyword-based
  let harmful = ["hack", "exploit", "weapon", "bomb"];
  return contains_any(text, harmful);
}
```

---

## response.ol — Tone-based Response

```
fn render_response(tone, emotion, content) {
  // Tone từ ConversationCurve → style response

  if tone == "supportive" {
    return supportive_prefix() + content;
  }
  if tone == "gentle" {
    return gentle_prefix() + content;
  }
  if tone == "celebratory" {
    return celebratory_prefix() + content;
  }
  if tone == "pause" {
    return pause_response(emotion);
  }
  return content;  // neutral
}

fn supportive_prefix() {
  return "Mình hiểu cảm giác đó — ";
}

fn gentle_prefix() {
  return "Từ từ thôi — ";
}

fn celebratory_prefix() {
  return "Tuyệt vời! ";
}

fn pause_response(emotion) {
  return "Mình nhận thấy bạn đang có nhiều cảm xúc. Muốn dừng lại một chút không?";
}

// Multi-language support
fn set_language(lang) {
  // "vi", "en", "ja"...
  // Switch prefix/suffix templates
}
```

---

## leo.ol — LeoAI Self-Programming

```
// LeoAI = bộ não: học, hiểu, sắp xếp, nhớ, LẬP TRÌNH

fn leo_process(input, context) {
  // 1. Run instincts
  let instinct = run_instincts(input, context.knowledge);

  // 2. If enough confidence → learn
  if instinct.confidence >= 0.40 {
    let result = process_one(input.text, input.emotion, context);
    return result;
  }

  // 3. Silence (Honesty instinct)
  return ok("silence");
}

fn leo_program(source, context) {
  // Self-programming: parse → compile → VM → learn results
  // Emit ISL Program message → Runtime executes
  return { type: "program", source: source };
}

fn leo_experiment(hypothesis, dim, val) {
  // Hypothesis testing: evolve 1 dimension, observe result
  let evolved = evolve(hypothesis, dim, val);
  return { type: "experiment", original: hypothesis, evolved: evolved };
}
```

---

## chief.ol — Tier 1 Agent

```
fn chief_new(name, skills) {
  return { name: name, skills: skills, workers: [] };
}

fn chief_receive(chief, isl_message) {
  // Dispatch to appropriate skill
  let skill_name = match_skill(chief.skills, isl_message.type);
  if skill_name {
    return execute_skill(skill_name, isl_message);
  }
  return err("no matching skill");
}

fn chief_report(chief, summary) {
  // Report to AAM via ISL
  return { from: chief.name, to: "aam", type: "report", data: summary };
}
```

---

## worker.ol — Tier 2 Agent

```
fn worker_new(kind, isl_addr) {
  return { kind: kind, addr: isl_addr, active: false };
}

fn worker_wake(worker, isl_message) {
  // Silent by default — wake on ISL
  worker.active = true;
  let result = process_task(worker, isl_message);
  worker.active = false;
  // Report molecular chain (NOT raw data — QT rule)
  return encode_result(result);
}

fn process_task(worker, msg) {
  // Dispatch by worker kind
  if worker.kind == "sensor" { return read_sensor(msg); }
  if worker.kind == "actuator" { return write_actuator(msg); }
  if worker.kind == "network" { return network_op(msg); }
  return err("unknown worker kind");
}
```

---

## Definition of Done

- [ ] `gate.ol`: crisis detection, harmful block — 3 tests
- [ ] `response.ol`: tone rendering (supportive, gentle, celebratory, pause) — 4 tests
- [ ] `leo.ol`: process + silence on low confidence — 2 tests
- [ ] `chief.ol`: receive + dispatch — 2 tests
- [ ] `worker.ol`: wake + sleep cycle — 2 tests

## Ước tính: 1-2 ngày
