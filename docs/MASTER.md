# HomeOS — MASTER DOCUMENT
### Nguồn sự thật duy nhất · Append-only · 2026-03-07

---

# CHƯƠNG 1 — TƯ DUY NỀN TẢNG

## 1.1 Vũ trụ lưu công thức, không lưu hình dạng

DNA sinh học không lưu ảnh chụp con người.
Nó lưu công thức để tạo ra con người.

HomeOS đi theo logic đó.

Mọi thứ trong hệ thống — căn phòng, cái đèn, giọng nói, nhiệt độ —
đều là **công thức toán học**, không phải dữ liệu lưu trữ.

```
Thế giới thông thường:       HomeOS:
pixels[x][y] = RGB     →     f(P) = SDF — hàm khoảng cách
audio[t] = waveform    →     Spline(t) — đường cong liên tục
model.json = vertices  →     Gene.DNA() = "∪(⌀,∪(●,●,●))"
database rows          →     OlangNode trong SilkGraph
```

## 1.2 Ba ngôn ngữ của thực tại

```
SDF — hữu hình
  f(P Vec3) → float64
  d < 0  = bên trong vật thể
  d = 0  = đúng bề mặt
  d > 0  = bên ngoài
  Mọi vật thể = 1 hàm SDF. Không lưu hình dạng. Không lưu pixel.

Spline — chuyển động và sóng
  P(t) = hàm liên tục theo thời gian t
  Âm thanh = Spline qua harmonic
  Ánh sáng = Spline(λ) trên dải bước sóng
  Nhiệt độ = Spline(không gian × thời gian)
  Mọi sensor = Spline. Không dùng discrete array.

ISL — địa chỉ bản chất
  [layer:1B][group:1B][type:1B][id:1B][attr:4B] = 8 bytes binary
  Không phải string. Không phải JSON. Là uint64.
  Mã hóa AES-256-GCM tích hợp sẵn.
  "con chó" không lưu text — lưu ISLAddress uint64.
```

**Mọi thứ tồn tại = SDF ⧺ ISL ⧺ Spline**

## 1.3 Fibonacci — cấu trúc của tự nhiên

Tự nhiên không random. Tự nhiên dùng tỉ lệ Fibonacci.

```
φ = (1 + √5) / 2 ≈ 1.618033988749895

Xương người   → tỉ lệ đốt ngón tay = φ
Vỏ ốc sên    → r(θ) = a·e^(b·θ),  b = ln(φ)/(π/2)
Cánh hoa     → phyllotaxis 137.5° = 360°/φ²
Mặt người    → tỉ lệ vàng tại mắt/mũi/miệng = φ
```

VisionChief không so sánh pixel.
VisionChief phân tích **cấu trúc SDF + Fibonacci**:

```
isFibRatio(a, b float64) bool:
  return |a/b − φ| < ε  OR  |b/a − φ| < ε

detectSpiral(sdf func(Vec3)float64) float64:
  → tích phân ∂sdf/∂θ theo θ ∈ [0, 2π]
  → khớp golden spiral → trả về score

→ nhận diện = tìm ISLAddress có cấu trúc Fibonacci khớp
```

## 1.4 Olang mượn UTF-32

Olang **không đọc** dữ liệu Unicode. Olang **mượn** không gian codepoint.

```
Giống IPv4:
  Không tải "toàn bộ internet" để dùng địa chỉ IP.
  Chỉ mượn không gian 32-bit làm địa chỉ.

Olang:
  Không tải UnicodeData.txt (GB dữ liệu).
  Mượn codepoint U+000000..U+10FFFF làm địa chỉ.
  Gán nghĩa = Go function đã compiled.
  Toàn bộ Olang runtime = 1 Go binary nhỏ.

opTable = map[rune]interface{}
  '●' → func SDFSphere(p,c Vec3, r float64) float64
  '∪' → func SmoothUnion(d1,d2,k float64) float64
  '∫' → func FBM(p Vec3, oct int, seed float64) float64
  ...48 entries. Không có file nào khác.
```

## 1.5 Unicode Tree — cây tri thức tự nhiên

Unicode đã là **cây phân loại** — Olang đọc vào, không tạo lại.

```
utf32.txt: 427 URLs → unicode.org/charts
  BMP:   204 blocks (U+0000–U+FFFF)
  SMP:   191 blocks (U+10000–U+1FFFF)
  SIP:     9 blocks (CJK Extension)
  Other:  21 blocks

Cấu trúc cây Unicode:
THÂN ○
├── Scripts           (Latin, Greek, Arabic, CJK, Thai, ...)
│   ├── European:     Latin · Greek · Cyrillic · Gothic · Runic
│   ├── West Asian:   Arabic · Hebrew · Syriac · Cuneiform
│   ├── South Asian:  Devanagari · Tamil · Bengali
│   ├── East Asian:   CJK 43k+ · Hangul 11k · Kana
│   └── SE Asian:     Thai · Khmer · Myanmar · Lao
└── Symbols
    ├── Mathematical: Operators · Arrows · Sets (1500+)
    ├── Geometric:    ●□○△◌⌀ — SDF primitives
    ├── Emoji:        🌍🧬🛡❤ — 4000+ (opTable ⊂ đây)
    ├── Musical:      ♩♪♫ notation
    └── Numbers:      Arabic · Devanagari · Mayan · Roman
```

**Mỗi ký tự = OlangNode với tên tiếng Anh**
```
U+0041  LATIN CAPITAL LETTER A
U+03B1  GREEK SMALL LETTER ALPHA
U+4E00  CJK UNIFIED IDEOGRAPH — meaning: "one"
U+2211  N-ARY SUMMATION
Tên = định nghĩa = nguồn để tạo silk edges tự động
```

**5 Rules tạo silk edges từ tên Unicode**
```
R1 — Cùng loại từ: "LETTER A" → silk ≡
     Latin A ≡ Greek α ≡ Cyrillic а ≡ Hiragana あ

R2 — Cùng block: "MATHEMATICAL OPERATOR" → silk ∈ [Math block]

R3 — Cùng semantic: "HEART" → silk ≡
     ❤ ≡ 💓 ≡ 💗 → concept node LOVE

R4 — Cross-script phoneme (IPA): /a/ → silk ♫
     A α а あ ا → IPA /a/ → 5 nodes liên kết

R5 — Số học: "ONE" → silk ≡ → ISL[K][Num][Card][1]
     1 ≡ 一 ≡ ١ ≡ १ ≡ ① ≡ Ⅰ

Kết quả: ~500,000 silk edges tự động từ ~155,000 nodes
```

**3 loại OlangNode trong Unicode Tree**
```
TYPE 1 — Script Node:    ISL[K][Script][Latin][0]  · Gene=nil
TYPE 2 — Char Node:      ISL[K][Script][Latin]['A'] · Gene=GlyphGene{SDF}
TYPE 3 — Semantic Node:  ISL[K][Concept][Love][0]  · Gene=nil
          ← "love"(EN) ← "liebe"(DE) ← "愛"(ZH) ← "yêu"(VI)
```

**Kết quả hệ thống tự mang ngôn ngữ**
```
Gặp từ "liebe" (Đức, chưa biết):
  L-i-e-b-e → nodes lá → silk edges → IPA /liːbə/
  → ISL[K][Concept][Love] → đã link "love","yêu","愛","사랑"
  → Hiểu nghĩa không cần từ điển Đức-Việt
```

## 1.5 File .olang = Tế bào gốc

