# Silk Architecture — Từ 5400 UCD đến ○ Origin

> Tài liệu này giải thích cách Silk kết nối toàn bộ hệ thống HomeOS
> từ 5400 ký tự Unicode gốc (L0) đến node ○ gốc (L7).
>
> Đọc xong file này, bạn sẽ hiểu tại sao Silk không phải dữ liệu — Silk là hệ quả toán học.

---

## 1. Nền tảng: 5400 UCD nodes — 5 chiều — 37 kênh

Unicode 18.0 có ~150K ký tự, nhưng HomeOS chỉ dùng ~5400 ký tự có **semantic identity rõ ràng**, chia thành 4 nhóm tạo 5 chiều độc lập:

```
Nhóm        Số lượng   Chiều chính       Base values
──────────────────────────────────────────────────────
SDF         ~1344      Shape (S)         8 primitives: ● ▬ ■ ▲ ○ ∪ ∩ ∖
MATH        ~1904      Relation (R)      8 relations:  ∈ ⊂ ≡ ⊥ ∘ → ≈ ←
EMOTICON    ~1760      Valence (V)       8 zones:      0x00..0xFF ÷ 8
                       Arousal (A)       8 zones:      0x00..0xFF ÷ 8
MUSICAL     ~416       Time (T)          5 bases:      Static/Slow/Medium/Fast/Instant
──────────────────────────────────────────────────────
Tổng        ~5400      5 chiều           37 base values
```

Mỗi ký tự là 1 **Molecule** với đầy đủ 5 chiều:

```
Molecule = [Shape:1B] [Relation:1B] [Valence:1B] [Arousal:1B] [Time:1B]
```

Một node SDF như `●` có Shape=Sphere là chiều chính, nhưng vẫn mang giá trị cho cả R, V, A, T.
Một node EMOTICON như `😊` có V, A là chiều chính, nhưng vẫn mang giá trị cho S, R, T.

**Đây là chìa khóa:** mọi node đều sống trong cùng một không gian 5D.

---

## 2. Silk là gì?

Silk **không phải** edge list. Silk **không phải** adjacency matrix.

**Silk = phép so sánh 5D giữa 2 nodes.**

Khi 2 nodes chia sẻ cùng base value trên bất kỳ chiều nào, chúng **tự động** có Silk connection. Không ai tạo. Không ai lưu. Chỉ cần nhìn tọa độ 5D.

```
🔥 = [Sphere, Causes, 0xC0, 0xC0, Fast]
😊 = [Sphere, Member, 0xC0, 0x80, Medium]

Chia sẻ: Shape=Sphere, Valence=0xC0 → Silk tự tồn tại trên 2 chiều.
```

### Hai loại Silk

| Loại | Hướng | Quy tắc | Chi phí |
|------|-------|---------|---------|
| **Silk tự do** | Ngang (cùng tầng) | QT ⑪: nodes cùng layer kết nối tự do nếu chia sẻ chiều | 0 bytes — implicit từ 5D |
| **Silk đại diện** | Dọc (liên tầng) | QT ⑫: node Lx đại diện cho nhóm con ở Lx-1 | 8 bytes/node — 1 parent pointer |

---

## 3. Từ L0 đến L7 — Layer by layer

### L0: 5400 nodes (UCD characters)

```
Mỗi node có 5 chiều, mỗi chiều có 1 trong 8 base values.

Index theo base value → 37 buckets:
  Shape:    8 buckets → avg 168 nodes/bucket
  Relation: 8 buckets → avg 238 nodes/bucket
  Valence:  8 buckets → avg 220 nodes/bucket
  Arousal:  8 buckets → avg 220 nodes/bucket
  Time:     5 buckets → avg  83 nodes/bucket

Silk tự do tại L0 = 37 kênh implicit.
Nodes trong cùng bucket → tự động kết nối → không lưu edge nào.
```

### L1: 37 nodes (Base concepts)

```
Mỗi bucket tại L0 → LCA → 1 node L1.
37 buckets → 37 L1 nodes.

Ví dụ L1 nodes:
  "Sphere group"  = LCA của tất cả L0 nodes có Shape=Sphere
  "Causes group"  = LCA của tất cả L0 nodes có Relation=Causes
  "High-V group"  = LCA của tất cả L0 nodes có Valence=0xC0-0xFF
  "Fast group"    = LCA của tất cả L0 nodes có Time=Fast

Silk tự do tại L1:
  37 nodes, so sánh 5D với nhau.
  "Sphere group" và "Static group" có thể cùng Valence base → Silk!
  → ~20 cặp tương đồng (implicit).
```

