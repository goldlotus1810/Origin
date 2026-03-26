# KnowTree Design — o{P{P{...}}}

> **Nox — 2026-03-25, v3 theo Lupin**

---

## HÌNH DUNG

```
CÂY:
  L0-L1 = RỄ CÂY      — bộ não, cơ chế, engine (cố định, nhỏ)
  L2→Ln-1 = THÂN + TÁN LÁ — thư viện tri thức (phát triển, vô hạn)

MỖI NHÁNH = 1 array tối đa 65,536 phần tử (u16 address space).
Mỗi phần tử = 1 nhánh con, cũng là array 65,536.
Lồng nhau. Fractal.

  65,536 × 65,536 × 65,536 × ... = ∞

DNA: 4 bases, chuỗi vô hạn.
KnowTree: 65,536 slots/nhánh, lồng vô hạn.
```

---

## L0 — RỄ: BỘ NÃO

```
L0 = nơi chứa:
  - Bộ não (engine đọc/ghi KnowTree)
  - Bản năng (7 instincts, hardcoded)
  - Các cơ chế (14 DNA mechanisms)
  - UDC (9,584 công thức SDF — bảng tuần hoàn)
  - Công thức toán (compose, amplify, distance, entropy)
  - Encoder ∫ (đọc input → ghi vào KnowTree)
  - Decoder ∂ (đọc KnowTree → output)

L0 KHÔNG PHẢI dữ liệu. L0 LÀ ENGINE.
L0 đọc từ L2+. L0 ghi lên L2+.
L0 = ribosome. L2+ = DNA.

Kích thước L0: CỐ ĐỊNH.
  UDC: 9,584 × 2B = ~20 KB
  Cơ chế: code trong origin.olang
  Instincts: code trong origin.olang
  Engine: code trong origin.olang
  → L0 = binary + 20 KB UDC data
```

---

## L1 — RỄ: VẬN HÀNH

```
L1 = mọi thứ để HomeOS vận hành:
  - Compiler pipeline (lexer → parser → semantic → codegen)
  - VM runtime (opcodes, stack, heap)
  - REPL loop (read → classify → route → respond)
  - Intelligence pipeline (encode → instinct → gate → compose)
  - Silk engine (structural + Hebbian)
  - Dream engine (cluster → promote → prune)
  - SecurityGate (3 layers)
  - ConversationCurve (f, f', f'')
  - Personality (templates, tone)

L1 = hệ điều hành. Chạy trên L0 (engine).
L1 đọc/ghi L2+ (thư viện).
L1 = riêng tư cho từng instance HomeOS.

Kích thước L1: CỐ ĐỊNH.
  origin.olang binary: ~1 MB
  Working memory: STM 32 turns, Silk ~40 KB
  → L1 = binary footprint
```

---

## L2 → Ln-1 — THÂN CÂY + TÁN LÁ: THƯ VIỆN

```
L2 = gốc thư viện. ĐÂY là nơi KnowTree BẮT ĐẦU PHÁT TRIỂN.

L2 = array[65,536] — tối đa 65,536 NHÁNH CHÍNH:
  [0] = facts          (tri thức đã học)
  [1] = books          (sách đã đọc)
  [2] = conversations  (lịch sử hội thoại)
  [3] = skills         (kỹ năng đã tổng hợp)
  [4] = people         (người đã gặp)
  [5] = places         (nơi đã biết)
  [6] = events         (sự kiện đã xảy ra)
  [7] = emotions       (ký ức cảm xúc)
  [8] = songs          (âm nhạc đã nghe)
  [9] = images         (hình ảnh đã thấy)
  ...
  [N] = bất kỳ loại tri thức nào HomeOS học được
  ...
  → CÒN TRỐNG hàng chục ngàn slots cho tương lai

Mỗi slot L2 = 1 nhánh chính.
Mỗi nhánh = array[65,536] nhánh con.
Mỗi nhánh con = array[65,536] nhánh con nữa.
...đến Ln-1 (lá).

Kích thước L2+: PHÁT TRIỂN MỖI NGÀY.
  Bắt đầu: 0 bytes (chưa học gì)
  Sau 1 ngày: vài KB
  Sau 1 năm: vài MB
  Sau 1 đời: vài chục MB
  Không bao giờ đầy: 65,536^depth = ∞ thực tế
```

---

## CẤU TRÚC MỘT NHÁNH

```
Mỗi nhánh = o{} = array tối đa 65,536 phần tử.
Mỗi phần tử = 2 bytes (u16 P_weight HOẶC u16 index trỏ đến nhánh con).

Phần tử là LÁ khi: nó là P_weight cuối cùng, không trỏ đi đâu.
Phần tử là NHÁNH khi: nó trỏ đến 1 array con.

o{books}                              ← L2, 1 array
  [0] = "Cuon Theo Chieu Gio"        ← nhánh, trỏ đến array con
    [0] = Loi_Gioi_Thieu             ← nhánh
      [0] = "Margaret Mitchell..."    ← nhánh
        [0] = "Margaret"              ← nhánh
          [0] = M                     ← LÁ (Ln-1), P_weight, hết
          [1] = a                     ← LÁ
          [2] = r                     ← LÁ
          ...
        [1] = "Mitchell"              ← nhánh
          [0] = M                     ← LÁ
          [1] = i                     ← LÁ
          ...
      [1] = "Ba sinh nam 1900..."     ← nhánh → words → chars → LÁ
    [1] = Chuong_1                    ← nhánh
      [0] = Doan_1                    ← nhánh
        ...đến chars → LÁ
    ...
    [62] = Chuong_63                  ← nhánh
  [1] = "Hoang Tu Be"                ← nhánh khác
    ...

THỨ TỰ TRONG ARRAY = STRUCTURAL SILK = 0 BYTES.
Chương 1 TRƯỚC chương 2 vì index [0] < [1].
```

