# Bài toán Lưu trữ, Ghi Tọa độ & Tìm kiếm trong KnowTree

> Ghi chú thiết kế — 2026-03-20
> Liên quan: `json/ucd.json`, `crates/ucd`, `PLAN_UDC_REBUILD.md`

---

## 1. Bài toán Lưu trữ

### P_weight = 5 bytes cố định

```
[S: u8][R: u8][V: u8][A: u8][T: u8]

S ∈ {Sphere, Line, Square, Triangle, Empty, Union, Intersect, SetMinus}  → 3 bits đủ, dùng u8
R ∈ {Member, Subset, Equiv, Orthogonal, Compose, Causes, Approximate, Inverse} → 3 bits đủ, dùng u8
V ∈ 0x00..0xFF  → u8 (liên tục, tự đo từ emoji)
A ∈ 0x00..0xFF  → u8 (liên tục, tự đo từ emoji)
T ∈ {Static, Slow, Medium, Fast, Instant}  → 3 bits đủ, dùng u8
```

### KnowTree root branch

```
65,536 × 5B = 328 KB   ← toàn bộ nằm trong L1 cache của CPU modern

index: u16  →  P_weight: [u8; 5]

Phân bổ:
  0x0000..0x256F  →  9,584  L0 anchor (UDC: SDF + MATH + EMOTICON + MUSICAL)
  0x2570..0xFFFF  → 55,952  alias + learned nodes (sài không hết)
```

### Tại sao không cần hash lúc lookup thông thường?

```
Alias "lửa" → index 0x2571 (ghi 1 lần lúc bootstrap)
KnowTree[0x2571] → P_weight[S=1, R=5, V=0xC0, A=0xC0, T=3]

Không cần: hash("lửa") → traverse → resolve
Thẳng 1 bước: u16 index → 5 bytes.  O(1).
```

---

## 2. Ghi Tọa độ (Write-Once / SEAL)

### Hai nguồn tọa độ — không còn con số tự cho

**L0 anchor (emoji/symbol):**
```
Nguồn đo:
  ① UnicodeData.txt → tên ký tự (NLP đơn giản → V, A sơ bộ)
  ② SVG/hình visual → shape decomposition → S, A (góc nhọn = A cao)
  ③ CLDR annotations → đồng thuận đa ngôn ngữ → V xác nhận
  ④ Vị trí trong block → calibration tương đối với hàng xóm

Kết quả: V và A tự đo từ bản chất emoji — không phải con số người gán.
```

**Alias node:**
```
"lửa" → canonical = U+1F525
P["lửa"] = P[U+1F525]   ← copy đầy đủ 5 bytes, SEAL ngay

Không phải pointer lúc runtime.
Không có "lazy resolve".
P của alias = P của canonical = immutable sau bootstrap.
```

### Quy trình SEAL

```
bootstrap(json/udc.json)
  ↓
for mỗi entry trong "characters":
  P = đo từ (emoji_visual + unicode_name + cldr + block_context)
  KnowTree[idx] = P          ← ghi RAM
  olang_writer.append_node() ← ghi file (append-only)
  SEAL                       ← đánh dấu immutable

for mỗi entry trong "alias_mapping":
  canonical_idx = resolve(target_codepoint)
  P = KnowTree[canonical_idx]   ← copy
  KnowTree[alias_idx] = P       ← ghi RAM
  olang_writer.append_alias()   ← ghi file
  SEAL

→ Sau bootstrap: KHÔNG AI ghi lại được. Chỉ Encoder học thêm qua QR.
```

---

## 3. Tìm kiếm lại (Search / Retrieval)

### Ba loại query

```
① Exact lookup   — "lửa" → u16 index → P  (O(1), thường dùng nhất)
② Codepoint      — U+1F525 → P            (O(log n) binary search)
③ Similarity     — tìm node gần tọa độ P* nhất  (vấn đề cần thiết kế)
```

### Bài toán Similarity Search

**Khoảng cách 5D không đồng nhất:**

