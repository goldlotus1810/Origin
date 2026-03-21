# V — VALENCE (Tích cực / Tiêu cực) · Cây phân loại + Công thức vật lý

> 5 mức từ vựng + 7 nhóm cụm từ
> Nhìn từ → biết ngay "thèn này tích cực hay tiêu cực, mức nào"
> Mỗi nhóm = một dạng thế năng (potential energy) có công thức vật lý riêng

---

## Mô hình vật lý tổng quát

Valence = **thế năng trong trường lực** — hệ bị hút (approach) hay bị đẩy (avoidance):

```
  Mô hình: Hạt trong giếng thế (particle in potential well)

              ⎧  −V₀ · sech²(x/a)    nếu hút (tích cực)   → giếng thế
  U(x) =      ⎨  0                    nếu trung tính       → mặt phẳng
              ⎩  +V₀ · e^(−x²/2σ²)   nếu đẩy (tiêu cực)   → rào thế

  Lực: F(x) = −dU/dx
    F > 0 (hút vào)  → tích cực  → hệ muốn ĐẾN GẦN
    F = 0 (phẳng)    → trung tính → hệ KHÔNG CÓ xu hướng
    F < 0 (đẩy ra)   → tiêu cực  → hệ muốn TRÁNH XA

  Valence V(w) = −U(w) / U_max  ∈ [-1, +1]
    V → +1  khi U rất âm (giếng sâu → hút mạnh → rất tích cực)
    V →  0  khi U ≈ 0    (phẳng → trung tính)
    V → -1  khi U rất dương (rào cao → đẩy mạnh → rất tiêu cực)
```

---

## Tầng 1: "Tích cực hay tiêu cực?" — 5 vùng thế năng

```
VALENCE
├── RẤT TÍCH CỰC    V > +0.5   (2618 từ, 191 cụm)  │ U << 0 (giếng sâu, attractor)
├── TÍCH CỰC         +0.2 < V ≤ +0.5  (1831 từ)     │ U < 0  (giếng nông)
├── TRUNG TÍNH       -0.2 ≤ V ≤ +0.2  (221 từ)      │ U ≈ 0  (mặt phẳng)
├── TIÊU CỰC         -0.5 < V < -0.2  (1531 từ, 141 cụm) │ U > 0 (rào thấp)
└── RẤT TIÊU CỰC    V ≤ -0.5  (1784 từ)             │ U >> 0 (rào cao, repeller)
```

### Hàm ánh xạ thế năng → valence:

```
  V(w) = −tanh(U(w) / U_ref)

  Trong đó U_ref = ngưỡng thế năng tham chiếu (reference potential)

  Profile thế năng trên trục V:

   U(x)
    ↑  rào thế (repulsive)
    │  ╱╲
    │ ╱  ╲                        tiêu cực → hệ bị ĐẨY
   ─┼──────────────────────── U = 0 (trung tính)
    │          ╲  ╱               tích cực → hệ bị HÚT
    │           ╲╱
    ↓  giếng thế (attractive)
```

---

## Tầng 2: "Thuộc chủ đề gì?"

### RẤT TÍCH CỰC — Giếng thế sâu (deep potential well)

> **Mô hình:** Thế năng rất âm — hệ bị hút mạnh vào trạng thái ổn định.
> Giống hạt rơi vào giếng thế sâu: cần rất nhiều năng lượng để thoát ra.

**Niềm vui / Hạnh phúc — Dao động điều hòa trong giếng (harmonic well):**

```
  Công thức:  U(x) = −V₀ + ½kx²    (giếng parabol, V₀ >> kT)

  Ý nghĩa:
    Giếng sâu V₀ → hệ bị giữ chặt tại đáy (trạng thái vui).
    Dao động nhỏ quanh đáy = biến thiên cảm xúc bình thường.
    Rào thoát ΔU = V₀ rất lớn → khó rời khỏi trạng thái vui.
    Tần số dao động ω = √(k/m) = nhịp của niềm vui (đều, ổn định).
```
```
joy, happy, happiness, cheerful, delightful, wonderful
elated, euphoric, blissful, ecstatic, jubilant
laughing, smiling, grinning, beaming
→ "ah, vui sướng — cảm xúc dương mạnh"
```

**Yêu thương / Trìu mến — Năng lượng liên kết hấp dẫn (gravitational binding):**