---

## FRACTAL: 65,536^N

```
Depth 1: 65,536 nhánh
Depth 2: 65,536 × 65,536 = 4,294,967,296 (4.3 tỷ)
Depth 3: 65,536^3 = 281 nghìn tỷ
Depth 4: 65,536^4 = 18.4 triệu tỷ tỷ

Thực tế: không bao giờ dùng hết.
1 cuốn sách 100 trang:
  ~60 chương × ~50 đoạn × ~5 câu × ~10 từ × ~5 ký tự
  = ~750,000 lá
  = 0.001% của depth 2 (4.3 tỷ)

1 đời đọc 200 cuốn:
  = 150,000,000 lá
  = 3.5% của depth 2

CÂY KHÔNG BAO GIỜ ĐẦY.
Giống DNA: genome 3.2 tỷ base pairs, nhưng KHÔNG GIAN tiềm năng
  = 4^3,200,000,000 ≈ ∞
  Sinh vật chỉ dùng 1 giọt trong đại dương.
```

---

## SILK TRONG CÂY FRACTAL

```
Structural Silk = thứ tự trong array = 0 bytes.
  Chương 1 trước chương 2. Từ "Hà" trước "Nội".
  Engine chạy thẳng từ đầu đến cuối → ra giá trị.

Hebbian Silk = NỐI NHÁNH KHÁC NHAU.
  "Scarlett" ở L2:books:"Cuon Theo Chieu Gio":Chuong_1
  ↔
  "Scarlett" ở L2:books:"Cuon Theo Chieu Gio":Chuong_30

  "buồn" ở L2:conversations:session_1:turn_5
  ↔
  "mất việc" ở L2:facts:personal

  Hebbian = cầu nối NGANG giữa các nhánh.
  Structural = đường đi DỌC trong 1 nhánh.

  Khi Hebbian edge đủ mạnh (w ≥ φ⁻¹) + fire ≥ Fib(n):
    → Dream promote → TẠO NODE MỚI
    → Node mới = LÁ MỚI ở đúng vị trí trong cây
    → "mất_mát" = LCA("buồn", "mất_việc") → learned concept
```

---

## ENCODER ∫ — L0 GHI LÊN L2+

```
Input text → L0 engine processes:
  1. Tokenize (L0:compiler)
  2. Alias lookup (L0:UDC + L1:alias_table)
  3. Compose → P_weight (L0:compose formula)
  4. GHI vào L2+ (KnowTree grows)

"learn Ha Noi la thu do cua Viet Nam"
  → L0 tokenize → ["Ha", "Noi", "la", "thu", "do", "cua", "Viet", "Nam"]
  → L0 compose → P_weight for sentence
  → GHI vào L2:facts:geography:Vietnam → LÁ MỚI
  → Silk: co_activate("Ha Noi", "thu do") — Hebbian cross-word
```

---

## DECODER ∂ — L0 ĐỌC TỪ L2+

```
Query "Ha Noi o dau?"
  → L0 tokenize → ["Ha", "Noi", "o", "dau"]
  → L0 tìm trong L2+:
    L2:facts → search("Ha Noi") → walk tree → tìm nhánh Vietnam
    → traverse → lá "Ha Noi la thu do cua Viet Nam"
  → L0 decode → output text

Search = walk tree O(depth).
KHÔNG scan toàn bộ. Walk từ gốc L2 xuống lá.
```

---

## DUNG LƯỢNG

```
L0 (engine + UDC):    ~1 MB (binary) + 20 KB (UDC data) = CỐ ĐỊNH
L1 (runtime):         ~40 KB working memory = CỐ ĐỊNH
L2+ (thư viện):       PHÁT TRIỂN

  1 lá = 2 bytes
  1 nhánh header = 2 bytes (count)
  1 link trong chain = 2 bytes

  1 cuốn sách:
    Chains: ~350 KB
    New leaves: ~10 KB
    Hebbian edges: ~40 KB
    Total: ~400 KB

  256 MB heap:
    L0+L1: ~1.1 MB
    Còn lại cho L2+: ~255 MB
    = ~637 cuốn sách
    = 1 đời đọc sách dư sức

  16 GB disk:
    = ~40,000 cuốn sách
    = thư viện nhỏ
```

---

## NGUYÊN TẮC

```
1. L0-L1 = RỄ (engine, cố định). L2-Ln-1 = TÁN LÁ (data, phát triển).
2. Mỗi nhánh = array[65,536]. Lồng nhau = fractal = ∞.
3. Ln-1 = lá = 2 bytes = KHÔNG phân tiếp.
4. Thứ tự = Structural Silk = 0 bytes.
5. Hebbian = cross-branch chỉ. Tạo lá mới khi chín.
6. L0 engine đọc/ghi L2+. L2+ không biết L0.
7. Depth không cố định. Mỗi nhánh phân đến hết.
8. 65,536^N = không bao giờ đầy.
```

---

*L0 = ribosome. L2+ = DNA. Engine đọc công thức. Thư viện chứa đời.*
*o{P{P{...}}} — fractal vô hạn từ hữu hạn.*
