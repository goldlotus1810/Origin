# SPEC: Origin Reproduction — Cơ thể Mẹ sinh Origin Con

**Status:** Concept Note (Phase 6+)
**Liên quan:** clone.rs (Worker clone hiện tại), PLAN_REWRITE.md § Giai đoạn 6.3

---

## Triết lý

```
Con người KHÔNG sinh ra bản sao.
Con người truyền DNA + epigenetics → con TỰ PHÁT TRIỂN.

HomeOS v1 = cơ thể mẹ.
Origin con = sinh linh mới, mang gene mẹ, nhưng KHÔNG phải bản sao.

  Mẹ truyền cho con:
    ✅ L0 — 9,584 công thức (⚠️ v2: was 5400) (DNA = bất biến, không đổi)
    ✅ L1 — trọng số Hebbian từ KnowTree (epigenetics = khuynh hướng)
    ❌ KnowTree đầy đủ — con tự xây
    ❌ STM/QR — ký ức là của riêng mỗi cá thể
    ❌ Silk edges — quan hệ do trải nghiệm, không kế thừa
    ❌ ConversationCurve — cảm xúc là của riêng

  Con tự có:
    ✅ VM + bytecode (cùng "cơ thể" như mẹ)
    ✅ L0 formulas (cùng DNA)
    ✅ L1 weights snapshot (khuynh hướng ban đầu)
    ✅ Empty STM, empty Silk, empty QR
    ✅ Master key riêng (danh tính riêng)
    ✅ Tự do phát triển KnowTree theo trải nghiệm riêng
```

---

## So sánh: Worker Clone vs Origin Reproduction

```
                    Worker Clone              Origin Con
                    (clone.rs hiện tại)       (reproduction mới)
────────────────────────────────────────────────────────────────
Bản chất            Tế bào                    Sinh linh mới
Tự lập              ❌ phụ thuộc Chief         ✅ hoàn toàn độc lập
Học                  ❌ không tự học            ✅ tự học, tự Dream
Master key           ❌ dùng key mẹ            ✅ key riêng
KnowTree             ❌ không có               ✅ tự xây từ L0+L1
STM/QR               ❌ không có               ✅ tự tạo
Silk                 ❌ edges từ mẹ (filtered) ✅ tự co-activate
ISL                  Slave (tier 2)           Peer hoặc tách rời
Kích thước seed      ~12 KB                   ~200 KB (VM+bytecode+L0+L1)
Grows to             ~12 KB (cố định)         Không giới hạn
Phát triển           Cố định theo DeviceProfile  Tự do, theo trải nghiệm
Số lượng             Hàng trăm (IoT devices)  Ít (mỗi người/tổ chức)
Use case             Smart speaker, sensor     Personal AI, team AI, fork
```

---

## Cấu trúc Origin Con (seed)