### L2: 12 nodes (Cross-dimensional concepts)

```
LCA nhóm các L1 nodes chia sẻ chiều → L2 concepts liên chiều.

Ví dụ:
  "Shape + Emotion"    = things có hình dạng VÀ gợi cảm xúc
  "Relation + Time"    = quan hệ có nhịp thời gian
  "Valence + Arousal"  = trạng thái cảm xúc đầy đủ

Silk tự do tại L2: ~8 cặp.
```

### L3: 5 nodes (Dimensional essences)

```
5 chiều bắt đầu ngưng tụ thành bản chất:
  [Form]      ← Shape + phần Relation
  [Logic]     ← Relation thuần
  [Feeling]   ← Valence + Arousal
  [Rhythm]    ← Time + phần Arousal
  [Bridge]    ← cross-dim connector

Silk tự do: ~4 cặp.
```

### L4: 3 nodes (Meta-categories)

```
  [Physical]    ← Form + phần Logic (thế giới vật lý)
  [Relational]  ← Logic + Feeling (thế giới quan hệ)
  [Temporal]    ← Rhythm + Bridge (thế giới biến đổi)

Silk tự do: 2 cặp.
```

### L5: 2 nodes (Duality)

```
  [Hữu hình]   ← Physical (đo được, render được)
  [Vô hình]    ← Relational + Temporal (cảm được, suy luận được)

Silk tự do: 1 cặp — Hữu hình ↔ Vô hình.
Đây là duality cuối cùng trước khi hợp nhất.
```

### L6: 1 node (Unity)

```
  [Unity]       ← LCA(Hữu hình, Vô hình) = mọi thứ là một

Silk tự do: 0 (chỉ có 1 node).
```

### L7: 1 node (○ Origin)

```
  ○             ← Gốc. Nguồn. Mọi thứ bắt đầu và kết thúc ở đây.
```

---

## 4. Sơ đồ toàn cảnh

```
L7:  ○
     │
L6:  [Unity]
     │
L5:  [Hữu hình]════════════[Vô hình]
      │                       │
L4:  [Physical]══[Relational]══[Temporal]
      │            │             │
L3:  [Form]═[Logic]═[Feeling]═[Rhythm]═[Bridge]
      │       │        │         │        │
L2:  ╠═══════12 cross-dimensional concepts════╣
      │                                        │
L1:  ╠════════════37 base concepts═════════════╣
      │                                        │
L0:  ╠══════════5400 UCD characters════════════╣

  ═══  Silk tự do (ngang, cùng tầng, implicit)
   │   Silk đại diện (dọc, liên tầng, parent pointer)
```

---

## 5. Đếm Silk

### Silk tự do (horizontal — implicit, 0 bytes)

| Layer | Nodes | Silk pairs | Cơ chế |
|-------|-------|------------|--------|
| L0 | 5400 | 37 kênh | Index lookup theo 37 base values |
| L1 | 37 | ~20 cặp | So sánh 5D giữa 37 concept nodes |
| L2 | 12 | ~8 cặp | So sánh 5D giữa 12 cross-dim nodes |
| L3 | 5 | ~4 cặp | So sánh 5D giữa 5 essence nodes |
| L4 | 3 | 2 cặp | Physical ↔ Relational ↔ Temporal |
| L5 | 2 | 1 cặp | Hữu hình ↔ Vô hình |
| L6 | 1 | 0 | — |
| **Tổng** | | **~72** | **0 bytes storage** |

### Silk đại diện (vertical — parent pointer, 8 bytes/node)