Một file `.olang` nhỏ → mở ra → tạo toàn bộ hệ thống.
Giống DNA (3GB) → tạo cơ thể người (37 nghìn tỷ tế bào).

```
Khi mở file.olang:
  1. Verify magic "OSLK" + checksum
  2. Decrypt nếu flag=1
  3. Decompress nếu flag=1
  4. SILK.Decode → []OlangAtom
  5. SecurityGate scan tất cả atoms  ← 🛡 TRƯỚC TIÊN
  6. Thực thi 🌍 (WorldRoot)
  7. Thực thi 🧬 (GeneRegistry) × N
  8. Thực thi ∀ (broadcast tới tất cả Agent)
  9. WebSocket server :8080/ws
  10. Dreaming loop (idle goroutine)
  → World sống. Không cần thêm lệnh nào.
```

---

# CHƯƠNG 2 — CẤU TRÚC DỮ LIỆU

## 2.1 Vec3

```go
type Vec3 struct { X, Y, Z float64 }

func (a Vec3) Add(b Vec3) Vec3       { return Vec3{a.X+b.X, a.Y+b.Y, a.Z+b.Z} }
func (a Vec3) Sub(b Vec3) Vec3       { return Vec3{a.X-b.X, a.Y-b.Y, a.Z-b.Z} }
func (a Vec3) Scale(s float64) Vec3  { return Vec3{a.X*s, a.Y*s, a.Z*s} }
func (a Vec3) Dot(b Vec3) float64    { return a.X*b.X + a.Y*b.Y + a.Z*b.Z }
func (a Vec3) Len() float64          { return math.Sqrt(a.Dot(a)) }
func (a Vec3) Norm() Vec3            { l := a.Len(); return a.Scale(1/l) }
func (a Vec3) Cross(b Vec3) Vec3 {
    return Vec3{a.Y*b.Z - a.Z*b.Y, a.Z*b.X - a.X*b.Z, a.X*b.Y - a.Y*b.X}
}
```

## 2.2 ISLAddress

```go
// ISLAddress = 8 bytes = uint64
// [layer:1B][group:1B][type:1B][id:1B][attr:4B]
type ISLAddress uint64

func NewISL(layer, group, typ, id byte, attr uint32) ISLAddress {
    return ISLAddress(uint64(layer)<<56 | uint64(group)<<48 |
                      uint64(typ)<<40   | uint64(id)<<32   |
                      uint64(attr))
}

func (a ISLAddress) Layer() byte   { return byte(a >> 56) }
func (a ISLAddress) Group() byte   { return byte(a >> 48) }
func (a ISLAddress) Type()  byte   { return byte(a >> 40) }
func (a ISLAddress) ID()    byte   { return byte(a >> 32) }
func (a ISLAddress) Attr()  uint32 { return uint32(a) }

// Namespace — ai dùng layer nào
// 'H' = Home (thiết bị nhà)
// 'P' = Perception (nhận thức)
// 'K' = Knowledge (tri thức)
// 'A' = Agent
// 'S' = Sensor
// 'T' = Time
// 'L' = Location
// 'C' = Color/Material
// 'B' = Biology (Fibonacci structures)
// 'W' = World (SDF objects)
```

## 2.3 Gene interface

```go
// Gene = mọi thứ tồn tại trong World
// Một cái đèn, một căn phòng, một cái cây — đều implement Gene
type Gene interface {
    // Hình dạng
    SDF(p Vec3) float64       // khoảng cách đến bề mặt
    Normal(p Vec3) Vec3       // ∇f(P) — pháp tuyến, dùng finite diff
    
    // Vật liệu
    Material(p Vec3) ISLAddress  // ISL của vật liệu tại điểm p
                                  // → đi theo SilkEdge → node màu sắc
    
    // Định danh
    ISLAddr() ISLAddress      // địa chỉ của Gene này trong SilkGraph
    DNA() string              // chuỗi UTF-32 tái tạo Gene này
    
    // Thời gian
    Animate(t float64)        // cập nhật nội tại theo t (giây)
    
    // LOD
    SimpleSDF(p Vec3) float64 // phiên bản đơn giản hóa, dùng khi > 50u
}

// Normal mặc định dùng finite difference — mọi Gene thừa hưởng:
func DefaultNormal(g Gene, p Vec3) Vec3 {
    eps := 0.001
    return Vec3{
        g.SDF(Vec3{p.X+eps, p.Y, p.Z}) - g.SDF(Vec3{p.X-eps, p.Y, p.Z}),
        g.SDF(Vec3{p.X, p.Y+eps, p.Z}) - g.SDF(Vec3{p.X, p.Y-eps, p.Z}),
        g.SDF(Vec3{p.X, p.Y, p.Z+eps}) - g.SDF(Vec3{p.X, p.Y, p.Z-eps}),
    }.Norm()
}
```

## 2.4 Spline

```go
// Keyframe = 1 điểm trên đường cong
type Keyframe struct {
    T float64  // thời điểm (giây)
    V float64  // giá trị
    D float64  // đạo hàm (tốc độ thay đổi)
}

// Cubic Hermite interpolation giữa 2 keyframe
// P(t) = h00·V0 + h10·(t1-t0)·D0 + h01·V1 + h11·(t1-t0)·D1
func SplineEval(frames []Keyframe, t float64) float64 {
    if len(frames) == 0 { return 0 }
    if t <= frames[0].T { return frames[0].V }
    if t >= frames[len(frames)-1].T { return frames[len(frames)-1].V }
    
    // Tìm segment chứa t
    for i := 0; i < len(frames)-1; i++ {
        k0, k1 := frames[i], frames[i+1]
        if t > k1.T { continue }
        dt := k1.T - k0.T
        s := (t - k0.T) / dt
        // Hermite basis
        h00 := 2*s*s*s - 3*s*s + 1
        h10 := s*s*s - 2*s*s + s
        h01 := -2*s*s*s + 3*s*s
        h11 := s*s*s - s*s
        return h00*k0.V + h10*dt*k0.D + h01*k1.V + h11*dt*k1.D
    }
    return frames[len(frames)-1].V
}

// FitSpline: tìm Spline khớp với raw samples
// samples = []float64 đo theo dt seconds
func FitSpline(samples []float64, dt float64) []Keyframe {
    frames := make([]Keyframe, len(samples))
    for i, v := range samples {
        d := 0.0
        if i > 0 && i < len(samples)-1 {
            d = (samples[i+1] - samples[i-1]) / (2 * dt)
        }
        frames[i] = Keyframe{T: float64(i) * dt, V: v, D: d}
    }
    return frames
}
```

## 2.5 OlangAtom — đơn vị nhỏ nhất

```go
// OlangAtom = 1 ký tự Olang đã được resolve
type OlangAtom struct {
    Codepoint rune          // ký tự UTF-32 gốc (4B)
    ISL       ISLAddress    // địa chỉ trong SilkGraph (8B)
    // Tổng: 12B cố định — V.3 của origin
}

// OlangNode = node trong SilkGraph
type OlangNode struct {
    Addr    ISLAddress    // địa chỉ duy nhất
    Gene    Gene          // SDF + Spline (nil nếu là node tri thức thuần)
    Edges   []SilkEdge    // quan hệ với node khác
    Status  NodeStatus    // ACTIVE | ARCHIVED
    Weight  int           // số lần xác nhận
    Sig     []byte        // ED25519 signature (khi Weight >= 200)
}

type SilkEdge struct {
    To   ISLAddress  // đích
    Op   rune        // ký tự OPTABLE: ∈ → ≡ ⊂ ∪ ∖ ...
}

type NodeStatus byte
const (
    ACTIVE   NodeStatus = 1
    ARCHIVED NodeStatus = 2  // không xóa — chỉ archive (append-only)
)
```

