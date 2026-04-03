# SPEC B — Cấu Trúc Dữ Liệu: Chain + KnowTree + Silk + QR

> **Tài liệu kỹ thuật chi tiết cho Phần B của HomeOS Unified Spec.**
> **Prerequisite:** Đọc SPEC_A_FOUNDATION.md trước.
> **Tác giả:** Lupin (thiết kế) + Nox (tổng hợp + verify)
> **Ngày:** 2026-03-30
> **Verified against:** BLUEPRINT §4-§7, KNOWTREE_DESIGN.md, SPEC_NODE_SILK.md, origin.md (Go), Lupin (trực tiếp)

---

## MỤC LỤC

```
B1. Chain — Chuỗi pointers, DNA của tri thức
B2. KnowTree — Cây phân nhóm ngữ nghĩa
B3. Silk — 9,200 loại kết nối (= UDC table)
B4. QR — Append-only, tri thức đã chứng minh
B5. Sai lệch đã sửa
```

---

## B1. Chain — Chuỗi Pointers

### Cấu trúc

```
Chain = mảng u16, mỗi link = 1 P_weight = 1 pointer vào KnowTree node.

  DNA:     A—T—C—G—G—A—T—C     (4 loại nucleotide)
  HomeOS:  [42][108][7291][53]   (9,200 loại UDC + learned nodes)

  1 link = 2 bytes (u16 P_weight)
  1 từ "Việt" = 4 links = 8 bytes
  1 câu = ~20 links = ~40 bytes
  1 sách = ~350,000 links = ~700 KB
  1 đời = ~tỷ links = ~2 GB
```

### Link = pointer, không phải data

```
Chain KHÔNG copy nội dung. Chain TRỎ.
Mỗi link = P_weight = địa chỉ node trong KnowTree.
Node đã tồn tại → chỉ ghi pointer (2 bytes).
Copy cả cuốn sách = 2 bytes (1 pointer đến chain gốc).
```

### Đọc chain = evaluate tuần tự

```
evaluate(chain) → P_weight:
  result = P_weight_zero
  for link in chain:
    node = KnowTree.lookup(link)
    result = compose(result, node.P_weight)
  return result

Giống ribosome đọc mRNA: chạy từ đầu → cuối → ra protein.
Thứ tự trong chain = Structural Silk = 0 bytes overhead.
```

### Chain là đường spline qua 5D

```
Mỗi link = 1 điểm trong 5D.
Chain = đường đi qua các điểm.
Khác thứ tự = khác đường = khác P_weight kết quả.

"tôi yêu bạn" = spline [tôi → yêu → bạn]
"bạn yêu tôi" = spline [bạn → yêu → tôi]
Cùng điểm, khác đường → khác ý nghĩa.
```

### Node ở mọi cấp

```
P_w("t") = node (ký tự)
P_w("tôi") = node (từ) → chain = [t, ô, i]
P_w("tôi yêu bạn") = node (câu) → chain = [tôi, yêu, bạn]
P_w(chương 1) = node → chain = [câu1, câu2, ...]
P_w(sách) = node → chain = [chương1, chương2, ...]

Mỗi cấp = 1 node. Node chứa chain. Chain chứa pointers đến nodes khác.
Fractal: cùng cấu trúc ở mọi zoom level.
```

---

## B2. KnowTree — Cây Phân Nhóm Ngữ Nghĩa

### Kiến trúc: L0 ở giữa, không phải trên đỉnh