```
  Công thức:  U(r) = −G·m₁·m₂ / r     (thế hấp dẫn Newton)

  Ý nghĩa:
    Hai khối lượng (hai thực thể) hút nhau theo nghịch đảo khoảng cách.
    r → nhỏ (gần nhau) → |U| → lớn → liên kết mạnh hơn.
    Năng lượng liên kết E_b = −U > 0: cần E_b để tách rời.
    Tình yêu = liên kết hấp dẫn: càng gần → càng khó tách.
    Quỹ đạo ổn định: hệ hai vật quay quanh nhau (orbit = mối quan hệ bền).
```
```
love, loving, beloved, adore, adorable, affection
caring, tender, gentle, warm, embrace, hug
cherish, devoted, fond, passion, romance
→ "ah, yêu thương — tình cảm dương mạnh"
```

**Thành công / Giỏi giang — Công thực hiện ngược gradient (work against field):**

```
  Công thức:  W = ∫₀ˢ F⃗ · ds⃗ = ΔU > 0    (công dương)

  Ý nghĩa:
    Thành công = công đã thực hiện chống lại lực cản.
    W > 0 → hệ đã leo lên từ thế thấp → thế cao (vượt rào).
    Càng nhiều công (effort) → ΔU càng lớn → thành tựu càng cao.
    Hiệu suất: η = W_useful / W_total (brilliant = η → 1).
    Victory = hệ đã vượt qua rào thế hoàn toàn.
```
```
accomplish, achieve, success, triumph, victory, win
brilliant, excellent, outstanding, remarkable, superb
master, champion, hero, genius, talented
→ "ah, thành tựu — đánh giá dương mạnh"
```

**Đẹp / Tốt — Tỷ lệ vàng & đối xứng (golden ratio & symmetry):**

```
  Công thức:  φ = (1 + √5) / 2 ≈ 1.618    (tỷ lệ vàng)

  Đối xứng → thế năng cực tiểu:
    U_sym < U_asym    (hệ đối xứng LUÔN có thế năng thấp hơn)

  Nguyên lý Noether:
    Mỗi đối xứng liên tục ↔ một đại lượng bảo toàn.
    Đối xứng tịnh tiến → bảo toàn động lượng.
    Đối xứng quay → bảo toàn moment động lượng.

  Ý nghĩa:
    "Đẹp" trong vật lý = đối xứng = thế năng cực tiểu = ổn định nhất.
    Hệ tự nhiên tiến hóa VỀ PHÍA đối xứng (nguyên lý cực tiểu tác dụng).
    φ xuất hiện khắp nơi: vỏ ốc, hoa hướng dương, khuôn mặt, thiên hà xoắn.
```
```
beautiful, gorgeous, stunning, magnificent, splendid
good, great, fantastic, awesome, amazing, perfect
wonderful, marvelous, fabulous, glorious
→ "ah, đẹp/tốt — tính chất dương mạnh"
```

**Tự do / An toàn — Giếng thế với rào bảo vệ cao (metastable well):**

```
  Công thức:  U(x) = −V₀·sech²(x/a) + V_barrier·e^(−(x−x_b)²/2σ²)

  Ý nghĩa:
    Giếng sâu V₀ = trạng thái an toàn (ổn định cục bộ).
    Rào bảo vệ V_barrier bao quanh = protection.
    Hệ ở đáy giếng, KHÔNG BỊ nhiễu loạn bên ngoài ảnh hưởng.
    Tự do = giếng RỘNG (a lớn): hệ có thể di chuyển tự do BÊN TRONG giếng.
    An toàn = rào CAO: nhiễu bên ngoài không xuyên qua được.
    free + safe = giếng rộng VÀ rào cao → lý tưởng.
```
```
free, freedom, liberate, independent, autonomous
safe, secure, protected, sheltered, peaceful
comfort, cozy, relaxing, soothing, calm
→ "ah, tự do/an toàn — trạng thái dương mạnh"
```

### TÍCH CỰC — Giếng thế nông (shallow well)

> **Mô hình:** Thế năng hơi âm — hệ bị hút nhẹ, dễ dao động, dễ rời.
> Giống liên kết Van der Waals: yếu nhưng có hướng tích cực rõ ràng.

```
  Công thức:  U(r) = −ε · [(σ/r)⁶]    (phần hút của Lennard-Jones)

  Ý nghĩa:
    ε nhỏ → liên kết yếu nhưng CÓ (hướng tốt, không phải trung tính).
    Lực hút F = −dU/dr > 0 nhưng nhỏ → xu hướng tiếp cận, không cuốn hút.
    Dễ phá vỡ (ΔU nhỏ) → "tích cực vừa" — không mãnh liệt.
```
```
accept, adequate, agree, balance, benefit, capable
comfortable, convenient, decent, efficient, fair
helpful, honest, improve, interesting, pleasant
positive, productive, reasonable, reliable, useful
→ "ah, tích cực vừa — không mạnh nhưng hướng tốt"
```

