# PLAN — Formula Engine: Gia tri = Cong thuc = Hinh dang

**Ngay:** 2026-03-22
**Cap nhat:** 2026-03-22 (ban chi tiet + T×S insight)
**Van de:** 3/5 chieu (R, V, A) co cong thuc trong UDC_DOC nhung code KHONG dung.

---

## ⚠️ CRITICAL INSIGHT: T × S = Vo han hinh dang

```
T KHONG CHI LA "thoi gian" — T LA THAM SO CHO SDF.

  S = WHAT shape (sphere, box, cylinder...)
  T = HOW BIG, WHERE, HOW FAST (amplitude, frequency, phase)
  S × T = hinh dang CU THE voi kich thuoc, vi tri, chuyen dong

Vi du:
  S=SPHERE, T={amp=3.0, phase=0}           → qua cau lon (r=3.0)
  S=SPHERE, T={amp=1.5, phase=π}           → qua cau nho (r=1.5), dich len tren
  compose(L, N) = union(f_L, f_N)          → ⛄ NGUOI TUYET

  S=BOX, T={amp=2.0, freq=0.1}            → hop rung (size=2, oscillate)
  S=TORUS, T={amp=0.5, freq=1.0}          → donut xoay
  S=CYLINDER, T={amp=5, phase=π/2}        → ong nghieng

18 SDF primitives × T spline knots = VO HAN hinh dang tu HUU HAN nguyen lieu.

SDF formula co san:
  f(p) = |p| - r         SPHERE    nhung r = T.amplitude
  f(p) = max(|p|-b, 0)   BOX       nhung b = T.amplitude × (1,1,1)
  f(p) = p.y - h         PLANE     nhung h = T.amplitude
  ... moi SDF primitive co THAM SO ma T cung cap.

T.frequency → chuyen dong (oscillation, rotation)
T.phase → vi tri/goc (offset, orientation)
T.amplitude → kich thuoc (radius, width, height)
T.duration → thoi gian ton tai (lifetime)

V du phuc tap:
  "nha" = compose(
    BOX   + T{amp=(4,3,4)}     → than nha (4×3×4)
    PYRAMID + T{amp=(5,2,5)}   → mai nha (tam giac tren)
    BOX   + T{amp=(1,2,0.1)}   → cua (hop nho)
  )

  "nguoi" = compose(
    SPHERE   + T{amp=1.0}      → dau
    CYLINDER + T{amp=(0.5,3)}  → than
    CYLINDER + T{amp=(0.3,2)}  → tay ×2
    CYLINDER + T{amp=(0.4,3)}  → chan ×2
  )

  BAT KY concept nao cung co the RENDER thanh hinh 3D
  vi S cho hinh dang, T cho kich thuoc, R cho cach ghep,
  V cho mau sac (gradient tu potential), A cho chuyen dong.

  5D → 1 vat the hoan chinh: hinh + quan he + cam xuc + nang luong + thoi gian.
```

---
Gia tri chi la so tinh — khong evaluate, khong reconstruct, khong render.

**Nguyen tac:** Doc gia tri → biet cong thuc → biet hinh dang. KHONG can ai giai thich.

---

## Hien trang

```
S: 18 SDF primitives → vsdf crate evaluate f(p) → DUNG ✅
   Doc S=1 (BOX) → goi f(p) = max(|x|-a, |y|-b) → render hinh vuong
   Code: crates/vsdf/src/shape/sdf.rs — SdfKind enum + sdf() dispatch

R: 16 relation types → CHI LA SO ❌
   Doc R=5 (ARITHMETIC) → ???  khong co gi xay ra
   DUNG RA: R=5 → (Z,+,x) ring → biet day la phep toan → compose theo ring rules
   Code hien tai: RelationBase enum trong molecular.rs — chi co 8 base, KHONG co eval

V: 8 muc valence → CHI LA SO ❌
   Doc V=6 (rat tich cuc) → ???  chi so sanh >, <
   DUNG RA: V=6 → gieng the sau U=-V0+½kx² → biet luc hut manh → approach behavior
   Code hien tai: EmotionDim { valence: u8 } — chi la byte, KHONG co physics

A: 8 muc arousal → CHI LA SO ❌
   Doc A=7 (cuc kich thich) → ???  chi so sanh
   DUNG RA: A=7 → supercritical E>>E_th → biet he bung no → urgent response
   Code hien tai: EmotionDim { arousal: u8 } — chi la byte, KHONG co physics

T: 4 muc time → Co 164 refs nhung chu yeu static labels ❌
   Doc T=3 (RHYTHMIC) → ???  label tinh
   DUNG RA: T=3 → psi=A·sin(2pift+phi) → biet co nhip → temporal pattern
   Code hien tai: TimeDim enum (5 bases) + VectorSpline da co trong vsdf
   T CAN LA SPLINE KNOT — moi observation = 1 diem tren duong cong
```

---

## Thiet ke: Formula Engine

### Nguyen ly

```
P_weight = [S:4][R:4][V:3][A:3][T:2] = 16 bits

Moi gia tri KHONG CHI la so — no la INDEX vao bang cong thuc.

S=3 → PLANE → f(p) = p.y - h
     Toi doc "3" → toi BIET no phang → toi BIET gradient = (0,1,0)
     Khong can tra bang. Gia tri TU MO TA.

R=5 → ARITHMETIC → (Z,+,x) ring
     Toi doc "5" → toi BIET compose = ring operation
     a ∘ b = a + b (cong) hoac a x b (nhan) tuy context

V=6 → DEEP WELL → U = -V0 + ½kx²
     Toi doc "6" → toi BIET approach behavior
     Luc hut F = -dU/dx > 0 → he muon den gan

A=7 → SUPERCRITICAL → E >> E_th, positive feedback
     Toi doc "7" → toi BIET urgent/explosive
     Phan ung day chuyen R(t) = R0·e^(lambda·t)

T=3 → RHYTHMIC → psi(t) = A·sin(2pi·f·t + phi)
     Toi doc "3" → toi BIET co pattern lap
     Can: frequency f, amplitude A, phase phi → encode vao spline knot
```

---

## Anh xa P_weight bits → UDC formula groups

### QUAN TRONG: Cach doc P_weight

```
Molecule.bits: u16 = [S:4][R:4][V:3][A:3][T:2]

Trich xuat:
  let s = (bits >> 12) & 0x0F;  // 0..15
  let r = (bits >>  8) & 0x0F;  // 0..15
  let v = (bits >>  5) & 0x07;  // 0..7
  let a = (bits >>  2) & 0x07;  // 0..7
  let t = bits & 0x03;           // 0..3

Code hien tai da co cac extractor nay trong Molecule (s(), r(), v(), a(), t()).

MOI GIA TRI la INDEX. Doc index → biet GROUP → biet CONG THUC.
```

---

# =====================================================================
# FE.1 — R DISPATCH: 16 Relation Types → Category Theory Operations
# =====================================================================

## FE.1.1 Rust Structs

**File:** `crates/olang/src/mol/relation_eval.rs` (TAO MOI)
**Dang ky:** them `pub mod relation_eval;` trong `crates/olang/src/mol/mod.rs`