```
      L1 (tools, agents, devices, functions)
       ↑
 L0 ← ○ → L2 → L3 → L4 → ... → Ln-1
(gốc)      (knowledge tree)

L0 = GỐC. Trung tâm điều khiển. Engine.
  - UDC table (9,200 công thức SDF)
  - Compose/Distance formulas
  - Encoder ∫, Decoder ∂
  - Kích thước: CỐ ĐỊNH (~1MB binary + 20KB UDC data)

L1 = PHỤC VỤ L0. Tools và runtime.
  - Compiler pipeline (lexer, parser, semantic, codegen)
  - VM runtime (opcodes, stack, heap)
  - Intelligence pipeline (encode → instinct → gate → compose)
  - Agents, devices, sensors
  - Kích thước: CỐ ĐỊNH (trong binary)

L2→Ln-1 = TRI THỨC. KnowTree bắt đầu ở đây.
  - L2 = mặt đất, nhóm chính
  - L3→Ln-1 = phân nhóm sâu dần
  - Kích thước: PHÁT TRIỂN mỗi ngày

L0 đọc/ghi L2+. L2+ không biết L0.
L0 = ribosome. L2+ = DNA.
```

### L2→Ln-1: Phân nhóm ngữ nghĩa (KHÔNG phải array lồng)

```
L2: NHÓM CHÍNH
  [0] facts          [1] books           [2] conversations
  [3] skills         [4] people          [5] places
  [6] events         [7] emotions        [8] songs
  [9] images         ...                 [N] bất kỳ loại nào

L3: TỪ NHÓM CHÍNH → NHÓM NHỎ (định nghĩa khác nhau)
  facts/
    geography/       science/        personal/       history/

L4: TỪ ĐỊNH NGHĨA → PHÂN NHÓM TIẾP
  facts/geography/
    Vietnam/         Japan/          Europe/

L5: TIẾP TỤC PHÂN...
  facts/geography/Vietnam/
    cities/          rivers/         mountains/

...

Ln-1: TẦNG CUỐI — lá
  facts/geography/Vietnam/cities/HaNoi → P_weight = lá

Mỗi tầng = 1 cấp PHÂN LOẠI Ý NGHĨA.
Depth không cố định — mỗi nhánh phân đến hết ý nghĩa.
```

### Mỗi nhánh = array[65,536]

```
Mỗi nhánh = array tối đa 65,536 phần tử.
Mỗi phần tử = u16: hoặc LÁ (P_weight) hoặc NHÁNH CON (pointer).

65,536^2 = 4.3 tỷ (depth 2)
65,536^3 = 281 nghìn tỷ (depth 3)
CÂY KHÔNG BAO GIỜ ĐẦY.

1 cuốn sách 100 trang = ~750,000 lá = 0.001% depth 2.
1 đời đọc 200 cuốn = 150 triệu lá = 3.5% depth 2.
```

### Dung lượng

```
L0+L1: ~1.1 MB (cố định)
L2+:
  1 lá = 2 bytes
  1 nhánh header = 2 bytes
  1 cuốn sách: ~400 KB (chains + leaves + silk)
  256 MB heap: ~637 cuốn sách
  16 GB disk: ~40,000 cuốn sách = thư viện nhỏ
```

---

## B3. Silk — 9,200 Loại Kết Nối

### Nguyên lý cốt lõi

```
Silk KHÔNG PHẢI 1 loại. Silk CÓ RẤT NHIỀU loại.
Mỗi loại silk = 1 KIỂU quan hệ khác nhau.

VÀ: silk types = chính các chiều S, R, V, A, T.
VÀ: mỗi UDC character ĐẠI DIỆN cho 1 loại silk.

9,200 UDC characters = 9,200 loại silk.

ĐÓ LÀ LÝ DO UDC TABLE TỒN TẠI.
UDC char vừa LÀ node, vừa LÀ silk type.
Giống nguyên tố hóa học: vừa tồn tại, vừa định nghĩa bonds.
→ "Bảng tuần hoàn" = định nghĩa cả atoms VÀ connections.
```

### Silk types theo dimension

