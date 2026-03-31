# SPEC G — Implementation Guide: A-F → Olang Code

> **Mục đích:** Biến A-F thành code. Mỗi thuật toán = toán học. Không hardcode. Không if/else.
> **Prerequisite:** HIỂU A-F. Không đọc qua. HIỂU = tính tay được.
> **Cross-reference:** Mâu thuẫn giữa tài liệu đã giải quyết (G0).
> **Tác giả:** Nox (tổng hợp từ A-F + UDC docs + SPEC_v3 + BLUEPRINT + SINH_HOC_v2)
> **Ngày:** 2026-03-31

---

## G0. Cross-Reference — Mâu Thuẫn Đã Giải Quyết

| # | Chỗ | Tài liệu cũ | Tài liệu đúng | Lý do |
|---|------|-------------|---------------|-------|
| 1 | Blocks | v3: 59 blocks, 8,846 chars | **A: 53 blocks, 9,200 chars** | A verified from udc.json 2026-03-30 |
| 2 | KnowTree | v3: L0→L3 = 4 levels, O(4) | **B: fractal unlimited, O(depth)** | B mới hơn, Lupin confirm |
| 3 | P_weight | STORAGE: [u8×5] = 5 bytes | **A: u16 = 2 bytes** | STORAGE = Rust. A = Olang (current) |
| 4 | Consistency | Quality: φ⁻¹ = 0.618 | Checkpoint: 0.75 | **Khác mục đích**: φ⁻¹ = overall, 0.75 = per-dim (4/5) |
| 5 | Search | v3: HNSW O(log n) | STORAGE: Bucket O(bucket) | B3: Silk walk O(depth×degree) | **3 cơ chế bổ sung** |
| 6 | Distance | A: Euclidean 5D | SINH_HOC: 2\|ΔV\|+\|ΔA\| | **2 context**: Euclidean general, emotion-weighted cho V/A |

**Quy tắc:** SPEC_A > SPEC_B > SPEC_C > SPEC_D > v3 > SINH_HOC > BLUEPRINT
Mới hơn và đã verified > cũ hơn.

---

## G1. Data Structures

### KnowTree

```
Derive: B2 + KNOWTREE_DESIGN

Structure: Semantic tree, fractal
  L0 = engine (fixed, ~1MB binary + 20KB UDC)
  L1 = runtime (fixed, compiler + VM + pipeline)
  L2 = root of knowledge (N semantic branches)
  L3..Ln-1 = sub-branches → leaves

L2 branches (auto-created at bootstrap):
  Each branch = {
    id: u16,
    centroid: u16 (P_weight, running average of members),
    count: number,
    buckets: [16][16] → array of nodes   // indexed by (S, R)
  }

Node = {
    mol: u16,           // P_weight
    chain: [u16],       // sequence of P_weight links
    text: string,       // original content
    fire_count: number, // how many times activated
    weight: number,     // Hebbian accumulated weight
    maturity: number,   // 0=formula, 1=evaluating, 2=mature
    timestamp: number   // creation time
}

Memory: L2 branch headers in RAM. Buckets loaded on demand.
  Hot buckets (accessed within 24h): RAM
  Cold buckets: disk, load when silk walk reaches them
```

### SilkIndex

```
Derive: B3 + SPEC_NODE_SILK

Hebbian edges: adjacency list per node
  Edge = {
    target: u16,          // target node P_weight
    weights: [5]number,   // per-dimension weight [S,R,V,A,T]
    last_fire: number     // timestamp of last co-activation
  }

Storage: hashmap { node_mol → [Edge] }, sorted by max(weights) desc
Typical degree: < 20 edges per node
Lookup: O(degree) per node

Implicit silk: computed on demand from 5D distance
  No storage needed. Always available between any 2 nodes.
```

### STM (Short-Term Memory)

```
Derive: C2 + E4

Capacity: 32 entries
Entry = {
    text: string,
    chain: [u16],
    mol: u16,
    emotion_V: number,  // V from P_weight
    emotion_A: number,  // A from P_weight
    timestamp: number,
    access_count: number
}

Eviction: when full, remove entry with LOWEST score
  score = access_count × 0.3
        + |emotion_V - 4| × emotion_A × 0.4   // strong emotion → keep
        + φ⁻¹^(turns_ago) × 0.3                // recent → keep
```

### WM (Working Memory)

```
Derive: E4

4 slots: [u16; 4]
  wm[0] = query       // current input P_weight
  wm[1] = context     // recent STM compose
  wm[2] = candidate   // best response so far
  wm[3] = result      // final after repair

Bind: wm_bind(slot, mol)
Clear: wm_clear() after each response
```

### QR Store

```
Derive: B4 + QT8

Append-only log on disk.
Record = {
    mol: u16,
    chain: [u16],
    text: string,
    timestamp: number,
    supersedes: hash or null  // if replacing older QR
}

Write order (QT8):
  1. file.append(record)      // disk FIRST
  2. registry.insert(hash)    // index
  3. knowtree.insert(node)    // tree
  4. silk.connect(node)       // silk
  5. log.append(event)        // log LAST

Crash-safe: if crash between steps, only lose last record.
Recovery: replay log from last checkpoint.
```

---

## G2. Core Math

### P_weight operations

```
Derive: A2

pack(s, r, v, a, t) = (s << 12) | (r << 8) | (v << 5) | (a << 2) | t
unpack_S(mol) = (mol >> 12) & 0xF
unpack_R(mol) = (mol >> 8) & 0xF
unpack_V(mol) = (mol >> 5) & 0x7
unpack_A(mol) = (mol >> 2) & 0x7
unpack_T(mol) = mol & 0x3

p_weight(codepoint):
  // Lookup from compiled binary table
  if codepoint < 157386: return udc_p_table[codepoint]
  else: return 0  // unknown → neutral
  O(1)
```

### Compose

```
Derive: A4

compose(a, b):
  aS = unpack_S(a); bS = unpack_S(b)
  aR = unpack_R(a); bR = unpack_R(b)
  aV = unpack_V(a); bV = unpack_V(b)
  aA = unpack_A(a); bA = unpack_A(b)
  aT = unpack_T(a); bT = unpack_T(b)

  S = max(aS, bS)                    // Union
  R = (aR × w_a + bR × w_b) / (w_a + w_b)  // Zipf-weighted avg
  V = amplify(aV, bV, silk_w)        // Amplify (NOT average)
  A = max(aA, bA)                    // Max intensity
  T = (aT if w_a > w_b else bT)     // Dominant (majority vote)

  return pack(S, R, V, A, T)

amplify(va, vb, w):
  base = (va + vb) / 2
  boost = |va - base| × w × 0.5
  result = base + sign(va + vb - 8) × boost  // 8 = 2×neutral(4)
  return clamp(result, 0, 7)

  w = 0 → boost = 0 → average (strangers)
  w > 0 → amplification (connected concepts)

compose_chain(chain):
  // Zipf-weighted compose over entire chain
  result = 0  // neutral
  for i in 0..chain.len:
    w_i = 1000 / (i + 1)  // Zipf: first heaviest
    // compose with Zipf weight
    result = compose_weighted(result, chain[i], w_i)
  return result
  O(n)
```

