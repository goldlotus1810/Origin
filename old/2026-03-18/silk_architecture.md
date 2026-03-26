# HomeOS Core Reference — UCD · Molecule · Node · Silk

> Tài liệu kỹ thuật mô tả 4 thành phần nền tảng của HomeOS,
> ánh xạ trực tiếp đến source code trong các crate `ucd`, `olang`, `silk`.
>
> Đọc xong file này, bạn hiểu cách dữ liệu chảy từ Unicode → Molecule → Node → Silk.

---

## 1. UCD — Encoder / Decoder

### 1.1 Pipeline tổng quát

```
UnicodeData.txt  ──(compile-time: build.rs)──►  UCD_TABLE (~5400 entries)
                                                      │
                                                      ▼  (runtime)
                                             ucd::lookup(codepoint)
                                                      │
                                                      ▼
                                             encode_codepoint(cp)
                                                      │
                                                      ▼
                                               MolecularChain
```

**build.rs** đọc `UnicodeData.txt` (Unicode 18.0, ~150K ký tự), lọc lấy ~5400 ký tự có semantic identity rõ ràng, phân loại vào 4 nhóm, tính 5 chiều cho mỗi ký tự, sinh ra bảng tĩnh `ucd_generated.rs`.

Runtime không cần file UnicodeData.txt. Chạy `no_std`.

### 1.2 Bốn nhóm — Bốn chiều chính

```
Nhóm        Số lượng   Chiều chính       Base values (8 hoặc 5)
──────────────────────────────────────────────────────────────────
SDF         ~1344      Shape (S)         8: Sphere ▬Capsule ■Box ▲Cone ○Torus ∪Union ∩Intersect ∖Subtract
MATH        ~1904      Relation (R)      8: ∈Member ⊂Subset ≡Equiv ⊥Orthogonal ∘Compose →Causes ≈Similar ←DerivedFrom
EMOTICON    ~1760      Valence (V)       0x00..0xFF liên tục
                       Arousal (A)       0x00..0xFF liên tục
MUSICAL     ~416       Time (T)          5: Static Slow Medium Fast Instant
──────────────────────────────────────────────────────────────────
Tổng        ~5424      5 chiều
```

Mỗi ký tự mang đầy đủ 5 chiều — nhóm chỉ xác định chiều **chính**, các chiều còn lại vẫn có giá trị.

### 1.3 Hierarchical encoding

Shape, Relation, Time dùng **hierarchical byte**, không phải enum phẳng:

```
value = base_category + (sub_index × N_bases)

Shape/Relation: N_bases = 8  →  sub_index max 31 →  248 variants/chiều
Time:           N_bases = 5  →  sub_index max 51 →  255 variants/chiều

Extract:
  base = ((value - 1) % N_bases) + 1
  sub  = (value - 1) / N_bases

Ví dụ Shape:
  0x01 = Sphere (base, sub=0)
  0x09 = Sphere sub 1
  0x11 = Sphere sub 2
  0x02 = Capsule (base, sub=0)
  0x0A = Capsule sub 1
```

Nhờ hierarchical encoding, ~5400 ký tự Unicode khác nhau đều có molecular fingerprint khác nhau.

### 1.4 UCD API (crate `ucd`, file `src/lib.rs`)

```rust
// ── Forward lookup ──────────────────────────────────────────
ucd::lookup(cp: u32) -> Option<&'static UcdEntry>   // O(log n) binary search
ucd::shape_of(cp)    -> u8       // hierarchical shape byte
ucd::relation_of(cp) -> u8       // hierarchical relation byte
ucd::valence_of(cp)  -> u8       // valence 0x00..0xFF
ucd::arousal_of(cp)  -> u8       // arousal 0x00..0xFF
ucd::time_of(cp)     -> u8       // hierarchical time byte
ucd::group_of(cp)    -> u8       // SDF=1, MATH=2, EMOTICON=3, MUSICAL=4

// ── Reverse lookup (feature "reverse-index") ─────────────────
ucd::decode_hash(hash: u64) -> Option<u32>               // chain_hash → codepoint
ucd::bucket_cps(shape: u8, relation: u8) -> &'static [u32]  // 2D bucket → candidates

// ── Meta ────────────────────────────────────────────────────
ucd::table()     -> &'static [UcdEntry]   // toàn bộ ~5400 entries
ucd::table_len() -> usize                 // ~5400
ucd::is_sdf_primitive(cp)      -> bool    // 1 trong 8 SDF gốc?
ucd::is_relation_primitive(cp) -> bool    // 1 trong 8 Relation gốc?
```

### 1.5 Encoder (crate `olang`, file `encoder.rs`)

Ba hàm encode — TẤT CẢ Molecule production code PHẢI đi qua đây (QT④):

