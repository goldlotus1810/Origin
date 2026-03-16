# HomeOS — Kế Hoạch Tiếp Theo

**Ngày:** 2026-03-16
**Dựa trên:** REVIEW.md, HomeOS_Roadmap.md, HomeOS_Solutions.md

---

## Trạng thái hiện tại

**Điểm: 8.66/10 (A-) · 1,644 tests · 0 clippy warnings · 54/80 features**

Foundation hoàn chỉnh: UCD, Olang, Silk, Emotion, Memory, ISL, HAL, VSDF, Agents, WASM.

---

## Kế hoạch Phase tiếp theo

### Phase 1 — VM Tính Toán Thật (Ưu tiên: CAO)

**Vấn đề:** `1 + 2` tạo sự kiện nhưng KHÔNG trả về `3`.

```
Cần làm:
  [1.1] Thêm Op::PushNum(f64) vào IR
  [1.2] Dispatch __hyp_add/__hyp_sub/__hyp_mul/__hyp_div trong Op::Call
  [1.3] Kết nối math.rs AST vào VM execution
  [1.4] Test: ○{1 + 2} → Output(3.0)

Files cần sửa:
  crates/olang/src/ir.rs      — thêm Op::PushNum
  crates/olang/src/vm.rs      — dispatch builtins
  crates/olang/src/syntax.rs  — parse numeric expressions
```

### Phase 2 — Duyệt Đồ Thị (Ưu tiên: TRUNG BÌNH)

**Vấn đề:** `why` và `explain` chỉ in hash, không duyệt đường đi.

```
Cần làm:
  [2.1] Implement find_path(from, to) → Vec<u64> trong walk.rs
  [2.2] Implement trace_origin(hash) → Vec<(u64, EdgeKind)>
  [2.3] Implement reachable(hash, depth) → BTreeSet<u64>
  [2.4] Kết nối vào runtime: why → trace_origin, explain → find_path

Files cần sửa:
  crates/silk/src/walk.rs        — thêm find_path, trace_origin, reachable
  crates/runtime/src/origin.rs   — dispatch why/explain
```

### Phase 3 — Tri Thức L1+ (Ưu tiên: TRUNG BÌNH)

**Vấn đề:** Chỉ có 35 node L0. Không biết H2O, F=ma, DNA.

```
Cần làm:
  [3.1] Định nghĩa 180+ domain nodes (toán, lý, hóa, sinh, triết)
  [3.2] Seed qua seeder tool (KHÔNG hardcode — dùng encode_codepoint)
  [3.3] LCA tự tính parent từ dưới lên
  [3.4] Silk connect domain → L0 concepts

Files cần sửa:
  tools/seeder/src/main.rs — thêm seed_l1_knowledge()
```

### Phase 4 — Toán Ký Hiệu (Ưu tiên: TRUNG BÌNH)

**Vấn đề:** math.rs có solve/derive/integrate nhưng chưa nối VM.

```
Cần làm:
  [4.1] Thêm Expr::MathEq, Expr::Derivative vào syntax.rs
  [4.2] Parser nhận diện ○{solve "2x + 3 = 7"}
  [4.3] VM dispatch math commands → math.rs
  [4.4] Test: ○{solve "2x + 3 = 7"} → x = 2
```

### Phase 5 — Điều Phối Agent (Ưu tiên: CAO)

**Vấn đề:** Agent hoạt động riêng lẻ, chưa phối hợp.

```
Cần làm:
  [5.1] Nối process_text() → LeoAI.process()
  [5.2] LeoAI.process() → đề xuất → AAM.review()
  [5.3] AAM approved → thực thi (ghi QR, gửi ISL)
  [5.4] Chief → Worker dispatch qua ISL
  [5.5] Test: end-to-end text → learn → dream → propose → approve

Files cần sửa:
  crates/runtime/src/origin.rs  — wire orchestration loop
  crates/agents/src/leo.rs      — full process() pipeline
```

---

## Cải thiện code (từ Review)

### Đã hoàn thành
- [x] 8 clippy warnings → 0
- [x] QT11 enforcement: `co_activate_same_layer()` method

### Đang tiến hành
- [ ] Giảm unwrap() trong olang (291 → target: <100)
  - Ưu tiên: math.rs(47), syntax.rs(47), semantic.rs(50)
  - Thay bằng `?`, `match`, `unwrap_or`

### Cần làm
- [ ] Thêm tests cho tools (inspector, server, bench)
- [ ] API documentation (`///` docs) cho core crates
- [ ] Giảm unwrap() trong isl(24), agents(18), runtime(16)

---

## Đường đi tới hạn

```
Phase 1 (VM tính toán)
├── Phase 2 (Duyệt đồ thị)
│   ├── Phase 5 (Điều phối Agent) — QUAN TRỌNG NHẤT
│   │   ├── Phase 6 (Cảm nhận)
│   │   └── Phase 8 (Tầng Build)
│   └── Phase 8 (Tầng Build)
├── Phase 3 (Tri thức)
│   └── Phase 4 (Toán ký hiệu)
└── Phase 7 (Compiler backends)
```

---

## Hạn chế đã nhận diện

| ID | Vấn đề | Hướng giải quyết | Chi tiết |
|----|--------|-------------------|----------|
| H1 | LCA mất thông tin chain dài | Weighted LCA + Mode + Variance | Đã có lca_with_variance() |
| H2 | decode_chain O(n) | Reverse index trong build.rs | Chưa implement |
| H3 | Ln-1 thay đổi theo thời gian | Branch watermark per branch | Đã có branch_watermark |
| H4 | Dream cluster sai | Silk co-activation filter | Configurable α,β,γ |
| H5 | SDF fitting khó | Iterative + confidence score | Chưa implement |
| H6 | Olang compile chưa đủ | 3 tầng: IR → Target → Execute | Partial (C/Rust/WASM done) |

Chi tiết: xem [HomeOS_Solutions.md](../HomeOS_Solutions.md).

---

*2026-03-16 · HomeOS Next Steps*