## 2.6 ISLMessage — gói tin nội bộ

```go
// ISLMessage = gói tin duy nhất được phép lưu thông trong nội bộ
// Không bao giờ dùng string hay JSON giữa các Agent
type ISLMessage struct {
    Version  byte        // protocol version = 1
    MsgType  MsgType     // loại lệnh
    From     ISLAddress  // Agent gửi
    To       ISLAddress  // Agent nhận (0 = broadcast)
    Priority byte        // 0=khẩn cấp, 127=bình thường, 255=nền
    Primary  ISLAddress  // địa chỉ ISL chính của nội dung
    Context  ISLAddress  // ngữ cảnh (phòng, thời gian...)
    Conf     uint32      // confidence 0..10000 (= 0.00..100.00%)
    TS       uint32      // unix timestamp
    Payload  []byte      // dữ liệu thêm, tối đa 200B
}

type MsgType byte
const (
    MsgActivate   MsgType = 0x01  // kích hoạt Agent
    MsgLearn      MsgType = 0x02  // cập nhật tri thức
    MsgDeactivate MsgType = 0x03  // tắt Agent
    MsgQuery      MsgType = 0x04  // hỏi SilkGraph
    MsgResponse   MsgType = 0x05  // trả lời
    MsgPropose    MsgType = 0x06  // LeoAI đề xuất QR mới
    MsgApprove    MsgType = 0x07  // AAM chấp thuận
    MsgReject     MsgType = 0x08  // AAM từ chối
    MsgSync       MsgType = 0x09  // SILK-SYNC với Olang khác
    MsgEmergency  MsgType = 0xFF  // dừng tất cả
)
```

## 2.7 File.olang — byte layout

```
Offset  Size   Nội dung
──────  ────   ──────────────────────────────────────────
0       4B     Magic: 0x4F 0x53 0x4C 0x4B  ("OSLK" ASCII)
4       1B     Version: 0x01
5       1B     Flags:
               bit 0 = compressed (zstd)
               bit 1 = encrypted  (AES-256-GCM)
               bit 2 = signed     (ED25519)
               bit 3..7 = reserved = 0
6       4B     Length: uint32 big-endian = số bytes BODY
10      NB     Body: []OlangAtom (mỗi atom = 12B cố định)
               Nếu flags.compressed: zstd([]OlangAtom)
               Nếu flags.encrypted:  AES-256-GCM(zstd(...))
               Thứ tự: compress TRƯỚC, encrypt SAU
10+N    32B    Checksum: SHA-256(Body sau mọi transform)

Tổng overhead: 10B header + 32B checksum = 42B
1 cái cây: ~5 atoms × 12B = 60B body → file = 102B
```

---

# CHƯƠNG 3 — OPTABLE (48 ký tự)