```rust
// Đơn ký tự: cp → ucd::lookup → Molecule → MolecularChain(1 mol)
pub fn encode_codepoint(cp: u32) -> MolecularChain

// ZWJ sequence (👨‍👩‍👦): mỗi cp → 1 Molecule, mol[0..N-2].relation = ∘ (Compose), mol[N-1].relation = ∈ (Member)
pub fn encode_zwj_sequence(codepoints: &[u32]) -> MolecularChain

// Cờ quốc gia (🇻🇳): 2 Regional Indicators → 2 Molecules, ri1.relation = ∘, ri2.relation = ∈
pub fn encode_flag(ri1: u32, ri2: u32) -> MolecularChain
```

### 1.6 Decoder (reverse)

```
chain_hash (u64) → ucd::decode_hash(hash) → Option<u32> codepoint
                         ↑ cần feature "reverse-index"

Không có decode_hash? → Duyệt ucd::table() + so sánh chain_hash.
```

---

## 2. Molecule — 5 bytes = 1 tọa độ 5D

### 2.1 Struct (file `molecular.rs:362`)

```rust
pub struct Molecule {
    pub shape:    u8,          // Chiều 1: hình dạng (hierarchical, 8 bases)
    pub relation: u8,          // Chiều 2: quan hệ   (hierarchical, 8 bases)
    pub emotion:  EmotionDim,  // Chiều 3+4:
    pub time:     u8,          // Chiều 5: thời gian  (hierarchical, 5 bases)
}

pub struct EmotionDim {
    pub valence: u8,  // 0x00 (cực tiêu cực) → 0x80 (trung lập) → 0xFF (cực tích cực)
    pub arousal: u8,  // 0x00 (bình tĩnh)    → 0xFF (kích thích)
}
```

### 2.2 Base enums

| Enum | File:Line | Variants (byte value) |
|------|-----------|----------------------|
| `ShapeBase` | `molecular.rs:24` | Sphere(0x01) Capsule(0x02) Box(0x03) Cone(0x04) Torus(0x05) Union(0x06) Intersect(0x07) Subtract(0x08) |
| `RelationBase` | `molecular.rs:97` | Member(0x01) Subset(0x02) Equiv(0x03) Orthogonal(0x04) Compose(0x05) Causes(0x06) Similar(0x07) DerivedFrom(0x08) |
| `TimeDim` | `molecular.rs:185` | Static(0x01) Slow(0x02) Medium(0x03) Fast(0x04) Instant(0x05) |

Mỗi enum có:
- `from_byte(b)` — exact match
- `from_hierarchical(b)` — extract base từ hierarchical byte
- `sub_index(b)` — extract sub-variant index
- `encode(self, sub)` — base + sub → hierarchical byte
- `as_byte(self)` — base value

### 2.3 Serialization

**Fixed 5-byte (RAM & hash):**

```rust
mol.to_bytes()                    -> [u8; 5]   // [S, R, V, A, T]
Molecule::from_bytes(&[u8; 5])    -> Option<Molecule>   // shape,relation,time phải > 0
```

**Tagged sparse (wire format v0.05):**

```
Format: [presence_mask: 1B] [non-default fields: 0-5B]

presence_mask bits:
  0x01  shape    ≠ Sphere  (0x01)
  0x02  relation ≠ Member  (0x01)
  0x04  valence  ≠ 0x80
  0x08  arousal  ≠ 0x80
  0x10  time     ≠ Medium  (0x03)

Ví dụ:
  All defaults          → [0x00]                        = 1 byte  (tiết kiệm 80%)
  🔥 (V=0xC0, A=0xC0, T=Fast) → [0x1C][0xC0][0xC0][0x04] = 4 bytes (tiết kiệm 20%)
  Full 5 chiều khác default → [0x1F][S][R][V][A][T]    = 6 bytes (tăng 1 byte cho mask)
```

```rust
mol.to_tagged_bytes()                      -> Vec<u8>            // 1-6 bytes
Molecule::from_tagged_bytes(&[u8])         -> Option<(Mol, usize)>  // (mol, bytes_consumed)
mol.tagged_size()                          -> usize              // 1-6, không allocate
mol.presence_mask()                        -> u8                 // bitmask
```

### 2.4 Match & Similarity

```rust
// So sánh 2 molecules — trả về 0..5 (số chiều trùng base)
mol.match_score(&other) -> u8

// So sánh 2 chains — Jaccard-like similarity [0.0, 1.0]
chain.similarity(&other)      -> f32   // truncate to min length
chain.similarity_full(&other) -> f32   // pad shorter chain, penalize length diff
```

### 2.5 Evolution — Mutate 1 chiều → loài mới