```rust
//! # relation_eval — Formula dispatch cho chieu R
//!
//! Doc R index (0-15) → biet nhom quan he → biet phep toan.
//! Moi nhom = 1 subcategory trong Category Theory.

use super::molecular::Molecule;

/// Ket qua cua 1 phep quan he giua 2 Molecule.
///
/// Khong tra ve Molecule moi — tra ve DAC TA cua quan he.
/// Pipeline phia sau dung dac ta nay de quyet dinh hanh vi.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelationResult {
    /// R=0: Phep toan dai so (Group/Ring/Field morphism)
    /// Doi voi cong/tru/nhan/chia giua 2 molecule.
    Algebraic {
        /// Kieu cau truc dai so: 0=group(+), 1=ring(+,x), 2=field(+,x,/)
        algebra_kind: u8,
        /// true = giao hoan (a*b = b*a)
        commutative: bool,
    },

    /// R=1: Quan he thu tu (Partial/Total order)
    /// a < b, a <= b, a > b, a >= b
    Order {
        /// -1 = a < b, 0 = a = b, 1 = a > b
        ordering: i8,
        /// true = thu tu toan phan (moi cap so sanh duoc)
        total: bool,
    },

    /// R=2: Bien doi bieu dien (Font functor V→V)
    /// Bold = scale, Italic = shear, DoubleStruck = projection
    Representation {
        /// 0=identity, 1=bold(scale), 2=italic(shear), 3=bold_italic
        /// 4=fraktur(lattice), 5=double_struck(projection), 6=script(affine)
        /// 7=monospace(quantize)
        transform_kind: u8,
    },

    /// R=3: Ma hoa so (Positional numeral encoding)
    /// N = Sum(d_i * b^i)
    Numeral {
        /// Co so: 2=binary, 10=decimal, 16=hex, 60=sexagesimal
        base: u16,
        /// Gia tri so (da giai ma)
        value: i64,
    },

    /// R=4: Dau cau — PDA operations (push/pop stack)
    Punctuation {
        /// 0=push(open bracket), 1=pop(close bracket), 2=separator, 3=terminal
        stack_op: u8,
        /// Do sau stack hien tai (0 = can bang)
        depth_delta: i8,
    },

    /// R=5: Quy doi tuyen tinh (Currency exchange matrix)
    LinearMap {
        /// Ty gia tuong doi (gia tri trong don vi tham chieu)
        rate: f32,
    },

    /// R=6: He cong tinh (Additive numeral, so co dai)
    Additive {
        /// Gia tri = tong don gian cac ky hieu
        value: i64,
    },

    /// R=7: Chuyen trang thai (DFA transition function)
    Automaton {
        /// 0=NUL(no-op), 1=activate, 2=inhibit, 3=push_dir, 4=pop_dir
        transition: u8,
    },

    /// R=8..15: Cac quan he bo sung (tu RelationBase hien tai)
    /// 8=Member, 9=Subset, 10=Equiv, 11=Orthogonal
    /// 12=Compose, 13=Causes, 14=Similar, 15=DerivedFrom
    CategoryMorphism {
        /// Ten morphism (khop voi RelationBase enum hien tai)
        kind: u8,
        /// true neu la invertible (iso/diffeomorphism)
        invertible: bool,
    },
}

/// Tinh chieu R dua tren UDC_R_RELATION_tree.md
///
/// Input: r = 4 bits tu P_weight (0..15)
///
/// Anh xa:
///   0  → R.0 TOAN TU    (Algebraic: group/ring/field)
///   1  → R.1 SO SANH     (Order: partial/total)
///   2  → R.2 CHU TOAN    (Representation: font transform)
///   3  → R.3 SO           (Numeral: positional encoding)
///   4  → R.4 DAU CAU     (Punctuation: PDA push/pop)
///   5  → R.5 TIEN TE     (LinearMap: exchange rate matrix)
///   6  → R.6 CO DAI      (Additive: sum of symbol values)
///   7  → R.7 DIEU KHIEN  (Automaton: state transition)
///   8  → Member  (a thuoc b)
///   9  → Subset  (a con b)
///   10 → Equiv   (a dong nhat b)
///   11 → Orthogonal (a vuong goc b)
///   12 → Compose (g∘f)
///   13 → Causes  (a → b, nhan qua)
///   14 → Similar (a ≈ b, d < epsilon)
///   15 → DerivedFrom (a ← b)
pub fn eval_relation(r: u8, a: &Molecule, b: &Molecule) -> RelationResult {
    match r {
        // === UDC_R groups 0..7 ===
        0 => {
            // R.0: Toan tu — cau truc dai so
            // Cong thuc: (G,*) closure + associativity + identity + inverse
            // Ring: (Z,+,x), Field: (R,+,x,/)
            // Heuristic: neu ca 2 molecule co S tuong tu → group, khac → ring
            let same_shape = a.s() == b.s();
            RelationResult::Algebraic {
                algebra_kind: if same_shape { 0 } else { 1 }, // group vs ring
                commutative: true, // default abelian
            }
        }
        1 => {
            // R.1: So sanh — partial/total order
            // Cong thuc: a <= b ⟺ phan xa + phan doi xung + bac cau
            // So sanh P_weight tong the
            let ord = if a.bits < b.bits { -1i8 }
                      else if a.bits > b.bits { 1i8 }
                      else { 0i8 };
            RelationResult::Order { ordering: ord, total: true }
        }
        2 => {
            // R.2: Chu toan — bieu dien (font functor)
            // Cong thuc: Bold = [alpha 0; 0 alpha], Italic = [1 k; 0 1]
            // Transform kind tu sub-index cua R byte goc
            RelationResult::Representation { transform_kind: 0 }
        }
        3 => {
            // R.3: So — ma hoa vi tri
            // Cong thuc: N = Sum(d_i * b^i)
            RelationResult::Numeral { base: 10, value: 0 }
        }
        4 => {
            // R.4: Dau cau — PDA stack ops
            // Cong thuc: delta(q, open) = PUSH, delta(q, close) = POP
            // Dyck language, Catalan number C_n
            RelationResult::Punctuation { stack_op: 2, depth_delta: 0 }
        }
        5 => {
            // R.5: Tien te — quy doi tuyen tinh
            // Cong thuc: v_target = R * v_source, det(R) = 1 (no-arbitrage)
            RelationResult::LinearMap { rate: 1.0 }
        }
        6 => {
            // R.6: Co dai — he cong tinh
            // Cong thuc: N = Sum_j val(symbol_j) (khong vi tri)
            RelationResult::Additive { value: 0 }
        }
        7 => {
            // R.7: Dieu khien — DFA transition
            // Cong thuc: delta(q, sigma) → q' (chuyen trang thai)
            RelationResult::Automaton { transition: 0 }
        }
        // === Category morphisms 8..15 (tu RelationBase enum hien tai) ===
        8 => RelationResult::CategoryMorphism {
            kind: 0x01, // Member: a thuoc b
            invertible: false,
        },
        9 => RelationResult::CategoryMorphism {
            kind: 0x02, // Subset: a con b
            invertible: false,
        },
        10 => RelationResult::CategoryMorphism {
            kind: 0x03, // Equiv: a dong nhat b
            invertible: true,  // equivalence = isomorphism
        },
        11 => RelationResult::CategoryMorphism {
            kind: 0x04, // Orthogonal: a vuong goc b (a·b = 0)
            invertible: false,
        },
        12 => RelationResult::CategoryMorphism {
            kind: 0x05, // Compose: g∘f
            invertible: false,
        },
        13 => RelationResult::CategoryMorphism {
            kind: 0x06, // Causes: a → b (nhan qua)
            invertible: false,
        },
        14 => RelationResult::CategoryMorphism {
            kind: 0x07, // Similar: a ≈ b, d(a,b) < epsilon
            invertible: true,
        },
        15 => RelationResult::CategoryMorphism {
            kind: 0x08, // DerivedFrom: a ← b
            invertible: false,
        },
        _ => RelationResult::CategoryMorphism {
            kind: 0, invertible: false,
        },
    }
}

/// Compose 2 Molecule theo R rules.
///
/// Day la phep hop morphism trong Category Theory:
///   g∘f: A → C  khi f: A → B, g: B → C
///
/// Ket qua: Molecule moi voi P_weight duoc tinh tu R dispatch.
pub fn compose_by_relation(r: u8, a: &Molecule, b: &Molecule) -> Molecule {
    let result = eval_relation(r, a, b);
    match result {
        RelationResult::Algebraic { algebra_kind, .. } => {
            match algebra_kind {
                0 => {
                    // Group: compose = cong tung chieu (mod range)
                    Molecule::pack(
                        a.s().wrapping_add(b.s()),
                        a.r(),
                        a.v().wrapping_add(b.v()),
                        a.a().wrapping_add(b.a()),
                        a.t(),
                    )
                }
                _ => {
                    // Ring: compose = nhan (cross product cac chieu)
                    Molecule::pack(
                        a.s() ^ b.s(),  // XOR cho shape
                        a.r(),
                        (a.v() + b.v()) / 2,
                        (a.a() + b.a()) / 2,
                        a.t() | b.t(),
                    )
                }
            }
        }
        RelationResult::Order { ordering, .. } => {
            // Order: tra ve molecule "lon hon"
            if ordering >= 0 { *a } else { *b }
        }
        _ => {
            // Default: tra ve a (identity morphism)
            *a
        }
    }
}
```

## FE.1.2 Vi tri trong codebase

```
crates/olang/src/mol/
  mod.rs            ← them: pub mod relation_eval;
  relation_eval.rs  ← FILE MOI (code o tren)
  molecular.rs      ← KHONG SUA (chi import)
```

## FE.1.3 Ket noi voi pipeline

```
Pipeline hien tai:
  text → encoder::encode_codepoint(cp) → Molecule { bits: u16 }
                                             ↓
  LCA compose: lca(a, b) → Molecule moi (chi dung shape CSG hien tai)
                                             ↓
  KnowTree store

SAU FE.1:
  text → encoder::encode_codepoint(cp) → Molecule { bits: u16 }
                                             ↓
  LCA compose: lca(a, b) dung compose_by_relation(a.r(), a, b)
                  |
                  ├── R=0 → Group compose (cong tung chieu)
                  ├── R=1 → Order compare (tra ve max)
                  ├── R=12 → Morphism compose (g∘f)
                  └── ...
                                             ↓
  KnowTree store + RelationResult metadata
```

## FE.1.4 Vi du cu the

```
Input:  "+" (PLUS SIGN, U+002B)
  encode → Molecule { S=???, R=0, V=4, A=4, T=0 }
  R=0 → eval_relation(0, a, b) → Algebraic { algebra_kind: 1(ring), commutative: true }
  → HE BIET: day la phep toan, ring (Z,+,x), giao hoan
  → compose_by_relation: a + b = cong tung chieu

Input:  "⊂" (SUBSET OF, U+2282)
  encode → Molecule { S=???, R=9, V=4, A=4, T=0 }
  R=9 → eval_relation(9, a, b) → CategoryMorphism { kind: 0x02(Subset), invertible: false }
  → HE BIET: day la quan he bao ham, khong invertible
  → compose: a.dims ⊂ b.dims (kiem tra tung chieu)

Input:  "$" (DOLLAR SIGN, U+0024)
  encode → Molecule { S=???, R=5, V=???, A=???, T=0 }
  R=5 → eval_relation(5, a, b) → LinearMap { rate: 1.0 }
  → HE BIET: day la quy doi tuyen tinh, v_target = R * v_source
```

---

# =====================================================================
# FE.2 — V DISPATCH: 8 Valence Levels → Potential Energy Physics
# =====================================================================

## FE.2.1 Rust Structs

**File:** `crates/olang/src/mol/valence_eval.rs` (TAO MOI)
**Dang ky:** them `pub mod valence_eval;` trong `crates/olang/src/mol/mod.rs`

