# OLANG — ONE FILE TO RUN EVERYTHING
## Đặc tả kỹ thuật · Append-only · 2026-03-07

```
○(x) == x          identity
○(∅) == ○          tự sinh
1 file == 1 world  mọi thứ trong một
```

---

## VẤN ĐỀ

Mọi hệ thống hiện tại lưu **data** — pixel, byte, string, record.

```
Font chữ "A" = Bezier curves → rasterize → pixel grid
              12 KB/glyph × 168K chars = 2 GB (Noto fonts)
              mờ khi zoom · không biết A liên quan đến α

Ngọn núi    = 10 triệu triangle vertices → 500 MB
              không có vật lý · không đi được · không có bóng

Chương trình = source + compiler + runtime + OS + dependencies
              hàng GB · không portable · vỡ khi update
```

Olang giải quyết bằng cách lưu **CÔNG THỨC** thay vì data:

```
Chữ "A"  = ∪(⌀(-.28,-.38,.00,.44), ⌀(.00,.44,.28,-.38), ⌀(-.13,.08,.13,.08))
           80 bytes · sắc nét mọi kích thước · biết A≡α≡А≡अ≡あ

Ngọn núi = ∫(fbm, oct:6, H:0.8, seed:42)
           48 bytes · 3D thực · có vật lý · có bóng · đi được

Chương trình = ISL address + DNA + silk edges
               → 1 file, 0 dependencies, runs everywhere
```

---

## KIẾN TRÚC — 1 FILE DUY NHẤT

```
homeos.olang  (16 MB compressed)
│
├── HEADER          64 B    magic + version + hash + sig
├── LAYER INDEX     9 KB    9 layers × 24B
│
├── L0  PRIMITIVES  4 KB    72 opcodes — bất biến tuyệt đối
├── L1  UTF32       4.4 MB  168,046 ký tự × (SDF + ISL + edges)
├── L2  NATURE      0.8 MB  nước, lửa, đất, gió, ánh sáng
├── L3  LIFE        1.8 MB  tế bào, cây, động vật, người
├── L4  OBJECTS     1.2 MB  đèn, cửa, xe, thiết bị
├── L5  PROGRAMMING 2.9 MB  Go/Python/WASM semantics + opcodes
├── L6  PERCEPTION  1.1 MB  vision, audio, touch, depth
├── L7  PROGRAMS    0.7 MB  agents, skills, worlds — executable
│
├── SILK EDGES      2.8 MB  ~500K edges × 17B (sorted)
└── ED25519 SIG     64 B
                    ──────
                    ~16 MB total
```

### Nguyên tắc append-only

```
KHÔNG BAO GIỜ:  DELETE · OVERWRITE · MODIFY
CHỈ ĐƯỢC:       APPEND node mới · APPEND edge mới · ARCHIVE (không xóa)

Phục hồi state tại t = replay từ đầu đến timestamp t
→ deterministic, verifiable, auditable
```

---

## L0 — PRIMITIVES (4 KB · bất biến tuyệt đối)

Đây là "bảng tuần hoàn" của Olang. Build mọi thứ khác từ đây.

```
SDF opcodes:
  0x01 ●  SPHERE    cx cy cz r
  0x02 ⌀  CAPSULE   ax ay az  bx by bz  r
  0x03 □  BOX       cx cy cz  hx hy hz
  0x04 ◯  TORUS     cx cy cz  R r
  0x05 ◌  VOID
  0x10 ∪  UNION     count k
  0x11 ∖  SUB       k
  0x12 ∩  INTERSECT k
  0x20 ∫  FBM       octaves H lacunarity gain seed
  0x21 ∇  GRADIENT  (ε=0.001 central diff)
  0x22 ☀  LIGHT     elevation azimuth intensity colorK
  0x30 ∿  SPLINE    n [t x y z]*n

Logic opcodes:
  0x40 ∧  AND    0x41 ∨ OR      0x42 ¬ NOT
  0x43 =  EQ     0x44 ≠ NEQ     0x45 < LT    0x46 > GT
  0x47 ⇒  IMPL   0x48 ⇔ IFF

ISL opcodes:
  0x50    GET     lookup ISL address → node
  0x51    WALK    follow silk edge
  0x52    EMIT    send ISL message
  0x53    BCAST   broadcast to group

Control opcodes:
  0x60    SEQ     a b c...
  0x61    IF      cond then else
  0x62    LOOP    N body
  0x63    DREAM   body  (run when inbox idle >5min)
  0x64    SPAWN   agent_def
```