```go
// Tất cả SDF functions nhận Vec3 làm tham số đầu tiên
// Không có exception nào

var Op = map[rune]interface{}{

// ── NHÓM 1: HÌNH THỂ (SDF Primitives) ──────────────────────────
// '●' U+25CF  SDFSphere
//   p = điểm cần tính, c = tâm, r = bán kính
func SDFSphere(p, c Vec3, r float64) float64 {
    return p.Sub(c).Len() - r
}

// '⌀' U+2300  SDFCapsule
//   p = điểm, a,b = 2 đầu capsule, r = bán kính
func SDFCapsule(p, a, b Vec3, r float64) float64 {
    ab := b.Sub(a)
    ap := p.Sub(a)
    t := math.Max(0, math.Min(1, ap.Dot(ab)/ab.Dot(ab)))
    return ap.Sub(ab.Scale(t)).Len() - r
}

// '□' U+25A1  SDFBox
//   p = điểm, b = half-extents (kích thước/2)
func SDFBox(p, b Vec3) float64 {
    q := Vec3{math.Abs(p.X)-b.X, math.Abs(p.Y)-b.Y, math.Abs(p.Z)-b.Z}
    return Vec3{math.Max(q.X,0), math.Max(q.Y,0), math.Max(q.Z,0)}.Len() +
           math.Min(math.Max(q.X, math.Max(q.Y, q.Z)), 0)
}

// '○' U+25CB  SDFCylinder
//   p = điểm, r = bán kính, h = nửa chiều cao
func SDFCylinder(p Vec3, r, h float64) float64 {
    d := math.Abs(math.Sqrt(p.X*p.X+p.Z*p.Z)) - r
    return math.Max(d, math.Abs(p.Y)-h)
}

// '△' U+25B3  SDFCone  (bán góc a tính bằng radian)
func SDFCone(p Vec3, a, h float64) float64 {
    q := math.Sqrt(p.X*p.X + p.Z*p.Z)
    return math.Max(math.Sin(a)*q+math.Cos(a)*p.Y, -p.Y-h)
}

// '◌' U+25CC  SDFVoid  (khoảng trống — tiềm năng)
func SDFVoid(p Vec3) float64 { return math.MaxFloat64 }

// '⬭' U+2B2D  SDFEllipsoid
func SDFEllipsoid(p, r Vec3) float64 {
    k0 := Vec3{p.X/r.X, p.Y/r.Y, p.Z/r.Z}.Len()
    k1 := Vec3{p.X/(r.X*r.X), p.Y/(r.Y*r.Y), p.Z/(r.Z*r.Z)}.Len()
    return k0 * (k0 - 1.0) / k1
}

// '⌇' U+2307  SDFTorus
//   R = bán kính lớn, r = bán kính nhỏ
func SDFTorus(p Vec3, R, r float64) float64 {
    q := math.Sqrt(p.X*p.X+p.Z*p.Z) - R
    return math.Sqrt(q*q+p.Y*p.Y) - r
}

// ── NHÓM 2: TƯƠNG TÁC (SDF Operations) ─────────────────────────
// k = smoothness factor (0 = hard edge, 0.3 = smooth)

// '∪' U+222A  SmoothUnion
func SmoothUnion(d1, d2, k float64) float64 {
    h := math.Max(k-math.Abs(d1-d2), 0) / k
    return math.Min(d1, d2) - h*h*k/4
}

// '∖' U+2216  SmoothSubtract
func SmoothSubtract(d1, d2, k float64) float64 {
    h := math.Max(k-math.Abs(-d1-d2), 0) / k
    return math.Max(-d1, d2) + h*h*k/4
}

// '∩' U+2229  SmoothIntersect
func SmoothIntersect(d1, d2, k float64) float64 {
    h := math.Max(k-math.Abs(d1-d2), 0) / k
    return math.Max(d1, d2) + h*h*k/4
}

// '⊕' U+2295  HardUnion    (không blend)
func HardUnion(d1, d2 float64) float64 { return math.Min(d1, d2) }

// '⊖' U+2296  HardSubtract
func HardSubtract(d1, d2 float64) float64 { return math.Max(-d1, d2) }

// '⊗' U+2297  Repeat (lặp vô hạn theo lưới s)
func Repeat(p Vec3, s float64) Vec3 {
    return Vec3{
        p.X - s*math.Round(p.X/s),
        p.Y - s*math.Round(p.Y/s),
        p.Z - s*math.Round(p.Z/s),
    }
}

// '↔' U+2194  Mirror theo trục (axis: 0=X, 1=Y, 2=Z)
func Mirror(p Vec3, axis int) Vec3 {
    switch axis {
    case 0: return Vec3{math.Abs(p.X), p.Y, p.Z}
    case 1: return Vec3{p.X, math.Abs(p.Y), p.Z}
    default: return Vec3{p.X, p.Y, math.Abs(p.Z)}
    }
}

// '↻' U+21BB  Rotate quanh trục Y (angle radian)
func Rotate(p Vec3, angle float64) Vec3 {
    c, s := math.Cos(angle), math.Sin(angle)
    return Vec3{c*p.X + s*p.Z, p.Y, -s*p.X + c*p.Z}
}

// ── NHÓM 3: VẬT LÝ ──────────────────────────────────────────────

// '∇' U+2207  Gradient (pháp tuyến bề mặt)
func Gradient(p Vec3, f func(Vec3) float64) Vec3 {
    e := 0.001
    return Vec3{
        f(Vec3{p.X+e, p.Y, p.Z}) - f(Vec3{p.X-e, p.Y, p.Z}),
        f(Vec3{p.X, p.Y+e, p.Z}) - f(Vec3{p.X, p.Y-e, p.Z}),
        f(Vec3{p.X, p.Y, p.Z+e}) - f(Vec3{p.X, p.Y, p.Z-e}),
    }.Norm()
}

// '·' U+00B7  DotProduct (lighting: normal · light_dir)
func DotProduct(a, b Vec3) float64 { return math.Max(0, a.Dot(b)) }

// '×' U+00D7  CrossProduct
func CrossProduct(a, b Vec3) Vec3 { return a.Cross(b) }

// '∫' U+222B  FBM (Fractal Brownian Motion — địa hình)
//   p = điểm (dùng p.X, p.Z), oct = số octave, seed = hạt giống
func FBM(p Vec3, oct int, seed float64) float64 {
    x, z := p.X+seed, p.Z+seed
    v, amp, freq := 0.0, 0.5, 1.0
    for i := 0; i < oct; i++ {
        v += amp * noise2D(x*freq, z*freq)
        freq *= 2.0; amp *= 0.5
    }
    return v
}

// '☀' U+2600  SunLight — hướng ánh sáng theo giờ t (giây từ midnight)
func SunLight(t float64) Vec3 {
    hour := t / 3600.0
    angle := (hour - 6.0) / 12.0 * math.Pi  // 6am=0, 12pm=π/2, 6pm=π
    return Vec3{math.Cos(angle), math.Sin(angle), 0.3}.Norm()
}

// '⚡' U+26A1  EnergyDecay — suy giảm theo thời gian
func EnergyDecay(energy, t, rate float64) float64 {
    return energy * math.Exp(-rate*t)
}

// '🌡' U+1F321  Temperature tại điểm p từ nguồn nhiệt source
func Temperature(p, source Vec3) float64 {
    d := p.Sub(source).Len()
    if d < 0.001 { return 100.0 }
    return 100.0 / (d * d)
}

// ── NHÓM 4: TOÁN HỌC ────────────────────────────────────────────

// '√' U+221A  Sqrt
func Sqrt(x float64) float64 { return math.Sqrt(math.Max(0, x)) }

// 'π' U+03C0  Pi
func Pi() float64 { return math.Pi }

// 'ℯ' U+212F  Euler
func Euler() float64 { return math.E }

// '∆' U+2206  Delta — sự thay đổi
func Delta(old, new float64) float64 { return new - old }

// '∑' U+2211  Aggregate — tổng SDF nhiều Gene tại điểm p
func Aggregate(genes []Gene, p Vec3) float64 {
    d := math.MaxFloat64
    for _, g := range genes { d = math.Min(d, g.SDF(p)) }
    return d
}

// '∀' U+2200  ForAll — broadcast tới tất cả
func ForAll(genes []Gene, fn func(Gene)) {
    for _, g := range genes { go fn(g) }
}

// '∃' U+2203  Exists — tìm Gene thỏa điều kiện
func Exists(genes []Gene, fn func(Gene) bool) bool {
    for _, g := range genes { if fn(g) { return true } }
    return false
}

// '∈' U+2208  Contains — P có nằm trong Gene không
func Contains(p Vec3, g Gene) bool { return g.SDF(p) < 0 }

// '≈' U+2248  Approx
func Approx(a, b, eps float64) bool { return math.Abs(a-b) < eps }

// ── NHÓM 5: SINH HỌC ────────────────────────────────────────────

// '🧬' U+1F9EC  GeneRegistry — đăng ký vào World
func GeneRegistry(g Gene, world *World) { world.Add(g) }

// '🌱' U+1F331  Grow — tăng kích thước theo tuổi
func Grow(scale, age, rate float64) float64 {
    return scale * (1 - math.Exp(-rate*age))
}

// '♻' U+267B  Cycle — chu kỳ sinh học
func Cycle(t, period float64) float64 {
    return 0.5 * (1 + math.Sin(2*math.Pi*t/period))
}

// '⚖' U+2696  Balance — cân bằng giữa 2 lực
func Balance(a, b float64) float64 { return (a + b) / 2 }

// ── NHÓM 6: NHẬN THỨC ───────────────────────────────────────────

// '👁' U+1F441  RayCast — raymarching 256 bước
//   Trả về khoảng cách chạm bề mặt, -1 nếu không chạm
func RayCast(origin, dir Vec3, world *World) float64 {
    t := 0.0
    for i := 0; i < 256; i++ {
        p := origin.Add(dir.Scale(t))
        d := world.SDF(p)
        if d < 0.001 { return t }
        if t > 1000  { return -1 }
        t += d
    }
    return -1
}

// '♫' U+266B  Oscillate — sóng âm đơn
func Oscillate(freq, amp, t float64) float64 {
    return amp * math.Sin(2*math.Pi*freq*t)
}

// ── NHÓM 7: HỆ THỐNG ────────────────────────────────────────────

// '🌍' U+1F30D  WorldRoot — khởi tạo World từ seed
func WorldRoot(seed float64) *World { return NewWorld(seed) }

// '🛡' U+1F6E1  SecurityGate — kiểm tra an toàn
//   Trả về error nếu vi phạm Rule 1 (không hại người)
func SecurityGate(atoms []OlangAtom) error {
    for _, a := range atoms {
        if isHarmful(a) {
            return fmt.Errorf("security: atom %U violates Rule 1", a.Codepoint)
        }
    }
    return nil
}

// '📡' U+1F4E1  Broadcast — gửi ISLMessage tới tất cả Agent
func Broadcast(msg ISLMessage, bus chan<- ISLMessage) { bus <- msg }

// '💾' U+1F4BE  Commit — lưu vào SilkGraph (append-only)
func Commit(node OlangNode, graph *SilkGraph) { graph.Add(node) }

// '🔄' U+1F504  Sync — xuất snapshot qua WebSocket
func Sync(world *World, ws WSConn) { ws.Send(world.Export()) }

} // end Op
```

---

# CHƯƠNG 4 — SILK GRAPH

## 4.1 SilkGraph