```rust
//! # valence_eval — Formula dispatch cho chieu V
//!
//! Doc V index (0-7) → biet nhom the nang → biet luc hut/day.
//! Moi nhom = 1 dang potential energy tu UDC_V_VALENCE_tree.md

/// Trang thai the nang (potential energy state).
///
/// Moi variant = 1 cong thuc vat ly cu the.
/// Force = -dU/dx: luc tac dung len he.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValenceState {
    /// Gia tri the nang U (dimensionless, [-1.0, +1.0])
    /// Am = gieng (hut), duong = rao (day), 0 = phang
    pub potential: f32,

    /// Luc: F = -dU/dx
    /// Duong = hut vao (approach), am = day ra (avoid)
    pub force: f32,

    /// Do sau/cao cua gieng/rao
    /// Quyet dinh "can bao nhieu nang luong de thoat/vuot"
    pub barrier: f32,

    /// Kieu potential landscape
    pub kind: ValenceKind,
}

/// Kieu canh quan the nang — tung loai co cong thuc rieng.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValenceKind {
    /// V=0: Rao the rat cao (U >> 0) — REPELLER
    /// Cong thuc: U(r) = +k·q1·q2/r (Coulomb day, cung dau)
    /// Luc: F = +k·q1·q2/r² > 0 (day)
    /// Nhom tu: hate, horror, death, torture, destroy
    HighBarrier,

    /// V=1: Rao the thap (U > 0) — LOW BARRIER
    /// Cong thuc: U(x) = U0·exp(-x²/2sigma²) (rao Gauss, U0 nho)
    /// Tunneling: T ≈ exp(-2·kappa·d) > 0 (co the vuot)
    /// Nhom tu: annoying, difficult, embarrass, lack
    LowBarrier,

    /// V=2: Rao the rat thap / gan phang
    /// Cong thuc: U(x) = epsilon · exp(-x²/2sigma²), epsilon nho
    /// Nhom tu: adverse, ambiguous, doubt
    VeryLowBarrier,

    /// V=3: Mat phang the (U = const) — FLAT
    /// Cong thuc: U(x) = const → F(x) = -dU/dx = 0
    /// Can bang phiem dinh (neutral equilibrium)
    /// Nhom tu: agent, process, standard, normal
    FlatLow,

    /// V=4: Mat phang the (hoi duong) — FLAT
    /// Cong thuc: U(x) = const → F(x) = 0
    /// Nhom tu: material, merely, mixed, regular
    FlatHigh,

    /// V=5: Gieng the nong (U < 0) — SHALLOW WELL
    /// Cong thuc: U(r) = -epsilon·(sigma/r)^6 (Van der Waals, phan hut)
    /// Luc hut F = -dU/dr > 0 nhung nho
    /// Nhom tu: good, helpful, pleasant, benefit
    ShallowWell,

    /// V=6: Gieng the sau (U << 0) — DEEP WELL
    /// Cong thuc:
    ///   Niem vui: U = -V0 + ½kx² (gieng parabola, V0 >> kT)
    ///   Yeu thuong: U = -G·m1·m2/r (hap dan Newton)
    ///   Thanh cong: W = integral(F·ds) > 0 (cong nguoc gradient)
    ///   Dep: phi = (1+sqrt(5))/2, doi xung → U_min
    /// Nhom tu: joy, love, triumph, beautiful
    DeepWell,

    /// V=7: Gieng the rat sau (U <<< 0) — VERY DEEP WELL
    /// Cong thuc: U(x) = -V0·sech²(x/a) + V_barrier·exp(-(x-xb)²/2sigma²)
    /// Tu do + An toan: gieng rong (a lon) VA rao cao
    /// Nhom tu: free, safe, ecstatic, blissful
    VeryDeepWell,
}

/// Tinh chieu V dua tren UDC_V_VALENCE_tree.md
///
/// Input: v = 3 bits tu P_weight (0..7)
///
/// Anh xa vao canh quan the nang:
///   0 → HighBarrier    U >> 0   (rat tieu cuc, V ≤ -0.5)
///   1 → LowBarrier     U > 0    (tieu cuc, -0.5 < V < -0.2)
///   2 → VeryLowBarrier U > 0    (hoi tieu cuc)
///   3 → FlatLow        U ≈ 0    (trung tinh thap, -0.2 ≤ V ≤ 0)
///   4 → FlatHigh       U ≈ 0    (trung tinh cao, 0 < V ≤ +0.2)
///   5 → ShallowWell    U < 0    (tich cuc, +0.2 < V ≤ +0.5)
///   6 → DeepWell       U << 0   (rat tich cuc, V > +0.5)
///   7 → VeryDeepWell   U <<< 0  (cuc tich cuc, V → +1.0)
pub fn eval_valence(v: u8) -> ValenceState {
    match v {
        0 => ValenceState {
            potential: 0.85,    // U >> 0 (rao rat cao)
            force: -0.9,        // day manh (avoid)
            barrier: 0.95,
            kind: ValenceKind::HighBarrier,
            // Cong thuc: U(r) = +k·q1·q2/r, F = +k·q1·q2/r²
            // Ung dung: hate, horror → he bi DAY MANH
        },
        1 => ValenceState {
            potential: 0.4,     // U > 0 (rao thap)
            force: -0.4,        // day nhe
            barrier: 0.5,
            kind: ValenceKind::LowBarrier,
            // Cong thuc: U = U0·exp(-x²/2sigma²), T ≈ exp(-2kd) > 0
            // Ung dung: annoying, difficult → kho chiu nhung VUOT DUOC
        },
        2 => ValenceState {
            potential: 0.15,
            force: -0.15,
            barrier: 0.2,
            kind: ValenceKind::VeryLowBarrier,
        },
        3 => ValenceState {
            potential: 0.0,     // U = 0
            force: 0.0,         // F = 0 (phang)
            barrier: 0.0,
            kind: ValenceKind::FlatLow,
            // Cong thuc: U(x) = const → F = 0
            // Ung dung: neutral words → KHONG co xu huong
        },
        4 => ValenceState {
            potential: 0.0,
            force: 0.0,
            barrier: 0.0,
            kind: ValenceKind::FlatHigh,
        },
        5 => ValenceState {
            potential: -0.35,   // U < 0 (gieng nong)
            force: 0.35,        // hut nhe (approach)
            barrier: 0.4,
            kind: ValenceKind::ShallowWell,
            // Cong thuc: U(r) = -epsilon·(sigma/r)^6 (Van der Waals)
            // Ung dung: good, helpful → xu huong TIEP CAN nhe
        },
        6 => ValenceState {
            potential: -0.75,   // U << 0 (gieng sau)
            force: 0.8,         // hut manh
            barrier: 0.85,
            kind: ValenceKind::DeepWell,
            // Cong thuc: U = -V0 + ½kx² (gieng parabola)
            // Ung dung: joy, love → he bi GIU CHAT tai day gieng
        },
        7 => ValenceState {
            potential: -0.95,   // U <<< 0 (gieng rat sau)
            force: 0.95,        // hut cuc manh
            barrier: 0.98,
            kind: ValenceKind::VeryDeepWell,
            // Cong thuc: U = -V0·sech²(x/a) + V_barrier
            // Ung dung: ecstatic, blissful → DINH CAO tich cuc
        },
        _ => ValenceState {
            potential: 0.0, force: 0.0, barrier: 0.0,
            kind: ValenceKind::FlatLow,
        },
    }
}

/// Tinh luc giua 2 Molecule dua tren Valence.
///
/// Dung cho Silk: khi 2 node co-activate,
/// luc nay quyet dinh chung TIEP CAN hay TRANH XA.
///
/// F > 0: approach (tiep can)
/// F < 0: avoid (tranh xa)
/// F = 0: neutral (khong co xu huong)
pub fn valence_force(v_a: u8, v_b: u8) -> f32 {
    let state_a = eval_valence(v_a);
    let state_b = eval_valence(v_b);
    // Luc hop = trung binh luc (khong phai trung binh V!)
    // Amplify: 2 cai tich cuc → luc hut MANH HON 2x
    let f = state_a.force + state_b.force;
    if f > 0.0 { f * 1.2 } else { f * 1.2 } // amplify, khong trung binh
}
```

## FE.2.2 Anh xa chi tiet: V value → Cong thuc UDC

```
V=0 (3 bits: 000) → HighBarrier → RAT TIEU CUC
  Nhom UDC:
    Ghet/Gian:  U(r) = +k·q1·q2/r  (Coulomb day)
    Buon/Dau:   U(r) → -∞ tai r_s   (sup do hap dan, lo den)
    So hai:     T = exp(-2·kappa·d) → 0  (khong thoat duoc)
    Xau/Hai:    N(t) = N0·exp(-lambda·t) (phan ra phong xa)
    Benh/Kho:   dS/dt > 0, F → 0  (suy thoai entropy)

V=1 (001) → LowBarrier → TIEU CUC
  Nhom UDC:
    Tieu cuc vua: U = U0·exp(-x²/2sigma²) (rao Gauss thap)
    Tunneling probability cao → van de CO THE giai quyet

V=2 (010) → VeryLowBarrier → HOI TIEU CUC
  (tuong tu V=1 nhung yeu hon)

V=3 (011) → FlatLow → TRUNG TINH (thap)
  Cong thuc: U(x) = const → F = 0
  Cac tu mo ta, khong mang gia tri

V=4 (100) → FlatHigh → TRUNG TINH (cao)
  Cong thuc: U(x) = const → F = 0
  Cac tu trung tinh, hoi duong

V=5 (101) → ShallowWell → TICH CUC
  Cong thuc: U(r) = -epsilon·(sigma/r)^6 (Van der Waals)
  Lien ket yeu nhung CO huong tich cuc

V=6 (110) → DeepWell → RAT TICH CUC
  Cong thuc:
    Niem vui: U = -V0 + ½kx²
    Yeu thuong: U = -G·m1·m2/r
    Thanh cong: W = integral(F·ds) > 0
    Dep/Tot: phi = (1+sqrt(5))/2

V=7 (111) → VeryDeepWell → CUC TICH CUC
  Cong thuc: U = -V0·sech²(x/a) + V_barrier
  Tu do + An toan: gieng rong VA rao cao
```

## FE.2.3 Vi du cu the

```
Input: "love" → encode → Molecule { V=6 }
  eval_valence(6) → ValenceState {
    potential: -0.75,   // gieng sau
    force: +0.8,        // hut manh → APPROACH
    barrier: 0.85,      // can nhieu nang luong de thoat
    kind: DeepWell,     // U = -G·m1·m2/r
  }
  → HE BIET: "love" = lien ket hap dan, cang gan → cang kho tach
  → Behavior: approach, maintain proximity

Input: "hate" → encode → Molecule { V=0 }
  eval_valence(0) → ValenceState {
    potential: +0.85,   // rao cao
    force: -0.9,        // day manh → AVOID
    barrier: 0.95,      // luc day cuc manh
    kind: HighBarrier,  // U = +k·q1·q2/r (Coulomb)
  }
  → HE BIET: "hate" = luc day, 2 thuc the KHONG tuong thich
  → Behavior: avoid, increase distance

Input: "process" → encode → Molecule { V=3 hoac V=4 }
  eval_valence(3) → ValenceState {
    potential: 0.0,
    force: 0.0,
    barrier: 0.0,
    kind: FlatLow,      // U = const, F = 0
  }
  → HE BIET: "process" = trung tinh, khong co xu huong
  → Behavior: neutral, no preference
```

---

# =====================================================================
# FE.3 — A DISPATCH: 8 Arousal Levels → Damped Oscillator Physics
# =====================================================================

## FE.3.1 Rust Structs

**File:** `crates/olang/src/mol/arousal_eval.rs` (TAO MOI)
**Dang ky:** them `pub mod arousal_eval;` trong `crates/olang/src/mol/mod.rs`

