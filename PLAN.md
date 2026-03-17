# PLAN: 3 Vấn đề cốt lõi — SecurityGate + Tests + Shape System

---

## Tóm tắt vấn đề

### Vấn đề 1: SecurityGate bị bypass hoàn toàn qua ○{}
- `process_text("○{malicious}")` → `process_olang()` → **KHÔNG gọi gate**
- VM events (STM push, graph co_activate) → **KHÔNG qua gate**
- Dream cycle → tạo nodes → **KHÔNG qua gate**
- Evolution detection → **KHÔNG qua gate**
- **Root cause**: Gate chỉ enforce trong `LearningLoop::process_one()`, nhưng nhiều path skip nó

### Vấn đề 2: 37 tests có `if table_len() == 0 { return; }` = dead code
- UCD table LUÔN có 5,279 entries (build.rs sinh lúc compile)
- Pattern này KHÔNG BAO GIỜ trigger → tests luôn chạy → dead code gây nhầm lẫn
- Nhưng vấn đề sâu hơn: cơ chế L0 "bảo vệ" này vô nghĩa — nếu UCD không có thì cả hệ thống không chạy

### Vấn đề 3: 5 primitives không thể vẽ thực tế
- ShapeBase: Sphere, Capsule, Box, Cone, Torus (5 hình) + Union, Intersect, Subtract (3 CSG ops)
- 18 SdfKind tồn tại nhưng 10 cái KHÔNG BAO GIỜ đạt được từ Molecule
- CSG ops ở ShapeBase nhưng KHÔNG CÓ cơ chế compose (Union CỦA GÌ?)
- Scene graph + Transform tồn tại nhưng disconnected khỏi Molecule pipeline
- Không thể vẽ người, cây, hay bất cứ gì phức tạp hơn 1 hình đơn

---

## PLAN

### Phase 1: Fix SecurityGate — Tường lửa phải là tường lửa

**File: `crates/runtime/src/origin.rs`**

#### Step 1.1: Gate check trong `process_olang()`
```
Trước khi compile/execute ○{}, gọi gate.check_text() trên raw expression.
- Block nếu Olang chứa harmful content
- process_olang() thêm gate check ngay dòng đầu
```

#### Step 1.2: Gate check cho VM events
```
Sau khi VM chạy xong, trước khi push STM/co_activate graph:
- VmEvent::Output(chain) → validate chain trước khi push STM
- VmEvent::CreateEdge → validate trước khi co_activate
Tạo helper: gate_check_vm_event() trong runtime
```

#### Step 1.3: Gate check cho Dream cycle
```
Dream tạo evolved nodes → cần validate:
- Sau evolve() thành công → check evolved chain content
- Không block dream hoàn toàn (dream = internal) nhưng log suspicious evolutions
```

#### Step 1.4: Giới hạn public accessors
```
- stm_mut(), graph_mut() → đổi thành pub(crate)
- Hoặc tạo gated wrapper: push_stm_gated(), co_activate_gated()
- External code phải đi qua gate
```

#### Step 1.5: Tests cho gate enforcement
```
- Test: ○{} expression với harmful content → bị block
- Test: normal text harmful → bị block (đã có)
- Test: VM event injection → bị gate check
```

---

### Phase 2: Fix Tests — Xóa dead code, thêm real tests

#### Step 2.1: Xóa `if table_len() == 0 { return; }` khỏi 37 tests
```
Files:
- crates/ucd/src/lib.rs (14 tests)
- crates/olang/src/encoder.rs (7 tests)
- crates/runtime/src/origin.rs (4 tests)
- crates/agents/src/encoder.rs (3 tests)
- crates/agents/src/learning.rs (3 tests)
- crates/agents/src/leo.rs (2 tests)
- crates/agents/src/instinct.rs (1 test)
- crates/agents/src/book.rs (1 test)
- crates/olang/src/knowtree.rs (1 test)
- tools/seeder/src/main.rs (1 test)

Đây là dead code — xóa sạch. Nếu UCD build.rs fail thì compile fail,
không cần runtime check.
```

#### Step 2.2: Thêm compile-time assertion
```
Trong crates/ucd/build.rs: thêm assert count >= 5000
→ Nếu UCD data bị hỏng → build fail, không compile, rõ ràng hơn
   runtime "skip silently"
```

