# HomeOS — Review

**Ngày:** 2026-03-18 (Phiên N)
**Phương pháp:** Build + test + đọc code + deep review agent + so sánh docs vs thực tế
**Tests:** 2,359 pass · 0 fail

---

## Tóm Tắt

HomeOS có nền tảng kỹ thuật xuất sắc (84K LoC, 2359 tests, 0 deps, kiến trúc DAG sạch). Phiên N đánh dấu bước ngoặt: **origin.olang = bộ nhớ duy nhất** (9 record types, RAM = cache). Agent hierarchy đã WIRED. 6/6 vấn đề hệ thống từ phiên L đã RESOLVED.

---

## I. Điểm Số

| Hạng mục | Điểm | Δ vs K | Ghi chú |
|----------|------|--------|---------|
| Thiết kế kiến trúc | 10/10 | = | DAG sạch, append-only, no circular deps |
| Chất lượng code | 9/10 | = | 0 unsafe, well-structured |
| Độ phủ test | 9/10 | = | 2,359 tests, mọi crate có test |
| Tuân thủ QT (23 rules) | 9.5/10 | +0.5 | QT8 enforced (file trước, RAM sau) |
| Tính năng hoạt động | 8.5/10 | +1.5 | Agent wired, memory persist, module system |
| Bảo mật | 9/10 | = | SecurityGate + CapabilityGate + native crypto |
| Độc lập (0 deps) | 10/10 | = | SHA-256, Ed25519, AES-256-GCM, homemath tự viết |
| **Tổng** | **9.3/10** | +0.3 | |

---

## II. Hoạt Động Thật Sự

### Emotion Pipeline: HOẠT ĐỘNG
- Input text → 7-layer analysis → V/A scores → ConversationCurve → tone selection
- Crisis detection → hotline (1800 599 920)
- Silk amplification → composite emotion (không trung bình)
- **MỚI:** ConversationCurve persist/restore qua origin.olang (RT_CURVE)

### VM + Olang: HOẠT ĐỘNG
- ○{1+2} = 3, ○{solve "2x+3=7"} → x = 2
- ○{dream}, ○{stats}, ○{program ...}, ○{read ...}
- 36 opcodes, 18 RelOps
- **MỚI:** Module system (ModuleLoader + DepGraph + cycle detection)
- **MỚI:** Stdlib 10 modules (math, string, vec, set, map, io, test...)
- **MỚI:** Semantic analysis (scope, effects, move-checking)

### Core Engine: HOẠT ĐỘNG
- 5D Molecule encoding, tagged sparse (1-6 bytes)
- LCA weighted + variance
- Registry append-only + crash recovery
- Silk Hebbian + φ⁻¹ decay
- ISL AES-256-GCM encrypted messaging
- **MỚI:** Hebbian persist/restore qua origin.olang (RT_HEBBIAN)
- **MỚI:** restore_learned() — boot replay without Hebbian formula

### Dream + Learning: HOẠT ĐỘNG
- STM learn 5 layers (paragraph → character)
- Dream auto-trigger Fibonacci
- Instincts wired into response flow
- SkillPattern → AAM approval pipeline
- **MỚI:** STM persist/restore qua origin.olang (RT_STM) — survive restart
- **MỚI:** KnowTree persist/restore (RT_KNOWTREE) — L2+ knowledge

### Agent Hierarchy: ✅ WIRED (was FACADE)
- **Chiefs (3):** Domain automation — HomeChief (temp >35°C → cooling), VisionChief (motion events), NetworkChief (security escalation)
- **Workers (4 kinds):** Sensor, Actuator, Camera, Network — full ISL frame parsing
- **MessageRouter:** 7-phase pump — Workers→Chiefs→LeoAI→AAM→Dream
- **BookReader:** Wired to runtime (○{read ...} → sentence parsing → STM → Silk → KnowTree)
- **Domain Skills (15):** Called in learning pipeline, not just tests
- **Compiler backends:** C/Rust/WASM generation working

---

## III. Cần Cải Thiện

### 1. Response Template (Priority: HIGH)
- ~10 templates, parameterized by Tone + Valence + Language
- Not personalized per conversation — fixed template pool
- Instinct results need richer surface in text