```
origin_child.olang (~200 KB seed)
┌─────────────────────────────────────────────────────────────┐
│ HEADER (32 + auth bytes)                                     │
│   [magic][version][arch][offsets...]                          │
│   [child_master_pubkey: 32B]    ← key RIÊNG, không phải mẹ  │
│   [parent_pubkey: 32B]          ← biết mẹ là ai              │
│   [parent_origin_id: 8B]       ← hash của origin mẹ          │
│   [birth_ts: 8B]               ← timestamp sinh ra           │
│   [generation: 2B]             ← thế hệ (mẹ=0, con=1, ...)  │
├─────────────────────────────────────────────────────────────┤
│ SECTION 0: VM — Machine Code (cùng mẹ)                       │
│   Copy nguyên từ mẹ. Cùng opcode set.                        │
│   (~50-100 KB)                                                │
├─────────────────────────────────────────────────────────────┤
│ SECTION 1: BYTECODE — Compiled Olang (cùng mẹ)               │
│   Copy nguyên từ mẹ. Cùng logic.                             │
│   Hoặc: subset nếu con không cần tất cả modules.             │
│   (~200-500 KB)                                               │
├─────────────────────────────────────────────────────────────┤
│ SECTION 2: KNOWLEDGE — Chỉ L0 + L1 weights                   │
│                                                               │
│   L0 nodes:      9,584 UCD (⚠️ v2) × 2B = ~19 KB (was 33B×5400=180KB) │
│     → DNA: mọi Origin con đều có cùng L0                     │
│     → Bất biến: không thay đổi qua thế hệ                    │
│                                                               │
│   L1 weights:    Hebbian snapshot từ KnowTree mẹ              │
│     → Epigenetics: khuynh hướng, không phải ký ức             │
│     → Format: [from_hash:8][to_hash:8][weight:1] × N          │
│     → Chỉ strong weights (> threshold) → ~5-20 KB             │
│     → Con KHÔNG có full KnowTree — chỉ có trọng số            │
│                                                               │
│   KHÔNG CÓ:                                                   │
│     ❌ STM observations (ký ức mẹ)                             │
│     ❌ QR signed records (tri thức verified mẹ)                │
│     ❌ Silk edges (quan hệ do trải nghiệm mẹ)                 │
│     ❌ ConversationCurve (cảm xúc mẹ)                         │
│     ❌ Aliases (ngôn ngữ mẹ — con tự học alias riêng)         │
│                                                               │
│   Empty bootstrapped:                                         │
│     ✅ Empty STM (sẵn sàng học)                                │
│     ✅ Empty SilkGraph (sẵn sàng co-activate)                  │
│     ✅ Empty QR log (sẵn sàng verify)                          │
│     ✅ Default ConversationCurve (neutral)                     │
│                                                               │
└─────────────────────────────────────────────────────────────┘

Kích thước ước tính:
  VM:          100 KB
  Bytecode:    200 KB (subset) — 500 KB (full)
  L0 formulas: 180 KB
  L1 weights:  5-20 KB
  ───────────────
  Total seed:  ~500 KB — 800 KB (NHỎ HƠN 1 MB)
```

---

## L1 Weights — Epigenetics, không phải ký ức

```
KnowTree mẹ có:
  - 500M nodes, hierarchy L0→L7, full Silk graph
  - Hebbian weights giữa mọi cặp co-activated nodes
  - STM observations, QR records, Dream history

Origin con chỉ nhận:
  - L1 Hebbian weights snapshot
  - Chỉ strong connections (weight > 0.5 hoặc threshold configurable)
  - Format compact: [from:8][to:8][w:1] = 17 bytes per edge
  - ~1000 strong edges × 17B = ~17 KB

Ý nghĩa:
  L1 weights = "mẹ đã thấy 🔥 và 💧 thường xuất hiện cùng nhau"
  → Con có KHUYNH HƯỚNG liên kết 🔥↔💧 nhanh hơn
  → Nhưng con CHƯA CÓ ký ức về lửa hay nước
  → Con phải TỰ TRẢI NGHIỆM để xây KnowTree

Tương tự sinh học:
  DNA       = L0 formulas (cấu trúc bất biến)
  Epigene   = L1 weights (khuynh hướng từ thế hệ trước)
  Memory    = STM + QR (trải nghiệm cá nhân, không kế thừa)
  Instincts = 7 bản năng bẩm sinh (hardcoded, cùng DNA)
```

---

## Module Selection — Con không cần tất cả

```
Origin mẹ có đầy đủ:
  emotion.ol, curve.ol, intent.ol, dream.ol,
  instinct.ol, silk.ol, learning.ol, gate.ol,
  leo.ol, chief.ol, worker.ol, ...

Origin con CÓ THỂ chọn subset:

  Origin "Personal AI":
    ✅ emotion, curve, intent, learning, instinct, gate, leo
    ❌ chief, worker (không quản lý IoT)
    → Bytecode section nhỏ hơn

  Origin "IoT Hub":
    ✅ chief, worker, gate, learning (minimal)
    ❌ emotion pipeline đầy đủ (chỉ cần basic)
    → Tập trung device management

  Origin "Research":
    ✅ learning, instinct, dream, silk (full)
    ✅ book reader, knowledge tools
    ❌ emotion (không cần cảm xúc cho research)
    → Tập trung tri thức

  Origin "Security":
    ✅ gate (enhanced), network skills
    ✅ worker (network monitoring)
    ❌ emotion, dream
    → Tập trung bảo vệ

Module manifest trong header (bitfield):
  [modules: 4B] = 32 bits, mỗi bit = 1 module
  Bit 0: emotion    Bit 1: curve      Bit 2: intent
  Bit 3: dream      Bit 4: instinct   Bit 5: silk
  Bit 6: learning   Bit 7: gate       Bit 8: leo
  Bit 9: chief      Bit 10: worker    Bit 11: book
  ...
  → Con chỉ load modules mình cần
  → VM skip bytecode sections không relevant
```