```go
type SilkGraph struct {
    mu      sync.RWMutex
    nodes   map[ISLAddress]*OlangNode
    ledger  []LedgerEntry   // append-only log
    sigKey  ed25519.PrivateKey
}

// Add thêm node (append-only — không DELETE, không OVERWRITE)
func (g *SilkGraph) Add(node OlangNode) {
    g.mu.Lock()
    defer g.mu.Unlock()
    existing, ok := g.nodes[node.Addr]
    if ok {
        existing.Weight++
        if existing.Weight >= 200 && existing.Sig == nil {
            existing.Sig = ed25519.Sign(g.sigKey, node.Addr.Bytes())
        }
        return
    }
    g.nodes[node.Addr] = &node
    g.ledger = append(g.ledger, LedgerEntry{TS: now(), Node: node})
}

// Query tìm node và đi theo silk edges
func (g *SilkGraph) Query(addr ISLAddress, depth int) []*OlangNode {
    // BFS depth-limited
    // confidence giảm 0.85 mỗi bước silk
}

// Traverse theo loại edge cụ thể
func (g *SilkGraph) Traverse(start ISLAddress, op rune) []ISLAddress

// Render: tính SDF của toàn bộ World tại điểm p
func (g *SilkGraph) Render(p Vec3) float64 {
    g.mu.RLock()
    defer g.mu.RUnlock()
    d := math.MaxFloat64
    for _, node := range g.nodes {
        if node.Gene != nil && node.Status == ACTIVE {
            d = math.Min(d, node.Gene.SDF(p))
        }
    }
    return d
}
```

## 4.2 Silk Edge operators (dùng ký tự OPTABLE)

```
Op   Ký tự   Ý nghĩa
─────────────────────────────────────────
∈    U+2208   thuộc về / member-of / phân loại
→    U+2192   produces / nhân quả / kết quả của
≡    U+2261   tương đương bản chất (khác ngữ cảnh)
⊂    U+2282   là tập con / is-a / kế thừa
∪    U+222A   hợp nhất hình học (SDF SmoothUnion)
∖    U+2216   loại trừ hình học (SDF SmoothSubtract)
⊥    U+22A5   độc lập / orthogonal (không liên quan)
∘    U+2218   compose / pipe / kết hợp
≈    U+2248   tương đồng gần (không hoàn toàn ≡)
```

## 4.3 Ngưỡng thăng cấp

```
Node status:
  Weight 1..9    = Lá đơn lẻ
  Weight 10..59  = Lá xác nhận — thêm silk edge đến nhánh gần nhất
  Weight 60..199 = Nhánh (Twig) — bất biến, LeoAI propose
  Weight 200+    = Cành (Branch) — ED25519 signed, AAM approve

Khi Weight đạt ngưỡng:
  LeoAI gửi MsgPropose đến AAM
  AAM kiểm tra + hỏi người dùng nếu cần
  YES → Commit vào SilkGraph, sign
  NO  → giữ nguyên, ghi lý do từ chối vào ledger
```

## 4.4 Xử lý trùng lặp (cùng bản chất, khác ngữ cảnh)

```
Cây ở Đà Lạt vs Cây ở Đà Nẵng:

  Gene: GIỐNG NHAU  → dùng chung 1 TreeGene template (SDF giống hệt)
  ISL:  KHÁC NHAU   → 2 OlangNode riêng biệt
    [W][T][Conifer][DaLat]    addr1
    [W][T][Conifer][DaNang]   addr2

  Silk edges:
    addr1 ──≡──► addr2   (tương đương bản chất)
    addr1 ──∈──► [W][T][Conifer]   (cùng loài)
    addr2 ──∈──► [W][T][Conifer]

  Khi query "cây thông":
    → cả addr1 và addr2 (kèm context địa điểm)
  Khi query "cây thông tại Đà Lạt":
    → chỉ addr1

Nguyên tắc:
  Cùng bản chất + khác ngữ cảnh = 2 node + 1 silk ≡
  Cùng bản chất + cùng ngữ cảnh = 1 node (dedup)
```

---

# CHƯƠNG 5 — BỘ NHỚ VÀ HỌC TẬP

## 5.1 Hai loại bộ nhớ

```
ĐN (Định Nghĩa đang học) = Short-term = Dendrites
  struct Observation {
      Addr       ISLAddress
      Value      []byte
      Confidence float64   // 0.0..1.0
      Source     string    // "sensor", "user", "teacher", "sync"
      TS         time.Time
      Status     ObsStatus // PENDING | CONFIRMED | REFUTED
  }
  Tự do thay đổi. Không cần cấp phép.
  Tồn tại trong RAM. Mất khi restart (trừ khi đã promote).

QR (Định Nghĩa đã chứng minh) = Long-term = Axon
  = OlangNode trong SilkGraph
  Cần AAM.Approve() để ghi.
  Append-only. ED25519 khi Weight >= 200.
  Tồn tại vĩnh viễn.
```

## 5.2 Vòng đời tri thức

```
QUAN SÁT → ĐN (Pending, conf=0.1)
    │
    ├── User bác bỏ         → Status=REFUTED, ghi log, kết thúc
    ├── Mâu thuẫn với QR    → gửi MsgPropose{conflict} lên AAM
    ├── Dreaming tự kiểm    → conf tăng/giảm
    └── conf >= 0.8         → MsgPropose lên AAM
                                  │
                                  ├── AAM.Approve = YES
                                  │     → Commit OlangNode vào SilkGraph
                                  │     → Status = CONFIRMED
                                  │     → Flush ĐN
                                  │
                                  └── AAM.Approve = NO
                                        → Status = REFUTED
                                        → ghi lý do vào ledger
```

## 5.3 Nhận biết đúng sai — Consensus

```go
// Khi nhiều nguồn quan sát cùng một thứ:
func mergeConfidence(observations []Observation) float64 {
    // 1 - (1-c1)(1-c2)(1-c3)...
    // Ba nguồn conf=0.3 → merged = 1-(0.7)³ = 0.657
    result := 1.0
    for _, o := range observations {
        result *= (1.0 - o.Confidence)
    }
    return 1.0 - result
}

// Mâu thuẫn giữa 2 nguồn:
// Không tự quyết. Gửi MsgPropose{conflict:true} lên AAM.
// Timeout chờ AAM: 30 giây
// Nếu AAM offline sau 30s: giữ observation có evidence nhiều hơn
// Ghi ConflictLog vào ledger (append-only)
```

## 5.4 Dreaming loop

```go
// Chạy khi inbox rỗng > 5 phút
// Goroutine idle priority (runtime.GOMAXPROCS không đếm)
func (l *LeoAI) dream(ctx context.Context) {
    ticker := time.NewTicker(5 * time.Minute)
    for {
        select {
        case <-ctx.Done(): return
        case <-ticker.C:
            if l.inbox.Len() > 0 { continue }  // có việc thì dừng dream
            l.runDreamCycle()
        }
    }
}

func (l *LeoAI) runDreamCycle() {
    for _, obs := range l.shortTerm.All() {
        // 1. Có mâu thuẫn với QR nào không?
        conflicts := l.silkGraph.FindConflicts(obs.Addr)
        if len(conflicts) > 0 {
            obs.Confidence *= 0.5  // giảm tin tưởng
            continue
        }
        // 2. Fibonacci structure?
        if obs.hasFibonacciPattern() { obs.Confidence += 0.1 }
        // 3. Spline fit tốt?
        if obs.splineFitScore > 0.8 { obs.Confidence += 0.1 }
        // 4. Đủ ngưỡng?
        if obs.Confidence >= 0.8 { l.proposeQR(obs) }
    }
}
```

## 5.5 Thăng cấp Lá → Nhánh → Cành