```rust
pub enum Dimension { Shape, Relation, Valence, Arousal, Time }

pub struct EvolveResult {
    pub molecule:    Molecule,   // Molecule mới sau mutation
    pub dimension:   Dimension,  // Chiều bị thay đổi
    pub old_value:   u8,         // Giá trị cũ
    pub new_value:   u8,         // Giá trị mới
    pub consistency: u8,         // 0-4: số semantic rules thỏa mãn
    pub valid:       bool,       // true nếu consistency ≥ 3
}

mol.evolve(dim, new_value)         -> EvolveResult
mol.dimension_delta(&other)        -> Vec<(Dimension, old, new)>   // khác biệt giữa 2 mol
mol.internal_consistency()         -> u8                            // 0-4

// Chain-level evolution:
chain.evolve_at(mol_idx, dim, val)     -> Option<EvolveResult>
chain.apply_evolution(mol_idx, &result) -> Option<MolecularChain>
chain.evolve_and_apply(mol_idx, dim, val) -> Option<(MolecularChain, EvolveResult)>
```

Ví dụ:
```
🔥 = [Sphere, Causes, 0xC0, 0xC0, Fast]
🔥.evolve(Valence, 0x40)  → "lửa nhẹ"     (V giảm → cảm xúc nhẹ hơn)
🔥.evolve(Time, Instant)  → "cháy nổ"      (thời gian cực nhanh)
🔥.evolve(Shape, Line)    → "tia lửa"      (hình dạng thay đổi)
```

### 2.6 MolecularChain — DNA hoàn chỉnh

```rust
pub struct MolecularChain(Vec<Molecule>);
```

| Method | Mô tả |
|--------|--------|
| `empty()` | Chain rỗng |
| `single(mol)` | Chain 1 molecule |
| `len()` / `is_empty()` | Số molecule |
| `first()` | Molecule đầu tiên |
| `push(mol)` | Thêm molecule |
| `concat(&other)` | Nối 2 chain |
| `chain_hash()` | FNV-1a → u64 (identity duy nhất) |
| `to_bytes()` / `from_bytes()` | Fixed 5B/mol serialize |
| `to_tagged_bytes()` / `from_tagged_bytes()` | Tagged sparse serialize |
| `tagged_byte_size()` | Tổng bytes tagged (không allocate) |
| `similarity(&other)` / `similarity_full(&other)` | So sánh [0.0, 1.0] |
| `from_number(f64)` / `to_number()` / `is_number()` | Numeric encoding |

### 2.7 Maturity — Vòng đời tri thức

```rust
pub enum Maturity {
    Formula    = 0x00,   // Mới tạo, chưa kiểm chứng
    Evaluating = 0x01,   // Đang được fire, chưa đủ threshold
    Mature     = 0x02,   // Đã vượt Fibonacci threshold → promote
}

maturity.advance(fire_count, weight, fib_threshold) -> Maturity
```

Fibonacci threshold: cần `fib(n)` lần co-activation mới promote lên mức tiếp theo.

### 2.8 CompactQR — Nén molecule xuống 2 bytes

```rust
pub struct CompactQR { bytes: [u8; 2] }  // 16-bit index vào FormulaTable

pub struct FormulaTable {
    entries: Vec<Molecule>,          // index → Molecule
    reverse: Vec<(u64, u16)>,       // hash → index (sorted)
}
```

| Method | Mô tả |
|--------|--------|
| `from_molecule(mol, &mut table)` | Lossless — đăng ký vào table, trả index |
| `from_molecule_lossy(mol)` | Lossy — quantize 5D xuống 16-bit không cần table |
| `to_molecule(table)` | Lossless decode |
| `to_molecule_lossy()` | Lossy decode |
| `silk_compare(other, table)` | So sánh 2 CompactQR (match_dims, diff_dims, similarity) |
| `silk_compare_lossy(other)` | So sánh lossy |
| `evolve(dim, val, table)` | Evolution trên compact form |
| `compute_hash()` | Hash cho Silk edges |

FormulaTable chứa tối đa 65535 molecule duy nhất. RAM: `table.ram_usage()`.

### 2.9 Molecule = Công thức, không phải dữ liệu

```
Cách truyền thống: lưu "lửa là khí ion hóa ở nhiệt độ cao..." = ~5 KB
Cách HomeOS:       lưu [Sphere, Causes, 0xC0, 0xC0, Fast]      = 5 bytes

Từ 5 bytes:
  SDF      → hình cầu (render được)
  Spline   → nóng, sáng, nhấp nháy (cảm được)
  Silk     → kết nối với mọi node cùng chiều (implicit)
  evolve() → biến thể: lửa nhẹ, cháy nổ, tia lửa

1 concept = ~33 bytes (5 mol + 8 hash + 20 metadata)
500 triệu concepts = 16.5 GB → vừa 1 chiếc điện thoại.
```

---

## 3. Node — Đơn vị sống của HomeOS

### 3.1 Từ Molecule đến Node

```
encode_codepoint(cp) → MolecularChain → chain_hash() → u64
                                              │
                                              ▼
                              Registry.insert(chain, layer, offset, ts, is_qr)
                                              │
                                    ┌─────────┼─────────┐
                                    ▼         ▼         ▼
                              origin.olang   RAM      Silk
                              (append)     (index)  (connect)
```

