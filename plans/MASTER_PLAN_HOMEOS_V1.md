# MASTER PLAN — HomeOS v1.0

> **Nox — 2026-03-25. Cap nhat lien tuc.**
> **Tong hop: Spec v3, Sora "Buc Tranh Tong The", PLAN_REWRITE, Kira findings.**

---

## HIEN TRANG SAU VM SCOPE FIX

```
VM scope bug: FIXED ✅ (op_ret chi restore khi return TO eval, khong boot→boot)
  Truoc: eval→boot→boot sub-fn → eval vars WIPED
  Sau:   eval→boot→boot sub-fn → eval vars PRESERVED

Silk:   9→17 edges (5 turns) — ALIVE (truoc: 9→10, 10 turns — DEAD)
Mol:    compose hoat dong (truoc: moi word = 146)
Search: keyword ×5 + mol — dung fact (truoc: random)

Binary: 1,021KB | Tests: 20/20 | Self-build: 3 doi
```

---

## KIEN TRUC MUC TIEU (tu Sora "Buc Tranh Tong The")

```
MOI THU = NODE. Compose len. Link ngang.

L1: 172,849 char nodes (lazy create — 0 boot cost)
L5+: word nodes = chain(char nodes)
L6+: fact nodes = chain(word nodes)
L7+: skill nodes = chain(fn nodes)

Search = walk tree: query → char path → word node → fact links
         O(word_length), khong O(knowledge_count)

KHONG keyword scan. KHONG mol similarity. TREE IS INDEX.
```

---

## LO TRINH — 5 GIAI DOAN

### GD.1: Xuong Song — KnowTree that (~200 LOC, 3-5 sessions)

> Muc tieu: Moi input = walk tree. Khong keyword scan.

| # | Task | Effort | Dep | Status |
|---|------|--------|-----|--------|
| N.1 | Lazy char node: gap char → tao node (hash table) | ~40 LOC | VM fix ✅ | TODO |
| N.2 | Word node: chain(char nodes) + bidirectional links | ~40 LOC | N.1 | TODO |
| N.3 | Fact node: chain(word nodes) + reverse links word↔fact | ~30 LOC | N.2 | TODO |
| N.4 | Tree search: query → char path → word → fact links | ~40 LOC | N.3 | TODO |
| N.5 | Replace __knowledge[] → tree storage | ~30 LOC | N.4 | TODO |
| N.6 | learn/respond dung tree thay keyword scan | ~20 LOC | N.5 | TODO |

### GD.2: Tuan Hoan — Neuron model (~150 LOC, 2-3 sessions)

> Muc tieu: STM → Silk → Dream → QR. Vong doi tri thuc.

| # | Task | Effort | Dep | Status |
|---|------|--------|-----|--------|
| NR.1 | STM = dendrites: node-based, evict oldest | ~30 LOC | GD.1 | TODO |
| NR.2 | Silk = synapse: Hebbian on nodes (khong string) | ~30 LOC | NR.1 | TODO |
| NR.3 | Dream = cluster STM by LCA, fibonacci folding | ~40 LOC | NR.2 | TODO |
| NR.4 | QR = axon: propose → approve → append-only | ~30 LOC | NR.3 | TODO |
| NR.5 | SilkWalk amplify: traverse graph, accumulate emotion | ~20 LOC | NR.2 | TODO |

### GD.3: Tu Duy — Skills + Instincts (~100 LOC, 2 sessions)

> Muc tieu: 7 instincts = 7 Skills (QT4). Stateless, isolated.

| # | Task | Effort | Dep | Status |
|---|------|--------|-----|--------|
| SK.1 | Skill trait: stateless, ExecContext | ~20 LOC | GD.2 | TODO |
| SK.2 | Honesty: confidence → fact/opinion/silence | ~15 LOC | SK.1 | ⚠️ partial |
| SK.3 | Contradiction: valence opposition | ~10 LOC | SK.1 | ⚠️ partial |
| SK.4 | Causality: temporal + coactivation | ~15 LOC | SK.1 | ⚠️ partial |
| SK.5 | Abstraction: LCA + variance | ~15 LOC | SK.1 | TODO |
| SK.6 | Analogy: A:B :: C:? = C + (B-A) 5D | ~15 LOC | SK.1 | TODO |
| SK.7 | Curiosity + Reflection | ~10 LOC | SK.1 | ⚠️ partial |