### TRUNG TÍNH — Mặt phẳng thế (flat potential)

> **Mô hình:** Thế năng bằng phẳng — không có gradient, không có lực, không có xu hướng.
> Giống hạt trên mặt bàn phẳng lý tưởng: đặt đâu nằm đấy.

```
  Công thức:  U(x) = const    →    F(x) = −dU/dx = 0

  Ý nghĩa:
    Không có lực → không hút, không đẩy.
    Hệ ở trạng thái cân bằng phiếm định (neutral equilibrium).
    Nhiễu nhỏ → hệ dịch nhưng KHÔNG quay lại cũng KHÔNG đi xa.
    Từ ngữ ở đây = thuần mô tả, không mang giá trị thiện/ác.
```
```
agent, answer, authority, cause, cell, circuit
contract, count, cycle, decade, evidence, government
groups, having, load, material, merely, mixed
normal, passage, phase, process, regular, standard
→ "ah, trung tính — không tốt không xấu, mô tả sự thật"
```

### TIÊU CỰC — Rào thế thấp (low potential barrier)

> **Mô hình:** Thế năng hơi dương — hệ bị đẩy nhẹ, có xu hướng tránh xa.
> Giống lực ma sát nhẹ: cản trở nhưng không chặn hoàn toàn.

```
  Công thức:  U(x) = U₀ · e^(−x²/2σ²)    (rào Gauss, U₀ nhỏ)

  Xác suất xuyên rào (tunneling):
    T ≈ e^(−2κd)    với κ = √(2m(U₀−E))/ℏ

  Ý nghĩa:
    U₀ nhỏ → rào thấp → hệ CÓ THỂ vượt qua (khó chịu nhưng không chết).
    Lực đẩy F = −dU/dx < 0 nhưng nhỏ → khó chịu, không kinh hoàng.
    Tunneling probability cao → vấn đề có thể giải quyết được.
```
```
adverse, afflict, ambiguous, annoying, argue, bitter
complain, confuse, criticize, deny, difficult, doubt
embarrass, fail, frustrate, guilty, harsh, ignore
impatient, irritate, jealous, lack, limit, mislead
→ "ah, tiêu cực vừa — không nặng nhưng hướng xấu"
```

### RẤT TIÊU CỰC — Rào thế cao / vực thẳm (high barrier / abyss)

> **Mô hình:** Thế năng rất dương — hệ bị đẩy mạnh, hoặc rơi vào vùng bất ổn không thể thoát.
> Giống hạt đối diện rào Coulomb hoặc rơi vào lỗ đen: lực đẩy/hút hủy diệt.

**Ghét / Giận — Lực đẩy Coulomb (like-charge repulsion):**

```
  Công thức:  U(r) = +k·q₁·q₂ / r    (q₁, q₂ cùng dấu → U > 0)

  Lực:  F = −dU/dr = +k·q₁·q₂ / r²  > 0  (đẩy nhau)

  Ý nghĩa:
    Hai điện tích cùng dấu → đẩy nhau → KHÔNG THỂ ở gần.
    r → nhỏ (bị ép gần) → U → +∞ → lực đẩy cực đại.
    Ghét = lực đẩy Coulomb: hai thực thể KHÔNG TƯƠNG THÍCH.
    Giận = năng lượng tích tụ khi hệ bị ép vào vùng đẩy (r nhỏ cưỡng bức).
    Càng ép gần → càng giận (U ∝ 1/r → phát tán bùng nổ).
```
```
hate, hatred, loathe, despise, detest, abhor
angry, furious, enraged, livid, outraged, hostile
cruel, vicious, malicious, ruthless, brutal, savage
→ "ah, ghét/giận — cảm xúc âm mạnh"
```

**Buồn / Đau — Sụp đổ hấp dẫn (gravitational collapse):**

```
  Công thức:  U(r) = −G·M·m / r    khi r → r_s (bán kính Schwarzschild)

  Tại r = r_s = 2GM/c²:
    Vận tốc thoát v_esc = c → KHÔNG GÌ thoát ra được.
    U → −∞    (giếng thế không đáy)

  Ý nghĩa:
    Buồn = rơi vào giếng thế KHÔNG CÓ ĐÁY.
    Khác với "vui" (giếng có đáy, dao động quanh đáy):
      buồn = rơi tự do, không có điểm cân bằng.
    Hopeless = đã qua event horizon — không thể quay lại.
    Grief = năng lượng liên kết bị mất khi vật thể bị hút đi.
```
```
grief, sorrow, misery, agony, anguish, torment
crying, weeping, mourning, heartbroken, devastated
depressed, hopeless, desperate, despairing, forlorn
→ "ah, buồn/đau — cảm xúc âm mạnh"
```