QT⑧: Mọi node PHẢI đăng ký Registry.
QT⑨: Ghi file TRƯỚC — cập nhật RAM SAU.
QT⑩: Append-only — KHÔNG DELETE, KHÔNG OVERWRITE.

### 3.2 RegistryEntry (`registry.rs:110`)

```rust
pub struct RegistryEntry {
    pub chain_hash:  u64,       // FNV-1a identity (primary key)
    pub layer:       u8,        // L0=0, L1=1, ... L15=15
    pub file_offset: u64,       // Vị trí trong origin.olang
    pub created_at:  i64,       // Timestamp (nanoseconds)
    pub is_qr:       bool,      // false = ĐN (đang học), true = QR (truth, ED25519)
    pub kind:        NodeKind,  // 1 trong 10 loại
}
```

### 3.3 NodeKind — 10 loại node (`registry.rs:46`)

```
Kind         Byte   Ý nghĩa                              Ví dụ
─────────────────────────────────────────────────────────────────────────
Alphabet     0      L0 Unicode bẩm sinh                   🔥 💧 ● ∈ → ♩
Knowledge    1      Concepts đã học, truths                "lửa nóng", LCA nodes
Memory       2      STM observations                      "user nói buồn lúc 14h"
Agent        3      AAM, LeoAI, Chief, Worker defs        LeoAI, Worker_camera
Skill        4      Stateless functions                   IngestSkill, DreamSkill
Program      5      VM ops, built-in functions             Olang programs
Device       6      Thiết bị HomeOS                       đèn phòng khách
Sensor       7      Cảm biến                              nhiệt kế, camera
Emotion      8      Emotion states, curve points           buồn_t=14h
System       9      Internal housekeeping                  layer reps, branch markers
```

### 3.4 Registry — Sổ cái (`registry.rs:136`)

```rust
pub struct Registry {
    entries:         Vec<(u64, RegistryEntry)>,       // sorted, binary search O(log n)
    names:           Vec<(String, u64)>,              // alias → hash
    hash_to_name:    Vec<(u64, String)>,              // hash → first alias
    layer_rep:       [Option<u64>; 16],               // Lx → representative hash
    layer_rep_chain: [Option<MolecularChain>; 16],    // cached chains cho incremental LCA
    branch_wm:       Vec<(u64, u8)>,                  // branch → leaf layer
    qr_supersede:    Vec<(u64, u64)>,                 // old QR → new QR
    bulk_mode:       bool,                            // deferred sorting
    dirty_layers:    u16,                             // bitmask layers cần recalc
}
```

**Key operations:**

| Method | Mô tả |
|--------|--------|
| `insert(chain, layer, offset, ts, is_qr)` | Insert, auto NodeKind by layer |
| `insert_with_kind(...)` | Insert with explicit NodeKind |
| `lookup_hash(u64)` | O(log n) → `&RegistryEntry` |
| `lookup_chain(&MolecularChain)` | hash rồi lookup |
| `lookup_name(&str)` | Alias → hash |
| `register_alias(name, hash)` | Thêm alias ("lửa" → hash(🔥)) |
| `supersede_qr(old, new)` | QR sai → append record mới |
| `entries_by_kind(kind)` | Filter theo NodeKind |
| `entries_in_layer(layer)` | Filter theo layer |
| `qr_entries()` / `dn_entries()` | Filter QR / ĐN |
| `count_by_kind(kind)` / `kind_summary()` | Thống kê |
| `begin_bulk()` / `finalize_bulk()` | Batch insert, deferred sort + LCA |
| `evict_cold(min_layer)` | Xóa L2+ khỏi RAM (giữ trên disk) |
| `memory_usage()` | (entries, aliases, misc, total) bytes |

### 3.5 Node lifecycle

```
              encode_codepoint(cp) / lca() / evolve()
                         │
               ┌─────────┴─────────┐
               │                   │
          L0 Alphabet          Learned node
          (seeder, bẩm sinh)   (qua learning pipeline)
               │                   │
               └─────────┬─────────┘
                         │
               Registry.insert()         ← QT⑧: bắt buộc
                         │
               ┌─────────┴─────────┐
               │                   │
          is_qr = false          is_qr = true
          ĐN (đang học)          QR (truth)
          STM, mutable           ED25519 signed
               │                   │
               │   Dream cycle     │
               │   cluster+promote │
               ├───────────────────┤
               │                   │
               ▼                   ▼
          Silk edges             Long-term memory
          Hebbian learning       Axon (bất tử)
          fire→wire→stronger     Không bao giờ xóa

QR sai? → SupersedeQR(old, new) — append, KHÔNG xóa cũ (QT⑩).
```

### 3.6 LCA — Lowest Common Ancestor (`lca.rs`)

