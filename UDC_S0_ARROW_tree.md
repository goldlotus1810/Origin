# S.0 — MŨI TÊN (Arrow) · Cây phân loại bằng từ ngữ

> Nhìn vào từ khóa → biết ngay "thèn này thuộc nhóm nào"
> Mỗi tầng = 1 câu hỏi. Trả lời = chọn nhánh.

---

## Tầng 1: "Nó là kiểu mũi tên gì?"

```
MŨI TÊN
├── ĐƠN         "arrow" — 1 đầu nhọn
├── ĐÔI          "double arrow" — 2 đầu nhọn ⇔
├── BA            "triple arrow" — 3 nét ⇛
├── MÓC           "harpoon" — nửa đầu nhọn ⇀
├── GẠCH NGANG   "dashed arrow" — nét đứt ⇢
├── LƯỢN SÓNG    "wave/squiggle arrow" — nét lượn ↝
├── VÒNG          "circular/anticlockwise" — xoay vòng ↺
└── ĐẶC BIỆT     "bent/hook/loop/curved" — gấp khúc ↩
```

---

## Tầng 2: "Nó chỉ hướng nào?"

```
(áp dụng cho MỌI kiểu ở tầng 1)

HƯỚNG
├── LÊN            "upwards" ↑
├── XUỐNG           "downwards" ↓
├── TRÁI            "leftwards" ←
├── PHẢI            "rightwards" →
├── LÊN-PHẢI        "upper right / north east" ↗
├── LÊN-TRÁI        "upper left / north west" ↖
├── XUỐNG-PHẢI       "lower right / south east" ↘
├── XUỐNG-TRÁI       "lower left / south west" ↙
├── TRÁI-PHẢI        "left right" ↔  (2 chiều ngang)
├── LÊN-XUỐNG        "up down" ↕     (2 chiều dọc)
└── TẤT CẢ           "four directions" — 4 hướng
```

---

## Tầng 3: "Nét nó dày mỏng ra sao?"

```
ĐỘ DÀY (weight)
├── MỎNG         "light" — nét mảnh
├── THƯỜNG        (không ghi gì) — nét bình thường
├── DÀY           "heavy" — nét đậm
└── RẤT DÀY       "very heavy / bold" — cực đậm
```

---

## Tầng 4: "Nó đặc hay rỗng?"

```
KIỂU TÔ (fill)
├── ĐẶC          "filled / black" — tô kín ▶
├── RỖNG          "open / white" — chỉ viền ▷
├── NỬA           "half-filled" — tô nửa
└── BÓNG          "shadowed" — có bóng đổ
```

---

## Tầng 5: "Nó có đuôi gì?"

```
ĐUÔI (tail)
├── KHÔNG ĐUÔI    "arrowhead only" — chỉ mũi nhọn ➤
├── ĐUÔI THẲNG    (mặc định) — thân thẳng →
├── ĐUÔI CONG     "curved" — thân cong ⤵
├── ĐUÔI GẤP      "bent" — gấp khúc ↳
└── ĐUÔI MÓC      "hook / barb" — có móc ↩
```

---

## Ví dụ đọc: Từ tên → biết nhóm

```
"RIGHTWARDS ARROW"
  Tầng 1: ĐƠN (arrow, không double/triple)
  Tầng 2: PHẢI (rightwards)
  Tầng 3: THƯỜNG (không light/heavy)
  Tầng 4: — (không filled/open)
  Tầng 5: ĐUÔI THẲNG (mặc định)
  → Nhìn vào biết: "mũi tên đơn, hướng phải, nét thường"

"HEAVY WIDE-HEADED SOUTH EAST ARROW"
  Tầng 1: ĐƠN
  Tầng 2: XUỐNG-PHẢI (south east)
  Tầng 3: DÀY (heavy)
  Tầng 4: — (wide-headed = đầu to, modifier phụ)
  Tầng 5: ĐUÔI THẲNG
  → "mũi tên đơn, xuống-phải, nét dày, đầu to"

"LEFT RIGHT OPEN-HEADED ARROW"
  Tầng 1: ĐƠN
  Tầng 2: TRÁI-PHẢI (left right)
  Tầng 3: THƯỜNG
  Tầng 4: RỖNG (open-headed)
  Tầng 5: ĐUÔI THẲNG
  → "mũi tên đơn, 2 chiều ngang, rỗng"

"ANTICLOCKWISE TOP SEMICIRCLE ARROW"
  Tầng 1: VÒNG (anticlockwise)
  Tầng 2: LÊN (top)
  Tầng 3: THƯỜNG
  Tầng 4: —
  Tầng 5: — (semicircle = hình dạng thân)
  → "mũi tên vòng ngược chiều kim đồng hồ, nửa trên"

"LEFTWARDS HARPOON WITH BARB DOWNWARDS"
  Tầng 1: MÓC (harpoon)
  Tầng 2: TRÁI (leftwards)
  Tầng 3: THƯỜNG
  Tầng 4: —
  Tầng 5: barb downwards = ngạnh hướng xuống (modifier phụ)
  → "mũi tên móc, hướng trái, ngạnh dưới"
```

---

## Tổng: 5 tầng × số nhánh

```
Tầng 1 — KIỂU:     8 nhánh
Tầng 2 — HƯỚNG:    11 nhánh
Tầng 3 — ĐỘ DÀY:   4 nhánh
Tầng 4 — TÔ:        4 nhánh
Tầng 5 — ĐUÔI:      5 nhánh

Tổ hợp lý thuyết: 8 × 11 × 4 × 4 × 5 = 7,040 khả năng
Unicode thực tế:   ~618 ký tự mũi tên (không dùng hết tổ hợp)

Mỗi ký tự mũi tên = 1 tuple (kiểu, hướng, độ_dày, tô, đuôi)
```

---

## Từ khóa → Tầng (cheat sheet)

| Thấy từ này trong tên | → Thuộc tầng | → Giá trị |
|----------------------|-------------|----------|
| arrow | 1 | ĐƠN |
| double | 1 | ĐÔI |
| triple | 1 | BA |
| harpoon | 1 | MÓC |
| dashed | 1 | GẠCH NGANG |
| wave, squiggle | 1 | LƯỢN SÓNG |
| clockwise, anticlockwise | 1 | VÒNG |
| bent, hook, loop, curved | 1 | ĐẶC BIỆT |
| upwards, up, north | 2 | LÊN |
| downwards, down, south | 2 | XUỐNG |
| leftwards, left, west | 2 | TRÁI |
| rightwards, right, east | 2 | PHẢI |
| upper right, north east | 2 | LÊN-PHẢI |
| upper left, north west | 2 | LÊN-TRÁI |
| lower right, south east | 2 | XUỐNG-PHẢI |
| lower left, south west | 2 | XUỐNG-TRÁI |
| left right | 2 | TRÁI-PHẢI |
| up down | 2 | LÊN-XUỐNG |
| light | 3 | MỎNG |
| heavy, bold | 3 | DÀY |
| very heavy | 3 | RẤT DÀY |
| filled, black | 4 | ĐẶC |
| open, white | 4 | RỖNG |
| half | 4 | NỬA |
| shadowed | 4 | BÓNG |
| barb | 5 | MÓC |
| curved | 5 | CONG |
| bent | 5 | GẤP |