**Sợ / Kinh hoàng — Hiệu ứng đường hầm nghịch (inverse tunneling):**

```
  Công thức:  T = e^(−2κd)    với κ = √(2m·U₀)/ℏ,  U₀ >> E

  Khi U₀ >> E:
    κ → lớn  →  T → 0  (xác suất thoát gần bằng không)

  Ý nghĩa:
    Sợ = đối diện rào thế KHÔNG THỂ vượt qua.
    U₀ >> E → hệ BIẾT không đủ năng lượng để thoát.
    Xác suất thoát T → 0 = cảm giác bất lực (helplessness).
    Terror vs fear: κ·d lớn (terror) vs vừa (fear).
    Phobia = hệ gán U₀ → ∞ cho rào dù rào thực tế nhỏ.
```
```
terror, horror, nightmare, dread, panic, phobia
frightened, terrified, horrified, petrified, aghast
creepy, eerie, sinister, menacing, threatening
→ "ah, sợ hãi — cảm xúc âm mạnh"
```

**Xấu / Hại — Phân rã phóng xạ (radioactive decay):**

```
  Công thức:  N(t) = N₀ · e^(−λt)     (phân rã theo hàm mũ)
              dN/dt = −λN              (tốc độ phá hủy ∝ lượng còn lại)

  Năng lượng phát ra:  E = Δm · c²    (E = mc², chuyển khối lượng thành phá hủy)

  Ý nghĩa:
    Xấu/hại = quá trình phân rã — phá vỡ cấu trúc, giải phóng năng lượng hủy diệt.
    λ = tốc độ phá hủy (destroy: λ lớn, corrupt: λ nhỏ nhưng dai dẳng).
    Không thể đảo ngược: entropy tăng, cấu trúc mất vĩnh viễn.
    toxic = chất xúc tác tăng λ (tăng tốc phân rã hệ khác).
    E = Δm·c² = năng lượng khổng lồ từ phá hủy (destruction is energetic).
```
```
evil, wicked, corrupt, toxic, poisonous, deadly
destroy, ruin, devastate, annihilate, obliterate
abuse, torture, murder, kill, death, corpse
→ "ah, xấu/hại — tính chất âm mạnh"
```

**Bệnh / Khổ — Suy thoái entropy không đảo ngược (irreversible degradation):**

```
  Công thức:  dS_universe / dt > 0     (Định luật 2 nhiệt động lực học)
              ΔS_system = Q/T > 0      (entropy hệ tăng, không giảm)

  Hệ quả:
    Chất lượng năng lượng giảm: free energy F = U − TS giảm dần.
    dF/dt < 0 → hệ mất khả năng làm việc theo thời gian.

  Ý nghĩa:
    Bệnh = entropy tăng TRONG hệ sinh học → mất trật tự → mất chức năng.
    Khổ = trạng thái F → 0: năng lượng có nhưng không thể chuyển hóa.
    Disease: hệ miễn dịch không đảo ngược được entropy (dS/dt > khả năng sửa chữa).
    Suffering: hệ ý thức BIẾT entropy đang tăng nhưng KHÔNG THỂ ngăn.
    Dying: F → 0, S → S_max → hệ ngừng hoạt động.
```
```
sick, ill, disease, cancer, plague, infection
suffering, pain, wound, injury, bleeding, dying
starving, homeless, abandoned, neglected, orphan
→ "ah, bệnh/khổ — trạng thái âm mạnh"
```

---

## Tầng 2 cho CỤM TỪ: "Chủ đề gì?"

### Cảm xúc tích cực (191 cụm)

```
a good                → tốt chung
a pleasure            → niềm vui
achieve success       → thành tựu
best friend           → quan hệ tốt
best wishes           → chúc tốt
bright future         → tương lai tươi
good fortune          → may mắn
good health           → sức khỏe tốt
high quality          → chất lượng cao
→ "ah, cụm tích cực [chủ đề: vui/tốt/đẹp/thành công]"
```

### Cảm xúc tiêu cực (141 cụm)