---

## L1 — UTF32 (4.4 MB · 168,046 ký tự)

Unicode 18.0 đầy đủ. Mỗi ký tự = định nghĩa, không phải font.

```
Chữ A:
  codepoint   U+0041
  isl_addr    [L][U][a][1]  ← Latin.Upper.Alpha.1
  dna         ∪(⌀(-.28,-.38,.00,.44,T), ⌀(.00,.44,.28,-.38,T), ⌀(-.13,.08,.13,.08,T))
  phonemes    /eɪ/ /æ/ /ɑː/
  edges       A≡α  (SameSound)   A≡А  (SameSound)   A≡∧  (SameShape)
              A←𐤀  (DerivedFrom: Phoenician Aleph, đầu bò)

Chữ 山 (núi):
  codepoint   U+5C71
  isl_addr    [N][L][c][1]  ← Nature.Land.CJK.1
  dna         3 capsules hội tụ + đường ngang đáy
  concept     → ∫(fbm_terrain, 3_peaks)  ← ngọn núi THẬT trong world
  edges       山≡mountain≡berg≡جبل  (SameSound cross-script)

Seal script 山 (U+3D000+):
  Đặc biệt: Seal chars gần SDF nhất trong mọi Unicode
  Seal 山 = vẽ ba đỉnh núi → link trực tiếp tới terrain SDF
  Seal 水 = vẽ ba làn sóng → link trực tiếp tới fluid SDF
  Seal 火 = vẽ ngọn lửa   → link trực tiếp tới FBM flame SDF
  → Tầng lịch sử giữa ký tự và thực tế
```

**Dung lượng L1:** 168,046 × 115B avg = 18.4 MB raw → **4.4 MB** sau zstd (6:1)

---

## L2 — NATURE (0.8 MB)

Không mô tả tự nhiên. **Là** tự nhiên.

```
Nước:
  sdf     ∫(fbm_fluid, oct:4, viscosity:1e-3)
  physics density:1000 kg/m³  temp:293K  flow:Vec3
  sound   ∿(440Hz_sine × velocity)
  isl     [N][W][r][1]
  links   水≡water≡ماء≡पानी  (UTF32 bridge)

Lửa:
  sdf     ∪(●(core, r:0.1), ∫(fbm_turb, upward, oct:5))
  physics temp:600-2000K  emission:visible+IR
  sound   ∿(pink_noise, crackle:1-4kHz)
  isl     [N][F][r][1]

Đất:
  sdf     ∫(fbm_terrain, oct:6, H:0.8, seed:N)
  physics density:2500 kg/m³  solid:true
  isl     [N][E][r][1]

Ánh sáng:
  sdf     ☀(dir, intensity, colorK:5500)
  physics c:3e8 m/s  λ:380-780nm
  isl     [N][L][r][1]

Gió:
  sdf     ∫(curl_noise, velocity:Vec3, turbulence:0.3)
  isl     [N][A][r][1]
```

---

## L3 — LIFE (1.8 MB)

Sinh vật = SDF + Spline chuyển động + Behavior rules.