```
THĂNG CẤP xảy ra trong Dreaming:

1. Cluster detection:
   Tìm nhóm ĐN có ISL cùng [layer][group]:
     n >= 10 → propose Twig (Nhánh)
     n >= 60 → propose Branch (Cành)

2. Tạo node cha:
   parent_addr = ISLAddress{layer, group, 0, 0, 0}
   parent_gene = ∑(children.Gene) = SmoothUnion của tất cả
   parent_edges = ∀ child → SilkEdge{child.Addr, ∈}

3. Archive lá cũ (không xóa):
   child.Status = ARCHIVED
   child vẫn trong ledger, vẫn có thể query với flag ALL

4. Propose lên AAM → Approve → Commit
```

---

# CHƯƠNG 6 — AGENT SYSTEM

## 6.1 Skill interface

```go
// Agent = ID + []Skill. Không hơn không kém. (QT4)

type Skill interface {
    CanHandle(msg ISLMessage) bool
    Execute(ctx ExecContext) SkillResult
}

type ExecContext struct {
    Msg   ISLMessage
    State map[string]any   // shared state trong cùng 1 Agent
    Graph *SilkGraph
    World *World
}

type SkillResult struct {
    Response        *ISLMessage   // gửi đi, nil nếu không cần
    StateUpdates    map[string]any
    ProposeNewSkill *SkillProposal // nil nếu không
}

type Agent struct {
    id     ISLAddress
    skills []Skill
    state  map[string]any
    inbox  <-chan ISLMessage
    outbox chan<- ISLMessage
}

// Run: im lặng mặc định — chỉ sống khi có message
func (a *Agent) Run(ctx context.Context) {
    for {
        select {
        case <-ctx.Done(): return
        case msg := <-a.inbox:
            for _, s := range a.skills {
                if !s.CanHandle(msg) { continue }
                result := s.Execute(ExecContext{msg, a.state, ...})
                // merge state, gửi response, propose skill
            }
        }
    }
}
```

## 6.2 Phân cấp Agent

```
AAM (Agent AI Master)
  Skills: Decide4D · SecurityGate · Broadcast
  Giao tiếp DUY NHẤT với người dùng
  Nhận text/voice/image → chuyển thành ISLMessage
  KHÔNG bao giờ giao tiếp trực tiếp với Worker
    │
    │ ISLMessage (không bao giờ plain text)
    ├──────────────────────────────────────┐
    ▼                                      ▼
LeoAI Chief                          HomeChief
  Skills:                               Skills:
    ShortTermMemory                       Route
    SilkGraphQuery                        Aggregate
    Dreaming                                │
    Cluster                            ┌───┴────┐
    Promote                            ▼        ▼
    ED25519Sign                    LightAgent  HVACAgent
                               Skills:        Skills:
                                 ActuatorLight  ActuatorHVAC
                                 SplineSchedule SplineTemp

QUY TẮC (bất biến):
  ✅ AAM ↔ Chief
  ✅ Chief ↔ Chief ngang hàng
  ✅ Chief ↔ Worker của mình
  ❌ AAM ↔ Worker trực tiếp
  ❌ Worker ↔ Worker bất kỳ
  ❌ Plain text giữa bất kỳ Agent nào
```

## 6.3 4D Decision Engine (AAM)

```go
type DecisionContext struct {
    // Chiều 1 — Ngữ cảnh
    ActiveSession string     // "watching", "cooking", "sleeping"
    RecentEvents  []ISLAddress

    // Chiều 2 — Không gian
    UserLocation  ISLAddress // ISL của phòng hiện tại
    ActiveGenes   []ISLAddress

    // Chiều 3 — Thời gian
    TimeOfDay     time.Time
    Season        string

    // Chiều 4 — Ưu tiên
    SafetyAlert   bool
    Priority      byte  // 0=khẩn, 127=bình thường, 255=nền
}
```

---

# CHƯƠNG 7 — PIPELINE RENDER

## 7.1 Go → WebSocket → JS

```
Go (50ms/frame):
  world.Tick(dt)              // Animate tất cả Gene
  snapshot := world.Export()  // serialize
  ws.Broadcast(snapshot)      // gửi

WorldSnapshot JSON:
{
  "t": 3661.5,              // giây từ midnight
  "genes": [
    {
      "isl":  "4831000000000000",   // hex của ISLAddress
      "type": "tree",
      "pos":  [10.5, 0.0, 23.1],   // Vec3 origin
      "dna":  "∪(⌀(10,0,5,0.6),∪(●c1,●c2,●c3,0.3),0.3)",
      "r":    5.2                   // bounding sphere radius
    }
  ],
  "lights": [
    { "isl": "...", "pos": [...], "on": true, "color": [1,0.9,0.8] }
  ]
}

JS nhận snapshot:
  ws.onmessage = (e) => {
    const snap = JSON.parse(e.data)
    updateWorld(snap)
    // không re-create genes — chỉ update nếu gene đã tồn tại
    // thêm nếu gene mới, xóa khỏi render nếu ARCHIVED
  }
```

## 7.2 ISDF — Isometric SDF Engine

**ISDF** = Isometric SDF Engine. Là renderer chính của HomeOS, xây từ HomeOS-DNA.html và mở rộng.

### Nguyên lý ISDF

```
Không phải rasterizer thông thường.
Không phải ray-tracer GPU.
ISDF = Isometric projection + SDF sphere-trace + Silk signal overlay

Tại sao isometric?
  - Không có perspective distortion → dễ đọc thông tin
  - Depth sort đơn giản (painter's algorithm)
  - Phù hợp với HomeOS tile-based floor plan
  - Giữ nguyên tỉ lệ SDF across zoom levels

Tại sao SDF?
  - Mọi Gene đã là SDF rồi — không cần convert
  - rayMarch 256 bước → hit detection chính xác
  - Normal = ∇f → shading tự nhiên
  - LOD trivial: SimpleSDF() khi xa
```

### Pipeline ISDF

```
World space (wx,wy,wz)
    ↓
[1] Orbit rotation (camTheta, camPhi)
    Y-axis: ct=cos(θ), st=sin(θ)
      rx =  wx·ct + wz·st
      rz = -wx·st + wz·ct
    X-axis: cp=cos(φ), sp=sin(φ)
      ry =  wy·cp - rz·sp
      rz2=  wy·sp + rz·cp

[2] Isometric projection (TW=38, TH=19, HS=30)
    Perspective scale = camDist / (camDist + rz2·55) · (camDist/640)
    sx = W/2 + camPanX + (rx - rz2)·TW·0.5·scale
    sy = H/2 + camPanY + (rx + rz2)·TH·0.5·scale - ry·HS·scale

[3] Depth sort: sort by rz2 (back→front, painter's algorithm)

[4] Silk edges (2D screen-space, quadratic Bezier)
    control point = midpoint + perpendicular offset 9%
    5 loại: ∈ ≡ ♫ ∘ ≈ — mỗi loại 1 dash pattern

[5] Signal particles (chạy đúng TRÊN đường Bezier silk)
    Bezier eval tại t=s.p:
      bx = (1-t)²·pa.sx + 2(1-t)t·cpx + t²·pb.sx
      by = (1-t)²·pa.sy + 2(1-t)t·cpy + t²·pb.sy
    Trail: 5 ghost points tại t-Δt·[1..5]

[6] SDF sphere render (shaded by sunLight)
    normal = 𝔻['∇'](viewOffset, sphereSDF, ε)
    diffuse = 𝔻['·'](normal, sunLight.xyz)
    shade = sunLight.a + max(0,diffuse)·sunLight.i
    Fill: radial gradient (highlight offset = sun direction)
    Specular: second gradient (Phong)

[7] 𝔻['👁'] rayMarch 256 bước — hit detection
    for i in 256:
      d = sdf(origin + dir·t)
      if d < 0.001: return t  (HIT)
      t += d
      if t > 500:   return -1 (MISS)
    Dùng cho: hover detection, visibility test, RAYCAST mode
```