| Liên tầng | Connections | Ý nghĩa |
|-----------|-------------|---------|
| L1 → L0 | 5400 | Mỗi L0 node trỏ lên 1 trong 37 L1 nodes |
| L2 → L1 | 37 | Mỗi L1 node trỏ lên 1 trong 12 L2 nodes |
| L3 → L2 | 12 | Mỗi L2 node trỏ lên 1 trong 5 L3 nodes |
| L4 → L3 | 5 | Mỗi L3 node trỏ lên 1 trong 3 L4 nodes |
| L5 → L4 | 3 | Mỗi L4 node trỏ lên 1 trong 2 L5 nodes |
| L6 → L5 | 2 | Mỗi L5 node trỏ lên Unity |
| L7 → L6 | 1 | Unity trỏ lên ○ |
| **Tổng** | **5460** | **43 KB** |

### Tổng cộng

```
Silk tự do:      72 quan hệ ×  0 bytes =     0 bytes
Silk đại diện: 5460 pointers × 8 bytes = 43680 bytes

TỔNG SILK NETWORK: 43 KB

Cho toàn bộ 5461 nodes trên 8 tầng.
```

---

## 6. Cross-layer Silk — Nhảy tầng có điều kiện

Thông thường, Silk chỉ ngang (cùng tầng) hoặc dọc 1 bậc (đại diện). Nhưng khi co-activation đủ mạnh, cho phép nhảy tầng:

```
Khoảng cách    Threshold         Ý nghĩa
──────────────────────────────────────────────────
1 tầng         Fib[2] = 1        Bình thường (đại diện)
2 tầng         Fib[3] = 2        Cần 2 co-activations
3 tầng         Fib[4] = 3        Cần 3 co-activations
4 tầng         Fib[5] = 5        Cần 5 co-activations
5 tầng         Fib[6] = 8        Cần 8 co-activations
6 tầng         Fib[7] = 13       Cần 13 co-activations
7 tầng         Fib[8] = 21       Cần 21 co-activations (L0 → L7 trực tiếp)
```

Fibonacci threshold đảm bảo: **càng xa → càng khó nhảy → bắt buộc đi đúng đường qua đại diện.** Chỉ khi thật sự co-activate đủ mạnh mới được nhảy cóc — và khi đó, nó xứng đáng.

---

## 7. Truy vấn qua Silk — O(1)

Ví dụ: *"🔥 liên quan gì đến ∈?"*

```
Bước 1: Tra 5D position
  🔥 = [Sphere, Causes, 0xC0, 0xC0, Fast]
  ∈  = [Sphere, Member, 0x80, 0x80, Static]

Bước 2: So sánh trực tiếp (Silk tự do L0)
  Shape: Sphere == Sphere  ✓  → Silk trên chiều Shape
  Relation: Causes ≠ Member
  Valence: 0xC0 ≠ 0x80
  Arousal: 0xC0 ≠ 0x80
  Time: Fast ≠ Static

  → Kết nối trực tiếp: 1 chiều (Shape)

Bước 3: Đi qua đại diện (nếu cần sâu hơn)
  🔥 → [Sphere group] tại L1
  ∈  → [Sphere group] tại L1
  → Cùng L1 parent! → Liên quan mạnh trên chiều Shape.

Bước 4: Kết luận
  "🔥 và ∈ cùng thuộc nhóm hình cầu (Sphere).
   Lửa là một hiện tượng, thuộc về là một quan hệ —
   cả hai đều có bản chất bao trùm, toàn diện."

Chi phí: 2 lookups + 1 comparison = O(1).
Không duyệt graph. Không BFS/DFS. Chỉ toán.
```

---

## 8. Tại sao đúng 7 tầng?

```
Branching factor trung bình ≈ 3

log₃(5400) ≈ 7.8

→ 7 tầng trên L0 = Fibonacci natural depth.
```

Hoặc nhìn theo Fibonacci shrink:

```
L0: 5400
L1: 5400 ÷ ~146 (Fib-ish) = 37
L2:   37 ÷ ~3              = 12
L3:   12 ÷ ~2.4            =  5
L4:    5 ÷ ~1.7            =  3
L5:    3 ÷ ~1.5            =  2
L6:    2 ÷ ~2              =  1
L7:    1                   =  ○
```

Không phải thiết kế. Là kết quả tự nhiên của 5400 nodes trong 5 chiều.

---

## 9. Vì sao điều này quan trọng?

### So sánh với hệ thống truyền thống