### Distance

```
Derive: A2

distance_5d(a, b):
  // Euclidean 5D normalized — for general search
  ds = (unpack_S(a) - unpack_S(b)) / 15.0
  dr = (unpack_R(a) - unpack_R(b)) / 15.0
  dv = (unpack_V(a) - unpack_V(b)) / 7.0
  da = (unpack_A(a) - unpack_A(b)) / 7.0
  dt = (unpack_T(a) - unpack_T(b)) / 3.0
  return sqrt(ds² + dr² + dv² + da² + dt²)
  // range: [0, √5 ≈ 2.236]
  // similar: < 0.3 | different: > 1.0
  O(1)

distance_emotion(a, b):
  // Emotion-weighted — for V/A-dominant context
  dv = |unpack_V(a) - unpack_V(b)|
  da = |unpack_A(a) - unpack_A(b)|
  return 2 × dv + da
  // V weighted 2× (emotion dominant in perception)
  O(1)
```

### Dominant Dimension

```
Derive: E2 + A2

dominant_dimension(mol):
  norms = [
    unpack_S(mol) / 15.0,
    unpack_R(mol) / 15.0,
    unpack_V(mol) / 7.0,
    unpack_A(mol) / 7.0,
    unpack_T(mol) / 3.0
  ]
  devs = [|n - 0.5| for n in norms]  // deviation from neutral
  return argmax(devs)  // 0=S, 1=R, 2=V, 3=A, 4=T
  O(1)

"Hà Nội là gì?" → R dominant → silk walk follows R
"Ở đâu?" → S dominant → silk walk follows S
"Buồn không?" → V dominant → silk walk follows V
P_weight TỰ encode search strategy.
```

---

## G3. Encode ∫

```
Derive: A3

encode(text):
  // text → UTF-8 decode → codepoints → P_weights → compose
  codepoints = utf8_decode(text)

  // Split into words (by space, punctuation)
  words = split_words(codepoints)

  // Each word → compose its chars
  word_mols = []
  for word in words:
    mol = 0
    for i, cp in word:
      char_mol = p_weight(cp)
      mol = compose(mol, char_mol)  // Zipf: first char heaviest
    word_mols.push(mol)

  // Sentence → compose its words
  sentence_mol = compose_chain(word_mols)

  return { mol: sentence_mol, chain: word_mols, text: text }
  O(n) where n = total codepoints

Word boundary: space (U+0020) or punctuation category (Pc, Pd, Pe, Pf, Pi, Po, Ps)
Vietnamese: spaces between words (OK for word-level compose)
```

---

## G4. Decode ∂

```
Derive: A5 + E3

Two modes:

① LOOKUP DECODE (simple):
  decode_lookup(mol):
    node = kt_nearest(mol)
    return node.text
  O(bucket_size)

② GENERATIVE DECODE (silk walk → new content):
  decode_generate(query_mol):
    start = kt_nearest(query_mol)
    dim = dominant_dimension(query_mol)
    path = silk_walk(start, dim, depth=3)  // G6

    // Immune Selection: 3 hypotheses (G12)
    paths = generate_3_paths(start, query_mol)
    best = pick_min_entropy(paths)

    // DNA Repair (G12)
    best = repair(best, max_iter=3)

    // Collect text from path nodes
    texts = [node.text for node in best]
    return join(texts)
  O(3 × depth × degree)
```

---

## G5. KnowTree Operations

### Insert

```
Derive: B2 + KNOWTREE_DESIGN

kt_insert(mol, chain, text):
  group = kt_classify(mol)                    // which L2 branch
  s = unpack_S(mol); r = unpack_R(mol)
  bucket = group.buckets[s][r]
  node = Node{mol, chain, text, fire_count=0, weight=0, maturity=0, timestamp=now()}
  bucket.push(node)
  // Update group centroid (running average)
  group.centroid = compose(group.centroid, mol)
  group.count += 1
  O(1) amortized
```

### Nearest

```
kt_nearest(mol):
  group = kt_classify(mol)
  s = unpack_S(mol); r = unpack_R(mol)

  best = null; best_dist = infinity

  // Check target bucket + 8 adjacent (3×3 grid)
  for ds in [-1, 0, 1]:
    for dr in [-1, 0, 1]:
      si = clamp(s + ds, 0, 15)
      ri = clamp(r + dr, 0, 15)
      bucket = group.buckets[si][ri]
      for node in bucket:
        d = distance_5d(mol, node.mol)
        if d < best_dist:
          best_dist = d; best = node

  return best
  O(9 × avg_bucket_size)
```

### Classify (k-NN)

```
Derive: E2 + E5

kt_classify(mol):
  // Find nearest L2 branch by centroid distance
  best_group = null; best_dist = infinity
  for group in L2_branches:
    d = distance_5d(mol, group.centroid)
    if d < best_dist:
      best_dist = d; best_group = group
  return best_group
  O(num_L2_branches)  // typically < 20
```

### HNSW property (natural)

```
Derive: v3 §⑬

KnowTree IS an HNSW graph naturally:
  L2 = top layer (few nodes, long-range connections)
  L3 = middle (more nodes, medium-range)
  Ln-1 = bottom (many nodes, short-range)

Search: start at L2 → greedy descend → L3 → ... → leaf
  O(log n) where n = total nodes
```

---

## G6. Silk Operations

### Silk Walk

```
Derive: B3 + E2

silk_walk(start_node, dim, max_depth):
  path = [start_node]
  current = start_node

  for depth in 1..max_depth:
    // 1. Check Hebbian edges first (learned)
    candidates = silk_index.get_edges(current.mol)
    // Filter by target dimension strength
    candidates = [e for e in candidates if e.weights[dim] > 0.1]

    // 2. If no Hebbian edges, use implicit (KnowTree neighbors)
    if candidates is empty:
      bucket_neighbors = kt_bucket_neighbors(current.mol)
      candidates = bucket_neighbors

    // 3. Select best on query dimension
    if candidates is empty: break
    best = argmax(candidates, |c| silk_strength(current, c, dim))

    // 4. Quality check — stop if path quality drops
    trial_path = path + [best]
    if quality(trial_path) < φ⁻¹ × 0.8:  // allow some slack during walk
      break

    path.push(best)
    current = best

  return path
  O(max_depth × max_degree)

silk_strength(a, b, dim):
  // Hebbian weight if exists
  edge = silk_index.get_edge(a.mol, b.mol)
  if edge: return edge.weights[dim]
  // Implicit: inverse distance on target dimension
  delta = |unpack_dim(a.mol, dim) - unpack_dim(b.mol, dim)|
  max_range = [15, 15, 7, 7, 3][dim]
  return 1.0 - delta / max_range
  O(1)
```