### SunLight Orbital Spline

```
𝔻['☀'](t):  t ∈ [0..24] giờ trong ngày
  angle a = (t-6)/24 · 2π
  lx = -cos(a)·0.6
  ly =  sin(a)·0.5 + 0.5
  lz = -0.4
  intensity i = max(0, sin((t-6)/12·π))
  ambient   a = 0.25

Ảnh hưởng:
  Sky colour: rgb(2+day·7, 3+day·9, 8+day·20)
  Stars: fade proportional to (1-dayFrac)
  Node shading: litL = baseL · (0.38 + shade·0.72)
  Specular: highlight offset = sun direction vector
```

### Spline Node Animation

```
Catmull-Rom: mỗi node có 4 control points orbit riêng
  Phase = f(node.id.charCodeAt(0), node.id.length)
  Amplitude = LR[layer] · 0.038
  Period = độc lập per node → không đồng bộ

splineEval(kf[], t):
  t periodic trong [0,1)
  Catmull-Rom với 4 points: P0,P1,P2,P3
  Trả về Vec3 position

Origin node: kf = 4×{0,0,0} → tuyệt đối bất động
L1 nodes: amp ≈ 0.285 world units (nhẹ)
L3 nodes: amp ≈ 0.065 world units (rất nhẹ)
```

### WebSocket → updateWorld

```go
// internal/ws/server.go — gửi 20fps
type WorldSnapshot struct {
    Time    float64     `json:"time"`    // world time [0..24]
    Nodes   []NodeState  `json:"nodes"`
    Signals []SigState   `json:"signals"`
    Stats   StatsState   `json:"stats"`
}
type NodeState struct {
    ID   string  `json:"id"`
    Glow float64 `json:"glow"`
    Sig  float64 `json:"sig"`
    X,Y,Z *float64 `json:"x,y,z,omitempty"` // override position nếu cần
}
type SigState struct {
    From string  `json:"from"`
    To   string  `json:"to"`
    P    float64 `json:"p"`  // progress [0..1]
}
```

```javascript
// JS nhận
ws.onmessage = (evt) => {
    const snap = JSON.parse(evt.data)
    updateWorld(snap)  // → update NODES[id].glow, sig, inject SIGS[]
}
```

### ISDF Safety (lỗi đã fix)

```
Vấn đề: zoom quá gần → scale lớn → sr2 lớn
         zoom quá xa  → scale nhỏ → sr2 có thể → 0 hoặc âm
         → createRadialGradient(r < 0) → IndexSizeError

Fix:
  sr2 = max(0.5, nodeR · scale · (camDist/120))
  tất cả r argument: max(0.5, value)
  tDist clamp: [280, 1400] (không cho camDist < 280)
  arc radius: Math.max(0.5, sr2)

Vấn đề: signal đi ngoài silk edge (đường thẳng vs Bezier)

Fix:
  Signal dùng cùng control point như silk edge:
    cpx = (pa.sx+pb.sx)/2 + (pb.sy-pa.sy)·0.09
    cpy = (pa.sy+pb.sy)/2 - (pb.sx-pa.sx)·0.09
  Bezier eval: bx = (1-t)²·pa + 2(1-t)t·cp + t²·pb
  Trail cũng dùng Bezier (không lerp thẳng)
```

## 7.3 LOD (Level of Detail)

```
Khoảng cách camera → Gene:
  < 50u    Gene.SDF() đầy đủ (tất cả capsule + sphere)
  50–200u  Gene.SimpleSDF() — 1 capsule + 1 sphere
  200–2km  1 ellipse + màu trung bình
  2–20km   density color map (không raycast)
  > 20km   terrain silhouette (chỉ FBM)
```

---

# CHƯƠNG 8 — SILK-SYNC (Olang ↔ Olang)

## 8.1 Giao thức

```
Transport: WebSocket (ws://)
Auth: ED25519 keypair — mỗi Olang có 1 keypair cố định
      Handshake: trao đổi public key + sign challenge
Frequency: event-driven + periodic 5 phút/lần
Delta sync: chỉ gửi ledger entries sau timestamp T
            không dump toàn bộ QR

Message format (dùng ISLMessage với MsgType=MsgSync):
  Payload: {
    "since": 1741234567,          // unix timestamp
    "entries": [LedgerEntry...],  // chỉ QR, không gửi ĐN
    "sig": "ed25519_signature"
  }
```

## 8.2 Conflict resolution

```
OlangA gửi QR{addr, value_A}
OlangB có QR{addr, value_B} khác value_A

Quy trình:
  1. So sánh evidence count: bên nào có weight cao hơn?
  2. So sánh timestamp: bên nào quan sát gần đây hơn?
  3. Nếu vẫn tie:
     new_value = merge(value_A, value_B)
     new_node{value: merged, uncertainty: HIGH}
  4. Ghi ConflictLog vào cả 2 ledger
  5. AAM của mỗi bên được thông báo

Timeout AAM: 30 giây → fallback về evidence count cao hơn
```

## 8.3 Học từ AI bên ngoài (Teacher API)

```
Pipeline:
  OlangA gửi ISLQuery → Teacher (Claude/GPT/Local)
  Teacher trả về text
  OlangA:
    parse text → candidate OlangAtoms
    Confidence = 0.3 (không tin ngay)
    Đặt vào ĐN (Short-term)
    Dreaming kiểm chứng với QR hiện có
    Nếu consistent → confidence tăng
    Nếu conflict   → confidence giảm, flag conflict
    Nếu conf >= 0.8 → MsgPropose lên AAM

Tại sao không tin ngay:
  AI bên ngoài không biết ngữ cảnh của HomeOS cụ thể.
  Chỉ QR của chính HomeOS mới là sự thật của HomeOS.
```

---

# CHƯƠNG 9 — ADAPTIVE TRAINING LOOP

```go
func AdaptiveConfig() TrainingConfig {
    var mem runtime.MemStats
    runtime.ReadMemStats(&mem)
    cpu := runtime.NumCPU()

    switch {
    case mem.Sys > 64*GB && cpu >= 32:
        return TrainingConfig{MaxIter: 1_000_000_000_000}
    case mem.Sys > 16*GB && cpu >= 8:
        return TrainingConfig{MaxIter: 1_000_000_000}
    case mem.Sys > 4*GB && cpu >= 4:
        return TrainingConfig{MaxIter: 100_000_000}
    default:  // thiết bị nhúng, Raspberry Pi
        return TrainingConfig{
            MaxIter:   100_000,
            TimeLimit: 5 * time.Minute,
        }
    }
}

// outOfResources: dừng khi RAM > 90% HOẶC có task quan trọng
func outOfResources() bool {
    var m runtime.MemStats
    runtime.ReadMemStats(&m)
    ramUsed := float64(m.Alloc) / float64(m.Sys)
    return ramUsed > 0.90 || inboxHasUrgent()
}

// Training không block. Chạy goroutine idle.
// Checkpoint: mỗi 10k iter → ghi state vào ledger
// Resume: đọc ledger entry cuối cùng có type=CHECKPOINT
```

---

# CHƯƠNG 10 — CẤU TRÚC THƯ MỤC