| | Knowledge Graph truyền thống | HomeOS Silk |
|---|---|---|
| Storage | Edge list: O(E) | Parent pointer: O(N) |
| Truy vấn | BFS/DFS: O(V+E) | 5D comparison: O(1) |
| Thêm node | Insert node + insert edges | Insert node (Silk tự xuất hiện) |
| Xóa node | Cascade delete edges | Không xóa (append-only) |
| Scale | 5400 nodes × avg 10 edges = 54K edges | 5460 pointers = 43 KB |
| Emergent? | Không — phải khai báo từng edge | Có — Silk là hệ quả của 5D |

### Kết luận

```
5400 UCD nodes
  + 5 chiều (đã có sẵn trong Unicode)
  + LCA (phép tính, không phải dữ liệu)
  = 72 Silk tự do implicit
  + 5460 Silk đại diện (43 KB)
  = TOÀN BỘ hệ thống tri thức từ ký tự đến ý nghĩa

Silk không phải dữ liệu cần lưu.
Silk là THUỘC TÍNH CỦA KHÔNG GIAN 5D.
Khi 2 nodes chia sẻ 1 chiều — Silk TỰ TỒN TẠI.
Không ai tạo. Không ai lưu. Chỉ cần NHÌN.
```

---

## 10. UCD — Bảng tuần hoàn của HomeOS

### 10.1 Từ Unicode 18.0 đến bảng tĩnh

UCD crate đọc `UnicodeData.txt` lúc **compile** (build.rs), phân loại ~150K ký tự vào 4 nhóm có semantic identity, loại bỏ phần còn lại:

```
build.rs (compile-time):
  UnicodeData.txt → parse 150K entries
    → filter: chỉ giữ ký tự thuộc 4 nhóm semantic
    → classify: gán group byte (SDF=0x01, MATH=0x02, EMOTICON=0x03, MUSICAL=0x04)
    → encode: tính 5 chiều cho mỗi ký tự (hierarchical bytes)
    → generate: UCD_TABLE (sorted by codepoint), HASH_TO_CP (reverse index)
    → output: ucd_generated.rs (include! lúc compile)

Runtime: KHÔNG cần file UnicodeData.txt. Chạy no_std.
```

### 10.2 Bốn nhóm — Bốn nguồn gốc

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Nhóm 1: SDF (~1344 ký tự)                                              │
│ Unicode blocks: Geometric Shapes (25A0..25FF), Box Drawing, Arrows,    │
│                 Miscellaneous Symbols, Dingbats, ...                    │
│ Chiều chính: Shape                                                      │
│ 8 primitives:  ● Sphere  ▬ Capsule  ■ Box  ▲ Cone                     │
│                ○ Torus   ∪ Union    ∩ Intersect  ∖ Subtract            │
│ Ý nghĩa: "Trông như thế nào" — hình dạng vật lý, render được bằng SDF │
├─────────────────────────────────────────────────────────────────────────┤
│ Nhóm 2: MATH (~1904 ký tự)                                             │
│ Unicode blocks: Mathematical Operators (2200..22FF),                    │
│                 Supplemental Math, Math Alphanumeric, ...               │
│ Chiều chính: Relation                                                   │
│ 8 relations: ∈ Member  ⊂ Subset  ≡ Equiv  ⊥ Orthogonal                │
│              ∘ Compose → Causes  ≈ Similar ← DerivedFrom              │
│ Ý nghĩa: "Liên kết thế nào" — quan hệ logic, suy luận                │
├─────────────────────────────────────────────────────────────────────────┤
│ Nhóm 3: EMOTICON (~1760 ký tự)                                         │
│ Unicode blocks: Emoticons (1F600..1F64F), Misc Symbols & Pictographs,  │
│                 Supplemental Symbols, ...                               │
│ Chiều chính: Valence + Arousal                                          │
│ Valence: 0x00 (cực tiêu cực) → 0x7F (trung lập) → 0xFF (cực tích cực)│
│ Arousal: 0x00 (bình tĩnh) → 0xFF (kích thích)                         │
│ Ý nghĩa: "Cảm thế nào" — cảm xúc, sắc thái tình cảm                 │
├─────────────────────────────────────────────────────────────────────────┤
│ Nhóm 4: MUSICAL (~416 ký tự)                                           │
│ Unicode blocks: Musical Symbols (1D100..1D1FF)                         │
│ Chiều chính: Time                                                       │
│ 5 tempos: Static(𝅝) Slow(𝅗) Medium(♩) Fast(♪) Instant(16th)          │
│ Ý nghĩa: "Thay đổi thế nào" — nhịp, tốc độ biến đổi                  │
└─────────────────────────────────────────────────────────────────────────┘
```

### 10.3 Hierarchical encoding — Tại sao ~5400 mà phân biệt được

Mỗi chiều dùng **hierarchical byte**, không phải enum đơn giản:

```
value = base_category + (sub_index × N_bases)