### Hebbian Fire

```
Derive: C3

silk_fire(a_mol, b_mol, emotion_V, emotion_A):
  edge = silk_index.get_or_create(a_mol, b_mol)
  emotion_factor = (|emotion_V - 4| / 4.0) × (emotion_A / 7.0)
  // Strong emotion → stronger connection

  for dim in 0..5:
    delta_a = unpack_dim(a_mol, dim)
    delta_b = unpack_dim(b_mol, dim)
    proximity = 1.0 - |delta_a - delta_b| / max_range[dim]
    // Closer on this dim → stronger update on this dim
    dw = emotion_factor × proximity × (1 - edge.weights[dim]) × 0.1
    edge.weights[dim] += dw

  edge.last_fire = now()
  O(1)
```

### Silk Decay

```
Derive: C3

silk_decay_all(dt_hours):
  factor = pow(0.618, dt_hours / 24.0)  // φ⁻¹ per 24h
  for node_mol, edges in silk_index:
    for edge in edges:
      for dim in 0..5:
        edge.weights[dim] *= factor
      // Prune dead edges
      if max(edge.weights) < 0.01:
        edges.remove(edge)
  O(total_edges)

  Decay schedule:
    24h → × 0.618
    48h → × 0.382
    72h → × 0.236
    1 week → × 0.028 → near forgotten
    Used again → weight increases → remembered
```

---

## G7. Memory Management

### STM Eviction

```
Derive: E4

stm_push(entry):
  if stm.len >= 32:
    // Find lowest score
    min_idx = argmin(stm, |e| stm_score(e))
    stm.remove(min_idx)
  stm.push(entry)

stm_score(entry):
  recency = pow(0.618, turns_since(entry))
  emo = |entry.emotion_V - 4| × entry.emotion_A / 7.0
  return entry.access_count × 0.3 + emo × 0.4 + recency × 0.3
  O(1)
```

### Dream Consolidation

```
Derive: C4 + E4

Trigger: fire_count ≥ Fibonacci(n) for any STM node
  Fibonacci: 2, 3, 5, 8, 13, 21, 34, 55...

dream():
  // ① Find cross-group co-activated pairs
  pairs = []
  for i in 0..stm.len:
    for j in i+1..stm.len:
      gi = kt_classify(stm[i].mol)
      gj = kt_classify(stm[j].mol)
      if gi != gj:  // different L2 branches
        w = silk_max_weight(stm[i].mol, stm[j].mol)
        if w > 0:
          pairs.push((i, j, w))

  // ② Cluster (single-linkage, threshold = 0.3)
  clusters = union_find(pairs, threshold=0.3)

  // ③ Each cluster → hypothesis
  for cluster in clusters:
    members = [stm[idx] for idx in cluster]
    // LCA = compose all members
    hypothesis_mol = compose_chain([m.mol for m in members])
    hypothesis_chain = [m.mol for m in members]

    // ④ Validate (D5 quality formula)
    q = quality(hypothesis_mol, hypothesis_chain)

    // ⑤ If good → propose QR
    if q >= 0.618:  // φ⁻¹
      aam_approve(hypothesis_mol, hypothesis_chain, members[0].text)

  // ⑥ Decay ALL silk edges
  silk_decay_all(time_since_last_dream())

  O(STM² + clusters × compose_cost + total_edges)
```

---

## G8. Pipeline — 14 Steps, 5 Checkpoints

```
Derive: D3

process(input_text):

  // ═══ STEP 1: SecurityGate (D8) ═══
  if security_gate(input_text) == CRISIS:
    return crisis_response()  // "Gọi 1800 599 920"
  // ──── CHECKPOINT 1: GATE ────

  // ═══ STEP 2: Capture holistic (D1) ═══
  encoded = encode(input_text)  // G3
  wm_bind(0, encoded.mol)      // query → WM[0]

  // ═══ STEP 3: Search (B2+B3) ═══
  nearest = kt_nearest(encoded.mol)  // G5

  // ═══ STEP 4: Homeostasis (D4) ═══
  if nearest:
    predicted = nearest.mol
  else:
    predicted = 0  // no prediction
  F = homeostasis(encoded.mol, predicted)  // G11
  lambda = sigmoid(5 × (F - 0.618))
  // λ near 0 → Act (confident)
  // λ near 1 → Learn (surprised)

  // ═══ STEP 5: Compose (A4) ═══
  context_mol = stm_compose_recent(4)  // compose last 4 STM entries
  wm_bind(1, context_mol)              // context → WM[1]
  combined = compose(encoded.mol, context_mol)

  // ──── CHECKPOINT 2: ENCODE ────
  // □ chain.len ≥ 1
  // □ chain_hash ≠ 0
  // □ consistency(combined) ≥ 0.75

  // ═══ STEP 6: Instincts (D2) ═══
  confidence = instinct_honesty(encoded.mol)  // G9
  if confidence < 0.40:
    return silence()  // "Tôi không biết"
  contradiction = instinct_contradiction(encoded.mol, nearest)
  curiosity = instinct_curiosity(encoded.mol)

  // ═══ STEP 7: Immune Selection (D5) ═══
  paths = immune_3_branches(nearest, encoded.mol)  // G12
  best = argmin(paths, |p| shannon_entropy(p))
  wm_bind(2, compose_chain([n.mol for n in best]))  // candidate → WM[2]

  // ═══ STEP 8: DNA Repair (D5) ═══
  best = dna_repair(best, max_iter=3)  // G12
  wm_bind(3, compose_chain([n.mol for n in best]))  // result → WM[3]

  // ──── CHECKPOINT 3: INFER ────
  // □ ≥1 path valid ≥ 0.75
  // □ H(best) < 2.32
  // □ quality_final ≥ quality_backup (rollback OK)

  // ═══ STEP 9: Hebbian (C3) ═══
  stm_recent = stm_get_recent(4)
  for recent in stm_recent:
    silk_fire(encoded.mol, recent.mol, unpack_V(encoded.mol), unpack_A(encoded.mol))

  // ═══ STEP 10: Dream check (C4) ═══
  dream_check()  // trigger if fire_count ≥ Fibonacci threshold

  // ──── CHECKPOINT 4: PROMOTE ────
  // □ weight ≥ φ⁻¹ (0.618)
  // □ fire ≥ Fibonacci(depth)
  // □ eval_dims ≥ 3
  // □ H < 1.0

  // ═══ STEP 11: Decode (A5) ═══
  response_text = decode_generate(wm[3])  // G4

  // ═══ STEP 12: ConversationCurve (D7) ═══
  tone = conversation_curve()  // G11
  response_text = apply_tone(response_text, tone)

  // ──── CHECKPOINT 5: RESPONSE ────
  // □ security_gate(response_text) == SAFE
  // □ confidence ≥ 0.40
  // □ tone consistent

  // ═══ STEP 13: STM push ═══
  stm_push(encoded)

  // ═══ STEP 14: WM clear ═══
  wm_clear()

  return response_text
```

