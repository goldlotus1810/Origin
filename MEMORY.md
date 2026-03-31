# HomeOS Project Memory

> Tóm tắt trạng thái dự án để AI contributors nắm nhanh context.
> Cập nhật: 2026-03-31

---

## Trạng thái tổng quan

- **Phase**: V2 Migration hoàn tất. Đang Phase 2 Optimization + CUT (Rust→Olang).
- **Tests**: ~1190 pass, 37 remaining (closure/self-compile/bytes builtins)
- **Binary**: `make smoke-binary` BẮT BUỘC trước commit
- **Build time**: Bootstrap 10.73s → 2.91s (3.7x) sau VM.1-5

---

## V2 Format — Thay đổi nền tảng

```
Molecule:  5B → 2B packed u16: [S:4][R:4][V:3][A:3][T:2]
Chain:     Vec<Molecule> → Vec<u16>
KnowTree:  320KB → 128KB cây phân tầng L0→L1→L3
UCD:       5,400 → 8,846 chars (59 Unicode blocks)
ShapeBase: 8 → 18 SDF primitives
Silk:      37 → 75 kênh × 31 patterns = 2,325 relation types
Compose:   KHÔNG average — Amplify qua Silk weight
```

---

## Formula Engine (FE.1-8) — DONE

```
FE.1: RelationOp   — 16 phép toán category theory
FE.2: ValenceState  — 8 trạng thái potential energy
FE.3: ArousalState  — 8 trạng thái damped oscillator
FE.4: SplineKnot   — 24B temporal spline observation
FE.5: TimeHistory   — Append-only, predict familiarity
FE.6: WIRED         — urgency A≥6 → crisis
FE.7: FormulaState  — Unified R+V+A dispatch
FE.8: ParametricSdf — T×S, CSG union/smooth_union
```

---

## VM Optimizations

```
DONE: VM.1 (zero-alloc strings), VM.2 (keyword hash), VM.3 (batch step),
      VM.4 (scope cache FNV-1a), VM.5 (builtin dispatch)
FREE: VM.6 (small-chain SSO), VM.7 (KnowTree sampling), VM.8 (Bellman path)
```

---

## Agent Hierarchy

```
AAM [tier 0]  — stateless, approve, quyết định cuối
Chiefs [tier 1] — LeoAI · HomeChief · VisionChief · NetworkChief
Workers [tier 2] — SILENT, báo cáo chain only

Luật: AAM↔Chief ✅  Chief↔Chief ✅  Chief↔Worker ✅
      AAM↔Worker ❌  Worker↔Worker ❌
```

---

## Runtime Auth

```
Virgin → Locked → Unlocked (Master Key + Argon2id)
```

---

## TASKBOARD tóm tắt

```
DONE:    Phase 0-16, V2 T1-T16, VM.1-5, FE.1-8, AUTH
FREE:    VM.6-8, TLC test suite, CUT.1-4, Mobile 7.2
BLOCKED: P2.2-2.5 (emotion/knowledge/agent .ol), REPL native, Browser E2E
```

---

## Key files

| File | Mục đích |
|------|---------|
| `olang/src/mol/formula.rs` | FE.1-3: R→16 ops, V→8 states, A→8 states |
| `olang/src/mol/spline.rs` | FE.4-5: TimeHistory, SplineKnot 24B |
| `vsdf/src/render/parametric.rs` | FE.8: T×S parametric shapes |
| `json/udc.json` | UCD source 8,846 chars |
| `docs/CHECK_TO_PASS_LOGIC_HANDBOOK.md` | 6 bug patterns + 5 checkpoints |

---

## Quy tắc nhớ

1. `make smoke-binary` BẮT BUỘC trước commit
2. Đọc TASKBOARD.md → claim task → rồi mới code
3. KHÔNG viết Rust mới ngoài Plan
4. Compose KHÔNG BAO GIỜ average — amplify qua Silk
5. Append-only — KHÔNG delete, KHÔNG overwrite
6. Giao tiếp với user bằng tiếng Việt