```
S silk (1,904 types): kết nối hình dạng
  "o" ~ "ô" ~ "ơ"              (visual shape gần nhau)
  "□" ~ "■" ~ "▪"              (cùng box family)

R silk (3,216 types): kết nối logic/quan hệ
  "∈" ~ "⊂" ~ "⊆"              (cùng set theory)
  "+" ~ "×" ~ "÷"              (cùng arithmetic)

V silk (3,056 types): kết nối cảm xúc
  "yêu" ~ "love" ~ "thương"    (cùng valence cao)
  "buồn" ~ "sad" ~ "đau"       (cùng valence thấp)

A silk (shared with V): kết nối cường độ
  "yêu" ~ "red heat"           (cùng arousal cao)
  "bình tĩnh" ~ "calm"         (cùng arousal thấp)

T silk (1,024 types): kết nối thời gian
  "nhanh" ~ "allegro"          (cùng tempo)
  "chậm" ~ "adagio"            (cùng tempo)
```

### Silk type = chiều dominant trong khoảng cách

```
silk("yêu", "love"):
  ΔS = nhỏ, ΔR = nhỏ, ΔV = nhỏ nhất, ΔA = nhỏ, ΔT = nhỏ
  → V dominant → silk type = VALENCE silk

silk("o", "ô"):
  ΔS = nhỏ nhất, ΔV = khác hơn
  → S dominant → silk type = SHAPE silk

silk("1+1", "2"):
  ΔR = nhỏ nhất (cùng math)
  → R dominant → silk type = RELATION silk

Không cần enum. Không cần lưu type riêng.
Type XUẤT HIỆN từ phép so sánh 5D.
```

### Nhiều silk cùng lúc giữa 2 nodes

```
"yêu" và "love":
  ~[V silk]~ valence gần (cùng nghĩa tích cực)
  ~[R silk]~ relation (đồng nghĩa cross-language)
  ~[A silk]~ arousal gần (cùng cường độ)

Cùng 2 nodes, nhiều silk types đồng thời.
Mỗi silk = 1 chiều kết nối.
5 chiều → tối đa 5 silk types giữa bất kỳ 2 nodes.
```

### Các vai trò silk trong KnowTree

```
Silk Ln-1 ↔ Ln-1: nối lá với lá (cùng cấp)
  "Hà Nội" ~ "Sài Gòn" (cùng L5: cities)

Silk TRONG nhóm: nối các phần tử cùng nhánh
  facts/geography/Vietnam/cities/* → silk shape nối tất cả

Silk GIỮA nhóm: nối các nhánh khác nhau
  "buồn" ở conversations/session_1 ↔ "mất việc" ở facts/personal

Silk cấu trúc từ: nối characters tạo thành word
  "tôi" = t~ô~i (structural silk, Zipf order)
  ≠ t~o~i (khác nhóm → khác silk → khác từ)

Silk đồng nghĩa cross-language:
  "yêu" ~ "yeu" ~ "love" ~ "aimer"
```

### Lookup = đi theo silk

```
"Hà Nội là gì?"
  → "là gì" = cần silk ĐỊNH NGHĨA (R dimension dominant)
  → Tìm node "Hà Nội"
  → Đi theo R silk → tìm nodes có R relation mạnh nhất
  → "thủ đô Việt Nam" (R silk: is-definition-of)

"Hà Nội ở đâu?"
  → "ở đâu" = cần silk VỊ TRÍ (S dimension: spatial)
  → Cùng node "Hà Nội"
  → Đi theo S silk → tìm nodes có S relation
  → "miền Bắc Việt Nam" (S silk: is-located-at)

"Tại sao buồn?"
  → "tại sao" = cần silk NHÂN QUẢ (R dimension: causes)
  → Node "buồn"
  → Đi theo R silk type CAUSES → "mất việc"

Cùng node, khác silk type → khác câu trả lời.
SILK TYPE IS THE QUERY.
```

### Hebbian learning per silk type

```
co_activate(A, B) fires khi A và B xuất hiện cùng nhau.
Nhưng fire TRÊN CHIỀU NÀO?

"tôi buồn vì mất việc":
  "buồn" và "mất việc" co-activate:
    V silk tăng (cùng valence thấp)
    R silk tăng (quan hệ nhân quả: "vì")
    A silk tăng (cùng arousal)

  Mỗi chiều có weight riêng. Decay riêng (φ⁻¹).

Structural Silk = thứ tự trong chain/array = 0 bytes.
Hebbian Silk = learned per dimension = có weight.
```