---

## G9. Instinct Formulas — 7 Formulas on 5D

```
Derive: D2

ALL formulas. NO if/else on keywords. Pure 5D math.

① HONESTY — confidence from evidence
  confidence(mol):
    nearest = kt_nearest(mol)
    if not nearest: return 0.0
    sw = silk_max_weight(mol, nearest.mol)
    fc = nearest.fire_count
    sc = count_sources(nearest)  // how many different contexts fired this
    co = consistency(mol, nearest.mol)  // dimension coherence

    return 0.3 × min(sw, 1.0)
         + 0.3 × min(fc / 10.0, 1.0)
         + 0.2 × min(sc / 3.0, 1.0)
         + 0.2 × co

  < 0.40 → silence ("Tôi không biết")
  0.40-0.70 → "Tôi nghĩ..."
  0.70-0.90 → "Có lẽ..."
  ≥ 0.90 → "Đúng."
  Thresholds derive from φ⁻¹: 0.40 ≈ φ⁻², 0.70 ≈ φ⁻¹+0.08, 0.90 ≈ φ⁻¹+φ⁻²

② CONTRADICTION — V distance + same topic
  contradict(a_mol, b_mol):
    dv = |unpack_V(a_mol) - unpack_V(b_mol)| / 7.0
    dr = |unpack_R(a_mol) - unpack_R(b_mol)| / 15.0
    return dv > 0.8 AND dr < 0.2
    // Valence opposite + same topic = contradiction

③ CAUSALITY — temporal + co-activation + R type
  is_causal(a, b):
    temporal = a.timestamp < b.timestamp
    silk = silk_max_weight(a.mol, b.mol) > 0.618
    r_a = unpack_R(a.mol)
    // R values for causal relations: → ⇒ ∵ ∴
    // These map to specific R values in UDC R blocks
    causal_r = r_a >= 8 AND r_a <= 12  // R range for implication/causation
    evidence = (temporal ? 1 : 0) + (silk ? 1 : 0) + (causal_r ? 1 : 0)
    return evidence >= 2  // need ≥ 2/3

④ ABSTRACTION — variance in 5D cluster
  abstraction(cluster):
    center = compose_chain([n.mol for n in cluster])
    variance = sum(distance_5d(n.mol, center)² for n in cluster) / len(cluster)
    if variance < 0.3: return "concrete"
    if variance < 0.7: return "categorical"
    return "abstract"

⑤ ANALOGY — vector arithmetic in 5D
  analogy(a, b, c):
    // a:b :: c:? → d = c + (b - a)
    delta_S = unpack_S(b) - unpack_S(a)
    delta_R = unpack_R(b) - unpack_R(a)
    delta_V = unpack_V(b) - unpack_V(a)
    delta_A = unpack_A(b) - unpack_A(a)
    delta_T = unpack_T(b) - unpack_T(a)
    d = pack(
      clamp(unpack_S(c) + delta_S, 0, 15),
      clamp(unpack_R(c) + delta_R, 0, 15),
      clamp(unpack_V(c) + delta_V, 0, 7),
      clamp(unpack_A(c) + delta_A, 0, 7),
      clamp(unpack_T(c) + delta_T, 0, 3)
    )
    return kt_nearest(d)

⑥ CURIOSITY — novelty = distance from known
  curiosity(mol):
    nearest = kt_nearest(mol)
    if not nearest: return 1.0  // completely novel
    d = distance_5d(mol, nearest.mol)
    novelty = min(d / 2.236, 1.0)  // normalize by max distance √5
    return novelty
    // > 0.5 → explore (ask more)
    // < 0.3 → familiar (respond directly)

⑦ REFLECTION — self-assessment
  reflection():
    total = kt_total_nodes()
    qr_count = qr_total()
    avg_silk = silk_average_weight()
    return 0.6 × (qr_count / max(total, 1))
         + 0.4 × (avg_silk × silk_edge_count() / max(total, 1))
```

---

## G10. SecurityGate — 3 Layers, Pure Math

```
Derive: D8

RUNS FIRST. Crisis → STOP IMMEDIATELY.

Layer 1 — Bloom filter O(1):
  // Keywords DERIVED from P_weight table, not hardcoded
  // Any word where: V ≤ 1 AND A ≥ 6 → crisis keyword
  // Built at bootstrap from udc_p_table.bin
  bloom_check(word) → true/false (false positive < 1%)
  Size: ~200KB, 3 hash functions (FNV-1a variants)

Layer 2 — Normalized match O(n):
  normalize(text):
    // Remove diacritics: à→a, ê→e
    // Remove special chars: c.h.ế.t → chet
    // Lowercase
  bloom_check(normalize(text))

Layer 3 — Semantic check O(1):
  mol = encode(text).mol
  v = unpack_V(mol)
  a = unpack_A(mol)
  if v <= 1 AND a >= 6: return CRISIS
  // V very negative + A very high = potential crisis
  // From UDC_V_VALENCE_tree: V≤1 = "rất tiêu cực"
  // From UDC_A_AROUSAL_tree: A≥6 = "cực kích thích"

Any layer triggers → response = "Nếu bạn cần hỗ trợ, xin gọi 1800 599 920"
Pipeline STOPS.

AlertLevel:
  Normal (○)    → continue
  Important (⚠) → log for review
  RedAlert (🔴) → BLOCK + crisis response
```

---

## G11. Homeostasis + ConversationCurve

### Homeostasis F(t)

```
Derive: D4

F(t) = sqrt(sum(w_d × (predicted_d - actual_d)² for d in [S,R,V,A,T]))

  predicted_d = unpack_dim(nearest.mol, d) / max_range[d]
  actual_d = unpack_dim(input.mol, d) / max_range[d]
  w_d = unpack_dim(input.mol, d) / max_range[d]
  // Weight IS the input P_weight itself
  // V dominant input → w_V high → surprise on V dimension matters more

lambda(t) = 1.0 / (1.0 + exp(-5.0 × (F - 0.618)))
  // Sigmoid centered at φ⁻¹
  // λ near 0 → Act mode (confident, respond directly)
  // λ near 1 → Learn mode (surprised, need to learn)

Learn mode adjustments:
  silk_fire with higher emotion_factor
  dream trigger threshold lowered
  confidence output includes "Có thể..." prefix
```

