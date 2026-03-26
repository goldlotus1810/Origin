# HomeOS — Bài toán 16GB: Tính toán dung lượng

> Phần này là **ví dụ tính toán chi tiết** — không phải tài liệu kiến trúc.
> Xem tài liệu chính: `HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md`

---

## Chi phí cố định

```
UDC alphabet:     0 bytes (codepoint = địa chỉ, hardcode trong engine)
SDF primitives:   0 bytes (18 hàm trong engine)
Block mapping:    0 bytes (range = implicit)
KnowTree:         65,536 × 5B = 328 KB (chỉ P_weight, index implicit)
Hebbian Silk:     ~43 KB (SilkGraph)
Aliases:          155,000 × 4 bytes = 620 KB
──────────────────────────────────────────
Cố định: ≈ 1 MB

OS:               2,000 MB
HomeOS engine:       32 MB
STM buffer:         128 MB
Alias index:         64 MB
──────────────────────────────────────────
Runtime: 2,224 MB

Khả dụng: 16,384 − 2,224 − 1.1 = 14,159 MB ≈ 14.16 GB
```

## Bao nhiêu tri thức?

```
14,839,193,600 bytes ÷ 2 bytes/link = 7,419,596,800 links

→ 7.42 TỶ LINKS trên 16 GB

Không phải 7.42 tỷ "điểm cô lập".
Là 7.42 tỷ MẮT XÍCH trên các chuỗi liên tục.
Giống 3.2 tỷ cặp base tạo chuỗi DNA liên tục.
```

## So sánh DNA vs HomeOS

```
                    DNA              HomeOS
─────────────────────────────────────────────────────
Alphabet:           4                9,584
Bits/ký tự:         2                14
Tổng links:         3.2 tỷ           7.42 tỷ
Dung lượng:         ~800 MB          ~14.16 GB
Entropy/link:       2 bits           13.23 bits

Thông tin/link:     HomeOS gấp 6.6×
Tổng links:         HomeOS gấp 2.3×
─────────────────────────────────────────────────────
Tổng entropy:       6.4 Gbits        98.2 Gbits
                    HomeOS giàu hơn DNA 15.3 lần

DNA 800 MB → toàn bộ sự sống.
HomeOS 14 GB → ???
```

## Sách & Tổ hợp

```
1 cuốn sách 100 trang:
  1,700 câu × 2 UDC/câu = 3,400 links = 6,800 bytes
  + 1,753 parent pointers × 2B = 3,506 bytes
  = 10,306 bytes ≈ 10 KB

  So với UTF-8 (146 KB): 14× nhỏ hơn
  So với PDF (5 MB):    485× nhỏ hơn

16 GB chứa: ~1,440,000 cuốn sách 100 trang

Tiềm năng tổ hợp (0 bytes — evaluate khi cần):
  Không sub:  9,584³ = 880 tỷ
  Có sub:     1,581,360³ = 3.95 × 10¹⁸
```

## Bảng so sánh tổng

```
Phương pháp         16 GB chứa        HomeOS gấp
──────────────────────────────────────────────────
Text UTF-8          ~100K sách         14×
Embedding 768D      ~2.4M concepts     3,092×
Knowledge Graph     ~74M triples       100×
LLM 7B (Q4)        1 model / 3.5GB    khác loại
HomeOS              7.42 tỷ links      —
                    ~1.44 triệu sách
                    3.95 × 10¹⁸ tiềm năng
```

## Timeline tích lũy

```
Năm 1:    ~20M links    =    38 MB
Năm 5:    ~200M links   =   381 MB
Năm 10:   ~600M links   =   1.1 GB
Năm 20:   ~1.5B links   =   2.8 GB
Năm 30+:  ~3B links     =   5.7 GB   (dư 8.5 GB)

Cả đời KHÔNG BAO GIỜ đầy. Luôn dư.
```