### 2. Command Execution (Priority: MEDIUM)
- typeof, explain, why, trace, inspect, assert — parsed ✅
- Execution mostly stubs — explain/why need LCA graph support

### 3. Compiler FFI (Priority: LOW)
- C/Rust/WASM backends generate syntactically correct code
- Need FFI stubs for actual execution (olang_device_write, etc.)

### 4. Type System Runtime (Priority: LOW)
- Semantic lowering done (scope, effects, move-checking)
- Not enforced at VM runtime yet

---

## IV. Điểm Mạnh Thật Sự

| Điểm mạnh | Chi tiết |
|-----------|---------|
| Zero dependencies | SHA-256, Ed25519, AES-256-GCM, homemath — tất cả tự viết |
| Privacy tuyệt đối | Local-only, append-only, no cloud, no data collection |
| Kiến trúc DAG sạch | Không circular deps, L0 không import L1 |
| 5D Molecule encoding | Độc đáo, hoạt động, ~5400 Unicode entries |
| Code discipline | 0 unsafe, 2359 tests |
| Self-awareness | Dự án tự biết điểm yếu qua nhiều bản review |
| **MỚI: QT8 enforced** | origin.olang = bộ nhớ duy nhất, 9 record types |
| **MỚI: Agent hierarchy live** | Chiefs+Workers+Router wired, not facades |
| **MỚI: Module system** | Import/export, cycle detection, stdlib |

---

## V. Khoảng Cách Vision vs Reality

| "Sinh linh" | Thiết kế | Thực tế | Gap |
|------------|----------|---------|-----|
| Tự học | STM → Dream → QR | STM hoạt động, Dream + persist | Gần ✅ |
| Tự nhớ | Short + Long-term | STM + Hebbian + Curve + KnowTree persist | **Đạt ✅** |
| Tự suy luận | 7 instincts | Chạy + wired vào output | Gần |
| Tự vận hành | Agent hierarchy | Chiefs+Workers+Router wired | Gần ✅ |
| Tự bảo vệ | SecurityGate | Hoạt động tốt | **Đạt ✅** |
| Tự biểu đạt | ConversationCurve | Tone đúng, text cần enrichment | Trung bình |
| Tự tái tạo | origin.olang executable | PLAN_REWRITE: 7 giai đoạn | Xa (roadmap) |

---

## VI. So Sánh Với Phiên Trước

| Metric | Phiên K (03-18) | Phiên N (03-18) | Δ |
|--------|-----------------|-----------------|---|
| Tests | 2,063 | 2,359 | +296 |
| LoC | ~82K | ~84K | +2K |
| Record types | 5 (Node/Edge/Alias/Amend/Kind) | 9 (+STM/Hebbian/KnowTree/Curve) | +4 |
| Memory persist | STM mất khi restart | Full persist/restore | **Major** |
| Agent orchestration | 0 messages (facade) | MessageRouter 7-phase wired | **Major** |
| Facade items | 6 | 0 (all resolved) | **-6** |
| Remaining gaps | 6 systemic | 4 (response, commands, FFI, types) | -2 |
| Module system | No | ModuleLoader + stdlib 10 modules | **New** |
| Spec files | 2 | 3 (+PLAN_REWRITE) | +1 |

---

## VII. Phép Ẩn Dụ

> HomeOS = cỗ máy với **động cơ Ferrari**, đã lắp **vô-lăng + hộp số**.
> Nền móng 90% xong. "Mặt tiền" cần sơn lại (response diversity).
> Tiếp theo: origin.olang tự đứng — cắt dây rốn khỏi Rust.

---

## VIII. Kết Luận

**Nền tảng:** Xuất sắc — 84K LoC Rust, zero deps, 2359 tests, kiến trúc sạch.
**QT8 enforced:** origin.olang = bộ nhớ duy nhất. RAM = cache. Full persist/restore.
**Agent hierarchy:** WIRED — Chiefs domain automation, Workers ISL, Router 7-phase pump.
**Còn thiếu:** Response diversity, command execution depth, compiler FFI stubs.
**Hướng đi:** PLAN_REWRITE — origin.olang = self-contained executable (7 giai đoạn).

---

*2026-03-18 · Phiên N · 2,359 tests · 9 record types · QT8 enforced*