```rust
//! # arousal_eval — Formula dispatch cho chieu A
//!
//! Doc A index (0-7) → biet che do nang luong → biet phan ung.
//! Moi nhom = 1 che do cua damped harmonic oscillator
//! Tu UDC_A_AROUSAL_tree.md

/// Trang thai nang luong (energy regime).
///
/// Mo hinh: x'' + 2·gamma·x' + omega0²·x = F(t)/m
/// Arousal A = tanh(E/E_th) ∈ [-1, +1]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArousalState {
    /// Nang luong hien tai (dimensionless, [0.0, 1.0])
    pub energy: f32,

    /// He so tat dan (damping coefficient)
    /// gamma > omega0 → overdamped (yen tinh)
    /// gamma < omega0 → underdamped (dao dong)
    /// gamma = 0 → khong tat dan (supercritical)
    pub gamma: f32,

    /// Tan so rieng (natural frequency)
    /// omega0 = sqrt(k/m)
    pub omega0: f32,

    /// He so khuech dai (feedback/cascade rate)
    /// lambda > 0 → positive feedback (bung no)
    /// lambda = 0 → stable
    /// lambda < 0 → negative feedback (tat dan them)
    pub lambda: f32,

    /// Kieu che do nang luong
    pub kind: ArousalKind,
}

/// Kieu che do nang luong — tung loai co cong thuc rieng.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArousalKind {
    /// A=0: Trang thai co ban (ground state)
    /// Cong thuc: E0 = ½·hbar·omega0 (zero-point energy)
    /// Bat dong, khong du nang luong de chuyen trang thai
    /// Nhom tu: asleep, frozen, numb, paralyzed
    GroundState,

    /// A=1: Buon chan / Met moi (heat death)
    /// Cong thuc: S → S_max, eta = W/Q → 0 (Carnot efficiency → 0)
    /// Nang luong co nhung KHONG THE chuyen hoa
    /// Nhom tu: bored, exhausted, lethargic
    HeatDeath,

    /// A=2: Tri tue tinh (pure information, no work)
    /// Cong thuc: H = -Sum(pi·log2(pi)), nhung W = 0, F = 0
    /// Thong tin cao, nang luong vat ly = 0
    /// Nhom tu: abstract, theoretical, philosophical
    PureInfo,

    /// A=3: Yen tinh (overdamped regime)
    /// Cong thuc: x(t) = (C1 + C2·t)·exp(-gamma·t), gamma > omega0
    /// Khong dao dong, tro ve can bang don dieu
    /// Nhom tu: calm, gentle, quiet, slow, patient
    Overdamped,

    /// A=4: Can bang nhiet dong (thermal equilibrium)
    /// Cong thuc: Delta_G = Delta_H - T·Delta_S = 0
    /// P(E) = exp(-E/kT) / Z (phan bo Boltzmann)
    /// Nhom tu: normal, standard, process
    Equilibrium,

    /// A=5: Kich thich cao (excited state)
    /// Cong thuc: E_n = E0 + n·Delta_E
    /// x(t) = A0·exp(-gamma·t)·cos(omega_d·t), gamma nho
    /// Nhom tu: active, bold, brave, eager, excited
    ExcitedState,

    /// A=6: Cong huong (resonance)
    /// Cong thuc: |X(omega)| = F0 / sqrt((omega0²-omega²)² + 4·gamma²·omega²)
    /// Tai cong huong: |X_max| = F0 / (2·gamma·omega0), gamma → 0 → ∞
    /// Nhom tu: ecstatic, furious, panicked, screaming
    Resonance,

    /// A=7: Sieu toi han (supercritical regime)
    /// Cong thuc: R(t) = H(t-t0)·R0·exp(lambda·t) (chain reaction)
    /// E >> E_th, positive feedback, bung no
    /// Nhom tu: explode, attack, alarm, crisis, earthquake
    Supercritical,
}

/// Tinh chieu A dua tren UDC_A_AROUSAL_tree.md
///
/// Input: a = 3 bits tu P_weight (0..7)
///
/// Anh xa vao pho nang luong:
///   0 → GroundState   E = E0          (cuc yen tinh, A ≤ -0.5)
///   1 → HeatDeath     S → S_max       (cuc yen tinh — met moi)
///   2 → PureInfo      H > 0, W = 0    (cuc yen tinh — suy nghi)
///   3 → Overdamped    gamma > omega0   (yen tinh, -0.5 < A < -0.2)
///   4 → Equilibrium   Delta_G = 0     (trung tinh, -0.2 ≤ A ≤ +0.2)
///   5 → ExcitedState  gamma < omega0   (kich thich cao, +0.2 < A ≤ +0.5)
///   6 → Resonance     omega = omega0   (cuc kich thich — cam xuc)
///   7 → Supercritical E >> E_th        (cuc kich thich — bung no)
pub fn eval_arousal(a: u8) -> ArousalState {
    match a {
        0 => ArousalState {
            energy: 0.02,       // E = E0 (zero-point, gan 0)
            gamma: 100.0,       // tat dan cuc manh
            omega0: 1.0,
            lambda: 0.0,
            kind: ArousalKind::GroundState,
            // x = 0, dx/dt = 0: he "dong bang" tai ground state
        },
        1 => ArousalState {
            energy: 0.05,
            gamma: 50.0,        // tat dan rat manh
            omega0: 1.0,
            lambda: 0.0,
            kind: ArousalKind::HeatDeath,
            // S → S_max: entropy cuc dai, khong con gradient
        },
        2 => ArousalState {
            energy: 0.08,
            gamma: 30.0,
            omega0: 1.0,
            lambda: 0.0,
            kind: ArousalKind::PureInfo,
            // H = -Sum(p·log(p)) > 0, nhung W = 0
        },
        3 => ArousalState {
            energy: 0.2,
            gamma: 3.0,         // gamma > omega0 → overdamped
            omega0: 1.0,
            lambda: 0.0,
            kind: ArousalKind::Overdamped,
            // x(t) = (C1+C2·t)·exp(-gamma·t): tro ve can bang, KHONG dao dong
        },
        4 => ArousalState {
            energy: 0.5,        // E ≈ E_th
            gamma: 1.0,         // gamma = omega0 → critically damped
            omega0: 1.0,
            lambda: 0.0,
            kind: ArousalKind::Equilibrium,
            // Delta_G = 0: can bang nhiet dong
        },
        5 => ArousalState {
            energy: 0.7,
            gamma: 0.3,         // gamma < omega0 → underdamped, dao dong
            omega0: 1.0,
            lambda: 0.0,
            kind: ArousalKind::ExcitedState,
            // x(t) = A0·exp(-gamma·t)·cos(omega_d·t), gamma nho
        },
        6 => ArousalState {
            energy: 0.9,
            gamma: 0.05,        // gamma rat nho → cong huong manh
            omega0: 1.0,
            lambda: 0.0,
            kind: ArousalKind::Resonance,
            // |X_max| = F0/(2·gamma·omega0) → rat lon khi gamma → 0
        },
        7 => ArousalState {
            energy: 0.98,
            gamma: 0.0,         // khong tat dan
            omega0: 1.0,
            lambda: 2.0,        // positive feedback → bung no
            kind: ArousalKind::Supercritical,
            // R(t) = H(t-t0)·R0·exp(lambda·t): phan ung day chuyen
        },
        _ => ArousalState {
            energy: 0.5, gamma: 1.0, omega0: 1.0, lambda: 0.0,
            kind: ArousalKind::Equilibrium,
        },
    }
}

/// Tinh urgency (khan cap) tu ArousalState.
///
/// urgency > 0.8 → trigger SecurityGate (crisis check)
/// urgency > 0.6 → prioritize response
/// urgency < 0.3 → co the delay
pub fn arousal_urgency(a: u8) -> f32 {
    let state = eval_arousal(a);
    match state.kind {
        ArousalKind::Supercritical => 0.95 + state.lambda * 0.025,
        ArousalKind::Resonance     => 0.85,
        ArousalKind::ExcitedState  => 0.6,
        ArousalKind::Equilibrium   => 0.4,
        ArousalKind::Overdamped    => 0.2,
        ArousalKind::PureInfo      => 0.1,
        ArousalKind::HeatDeath     => 0.05,
        ArousalKind::GroundState   => 0.02,
    }
}

/// Mo phong oscillator tai thoi diem t.
///
/// Tra ve bien do x(t) dua tren ArousalState.
/// Dung de animate/render response timing.
pub fn oscillator_at(state: &ArousalState, t: f32) -> f32 {
    let g = state.gamma;
    let w = state.omega0;
    let lam = state.lambda;

    if lam > 0.0 {
        // Supercritical: R(t) = exp(lambda·t) (bung no)
        (lam * t).exp().min(100.0) // clamp de tranh overflow
    } else if g > w {
        // Overdamped: x(t) = exp(-gamma·t) (tat dan don dieu)
        (-g * t).exp()
    } else if g > 0.01 {
        // Underdamped: x(t) = exp(-gamma·t)·cos(omega_d·t)
        let omega_d = (w * w - g * g).sqrt();
        (-g * t).exp() * (omega_d * t).cos()
    } else {
        // Resonance / no damping: x(t) = cos(omega0·t)
        (w * t).cos()
    }
}
```

## FE.3.2 Anh xa chi tiet: A value → Cong thuc UDC

```
A=0 (000) → GroundState
  Cong thuc: E0 = ½·hbar·omega0
  He tai muc nang luong thap nhat, KHONG du Delta_E de chuyen
  Tu: asleep, dormant, frozen, numb, paralyzed, stagnant

A=1 (001) → HeatDeath
  Cong thuc: S → S_max, dS/dt = 0, eta → 0
  Nang luong CO nhung khong chuyen hoa duoc
  Tu: bored, exhausted, fatigued, lethargic, apathetic

A=2 (010) → PureInfo
  Cong thuc: H = -Sum(p·log2(p)) > 0, nhung W = 0
  Thong tin nhieu, hanh dong khong
  Tu: abstract, academic, theoretical, philosophical

A=3 (011) → Overdamped
  Cong thuc: x(t) = (C1+C2·t)·exp(-gamma·t), gamma > omega0
  Khong dao dong, tro ve can bang tu tu
  Tu: calm, gentle, quiet, slow, patient, tranquil

A=4 (100) → Equilibrium
  Cong thuc: Delta_G = 0, P(E) = exp(-E/kT)/Z
  Can bang, khong thien lech
  Tu: normal, standard, process, cycle

A=5 (101) → ExcitedState
  Cong thuc: E_n = E0 + n·Delta_E, x(t) = A0·exp(-gamma·t)·cos(omega_d·t)
  Dao dong ro rang, gamma nho (tat cham)
  Tu: active, bold, brave, eager, excited, lively

A=6 (110) → Resonance
  Cong thuc: |X(omega)| = F0 / sqrt((omega0²-omega²)² + 4gamma²omega²)
  Bien do cuc dai khi omega = omega0
  Tu: ecstatic, furious, panicked, terrified, screaming

A=7 (111) → Supercritical
  Cong thuc: R(t) = H(t-t0)·R0·exp(lambda·t)
  Phan ung day chuyen, positive feedback
  Tu: explode, attack, alarm, crisis, earthquake, tsunami
```

## FE.3.3 Vi du cu the

```
Input: "earthquake" → encode → Molecule { A=7 }
  eval_arousal(7) → ArousalState {
    energy: 0.98,
    gamma: 0.0,
    lambda: 2.0,
    kind: Supercritical,
  }
  arousal_urgency(7) → 0.95 + 0.05 = 1.0
  → HE BIET: KHAN CAP → SecurityGate → Crisis check NGAY
  → oscillator_at(state, t=0.5) → exp(2.0*0.5) = exp(1.0) ≈ 2.718
  → Bien do TANG NHANH → phan hoi URGENT

Input: "calm" → encode → Molecule { A=3 }
  eval_arousal(3) → ArousalState {
    energy: 0.2,
    gamma: 3.0,
    kind: Overdamped,
  }
  arousal_urgency(3) → 0.2
  → HE BIET: yen tinh, co the delay
  → oscillator_at(state, t=1.0) → exp(-3.0) ≈ 0.05
  → Bien do TAT NHANH → phan hoi nhe nhang, khong voi
```