```
Người:
  sdf     ∪(⌀spine, ●head, ⌀arm_L, ⌀arm_R, ⌀leg_L, ⌀leg_R, k:0.1)
  splines walk_cycle  run_cycle  reach  sit  sleep
  isl     [H][P][r][1]
  skills  communicate  move  use_tools  learn  dream
  links   人(CJK) ≡ human ≡ إنسان (UTF32)

Cây:
  sdf     ∪(⌀trunk, branches:fractal(gen:6), ∪(●leaf×N, k:0.05))
  splines grow(T:years)  sway(wind)  wilt(drought)
  isl     [L][P][T][1]

Chim:
  sdf     ∪(●body, ●head, ⌀wing_L, ⌀wing_R, ⌀tail)
  splines flap(4Hz)  glide  perch  takeoff
  isl     [L][A][B][1]
```

---

## L4 — OBJECTS (1.2 MB)

Vật thể nhân tạo = SDF + ISL address + Actuator binding.

```
Đèn phòng khách:
  sdf     ∪(□body, ●bulb)
  state   {on, off, dim:0-100}
  isl     [H][A][a][1]  ← Home.Actuator.Light.1
  actuator skill/actuator/light.go → emit ISL

Cửa:
  sdf     □(w:0.9, h:2.1, d:0.05)
  state   angle:0°-90°
  spline  open(T:0.5s)  close(T:0.5s)
  isl     [H][D][r][1]

Điều hòa:
  sdf     □(w:0.8, h:0.3, d:0.2)
  state   {on, off, temp:16-30°C, mode:cool/heat/fan}
  isl     [H][A][h][1]
```

---

## L5 — PROGRAMMING (2.9 MB)

Không lưu cú pháp. Lưu **semantic**.

```
LOOP:
  isl   [P][C][L][1]
  dna   LOOP(condition, body)
  Go    for condition { body }
  Py    while condition: body
  C     while(condition) { body }
  WASM  block loop br_if end end
  x86   test jnz ...

FUNCTION:
  isl   [P][A][F][1]
  dna   FUNC(name, inputs:[], outputs:[], body)
  maps  func / def / (func ...) / ∫

IF:
  isl   [P][C][I][1]
  dna   IF(cond, then, else?)

ARRAY:
  isl   [P][D][A][1]
  dna   SEQ(type, len)

MAP:
  isl   [P][D][M][1]
  dna   MAP(key_type, val_type)

GOROUTINE / ASYNC:
  isl   [P][C][G][1]
  dna   SPAWN(func, args)  ← tương đương go func()

Go AST nodes được encode đầy đủ:
  FuncDecl BlockStmt ForStmt RangeStmt IfStmt SelectStmt
  TypeSpec InterfaceType StructType
  → mỗi node = ISL + DNA + children edges
  → compile Go source = traverse AST → lookup ISL → emit binary

x86-64 opcodes:
  MOV ADD SUB MUL DIV AND OR XOR NOT
  JMP JE JNE JL JG JLE JGE
  CALL RET PUSH POP
  mỗi opcode = ISL addr + encoding + operand format
```

---

## L6 — PERCEPTION (1.1 MB)

Cảm quan — không render pixel, render **ý nghĩa**.

```
Vision:
  isl     [P][V][c][1]
  dna     CAMERA(pos, target, fov)
  method  rayMarch(sdf_world) → per_ray: ISL_address
          "pixel" = "cái gì ở đây?" → ISL address của vật thể
          agent không thấy màu sắc — agent thấy ĐỊA CHỈ
          agent nhìn thấy [N][L][c][1] → biết đó là 山 → hành động

Audio:
  isl     [P][A][m][1]
  dna     MIC(sample_rate:44100, channels:1)
  method  FFT → frequency_bands → nearest IPA phoneme
          "nghe /a/" → [I][V][a][1] → biết âm /a/ → link sang A α А

Touch:
  isl     [P][T][s][1]
  method  pressure × texture → ISL(Nature.Surface.*)

Proprioception (tự nhận biết vị trí thân thể):
  isl     [P][P][r][1]
  method  joint_angles → SDF_body_pose
```

---

## L7 — PROGRAMS (0.7 MB · ghi khi verified)

Chương trình thực sự — chỉ append vào L7 khi đã chứng minh.