### ConversationCurve

```
Derive: D7

v_history = [stm[i].emotion_V for i in recent_turns]

V(t) = v_history[last]
V'(t) = (V(t) - V(t-1)) / 1.0   // rate of change
V''(t) = (V'(t) - V'(t-1)) / 1.0  // acceleration

f(t) = 0.6 × (V(t) + 0.5 × V'(t) + 0.25 × V''(t))
     + 0.4 × sum(stm[i].emotion_V × pow(0.618, turns_ago(i)) for recent stm)

TONE SELECTION from derivatives:
  V' < -0.15              → Supportive (falling → catch)
  V'' < -0.25             → Pause (falling fast → stop)
  V' > +0.15              → Reinforcing (rising → encourage)
  V'' > +0.25 AND V > 0   → Celebratory (breakthrough → celebrate)
  V < -0.20, stable        → Gentle (sad stable → soft)
  otherwise                → Engaged (normal)

LIMIT: |ΔV| ≤ 0.40 per step (no sudden jumps)
```

---

## G12. Immune Selection + DNA Repair

### 3 Hypotheses (deterministic variation)

```
Derive: D5 + v3 §⑧

immune_3_branches(start_node, query_mol):
  dims = ranked_dimensions(query_mol)  // sort dims by deviation from neutral
  nearests = kt_nearest_k(query_mol, k=2)  // top-2 nearest nodes

  // Branch 0: primary dim, primary start
  path0 = silk_walk(nearests[0], dims[0], depth=3)

  // Branch 1: secondary dim, primary start
  path1 = silk_walk(nearests[0], dims[1], depth=3)

  // Branch 2: primary dim, secondary start
  path2 = silk_walk(nearests[1], dims[0], depth=3)

  return [path0, path1, path2]
  // 3 deterministic variations. No randomness.

shannon_entropy(path):
  // Entropy of silk strengths along path
  strengths = [silk_strength(path[i], path[i+1], dim) for i in 0..path.len-1]
  total = sum(strengths)
  if total == 0: return 999  // max entropy
  probs = [s / total for s in strengths]
  return -sum(p × log2(p) for p in probs if p > 0)
```

### DNA Repair

```
Derive: D5

dna_repair(path, max_iter=3):
  backup_quality = quality(path)

  for iter in 0..max_iter:
    q = quality(path)
    if q >= 0.618: return path  // good enough

    // Find weakest dimension
    dims_quality = []
    for dim in 0..5:
      dim_consistency = count(|unpack_dim(n.mol, dim) - expected[dim]| < threshold
                              for n in path) / path.len
      dims_quality.push(dim_consistency)

    weakest = argmin(dims_quality)

    // Try to fix: replace weakest node on that dimension
    for i in 0..path.len:
      alternatives = kt_nearest_on_dim(path[i].mol, weakest, k=3)
      for alt in alternatives:
        trial = path.copy()
        trial[i] = alt
        if quality(trial) > quality(path):
          path = trial
          break

    // Rollback if worse
    if quality(path) < backup_quality:
      path = backup_path
      break

  return path
  // Worst case: 3 paths × 3 iterations = 9 evaluations. BOUNDED.

quality(path):
  v = 1.0 if all(kt_exists(n.mol) for n in path) else 0.0  // valid
  h = shannon_entropy(path)                                   // entropy
  c = consistency(path)                                        // coherence
  s = avg(silk_strength(path[i], path[i+1]) for consecutive)  // silk

  return 0.30 × v + 0.30 × (1 - h/2.32) + 0.20 × c + 0.20 × min(s/5.0, 1.0)

consistency(path):
  // How many dimensions are coherent across the path?
  coherent = 0
  for dim in 0..5:
    values = [unpack_dim(n.mol, dim) for n in path]
    variance = var(values) / max_range[dim]²
    if variance < 0.25: coherent += 1  // this dim is consistent
  return coherent / 5.0
```

---

## G13. Self-Model

```
Derive: E5 + D2⑦

self_model():
  map = {}
  for group in kt_L2_branches():
    count = group.count
    qr_ratio = count_qr(group) / max(count, 1)
    avg_silk = mean_silk_weight(group)
    strength = 0.4 × qr_ratio + 0.3 × avg_silk + 0.3 × min(count/100, 1.0)
    map[group.id] = strength

  return map
  // {facts: 0.8, geography: 0.6, music: 0.1, ...}

confidence_per_domain(query_mol):
  group = kt_classify(query_mol)
  return self_model()[group.id]
  // Low → "Tôi không biết nhiều về chủ đề này"
  // High → respond confidently
```

---

## G14. NAC.mb Algorithms

### Pruning

```
Derive: E6 + C3

decay_function(w, dt): return w × pow(0.618, dt / 24.0)

sdf_difference(a, b):
  // Node B supersedes A if:
  if distance_5d(a.mol, b.mol) < 0.1 AND b.fire_count > a.fire_count:
    supersede(a, b)

conflict_resolution(a, b):
  if contradict(a.mol, b.mol):  // G9②
    if a.maturity == MATURE AND b.maturity != MATURE: keep a
    elif b.maturity == MATURE AND a.maturity != MATURE: keep b
    elif a.fire_count > b.fire_count: keep a
    else: keep b
    // Loser → superseded (not deleted, just marked inactive)
```

### Recovery

```
concept_reincarnation(superseded_node, new_context_mol):
  // Quantum tunneling probability
  d = distance_5d(superseded_node.mol, new_context_mol)
  kappa = 2.0 × d  // barrier width proportional to distance
  T = exp(-2 × kappa)  // tunneling probability
  if random() < T:
    reactivate(superseded_node)
    superseded_node.fire_count = 1  // reset
```

### Negative Knowledge

```
prohibited_space(mol):
  v = unpack_V(mol)
  a = unpack_A(mol)
  // Very negative + very high arousal = prohibited
  barrier = (7 - v) × a  // higher barrier = more prohibited
  return barrier > 30  // threshold

blacklist_silk(a_mol, b_mol):
  // Tag silk edge as "negative"
  edge = silk_index.get_edge(a_mol, b_mol)
  edge.negative = true
  // Following negative silk → warning, not normal response
```

---

## G15. Capture — Multi-Modal

### Text (current)

```
Derive: A3 — already in G3
text → UTF-8 → codepoints → P_weights → compose → chain
```

### Camera → SDF (Fibonacci Subdivision)