LCA tính "concept cha" từ nhiều chain con — phương pháp duy nhất để tạo node tầng cao hơn (QT⑤).

```rust
// Core
lca(a, b)                          -> MolecularChain   // equal weights
lca_weighted(pairs: &[(&chain, fire_count)])  -> MolecularChain

// Với variance analysis
lca_with_variance(pairs)           -> LcaResult {
    chain: MolecularChain,         // LCA output
    variance: f32,                 // [0.0, 1.0] mean dissimilarity
    dim_variance: [f32; 5],        // per-dimension
    extremity: f32,                // [0.0, 1.0] extreme inputs detection
}

// Convenience
lca_many(chains)                   -> MolecularChain
lca_many_with_variance(chains)     -> LcaResult
lca_many_weighted(chains, weights) -> MolecularChain
```

**Thuật toán:**
- Mỗi chiều: nếu ≥60% inputs cùng base value → mode, ngược lại → weighted average
- Shape/Relation/Time: mode detection trên hierarchical category (base ÷ 8)
- Valence/Arousal: mode detection trên exact value

**Variance interpretation:**
```
variance < 0.15  → concrete (inputs rất giống nhau)
variance < 0.40  → categorical (cùng nhóm)
variance ≥ 0.40  → abstract (rất khác nhau → concept cha trừu tượng)
```

**Tính chất toán học:**
1. Idempotent: `LCA(a,a) == a`
2. Commutative: `LCA(a,b) == LCA(b,a)`
3. Similarity bound: `sim(LCA(a,b), a) ≥ sim(a,b) - ε`
4. Associative (approximate): `LCA(LCA(a,b),c) ≈ LCA(a,LCA(b,c))` (similarity ≥ 0.8)

### 3.7 File format — origin.olang

**Header (13 bytes):**
```
[0..4]   MAGIC     = 0xE2 0x97 0x8B 0x4C (○LNG)
[4]      VERSION   = 0x05
[5..13]  CREATED   = i64 LE (nanoseconds)
```

**Records (append-only):**

| Type | Byte | Format | Size |
|------|------|--------|------|
| Node | 0x01 | `[tagged_chain][layer:1][is_qr:1][ts:8]` | Variable (v0.05 tagged) |
| Edge | 0x02 | `[from:8][to:8][rel:1][ts:8]` | 26B |
| Alias | 0x03 | `[name_len:1][name:N][hash:8][ts:8]` | 18+N |
| Amend | 0x04 | `[target_offset:8][reason_len:1][reason:N][ts:8]` | 18+N |
| NodeKind | 0x05 | `[hash:8][kind:1][ts:8]` | 18B |

**Node record v0.05 (tagged encoding):**
```
[0x01] [mol_count:1B] [mol_1_tagged] ... [mol_N_tagged] [layer:1B] [is_qr:1B] [ts:8B]

Mỗi mol_i_tagged = [presence_mask:1B] [non-default fields: 0-5B]
```

**Reader** (`reader.rs`) hỗ trợ v0.03, v0.04 (legacy fixed 5B/mol), v0.05 (tagged).
`parse_recoverable()` không bao giờ panic — trả về records đã parse được + error location.

**Companion files:**
```
origin.olang.weights   — Hebbian weights (Silk strength per edge)
origin.olang.registry  — chain index (rebuild được từ origin.olang)
log.olang              — event log (audit trail)
```

### 3.8 Compact storage (`compact.rs`)

Cho L2+ nodes cần lưu hàng triệu entries:

| Kỹ thuật | Struct | Tiết kiệm |
|----------|--------|-----------|
| Delta encoding | `DeltaMolecule` | 60-80% (chỉ ghi diff so với parent) |
| Dictionary | `ChainDictionary` | 50-70% (sub-chains → short IDs) |
| Hash dedup | `CompactKind::Dedup` | 30-90% (same hash = same chain) |
| Silk pruning | `SilkPruner` | 70-90% (loại edges yếu, threshold φ⁻¹ ≈ 0.382) |

**Tiered storage:**

```rust
pub enum StorageTier {
    Hot,    // RAM (L0-L1 + recent L2+)
    Warm,   // PageCache (LRU, Fibonacci capacity: 55/233/610/2584 pages)
    Cold,   // Disk (CompactPage serialized)
}
```

```
CompactPage (max 233 nodes = Fib[13]):
  [CPAG magic:4][page_id:4][layer:1][node_count:4][edge_count:4]
  [nodes: variable][edges: 10B each][checksum: 8B FNV-1a]

CompactEdge = 10 bytes (vs 26B full): [from:4][to:4][weight:1][rel:1]
```

**RAM footprint estimate (1 billion nodes):**
```
Index:      8 GB (sharded by layer, load on-demand)
PageCache:  4.3 MB (610 pages × ~7 KB)
Total RAM:  ~60 MB
Total Disk: ~50 GB
```