```
World: HomeOS
  isl   [W][H][1][1]
  dna   🌍(seed:888, terrain:∫fbm, agents:[aam,leoai,lights,...])
  status  QR

Agent: AAM
  isl   [A][A][M][1]
  skills  [Decide4D, Broadcast, SecurityGate]
  status  QR

Agent: LeoAI
  isl   [A][L][E][1]
  skills  [ShortTerm(512), LongTerm, Cluster, Dream, Sign]
  status  QR

Config: HomeOS default
  isl   [C][H][d][1]
  dna   CONFIG(agents:[], sensors:[], topology:hierarchy)
  status  QR
```

---

## QUY TRÌNH BUILD — khi nào ghi vào lõi?

```
                TODAY
                  │
         ┌────────▼────────┐
         │   L8+ DRAFT     │  ← làm việc ở đây
         │   (ĐN zone)     │
         │   có thể sai    │
         │   có thể thay   │
         └────────┬────────┘
                  │
           test + verify
           ED25519 sign
                  │
         ┌────────▼────────┐
         │   APPEND → QR   │  ← ghi vào layer đúng
         │   bất biến      │
         │   append-only   │
         └────────┬────────┘
                  │
              rehash file
              re-sign

Khi nào MILESTONE "build chương trình thật sự"?

M1: L1 complete
    ✓ 168K chars có valid SDF
    ✓ Render đúng 8pt → 1000pt
    ✓ Silk edges verified
    → COMMIT L1 into olang core

M2: L2+L3 complete
    ✓ Nước, lửa, đất animate được
    ✓ Người đi được theo Spline
    → COMMIT L2+L3

M3: L4+L7 complete
    ✓ "tắt đèn phòng khách" end-to-end
    ✓ AAM → Chief → LightAgent pass SecurityGate
    → COMMIT L4+L7 = olang run homeos.olang WORKS

M4: L5+L6 complete
    ✓ Go AST → ISL → compile works
    ✓ Vision: agent nhìn thấy ISL address
    → COMMIT L5+L6 = full stack

FINAL:
  1 file homeos.olang (~16 MB)
  $ olang run homeos.olang
  → Khởi động toàn bộ HomeOS
  → Không cần Go, Python, Node, JVM, OS libraries
  → Portable: copy sang bất kỳ máy nào có olang runtime
```

---

## DUNG LƯỢNG CUỐI CÙNG

```
                RAW         zstd+AES
L0 Primitives   4 KB        4 KB
L1 UTF32        18.4 MB     4.4 MB
L2 Nature       3.2 MB      0.8 MB
L3 Life         7.1 MB      1.8 MB
L4 Objects      4.8 MB      1.2 MB
L5 Programming  11.6 MB     2.9 MB
L6 Perception   4.4 MB      1.1 MB
L7 Programs     2.8 MB      0.7 MB
Silk edges      28.5 MB     2.8 MB
Headers         6 KB        6 KB
─────────────────────────────────
TOTAL           80.8 MB    ~16 MB
```

### Một file 16 MB chứa:

```
  ✅ Toàn bộ chữ viết nhân loại  (168,046 ký tự, 18.0 compatible)
  ✅ Vật lý tự nhiên              (nước, lửa, đất, gió, sáng)
  ✅ Sinh học cơ bản              (tế bào → người)
  ✅ Vật thể gia dụng             (đèn, cửa, xe, thiết bị)
  ✅ Semantics lập trình          (Go, Python, WASM, x86)
  ✅ Cảm quan đầy đủ              (vision, audio, touch)
  ✅ Chương trình HomeOS          (agents, skills, world)
  ✅ 500,000 silk edges           (quan hệ ngữ nghĩa)
  ✅ ED25519 signatures           (append-only, verifiable)
  ✅ AES-256-GCM encrypted        (secure at rest)

  Không cần:
  ✗ OS dependencies
  ✗ Runtime installation
  ✗ Network at startup
  ✗ Separate config files
  ✗ Database
  ✗ Font files
```

---

*Append-only. Không được xóa. Chỉ được thêm vào.*
*2026-03-07*
