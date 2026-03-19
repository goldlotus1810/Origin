# PLAN 2.3 — Knowledge Layer bằng Olang (~650 LOC)

**Phụ thuộc:** PLAN_2_1 (iter.ol, sort.ol, hash.ol, mol.ol, chain.ol)
**Mục tiêu:** Port knowledge ops (Silk, Dream, Instinct, Learning) sang Olang
**Tham chiếu:** `crates/silk/`, `crates/memory/`, `crates/agents/instinct.rs`

---

## Files cần viết

| File | LOC | Port từ Rust | Mô tả |
|------|-----|-------------|-------|
| `silk_ops.ol` | ~150 | `silk/graph.rs` | Implicit Silk (5D comparison), Hebbian update, walk |
| `dream.ol` | ~200 | `memory/dream.rs` | Clustering, scoring, propose promote |
| `instinct.ol` | ~200 | `agents/instinct.rs` | 7 bản năng (Honesty→Reflection) |
| `learning.ol` | ~100 | `agents/learning.rs` | Pipeline orchestration: encode→STM→Silk |

---

## silk_ops.ol — Implicit Silk

```
// Silk = implicit relationships from 5D distance
// 37 channels × 31 compound patterns = 1147 relationship types

fn implicit_silk(mol_a, mol_b) {
  // 5D distance → relationship strength
  let d = distance_5d(mol_a, mol_b);
  if d < 0.1 { return 1.0; }       // very similar
  if d > 0.8 { return 0.0; }       // unrelated
  return 1.0 - d;                    // linear falloff
}

fn hebbian_update(weight, co_active) {
  // Fire together → wire together
  // weight += lr * (co_active - weight * decay)
  let lr = 0.01;
  let decay = 0.618;  // φ⁻¹
  if co_active {
    return clamp(weight + lr * (1.0 - weight * decay), 0.0, 1.0);
  } else {
    return weight * decay;
  }
}

fn walk_weighted(graph, start_hash, depth) {
  // BFS/DFS walk, accumulate emotion via Silk weights
  // Returns composite EmotionTag (amplified, not averaged)
}

fn co_activate(graph, hash_a, hash_b, emotion) {
  // Record co-activation between two concepts
  // Silk edge gets EmotionTag of the moment
}
```

---

## dream.ol — Offline Consolidation

```
fn dream_cycle(stm_entries, silk_graph) {
  // STM → cluster → score → propose promote
  // 1. Cluster by 5D similarity
  // 2. Score clusters: α=0.3(cohesion) + β=0.4(fire_count) + γ=0.3(connectivity)
  // 3. Top clusters → propose QR promotion

  let clusters = cluster_5d(stm_entries);
  let scored = map(clusters, fn(c) {
    return { cluster: c, score: dream_score(c, silk_graph) };
  });
  let sorted = sort_by(scored, fn(s) { return s.score; });

  // Top 3 → propose
  return take(sorted, 3);
}

fn cluster_5d(entries) {
  // Simple: pairwise distance, merge if < threshold
  // Threshold = Fib-based (φ⁻¹ ≈ 0.382)
}

fn dream_score(cluster, graph) {
  let cohesion = avg_pairwise_similarity(cluster);
  let fire = avg_fire_count(cluster);
  let connectivity = avg_silk_degree(cluster, graph);
  return 0.3 * cohesion + 0.4 * fire + 0.3 * connectivity;
}
```

---

## instinct.ol — 7 Bản năng bẩm sinh

```
// Thứ tự ưu tiên: Honesty → Contradiction → Causality → Abstraction →
//                  Analogy → Curiosity → Reflection

fn run_instincts(observation, knowledge) {
  // ① Honesty: confidence < 0.40 → Silence (BlackCurtain)
  let confidence = assess_confidence(observation, knowledge);
  if confidence < 0.40 { return { action: "silence", reason: "insufficient evidence" }; }

  // ② Contradiction: valence opposition → flag
  let contradiction = detect_contradiction(observation, knowledge);
  if contradiction { return { action: "flag", reason: "contradiction detected" }; }

  // ③ Causality: temporal + co-activation → causal link
  let causal = detect_causality(observation, knowledge);

  // ④ Abstraction: N chains → LCA → categorical
  let abstraction = abstract_concept(observation, knowledge);

  // ⑤ Analogy: A:B :: C:? → delta 5D
  let analogy = find_analogy(observation, knowledge);

  // ⑥ Curiosity: 1 - nearest_similarity → novelty score
  let novelty = 1.0 - nearest_similarity(observation, knowledge);

  // ⑦ Reflection: qr_ratio + connectivity → knowledge quality
  let quality = reflect_quality(knowledge);

  return {
    action: "process",
    confidence: confidence,
    causal: causal,
    abstraction: abstraction,
    analogy: analogy,
    novelty: novelty,
    quality: quality,
  };
}
```

---

## learning.ol — Pipeline Orchestration

```
fn process_one(text, emotion, context) {
  // Gate → Encode → Instinct → STM → Silk → Curve

  // 1. SecurityGate check
  let gate_result = gate_check(text);
  if gate_result == "block" { return err("blocked by gate"); }

  // 2. Encode text → MolecularChain
  let chain = encode_text(text);

  // 3. Run instincts
  let instinct_result = run_instincts(chain, context.knowledge);
  if instinct_result.action == "silence" { return ok("silence"); }

  // 4. Push to STM
  stm_push(context.stm, chain, emotion);

  // 5. Co-activate Silk
  co_activate_words(context.silk, text, emotion);

  // 6. Update ConversationCurve
  curve_push(context.curve, emotion);

  return ok(instinct_result);
}
```

---

## Rào cản

| Rào cản | Giải pháp |
|---------|-----------|
| STM/Silk data structure lớn | In-memory arrays, capped size |
| Dream clustering O(N²) | Cap N at 1000, simple pairwise |
| Instinct cần knowledge base | Start with STM-only, no QR |
| Word encoding cần UCD | Builtin __encode_word, host provides |

---

## Definition of Done

- [ ] `silk_ops.ol`: implicit_silk, hebbian_update, co_activate — 3 tests
- [ ] `dream.ol`: cluster_5d, dream_score, dream_cycle — 3 tests
- [ ] `instinct.ol`: 7 instincts run in order, honesty blocks correctly — 3 tests
- [ ] `learning.ol`: process_one end-to-end — 2 tests

## Ước tính: 2 ngày