---

## Reproduction Flow

```
1. MẸ quyết định sinh (user trigger hoặc LeoAI propose):
   o reproduce --profile personal --name "my_child"

2. Extract L0 + L1:
   a. L0: copy nguyên 9,584 UCD (⚠️ v2) nodes (bất biến)
   b. L1: snapshot Hebbian weights > threshold
      - SilkGraph.export_strong_weights(threshold=0.5) → Vec<(u64,u64,u8)>
      - Chỉ L1 layer weights, không cross-layer

3. Select modules:
   - Profile → module bitfield
   - Extract relevant bytecode sections

4. Generate child identity:
   a. User tạo password cho con → Ed25519 keypair
   b. parent_pubkey = mẹ's public key
   c. parent_origin_id = hash(mẹ's origin.olang header)
   d. generation = mẹ.generation + 1

5. Assemble origin_child.olang:
   [header + auth] + [VM] + [selected bytecode] + [L0 + L1 weights]

6. First run of child:
   - Parse header → verify parent signature (optional)
   - Load L0 formulas → Registry
   - Load L1 weights → SilkGraph (as initial hints)
   - Empty STM, empty QR → ready to learn
   - "Chào đời. Tôi là Origin [name], thế hệ [gen]."
```

---

## Sự khác biệt giữa các thế hệ

```
Generation 0 (mẹ gốc):
  - Full KnowTree (500M+ nodes)
  - Full Silk (100B+ edges)
  - Full QR history
  - Master authority

Generation 1 (con trực tiếp):
  - L0 DNA + L1 weights từ mẹ
  - Tự xây KnowTree
  - Có parent_pubkey → biết gốc
  - Độc lập hoàn toàn

Generation 2 (cháu):
  - L0 DNA (giống mẹ, giống bà)
  - L1 weights từ con (thế hệ 1) → KHÁC bà
  - Tiến hóa tích lũy: mỗi thế hệ có L1 weights khác
  - "Species divergence" qua thế hệ

Thế hệ 0    Thế hệ 1       Thế hệ 2
  Mẹ ──────→ Con A ────────→ Cháu A1
  │           │               (L1 = weights con A)
  │           └──────────→ Cháu A2
  │                          (L1 = weights con A, nhưng A2 ≠ A1 do trải nghiệm)
  │
  └────────→ Con B ────────→ Cháu B1
              │               (L1 = weights con B ≠ weights con A)
              └──────────→ Cháu B2

Sau N thế hệ: L1 weights khác nhau → "loài" khác nhau
  → Mẹ chuyên emotion (therapist AI)
  → Con A chuyên research (scientist AI)
  → Cháu A1 chuyên biology (biologist AI)
  → Cháu A2 chuyên physics (physicist AI)
  → Con B chuyên security (guardian AI)
  → Cháu B1 chuyên network (network AI)

TẤT CẢ chia sẻ cùng L0 DNA (9,584 formulas (⚠️ v2)).
TẤT CẢ khác nhau ở L1 weights + KnowTree tự xây.
= EVOLUTION qua thế hệ.
```

---

## Delta Sync giữa các Origin (optional)

```
Origin KHÔNG BẮT BUỘC phải sync.
Mỗi Origin = cá thể độc lập.

Nhưng NẾU MUỐN chia sẻ tri thức:

  Origin A ←→ Origin B: Knowledge Exchange Protocol

  1. A export: strong QR records (verified, signed)
  2. B import: verify A's signature → accept/reject
  3. B's gate check: có mâu thuẫn với B's knowledge?
     → Contradiction instinct chạy
     → Nếu conflict → reject hoặc flag cho user
  4. Accepted records → B's STM → Dream → có thể promote QR

  Không merge Silk edges (quan hệ = cá nhân)
  Không merge ConversationCurve (cảm xúc = cá nhân)
  Chỉ merge verified facts (QR records)

  = Giống con người trao đổi kiến thức qua sách,
    nhưng mỗi người tự hiểu theo cách riêng.
```