---

# =====================================================================
# FE.4 — T SPLINE KNOT: Moi observation = 1 diem tren duong cong
# =====================================================================

## FE.4.1 Rust Structs

**File:** `crates/vsdf/src/dynamics/time_knot.rs` (TAO MOI)
**Dang ky:** them `pub mod time_knot;` trong `crates/vsdf/src/dynamics/mod.rs`

Ly do dat trong vsdf: VectorSpline DA CO san trong vsdf/dynamics/spline.rs.
SplineKnot la phan mo rong, dung chung VectorSpline lam backend.

```rust
//! # time_knot — T dimension: moi observation = 1 SplineKnot
//!
//! T khong phai label tinh — T la DIEM tren duong cong thoi gian.
//! Moi lan he observe 1 concept → tao 1 SplineKnot → append vao history.
//! History = chuoi knots = duong cong hoc tap (learning curve).
//!
//! Tu UDC_T_TIME_tree.md:
//!   T.0: Que Dich (FSM 64 states, phase transitions)
//!   T.1: Tu Quai (ternary encoding, 81 states)
//!   T.2: Byzantine (melody contour, tempo scaling)
//!   T.3: Znamenny (differential operators on pitch)
//!   T.4: Nhac Tay (Fourier analysis: f, A, phi, w(t))
//!   T.5: Hy Lap co (mora units, ratio rhythms 3:4:5)

extern crate alloc;
use alloc::vec::Vec;

/// 1 SplineKnot = 1 observation tren duong cong T.
///
/// Moi knot ghi lai "diem nao, luc nao, cuong do nao, pha nao".
/// Append-only: chi them, khong xoa, giong DNA.
///
/// Size: 24 bytes = 4 (timestamp) + 4 (amplitude) + 4 (frequency)
///       + 4 (phase) + 4 (duration) + 2 (kind) + 2 (padding)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplineKnot {
    /// Thoi diem observation (unix timestamp truncated, u32)
    pub timestamp: u32,

    /// Bien do (amplitude/intensity) cua observation
    /// Text input: 1.0 (binh thuong), sensor: do do cuong do
    /// VD: anh sang 1000 lux → amp = 1000.0
    pub amplitude: f32,

    /// Tan so (frequency) — tu cong thuc T
    /// T=0 (timeless): freq = 0
    /// T=1 (sequential): freq = 0 (khong lap)
    /// T=2 (cyclical): freq = 1/period
    /// T=3 (rhythmic): freq = f cua psi(t) = A·sin(2pi·f·t + phi)
    pub frequency: f32,

    /// Pha ban dau (phase)
    /// Tu UDC_T:
    ///   Que Dich: pha trong chu ky bien doi (0, pi/3, 2pi/3, pi, 4pi/3, 5pi/3)
    ///   Nhac: pha cua ham song
    pub phase: f32,

    /// Do dai observation (milliseconds)
    /// Text input: thoi gian doc/xu ly
    /// Sensor: thoi gian do
    pub duration_ms: u32,

    /// Kieu thoi gian (2 bits tu P_weight, expanded)
    pub kind: TimeKnotKind,
}

/// Kieu SplineKnot — map tu T value (0..3)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeKnotKind {
    /// T=0: Timeless / Static — khong co thoi gian
    /// Que Dich: khong thuoc chu ky nao
    /// Nhac: whole note, fermata (keo dai vo han)
    /// Cong thuc: psi(t) = const (ham hang so)
    Timeless,

    /// T=1: Sequential — co thu tu, khong lap
    /// Que Dich: chuyen trang thai tren hypercube Q6
    /// Nhac: sequence of notes (melody line)
    /// Cong thuc: m(t) = Sum(Delta_p_i · H(t - t_i)) (ham buoc tich luy)
    Sequential,

    /// T=2: Cyclical — co chu ky, lap lai
    /// Que Dich: chu ky 64 states
    /// Nhac Byzantine: tempo scaling t' = alpha·t
    /// Cong thuc: psi(t) = A·sin(2pi·t/T) (chu ky T)
    Cyclical,

    /// T=3: Rhythmic — co nhip, tan so, pha
    /// Nhac Tay: fn(t) = An·sin(2pi·fn·t + phi_n)·w(t) (Fourier)
    /// Znamenny: dp/dt (dao ham cao do)
    /// Cong thuc: psi(t) = A·sin(2pi·f·t + phi)
    Rhythmic,
}

/// Tao SplineKnot tu T value + observation data.
///
/// Day la "eval_time" — tuong tu eval_valence/eval_arousal
/// nhung tra ve KNOT (diem tren duong cong) thay vi STATE (trang thai).
///
/// Input:
///   t_val: 2 bits tu P_weight (0..3)
///   timestamp: thoi diem hien tai (unix epoch truncated to u32)
///   duration_ms: thoi gian observation (ms)
///   sensor_freq: tan so tu sensor (0.0 neu text input)
///   sensor_amp: cuong do tu sensor (1.0 neu text input)
///   sensor_phase: pha tu sensor (0.0 default)
pub fn create_knot(
    t_val: u8,
    timestamp: u32,
    duration_ms: u32,
    sensor_freq: f32,
    sensor_amp: f32,
    sensor_phase: f32,
) -> SplineKnot {
    let kind = match t_val {
        0 => TimeKnotKind::Timeless,
        1 => TimeKnotKind::Sequential,
        2 => TimeKnotKind::Cyclical,
        3 => TimeKnotKind::Rhythmic,
        _ => TimeKnotKind::Timeless,
    };

    match kind {
        TimeKnotKind::Timeless => SplineKnot {
            timestamp,
            amplitude: sensor_amp,
            frequency: 0.0,         // khong co tan so
            phase: 0.0,             // khong co pha
            duration_ms,
            kind,
        },
        TimeKnotKind::Sequential => SplineKnot {
            timestamp,
            amplitude: sensor_amp,
            frequency: 0.0,         // sequential khong lap
            phase: 0.0,
            duration_ms,
            kind,
        },
        TimeKnotKind::Cyclical => {
            // Cyclical: freq tu sensor hoac uoc luong tu duration
            let freq = if sensor_freq > 0.0 {
                sensor_freq
            } else if duration_ms > 0 {
                1000.0 / duration_ms as f32  // 1/period
            } else {
                0.0
            };
            SplineKnot {
                timestamp,
                amplitude: sensor_amp,
                frequency: freq,
                phase: sensor_phase,
                duration_ms,
                kind,
            }
        }
        TimeKnotKind::Rhythmic => {
            // Rhythmic: day du f, A, phi tu Fourier
            // psi(t) = A · sin(2pi·f·t + phi)
            SplineKnot {
                timestamp,
                amplitude: sensor_amp,
                frequency: sensor_freq,
                phase: sensor_phase,
                duration_ms,
                kind,
            }
        }
    }
}

/// T History cho 1 concept — chuoi SplineKnot append-only.
///
/// Giong DNA: chi them, khong xoa. Moi observation = 1 nucleotide.
/// Doc history → biet "concept nay da duoc observe bao nhieu lan,
/// cuong do thay doi the nao, co pattern gi khong".
#[derive(Debug, Clone)]
pub struct TimeHistory {
    /// Chain hash cua concept nay (dinh danh)
    pub chain_hash: u64,

    /// Chuoi knots, append-only, thu tu thoi gian
    pub knots: Vec<SplineKnot>,
}

impl TimeHistory {
    /// Tao history moi cho 1 concept.
    pub fn new(chain_hash: u64) -> Self {
        Self {
            chain_hash,
            knots: Vec::new(),
        }
    }

    /// Append 1 knot — chi co phep ADD, khong DELETE.
    pub fn append(&mut self, knot: SplineKnot) {
        self.knots.push(knot);
    }

    /// So lan observe (= so knots).
    pub fn observation_count(&self) -> usize {
        self.knots.len()
    }

    /// Knot moi nhat (observation gan nhat).
    pub fn latest(&self) -> Option<&SplineKnot> {
        self.knots.last()
    }

    /// Amplitude trung binh (do "quen thuoc" cua concept).
    pub fn mean_amplitude(&self) -> f32 {
        if self.knots.is_empty() { return 0.0; }
        let sum: f32 = self.knots.iter().map(|k| k.amplitude).sum();
        sum / self.knots.len() as f32
    }

    /// Amplitude tang hay giam theo thoi gian?
    /// > 0: dang hoc (amplitude tang) — "cang gap cang hieu"
    /// < 0: dang quen (amplitude giam)
    /// = 0: on dinh
    pub fn amplitude_trend(&self) -> f32 {
        if self.knots.len() < 2 { return 0.0; }
        let n = self.knots.len();
        let first_half: f32 = self.knots[..n/2].iter().map(|k| k.amplitude).sum::<f32>()
            / (n / 2) as f32;
        let second_half: f32 = self.knots[n/2..].iter().map(|k| k.amplitude).sum::<f32>()
            / (n - n / 2) as f32;
        second_half - first_half
    }

    /// Kiem tra co pattern cyclical khong (frequency > 0 trong nhieu knots).
    pub fn has_cyclic_pattern(&self) -> bool {
        let cyclic_count = self.knots.iter()
            .filter(|k| k.kind == TimeKnotKind::Cyclical || k.kind == TimeKnotKind::Rhythmic)
            .filter(|k| k.frequency > 0.0)
            .count();
        cyclic_count as f32 / self.knots.len().max(1) as f32 > 0.5
    }
}
```

## FE.4.2 Anh xa chi tiet: T value → Cong thuc UDC

```
T=0 (2 bits: 00) → Timeless
  UDC groups: phan cua T.0 Que Dich (timeless states)
  Cong thuc: psi(t) = const
  Vi du: concept "truth" — khong thay doi theo thoi gian
  Knot: { freq: 0, phase: 0, amp: 1.0 }

T=1 (01) → Sequential
  UDC groups: T.0 Que Dich (state transitions), T.1 Tu Quai
  Cong thuc: m(t) = Sum(Delta_p · H(t - t_i)) (tong tich luy cac buoc)
  Vi du: "A roi B roi C" — co thu tu, khong lap
  Knot: { freq: 0, phase: 0, amp: cuong_do_buoc }

T=2 (10) → Cyclical
  UDC groups: T.2 Byzantine (tempo), T.0 Que Dich (cycles)
  Cong thuc: psi(t) = A·sin(2pi·t/T)
  Vi du: "mua" → lap lai moi nam (period = 365 ngay)
  Knot: { freq: 1/365, phase: pha_hien_tai, amp: cuong_do }

T=3 (11) → Rhythmic
  UDC groups: T.4 Nhac Tay (Fourier), T.3 Znamenny (vi phan)
  Cong thuc: fn(t) = An·sin(2pi·fn·t + phi_n)·w(t)
  Vi du: sensor anh sang 580nm → freq = c/580nm, amp = intensity
  Knot: { freq: 5.17e14, phase: 0, amp: 1000.0 }
```

