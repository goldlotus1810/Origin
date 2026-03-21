# S.2 — VẼ HỘP (Box Drawing) · Cây phân loại bằng từ ngữ

> 128 cụm từ Unicode → phân vào cây theo từ khóa
> Nhìn tên → biết "thèn này nét gì, đi hướng nào, dày mỏng ra sao"

---

## Tầng 1: "Nét đi hướng nào?"

```
VẼ HỘP
├── NGANG        "horizontal" — nét ngang ─
├── DỌC          "vertical" — nét dọc │
├── GÓC          "down and right" / "up and left" / ... — góc ┌ ┐ └ ┘
├── GIAO         "vertical and horizontal" — giao cắt ┼
├── NỬA          "left" / "right" (T-junction) — nối 3 hướng ├ ┤ ┬ ┴
└── CUNG         "arc" — bo góc ╭ ╮ ╰ ╯
```

---

## Tầng 2: "Nét dày mỏng ra sao?"

```
ĐỘ DÀY
├── MỎNG        "light" — nét mảnh
├── DÀY         "heavy" — nét đậm
├── ĐÔI         "double" — nét kép ═ ║
└── PHA          "heavy and light" / "light and heavy" — pha trộn dày mỏng
```

---

## Tầng 3: "Góc cụ thể nào?"

```
GÓC (chỉ áp dụng cho kiểu GÓC + NỬA + GIAO)
├── XUỐNG-PHẢI   "down and right" — ┌
├── XUỐNG-TRÁI   "down and left" — ┐
├── LÊN-PHẢI     "up and right" — └
├── LÊN-TRÁI     "up and left" — ┘
├── DỌC-PHẢI     "vertical and right" — ├
├── DỌC-TRÁI     "vertical and left" — ┤
├── NGANG-XUỐNG  "down and horizontal" — ┬
├── NGANG-LÊN    "up and horizontal" — ┴
└── GIAO ĐẦY     "vertical and horizontal" — ┼
```

---

## Phân loại cụ thể 128 cụm

### NGANG (horizontal) — ~6 cụm

```
BOX DRAWINGS LIGHT HORIZONTAL             → ngang + mỏng  ─
BOX DRAWINGS HEAVY HORIZONTAL             → ngang + dày   ━
BOX DRAWINGS DOUBLE HORIZONTAL            → ngang + đôi   ═
BOX DRAWINGS LIGHT TRIPLE DASH HORIZONTAL → ngang + mỏng + gạch 3
BOX DRAWINGS HEAVY TRIPLE DASH HORIZONTAL → ngang + dày + gạch 3
BOX DRAWINGS LIGHT QUADRUPLE DASH HORIZONTAL → ngang + mỏng + gạch 4
→ "ah, nét ngang [mỏng/dày/đôi] [kiểu gạch]"
```

### DỌC (vertical) — ~6 cụm

```
BOX DRAWINGS LIGHT VERTICAL               → dọc + mỏng  │
BOX DRAWINGS HEAVY VERTICAL               → dọc + dày   ┃
BOX DRAWINGS DOUBLE VERTICAL              → dọc + đôi   ║
BOX DRAWINGS LIGHT TRIPLE DASH VERTICAL   → dọc + mỏng + gạch 3
BOX DRAWINGS HEAVY TRIPLE DASH VERTICAL   → dọc + dày + gạch 3
→ "ah, nét dọc [mỏng/dày/đôi]"
```

### GÓC (corner) — ~40 cụm

**Góc xuống-phải (┌):**
```
BOX DRAWINGS LIGHT DOWN AND RIGHT         → ┌ mỏng
BOX DRAWINGS HEAVY DOWN AND RIGHT         → ┌ dày
BOX DRAWINGS DOUBLE DOWN AND RIGHT        → ┌ đôi
BOX DRAWINGS DOWN LIGHT AND RIGHT HEAVY   → ┌ dọc mỏng + ngang dày
BOX DRAWINGS DOWN HEAVY AND RIGHT LIGHT   → ┌ dọc dày + ngang mỏng
BOX DRAWINGS DOWN DOUBLE AND RIGHT SINGLE → ┌ dọc đôi + ngang đơn
BOX DRAWINGS DOWN SINGLE AND RIGHT DOUBLE → ┌ dọc đơn + ngang đôi
→ "ah, góc xuống-phải [dày/mỏng/đôi/pha]"
```

**Góc xuống-trái (┐):**
```
BOX DRAWINGS LIGHT DOWN AND LEFT          → ┐ mỏng
BOX DRAWINGS HEAVY DOWN AND LEFT          → ┐ dày
BOX DRAWINGS DOUBLE DOWN AND LEFT         → ┐ đôi
BOX DRAWINGS DOWN LIGHT AND LEFT HEAVY    → ┐ pha
BOX DRAWINGS DOWN HEAVY AND LEFT LIGHT    → ┐ pha
→ "ah, góc xuống-trái [dày/mỏng/đôi/pha]"
```

**Góc lên-phải (└):**
```
BOX DRAWINGS LIGHT UP AND RIGHT           → └ mỏng
BOX DRAWINGS HEAVY UP AND RIGHT           → └ dày
BOX DRAWINGS DOUBLE UP AND RIGHT          → └ đôi
BOX DRAWINGS UP LIGHT AND RIGHT HEAVY     → └ pha
BOX DRAWINGS UP HEAVY AND RIGHT LIGHT     → └ pha
→ "ah, góc lên-phải [dày/mỏng/đôi/pha]"
```