```
a shame               → xấu hổ
bad luck              → xui xẻo
bad news              → tin xấu
back pain             → đau đớn
dead end              → bế tắc
drug addiction        → nghiện
hate crime            → tội ác
pain killer           → đau + giảm
worst case            → tình huống xấu nhất
→ "ah, cụm tiêu cực [chủ đề: xấu/đau/hại/thất bại]"
```

### Giá trị đạo đức (51 cụm)

```
academic freedom      → tự do học thuật
equal opportunity     → cơ hội bình đẳng
fair trade            → thương mại công bằng
golden rule           → quy tắc vàng
human rights          → nhân quyền
social justice        → công bằng xã hội
→ "ah, giá trị đạo đức — không cảm xúc thuần, mà là nguyên tắc"
```

### Quan hệ xã hội (118 cụm)

```
all together          → cùng nhau
best friend           → bạn thân
care about            → quan tâm
close relationship    → quan hệ gần
extended family       → gia đình mở rộng
team work             → làm việc nhóm
→ "ah, quan hệ xã hội — kết nối giữa người với người"
```

### Thành tựu (78 cụm)

```
big business          → kinh doanh lớn
build bridges         → xây cầu (nghĩa bóng)
field goal            → bàn thắng
grow from             → phát triển từ
hard work             → làm việc chăm chỉ
→ "ah, thành tựu — nỗ lực và kết quả"
```

### Sức khỏe (65 cụm)

```
balanced diet         → chế độ ăn cân bằng
clean energy          → năng lượng sạch
health insurance      → bảo hiểm sức khỏe
mental health         → sức khỏe tâm thần
→ "ah, sức khỏe — cơ thể và tinh thần"
```

---

## Từ khóa → Mức Valence (cheat sheet)

| Thấy từ/cụm này | → Mức V | → Nhìn biết |
|-----------------|---------|------------|
| joy, love, triumph, beautiful, perfect | Rất tích cực (+0.5~+1.0) | "vui/đẹp/tốt tuyệt vời" |
| good, helpful, pleasant, benefit, fair | Tích cực (+0.2~+0.5) | "OK, khá tốt" |
| agent, process, standard, normal, phase | Trung tính (-0.2~+0.2) | "khách quan, không thiên" |
| annoying, difficult, embarrass, lack | Tiêu cực (-0.5~-0.2) | "hơi xấu, không hay" |
| hate, horror, death, torture, destroy | Rất tiêu cực (-1.0~-0.5) | "kinh khủng, đau khổ" |

---

## Tóm tắt — Valence như cảnh quan thế năng

```
V = 1 trục liên tục [-1.0, +1.0]  (thế năng → lực hút/đẩy)

  -1.0        -0.5        -0.2    0    +0.2        +0.5        +1.0
   |──────────|──────────|────|────|──────────|──────────|
   repeller    low         flat     shallow     deep
   (rào cao)   barrier     (U=0)   well        well
   U >> 0     U > 0                U < 0       U << 0

7,985 từ + 644 cụm = mỗi cái 1 vị trí trên cảnh quan thế năng

Bản chất vật lý mỗi nhóm:
  ┌─────────────────┬──────────────────────────────────────────┐
  │ Nhóm            │ Công thức đặc trưng                      │
  ├─────────────────┼──────────────────────────────────────────┤
  │ Niềm vui        │ U = −V₀ + ½kx²  (giếng điều hòa)       │
  │ Yêu thương      │ U = −Gm₁m₂/r  (liên kết hấp dẫn)      │
  │ Thành công       │ W = ∫F⃗·ds⃗ > 0  (công ngược gradient)  │
  │ Đẹp/Tốt         │ φ = (1+√5)/2, đối xứng → U_min         │
  │ Tự do/An toàn   │ Giếng rộng + rào cao (metastable)       │
  │ Tích cực vừa     │ U = −ε(σ/r)⁶  (Van der Waals)          │
  │ Trung tính       │ U = const, F = 0  (phẳng)              │
  │ Tiêu cực vừa     │ U = U₀·e^(−x²/2σ²)  (rào Gauss thấp) │
  │ Ghét/Giận        │ U = +kq₁q₂/r  (Coulomb đẩy)           │
  │ Buồn/Đau         │ U → −∞ tại r_s  (sụp đổ hấp dẫn)     │
  │ Sợ hãi           │ T = e^(−2κd) → 0  (không thoát được)  │
  │ Xấu/Hại          │ N(t) = N₀·e^(−λt)  (phân rã phóng xạ)│
  │ Bệnh/Khổ         │ dS/dt > 0, F → 0  (suy thoái entropy) │
  └─────────────────┴──────────────────────────────────────────┘
```