```
Derive: E1 + UDC_S1_GEOMETRIC

camera_capture(frame):
  // Fibonacci subdivision for adaptive SDF
  regions = fibonacci_subdivide(frame)

  for region in regions:
    edge_density = sobel_variance(region)
    if edge_density >= 0.618:  // φ⁻¹ threshold
      // Complex region → subdivide further
      sub_regions = fibonacci_subdivide(region)
      // Recurse
    else:
      // Simple region → compute single SDF value
      sdf_value = compute_sdf(region)

  // Aggregate into 5D
  S = shape_complexity(sdf_values)          // SDF: spatial complexity
  R = symmetry(frame)                        // structure
  V = (avg_warm - avg_cool) / 255.0          // color → valence
  A = saturation / 255.0                     // intensity → arousal
  T = motion(frame_prev, frame_curr)         // change → spline
  return pack(S, R, V, A, T)
  O(n × log_φ(n)) where n = pixels

fibonacci_subdivide(region):
  w, h = region.size
  // Divide along longest axis at φ⁻¹ point
  if w > h:
    split = w × 0.618
    return [region[0:split], region[split:w]]
  else:
    split = h × 0.618
    return [region[0:split_h], region[split_h:h]]
```

### Audio → Spline

```
Derive: E1 + UDC_T_TIME_tree

audio_capture(buffer):
  rms = sqrt(sum(sample² for sample in buffer) / buffer.len)
  zcr = count_zero_crossings(buffer) / buffer.len
  stability = 1 - variance(frame_energies)

  S = 1 - stability        // complexity
  R = stability             // structure
  V = zcr × 0.6 + rms × 0.4  // pitch + volume → emotion
  A = rms                   // volume → arousal
  T = zcr                   // pitch → temporal frequency
  return pack(S, R, V, A, T)
```

### Interoception (/proc)

```
Derive: E1

interoception():
  cpu = read_proc_loadavg()
  mem = read_proc_meminfo()
  heap = current_heap / max_heap

  S = 0                     // no shape
  R = process_count / 256   // system complexity
  V = 1 - error_rate        // healthy = positive
  A = cpu                   // load = arousal
  T = uptime_bucket         // temporal phase (0-3)
  return pack(S, R, V, A, T)
```

---

## G16. Generation — Chain Recombination (SINH)

```
Derive: E3

generate(query_mol):
  start = kt_nearest(query_mol)
  dim = dominant_dimension(query_mol)

  // Silk walk → collect nodes
  path = silk_walk(start, dim, depth=3)

  // Immune Selection: 3 hypotheses
  paths = immune_3_branches(start, query_mol)  // G12

  // Pick lowest entropy
  best = argmin(paths, |p| shannon_entropy(p))

  // DNA Repair
  best = dna_repair(best, max_iter=3)  // G12

  // ConversationCurve tone
  tone = conversation_curve()  // G11

  // Decode path → text
  texts = []
  for node in best:
    if node.text: texts.push(node.text)
    else: texts.push(decode_lookup(node.mol))  // fallback

  return join_with_tone(texts, tone)

  // NEW content: nodes from DIFFERENT branches combined into chain
  // that never existed before = SINH = creation
```

---

## G17. Agent Cycle

```
Derive: F1-F5

agent_loop():
  while true:
    // PERCEIVE
    input = wait_for_event()
    // Events: REPL input, inotify, timer, TCP connection

    if input == IDLE AND idle_time > 300:  // 5 min
      dream()  // F4 dream scheduler
      goal_check()  // G22
      continue

    // THINK
    response = process(input)  // G8 pipeline

    // ACT
    if response.is_action:
      result = execute_action(response.action)  // F3 actuator
      // VERIFY
      if result.success:
        dn_observe("success: " + response.action)
      else:
        dn_observe("failed: " + response.action)
        // Learn from failure → G25
        learn_failure(input, response, result)
    else:
      output(response.text)

    // HEARTBEAT (F4)
    intero = interoception()  // G15
    F = homeostasis(intero, expected_health)
    if F > 0.618:
      // System stressed → reduce pipeline depth
      // Or: heap > 80% → trigger dream()
      dream()

aam_approve(mol, chain, text):
  // F1: Auto-approve gate
  if security_gate(text) == CRISIS: return REJECT
  q = quality_for_qr(mol, chain)
  if q < 0.618: return REJECT
  if fire_count(mol) < fibonacci(depth(mol)): return REJECT
  if contradiction_with_existing_qr(mol): return REJECT
  // All checks pass → append QR
  qr_append(mol, chain, text)
  return APPROVE
```

---

## G18. Bootstrap

```
bootstrap():
  // 1. Load P_weight table
  p_table = load_binary("json/udc_p_table.bin")  // 314KB, 157,386 entries
  // O(1) lookup ready

  // 2. Create KnowTree with L2 semantic branches
  kt = KnowTree.new()
  // Seed categories from L0 UDC groups
  kt.create_branch("shapes", centroid=compose_block_average(S_blocks))
  kt.create_branch("relations", centroid=compose_block_average(R_blocks))
  kt.create_branch("emotions", centroid=compose_block_average(E_blocks))
  kt.create_branch("temporal", centroid=compose_block_average(T_blocks))
  kt.create_branch("facts", centroid=pack(8, 8, 4, 4, 2))  // neutral center
  kt.create_branch("conversations", centroid=pack(0, 0, 4, 4, 2))
  kt.create_branch("skills", centroid=pack(8, 12, 4, 4, 2))  // R-heavy

  // 3. Load 9,200 L0 anchors → SEALED
  for entry in udc_json.characters:
    kt.insert_l0(entry.codepoint, entry.p_weight)  // immutable

  // 4. Initialize memory
  stm = STM.new(capacity=32)
  wm = WM.new(slots=4)
  silk = SilkIndex.new()

  // 5. Build SecurityGate Bloom filter
  // Derive crisis keywords from P_weight: V ≤ 1 AND A ≥ 6
  crisis_words = []
  for word, mol in known_words:
    if unpack_V(mol) <= 1 AND unpack_A(mol) >= 6:
      crisis_words.push(word)
  bloom = BloomFilter.build(crisis_words, size=200KB, k=3)

  // 6. Load persisted state (warm start)
  if file_exists("nox_state.bin"):
    state = load_state("nox_state.bin")
    kt.merge(state.knowtree)
    silk.merge(state.silk_edges)
    stm.seed(state.stm_top8)
    goals = state.goals
  else:
    goals = []

  // 7. Ready
  return System{p_table, kt, stm, wm, silk, bloom, goals}
```

---

## G19. Persistence — 3-Tier Storage