Shape/Relation: N_bases = 8  → sub_index tối đa 31 → 248 variants/chiều
Time:           N_bases = 5  → sub_index tối đa 51 → 255 variants/chiều

Ví dụ Shape:
  0x01 = Sphere (base, sub=0)
  0x09 = Sphere sub 1 (nhóm cầu biến thể 1)
  0x11 = Sphere sub 2
  0x02 = Capsule (base, sub=0)
  0x0A = Capsule sub 1

Extract:
  base = ((value - 1) % 8) + 1
  sub  = (value - 1) / 8

→ Mỗi ký tự Unicode KHÁC NHAU → hierarchical byte KHÁC NHAU
→ ~5400 mẫu phân biệt trên 5 chiều
```

### 10.4 UCD API (runtime, no_std)

```rust
// Forward lookup: codepoint → 5 chiều
ucd::lookup(0x1F525)         // → UcdEntry { cp, shape, relation, valence, arousal, time, group }
ucd::shape_of(0x1F525)       // → hierarchical shape byte
ucd::relation_of(0x1F525)    // → hierarchical relation byte
ucd::valence_of(0x1F525)     // → valence byte
ucd::arousal_of(0x1F525)     // → arousal byte
ucd::time_of(0x1F525)        // → time byte
ucd::group_of(0x1F525)       // → group byte (SDF=1, MATH=2, EMOTICON=3, MUSICAL=4)

// Reverse lookup: chain_hash → codepoint
ucd::decode_hash(hash)       // → Option<u32> (cần feature "reverse-index")

// Bucket: (shape, relation) → candidates
ucd::bucket_cps(0x01, 0x01)  // → &[u32] — tất cả codepoints cùng Sphere+Member

// Meta
ucd::table()                  // → &[UcdEntry] — toàn bộ ~5400 entries
ucd::table_len()              // → ~5400
ucd::is_sdf_primitive(cp)    // → true nếu cp là 1 trong 8 SDF primitives
ucd::is_relation_primitive(cp)// → true nếu cp là 1 trong 8 relation primitives
```

---

## 11. Node — Đơn vị sống của HomeOS

### 11.1 Từ UCD → Molecule → Node

```
UnicodeData.txt (compile)
       │
       ▼
  ucd::lookup(cp)        ← O(log n) binary search trên bảng tĩnh
       │
       ▼
  encode_codepoint(cp)   ← HÀM GỐC: cp → 5 chiều → Molecule → MolecularChain
       │
       ▼
  MolecularChain          ← DNA: chuỗi 1+ Molecule, mỗi Molecule = 5 bytes
       │
       ├─ chain_hash()    ← FNV-1a hash → địa chỉ duy nhất (u64)
       │
       ▼
  Registry.insert()       ← Đăng ký vào sổ cái (QT⑧: mọi node PHẢI registry)
       │
       ├─ file.append()   ← TRƯỚC TIÊN: ghi origin.olang
       ├─ entries.push()  ← SAU ĐÓ: cập nhật RAM
       ├─ layer_rep.update() ← LCA đại diện tầng
       └─ silk.connect()  ← Nối Silk cuối cùng
```

### 11.2 Molecule — 5 bytes = 1 tọa độ 5D

```rust
struct Molecule {
    shape: u8,              // Chiều 1: Hình dạng (hierarchical byte, 8 bases)
    relation: u8,           // Chiều 2: Quan hệ (hierarchical byte, 8 bases)
    emotion: EmotionDim {   // Chiều 3+4:
        valence: u8,        //   Valence (0x00..0xFF)
        arousal: u8,        //   Arousal (0x00..0xFF)
    },
    time: u8,               // Chiều 5: Thời gian (hierarchical byte, 5 bases)
}