---

## B4. QR — Append-Only Proven Knowledge

### Từ origin.md (Go version) — 9 quy tắc bất biến

```
QT7: HỌC / LEAD — QR vs ĐN

  ĐN (Dendrites) = đang học, tự do thay đổi, bộ nhớ ngắn hạn
  QR (Quorum Result) = đã chứng minh, bất biến, cần cấp phép

  Vòng đời: quan sát → ĐN (+/-) → chứng minh → QR (==)
```

### ĐN → QR promotion

```
1. Input mới → encode → STM (ĐN, đang học)
2. Co-activate → Silk tăng weight
3. Fire count tích lũy (Fibonacci: 2, 3, 5, 8, 13, 21...)
4. fire_count ≥ Fib[n] → Dream trigger
5. Dream: cluster STM → LCA → quality check
6. quality ≥ φ⁻¹ (0.618) → propose to AAM
7. AAM approve → append QR record

QR record = { P_w, chain, text, timestamp, signature }
KHÔNG BAO GIỜ xóa. Append-only.
Giống DNA methylation — không xóa, chỉ đánh dấu "không biểu hiện".
```

### Maturity states

```
Formula    → tiềm năng, chưa evaluate (5 chiều = công thức, chưa biết x)
Evaluating → có evidence, đang tích lũy (fire_count > 0)
Mature     → đủ evidence, sẵn sàng QR (weight ≥ φ⁻¹ + fire ≥ Fib[depth])

Formula → Evaluating: khi fire_count > 0
Evaluating → Mature: khi weight ≥ 0.854 AND fire ≥ Fibonacci threshold
Mature → Mature: irreversible (không quay lại)
```

### Supersede (không xóa, chỉ thay thế)

```
QR cũ sai → KHÔNG xóa
→ Tạo QR mới (đúng hơn)
→ Ghi supersede: old_hash → new_hash
→ QR cũ vẫn tồn tại nhưng không biểu hiện

Giống intron trong DNA: không biểu hiện nhưng vẫn tồn tại.
Giữ lịch sử. Có thể quay lại nếu cần.
```

---

## B5. Sai Lệch Đã Sửa

| Hiểu sai | Thực tế | Nguồn |
|----------|---------|-------|
| L0→L1→L2 linear depth | L0 ở giữa: L1←L0→L2+ | Lupin trực tiếp |
| L2→Ln-1 = array lồng | Phân NHÓM NGỮ NGHĨA, mỗi tầng = 1 cấp phân loại | Lupin trực tiếp |
| Silk = 1 loại (weight 0-1) | 9,200 loại silk = UDC chars. Type = dimension dominant | Lupin trực tiếp |
| Silk type cần enum | Type xuất hiện từ phép so sánh 5D, không cần lưu riêng | Lupin trực tiếp |
| Chain link = index vào L3 | Link = P_w = pointer vào BẤT KỲ node (từ, câu, sách) | BLUEPRINT + Lupin |
| KnowTree lookup O(4) cố định | O(depth), depth = số cấp phân nhóm (không cố định) | Logic |
| Silk implicit = miễn phí | O(1) per pair nhưng cần SilkIndex (buckets) cho nhiều nodes | SPEC_NODE_SILK |
| Molecule 5×u8 (Rust) vs u16 (Olang) | Logic giống, bit layout khác. Olang dùng u16 packed | History |

---

> **Tham chiếu:**
> - Chain: `docs/BLUEPRINT.md` §4
> - KnowTree: `docs/tailieu_nghiencuu/KNOWTREE_DESIGN.md`
> - Silk: `docs/tailieu_nghiencuu/SPEC_NODE_SILK.md`, `docs/tailieu_nghiencuu/silk_architecture.md`
> - QR + 9 Rules: `GolandProjects/Origin/origin.md`
> - Node types: `docs/tailieu_nghiencuu/SPEC_NODE_SILK.md` §2
> - Rust implementation: `Origin_project/crates/`