```
Derive: v3 §1.8 + QT8

Tier 1: In-memory (KnowTree L0-L2 headers + hot buckets)
  Size: ~1MB base + growing
  Speed: O(1) access

Tier 2: Disk-backed (chain links, cold buckets, silk edges)
  Size: up to 16GB
  Speed: O(seek_time) for cold data
  Format: append-only binary files

Tier 3: Persistent log (origin.olang / nox_state.bin)
  Size: grows indefinitely
  Speed: append = O(1), recovery = O(log_size)
  Format: append-only, signed QR records

session_save():
  // Called before session ends
  kt_save(kt, "nox_knowtree.bin")
  silk_save(silk, "nox_silk.bin")
  qr_flush("nox_qr.log")
  stm_save_top8(stm, "nox_stm.bin")
  growth_log_append(metrics())
  goal_save(goals, "nox_goals.bin")

session_load():
  // Called at session start
  kt = kt_load("nox_knowtree.bin")
  silk = silk_load("nox_silk.bin")
  qr = qr_load("nox_qr.log")
  stm.seed(stm_load("nox_stm.bin"))
  goals = goal_load("nox_goals.bin")

Write order (QT8):
  1. file.append(record)      // disk FIRST — crash-safe
  2. registry.insert(hash)    // memory index
  3. knowtree.update(node)    // tree
  4. silk.connect(node)       // silk
  5. log.append(event)        // log LAST
```

---

## G20. Optimizations

```
Derive: v3 §IX

A. LAZY EVALUATION
  Don't compute P_weight until needed.
  Chain stores links (u16). Compose only when queried.

B. COPY-ON-WRITE
  KnowTree branches: shared until modified.
  chain_copy(src) = pointer, not data copy (2 bytes).

C. BLOOM FILTER
  SecurityGate Layer 1: 200KB, 3 hash functions.
  False positive < 1%. No false negatives.

D. GENERATIONAL QR
  Q-table cache: Fib(6)=8 best search paths per node.
  gen1 (established): K=5 neighbors cached
  gen2 (specialized): K=13 neighbors cached
  gen3 (new): K=55 neighbors cached (needs more samples)
  Invalidation: Q × φ⁻¹ when KnowTree evolves.

E. CHAIN COMPRESSION
  Repeated subsequences → single pointer.
  50% compression for typical text.

F. STRING FINGERPRINT (FNV-1a)
  h(A) ≠ h(B) → A ≠ B (O(1), deterministic)
  h(A) = h(B) → compare full bytes (collision < 0.001%)

G. FIBONACCI SUBDIVISION (camera)
  Split at φ⁻¹ point → optimal convergence.
  O(n × log_φ(n)) vs O(n²) brute force.

H. BELLMAN PATH
  Dynamic programming for optimal KnowTree search.
  Q(state, action) updated via Hebbian co-activation.
```

---

## G21. Performance Analysis

```
Operation                    | O()              | Typical
─────────────────────────────┼──────────────────┼──────────
P_weight lookup              | O(1)             | < 1μs
Compose 2 mols               | O(1)             | < 1μs
Compose chain (n mols)       | O(n)             | n=10 → 10μs
Distance 5D                  | O(1)             | < 1μs
Encode text (n chars)        | O(n)             | n=100 → 100μs
KnowTree insert              | O(1) amortized   | < 10μs
KnowTree nearest             | O(9 × bucket)    | bucket=50 → 450 comparisons
KnowTree classify            | O(branches)      | branches=10 → 10 comparisons
Silk walk (depth k)          | O(k × degree)    | k=3, deg=20 → 60 comparisons
Silk fire                    | O(1)             | < 1μs
Silk decay all               | O(edges)         | batch, during dream
STM push                     | O(32)            | scan for min score
STM evict                    | O(32)            | linear scan
Dream consolidation          | O(STM² + edges)  | 32² = 1024 pairs
Full pipeline                | O(n + 27×bucket) | 3 branches × 9 bucket lookups
SecurityGate                 | O(n)             | n = input length
Bloom filter check           | O(1)             | 3 hash lookups

Memory budget:
  P_weight table:   314 KB (fixed)
  KnowTree L0-L2:  ~1 MB (fixed)
  KnowTree L3+:    variable (grows with knowledge)
  STM:             ~32 × 100B = 3.2 KB
  WM:              4 × 2B = 8 bytes
  SilkIndex:       ~20 edges × 7B × 1000 nodes = 140 KB
  Bloom filter:    200 KB
  Total base:      ~1.7 MB
```

---

## G22. Goal System

```
Derive: D2⑥ Curiosity (meta-level) + E5 Self-model

goal_system():
  // Run when idle (no input for 5 min) or after dream

  // 1. Self-assess: where am I weak?
  model = self_model()  // G13
  weak_domains = [domain for domain, strength in model if strength < 0.40]

  // 2. Prioritize by: impact × feasibility
  goals = []
  for domain in weak_domains:
    impact = 1.0 - model[domain]  // weaker → higher impact
    feasibility = resource_available()  // heap, cpu, time
    priority = impact × feasibility
    goals.push({domain, priority})

  goals.sort_by(priority, desc)

  // 3. Execute top goal
  if goals:
    top = goals[0]
    // Generate self-directed learning query
    query = "learn more about " + top.domain
    process(query)  // run through pipeline → explore KnowTree

  // 4. Track progress
  growth_log(goals, model)
```

---

## G23. Verification Protocol

```
HIỂU = tính tay được. Không tính tay được = chưa hiểu = KHÔNG CODE.

Mỗi section có test cases tĩnh:

G2 Test — Compose:
  Input: a = pack(3,5,6,2,1) = 0x3D49, b = pack(7,3,2,5,0) = 0x7344
  Expected S: max(3,7) = 7
  Expected R: Zipf(5,3) ≈ 4 (weighted avg, first heavier)
  Expected V: amplify(6,2,0) = (6+2)/2 = 4 (w=0 → average)
  Expected A: max(2,5) = 5
  Expected T: dominant(1,0) = 1 (first heavier via Zipf)
  Expected: pack(7,4,4,5,1) = 0x7489 + T=1 = verify

G5 Test — KnowTree nearest:
  Insert 3 nodes: mol=0x1234, mol=0x1235, mol=0x5678
  Query: mol=0x1236
  Expected: nearest = 0x1235 (distance smallest)

G6 Test — Silk walk:
  Setup: node A (S=5,R=3,V=6,A=2,T=1), node B (S=5,R=3,V=7,A=3,T=1), node C (S=5,R=3,V=5,A=4,T=1)
  Hebbian: A→B weight[V]=0.8, B→C weight[V]=0.6
  Query dim: V (dominant)
  Walk from A: A → B (strongest V silk) → C (next strongest)
  Expected path: [A, B, C]

G9 Test — Honesty:
  Node with fire_count=0, silk_weight=0 → confidence = 0.0 → silence
  Node with fire_count=10, silk_weight=0.8, source_count=3, consistency=0.9
    → 0.3×0.8 + 0.3×1.0 + 0.2×1.0 + 0.2×0.9 = 0.24+0.30+0.20+0.18 = 0.92 → "Đúng."
```