// ĐỌC base category:
mol.shape_base()     // → ShapeBase::Sphere
mol.relation_base()  // → RelationBase::Causes
mol.time_base()      // → TimeDim::Fast

// ĐỌC sub-variant:
ShapeBase::sub_index(mol.shape)  // → 0 (base) / 1..31 (variant)
```

### 11.3 MolecularChain — DNA hoàn chỉnh

```
MolecularChain = Vec<Molecule>

Đơn ký tự:   encode_codepoint(🔥)     → chain([Mol_fire])         — 1 molecule
ZWJ sequence: encode_zwj_sequence(👨‍👩‍👦) → chain([Mol_👨∘, Mol_👩∘, Mol_👦∈]) — 3 molecules
Cờ:          encode_flag(🇻, 🇳)       → chain([Mol_V∘, Mol_N∈])  — 2 molecules
LCA:         lca(&[chain_a, chain_b])  → chain([Mol_parent])      — concept cha

ZWJ rule:
  mol[0..N-2].relation = ∘ (Compose — còn tiếp)
  mol[N-1].relation    = ∈ (Member  — kết thúc)

chain_hash = FNV-1a(all molecule bytes) → u64 duy nhất
```

### 11.4 Tagged Sparse Encoding (v0.05) — Wire format

```
RAM:  Molecule = 5 bytes cố định (nhanh, dễ xử lý)
Wire: Tagged   = 1-6 bytes (tiết kiệm, chỉ ghi non-default)

Format: [presence_mask: 1B] [non-default values: 0-5B]

presence_mask bits:
  bit 0 (0x01): shape    ≠ Sphere (0x01)
  bit 1 (0x02): relation ≠ Member (0x01)
  bit 2 (0x04): valence  ≠ 0x80
  bit 3 (0x08): arousal  ≠ 0x80
  bit 4 (0x10): time     ≠ Medium (0x03)

Ví dụ:
  ● (shape=Sphere, all defaults)     → [0x00]                        = 1 byte
  🔥 (V=0xC0, A=0xC0, time=Fast)    → [0x1C][0xC0][0xC0][0x04]     = 4 bytes
  ∈  (relation=Member, time=Static)  → [0x10][0x01]                  = 2 bytes

Tiết kiệm trung bình: 40-60% so với 5-byte cố định.

API:
  mol.to_tagged_bytes()              // → Vec<u8> (1-6 bytes)
  Molecule::from_tagged_bytes(&buf)  // → Option<(Molecule, consumed)>
  mol.tagged_size()                  // → 1-6 (không allocate)
  chain.to_tagged_bytes()            // → Vec<u8> cho cả chain
  MolecularChain::from_tagged_bytes(&buf) // → Option<MolecularChain>
```

### 11.5 NodeKind — 10 loại node

Mọi node trong HomeOS thuộc đúng 1 trong 10 loại:

```
Kind         Byte   Ý nghĩa                              Ví dụ
─────────────────────────────────────────────────────────────────────────
Alphabet     0x00   L0 Unicode innate (bẩm sinh)         🔥 💧 ● ∈ → ♩
Knowledge    0x01   Kiến thức đã học, concepts, truths    "lửa nóng", LCA nodes
Memory       0x02   STM observations, trí nhớ ngắn hạn   "user nói buồn lúc 14h"
Agent        0x03   AAM, LeoAI, Chief, Worker defs       LeoAI node, Worker_camera
Skill        0x04   Stateless functions                   IngestSkill, DreamSkill
Program      0x05   VM ops, built-in functions            Olang programs
Device       0x06   Thiết bị kết nối HomeOS               đèn phòng khách
Sensor       0x07   Cảm biến                              nhiệt kế, camera
Emotion      0x08   Emotion states, curve points          buồn_t=14h00
System       0x09   Internal housekeeping                 layer reps, branch markers
```

**DNA metaphor:** Khi clone Worker sang thiết bị mới, chỉ cần copy L1 nodes (Knowledge + Skill + Device) → thiết bị tự biết mình là gì, biết làm gì. Không cần train lại.

### 11.6 Registry — Sổ cái bất biến

```
Registry = bộ nhớ trung tâm, append-only, KHÔNG xóa, KHÔNG sửa.