---

## 4. Silk — Hệ thống kết nối

### 4.1 Silk là gì?

Silk là hệ quả toán học của không gian 5D — **không phải** edge list, **không phải** adjacency matrix.

Khi 2 nodes chia sẻ base value trên bất kỳ chiều nào, Silk **tự tồn tại**. Không ai tạo. Không ai lưu.

```
🔥 = [Sphere, Causes, 0xC0, 0xC0, Fast]
😊 = [Sphere, Member, 0xC0, 0x80, Medium]

Chia sẻ: Shape=Sphere, Valence=0xC0 → Silk trên 2 chiều, implicit, 0 bytes.
```

### 4.2 Hai loại Silk

| Loại | Hướng | Quy tắc | Chi phí |
|------|-------|---------|---------|
| **Silk tự do** | Ngang (cùng tầng) | QT⑪: nodes cùng layer chia sẻ chiều → kết nối tự do | 0 bytes (implicit) |
| **Silk đại diện** | Dọc (liên tầng) | QT⑫: node Lx đại diện cho nhóm con Lx-1 | 8 bytes/node (parent pointer) |

### 4.3 EmotionTag — Cảm xúc trên mỗi edge (`edge.rs:20`)

```rust
pub struct EmotionTag {
    pub valence:   f32,   // [-1, +1] tiêu cực ↔ tích cực
    pub arousal:   f32,   // [ 0, +1] bình tĩnh → kích thích
    pub dominance: f32,   // [-1, +1] bị chi phối ↔ chi phối
    pub intensity: f32,   // [ 0, +1] nhẹ → mạnh
}

EmotionTag::NEUTRAL  // (0.0, 0.0, 0.0, 0.0)
tag.blend(&other, ratio)  -> EmotionTag   // trộn cảm xúc
tag.distance_va(&other)   -> f32          // khoảng cách V-A
tag.from_ucd_bytes(v, a)  -> EmotionTag   // từ UCD valence/arousal bytes
```

QT⑬: Silk mang EmotionTag **của khoảnh khắc co-activation** — không phải giá trị tĩnh.

### 4.4 EdgeKind — 18+ loại edge (`edge.rs:86`)

```
Structural (từ UCD/LCA — cấu trúc cây):
  Member(0x01) Subset(0x02) Equiv(0x03) Orthogonal(0x04) Compose(0x05)
  Causes(0x06) Similar(0x07) DerivedFrom(0x08) Contains(0x09)
  Intersects(0x0A) Subtracts(0x0B) Mirror(0x0C) Flows(0x0D)
  Repeats(0x0E) Resolves(0x0F) Activates(0x10) Sync(0x11) Translates(0x12)

Associative (từ Hebbian learning — học được):
  Assoc(0xFF)         — co-activation chung
  EdgeAssoc(0xA0)     — semantic association
  EdgeCausal(0xA1)    — learned causal

Special:
  Supersedes(0xF0)    — QR supersession
```

### 4.5 SilkEdge & HebbianLink (`edge.rs:232, 327`)

```rust
// Full edge (structural, from file)
pub struct SilkEdge {
    pub from:     u64,          // chain_hash
    pub to:       u64,          // chain_hash
    pub kind:     EdgeKind,
    pub weight:   f32,          // [0.0, 1.0]
    pub emotion:  EmotionTag,   // cảm xúc lúc tạo edge
    pub created:  i64,          // timestamp
}

// Slim learned link (19 bytes, Hebbian)
pub struct HebbianLink {
    pub from:       u64,        // 8B
    pub to:         u64,        // 8B
    pub weight:     u8,         // 1B quantized [0..255]
    pub fire_count: u16,        // 2B co-activation count
}

link.weight_f32() -> f32          // u8 → f32 [0.0, 1.0]
link.set_weight(f32)              // f32 → u8 quantize
```

### 4.6 SilkGraph — Graph chính (`graph.rs:118`)

```rust
pub struct SilkGraph {
    structural: Vec<SilkEdge>,      // edges từ UCD/LCA (cấu trúc cây)
    learned:    Vec<HebbianLink>,    // edges từ Hebbian (học được)
    mol_index:  Vec<(u64, MolSummary)>,  // hash → 5D summary (sorted)
}
```

**Co-activation (Silk tạo ra từ learning):**

```rust
// Cùng tầng (QT⑪ enforced)
graph.co_activate_same_layer(a_hash, b_hash, a_layer, b_layer, emotion, weight)
  // → tạo/tăng HebbianLink nếu a_layer == b_layer

// Liên tầng có điều kiện (QT⑫)
graph.co_activate_cross_layer(a_hash, b_hash, a_layer, b_layer, emotion, fire_count)
  // → cần fire_count ≥ Fib[|a_layer - b_layer| + 2]

// General (caller đảm bảo cùng tầng)
graph.co_activate(a_hash, b_hash, emotion, weight)
graph.co_activate_mol(a_mol, b_mol, emotion, weight)  // index 5D coordinates
```