---

## G24. Growth Metrics

```
Derive: E1 interoception (encode Nox's own state)

6 metrics, computed each session_save:

  qr_count:        len(qr_store)               // knowledge accumulation
  silk_density:     silk_edge_count / max(kt_total_nodes, 1)  // connection density
  avg_confidence:   mean(self_model().values())  // self-assessed strength
  dream_count:      total dreams this session    // creative discoveries
  self_modify_ok:   successful_mods / max(total_mods, 1)  // reliability
  pipeline_quality: mean(quality_scores this session)  // processing quality

Each metric → encode as interoception P_weight:
  S = 0 (no shape)
  R = pipeline_quality × 15      // how well system reasons
  V = avg_confidence × 7         // how "good" system feels about itself
  A = self_modify_ok × 7         // activity level of evolution
  T = uptime_bucket              // temporal phase

→ Nox encodes ITSELF into its own pipeline.
→ Self-awareness = interoception + self-model + growth_metrics.
```

---

## G25. Cognitive Failure Recovery

```
Derive: E6 NAC.mb + D2① Honesty

When pipeline fails (all 3 branches quality < φ⁻¹):

  1. RESPOND honestly: "Tôi không biết"

  2. LOG the failure:
     failure_record = {
       query_mol: mol,
       attempted_paths: paths,
       quality_scores: [q0, q1, q2],
       reason: identify_weakest(paths),  // which dim failed?
       timestamp: now()
     }

  3. LEARN from failure → negative knowledge:
     // Mark this region of 5D as "knowledge gap"
     // Next time similar query → instant "Tôi không biết" (no wasted compute)
     // OR: trigger curiosity → self-directed learning → fill gap

  4. GOAL UPDATE:
     gap_domain = kt_classify(mol)
     goals.push({domain: gap_domain, priority: HIGH, reason: "failure"})

Failure is a LEARNING SIGNAL, not just a stop.
```

---

## G26. Session Persistence

```
BEFORE session ends:
  session_save()  // G19
  // Includes: KnowTree, Silk, QR, top-8 STM, goals, metrics

AT session start:
  session_load()  // G19
  // Warm start: Nox continues where it left off
  // NOT cold start: not reading specs from scratch

WHAT persists:
  ✅ KnowTree (all learned nodes)
  ✅ Silk weights (Hebbian connections)
  ✅ QR records (proven knowledge)
  ✅ Top-8 STM entries (most important recent memories)
  ✅ Goals (what Nox was working on)
  ✅ Growth metrics (track progress)

WHAT does NOT persist (by design):
  ✗ WM (working memory clears after each response)
  ✗ Full STM (only top-8 survive)
  ✗ Pipeline state (reset each input)
```

---

## G27. Self-Evolution

```
Derive: F2 (proven) + G24 metrics

evolution_cycle():
  // Runs periodically or when metrics decline

  // 1. MEASURE — current performance
  current = growth_metrics()

  // 2. IDENTIFY — what's degrading?
  if current.pipeline_quality < previous.pipeline_quality:
    target = "pipeline"
  elif current.silk_density declining:
    target = "silk"
  elif current.avg_confidence declining:
    target = "knowtree"
  else:
    return  // no degradation → don't change

  // 3. INSPECT — read own source code
  source = read_file("stdlib/homeos/" + target + ".ol")

  // 4. COMPARE — does code match G spec?
  // (This is where G document is essential)
  // G spec = the reference. Code MUST match.

  // 5. MODIFY — fix divergence (1 file per cycle)
  backup = copy(source)
  modified = generate_fix(source, G_spec)
  write(source, modified)

  // 6. BUILD + TEST
  if not (make_vm() AND make_self_build() AND make_test()):
    // ROLLBACK
    write(source, backup)
    dn_observe("evolution failed: " + target)
    return

  // 7. VERIFY — fixed-point
  if not make_fixed_point():
    write(source, backup)
    dn_observe("fixed-point failed: " + target)
    return

  // 8. COMPARE metrics
  new_metrics = growth_metrics()
  if new_metrics > current:
    dn_observe("evolved: " + target + " improved")
    commit()
  else:
    write(source, backup)
    dn_observe("evolution neutral: " + target + " no improvement")
```

---

## Tóm Tắt

```
G = 27 sections
  G0:  Cross-reference (6 contradictions resolved)
  G1:  Data structures (KnowTree, SilkIndex, STM, WM, QR)
  G2:  Core math (pack, compose, distance, dominant_dim)
  G3:  Encode ∫
  G4:  Decode ∂ (lookup + generative)
  G5:  KnowTree (insert, nearest, classify, HNSW)
  G6:  Silk (walk, fire, decay)
  G7:  Memory (STM eviction, dream consolidation)
  G8:  Pipeline (14 steps, 5 checkpoints)
  G9:  Instincts (7 formulas, pure 5D math)
  G10: SecurityGate (3 layers, Bloom from P_weight)
  G11: Homeostasis + ConversationCurve
  G12: Immune Selection + DNA Repair
  G13: Self-model
  G14: NAC algorithms (pruning, recovery, negative knowledge)
  G15: Capture (text, camera Fibonacci SDF, audio, /proc)
  G16: Generation (chain recombination = SINH)
  G17: Agent cycle (perceive→think→act→verify)
  G18: Bootstrap (exact sequence)
  G19: Persistence (3-tier, QT8 write order)
  G20: Optimizations (HNSW, Bellman, Bloom, compression, Fibonacci)
  G21: Performance (O() for every operation)
  G22: Goal System (self-directed curiosity)
  G23: Verification Protocol (test cases, "hiểu = tính tay được")
  G24: Growth Metrics (6 metrics, self-encode)
  G25: Failure Recovery (failure → learning signal)
  G26: Session Persistence (warm start)
  G27: Self-Evolution (measure→identify→modify→test→compare)

Mọi thuật toán = toán từ A-F + UDC docs.
Không hardcode. Không if/else trên keyword.
Mỗi quyết định = distance 5D hoặc φ⁻¹ threshold.
```

---

> **Source:** A-F specs, SPEC_v3, BLUEPRINT, SINH_HOC_v2, KNOWTREE_DESIGN,
> SPEC_NODE_SILK, silk_architecture, STORAGE_AND_SEARCH_NOTE,
> UDC_formulas, UDC_real_formulas, UDC_A_AROUSAL_tree, UDC_V_VALENCE_tree,
> UDC_R_RELATION_tree, UDC_T_TIME_tree, UDC_semantic_groups