---

### Phase 3: Shape System — Mở rộng từ 5 primitives lên composition system

#### Step 3.1: Mở rộng ShapeBase encoding
```
Hiện tại: 8 values (1-8), dùng hierarchical: base + sub_index*8
→ sub_index chưa bao giờ dùng cho shape diversification

Kế hoạch: sử dụng sub_index để encode THÊM SdfKind:
  base=Sphere(1), sub=0 → Sphere
  base=Sphere(1), sub=1 → Ellipsoid (sphere biến dạng)
  base=Capsule(2), sub=0 → Capsule
  base=Capsule(2), sub=1 → Cylinder (capsule không bo)
  base=Cone(4), sub=0 → Cone
  base=Cone(4), sub=1 → HexPrism (cone discrete)
  base=Torus(5), sub=0 → Torus
  base=Torus(5), sub=1 → Helix (torus cuộn)
  ... vv

→ 10 SdfKind unreachable trở nên reachable qua sub_index
→ Không thay đổi wire format, backward compatible
```

#### Step 3.2: Composition qua MolecularChain (không phải single Molecule)
```
Hiện tại: 1 Molecule → 1 NodeBody → 1 primitive (luôn đơn lẻ)

Kế hoạch: MolecularChain → SceneNode tree
  Chain [Union][Sphere][Capsule] = "Sphere ∪ Capsule"
  Chain [Intersect][Box][Sphere] = "Box ∩ Sphere"

  CSG ShapeBase (Union/Intersect/Subtract) = OPERATORS trong chain
  Geometric ShapeBase = OPERANDS

  Parser: đọc chain left-to-right, build SceneNode tree
  → chain_to_scene(chain) → SceneNode with children

  Ví dụ "người":
  Chain [Union][Sphere][Union][Capsule][Capsule][Capsule][Capsule][Sphere][Sphere]
  = head(Sphere) ∪ body(Capsule) ∪ leftArm(Capsule) ∪ rightArm(Capsule)
    ∪ leftLeg(Capsule) ∪ rightLeg(Capsule)...

  Params (scale, position) → từ V/A/T bytes + parent-relative transform
```

#### Step 3.3: body_from_chain() — chain → SceneNode
```
Mới: body_from_chain(chain: &MolecularChain) → SceneNode
  - Nếu chain.len() == 1 → single primitive (backward compatible)
  - Nếu chain có CSG operators → build composition tree
  - Transform per-molecule từ position trong chain (Fibonacci spacing)

File: crates/vsdf/src/body.rs — thêm function mới, giữ body_from_molecule()
```

#### Step 3.4: Innate shape formulas cho emoticons
```
Mapping Unicode emoticon → composition chain template:
  🔥 (fire)  → [Union][Cone][Sphere] + spline:Flicker
  💧 (water) → [Sphere] + spline:Flow
  🌳 (tree)  → [Union][Cone][Capsule] (crown + trunk)
  🧑 (person)→ [Union][Sphere][Capsule][Capsule][Capsule][Capsule][Capsule]
  🏠 (house) → [Union][Box][Cone] (walls + roof)

  ~80-120 templates phủ ~1760 emoticons (nhiều dùng chung template)
  Lưu trong UCD build.rs hoặc innate module riêng
```

---

## Thứ tự thực hiện

```
Phase 1 (SecurityGate)  ← ưu tiên cao nhất, security hole
  1.1 gate trong process_olang     ~30 lines
  1.2 gate cho VM events           ~40 lines
  1.3 gate cho Dream               ~20 lines
  1.4 restrict public accessors    ~20 lines
  1.5 tests                        ~60 lines

Phase 2 (Tests cleanup)  ← đơn giản, xóa dead code
  2.1 xóa 37 skip patterns        ~37 deletions
  2.2 compile-time assert          ~5 lines

Phase 3 (Shape System)  ← lớn nhất, chia nhỏ
  3.1 sub_index → SdfKind mapping  ~50 lines
  3.2 chain composition design     ~100 lines
  3.3 body_from_chain()            ~150 lines
  3.4 innate templates             ~200 lines (data-heavy)
```

## Tổng: ~700 lines code mới/sửa