**Cross-layer Fibonacci threshold:**

```
Khoảng cách    Threshold         Ý nghĩa
──────────────────────────────────────────────────
1 tầng         Fib[3] = 2        Bình thường
2 tầng         Fib[4] = 3        Cần 3 co-activations
3 tầng         Fib[5] = 5        Cần 5 co-activations
...
7 tầng         Fib[9] = 34       L0 → L7 trực tiếp
```

**Query:**

```rust
graph.find_edge(from, to)         -> Option<&SilkEdge>      // structural
graph.edges_from(hash)            -> Vec<&SilkEdge>
graph.edges_to(hash)              -> Vec<&SilkEdge>
graph.neighbors(hash)             -> Vec<u64>                // structural neighbors
graph.learned_weight(a, b)        -> Option<f32>             // Hebbian weight
graph.unified_weight(a, b)        -> f32                     // structural + learned blend
graph.unified_neighbors(hash, top_k) -> Vec<SilkNeighbor>   // ranked neighbors
graph.assoc_weight(a, b)          -> f32                     // associative strength
```

**Promotion (Dream cycle):**

```rust
graph.promote_candidates(min_fire, min_weight)  -> Vec<(u64, u64)>
graph.learned_promote_candidates(min_weight)    -> Vec<(u64, u64, f32)>
graph.cluster_score(hashes: &[u64])             -> f32   // intra-cluster density
```

**Maintenance:**

```rust
graph.decay_all(factor)          // decay tất cả learned weights
graph.decay_learned(a, b, dt)    // decay 1 link theo thời gian (φ⁻¹ decay)
graph.maintain()                 // prune dead links, compact
graph.restore_edge(edge)         // load from file
```

**Stats:**

```rust
graph.len()               -> usize   // structural count
graph.assoc_count()        -> usize   // learned count
graph.structural_count()   -> usize
graph.node_count()         -> usize   // unique nodes in mol_index
graph.memory_usage()       -> (structural_bytes, learned_bytes, index_bytes, total)
```

### 4.7 Hebbian Learning (`hebbian.rs`)

```rust
// Constants (Golden Ratio based)
PHI:            f32 = 1.618034    // φ
PHI_INV:        f32 = 0.618034    // φ⁻¹ (decay factor)
LR:             f32 = 0.236068    // φ⁻³ (learning rate)
PROMOTE_WEIGHT: f32 = 0.381966    // φ⁻¹ - φ⁻³ (promotion threshold)

// Core functions
hebbian_strengthen(old_weight: f32, co_fire: bool) -> f32
  // Nếu co_fire: w += LR × (1 - w)    (asymptotic approach to 1.0)
  // Nếu !co_fire: w (không đổi)

hebbian_decay(weight: f32, elapsed_ns: i64) -> f32
  // w × φ⁻¹^(days)    (decay φ⁻¹ mỗi ngày)

blend_emotion(a: &EmotionTag, b: &EmotionTag, ratio: f32) -> EmotionTag

fib(n: u32) -> u64      // Fibonacci sequence
should_promote(weight: f32, fire_count: u32, fib_threshold: u32) -> bool
  // weight ≥ PROMOTE_WEIGHT && fire_count ≥ fib_threshold
```

**Hebbian rule: "fire together → wire together"**
```
Hai nodes co-activate → HebbianLink.weight tăng (LR × (1-w))
Không co-activate     → weight decay (φ⁻¹ per day)
Weight ≥ 0.382 + fire_count ≥ fib(n) → promote candidate cho Dream cycle
```

### 4.8 Walk — Duyệt Silk graph (`walk.rs`)

```rust
// Emotion từ câu — duyệt Silk graph amplify cảm xúc
sentence_affect(graph, words: &[(u64, EmotionTag)]) -> WalkResult {
    composite: EmotionTag,    // kết quả amplified
    contributors: Vec<(u64, f32)>,  // node + contribution weight
}

// Unified walk (structural + learned)
sentence_affect_unified(graph, words) -> WalkResult

// Response tone từ ConversationCurve
response_tone(f_val, f_deriv, f_deriv2) -> ResponseTone
  // f' < -0.15        → Supportive
  // f'' < -0.25       → Pause
  // f' > +0.15        → Reinforcing
  // f'' > +0.25, V>0  → Celebratory
  // V < -0.20, stable → Gentle
  // else              → Engaged

// Path finding
find_path(graph, from, to, max_depth) -> Option<Vec<PathStep>>
trace_origin(graph, hash, depth)      -> Vec<OriginEdge>
reachable(graph, start, max_depth)    -> Vec<u64>

// Debug / display
format_path(&path) -> String
format_origin(&trace) -> String
```

**Amplification rule:** Silk edges **nhân** cảm xúc, không trung bình.