## FE.4.3 Vi du cu the

```
=== TEXT INPUT ===

Observe "Hinh vuong" lan 1 (doc ten):
  t_val = 0 (Timeless — concept co dinh)
  create_knot(0, now, 200, 0.0, 1.0, 0.0)
  → SplineKnot { timestamp: now, amp: 1.0, freq: 0, phase: 0,
                  duration_ms: 200, kind: Timeless }

Observe "Hinh vuong" lan 2 (doc alias "Rectangle"):
  t_val = 0
  create_knot(0, now+60s, 150, 0.0, 0.8, 0.0)
  → SplineKnot { amp: 0.8, duration_ms: 150 }

Observe "Hinh vuong" lan 3 (doc dinh nghia "S = a²"):
  t_val = 0
  create_knot(0, now+120s, 500, 0.0, 1.5, 0.0)
  → SplineKnot { amp: 1.5, duration_ms: 500 }

History["Hinh vuong"] = [knot1, knot2, knot3]
  observation_count = 3
  mean_amplitude = (1.0 + 0.8 + 1.5) / 3 = 1.1
  amplitude_trend = 1.5 - 1.0 = +0.5 (DANG HOC — amplitude tang)

=== SENSOR INPUT ===

Sensor "light" lan 1: lambda=580nm, I=1000 lux
  t_val = 3 (Rhythmic — anh sang la song)
  freq = c / 580nm = 3e8 / 580e-9 = 5.17e14 Hz
  create_knot(3, now, 100, 5.17e14, 1000.0, 0.0)
  → SplineKnot { freq: 5.17e14, amp: 1000.0, kind: Rhythmic }
  → Silk → P{"anh sang vang"}

Sensor "light" lan 2: lambda=450nm, I=500 lux
  freq = 3e8 / 450e-9 = 6.67e14 Hz
  create_knot(3, now+5s, 100, 6.67e14, 500.0, PI/4)
  → SplineKnot { freq: 6.67e14, amp: 500.0, phase: pi/4, kind: Rhythmic }
  → Silk → P{"anh sang xanh"}

History["light"] = [knot1, knot2]
  amplitude_trend = 500 - 1000 = -500 (cuong do GIAM)
  freq thay doi 5.17e14 → 6.67e14 (buoc song NGAN hon)
  → HE BIET: "anh sang thay doi tu vang → xanh, cuong do giam"
```

---

# =====================================================================
# FE.5 — T SPLINE INTERPOLATION: History → Curve → Behavior Prediction
# =====================================================================

## FE.5.1 Rust Structs

**File:** `crates/vsdf/src/dynamics/time_spline.rs` (TAO MOI)
**Dang ky:** them `pub mod time_spline;` trong `crates/vsdf/src/dynamics/mod.rs`

```rust
//! # time_spline — Chuyen TimeHistory → VectorSpline → du doan behavior
//!
//! FE.4 tao knots. FE.5 noi chung thanh duong cong lien tuc.
//! Doc duong cong → biet: "concept nay dang hoc hay quen?
//! Co pattern lap khong? Cuong do tang hay giam?"

use super::spline::{BezierSegment, VectorSpline};
use super::time_knot::{SplineKnot, TimeHistory, TimeKnotKind};

/// Chuyen TimeHistory → VectorSpline (amplitude theo thoi gian).
///
/// Moi cap knots ke nhau → 1 BezierSegment.
/// Ket qua: spline lien tuc, evaluate tai t ∈ [0,1] → amplitude.
///
/// t=0 = observation dau tien
/// t=1 = observation moi nhat
/// t>1 = TUONG LAI (extrapolation → prediction)
pub fn history_to_amplitude_spline(history: &TimeHistory) -> VectorSpline {
    let knots = &history.knots;
    if knots.is_empty() {
        return VectorSpline::flat(0.0);
    }
    if knots.len() == 1 {
        return VectorSpline::flat(knots[0].amplitude);
    }

    let mut spline = VectorSpline::new();
    for i in 0..knots.len() - 1 {
        let a = knots[i].amplitude;
        let b = knots[i + 1].amplitude;
        // Bezier linear giua 2 knots
        spline.push(BezierSegment::linear(a, b));
    }
    spline
}

/// Chuyen TimeHistory → VectorSpline (frequency theo thoi gian).
///
/// Chi co y nghia cho Cyclical/Rhythmic knots.
pub fn history_to_frequency_spline(history: &TimeHistory) -> VectorSpline {
    let knots = &history.knots;
    if knots.is_empty() {
        return VectorSpline::flat(0.0);
    }
    if knots.len() == 1 {
        return VectorSpline::flat(knots[0].frequency);
    }

    let mut spline = VectorSpline::new();
    for i in 0..knots.len() - 1 {
        let a = knots[i].frequency;
        let b = knots[i + 1].frequency;
        spline.push(BezierSegment::linear(a, b));
    }
    spline
}

/// Du doan amplitude tai thoi diem tuong lai.
///
/// t_future: so buoc quan sat trong tuong lai (1 = lan toi, 2 = lan sau nua)
/// Extrapolation don gian: tiep tuc xu huong hien tai.
pub fn predict_amplitude(history: &TimeHistory, steps_ahead: u32) -> f32 {
    let spline = history_to_amplitude_spline(history);
    if spline.is_empty() { return 0.0; }

    // Extrapolate: t > 1.0
    let n = history.knots.len() as f32;
    let t_extra = 1.0 + (steps_ahead as f32 / n);

    // Lay gia tri va dao ham tai diem cuoi
    let last_val = spline.evaluate(1.0);
    let last_deriv = spline.derivative(1.0);

    // Linear extrapolation
    let predicted = last_val + last_deriv * (steps_ahead as f32 / n);
    predicted.max(0.0) // amplitude khong am
}

/// Behavior prediction tu TimeHistory.
///
/// Tra ve: (learning_rate, familiarity, periodicity)
///   learning_rate: > 0 dang hoc, < 0 dang quen, = 0 on dinh
///   familiarity: 0.0 (moi) → 1.0 (rat quen)
///   periodicity: 0.0 (khong lap) → 1.0 (rat co nhip)
pub fn predict_behavior(history: &TimeHistory) -> (f32, f32, f32) {
    let n = history.observation_count();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }

    // Learning rate = amplitude trend
    let learning_rate = history.amplitude_trend();

    // Familiarity = sigmoid(observation_count)
    // 1 observation = 0.2, 5 = 0.75, 10+ = 0.95
    let familiarity = 1.0 - 1.0 / (1.0 + n as f32 * 0.3);

    // Periodicity = ti le knots co frequency > 0
    let periodic_count = history.knots.iter()
        .filter(|k| k.frequency > 0.0)
        .count();
    let periodicity = periodic_count as f32 / n as f32;

    (learning_rate, familiarity, periodicity)
}
```

## FE.5.2 Storage Format: Cach luu SplineKnot trong origin.olang

```
origin.olang da co cac record types (0x01 Node, 0x02 Edge, ...).
Them record type moi:

0x0A TimeKnot  [chain_hash:8][timestamp:4][amplitude:4][frequency:4]
               [phase:4][duration_ms:4][kind:1][padding:3]
               = 32 bytes per knot

Ghi trong origin.olang:
  [0x0A][chain_hash:8 bytes][knot_data:24 bytes][ts:8 bytes] = 41 bytes total

Doc lai:
  1. Scan file, filter records 0x0A
  2. Group by chain_hash → TimeHistory per concept
  3. Sort by timestamp → ordered knots
  4. Build splines on-demand

APPEND-ONLY: chi ghi them, khong xoa, khong sua.
Giong DNA: moi observation = 1 "base pair" them vao cuoi chuoi.

Uoc tinh dung luong:
  1 concept × 100 observations × 41 bytes = 4.1 KB
  10,000 concepts × 100 obs = 41 MB
  → Hoan toan kha thi cho file append-only
```

## FE.5.3 Vi du cu the

```
TimeHistory cho "Hinh vuong" sau 5 observations:
  knots = [
    { amp: 1.0, dur: 200 },   // lan 1: doc ten
    { amp: 0.8, dur: 150 },   // lan 2: doc alias
    { amp: 1.5, dur: 500 },   // lan 3: doc dinh nghia (sau hon)
    { amp: 1.2, dur: 300 },   // lan 4: bai tap
    { amp: 1.8, dur: 400 },   // lan 5: van dung
  ]

amplitude_spline:
  Segment 0: 1.0 → 0.8 (giam nhe — quen alias)
  Segment 1: 0.8 → 1.5 (tang manh — hieu dinh nghia)
  Segment 2: 1.5 → 1.2 (giam nhe — onl dinh)
  Segment 3: 1.2 → 1.8 (tang — van dung tot)

predict_behavior:
  learning_rate = +0.4 (DANG HOC — amplitude tang)
  familiarity = 1.0 - 1.0/(1+5*0.3) = 1.0 - 1.0/2.5 = 0.6
  periodicity = 0.0 (Timeless knots, khong lap)

predict_amplitude(steps_ahead=3):
  last_val = 1.8
  last_deriv = (1.8 - 1.2) * 5 = 3.0 (tang nhanh)
  predicted = 1.8 + 3.0 * (3/5) = 1.8 + 1.8 = 3.6
  → HE DU DOAN: concept "Hinh vuong" se con tiep tuc duoc hoc sau

Spline visualization:
  amp
  2.0 |                                    *
  1.5 |           *               *
  1.0 | *                   *
  0.5 |      *
  0.0 |___________________________________
      t=0   t=1   t=2   t=3   t=4   t=future
```

---

# =====================================================================
# FE.6 — WIRE: Ket noi Formula Engine vao Pipeline
# =====================================================================

## FE.6.1 Rust Structs

**File:** `crates/olang/src/mol/formula_engine.rs` (TAO MOI)
**Dang ky:** them `pub mod formula_engine;` trong `crates/olang/src/mol/mod.rs`