**Góc lên-trái (┘):**
```
BOX DRAWINGS LIGHT UP AND LEFT            → ┘ mỏng
BOX DRAWINGS HEAVY UP AND LEFT            → ┘ dày
BOX DRAWINGS DOUBLE UP AND LEFT           → ┘ đôi
BOX DRAWINGS UP LIGHT AND LEFT HEAVY      → ┘ pha
BOX DRAWINGS UP HEAVY AND LEFT LIGHT      → ┘ pha
→ "ah, góc lên-trái [dày/mỏng/đôi/pha]"
```

### T-JUNCTION (nửa) — ~40 cụm

**T dọc-phải (├):**
```
BOX DRAWINGS LIGHT VERTICAL AND RIGHT     → ├ mỏng
BOX DRAWINGS HEAVY VERTICAL AND RIGHT     → ├ dày
BOX DRAWINGS DOUBLE VERTICAL AND RIGHT    → ├ đôi
BOX DRAWINGS VERTICAL LIGHT AND RIGHT HEAVY → ├ pha
BOX DRAWINGS VERTICAL HEAVY AND RIGHT LIGHT → ├ pha
→ "ah, T dọc-phải [dày/mỏng/đôi/pha]"
```

**T dọc-trái (┤):**
```
BOX DRAWINGS LIGHT VERTICAL AND LEFT      → ┤ mỏng
BOX DRAWINGS HEAVY VERTICAL AND LEFT      → ┤ dày
BOX DRAWINGS DOUBLE VERTICAL AND LEFT     → ┤ đôi
→ "ah, T dọc-trái [dày/mỏng/đôi/pha]"
```

**T ngang-xuống (┬):**
```
BOX DRAWINGS LIGHT DOWN AND HORIZONTAL    → ┬ mỏng
BOX DRAWINGS HEAVY DOWN AND HORIZONTAL    → ┬ dày
BOX DRAWINGS DOUBLE DOWN AND HORIZONTAL   → ┬ đôi
BOX DRAWINGS DOWN HEAVY AND HORIZONTAL LIGHT → ┬ pha
→ "ah, T ngang-xuống [dày/mỏng/đôi/pha]"
```

**T ngang-lên (┴):**
```
BOX DRAWINGS LIGHT UP AND HORIZONTAL      → ┴ mỏng
BOX DRAWINGS HEAVY UP AND HORIZONTAL      → ┴ dày
BOX DRAWINGS DOUBLE UP AND HORIZONTAL     → ┴ đôi
→ "ah, T ngang-lên [dày/mỏng/đôi/pha]"
```

### GIAO CẮT (cross) — ~12 cụm

```
BOX DRAWINGS LIGHT VERTICAL AND HORIZONTAL     → ┼ mỏng
BOX DRAWINGS HEAVY VERTICAL AND HORIZONTAL     → ┼ dày
BOX DRAWINGS DOUBLE VERTICAL AND HORIZONTAL    → ┼ đôi
BOX DRAWINGS VERTICAL LIGHT AND HORIZONTAL HEAVY → ┼ dọc mỏng + ngang dày
BOX DRAWINGS VERTICAL HEAVY AND HORIZONTAL LIGHT → ┼ dọc dày + ngang mỏng
BOX DRAWINGS VERTICAL DOUBLE AND HORIZONTAL SINGLE → ┼ dọc đôi + ngang đơn
BOX DRAWINGS VERTICAL SINGLE AND HORIZONTAL DOUBLE → ┼ dọc đơn + ngang đôi
→ "ah, giao cắt [dày/mỏng/đôi/pha]"
```

### CUNG (arc) — ~4 cụm

```
BOX DRAWINGS LIGHT ARC DOWN AND RIGHT     → ╭ bo góc mỏng
BOX DRAWINGS LIGHT ARC DOWN AND LEFT      → ╮ bo góc mỏng
BOX DRAWINGS LIGHT ARC UP AND LEFT        → ╯ bo góc mỏng
BOX DRAWINGS LIGHT ARC UP AND RIGHT       → ╰ bo góc mỏng
→ "ah, bo góc [hướng]"
```

### ĐẶC BIỆT — ~8 cụm

```
BOX DRAWINGS LIGHT DIAGONAL UPPER RIGHT TO LOWER LEFT  → chéo ╲
BOX DRAWINGS LIGHT DIAGONAL CROSS                       → chéo cắt ╳
BOX DRAWINGS LIGHT LEFT                                 → nửa ngang trái
BOX DRAWINGS LIGHT RIGHT                                → nửa ngang phải
BOX DRAWINGS LIGHT UP                                   → nửa dọc trên
BOX DRAWINGS LIGHT DOWN                                 → nửa dọc dưới
→ "ah, nét đặc biệt [chéo/nửa]"
```

---

## Từ khóa → Tầng (cheat sheet)

| Thấy từ này | → Tầng | → Giá trị |
|-------------|--------|----------|
| horizontal | 1 | NGANG |
| vertical | 1 | DỌC |
| down and right/left, up and right/left | 1 | GÓC |
| vertical and horizontal | 1 | GIAO |
| vertical and right/left | 1 | T-DỌC |
| down/up and horizontal | 1 | T-NGANG |
| arc | 1 | CUNG |
| light | 2 | MỎNG |
| heavy | 2 | DÀY |
| double | 2 | ĐÔI |
| X light and Y heavy | 2 | PHA (X mỏng Y dày) |

---

## Tóm tắt

```
128 cụm box drawing → tuple (kiểu_nét, độ_dày, hướng_góc)

"BOX DRAWINGS" + [LIGHT/HEAVY/DOUBLE] + [hướng]
→ "ah, vẽ hộp [mỏng/dày/đôi] [ngang/dọc/góc/T/giao/cung]"

Mọi cụm đều theo pattern:
  BOX DRAWINGS {weight} {direction}
  BOX DRAWINGS {dir1} {weight1} AND {dir2} {weight2}
```