```
homeos/
│
├── internal/
│   ├── gene/
│   │   ├── vec3.go          Vec3 + phép toán
│   │   ├── sdf.go           Tất cả SDF primitives
│   │   ├── optable.go       Op map[rune]interface{} — 48 ký tự
│   │   ├── gene.go          Gene interface + DefaultNormal
│   │   ├── spline.go        Keyframe + SplineEval + FitSpline
│   │   └── instances/
│   │       ├── terrain.go   TerrainGene: ∫(fbm)
│   │       ├── tree.go      TreeGene: ∪(⌀,∪(●,●,●))
│   │       ├── sun.go       SunGene: ☀(t) + Spline quỹ đạo
│   │       └── light.go     LightGene: ● khi bật / ◌ khi tắt
│   │
│   ├── silk/
│   │   ├── atom.go          OlangAtom{Codepoint, ISL}
│   │   ├── edge.go          SilkEdge{To ISLAddress, Op rune}
│   │   ├── graph.go         SilkGraph: Add/Query/Traverse/Render
│   │   ├── compiler.go      Tokenize/Resolve/Encode/Decode (12B/sym)
│   │   ├── ledger.go        Append-only log + ED25519
│   │   └── bootstrap.go     Đọc file.olang → khởi động World
│   │
│   ├── isl/
│   │   ├── address.go       ISLAddress uint64 + namespace
│   │   ├── message.go       ISLMessage + MsgType
│   │   └── codec.go         AES-256-GCM encode/decode
│   │
│   ├── memory/
│   │   ├── shortterm.go     Observation + ShortTermMemory
│   │   ├── longterm.go      = SilkGraph (wrapper)
│   │   └── dreaming.go      Idle loop: kiểm chứng + promote
│   │
│   ├── learning/
│   │   ├── cluster.go       Fibonacci-based clustering
│   │   ├── promote.go       Lá→Nhánh→Cành
│   │   ├── conflict.go      Resolve mâu thuẫn giữa nguồn
│   │   └── training.go      AdaptiveTrainingLoop
│   │
│   ├── perception/
│   │   ├── fibonacci.go     Phi + IsFibRatio + DetectSpiral
│   │   ├── spline_fit.go    FitSpline từ raw sensor samples
│   │   ├── vision.go        VisionChief: image → Fibonacci SDF → ISL
│   │   └── audio.go         AudioChief: audio → Spline formant → ISL
│   │
│   ├── agents/
│   │   ├── skill.go         Skill interface + ExecContext + SkillResult
│   │   ├── agent.go         Agent struct + Run()
│   │   └── registry.go      SkillRegistry
│   │
│   ├── aam/
│   │   ├── aam.go           AAM + 4D DecisionContext
│   │   └── security.go      SecurityGate (Rule 1 bất biến)
│   │
│   ├── leoai/
│   │   ├── leoai.go         LeoAI Chief
│   │   └── teach.go         Teacher API integration
│   │
│   ├── sync/
│   │   ├── silk_sync.go     Olang ↔ Olang SILK-SYNC
│   │   └── teacher.go       Học từ AI bên ngoài
│   │
│   └── ws/
│       ├── snapshot.go      WorldSnapshot + Export()
│       └── server.go        WebSocket :8080/ws
│
├── cmd/homeos/
│   └── main.go              Đọc file.olang → bootstrap
│
└── world/
    └── world.go             World: []Gene + Tick() + SDF() + Export()
```

---

# CHƯƠNG 11 — QUY TẮC BẤT BIẾN

```
① 🛡 Không hại con người
   Tuyệt đối. Không ai ghi đè. SecurityGate chạy TRƯỚC mọi thứ.

② Append-only
   ○ chỉ WRITE. Không DELETE. Không OVERWRITE.
   Node cũ → ARCHIVED, không mất.

③ ISL = binary
   ISLAddress = uint64. Không bao giờ là string trong nội bộ.
   Giao tiếp Agent = ISLMessage. Không bao giờ plain text.

④ SDF lưu công thức
   Không lưu hình dạng. Không lưu pixel. Không lưu vertex.

⑤ Spline thay thế discrete array
   Mọi sensor = Spline. Không dùng []float64 raw.
   Mọi chuyển động = Catmull-Rom. Không dùng lerp thẳng.

⑥ Olang mượn UTF-32, không đọc UTF-32
   opTable = 48 entries Go function. Không cần file nào.
   Unicode Tree = đọc cấu trúc vào, không tạo lại.

⑦ file.olang = tế bào gốc
   1 file nhỏ → toàn bộ World tự khởi động.

⑧ ĐN = chưa chắc. QR = đã chứng minh.
   Không ghi thẳng vào QR từ nguồn ngoài.

⑨ Fibonacci = cấu trúc nhận diện
   VisionChief dùng SDF + Fibonacci, không dùng pixel matching.

⑩ Agent im lặng mặc định
   Chỉ phát tín hiệu khi có ∆. CPU ≈ 0 khi không có việc.

⑪ ISDF = renderer duy nhất
   Isometric + SDF sphere-trace + Silk signal overlay.
   Signal đi TRÊN đường Bezier của silk edge (không đi ngoài).
   tDist ∈ [280, 1400] — không cho sr2 âm.

⑫ UX hiện tại giữ nguyên
   covenant.html · login · sidebar · chat composer.
   Chỉ thay rafISO() bằng ISDF engine.
```

---

# CHƯƠNG 12 — VÍ DỤ CHẠY THẬT

## 12.1 "Tắt đèn phòng khách"

```
1. User nói → AudioChief
   Spline formant → ISL{꜉tắt, ꜈đèn, ꜈phòng_khách}
   → ISLAddress: [H][A][off][living]

2. AAM nhận ISLMessage
   🛡 SecurityGate: không hại → pass
   4D Decision: location=living, time=22:00 → HomeChief

3. HomeChief
   SilkGraph.Traverse([H][A][living], ∈):
     → LightGene_living (ISLAddress đã biết)
   Gửi ISLMessage{MsgActivate, to=LightAgent}

4. LightAgent
   Skill ActuatorLight.Execute():
     LightGene.Toggle(OFF)
     LightGene.sdf = SDFVoid()
     gpio.Write(pin, LOW)

5. World update
   SilkGraph.Update(LightGene)
   snapshot → WebSocket → JS render phòng tối

6. Feedback
   ISLMessage ngược lên → AAM → TTS
   "Đèn phòng khách đã tắt."

Tổng data nội bộ: ~30 bytes ISL
```

## 12.2 "Cái chân trong ảnh là của loài vật nào?"

```
SONG SONG:
  AudioChief: Spline formant
    "chân" → [P][Body][limb][?]
    "loài vật" → [B][Animal][?][?]

  VisionChief: Fibonacci SDF analysis
    Phát hiện tỉ lệ đốt: 1:φ → [P][Body][limb][thin]
    Phát hiện màu hồng nhạt → [C][Pink][light][?]
    FBM pattern cát → [L][Coast][sand][?]

LeoAI query SilkGraph:
  Traverse([P][Body][limb][thin], tơ):
    → [B][Bird][seagull] (chân mỏng + hồng = mòng biển ✅)
  Confirm: [C][Pink] → [B][Bird][seagull] ✅
  Confirm: [L][Coast][sand] → [B][Bird][seagull] ✅
  confidence = 0.97

AAM → TTS: "Đó là chân chim mòng biển."
```

---

*Append-only · Không được xóa · Cập nhật: 2026-03-07*