```
S, R, T: discrete enum  →  distance = 0 nếu bằng, 1 nếu khác  (Hamming)
V, A:    continuous u8  →  distance = |v1 - v2| / 255           (Manhattan)

d(P1, P2) = wS·δ(S1≠S2) + wR·δ(R1≠R2) + wV·|V1-V2|/255 + wA·|A1-A2|/255 + wT·δ(T1≠T2)

Trọng số theo độ quan trọng cảm xúc (đề xuất):
  wS = 0.10   (hình dạng ít ảnh hưởng cảm xúc nhất)
  wR = 0.15
  wV = 0.35   (Valence = chiều cảm xúc chính)
  wA = 0.30   (Arousal = cường độ)
  wT = 0.10
```

**Tại sao V và A chiếm 65% trọng số:**
```
Bộ nhớ người dùng lưu trữ cảm xúc chủ yếu qua V và A (Russell's circumplex).
Hai emoji có cùng hình dạng nhưng khác V/A = trải nghiệm khác nhau hoàn toàn.
Hai emoji khác hình nhưng cùng V/A = cảm giác "gần nhau".
```

### Chiến lược tìm kiếm thực tế

**Bucket index (đã có trong ucd/lib.rs):**
```
Bucket (S, R) → danh sách codepoints
→ Thu hẹp không gian tìm kiếm từ 9,584 xuống ~50-200
→ Trong bucket, sort theo |V1-V2| + |A1-A2|
→ Top-k là kết quả
```

**Khi cần full similarity (ít dùng hơn):**
```
SIMD scan 328KB = ~80μs trên CPU thường
Không cần index phức tạp — brute force O(n) đủ nhanh
vì n = 65,536 và mỗi entry = 5 bytes
```

**Walk theo Silk (cho pipeline cảm xúc):**
```
KHÔNG dùng similarity search trực tiếp cho emotion processing.
Thay vào đó: walk Silk graph từ node → neighbors → amplify
→ V, A không trung bình; chúng AMPLIFY qua Silk walk
→ cortisol (V thấp) + adrenaline (A cao) = tổ hợp mạnh hơn tổng
```

---

## 4. Alias có P riêng — hệ quả thiết kế

```
Trước:  "lửa" → pointer → U+1F525 → P   (2 bước lúc runtime)
Giờ:    "lửa" → u16 → P                 (1 bước, P đã sẵn)

Điều này có nghĩa:
  - Alias "lửa" và emoji 🔥 có CÙNG P_weight (bytes giống hệt nhau)
  - Silk walk từ "lửa" và từ 🔥 cho cùng kết quả
  - Cross-language tự nhiên: "fire", "lửa", "炎" → P như nhau
  - KHÔNG cần translate layer; ngôn ngữ tự hội tụ về cùng tọa độ 5D
```

**16-bit space đủ vì:**
```
9,584  L0 anchors (UDC)
~18,000 alias cần thiết (vi + en + ja + zh thực tế)
────────────────────────
~27,584 tổng cộng   <   65,536   →   còn 37,952 slots trống

Phần dư dùng cho:
  - Learned nodes (Encoder ∫ₜ → ΔP → QR khi chín)
  - Session-specific concepts
  - L1 LCA clusters (gom kinh nghiệm)
```

---

## 5. Vấn đề mở

```
① V và A "tự đo" — cần tool đo cụ thể
   Hiện tại: build.rs dùng name pattern matching (thô)
   Cần: CLDR annotations parser + shape analysis pipeline
   → Đây là Task 0 thực sự trước khi điền characters

② Khi V/A của anchor sai → tất cả alias kế thừa sai theo
   → Cần validation: so sánh chéo giữa các ngôn ngữ
   → "lửa" vi + "fire" en + "火" zh phải cho cùng V, A ± nhỏ

③ KnowTree index assignment — ai quyết định alias "lửa" = index 0x2571?
   → Cần stable assignment: hash(lang + word) % remaining_slots?
   → Hoặc sequential: ghi vào file theo thứ tự → index = thứ tự ghi
   → APPEND-ONLY: index không bao giờ thay đổi sau khi ghi

④ Similarity search threshold
   → d < 0.10: cùng cảm xúc (rất gần)
   → d < 0.25: liên quan
   → d > 0.50: trái ngược hoặc không liên quan
   → Threshold này cần empirical validation
```

---

## Tóm tắt một dòng

```
KnowTree = 328KB array; P = đo 1 lần từ emoji visual; alias = copy P của canonical;
search = bucket(S,R) rồi sort(|ΔV|+|ΔA|); emotion = Silk walk, không search.
```