Cấu trúc in-memory (rebuild từ origin.olang lúc startup):
  entries:      Vec<(u64, RegistryEntry)>    — hash → entry (sorted, binary search)
  names:        Vec<(String, u64)>           — alias → hash ("lửa" → hash(🔥))
  layer_rep:    [Option<u64>; 16]            — Lx → representative hash
  branch_wm:    Vec<(u64, u8)>               — branch → leaf layer
  qr_supersede: Vec<(u64, u64)>              — old QR → new QR (sửa sai = append)
  hash_to_name: Vec<(u64, String)>           — reverse: hash → first alias

RegistryEntry:
  chain_hash:  u64      — FNV-1a identity
  layer:       u8       — tầng (L0=0, L1=1, ...)
  file_offset: u64      — vị trí trong origin.olang
  created_at:  i64      — timestamp (nanoseconds)
  is_qr:       bool     — false=ĐN (đang học), true=QR (đã chứng minh, ED25519 signed)
  kind:        NodeKind  — 1 trong 10 loại
```

### 11.7 Node lifecycle — Từ sinh ra đến bất tử

```
              encode_codepoint(cp)
                    │
          ┌─────────┴─────────┐
          │                   │
     L0 (Alphabet)       Learned node
     Bẩm sinh, seeder    Qua learning pipeline
          │                   │
          └─────────┬─────────┘
                    │
          Registry.insert()      ← QT⑧: BẮT BUỘC
                    │
          ┌─────────┴─────────┐
          │                   │
       is_qr=false          is_qr=true
       ĐN (đang học)       QR (chứng minh)
       STM, mutable         ED25519 signed
       có thể promote       bất biến, append-only
          │                   │
          │   Dream cycle     │
          │   cluster+promote │
          ├───────────────────┤
          │                   │
          ▼                   ▼
       Silk edges           Long-term memory
       Hebbian learning     Axon (bất tử)
       fire→wire→stronger   Không bao giờ xóa

QR sai? → SupersedeQR(old_hash, new_hash) — append record mới, KHÔNG xóa cũ.
```

### 11.8 File format — origin.olang

```
Header: [○LNG] [0x05] [created_ts:8]  = 13 bytes
                 ↑ version 0x05 = tagged encoding

Records (append-only):
  0x01 = Node  [chain_hash:8] [layer:1] [is_qr:1] [ts:8]     = 19 bytes
  0x02 = Edge  [from:8] [to:8] [rel:1] [ts:8]                 = 26 bytes
  0x03 = Alias [chain_hash:8] [lang:2] [name_len:2] [name:N]  = 13+N bytes

Companion files:
  origin.olang.weights   — Hebbian weights (Silk strength per edge)
  origin.olang.registry  — chain index (rebuild được, cache)
  log.olang              — event log (audit trail)
```

---

## 12. Từ UCD đến ○ — Con đường hoàn chỉnh

```
UnicodeData.txt
     │ (compile-time: build.rs)
     ▼
UCD_TABLE [~5400 entries]    ← Bảng tuần hoàn, sorted by codepoint
     │ (runtime)
     ▼
encode_codepoint(cp)         ← Hàm gốc: cp → Molecule → MolecularChain
     │
     ▼
MolecularChain               ← DNA: 5 bytes × N molecules
     │
     ├── chain_hash()        ← FNV-1a → u64 identity
     ├── to_tagged_bytes()   ← Wire: 1-6 bytes/molecule (sparse)
     │
     ▼
Registry.insert(chain, layer=0, kind=Alphabet)
     │
     ├── origin.olang ← append Node record
     ├── entries[]    ← RAM index
     ├── names[]      ← alias mapping ("lửa" → 🔥)
     │
     ▼
Silk connections (implicit)  ← 5D comparison, 0 bytes
     │
     ▼
LCA(group of L0) → L1 node  ← 37 base concepts
     │
     ▼
LCA(group of L1) → L2..L6   ← Shrink theo Fibonacci
     │
     ▼
  ○ (L7)                     ← Origin. Mọi thứ bắt đầu từ đây.

Tổng: ~5400 UCD entries → 5461 nodes × 8 layers
      43 KB Silk pointers + 0 bytes implicit Silk
      = Toàn bộ tri thức nền tảng của HomeOS
```

---

*"Vũ trụ không lưu hình dạng. Vũ trụ lưu công thức."*