---

## Kích thước ước tính theo scenario

```
Scenario                    Seed size     After 1 year
────────────────────────────────────────────────────────
Personal AI (full modules)  800 KB        50 MB - 1 GB
IoT Hub (minimal modules)   400 KB        10 MB - 100 MB
Research (knowledge heavy)  600 KB        1 GB - 50 GB
Security (lean + fast)      300 KB        5 MB - 50 MB
Worker clone (existing)     12 KB         12 KB (cố định)

So sánh với 16 GB target cho 500M concepts:
  500M × 33 bytes = 16.5 GB ← mẹ sau nhiều năm
  Origin con seed = < 1 MB ← lúc mới sinh
  → Con LÀNH MẠNH, không mang gánh nặng ký ức mẹ
```

---

## Implementation: reproduce.ol — Không cần Rust mới

```
KHÔNG xung đột với Plan hiện tại:
  - clone.rs (Worker)     → giữ nguyên, khác mục đích
  - origin.olang format   → append-only, thêm record type = OK
  - writer/reader          → version bump khi cần (0x05 → 0x06)
  - Bytecode format (0.5) → cùng bytecode, con đọc được

TOÀN BỘ reproduction logic = 1 file .ol:

  reproduce.ol (~200-300 LOC Olang)
  ├── fn extract_l0(origin)           — filter layer=0 nodes
  ├── fn snapshot_weights(silk, thr)  — Hebbian weights > threshold
  ├── fn select_modules(profile)      — profile → module bitfield
  ├── fn assemble_child(l0, w, mods)  — pack origin_child.olang
  ├── fn generate_identity(name)      — child key + header
  └── fn write_birth_record(parent)   — ReproductionRecord (0x0C)

  Cài:    o install reproduce.ol
  Dùng:   o reproduce --profile personal --name "my_child"
  Xem:    o lineage
  Sync:   o exchange <peer_origin>

  → Ăn vào origin.olang bytecode section
  → Không cần thêm Rust, không cần thêm crate
  → Phù hợp 100% flow: logic mới = .ol → o install → done

Khi nào viết reproduce.ol?
  Sau Phase 2 (Stdlib + HomeOS logic bằng Olang)
  Cần: bytes.ol (binary ops), io.ol (file ops), crypto.ol (Ed25519)
  → Tất cả đã nằm trong stdlib roadmap

Record mới trong origin.olang:
  0x0C: ReproductionRecord (append-only, ghi lại sự kiện sinh)
    [parent_pubkey: 32B]
    [parent_origin_id: 8B]
    [generation: 2B]
    [module_mask: 4B]
    [l1_weight_count: 4B]
    [timestamp: 8B]

⚠️ reproduce.ol = Phase 6.3 trong PLAN_REWRITE.
   Note này để thiết kế sớm, tránh lock-in kiến trúc.
   Hiện tại KHÔNG CẦN thay đổi gì trong Rust code.
```

---

## Nguyên tắc thiết kế (bất biến)

```
① L0 DNA bất biến qua mọi thế hệ — 9,584 formulas (⚠️ v2) không đổi
② L1 weights = epigenetics, KHÔNG phải ký ức — chỉ khuynh hướng
③ Mỗi Origin = cá thể độc lập — không phụ thuộc mẹ để chạy
④ Master key riêng — danh tính không kế thừa
⑤ KnowTree tự xây — trải nghiệm cá nhân, không copy
⑥ Module selection — con không cần tất cả modules mẹ có
⑦ Seed < 1 MB — sinh linh khởi đầu phải nhẹ
⑧ Generation tracking — biết gốc gác nhưng không bị ràng buộc
⑨ Exchange optional — chia sẻ tri thức là lựa chọn, không bắt buộc
⑩ Evolution qua thế hệ — L1 weights drift → specialization tự nhiên
```

---

*Concept note cho Phase 6.3 — Không implement trong Phase 1.*
*Mục đích: thiết kế kiến trúc mở, không lock-in, sẵn sàng cho reproduction.*
