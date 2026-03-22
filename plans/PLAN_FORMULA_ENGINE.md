# PLAN — Formula Engine: Giá trị = Công thức = Hình dạng

**Ngày:** 2026-03-22
**Vấn đề:** 3/5 chiều (R, V, A) có công thức trong UDC_DOC nhưng code KHÔNG dùng.
Giá trị chỉ là số tĩnh — không evaluate, không reconstruct, không render.

**Nguyên tắc:** Đọc giá trị → biết công thức → biết hình dạng. KHÔNG cần ai giải thích.

---

## Hiện trạng

```
S: 18 SDF primitives → vsdf crate evaluate f(p) → DÙNG ✅
   Đọc S=1 (BOX) → gọi f(p) = max(|x|-a, |y|-b) → render hình vuông

R: 16 relation types → CHỈ LÀ SỐ ❌
   Đọc R=5 (ARITHMETIC) → ???  không có gì xảy ra
   ĐÚNG RA: R=5 → (ℤ,+,×) ring → biết đây là phép toán → compose theo ring rules

V: 8 mức valence → CHỈ LÀ SỐ ❌
   Đọc V=6 (rất tích cực) → ???  chỉ so sánh >, <
   ĐÚNG RA: V=6 → giếng thế sâu U=-V₀+½kx² → biết lực hút mạnh → approach behavior

A: 8 mức arousal → CHỈ LÀ SỐ ❌
   Đọc A=7 (cực kích thích) → ???  chỉ so sánh
   ĐÚNG RA: A=7 → supercritical E>>E_th → biết hệ bùng nổ → urgent response

T: 4 mức time → Có 164 refs nhưng chủ yếu static labels ❌
   Đọc T=3 (RHYTHMIC) → ???  label tĩnh
   ĐÚNG RA: T=3 → ψ=A·sin(2πft+φ) → biết có nhịp → temporal pattern
   T CẦN LÀ SPLINE KNOT — mỗi observation = 1 điểm trên đường cong
```

---

## Thiết kế: Formula Engine

### Nguyên lý

```
P_weight = [S:4][R:4][V:3][A:3][T:2] = 16 bits

Mỗi giá trị KHÔNG CHỈ là số — nó là INDEX vào bảng công thức.

S=3 → PLANE → f(p) = p.y - h
     Tôi đọc "3" → tôi BIẾT nó phẳng → tôi BIẾT gradient = (0,1,0)
     Không cần tra bảng. Giá trị TỰ MÔ TẢ.

R=5 → ARITHMETIC → (ℤ,+,×) ring
     Tôi đọc "5" → tôi BIẾT compose = ring operation
     a ∘ b = a + b (cộng) hoặc a × b (nhân) tùy context

V=6 → DEEP WELL → U = -V₀ + ½kx²
     Tôi đọc "6" → tôi BIẾT approach behavior
     Lực hút F = -dU/dx > 0 → hệ muốn đến gần

A=7 → SUPERCRITICAL → E >> E_th, positive feedback
     Tôi đọc "7" → tôi BIẾT urgent/explosive
     Phản ứng dây chuyền R(t) = R₀·e^(λt)

T=3 → RHYTHMIC → ψ(t) = A·sin(2πft + φ)
     Tôi đọc "3" → tôi BIẾT có pattern lặp
     Cần: frequency f, amplitude A, phase φ → encode vào spline knot
```

### Cấu trúc Formula Dispatch

```rust
/// Evaluate formula cho dimension S
fn eval_shape(s: u8, point: Vec3) -> f64 {
    match s {
        0  => sphere(point),      // |p| - r
        1  => box_sdf(point),     // max(|p| - b, 0)
        2  => capsule(point),     // |p - clamp| - r
        3  => plane(point),       // p.y - h
        ...
        17 => death_star(point),  // opSubtract
        _  => 0.0,
    }
}

/// Evaluate formula cho dimension R
fn eval_relation(r: u8, a: &Molecule, b: &Molecule) -> Molecule {
    match r {
        0  => identity(a),             // a → a
        1  => member(a, b),            // a ∈ b
        2  => subset(a, b),            // a ⊂ b
        3  => equality(a, b),          // a ≡ b
        4  => order(a, b),             // a < b (partial order)
        5  => arithmetic(a, b),        // ring (ℤ,+,×)
        6  => logical(a, b),           // Boolean a∧b, a∨b
        7  => set_op(a, b),            // A∪B, A∩B
        8  => compose(a, b),           // g∘f
        9  => causes(a, b),            // a → b (causality)
        10 => approximate(a, b),       // a ≈ b (d < ε)
        11 => orthogonal(a, b),        // a ⊥ b
        12 => aggregate(a, b),         // Σ, ∫
        13 => directional(a, b),       // a → b (vector)
        14 => bracket(a, b),           // (a, b) grouping
        15 => inverse(a, b),           // a⁻¹
        _  => identity(a),
    }
}

/// Evaluate formula cho dimension V
fn eval_valence(v: u8) -> ValenceState {
    match v {
        0 => ValenceState::Repeller   { barrier: HIGH,   force: STRONG_REPEL },
        1 => ValenceState::LowBarrier { barrier: LOW,    force: MILD_REPEL },
        2 => ValenceState::LowBarrier { barrier: LOW,    force: MILD_REPEL },
        3 => ValenceState::Flat       { gradient: ZERO },
        4 => ValenceState::Flat       { gradient: ZERO },
        5 => ValenceState::ShallowWell{ depth: MODERATE, force: MILD_ATTRACT },
        6 => ValenceState::DeepWell   { depth: HIGH,     force: STRONG_ATTRACT },
        7 => ValenceState::DeepWell   { depth: VERY_HIGH,force: VERY_STRONG },
        _ => ValenceState::Flat       { gradient: ZERO },
    }
}

/// Evaluate formula cho dimension A
fn eval_arousal(a: u8) -> ArousalState {
    match a {
        0 => ArousalState::GroundState    { energy: E0,     action: FROZEN },
        1 => ArousalState::Overdamped     { gamma: HIGH,    decay: FAST },
        2 => ArousalState::Overdamped     { gamma: MODERATE,decay: MODERATE },
        3 => ArousalState::Equilibrium    { delta_g: ZERO },
        4 => ArousalState::Equilibrium    { delta_g: ZERO },
        5 => ArousalState::ExcitedState   { gamma: LOW,     oscillation: MILD },
        6 => ArousalState::ExcitedState   { gamma: VERY_LOW,oscillation: STRONG },
        7 => ArousalState::Supercritical  { energy: HIGH,   feedback: POSITIVE },
        _ => ArousalState::Equilibrium    { delta_g: ZERO },
    }
}

/// Evaluate formula cho dimension T — SPLINE KNOT
fn eval_time(t: u8, observation: &Observation) -> SplineKnot {
    match t {
        0 => SplineKnot::Timeless   { phase: STATIC },
        1 => SplineKnot::Sequential { duration: observation.duration,
                                      order: observation.sequence_index },
        2 => SplineKnot::Cyclical   { period: observation.cycle_period,
                                      phase: observation.cycle_phase },
        3 => SplineKnot::Rhythmic   { frequency: observation.freq,
                                      amplitude: observation.amp,
                                      phase: observation.phase },
        _ => SplineKnot::Timeless   { phase: STATIC },
    }
}
```