### GD.4: Cam Xuc — ConversationCurve (~80 LOC, 1-2 sessions)

> Muc tieu: f(x), f'(x), f''(x). Trajectory, khong snapshot.

| # | Task | Effort | Dep | Status |
|---|------|--------|-----|--------|
| CC.1 | V(t) + derivatives (toc do + gia toc) | ~20 LOC | GD.3 | TODO |
| CC.2 | Window variance (emotional instability) | ~15 LOC | CC.1 | TODO |
| CC.3 | Tone from derivatives (khong tu V hien tai) | ~15 LOC | CC.2 | TODO |
| CC.4 | SilkWalk amplify context | ~15 LOC | CC.3 | TODO |
| CC.5 | Cross-modal weight (bio > audio > text) | ~15 LOC | CC.4 | TODO |

### GD.5: Xa Hoi — Agents + ISL (future)

> Muc tieu: AAM → LeoAI → Chiefs → Workers.

| # | Task | Effort | Dep | Status |
|---|------|--------|-----|--------|
| AG.1 | AAM: stateless approve/reject | ~20 LOC | GD.4 | TODO |
| AG.2 | LeoAI: orchestrate Skills | ~40 LOC | AG.1 | TODO |
| AG.3 | ISL: 4-byte addr, AES-256-GCM | ~100 LOC | AG.2 | TODO |
| AG.4 | Worker: HomeOS thu nho | ~50 LOC | AG.3 | TODO |

---

## TRUOC KHI BAT DAU GD.1

### Da fix:
```
✅ VM eval↔boot scope bug (op_ret check r12 == boot_bc_base)
✅ Case-insensitive _a_has (inline lowercase)
✅ Gate zero-score → khong tra random fact
✅ Persistence no dupes
✅ Greeting/goodbye router
✅ Math ?/= strip
✅ Dict pretty-print
✅ UTF-8 __utf8_cp/__utf8_len builtins
```

### Con fix cho v1.1 (song song voi GD.1):
```
□ sort_by boot closure (van chua work — khac issue voi scope)
□ set_at/push auto-emit noise
□ fn_node_fire wiring
```

---

## NGUYEN TAC BAT BIEN

```
1. Unicode la nguon su that DUY NHAT
2. Moi thu = node. Compose len. Link ngang.
3. 0 hardcoded molecule. 0 keyword hack.
4. Tree IS index. Walk tree. Khong scan array.
5. Im lang khi khong biet. Hoi lai khi khong chac.
6. Compose ≠ Average. Khuech dai dominant (φ⁻¹).
7. fn{fn{...}} == fn. ∞-1.
8. Viet bang Olang. Khong Rust moi.
9. Moi module = 1 file .ol. Test rieng duoc.
10. Node truoc. Agent sau. Tree truoc. Search sau.
```

---

## METRIC

```
Hien tai:
  Binary: 1,021KB | Tests: 20/20 | LOC: 21,559
  Knowledge: 28 embedded + 166 training (flat array)
  Silk: 17 edges (mol-keyed, working after scope fix)
  Nodes: linear growth (1 per input)

Muc tieu GD.1:
  Knowledge: tree-based (lazy nodes)
  Search: O(word_length) tree walk
  0 keyword scan. 0 __knowledge[] array.

Muc tieu GD.5:
  Agents: AAM → LeoAI → Workers
  ISL: cross-device messaging
  0 hardcoded anything
```

---

*"Chua co linh hon. Gio xay." — Sora*
*"VM scope fix = mo khoa. Tree = xuong song." — Nox*
