# PLAN: V2 Migration Tổng Thể — BIG BANG

> **Status:** DRAFT
> **Ngày:** 2026-03-21
> **Tham chiếu:** AUDIT_TONG_HOP.md (51 issues), PLAN_PWEIGHT_MIGRATION.md
> **Nguyên tắc:** Molecule thay → Chain thay → LCA thay → KnowTree thay → HẾT thay. Không incremental.

---

## Dependency Graph

```
T1 UCD build.rs (58 blocks, 9584 entries, đọc udc.json)
 ├→ T2 ShapeBase 8→18 SDF (tách CSG ops)
 │   └→ T2b SdfPrimitive agents 5→18
 └→ T3 Molecule 5B→2B (packed u16)
     ├→ T4 Chain Vec<Mol>→Vec<u16>
     │   ├→ T7 Writer/Reader (serialize 2B/link)
     │   ├→ T8 Registry (codepoint array thay hash)
     │   └→ T10 Downstream crates (silk/agents/memory/vsdf)
     ├→ T5 LCA (amplify/Union/max/dominant)
     │   └→ T10
     ├→ T6 KnowTree (array 65536×2B)
     │   └→ T8
     ├→ T9 VM + Bytecode (PushMol 2B)
     │   └→ T11 .ol files (stdlib + HomeOS + bootstrap)
     └→ T12 Tests rebuild
```

**Thứ tự thực hiện:**
```
Layer 0: T1 (UCD) + T2 (ShapeBase)     ← song song, không phụ thuộc nhau
Layer 1: T3 (Molecule)                  ← phụ thuộc T1+T2
Layer 2: T4+T5+T6 (Chain+LCA+KnowTree) ← song song, đều phụ thuộc T3
Layer 3: T7+T8+T9 (Storage+Registry+VM) ← phụ thuộc T4/T6
Layer 4: T10+T11 (Downstream+.ol)       ← phụ thuộc T3-T9
Layer 5: T12 (Tests)                    ← cuối cùng
```

---

## Tổng quan 12 Tasks

| Task | Tên | Files | Depends | Ước tính |
|------|-----|-------|---------|----------|
| T1 | UCD build.rs rebuild | 2 files | — | Lớn |
| T2 | ShapeBase 18 SDF | 3 files | — | Nhỏ |
| T3 | Molecule 2B packed | 2 files | T1,T2 | Lớn |
| T4 | Chain Vec<u16> | 2 files | T3 | TB |
| T5 | LCA v2 rules | 1 file | T3 | TB |
| T6 | KnowTree array | 2 files | T3 | TB |
| T7 | Writer/Reader v2 | 2 files | T4 | TB |
| T8 | Registry codepoint | 1 file | T4,T6 | TB |
| T9 | VM PushMol 2B | 3 files | T3 | Nhỏ |
| T10 | Downstream crates | ~10 files | T3-T8 | Lớn |
| T11 | .ol files update | ~15 files | T9 | Lớn |
| T12 | Tests rebuild | ~12 files | T10,T11 | TB |

---

_(Chi tiết từng Task xem bên dưới — thêm dần)_
