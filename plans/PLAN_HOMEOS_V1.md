# PLAN — HomeOS v1.0 + origin.olang v1.0

> **Date:** 2026-03-25
> **By:** Nox (builder), dua tren phan tich cua Sora (docs/sora/HOMEOS_ON_OLANG1.md)
> **Nguyen tac:** Gate truoc, tra loi sau. Handcode == Zero. Intelligence tu data + algorithm.

---

## Hien trang

```
Olang 1.0:  DONE. 1,008KB. Full functional language.
            map/filter/reduce/pipe/sort/split/join/contains/lambda/persistence.

HomeOS:     ~10,000 LOC. 10-stage pipeline. 90 hardcoded patterns.
            Van de: "hi" → tra fact random. "2+1?" → parse error.
            agent_respond() = if-else spaghetti.
```

## Muc tieu

```
HomeOS v1.0: Classifier → Router → Gate → Compose
  "hi" → greeting (khong tra fact)
  "2+1?" → 3 (math eval)
  "viet nam o dau?" → tim dung fact (case-insensitive)
  "asdfghjk" → "Minh chua hieu. Ban muon hoi gi?"
  Biet thi noi. Khong biet thi hoi. Khong phu hop thi im.
```

---

## Kien truc moi

```
Input
  |
  v
CLASSIFIER (classify input type)
  |
  +-- "math"     → eval_math("2+1?") → emit 3
  +-- "code"     → compile + run (hien tai)
  +-- "command"  → dispatch (learn/save/load/test/build/memory/fns/help/exit)
  +-- "greeting" → smart_greet(context)
  +-- "question" → GATE → knowledge search → compose response
  +-- "chat"     → GATE → knowledge search → compose response
```

## Sprints

### Sprint 1: Foundation (~80 LOC) — stdlib/homeos/classify.ol

```
1. _str_lower(s)          — lowercase converter (ASCII)
2. _str_eq_ci(a, b)       — case-insensitive string compare
3. classify(input)         — router: math/code/command/greeting/question/chat
4. Tests: 20+ classify cases
```

### Sprint 2: Handlers (~60 LOC) — stdlib/homeos/handlers.ol

```
5. eval_math(input)        — strip ?/= → compile as "emit expr"
6. smart_greet(context)    — context-aware greeting (first visit vs returning)
7. ask_back(input)         — "Minh chua hieu. Ban muon hoi gi?"
```

### Sprint 3: Gate (~50 LOC) — update encoder.ol

```
8. knowledge_search_scored(query)  — return { text, score } (not just text)
9. gate_decide(input)              — score >= HIGH → respond, LOW → ask_back, ZERO → unknown
10. Update knowledge_search: case-insensitive matching (_str_eq_ci)
```

### Sprint 4: Integrate (~50 LOC) — update repl.ol

```
11. New repl_eval flow:
    input → classify → router → handler → output
    Thay the hien tai: input → tokenize → compile → eval (moi thu la code)

12. agent_respond_v2:
    classify → gate_decide → compose from knowledge (khong template)

13. Integration tests: 30+ scenarios
```

### Sprint 5: Polish (~40 LOC)

```
14. Error messages: "Parse error" → "Minh khong hieu cu phap. Thu: emit 2+3"
15. Empty input handling
16. Multi-line support (?)
17. Update help text
```

---

## Tong

```
~280 LOC moi. 5 sprints. ~3-5 sessions.
Thay the ~90 hardcoded patterns.
HomeOS tu chatbot → knowledge engine.
```

---

## Nguyen tac thiet ke

```
1. Gate truoc, tra loi sau
   - Phan loai input TRUOC KHI xu ly
   - Khong bao gio tra fact khi khong lien quan

2. Handcode == Zero
   - Khong them if-else cho tung case
   - Intelligence tu knowledge search + mol similarity

3. Honest
   - Biet thi noi (score >= HIGH)
   - Khong biet thi hoi lai (score < LOW)
   - Khong phu hop thi im

4. Compose tu data
   - Response = knowledge match + context + emotion
   - Khong phai template "Minh nghe roi" + fact random

5. Test-driven
   - Moi module test rieng
   - Integration test 30+ scenarios
```

---

## Dependencies

```
Olang 1.0 features used:
  split()      — tokenize input
  contains()   — pattern detection
  sort()       — ranking results
  map/filter() — transform arrays
  pipe()       — compose handlers
  save/load    — persistent knowledge
  __to_number()— math evaluation
  try/catch    — error handling

Khong can feature moi. Olang 1.0 du.
```