---

## T Triệt Để — Spline Accumulation

```
Mỗi lần observe 1 concept → tạo 1 SplineKnot → append vào T history.

Ví dụ: observe "Hình vuông" 3 lần:

  Lần 1: đọc tên "Hình vuông"
    → T₁ = SplineKnot { duration: 200ms, freq: 0, amp: 1.0, phase: 0 }

  Lần 2: đọc alias "Rectangle"
    → T₂ = SplineKnot { duration: 150ms, freq: 0, amp: 0.8, phase: π/3 }

  Lần 3: đọc định nghĩa "S = a²"
    → T₃ = SplineKnot { duration: 500ms, freq: 0, amp: 1.5, phase: 2π/3 }

  T_history["Hình vuông"] = [T₁, T₂, T₃]

  Spline interpolation: ψ(t) = Σ Tᵢ.amp × B(t - tᵢ)
  → Đường cong học tập của "Hình vuông"
  → Đọc spline → biết: "đã gặp 3 lần, mỗi lần sâu hơn (amp tăng)"

Sensor "light":
  Lần 1: λ = 580nm, I = 1000 lux
    → T₁ = SplineKnot { frequency: c/580nm, amplitude: 1000, phase: 0 }
    → Silk → P{"ánh sáng vàng"}

  Lần 2: λ = 450nm, I = 500 lux
    → T₂ = SplineKnot { frequency: c/450nm, amplitude: 500, phase: π/4 }
    → Silk → P{"ánh sáng xanh"}

  T_history["light"] = [T₁, T₂]
  → Spline → biết: "ánh sáng thay đổi từ vàng → xanh, cường độ giảm"
```

---

## Tasks

| ID | Task | Effort | Status |
|----|------|--------|--------|
| FE.1 | Formula dispatch cho R (16 relation types → operations) | ~200 LOC | FREE |
| FE.2 | Formula dispatch cho V (8 levels → ValenceState) | ~100 LOC | FREE |
| FE.3 | Formula dispatch cho A (8 levels → ArousalState) | ~100 LOC | FREE |
| FE.4 | T SplineKnot accumulation (mỗi observe → append knot) | ~300 LOC | FREE |
| FE.5 | T Spline interpolation (history → curve → behavior) | ~200 LOC | FREE |
| FE.6 | Wire formula engine vào pipeline (encode → eval → store) | ~200 LOC | FREE |
| FE.7 | Test: đọc P_weight → reconstruct formula → verify shape | ~150 LOC | FREE |

Tổng: ~1,250 LOC. Đây là **core architecture** — biến P_weight từ số tĩnh thành **công thức sống**.

---

## Dependency

```
FE.1 (R dispatch) ← independent
FE.2 (V dispatch) ← independent
FE.3 (A dispatch) ← independent
FE.4 (T knots)    ← independent
FE.5 (T spline)   ← FE.4
FE.6 (wire)       ← FE.1-5
FE.7 (tests)      ← FE.6

FE.1-4 song song. FE.5 sau FE.4. FE.6-7 cuối.
```

---

## Tại sao quan trọng

```
HIỆN TẠI:
  P_weight = 2 bytes số tĩnh → so sánh >, < → xong
  "Hình vuông" gặp 1 lần = gặp 100 lần (T không thay đổi)
  Sensor đo 580nm = đo 450nm (T không phân biệt)

SAU FORMULA ENGINE:
  P_weight = 2 bytes → dispatch → CÔNG THỨC SỐNG
  "Hình vuông" gặp 100 lần → T spline dày → "hiểu sâu"
  580nm vs 450nm → T knots khác → "phân biệt được"

  Giá trị TỰ MÔ TẢ. Đọc số → biết hình dạng.
  Không cần AI, không cần lookup, không cần annotation.
  DNA không cần giải thích ATCG — ribosome ĐỌC và LÀM.
```