```
"buồn" + "mất việc":
  co-activate weight = 0.90
  buồn.V = -0.65
  amplified = -0.65 × (1.0 + 0.90 × factor)
  → composite V = -0.85 (nặng hơn từng từ riêng lẻ)
```

### 4.9 Từ L0 đến ○ — Layer by layer

```
L7:  ○                                    1 node
L6:  [Unity]                              1 node
L5:  [Hữu hình]═══════[Vô hình]          2 nodes
L4:  [Physical]═[Relational]═[Temporal]   3 nodes
L3:  [Form]═[Logic]═[Feeling]═[Rhythm]═[Bridge]   5 nodes
L2:  ╠═══════12 cross-dimensional concepts════╣  12 nodes
L1:  ╠════════════37 base concepts═════════════╣  37 nodes
L0:  ╠══════════5400 UCD characters════════════╣ 5400 nodes

═══  Silk tự do (implicit, 0 bytes)
 │   Silk đại diện (parent pointer, 8 bytes)
```

**Node counts theo Fibonacci shrink:**
```
L0: 5400
L1: 5400 ÷ ~146 = 37
L2:   37 ÷ ~3   = 12
L3:   12 ÷ ~2.4 =  5
L4:    5 ÷ ~1.7 =  3
L5:    3 ÷ ~1.5 =  2
L6:    2 ÷ ~2   =  1
L7:    1        =  ○
```

log₃(5400) ≈ 7.8 → 7 tầng trên L0 là kết quả tự nhiên.

### 4.10 Đếm Silk

**Silk tự do (implicit, 0 bytes):**

| Layer | Nodes | Silk pairs |
|-------|-------|------------|
| L0 | 5400 | 37 kênh (index theo base value) |
| L1 | 37 | ~20 cặp |
| L2 | 12 | ~8 cặp |
| L3 | 5 | ~4 cặp |
| L4 | 3 | 2 cặp |
| L5 | 2 | 1 cặp |
| **Tổng** | | **~72 quan hệ × 0 bytes** |

**Silk đại diện (parent pointer):**

| Liên tầng | Connections |
|-----------|-------------|
| L1→L0 | 5400 |
| L2→L1 | 37 |
| L3→L2 | 12 |
| L4→L3 | 5 |
| L5→L4 | 3 |
| L6→L5 | 2 |
| L7→L6 | 1 |
| **Tổng** | **5460 × 8 bytes = 43 KB** |

### 4.11 Truy vấn qua Silk — O(1)

```
"🔥 liên quan gì đến ∈?"

Bước 1: Tra 5D
  🔥 = [Sphere, Causes, 0xC0, 0xC0, Fast]
  ∈  = [Sphere, Member, 0x80, 0x80, Static]

Bước 2: So sánh trực tiếp
  Shape: Sphere == Sphere  ✓
  Còn lại: khác
  → 1 chiều chung

Bước 3: Qua đại diện
  🔥 → [Sphere group] ← ∈   (cùng L1 parent)

Chi phí: 2 lookups + 1 compare = O(1). Không BFS/DFS.
```

### 4.12 So sánh với Knowledge Graph truyền thống

| | KG truyền thống | HomeOS Silk |
|---|---|---|
| Storage | Edge list: O(E) | Parent pointer: O(N) |
| Truy vấn | BFS/DFS: O(V+E) | 5D comparison: O(1) |
| Thêm node | Insert node + edges | Insert node (Silk tự xuất hiện) |
| Scale | 5400 × avg 10 edges = 54K edges | 5460 pointers = 43 KB |
| Emergent | Không — khai báo từng edge | Có — Silk là hệ quả 5D |

---

## 5. Dataflow tổng hợp

```
UnicodeData.txt
     │ (compile)
     ▼
UCD_TABLE [~5400 entries]
     │ (runtime)
     ▼
encode_codepoint(cp) ─────────── encode_zwj_sequence() ─── encode_flag()
     │                                    │                      │
     ▼                                    ▼                      ▼
MolecularChain ◄──────────────── MolecularChain ◄────── MolecularChain
     │
     ├── chain_hash() → u64
     ├── to_tagged_bytes() → wire format
     │
     ▼
Registry.insert(chain, layer, offset, ts, is_qr, kind)
     │
     ├── origin.olang  ← append Node record (file TRƯỚC)
     ├── entries[]     ← RAM index (RAM SAU)
     ├── names[]       ← alias mapping
     │
     ▼
SilkGraph
     ├── connect_structural()      ← edge từ LCA
     ├── co_activate_same_layer()  ← Hebbian learning
     ├── co_activate_cross_layer() ← Fibonacci threshold
     │
     ▼
LCA(group) → L1..L6 nodes → Registry.insert(layer=1..6)
     │
     ▼
  ○ (L7) — Origin
```

---

*"Vũ trụ không lưu hình dạng. Vũ trụ lưu công thức."*