```rust
//! # formula_engine — Trung tam dispatch cho tat ca 5 chieu
//!
//! Pipeline: encode → eval_formula → store result
//!
//! Doc P_weight → trich xuat S,R,V,A,T → goi eval tung chieu → tra ve FormulaState

use super::molecular::Molecule;
use super::relation_eval::{RelationResult, eval_relation};
use super::valence_eval::{ValenceState, eval_valence};
use super::arousal_eval::{ArousalState, eval_arousal};

/// Trang thai day du cua 1 Molecule sau khi evaluate tat ca cong thuc.
///
/// Day la "phenotype" — bieu hien thuc te cua "genotype" P_weight.
/// Giong ribosome doc DNA → tao protein:
///   P_weight (16 bits) → FormulaState (full physics)
#[derive(Debug, Clone)]
pub struct FormulaState {
    /// Molecule goc (P_weight 16 bits)
    pub molecule: Molecule,

    /// Ket qua evaluate chieu V → ValenceState
    pub valence: ValenceState,

    /// Ket qua evaluate chieu A → ArousalState
    pub arousal: ArousalState,

    /// T kind (dung de tao SplineKnot khi observe)
    pub time_kind: u8,

    /// Urgency (tu arousal) — dung cho SecurityGate
    pub urgency: f32,

    /// Force (tu valence) — dung cho Silk walk
    pub force: f32,
}

/// Evaluate FULL formula tu Molecule.
///
/// Day la HAM CHINH cua Formula Engine.
/// Tuong duong ribosome doc codon → tao amino acid.
///
/// Input: Molecule (16 bits P_weight)
/// Output: FormulaState (day du physics cho moi chieu)
pub fn evaluate(mol: &Molecule) -> FormulaState {
    let v_val = mol.v();
    let a_val = mol.a();
    let t_val = mol.t();

    let valence_state = eval_valence(v_val);
    let arousal_state = eval_arousal(a_val);
    let urgency = super::arousal_eval::arousal_urgency(a_val);

    FormulaState {
        molecule: *mol,
        valence: valence_state,
        arousal: arousal_state,
        time_kind: t_val,
        urgency,
        force: valence_state.force,
    }
}

/// Evaluate RELATION giua 2 Molecule.
///
/// Dung trong LCA compose va Silk walk.
pub fn evaluate_relation(a: &Molecule, b: &Molecule) -> RelationResult {
    eval_relation(a.r(), a, b)
}

/// Quick check: co can urgent response khong?
///
/// Dung trong pipeline truoc khi xu ly:
///   if needs_urgent(mol) { security_gate.check_crisis(); }
pub fn needs_urgent(mol: &Molecule) -> bool {
    super::arousal_eval::arousal_urgency(mol.a()) > 0.8
}

/// Quick check: approach hay avoid?
///
/// Dung trong Silk walk de quyet dinh huong di.
///   > 0: approach (tiep can node nay)
///   < 0: avoid (tranh node nay)
///   = 0: neutral
pub fn approach_tendency(mol: &Molecule) -> f32 {
    eval_valence(mol.v()).force
}
```

## FE.6.2 Noi vao pipeline hien tai

```
=== TRUOC (hien tai) ===

crates/runtime/src/lib.rs: HomeRuntime::process_text()
  text → T1:infer_context → T2:sentence_affect → T3:ctx.apply
       → T4:estimate_intent → T5:Crisis check → T6:learning → T7:render

T2:sentence_affect chi lam:
  words → lookup valence/arousal → average → EmotionState
  → Problem: AVERAGE xoa mat thong tin, khong dung physics

T5:Crisis check chi lam:
  if certain_keywords_found { flag_crisis() }
  → Problem: hardcode keywords, khong dung arousal physics

=== SAU FE.6 ===

T2:sentence_affect:
  words → encode_codepoint → Molecule → formula_engine::evaluate()
        → FormulaState { valence: physics, arousal: physics }
  KHONG average! Dung Silk walk voi amplify (cort + adrenaline = manh hon)

T5:Crisis check:
  if formula_engine::needs_urgent(&mol) { security_gate.check_crisis() }
  → Dung arousal physics: A=7 (Supercritical) → urgency=0.95 → TRIGGER

T6:learning:
  create_knot(mol.t(), now, duration, ...) → append vao TimeHistory
  → Moi observation = 1 diem tren duong cong
  → predict_behavior() → biet concept dang hoc hay quen

T7:render:
  FormulaState.arousal.kind → quyet dinh animation speed
    Supercritical → render ngay, flashing
    Overdamped → render cham, fade in muot
  FormulaState.valence.kind → quyet dinh mau sac
    DeepWell → mau am (xanh/tim)
    HighBarrier → mau nong (do/cam)
```

## FE.6.3 Files can sua

```
1. crates/olang/src/mol/mod.rs
   THEM:
     pub mod relation_eval;
     pub mod valence_eval;
     pub mod arousal_eval;
     pub mod formula_engine;

2. crates/vsdf/src/dynamics/mod.rs
   THEM:
     pub mod time_knot;
     pub mod time_spline;

3. crates/context/src/emotion.rs (NEU TON TAI)
   SUA: thay tinh toan valence/arousal static bang eval_valence/eval_arousal

4. crates/agents/src/gate.rs (hoac tuong duong)
   SUA: thay crisis keyword check bang needs_urgent()

5. crates/runtime/src/lib.rs
   SUA: wire formula_engine::evaluate() vao process_text() pipeline
```

---

# =====================================================================
# FE.7 — TEST: Doc P_weight → Reconstruct Formula → Verify
# =====================================================================

## FE.7.1 Rust Test File

**File:** `crates/olang/src/mol/formula_engine_tests.rs` (TAO MOI)
**Hoac:** viet #[cfg(test)] mod tests {} trong moi file eval

```rust
//! Tests cho Formula Engine
//!
//! Moi test CHUNG MINH: doc P_weight → biet cong thuc → biet hinh dang.
//! Khong can ai giai thich.

#[cfg(test)]
mod tests {
    use super::*;

    // ── R DISPATCH ──────────────────────────────────────────────

    #[test]
    fn r0_algebraic_is_group() {
        let a = Molecule::pack(0x10, 0x00, 0x80, 0x80, 0x00); // R=0
        let b = Molecule::pack(0x10, 0x00, 0x80, 0x80, 0x00);
        let result = eval_relation(0, &a, &b);
        assert!(matches!(result, RelationResult::Algebraic { .. }));
        // Doc R=0 → BIET la phep toan dai so
    }

    #[test]
    fn r1_order_compares() {
        let a = Molecule::pack(0x10, 0x10, 0x80, 0x80, 0x00); // R=1
        let b = Molecule::pack(0x20, 0x10, 0x80, 0x80, 0x00);
        let result = eval_relation(1, &a, &b);
        match result {
            RelationResult::Order { ordering, .. } => {
                assert!(ordering < 0, "a < b vi a.bits < b.bits");
            }
            _ => panic!("R=1 phai tra ve Order"),
        }
    }

    #[test]
    fn r9_subset_is_category_morphism() {
        let a = Molecule::pack(0x10, 0x90, 0x80, 0x80, 0x00); // R=9
        let b = Molecule::pack(0x10, 0x90, 0x80, 0x80, 0x00);
        let result = eval_relation(9, &a, &b);
        match result {
            RelationResult::CategoryMorphism { kind, invertible } => {
                assert_eq!(kind, 0x02); // Subset
                assert!(!invertible); // Subset khong invertible
            }
            _ => panic!("R=9 phai tra ve CategoryMorphism"),
        }
    }

    // ── V DISPATCH ──────────────────────────────────────────────

    #[test]
    fn v0_is_high_barrier_repeller() {
        let state = eval_valence(0);
        assert_eq!(state.kind, ValenceKind::HighBarrier);
        assert!(state.force < 0.0, "V=0 → day ra (force < 0)");
        assert!(state.potential > 0.0, "V=0 → rao cao (U > 0)");
    }

    #[test]
    fn v7_is_very_deep_well() {
        let state = eval_valence(7);
        assert_eq!(state.kind, ValenceKind::VeryDeepWell);
        assert!(state.force > 0.0, "V=7 → hut vao (force > 0)");
        assert!(state.potential < 0.0, "V=7 → gieng sau (U < 0)");
    }

    #[test]
    fn v3_v4_are_flat() {
        let s3 = eval_valence(3);
        let s4 = eval_valence(4);
        assert!(s3.force.abs() < 0.01, "V=3 → phang, F ≈ 0");
        assert!(s4.force.abs() < 0.01, "V=4 → phang, F ≈ 0");
    }

    #[test]
    fn valence_symmetry() {
        // V=0 va V=7 phai doi xung: |force| gan bang
        let s0 = eval_valence(0);
        let s7 = eval_valence(7);
        assert!((s0.force.abs() - s7.force.abs()).abs() < 0.1,
            "Day va hut phai doi xung");
    }

    // ── A DISPATCH ──────────────────────────────────────────────

    #[test]
    fn a0_is_ground_state() {
        let state = eval_arousal(0);
        assert_eq!(state.kind, ArousalKind::GroundState);
        assert!(state.energy < 0.1, "A=0 → nang luong cuc thap");
        assert!(state.gamma > 10.0, "A=0 → tat dan cuc manh");
    }

    #[test]
    fn a7_is_supercritical() {
        let state = eval_arousal(7);
        assert_eq!(state.kind, ArousalKind::Supercritical);
        assert!(state.energy > 0.9, "A=7 → nang luong cuc cao");
        assert!(state.lambda > 0.0, "A=7 → positive feedback");
    }

    #[test]
    fn a7_triggers_urgent() {
        assert!(arousal_urgency(7) > 0.8, "A=7 → khan cap");
        assert!(arousal_urgency(0) < 0.1, "A=0 → khong khan cap");
    }

    #[test]
    fn oscillator_supercritical_explodes() {
        let state = eval_arousal(7);
        let x0 = oscillator_at(&state, 0.0);
        let x1 = oscillator_at(&state, 1.0);
        assert!(x1 > x0, "Supercritical: bien do TANG theo thoi gian");
    }

    #[test]
    fn oscillator_overdamped_decays() {
        let state = eval_arousal(3);
        let x0 = oscillator_at(&state, 0.0);
        let x1 = oscillator_at(&state, 1.0);
        assert!(x1 < x0, "Overdamped: bien do GIAM theo thoi gian");
    }

    // ── T SPLINE ────────────────────────────────────────────────

    #[test]
    fn knot_timeless_has_zero_freq() {
        let knot = create_knot(0, 1000, 200, 0.0, 1.0, 0.0);
        assert_eq!(knot.kind, TimeKnotKind::Timeless);
        assert_eq!(knot.frequency, 0.0);
    }

    #[test]
    fn knot_rhythmic_preserves_freq() {
        let knot = create_knot(3, 1000, 100, 440.0, 0.8, 1.57);
        assert_eq!(knot.kind, TimeKnotKind::Rhythmic);
        assert!((knot.frequency - 440.0).abs() < 0.01);
        assert!((knot.phase - 1.57).abs() < 0.01);
    }

    #[test]
    fn history_append_only() {
        let mut h = TimeHistory::new(0xDEAD);
        h.append(create_knot(0, 100, 200, 0.0, 1.0, 0.0));
        h.append(create_knot(0, 200, 150, 0.0, 1.5, 0.0));
        assert_eq!(h.observation_count(), 2);
        assert!((h.mean_amplitude() - 1.25).abs() < 0.01);
    }

    #[test]
    fn history_amplitude_trend_positive_when_learning() {
        let mut h = TimeHistory::new(0xBEEF);
        h.append(create_knot(0, 100, 200, 0.0, 0.5, 0.0));
        h.append(create_knot(0, 200, 200, 0.0, 0.7, 0.0));
        h.append(create_knot(0, 300, 200, 0.0, 1.0, 0.0));
        h.append(create_knot(0, 400, 200, 0.0, 1.2, 0.0));
        assert!(h.amplitude_trend() > 0.0, "Amplitude tang → dang hoc");
    }

    #[test]
    fn spline_from_history_evaluates() {
        let mut h = TimeHistory::new(0xCAFE);
        h.append(create_knot(0, 100, 200, 0.0, 0.0, 0.0));
        h.append(create_knot(0, 200, 200, 0.0, 1.0, 0.0));
        let spline = history_to_amplitude_spline(&h);
        assert!((spline.evaluate(0.0) - 0.0).abs() < 0.1);
        assert!((spline.evaluate(1.0) - 1.0).abs() < 0.1);
    }

    #[test]
    fn predict_behavior_new_concept() {
        let h = TimeHistory::new(0x1234);
        let (lr, fam, per) = predict_behavior(&h);
        assert_eq!(lr, 0.0, "Chua co observation → lr = 0");
        assert_eq!(fam, 0.0, "Chua co → khong quen biet");
        assert_eq!(per, 0.0, "Chua co → khong co nhip");
    }

    // ── FORMULA ENGINE (tich hop) ───────────────────────────────

    #[test]
    fn full_pipeline_love() {
        // "love" → V=6, A=5
        let mol = Molecule::pack(0x10, 0x10, 0xE0, 0xA0, 0x00);
        // V = 0xE0 >> 5 = 7, A = 0xA0 >> 5 = 5
        let state = evaluate(&mol);
        assert!(state.force > 0.5, "love → approach behavior");
        assert!(state.urgency < 0.8, "love khong khan cap");
    }

    #[test]
    fn full_pipeline_danger() {
        // "danger" → V=1, A=7
        let mol = Molecule::pack(0x10, 0x10, 0x20, 0xF0, 0x00);
        // V = 0x20 >> 5 = 1, A = 0xF0 >> 5 = 7
        let state = evaluate(&mol);
        assert!(state.force < 0.0, "danger → avoid behavior");
        assert!(state.urgency > 0.8, "danger → KHAN CAP");
    }

    #[test]
    fn full_pipeline_neutral() {
        // "process" → V=4, A=4
        let mol = Molecule::pack(0x10, 0x10, 0x80, 0x80, 0x00);
        // V = 0x80 >> 5 = 4, A = 0x80 >> 5 = 4
        let state = evaluate(&mol);
        assert!(state.force.abs() < 0.1, "process → neutral");
        assert!(state.urgency < 0.5, "process → khong khan cap");
    }
}
```

