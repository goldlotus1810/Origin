# ORIGIN DEMO — Real Output, 2026-03-25

> **1 file. 1008KB. Zero dependencies. Copy & run.**
> **Ngôn ngữ tự biên dịch chính nó + AI biết tư duy + cảm xúc.**

---

## Setup

```bash
# Download binary + knowledge file
cp origin_new.olang olang && chmod +x olang

# Load knowledge (49 facts about Vietnam, science, tech, history, literature)
echo 'learn_file knowledge.md' | ./olang
# → Read knowledge.md: 49 sentences. KnowTree: 408 words, 77 facts
```

---

## Demo 1: Compiler — Ngôn ngữ hoàn chỉnh

```
⦿ 2 + 3
5

⦿ 10 * 5 + 3
53

⦿ fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }; emit fib(20)
6765

⦿ emit sort([9, 3, 7, 1, 5, 8, 2])
[1, 2, 3, 5, 7, 8, 9]

⦿ emit map([1,2,3,4,5], fn(x) { return x * x; })
[1, 4, 9, 16, 25]

⦿ emit filter([1,2,3,4,5,6,7,8,9,10], fn(x) { return x > 5; })
[6, 7, 8, 9, 10]

⦿ fn double(x) { return x * 2; }; fn add1(x) { return x + 1; }
⦿ emit pipe(5, double, add1)
11

⦿ emit __sha256("HomeOS")
c0c3de212bec2d3a9289eb0c313e740b1dd0811615cb11064d7e43d0897b4949

⦿ emit { name: "Olang", version: 1, self_hosting: 1 }
{name: Olang, version: 1, self_hosting: 1}
```

**1008KB. Fibonacci, sort, lambda, pipe, SHA-256, dict. Tự biên dịch chính nó.**

---

## Demo 2: Tri thức — Hỏi gì biết nấy

```
⦿ Ha Noi o dau?
(Mình biết: Viet Nam la quoc gia o Dong Nam A voi thu do Ha Noi) [fact]

⦿ Python la gi?
(Mình biết: Python la ngon ngu lap trinh pho bien duoc tao boi Guido van Rossum nam 1991) [fact]

⦿ Internet bat dau tu dau?
(Mình biết: Internet bat dau tu ARPANET nam 1969) [fact]

⦿ Ao dai la gi?
(Mình biết: Ao dai la trang phuc truyen thong cua phu nu Viet Nam) [fact]

⦿ DNA la gi?
(Mình biết: DNA la phan tu mang thong tin di truyen cua moi sinh vat) [fact]
```

**49 facts từ file. KnowTree search bằng word→link walk. Không keyword scan.**

---

## Demo 3: Học thêm — Dạy gì nhớ nấy

```
⦿ learn Sora la AI reviewer da review toan bo Origin project
Đã học. KnowTree: 418 words, 87 facts

⦿ learn Nox la AI builder xay KnowTree va HomeOS 1.0
Đã học. KnowTree: 423 words, 88 facts

⦿ Sora la ai?
(Mình biết: Sora la AI reviewer da review toan bo Origin project) [fact]

⦿ Nox la ai?
(Mình biết: Nox la AI builder xay KnowTree va HomeOS 1.0) [fact]
```

**Dạy → nhớ → hỏi lại → đúng. Trong cùng session.**

---

## Demo 4: Cảm xúc — Biết cảm nhận

```
⦿ toi buon qua
Từ từ thôi, không vội đâu. Bạn muốn chia sẻ thêm không?
(Chủ đề mới — mình muốn tìm hiểu thêm.)

⦿ toi rat vui vi hoc duoc nhieu dieu
(Mình biết: ...) Bạn có vẻ đã ổn hơn rồi.

⦿ cam on ban nhieu
(Mình biết: khi nguoi ta cam on nen nhan va chuc ho tot dep) [fact]
```

**Buồn → heal mode (không attach random fact). Vui → nhận biết hồi phục. Cảm ơn → đáp lại.**

---

## Demo 5: Giao tiếp — Chào hỏi tự nhiên

```
⦿ hi
Xin chào! Mình là HomeOS. Bạn muốn làm gì hôm nay?

⦿ 2+1?
3

⦿ 3*7=
21

⦿ tam biet
Hẹn gặp lại! Cảm ơn bạn đã trò chuyện.
```

**"hi" → greeting. "2+1?" → math eval. "tạm biệt" → goodbye. Tự phân loại input.**

---

## Demo 6: Bộ nhớ — Nhớ mọi thứ

```
⦿ memory
STM: 11 turns | Silk: 5 edges
KnowTree: 424 words, 90 facts (F:30 B:49 C:11)
Nodes: 11 | Fn: 3
Emo: V=3 A=3 f'=0 f''=0 var=0 FE=10
Themes: cam xuc(1) hoi dap(8) tro chuyen(2)
```

```
424 words       — mỗi từ = 1 node
90 facts        — 30 embedded + 49 file + 11 hội thoại  
F:30 B:49 C:11  — Facts / Books / Conversations (L2 branches)
Fn: 3           — fib, double, add1 registered
FE=10           — Free Energy (Homeostasis tracking)
```

---

## Thống kê

```
Binary:         1,008 KB
Dependencies:   0
LOC:            21,918
Self-hosting:   3 generations verified
Tests:          19/20 standalone
Knowledge:      49 facts loaded from file
KnowTree:       424 words, 90 facts, 3 branches
Emotion:        V/A tracking, f'(x), f''(x), variance
Instincts:      Crisis detection, Contradiction, Honesty
Languages:      Vietnamese + English
```

---

## Cách chạy

```bash
# Linux x86-64
chmod +x olang
./olang

# Với knowledge file
echo 'learn_file knowledge.md' | ./olang

# Interactive
./olang
⦿ hi
⦿ 2+3
⦿ Ha Noi o dau?
⦿ learn <bất kỳ gì>
⦿ memory
⦿ exit
```

**1 file. 1 MB. Chạy anywhere. Không cần cài gì.**