## FE.7.2 Chay test

```bash
# Chay tat ca FE tests
cargo test -p olang -- formula_engine
cargo test -p olang -- relation_eval
cargo test -p olang -- valence_eval
cargo test -p olang -- arousal_eval
cargo test -p vsdf -- time_knot
cargo test -p vsdf -- time_spline

# Chay tat ca cung luc
cargo test --workspace -- formula_engine relation_eval valence_eval arousal_eval time_knot time_spline
```

---

## Cau truc Formula Dispatch tong the

```rust
// PSEUDO-CODE — toan bo flow

fn process_molecule(mol: &Molecule, obs_time: u32, obs_duration: u32) {
    // 1. EVALUATE — doc P_weight → biet tat ca
    let state = formula_engine::evaluate(mol);

    // 2. URGENCY CHECK — SecurityGate
    if state.urgency > 0.8 {
        security_gate::check_crisis(mol);
    }

    // 3. BEHAVIOR — approach hay avoid?
    let tendency = state.force;
    //   > 0: di ve phia node nay (approach)
    //   < 0: tranh xa node nay (avoid)

    // 4. TIME KNOT — ghi observation
    let knot = create_knot(
        state.time_kind,
        obs_time,
        obs_duration,
        0.0,  // sensor_freq (0 cho text)
        1.0,  // sensor_amp (1.0 cho text)
        0.0,  // sensor_phase
    );
    time_history.append(knot);

    // 5. PREDICT — doc spline, du doan
    let (learning_rate, familiarity, periodicity) = predict_behavior(&time_history);

    // 6. RENDER — dung physics de quyet dinh
    match state.arousal.kind {
        Supercritical => render_urgent(mol),     // flashing, immediate
        Resonance     => render_intense(mol),    // bold, vibrant
        ExcitedState  => render_active(mol),     // animated
        Equilibrium   => render_normal(mol),     // standard
        Overdamped    => render_calm(mol),        // slow fade-in
        _             => render_minimal(mol),     // static
    }
}
```

---

## T Triet De — Spline Accumulation

```
Moi lan observe 1 concept → tao 1 SplineKnot → append vao T history.

Vi du: observe "Hinh vuong" 3 lan:

  Lan 1: doc ten "Hinh vuong"
    → T1 = SplineKnot { duration: 200ms, freq: 0, amp: 1.0, phase: 0 }

  Lan 2: doc alias "Rectangle"
    → T2 = SplineKnot { duration: 150ms, freq: 0, amp: 0.8, phase: pi/3 }

  Lan 3: doc dinh nghia "S = a²"
    → T3 = SplineKnot { duration: 500ms, freq: 0, amp: 1.5, phase: 2pi/3 }

  T_history["Hinh vuong"] = [T1, T2, T3]

  Spline interpolation: psi(t) = Sum Ti.amp × B(t - ti)
  → Duong cong hoc tap cua "Hinh vuong"
  → Doc spline → biet: "da gap 3 lan, moi lan sau hon (amp tang)"

Sensor "light":
  Lan 1: lambda = 580nm, I = 1000 lux
    → T1 = SplineKnot { frequency: c/580nm, amplitude: 1000, phase: 0 }
    → Silk → P{"anh sang vang"}

  Lan 2: lambda = 450nm, I = 500 lux
    → T2 = SplineKnot { frequency: c/450nm, amplitude: 500, phase: pi/4 }
    → Silk → P{"anh sang xanh"}

  T_history["light"] = [T1, T2]
  → Spline → biet: "anh sang thay doi tu vang → xanh, cuong do giam"
```

---

## Tasks

| ID | Task | Effort | Status | Files |
|----|------|--------|--------|-------|
| FE.1 | R dispatch (16 relation types → operations) | ~200 LOC | FREE | `crates/olang/src/mol/relation_eval.rs` |
| FE.2 | V dispatch (8 levels → ValenceState) | ~120 LOC | FREE | `crates/olang/src/mol/valence_eval.rs` |
| FE.3 | A dispatch (8 levels → ArousalState) | ~130 LOC | FREE | `crates/olang/src/mol/arousal_eval.rs` |
| FE.4 | T SplineKnot (moi observe → append knot) | ~200 LOC | FREE | `crates/vsdf/src/dynamics/time_knot.rs` |
| FE.5 | T Spline interp (history → curve → predict) | ~150 LOC | FREE | `crates/vsdf/src/dynamics/time_spline.rs` |
| FE.6 | Wire formula engine vao pipeline | ~100 LOC | FREE | `crates/olang/src/mol/formula_engine.rs` + sua pipeline |
| FE.7 | Test suite | ~200 LOC | FREE | Tests trong tung file + integration test |

Tong: ~1,100 LOC code + ~300 LOC test = ~1,400 LOC.
Day la **core architecture** — bien P_weight tu so tinh thanh **cong thuc song**.

---

## Dependency

```
FE.1 (R dispatch) ← doc lap
FE.2 (V dispatch) ← doc lap
FE.3 (A dispatch) ← doc lap
FE.4 (T knots)    ← doc lap
FE.5 (T spline)   ← FE.4 (can SplineKnot + TimeHistory)
FE.6 (wire)       ← FE.1-5 (can tat ca dispatchers)
FE.7 (tests)      ← FE.1-6 (test tich hop)

FE.1-4 song song. FE.5 sau FE.4. FE.6-7 cuoi.

Thu tu lam:
  Session 1: FE.2 + FE.3 (don gian nhat, doc lap)
  Session 2: FE.1 (phuc tap hon — 16 arms)
  Session 3: FE.4 + FE.5 (T system)
  Session 4: FE.6 + FE.7 (wire + test)
```

---

## File Map — Moi file o dau

```
crates/olang/src/mol/
├── mod.rs               ← THEM 4 dong pub mod
├── molecular.rs         ← KHONG SUA (chi reference)
├── relation_eval.rs     ← MOI (FE.1)
├── valence_eval.rs      ← MOI (FE.2)
├── arousal_eval.rs      ← MOI (FE.3)
└── formula_engine.rs    ← MOI (FE.6)

crates/vsdf/src/dynamics/
├── mod.rs               ← THEM 2 dong pub mod
├── spline.rs            ← DA CO (VectorSpline, BezierSegment) — KHONG SUA
├── time_knot.rs         ← MOI (FE.4)
└── time_spline.rs       ← MOI (FE.5)

crates/olang/src/mol/formula_engine_tests.rs ← MOI (FE.7)
  HOAC viet tests trong tung file tren
```

---

## Tai sao quan trong

```
HIEN TAI:
  P_weight = 2 bytes so tinh → so sanh >, < → xong
  "Hinh vuong" gap 1 lan = gap 100 lan (T khong thay doi)
  Sensor do 580nm = do 450nm (T khong phan biet)
  "love" va "hate" chi khac 1 byte — khong co physics
  "earthquake" khong trigger urgent — chi la 1 tu nhu moi tu

SAU FORMULA ENGINE:
  P_weight = 2 bytes → dispatch → CONG THUC SONG
  "Hinh vuong" gap 100 lan → T spline day → "hieu sau"
  580nm vs 450nm → T knots khac → "phan biet duoc"
  "love" → V=6 DeepWell → force=+0.8 → APPROACH behavior
  "hate" → V=0 HighBarrier → force=-0.9 → AVOID behavior
  "earthquake" → A=7 Supercritical → urgency=0.95 → CRISIS CHECK

  Gia tri TU MO TA. Doc so → biet hinh dang.
  Khong can AI, khong can lookup, khong can annotation.
  DNA khong can giai thich ATCG — ribosome DOC va LAM.
```
