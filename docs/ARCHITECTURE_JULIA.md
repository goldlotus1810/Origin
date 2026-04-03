# ORIGIN/NOX — KIẾN TRÚC JULIA
## Tài liệu gốc cho mọi session. ĐỌC TRƯỚC KHI LÀM GÌ.

**Version:** 1.0 — 2026-04-03
**Mục đích:** Từ spec đến ứng dụng thực tế trên nền Julia
**Nguồn gốc:** Tổng hợp từ 4 specs + verified claims + Claude CLI architecture + Julia ecosystem research

---

## MỤC LỤC

- [§0 — TẠI SAO JULIA, KHÔNG PHẢI C VM](#0-tại-sao-julia)
- [§1 — TỔNG QUAN KIẾN TRÚC](#1-tổng-quan-kiến-trúc)
- [§2 — MOLECULAR ENGINE](#2-molecular-engine)
  - [§2.1 — Molecule 16-bit Format](#21-molecule-16-bit-format)
  - [§2.2 — 5 Dimensions](#22-5-dimensions)
  - [§2.3 — Distance Metric](#23-distance-metric)
  - [§2.4 — 42 Encode Formulas](#24-42-encode-formulas)
  - [§2.5 — Molecular Chain](#25-molecular-chain)
  - [§2.6 — LCA Compose (5 Rules)](#26-lca-compose-5-rules)
  - [§2.7 — Vietnamese Pipeline](#27-vietnamese-pipeline)
  - [§2.8 — NRC-VAD Integration](#28-nrc-vad-integration)
  - [§2.9 — Decode (Chain → Text)](#29-decode-chain--text)
  - [§2.10 — Verified Claims & Honest Limitations](#210-verified-claims--honest-limitations)
- [§3 — KNOWTREE](#3-knowtree)
  - [§3.1 — Data Structure](#31-data-structure)
  - [§3.2 — Operations](#32-operations)
  - [§3.3 — Eviction Strategy](#33-eviction-strategy)
  - [§3.4 — VP-Tree Search](#34-vp-tree-search)
- [§4 — SILK (HEBBIAN LEARNING)](#4-silk-hebbian-learning)
  - [§4.1 — Edge Structure](#41-edge-structure)
  - [§4.2 — Co-activation (Learning)](#42-co-activation-learning)
  - [§4.3 — Decay Formula](#43-decay-formula)
  - [§4.4 — Implicit Layer](#44-implicit-layer)
  - [§4.5 — ShadowVector](#45-shadowvector)
- [§5 — PIPELINE (8 STAGES)](#5-pipeline-8-stages)
  - [§5.1 — Stage Overview](#51-stage-overview)
  - [§5.2 — Checkpoints (3)](#52-checkpoints-3)
  - [§5.3 — Timing Budget](#53-timing-budget)
- [§6 — INSTINCTS (7 HEURISTICS)](#6-instincts-7-heuristics)
  - [§6.1 — Ordering (Kahneman S1/S2)](#61-ordering-kahneman-s1s2)
  - [§6.2 — Scoring & Competition](#62-scoring--competition)
  - [§6.3 — 7 Formulas](#63-7-formulas)
- [§7 — DREAM CYCLE](#7-dream-cycle)
- [§8 — MEMORY MODEL (3-LAYER)](#8-memory-model-3-layer)
- [§9 — CONFIDENCE TYPE](#9-confidence-type)
- [§10 — SELF-MODIFICATION (SAFE)](#10-self-modification-safe)
- [§11 — PERSISTENCE & STATE](#11-persistence--state)
- [§12 — JULIA PROJECT STRUCTURE](#12-julia-project-structure)
- [§13 — IMPLEMENTATION PHASES](#13-implementation-phases)
- [§14 — PATTERNS HỌC TỪ CLAUDE CLI](#14-patterns-học-từ-claude-cli)
- [§15 — QUYẾT ĐỊNH ĐÃ ĐÓNG](#15-quyết-định-đã-đóng)
- [§16 — BLOCKING ISSUES & DEPENDENCIES](#16-blocking-issues--dependencies)

---

## §0 — TẠI SAO JULIA

### Vấn đề với C VM tự viết

Viết VM từ zero bằng C giải quyết 0% giá trị cốt lõi của Origin. Các vấn đề:

| Phải tự viết trong C | Julia đã có |
|---|---|
| Bytecode interpreter | JIT → LLVM native code |
| GC (Cheney semi-space) | Generational GC built-in |
| Concurrency (scheduler, yield) | `Task`, `Channel`, M:N scheduler, work-stealing |
| Unicode string handling | UTF-8 native, `Unicode.normalize()` |
| Closures + upvalue capture | First-class closures, correct lifetime |
| Error handling (try/finally) | `try/catch/finally` built-in |
| Module system | `module`, `import`, `using` |
| Self-modification (unsafe string replace) | `Meta.parse()` → validate → `eval()` safe |
| NaN boxing (value representation) | Native Float64, Int64, Any |

### Cái giữ lại 100%

Toàn bộ cognitive science và molecular engine:
- 16-bit molecule encoding + 42 formulas
- KnowTree (256-bucket hash + chains)
- Silk (Hebbian learning + stretched exponential decay)
- Pipeline (cognitive loop)
- 7 Instincts (reasoning heuristics)
- Dream cycle (clustering + hypothesis)
- NRC-VAD emotion lexicon
- Vietnamese tone pipeline

### Cái mất

- "Zero dependency, 46KB" → Julia runtime ~500MB
- "Bare metal" → Cần Julia installed
- Cold start ~1-2s (sau JIT warmup thì nhanh)

**Quyết định:** Brain hoạt động > binary nhỏ. Ship > perfection.

---

## §1 — TỔNG QUAN KIẾN TRÚC

```
┌──────────────────────────────────────────────────┐
│                   APPLICATION                     │
│  REPL / CLI / API / IDE Bridge                   │
├──────────────────────────────────────────────────┤
│                   PIPELINE                        │
│  Gate → Encode → Search+Compose → Instincts      │
│       → Learn → Dream → Decode                   │
├──────────┬──────────┬──────────┬─────────────────┤
│ INSTINCTS│  DREAM   │  SELF-   │   PERSISTENCE   │
│ 7 heur.  │  cycle   │  MODIFY  │   save/load     │
├──────────┴──────────┴──────────┴─────────────────┤
│                    BRAIN                          │
│          KnowTree    +    Silk                    │
├──────────────────────────────────────────────────┤
│              MOLECULAR ENGINE                     │
│  encode  decode  distance  compose  chain         │
├──────────────────────────────────────────────────┤
│              JULIA RUNTIME                        │
│  JIT  GC  Tasks  Channels  Unicode  Meta          │
└──────────────────────────────────────────────────┘
```

### Package Structure

```
Origin.jl/
├── Project.toml
├── src/
│   ├── Origin.jl              # Main module, exports public API
│   ├── Molecule.jl            # §2 — encode/decode/distance/compose
│   ├── Chain.jl               # §2.5 — MolecularChain operations
│   ├── KnowTree.jl           # §3 — fact storage & retrieval
│   ├── Silk.jl                # §4 — Hebbian learning graph
│   ├── Pipeline.jl            # §5 — 8-stage cognitive loop
│   ├── Instincts.jl           # §6 — 7 reasoning heuristics
│   ├── Dream.jl               # §7 — consolidation cycle
│   ├── Memory.jl              # §8 — 3-layer decay model
│   ├── Confidence.jl          # §9 — Confidence <: Real type
│   ├── SelfModify.jl          # §10 — safe code evolution
│   ├── Persist.jl             # §11 — save/load state
│   ├── Vietnamese.jl          # §2.7 — tone detection + segmentation
│   ├── NrcVad.jl              # §2.8 — emotion lexicon
│   ├── VPTree.jl              # §3.4 — vantage point tree search
│   └── ShadowVector.jl        # §4.5 — contextual embedding
├── data/
│   ├── nrc_vad_v2.tsv         # 19,971 emotion entries
│   ├── vn_emolex.tsv          # 12,795 Vietnamese entries
│   ├── unicode_name_flags.bin # Pre-computed (build script output)
│   └── vn_compounds.txt       # 30K Vietnamese compound words
├── build/
│   ├── build_unicode_flags.jl # §2.4 — Parse UnicodeData.txt → flags
│   ├── build_nrc_vad.jl       # §2.8 — Parse NRC-VAD → binary
│   └── build_vn_dict.jl       # §2.7 — Parse Vietnamese dictionary
├── test/
│   ├── test_molecule.jl
│   ├── test_knowtree.jl
│   ├── test_silk.jl
│   ├── test_pipeline.jl
│   ├── test_instincts.jl
│   ├── test_dream.jl
│   └── test_confidence.jl
├── repl/
│   └── origin_repl.jl         # Interactive REPL interface
└── docs/
    └── ARCHITECTURE_JULIA.md  # This file (symlink from spec/)
```

---

## §2 — MOLECULAR ENGINE

### §2.1 — Molecule 16-bit Format

Mỗi semantic unit encode thành 1 `UInt16`:

```
Bit layout: [S:4][R:4][V:3][A:3][T:2] = 16 bits

S (Shape)    — bits 15:12 — structural complexity (0-15)
R (Relation) — bits 11:8  — semantic role (0-15)
V (Valence)  — bits 7:5   — emotional polarity (0-7)
A (Arousal)  — bits 4:2   — intensity (0-7)
T (Time)     — bits 1:0   — temporal state (0-3)
```

**Julia implementation:**

```julia
# src/Molecule.jl

struct Molecule
    pw::UInt16
end

# Dimension extraction
shape(m::Molecule)    = (m.pw >> 12) & 0x0F  # 4 bits
relation(m::Molecule) = (m.pw >> 8)  & 0x0F  # 4 bits
valence(m::Molecule)  = (m.pw >> 5)  & 0x07  # 3 bits
arousal(m::Molecule)  = (m.pw >> 2)  & 0x07  # 3 bits
time(m::Molecule)     = m.pw & 0x03           # 2 bits

# Packing
function mol_pack(s::UInt8, r::UInt8, v::UInt8, a::UInt8, t::UInt8)::Molecule
    pw = (UInt16(s & 0x0F) << 12) |
         (UInt16(r & 0x0F) << 8)  |
         (UInt16(v & 0x07) << 5)  |
         (UInt16(a & 0x07) << 2)  |
         UInt16(t & 0x03)
    Molecule(pw)
end

# Unpacking
function mol_unpack(m::Molecule)
    (shape(m), relation(m), valence(m), arousal(m), time(m))
end
```

### §2.2 — 5 Dimensions

**S — Shape (4 bits, 16 values)**

Dựa trên SDF (Signed Distance Field) primitive complexity:

| S | Name | SDF Primitive | Ví dụ |
|---|------|---------------|-------|
| 0 | Point | Point | Dấu chấm, period |
| 1 | Line | Line segment | Gạch ngang, dash |
| 2 | Triangle | Triangle | Delta, arrows đơn giản |
| 3 | Rectangle | Box | Brackets, frames |
| 4 | Circle | Sphere | Letters O, 0, circles |
| 5 | Ellipse | Ellipsoid | Letters e, c, rounded |
| 6 | Cross | Cross/Plus | +, ×, dagger |
| 7 | Star | Star/Polygon | *, asterisk |
| 8 | Spiral | Torus/Helix | @, snail, spiral |
| 9 | Wave | Sine/Cosine | ~, wavy lines |
| 10 | Branch | Tree/Fork | Y, arrows phân nhánh |
| 11 | Grid | Grid/Matrix | #, tables |
| 12 | Nested | Nested shapes | (), {}, compound |
| 13 | Symmetric | Mirror/Rotate | ⊕, ♦, symmetric |
| 14 | Fractal | Self-similar | Complex emoji |
| 15 | Composite | Multi-part | Compound characters |

**R — Relation (4 bits, 16 values)**

| R | RelationOp | Ý nghĩa | Ví dụ |
|---|-----------|---------|-------|
| 0 | IDENTITY | Chính nó | a = a |
| 1 | MEMBER | Thuộc tập | ∈, thuộc |
| 2 | EQUALITY | Bằng nhau | =, ≡ |
| 3 | ORDER | Thứ tự | <, >, ≤ |
| 4 | ARITHMETIC | Phép tính | +, -, ×, ÷ |
| 5 | COMPOSE | Kết hợp | ∘, pipe |
| 6 | CJK/ANCIENT | Ideograph | 漢字 |
| 7 | LOGICAL | Logic | ∧, ∨, ¬ |
| 8 | SETOP | Tập hợp | ∪, ∩, ⊂ |
| 9 | CAUSES | Nhân quả | →, ⟹ |
| 10 | APPROXIMATE | Xấp xỉ | ≈, ~ |
| 11 | AGGREGATE | Tổng hợp | Σ, Π |
| 12 | DIRECTIONAL | Hướng | ↑, →, ↓, ← |
| 13 | BRACKET | Đóng mở | (), [], {} |
| 14 | INVERSE | Nghịch đảo | ⁻¹, NOT |
| 15 | MODIFIER | Bổ nghĩa | Diacritics, accents |

**V — Valence (3 bits, 8 values)**

Emotional polarity, from Coulomb repulsion model:

| V | Range | Ý nghĩa | NRC-VAD mapping |
|---|-------|---------|-----------------|
| 0 | [0.000, 0.125) | Very negative | fear, disgust |
| 1 | [0.125, 0.250) | Negative | sadness, anger |
| 2 | [0.250, 0.375) | Slightly negative | worry, doubt |
| 3 | [0.375, 0.500) | Neutral-low | calm, mundane |
| 4 | [0.500, 0.625) | Neutral-high | okay, normal |
| 5 | [0.625, 0.750) | Slightly positive | interest |
| 6 | [0.750, 0.875) | Positive | happiness, pride |
| 7 | [0.875, 1.000] | Very positive | joy, love, ecstasy |

Quantization: `v = clamp(floor(nrc_valence × 8), 0, 7)`

**A — Arousal (3 bits, 8 values)**

Intensity, from damped harmonic oscillator model:

| A | Range | Ý nghĩa | NRC-VAD mapping |
|---|-------|---------|-----------------|
| 0 | [0.000, 0.125) | Ground state | comatose, dead |
| 1 | [0.125, 0.250) | Very low | sleepy, bored |
| 2 | [0.250, 0.375) | Low | relaxed, calm |
| 3 | [0.375, 0.500) | Medium-low | attentive |
| 4 | [0.500, 0.625) | Medium-high | engaged |
| 5 | [0.625, 0.750) | High | excited, alert |
| 6 | [0.750, 0.875) | Very high | thrilled, angry |
| 7 | [0.875, 1.000] | Supercritical | panic, rage, ecstasy |

Quantization: `a = clamp(floor(nrc_arousal × 8), 0, 7)`

**T — Time (2 bits, 4 values)**

| T | Ý nghĩa | Ví dụ |
|---|---------|-------|
| 0 | STATIC | Constants, symbols, letters |
| 1 | SLOW | Nouns, stable concepts |
| 2 | MEDIUM | Verbs, actions |
| 3 | FAST | Exclamations, dynamic emoji |

### §2.3 — Distance Metric

**Weighted Manhattan distance:**

```julia
function mol_dist(a::Molecule, b::Molecule)::Int
    ds = abs(Int(shape(a))    - Int(shape(b)))     # max 15
    dr = abs(Int(relation(a)) - Int(relation(b)))   # max 15
    dv = abs(Int(valence(a))  - Int(valence(b)))    # max 7
    da = abs(Int(arousal(a))  - Int(arousal(b)))    # max 7
    dt = abs(Int(time(a))     - Int(time(b)))       # max 3
    return ds + dr + 2*dv + 2*da + 4*dt             # max 70
end
```

**Weights: (1, 1, 2, 2, 4)**

⚠️ HONEST NOTE: Weights là TUNABLE engineering choice.
- Papers (Russell 1980, Zajonc 1980) mô tả V/A là dimensions cảm xúc, KHÔNG nói weight cụ thể
- Default (1,1,2,2,4) giả định emotion quan trọng hơn structure
- CẦN ĐO bằng experiment: encode 1000 words, measure retrieval quality, tune weights
- Xem §2.10 cho verified claims chi tiết

**Similarity (normalized):**

```julia
mol_similarity(a::Molecule, b::Molecule)::Float64 = 1.0 - mol_dist(a, b) / 70.0
```

### §2.4 — 42 Encode Formulas

**Architecture:** Master router F₀ dispatch theo Unicode block → 5 per-dimension encoders

```julia
function encode_codepoint(cp::UInt32)::Molecule
    # F₀: Master router
    s = encode_shape(cp)
    r = encode_relation(cp)
    v = encode_valence(cp)
    a = encode_arousal(cp)
    t = encode_time(cp)
    mol_pack(s, r, v, a, t)
end
```

**Shape encoder (f_S):** Keyword matching trên Unicode name flags

```julia
function encode_shape(cp::UInt32)::UInt8
    flags = UNICODE_FLAGS[cp + 1]  # 1-indexed in Julia
    # Priority order — first match wins
    (flags & FLAG_ARROW)     != 0 && return 0x02  # Triangle
    (flags & FLAG_GEOMETRIC) != 0 && return 0x03  # Rectangle
    (flags & FLAG_CIRCLE)    != 0 && return 0x04  # Circle
    (flags & FLAG_STAR)      != 0 && return 0x07  # Star
    (flags & FLAG_BRACKET)   != 0 && return 0x0C  # Nested
    # ... (10 keyword classifiers total)
    return default_shape_by_block(cp)
end
```

**BLOCKING DEPENDENCY:** `UNICODE_FLAGS` array phải được pre-compute bởi `build/build_unicode_flags.jl`

Input: `UnicodeData.txt` (Unicode Consortium)
Output: `data/unicode_name_flags.bin` — 65,536 × UInt16 = 128KB
Process: Parse Unicode name → extract keywords → set bits

```julia
# build/build_unicode_flags.jl
# Chạy 1 LẦN trước khi dùng Origin.jl

const FLAG_ARROW     = UInt16(1 << 0)
const FLAG_GEOMETRIC = UInt16(1 << 1)
const FLAG_CIRCLE    = UInt16(1 << 2)
const FLAG_STAR      = UInt16(1 << 3)
const FLAG_BRACKET   = UInt16(1 << 4)
const FLAG_MUSICAL   = UInt16(1 << 5)
const FLAG_MATH      = UInt16(1 << 6)
const FLAG_CJK       = UInt16(1 << 7)
const FLAG_MODIFIER  = UInt16(1 << 8)
const FLAG_DIGIT     = UInt16(1 << 9)

function build_unicode_flags(ucd_path::String)::Vector{UInt16}
    flags = zeros(UInt16, 65536)
    for line in eachline(ucd_path)
        fields = split(line, ';')
        cp = parse(UInt32, fields[1], base=16)
        cp > 0xFFFF && continue  # BMP only
        name = uppercase(fields[2])
        
        occursin("ARROW", name)          && (flags[cp+1] |= FLAG_ARROW)
        occursin("CIRCLE", name)         && (flags[cp+1] |= FLAG_CIRCLE)
        occursin("STAR", name)           && (flags[cp+1] |= FLAG_STAR)
        occursin("BRACKET", name)        && (flags[cp+1] |= FLAG_BRACKET)
        occursin("MUSICAL", name)        && (flags[cp+1] |= FLAG_MUSICAL)
        occursin("MATHEMATICAL", name)   && (flags[cp+1] |= FLAG_MATH)
        occursin("CJK", name)           && (flags[cp+1] |= FLAG_CJK)
        occursin("MODIFIER", name)       && (flags[cp+1] |= FLAG_MODIFIER)
        occursin("DIGIT", name)          && (flags[cp+1] |= FLAG_DIGIT)
        
        # Geometric: SQUARE, TRIANGLE, RECTANGLE, DIAMOND
        any(w -> occursin(w, name), ["SQUARE","TRIANGLE","RECTANGLE","DIAMOND"]) &&
            (flags[cp+1] |= FLAG_GEOMETRIC)
    end
    # Write binary
    open("data/unicode_name_flags.bin", "w") do io
        write(io, flags)
    end
    flags
end
```

**Valence encoder (f_V):** NRC-VAD lookup → quantize

```julia
function encode_valence(cp::UInt32)::UInt8
    # Level 1: Character-level (emoji, symbols)
    emoji_v = emoji_valence_default(cp)
    emoji_v !== nothing && return emoji_v
    
    # Level 2: Block-level default
    return block_default_valence(cp)
end

# Word-level valence (called from chain-level, not codepoint-level)
function word_valence(word::String)::Union{UInt8, Nothing}
    entry = get(NRC_VAD, lowercase(word), nothing)
    entry === nothing && return nothing
    return UInt8(clamp(floor(Int, entry.valence * 8), 0, 7))
end
```

**Arousal encoder (f_A):** Similar pattern, NRC-VAD lookup

**Time encoder (f_T):** Musical notation keywords + block defaults

### §2.5 — Molecular Chain

```julia
# src/Chain.jl

const MolecularChain = Vector{Molecule}

function encode_text(text::String)::MolecularChain
    [encode_codepoint(UInt32(c)) for c in text]
end

function chain_hash(chain::MolecularChain)::UInt64
    h = UInt64(0x9e3779b97f4a7c15)  # Fibonacci hash seed
    for m in chain
        h = xor(h, UInt64(m.pw)) * UInt64(0x517cc1b727220a95)
        h = xor(h >> 32, h)
    end
    h
end

function chain_similarity(a::MolecularChain, b::MolecularChain)::Float64
    len = min(length(a), length(b))
    len == 0 && return 0.0
    total = sum(mol_similarity(a[i], b[i]) for i in 1:len)
    # Length penalty
    max_len = max(length(a), length(b))
    (total / len) * (len / max_len)
end
```

### §2.6 — LCA Compose (5 Rules)

Khi combine nhiều molecules (ví dụ: tóm tắt 1 cluster), mỗi dimension dùng rule riêng:

```julia
function chain_lca(chains::Vector{MolecularChain},
                   weights::Vector{Float64})::MolecularChain
    # Tính LCA cho mỗi position
    max_len = maximum(length.(chains))
    result = Molecule[]
    for pos in 1:max_len
        mols = [chains[i][pos] for i in eachindex(chains) if pos <= length(chains[i])]
        ws = [weights[i] for i in eachindex(chains) if pos <= length(chains[i])]
        push!(result, compose_molecules(mols, ws))
    end
    result
end

function compose_molecules(mols::Vector{Molecule}, weights::Vector{Float64})::Molecule
    s = compose_union([shape(m) for m in mols], weights)       # S: MAX by weight
    r = compose_relation([relation(m) for m in mols])           # R: Idempotent/COMPOSE
    v = compose_amplify([valence(m) for m in mols], weights)    # V: Weighted deviation
    a = compose_max([arousal(m) for m in mols])                 # A: MAX
    t = compose_dominant([time(m) for m in mols], weights)      # T: MAX by weight
    mol_pack(s, r, v, a, t)
end
```

**5 Rules detail:**

```julia
# S: Union — dominant shape (highest weight wins, tiebreak by value)
function compose_union(values::Vector{UInt8}, weights::Vector{Float64})::UInt8
    max_w = maximum(weights)
    candidates = [values[i] for i in eachindex(values) if weights[i] ≈ max_w]
    maximum(candidates)
end

# R: Idempotent — if all same, keep. If different, return COMPOSE (0x05)
function compose_relation(values::Vector{UInt8})::UInt8
    all(==(values[1]), values) ? values[1] : 0x05
end

# V: Amplify — weighted mean + deviation boost (NOT average)
function compose_amplify(values::Vector{UInt8}, weights::Vector{Float64})::UInt8
    total_w = sum(weights)
    base = sum(Float64(values[i]) * weights[i] for i in eachindex(values)) / total_w
    dev = sum(abs(Float64(values[i]) - base) * weights[i] for i in eachindex(values)) / total_w
    # Boost: move away from neutral (4) by deviation amount
    result = if base >= 4.0
        base + dev * 0.5
    else
        base - dev * 0.5
    end
    UInt8(clamp(round(Int, result), 0, 7))
end

# A: Max — take highest arousal
function compose_max(values::Vector{UInt8})::UInt8
    maximum(values)
end

# T: Dominant — same as S union
compose_dominant = compose_union
```

⚠️ **HONEST NOTE on associativity:**
LCA compose is APPROXIMATELY associative: `LCA(LCA(a,b),c) ≈ LCA(a,LCA(b,c))`
- V amplify and S union rules ensure idempotency and commutativity
- But floating-point rounding means NOT exactly equal
- This is ACCEPTABLE for a fuzzy semantic system. Document, don't hide.

### §2.7 — Vietnamese Pipeline

```julia
# src/Vietnamese.jl

using Unicode

# 6 Vietnamese tones with V/A modulation
const VN_TONES = Dict{Char, Tuple{Float64, Float64}}(
    '\u0300' => (-0.15, -0.10),  # huyền (grave) — V down, A down
    '\u0301' => (+0.15, +0.10),  # sắc (acute) — V up, A up
    '\u0309' => (-0.08, +0.05),  # hỏi (hook) — V slightly down, A slightly up
    '\u0303' => (+0.05, +0.15),  # ngã (tilde) — V neutral, A up
    '\u0323' => (-0.10, +0.15),  # nặng (dot below) — V down, A up
    # ngang (level) = no diacritic = no modulation
)

function encode_vietnamese_char(text::String, pos::Int)::Molecule
    # NFD decompose to separate base + combining marks
    nfd = Unicode.normalize(string(text[pos]), :NFD)
    base_cp = UInt32(nfd[1])
    base_mol = encode_codepoint(base_cp)
    
    # Check for tone marks in combining characters
    (s, r, v, a, t) = mol_unpack(base_mol)
    for i in 2:length(nfd)  # ncollect thay vì nextind
        mark = nfd[i]
        if haskey(VN_TONES, mark)
            dv, da = VN_TONES[mark]
            v = UInt8(clamp(round(Int, Float64(v) + dv * 8), 0, 7))
            a = UInt8(clamp(round(Int, Float64(a) + da * 8), 0, 7))
        end
    end
    
    mol_pack(s, r, v, a, t)
end

# Word segmentation (longest-match greedy)
function segment_vietnamese(text::String, dict::Set{String})::Vector{String}
    words = String[]
    i = 1
    while i <= length(text)
        best_len = 1
        # Try longest match first (max compound word = 4 syllables ≈ 20 chars)
        for len in min(20, length(text)-i+1):-1:2
            candidate = text[i:prevind(text, i+len)]
            if candidate in dict
                best_len = length(candidate)
                break
            end
        end
        push!(words, text[i:prevind(text, i+best_len)])
        i += best_len
    end
    words
end
```

### §2.8 — NRC-VAD Integration

```julia
# src/NrcVad.jl

struct VadEntry
    valence::Float32   # [0, 1]
    arousal::Float32   # [0, 1]
    dominance::Float32 # [0, 1]
end

# Global loaded at module init
const NRC_VAD = Dict{String, VadEntry}()

function load_nrc_vad!(path::String)
    empty!(NRC_VAD)
    for line in eachline(path)
        startswith(line, "Word") && continue  # header
        parts = split(line, '\t')
        length(parts) >= 4 || continue
        word = lowercase(parts[1])
        v = parse(Float32, parts[2])
        a = parse(Float32, parts[3])
        d = parse(Float32, parts[4])
        NRC_VAD[word] = VadEntry(v, a, d)
    end
end
```

**Coverage:**
- English: 44,728 / ~170,000 words = **~26%**
- Vietnamese (VnEmoLex): 12,795 / ~50,000 = **~26%**
- Unknown words → fallback to codepoint-level encoding (neutral V=4, A=4)

⚠️ **HONEST:** 74% words không có emotion data. Fallback là silent.
TODO: Log unknown words, build custom lexicon over time via Silk learning.

### §2.9 — Decode (Chain → Text)

```julia
# Inverted map: Molecule → Vector of candidate codepoints
const DECODE_MAP = Dict{UInt16, Vector{Tuple{UInt32, Float32}}}()
# Populated at init: for each BMP codepoint, encode → store in bucket

function decode_chain(chain::MolecularChain; context::String="")::String
    buf = IOBuffer()
    for mol in chain
        candidates = get(DECODE_MAP, mol.pw, Tuple{UInt32, Float32}[])
        if isempty(candidates)
            # Nearest neighbor fallback
            best_cp = nearest_decode(mol)
            write(buf, Char(best_cp))
        elseif length(candidates) == 1
            write(buf, Char(candidates[1][1]))
        else
            # Disambiguation: use context (script detection, frequency)
            cp = disambiguate(candidates, context)
            write(buf, Char(cp))
        end
    end
    String(take!(buf))
end
```

⚠️ **HONEST:** Decode là LOSSY reconstruction. `decode(encode(text)) ≈ text`, không `==`.
Collision ratio ~2.36:1 trên BMP. Acceptable cho semantic search, KHÔNG cho lossless storage.

### §2.10 — Verified Claims & Honest Limitations

**Kết quả verify 4 claims (đã kiểm chứng với papers gốc):**

| # | Claim | Kết quả | Hành động |
|---|-------|---------|-----------|
| 1 | Similarity weights 0.3/0.2/0.5 | **TÙY Ý** — Russell/Zajonc KHÔNG support con số cụ thể | Ghi TUNABLE, default, KHÔNG cite sai paper |
| 2 | Circadian consolidation | **METAPHOR** — Born & Wilhelm 2012 (*Psychological Research*, NOT *Psychological Bulletin*). Bio cycle = 90 min SWS+REM, KHÔNG linear 30 min | Gọi "idle-phase scheduling", KHÔNG gọi "circadian". Thresholds 60s/5m/30m = tunable engineering |
| 3 | BuildZone confidence ≥ 0.90 | **Framework đúng, số tùy ý** — NARS default ~0.6, Peirce abduction không cho số | Dùng 0.90 nhưng ghi = tunable, không cite NARS cho số |
| 4 | Instinct ordering (Kahneman) | **CLAIM MẠNH NHẤT** ✅ — Kahneman S1/S2 thật, CLARION/LIDA/CEUR 2022 có precedent | Giữ nguyên, cite đúng |

**Limitations thẳng thắn:**

1. **16-bit encoding là lossy hash**, không phải "semantic encoder". 65,536 slots cho 155K codepoints = collision.
2. **Distance metric weights arbitrary.** CẦN đo empirically, hiện tại là engineering default.
3. **NRC-VAD coverage ~26%.** Đa số words fallback neutral. Cần build custom lexicon.
4. **LCA compose approximately associative.** Floating-point rounding gây sai lệch nhỏ.
5. **CJK support minimal.** Tất cả CJK → R=6, mất semantic granularity.

---

## §3 — KNOWTREE

### §3.1 — Data Structure

```julia
# src/KnowTree.jl

mutable struct KTNode
    pw::UInt16              # P_weight (molecule hash)
    fire_count::UInt16      # activation count
    chain::MolecularChain   # stored molecule sequence
    maturity::UInt8         # 0=Raw, 1=Evaluating, 2=Mature (QR)
    layer::UInt8            # 0=L0 (immutable), 1-3=higher
    edges::Vector{UInt32}   # Silk edge indices
    last_fire::Int64        # Unix ms timestamp
    shadow::ShadowVector    # 8-float contextual embedding
end

mutable struct KnowTree
    nodes::Vector{KTNode}
    buckets::Vector{Vector{Int}}   # 256 buckets, each = vector of node indices
    count::Int
    capacity::Int
end

function KnowTree(; capacity::Int=100_000)
    KnowTree(
        KTNode[],
        [Int[] for _ in 1:256],
        0,
        capacity
    )
end

# Bucket assignment: top 8 bits of P_weight
bucket_id(pw::UInt16)::Int = Int(pw >> 8) + 1  # 1-indexed
```

### §3.2 — Operations

```julia
function kt_store!(kt::KnowTree, chain::MolecularChain)::Int
    pw = isempty(chain) ? UInt16(0) : chain[1].pw
    bid = bucket_id(pw)
    
    # Check existing: exact chain match → increment fire_count
    for idx in kt.buckets[bid]
        node = kt.nodes[idx]
        if node.chain == chain
            node.fire_count += 1
            node.last_fire = time_ms()
            return idx
        end
    end
    
    # New node
    if kt.count >= kt.capacity
        kt_evict!(kt)  # Make room
    end
    
    node = KTNode(pw, 1, copy(chain), 0, 1, UInt32[], time_ms(),
                  ShadowVector())
    push!(kt.nodes, node)
    idx = length(kt.nodes)
    push!(kt.buckets[bid], idx)
    kt.count += 1
    idx
end

function kt_lookup(kt::KnowTree, chain::MolecularChain)::Union{Int, Nothing}
    pw = isempty(chain) ? UInt16(0) : chain[1].pw
    bid = bucket_id(pw)
    for idx in kt.buckets[bid]
        kt.nodes[idx].chain == chain && return idx
    end
    nothing
end

function kt_nearest(kt::KnowTree, query::Molecule; k::Int=10)::Vector{Int}
    bid = bucket_id(query.pw)
    candidates = [(idx, mol_dist(query, Molecule(kt.nodes[idx].pw)))
                  for idx in kt.buckets[bid]]
    # Also check adjacent buckets for boundary cases
    for adj in [bid-1, bid+1]
        1 <= adj <= 256 || continue
        for idx in kt.buckets[adj]
            push!(candidates, (idx, mol_dist(query, Molecule(kt.nodes[idx].pw))))
        end
    end
    sort!(candidates, by=x->x[2])
    [c[1] for c in candidates[1:min(k, length(candidates))]]
end
```

**Complexity:** O(bucket_size) per lookup. With 256 buckets and 100K nodes: ~400 nodes/bucket avg.
NOT O(1). Honest: O(n/256). Acceptable for 100K. For 1M+, need VP-Tree (§3.4).

### §3.3 — Eviction Strategy

```julia
function kt_evict!(kt::KnowTree; target::Int=0)
    # Evict least useful nodes (skip L0 = immutable)
    if target == 0
        target = kt.capacity ÷ 10  # Free 10% space
    end
    
    now = time_ms()
    scores = [(i, usefulness(kt.nodes[i], now))
              for i in eachindex(kt.nodes)
              if kt.nodes[i].layer > 0]  # Don't evict L0
    sort!(scores, by=x->x[2])
    
    evicted = 0
    for (idx, _) in scores
        evicted >= target && break
        kt_remove!(kt, idx)
        evicted += 1
    end
end

function usefulness(node::KTNode, now::Int64)::Float64
    recency_hours = (now - node.last_fire) / 3_600_000.0
    node.fire_count / (recency_hours + 1.0)
end
```

### §3.4 — VP-Tree Search

For KnowTree > 10K nodes, build VP-Tree over ShadowVectors for O(log n) nearest-k:

```julia
# src/VPTree.jl

struct VPNode
    point_idx::Int          # Index into KnowTree.nodes
    radius::Float64         # Median distance to children
    left::Union{VPNode, Nothing}   # Inside sphere
    right::Union{VPNode, Nothing}  # Outside sphere
end

function vptree_build(indices::Vector{Int}, shadows::Vector{ShadowVector})::Union{VPNode, Nothing}
    isempty(indices) && return nothing
    
    # Pick random vantage point
    vp_idx = indices[rand(1:length(indices))]
    rest = filter(!=(vp_idx), indices)
    isempty(rest) && return VPNode(vp_idx, 0.0, nothing, nothing)
    
    # Compute distances to vantage point
    dists = [(i, shadow_dist(shadows[vp_idx], shadows[i])) for i in rest]
    
    # Median split (Julia's partialsort! is O(n))
    mid = length(dists) ÷ 2 + 1
    partialsort!(dists, mid, by=x->x[2])
    radius = dists[mid][2]
    
    inside = [d[1] for d in dists[1:mid]]
    outside = [d[1] for d in dists[mid+1:end]]
    
    VPNode(vp_idx, radius,
           vptree_build(inside, shadows),
           vptree_build(outside, shadows))
end

function vptree_search(node::Union{VPNode, Nothing}, query::ShadowVector,
                       shadows::Vector{ShadowVector}, k::Int,
                       results::Vector{Tuple{Int, Float64}})
    node === nothing && return
    
    d = shadow_dist(query, shadows[node.point_idx])
    
    # Add current node as candidate
    if length(results) < k
        push!(results, (node.point_idx, d))
        sort!(results, by=x->x[2])
    elseif d < results[end][2]
        results[end] = (node.point_idx, d)
        sort!(results, by=x->x[2])
    end
    
    tau = length(results) < k ? Inf : results[end][2]
    
    # Search inside sphere if query might overlap
    if d < node.radius + tau
        vptree_search(node.left, query, shadows, k, results)
    end
    # Search outside sphere if query might overlap
    if d >= node.radius - tau
        vptree_search(node.right, query, shadows, k, results)
    end
end
```

---

## §4 — SILK (HEBBIAN LEARNING)

### §4.1 — Edge Structure

```julia
# src/Silk.jl

mutable struct SilkEdge
    from::UInt32            # KnowTree node index
    to::UInt32              # KnowTree node index
    weight::Float64         # [0.0, 1.0]
    fire_count::UInt16
    edge_kind::UInt8        # RelationOp (0-15) or learned (16-21)
    layer::UInt8            # 0=implicit, 1=learned, 2=structural
    last_fire::Int64        # Unix ms
    reward_sum::Float64     # Running sum for average
    reward_count::UInt16
end

mutable struct SilkGraph
    edges::Vector{SilkEdge}
    node_edges::Dict{UInt32, Vector{Int}}  # node_id → edge indices
end
```

### §4.2 — Co-activation (Learning)

```julia
function silk_co_activate!(sg::SilkGraph, a::UInt32, b::UInt32;
                           reward::Float64=1.0)
    edge = find_or_create_edge!(sg, a, b)
    
    # Hebbian update: Δw = reward × (1 - w) × η
    # η (learning rate) = 0.236 (φ⁻³ — design choice, consistent with φ family)
    η = 0.236
    Δw = reward * (1.0 - edge.weight) * η
    edge.weight = clamp(edge.weight + Δw, 0.0, 1.0)
    
    edge.fire_count += 1
    edge.last_fire = time_ms()
    edge.reward_sum += reward
    edge.reward_count += 1
end
```

⚠️ **HONEST:** Learning rate φ⁻³ = 0.236 là design choice consistent với φ family.
Jaeger (2022) dùng similar range cho reservoir computing. Nhưng KHÔNG phải "scientifically proven optimal".
Ghi: tunable, default 0.236.

### §4.3 — Decay Formula

**Stretched exponential decay:**

```julia
# w(t) = w₀ × exp(-(t/τ)^β)
# β = φ⁻¹ = 0.618034 (golden ratio inverse)
# τ = 78.37 hours (calibrated: w(24h) = 0.618 exactly)

const DECAY_BETA = 0.618034    # φ⁻¹
const DECAY_TAU  = 78.37       # hours

function silk_decay!(sg::SilkGraph, now::Int64)
    for edge in sg.edges
        edge.layer == 2 && continue  # Structural edges don't decay
        hours = (now - edge.last_fire) / 3_600_000.0
        hours <= 0 && continue
        edge.weight *= exp(-(hours / DECAY_TAU)^DECAY_BETA)
        # Prune dead edges
        if edge.weight < 0.001
            edge.weight = 0.0  # Mark for cleanup
        end
    end
end
```

**Decay behavior:**
- After 24h: weight × 0.618 (φ⁻¹)
- After 1 week: weight × 0.028
- After 1 month: weight ≈ 0

⚠️ **HONEST:** φ⁻¹ chosen for mathematical elegance (consistent φ family).
Real neuroscience (Ebbinghaus, Wickelgren) shows POWER-LAW fits better for long-term.
Stretched exponential is a COMPROMISE between pure exponential and power-law.
TODO Phase 4: Measure actual retention, compare φ⁻¹ vs power-law, tune.

### §4.4 — Implicit Layer

37 edge types computed from 5D distance — zero storage cost:

```julia
function silk_implicit_similarity(a::Molecule, b::Molecule)::Float64
    1.0 - mol_dist(a, b) / 70.0
end

# Implicit edges are NOT stored — computed on-the-fly
# Used when no explicit Silk edge exists between two nodes
```

### §4.5 — ShadowVector

8-dimensional contextual embedding, learned from co-activation via EMA:

```julia
# src/ShadowVector.jl

const SHADOW_DIM = 8

struct ShadowVector
    dims::NTuple{8, Float32}
end

ShadowVector() = ShadowVector(ntuple(_ -> Float32(0), 8))

function shadow_dist(a::ShadowVector, b::ShadowVector)::Float64
    sqrt(sum((Float64(a.dims[i]) - Float64(b.dims[i]))^2 for i in 1:SHADOW_DIM))
end

# Initialize from P_weight (5D → first 5 dims, last 3 = zero until learned)
function shadow_from_molecule(m::Molecule)::ShadowVector
    ShadowVector((
        Float32(shape(m)) / 15f0,
        Float32(relation(m)) / 15f0,
        Float32(valence(m)) / 7f0,
        Float32(arousal(m)) / 7f0,
        Float32(time(m)) / 3f0,
        0f0, 0f0, 0f0  # Contextual dims — learned via EMA
    ))
end

# EMA update when two nodes co-activate
function shadow_update!(sv::ShadowVector, other::ShadowVector; α::Float32=0.1f0)
    new_dims = ntuple(8) do i
        sv.dims[i] * (1f0 - α) + other.dims[i] * α
    end
    ShadowVector(new_dims)  # Note: return new, ShadowVector is immutable struct
end
```

---

## §5 — PIPELINE (8 STAGES)

### §5.1 — Stage Overview

Simplified from 15 stages to 8. Merged stages that don't truly depend on each other.

```
Input text
    │
    ▼
┌─── Stage 1: GATE ─────────────────────────────────┐
│ Security check. Reject harmful input.              │
│ IF fail → return "I cannot help with that."        │
└────────────────────┬───────────────────────────────┘
                     │ CP1: input validated
                     ▼
┌─── Stage 2: ENCODE ────────────────────────────────┐
│ text → MolecularChain via §2.4 formulas            │
│ Vietnamese: NFD + tone modulation                   │
│ Word-level: NRC-VAD lookup, segment compounds       │
└────────────────────┬───────────────────────────────┘
                     │
                     ▼
┌─── Stage 3: SEARCH + COMPOSE ──────────────────────┐
│ KnowTree nearest-k (§3.2)                          │
│ Homeostasis check: surprise = 1 - max_similarity    │
│   If surprise > 0.5 → increase learning rate        │
│ LCA compose nearest chains (§2.6)                   │
│ Silk weight lookup on edges                         │
└────────────────────┬───────────────────────────────┘
                     │ CP2: encode + search complete
                     ▼
┌─── Stage 4: INSTINCTS ────────────────────────────┐
│ Run 7 heuristics (§6) in order:                    │
│   System 1 (fast): Honesty, Contradiction          │
│   System 2 (slow): Causality, Abstraction,         │
│                     Analogy, Curiosity              │
│   Meta: Reflection                                  │
│ Normalize scores → [0,1], select top-3 branches    │
│ Immune selection: pick lowest entropy               │
└────────────────────┬───────────────────────────────┘
                     │
                     ▼
┌─── Stage 5: LEARN ─────────────────────────────────┐
│ Silk co-activate all pairs in response (§4.2)       │
│ Hebbian update: Δw = reward × (1-w) × 0.236        │
│ Fire count increment on touched KT nodes            │
│ Dream trigger check: fire_count ∈ Fibonacci?        │
└────────────────────┬───────────────────────────────┘
                     │ CP3: learning complete
                     ▼
┌─── Stage 6: DREAM (conditional) ───────────────────┐
│ Only runs if Fibonacci trigger hit (§7)             │
│ Cluster → LCA → validate → promote/reject           │
│ Skip if not triggered.                              │
└────────────────────┬───────────────────────────────┘
                     │
                     ▼
┌─── Stage 7: DECODE ────────────────────────────────┐
│ Response chain → text (§2.9)                        │
│ Apply tone/style from instinct selection            │
└────────────────────┬───────────────────────────────┘
                     │
                     ▼
┌─── Stage 8: FEEDBACK ──────────────────────────────┐
│ Await user reaction (implicit or explicit)          │
│ Positive: silk_co_activate with reward > 0          │
│ Negative: silk_co_activate with reward < 0          │
│ Timeout (no reaction): reward = 0.5 (neutral)      │
└────────────────────────────────────────────────────┘
```

### §5.2 — Checkpoints (3)

Reduced from 6 to 3. Only check at points where failure = fundamentally wrong path.

| CP | After | Checks | On Fail |
|----|-------|--------|---------|
| CP1 | GATE | Input not empty, not harmful | Return rejection |
| CP2 | SEARCH+COMPOSE | ≥1 nearest found, chain valid | Return "I don't know" |
| CP3 | LEARN | Silk update didn't corrupt state | Rollback to pre-learn snapshot |

### §5.3 — Timing Budget

Reference: LIDA cognitive cycle = 260-390ms for 9 steps.
Origin target: <500ms total for 8 stages on commodity hardware.

| Stage | Target | Notes |
|-------|--------|-------|
| GATE | <1ms | Simple keyword check |
| ENCODE | <10ms | 42 formulas per codepoint, cached after first encode |
| SEARCH+COMPOSE | <50ms | Bucket scan + LCA, VP-Tree if >10K nodes |
| INSTINCTS | <100ms | 7 formulas, each O(1) |
| LEARN | <10ms | Silk update, Hebbian |
| DREAM | <200ms | Only runs occasionally (Fibonacci trigger) |
| DECODE | <10ms | Inverted map lookup |
| FEEDBACK | async | Non-blocking, update on next cycle |
| **TOTAL** | **<400ms** | Without dream; <600ms with dream |

---

## §6 — INSTINCTS (7 HEURISTICS)

### §6.1 — Ordering (Kahneman S1/S2)

**Verified claim ✅:** Kahneman System 1/System 2 mapping is legitimate.
CLARION, LIDA, and CEUR 2022 (Conway-Smith & West) provide AI precedent.

| Phase | Instinct | System | Speed | Purpose |
|-------|----------|--------|-------|---------|
| Gate | Honesty | S1 (fast) | <1ms | Confidence check — can I answer? |
| Gate | Contradiction | S1 (fast) | <1ms | Internal conflict detection |
| Deliberate | Causality | S2 (slow) | <5ms | Temporal reasoning |
| Deliberate | Abstraction | S2 (slow) | <5ms | Cluster variance analysis |
| Deliberate | Analogy | S2 (slow) | <10ms | Vector arithmetic A:B::C:D |
| Deliberate | Curiosity | S2 (slow) | <1ms | Novelty detection |
| Meta | Reflection | Meta | <5ms | Self-evaluation |

**Honesty is ALWAYS first** (non-negotiable gate). If confidence < threshold → return "I don't know."
Remaining 6 compute scores in parallel, compete.

### §6.2 — Scoring & Competition

⚠️ **FIX from original spec:** Scores MUST be normalized to [0,1] before comparison.
Original spec had incomparable metrics (V distance vs cluster variance vs fire_count).

```julia
function run_instincts(input::MolecularChain, kt::KnowTree, sg::SilkGraph,
                       nearest::Vector{Int})::Vector{Tuple{Symbol, Float64}}
    results = Tuple{Symbol, Float64}[]
    
    # System 1 — Gate (sequential, must pass)
    honesty_score = instinct_honesty(input, kt, sg, nearest)
    push!(results, (:honesty, honesty_score))
    honesty_score < 0.3 && return results  # Gate: can't answer
    
    contradiction_score = instinct_contradiction(input, kt, nearest)
    push!(results, (:contradiction, contradiction_score))
    
    # System 2 — Deliberate (can run in parallel via @spawn)
    s2_scores = Dict{Symbol, Float64}()
    @sync begin
        Threads.@spawn s2_scores[:causality]   = instinct_causality(input, kt, sg, nearest)
        Threads.@spawn s2_scores[:abstraction]  = instinct_abstraction(input, kt, nearest)
        Threads.@spawn s2_scores[:analogy]      = instinct_analogy(input, kt, nearest)
        Threads.@spawn s2_scores[:curiosity]    = instinct_curiosity(input, kt, nearest)
    end
    for (k, v) in s2_scores
        push!(results, (k, v))
    end
    
    # Meta
    reflection_score = instinct_reflection(kt, sg)
    push!(results, (:reflection, reflection_score))
    
    # Sort by score, return all (pipeline decides what to do)
    sort!(results, by=x->x[2], rev=true)
end
```

### §6.3 — 7 Formulas

```julia
# 1. HONESTY — Can I answer with confidence?
function instinct_honesty(input::MolecularChain, kt::KnowTree,
                          sg::SilkGraph, nearest::Vector{Int})::Float64
    isempty(nearest) && return 0.0
    
    # Weighted factors (all normalized to [0,1])
    silk_w = mean_silk_weight(sg, nearest)           # avg edge weight
    fire = min(1.0, mean_fire_count(kt, nearest) / 20.0)  # normalized by 20 fires
    sources = min(1.0, count_unique_sources(kt, nearest) / 5.0)  # normalized by 5 sources
    consistency = chain_consistency(kt, nearest)      # how similar are nearest chains
    
    0.3 * silk_w + 0.3 * fire + 0.2 * sources + 0.2 * consistency
end

# 2. CONTRADICTION — Do my facts conflict?
function instinct_contradiction(input::MolecularChain, kt::KnowTree,
                                nearest::Vector{Int})::Float64
    length(nearest) < 2 && return 0.0
    
    max_conflict = 0.0
    for i in 1:length(nearest), j in (i+1):length(nearest)
        ni, nj = kt.nodes[nearest[i]], kt.nodes[nearest[j]]
        # High V distance + both high A = contradiction signal
        vdist = abs(Float64(valence(Molecule(ni.pw))) - Float64(valence(Molecule(nj.pw)))) / 7.0
        both_aroused = (arousal(Molecule(ni.pw)) >= 5 && arousal(Molecule(nj.pw)) >= 5) ? 1.0 : 0.0
        rdist = abs(Float64(relation(Molecule(ni.pw))) - Float64(relation(Molecule(nj.pw)))) / 15.0
        
        conflict = 0.4 * vdist + 0.3 * rdist + 0.3 * both_aroused
        max_conflict = max(max_conflict, conflict)
    end
    max_conflict
end

# 3. CAUSALITY — Temporal + co-activation + R type
function instinct_causality(input::MolecularChain, kt::KnowTree,
                            sg::SilkGraph, nearest::Vector{Int})::Float64
    length(nearest) < 2 && return 0.0
    
    score = 0.0
    for i in 1:min(5, length(nearest))
        node = kt.nodes[nearest[i]]
        # Condition 1: Temporal ordering (T dimension)
        temporal = time(Molecule(node.pw)) >= 2 ? 0.33 : 0.0
        # Condition 2: Co-activation in Silk
        coact = has_silk_edge(sg, nearest[1], nearest[i]) ? 0.33 : 0.0
        # Condition 3: R is CAUSES (0x09) or ORDER (0x03)
        r = relation(Molecule(node.pw))
        rtype = (r == 0x09 || r == 0x03) ? 0.34 : 0.0
        
        score = max(score, temporal + coact + rtype)
    end
    score  # Need ≥ 2/3 conditions for strong causality
end

# 4. ABSTRACTION — Cluster variance of nearest-10
function instinct_abstraction(input::MolecularChain, kt::KnowTree,
                              nearest::Vector{Int})::Float64
    length(nearest) < 3 && return 0.0
    
    # Variance of P_weight distances within cluster
    dists = Float64[]
    for i in 1:length(nearest), j in (i+1):length(nearest)
        push!(dists, Float64(mol_dist(Molecule(kt.nodes[nearest[i]].pw),
                                       Molecule(kt.nodes[nearest[j]].pw))) / 70.0)
    end
    var = isempty(dists) ? 0.0 : sum(dists) / length(dists)
    
    # Low variance = concrete, high = abstract
    # Thresholds: <0.15 concrete, <0.40 categorical, >=0.40 abstract
    # Normalize to [0,1]: abstraction score
    clamp(var / 0.5, 0.0, 1.0)
end

# 5. ANALOGY — A:B :: C:D via 5D vector arithmetic
function instinct_analogy(input::MolecularChain, kt::KnowTree,
                          nearest::Vector{Int})::Float64
    length(nearest) < 3 && return 0.0
    
    # Try to find A:B pattern, then project to C:?
    a, b, c = kt.nodes[nearest[1]], kt.nodes[nearest[2]], kt.nodes[nearest[3]]
    ma, mb, mc = Molecule(a.pw), Molecule(b.pw), Molecule(c.pw)
    
    # Delta: what changes from A→B?
    ds = Int(shape(mb)) - Int(shape(ma))
    dr = Int(relation(mb)) - Int(relation(ma))
    dv = Int(valence(mb)) - Int(valence(ma))
    da = Int(arousal(mb)) - Int(arousal(ma))
    dt = Int(time(mb)) - Int(time(ma))
    
    # Apply to C
    d = mol_pack(
        UInt8(clamp(Int(shape(mc)) + ds, 0, 15)),
        UInt8(clamp(Int(relation(mc)) + dr, 0, 15)),
        UInt8(clamp(Int(valence(mc)) + dv, 0, 7)),
        UInt8(clamp(Int(arousal(mc)) + da, 0, 7)),
        UInt8(clamp(Int(time(mc)) + dt, 0, 3))
    )
    
    # Score: how well does D match something in KnowTree?
    nearest_d = kt_nearest(kt, d, k=1)
    isempty(nearest_d) ? 0.0 : mol_similarity(d, Molecule(kt.nodes[nearest_d[1]].pw))
end

# 6. CURIOSITY — Novelty = 1 - nearest_similarity
function instinct_curiosity(input::MolecularChain, kt::KnowTree,
                            nearest::Vector{Int})::Float64
    isempty(nearest) && return 1.0  # Everything is novel if KT empty
    best_sim = maximum(mol_similarity(input[1], Molecule(kt.nodes[i].pw)) for i in nearest)
    1.0 - best_sim  # High novelty = high curiosity
end

# 7. REFLECTION — Self-evaluation: how good is my knowledge?
function instinct_reflection(kt::KnowTree, sg::SilkGraph)::Float64
    kt.count == 0 && return 0.0
    
    # QR ratio: what fraction of knowledge is mature?
    qr_count = count(n -> n.maturity == 2, kt.nodes)
    qr_ratio = qr_count / kt.count
    
    # Connectivity: how well-connected is the Silk graph?
    edge_count = length(sg.edges)
    connectivity = min(1.0, edge_count / (kt.count * 2.0))  # Normalize by 2× nodes
    
    0.6 * qr_ratio + 0.4 * connectivity
end
```

---

## §7 — DREAM CYCLE

**Trigger:** Fibonacci STM count {2, 3, 5, 8, 13, 21, 34, 55}

```julia
# src/Dream.jl

const FIBONACCI_TRIGGERS = Set([2, 3, 5, 8, 13, 21, 34, 55])

function should_dream(stm_count::Int)::Bool
    stm_count in FIBONACCI_TRIGGERS
end

function dream_cycle!(kt::KnowTree, sg::SilkGraph)
    # 1. Select top-10 most active nodes
    active = sort(collect(1:length(kt.nodes)),
                  by=i->kt.nodes[i].fire_count, rev=true)
    top10 = active[1:min(10, length(active))]
    
    # 2. Cluster by LCA similarity (threshold 0.3)
    clusters = cluster_by_similarity(kt, top10, threshold=0.3)
    
    for cluster in clusters
        length(cluster) < 2 && continue
        
        # 3. Compute LCA representative
        chains = [kt.nodes[i].chain for i in cluster]
        weights = [Float64(kt.nodes[i].fire_count) for i in cluster]
        representative = chain_lca(chains, weights ./ sum(weights))
        
        # 4. Score
        freq = mean(kt.nodes[i].fire_count for i in cluster) / 20.0
        conn = mean(length(kt.nodes[i].edges) for i in cluster) / 5.0
        emotion = mean(Float64(arousal(Molecule(kt.nodes[i].pw))) for i in cluster) / 7.0
        score = 0.3 * freq + 0.4 * conn + 0.3 * emotion
        
        # 5. If score >= 0.5: create proposal
        if score >= 0.5
            new_idx = kt_store!(kt, representative)
            # Co-activate with all cluster members
            for i in cluster
                silk_co_activate!(sg, UInt32(new_idx), UInt32(i), reward=score)
            end
        end
    end
end

# QR Promotion check (called periodically)
function check_promotion!(kt::KnowTree, sg::SilkGraph, idx::Int)
    node = kt.nodes[idx]
    node.maturity >= 2 && return  # Already QR
    
    # BCM adaptive threshold
    avg_fire = mean(n.fire_count for n in kt.nodes)
    theta = clamp(avg_fire^1.5 / 100.0, 0.3, 0.95)
    
    # Conditions for promotion:
    silk_w = mean_edge_weight(sg, UInt32(idx))
    fib_depth = findfirst(f -> node.fire_count >= f, [2,3,5,8,13,21,34,55])
    
    if silk_w >= theta && fib_depth !== nothing && fib_depth >= 3
        node.maturity = 2  # QR promoted
    end
end
```

⚠️ **HONEST NOTE on Dream validation:**
Dream clusters from domain D, then validates in same domain = potentially circular.
MITIGATION: Cross-validate with instinct_contradiction. If dream hypothesis contradicts
existing QR knowledge, reject it.
TODO Phase 4: Implement cross-domain validation.

---

## §8 — MEMORY MODEL (3-LAYER)

```julia
# src/Memory.jl

# Layer 1: STM (Short-Term Memory)
# Pure exponential decay, half-life 4 hours
# Implementation: ring buffer of recent chains, auto-evict after 4h
const STM_HALFLIFE_MS = 4 * 3_600_000  # 4 hours

# Layer 2: Silk (Medium-Term)
# Stretched exponential, half-life ~24h, long tail ~1 week
# See §4.3 for formula

# Layer 3: QR (Long-Term / Permanent)
# No decay. Once promoted (§7 check_promotion!), stays forever.
# Only removed by explicit contradiction or user command.

mutable struct MemorySystem
    stm::Vector{Tuple{MolecularChain, Int64}}  # (chain, timestamp)
    kt::KnowTree                                # Layer 2+3 storage
    sg::SilkGraph                                # Layer 2 connections
end

function memory_tick!(mem::MemorySystem, now::Int64)
    # STM cleanup
    filter!(entry -> (now - entry[2]) < STM_HALFLIFE_MS * 3, mem.stm)
    
    # Silk decay
    silk_decay!(mem.sg, now)
end
```

---

## §9 — CONFIDENCE TYPE

```julia
# src/Confidence.jl

struct Confidence <: Real
    value::Float64
    
    function Confidence(v::Real)
        new(clamp(Float64(v), 0.0, 1.0))
    end
end

# Truthiness — THIS IS THE RESOLUTION of the bool/confidence debate
Base.Bool(c::Confidence) = c.value > 0.5
# ↑ threshold 0.5 is DEFAULT. Can be overridden per-context.

# Łukasiewicz logic (min/max)
Base.:(&)(a::Confidence, b::Confidence) = Confidence(min(a.value, b.value))
Base.:(|)(a::Confidence, b::Confidence) = Confidence(max(a.value, b.value))
Base.:(!)(c::Confidence) = Confidence(1.0 - c.value)

# Comparisons return Confidence, not Bool
Base.:(==)(a::Confidence, b::Confidence) = Confidence(1.0 - abs(a.value - b.value))
Base.:(<)(a::Confidence, b::Confidence) = Confidence(max(0.0, b.value - a.value))

# Arithmetic (for weighted combinations)
Base.:(+)(a::Confidence, b::Confidence) = Confidence(a.value + b.value)
Base.:(*)(a::Confidence, b::Real) = Confidence(a.value * Float64(b))
Base.:(*)(a::Real, b::Confidence) = Confidence(Float64(a) * b.value)

# Display
Base.show(io::IO, c::Confidence) = print(io, "Conf(", round(c.value, digits=3), ")")

# Convert from comparison results
Base.convert(::Type{Confidence}, x::Bool) = Confidence(x ? 1.0 : 0.0)
Base.convert(::Type{Confidence}, x::Float64) = Confidence(x)
```

---

## §10 — SELF-MODIFICATION (SAFE)

Julia's metaprogramming = safe self-modification built-in.

```julia
# src/SelfModify.jl

struct ModifyResult
    success::Bool
    old_code::String
    new_code::String
    error::Union{String, Nothing}
end

function safe_modify(func_name::Symbol, new_body::String)::ModifyResult
    old_code = string(methods(getfield(Main, func_name)).ms[1])
    
    # Step 1: Parse — catches syntax errors BEFORE eval
    parsed = try
        Meta.parse("function $func_name $(new_body) end")
    catch e
        return ModifyResult(false, old_code, new_body, "Parse error: $e")
    end
    
    # Step 2: Type-check the AST (basic validation)
    if parsed.head != :function
        return ModifyResult(false, old_code, new_body, "Not a function definition")
    end
    
    # Step 3: Eval in sandbox module first
    sandbox = Module(:Sandbox)
    try
        Core.eval(sandbox, parsed)
    catch e
        return ModifyResult(false, old_code, new_body, "Eval error in sandbox: $e")
    end
    
    # Step 4: If sandbox passed, eval in real module
    try
        eval(parsed)
        ModifyResult(true, old_code, string(new_body), nothing)
    catch e
        ModifyResult(false, old_code, new_body, "Eval error: $e")
    end
end
```

**So với C spec:**
- C: string replace → file write → compile → hope → rollback via file copy
- Julia: parse → validate AST → sandbox eval → real eval. No file I/O. No race condition.

---

## §11 — PERSISTENCE & STATE

```julia
# src/Persist.jl

using Serialization

function origin_save(mem::MemorySystem, path::String)
    # Atomic write: write to temp, then rename
    tmp = path * ".tmp"
    open(tmp, "w") do io
        serialize(io, mem)
    end
    mv(tmp, path, force=true)  # Atomic rename on POSIX
end

function origin_load(path::String)::MemorySystem
    open(path) do io
        deserialize(io)
    end
end
```

For production: Consider JLD2.jl for portable, versioned serialization.

---

## §12 — JULIA PROJECT STRUCTURE

```toml
# Project.toml
name = "Origin"
uuid = "..."  # Generate with UUIDs.uuid4()
version = "0.1.0"

[deps]
Unicode = "4ec0a83e-493e-50e2-b9ac-8f72acf5a8f5"

[extras]
Test = "8dfed614-e22c-5e08-85e1-65c5234f0b40"

[targets]
test = ["Test"]
```

**Zero external dependencies for core.** Only `Unicode` from stdlib.
Optional deps for tooling: `JLD2` (persistence), `REPL` (interactive).

---

## §13 — IMPLEMENTATION PHASES

### Phase 1: Molecular Engine (1-2 tuần)

**Files:** `Molecule.jl`, `Chain.jl`, `NrcVad.jl`, `Vietnamese.jl`
**Build:** `build/build_unicode_flags.jl`
**Test:** `test/test_molecule.jl`

Deliverable: `encode("xin chào")` → chain, `decode(chain)` → text, `mol_dist` works.

Tasks:
1. Implement `Molecule` struct + pack/unpack
2. Implement distance metric
3. Run `build_unicode_flags.jl` to generate flags
4. Implement 5 encode functions (f_S, f_R, f_V, f_A, f_T)
5. Load NRC-VAD lexicon
6. Implement Vietnamese pipeline (NFD + tones)
7. Implement MolecularChain + hash + similarity
8. Implement decode (inverted map)
9. Test: encode/decode round-trip, distance properties, Vietnamese tones

### Phase 2: KnowTree + Silk (2-3 tuần)

**Files:** `KnowTree.jl`, `Silk.jl`, `ShadowVector.jl`, `VPTree.jl`
**Test:** `test/test_knowtree.jl`, `test/test_silk.jl`

Deliverable: Store facts, retrieve nearest, learn from co-activation, decay over time.

Tasks:
1. Implement KTNode + KnowTree with 256 buckets
2. Implement kt_store!, kt_lookup, kt_nearest
3. Implement eviction strategy
4. Implement SilkEdge + SilkGraph
5. Implement silk_co_activate! (Hebbian)
6. Implement silk_decay! (stretched exponential)
7. Implement ShadowVector + shadow_from_molecule
8. Implement VP-Tree (build + search) using partialsort!
9. Test: store/retrieve, learning curve, decay over simulated time

### Phase 3: Pipeline + Instincts + Dream (2-3 tuần)

**Files:** `Pipeline.jl`, `Instincts.jl`, `Dream.jl`, `Memory.jl`, `Confidence.jl`
**Test:** `test/test_pipeline.jl`, `test/test_instincts.jl`, `test/test_dream.jl`

Deliverable: Full cognitive loop — input text → process → output text, learning from interaction.

Tasks:
1. Implement Confidence type
2. Implement 8-stage pipeline
3. Implement 7 instinct formulas with normalized scoring
4. Implement dream cycle (Fibonacci trigger, clustering, promotion)
5. Implement 3-layer memory model
6. Implement 3 checkpoints
7. Test: end-to-end pipeline, instinct ordering, dream produces valid hypotheses

### Phase 4: Self-Modify + Persist + REPL (1-2 tuần)

**Files:** `SelfModify.jl`, `Persist.jl`, `repl/origin_repl.jl`
**Test:** `test/test_confidence.jl`

Deliverable: Interactive system that persists state, can modify its own behavior.

Tasks:
1. Implement safe_modify (parse → sandbox → eval)
2. Implement origin_save / origin_load (atomic writes)
3. Build REPL interface
4. End-to-end integration test

### Phase 5: Measure & Tune (1-2 tuần)

No new code — experiments only.

Tasks:
1. **Measure collision rate:** Encode 10K words, count unique P_weights, report collision %
2. **Measure distance weights:** Encode word pairs with known similarity, tune (1,1,2,2,4)
3. **Measure decay:** Teach 100 facts, simulate 24h/1week/1month, compare φ⁻¹ vs power-law
4. **Measure dream quality:** After 1000 interactions, count dream proposals accepted vs rejected
5. **Measure instinct accuracy:** Present known reasoning tasks, check instinct scores match expected

---

## §14 — PATTERNS HỌC TỪ CLAUDE CLI

Claude CLI (512K LOC TypeScript) có nhiều patterns áp dụng được cho Origin:

### 14.1 — Tool Loop as State Machine

Claude CLI's QueryEngine: `submitMessage()` → model call → tool execution → recovery → repeat.
7 transition types, explicit state machine.

**Áp dụng:** Origin Pipeline = state machine. 8 stages = 8 states. Transitions explicit.
Không dùng recursion. Không dùng implicit control flow.

### 14.2 — Layered Recovery

Claude CLI: 4 layers (collapse → compact → escalate → multi-turn).
Mỗi layer thử trước khi escalate.

**Áp dụng:** Origin Pipeline checkpoints = recovery layers.
CP1 fail → reject. CP2 fail → "I don't know". CP3 fail → rollback.
Không crash. Không silent failure.

### 14.3 — Forked Subagents for Background Work

Claude CLI spawns AI agents cho: memory extraction, dream consolidation, context compression.
Non-blocking. Background.

**Áp dụng:** Origin Dream = background Task.
```julia
# Dream runs in background, doesn't block pipeline
@spawn dream_cycle!(kt, sg)
```

### 14.4 — Memory System (Persistent + Session)

Claude CLI: 2-tier memory.
- Session memory: temporary, current conversation
- Persistent memory: file-based, survives restarts

**Áp dụng:** Origin 3-layer = same concept:
- STM = session memory (ring buffer, 4h expiry)
- Silk = medium-term (learned associations, decay)
- QR = persistent memory (promoted facts, no decay)

### 14.5 — Permission Modes (Conservative Default)

Claude CLI: 5 modes from full-manual to fully-automated.
Default = ask user. Auto = ML classifier with circuit breaker.

**Áp dụng:** Origin SecurityGate = permission check.
Default: conservative. Self-modify: requires explicit approval.
L0 layer: immutable, never modified.

### 14.6 — Streaming-First Architecture

Claude CLI: Tools execute while model streams. Results streamed back realtime.

**Áp dụng:** Origin Pipeline: encode starts while user is still typing.
Julia Channels enable streaming between stages.

```julia
# Pipelined execution
encode_ch = Channel{MolecularChain}(1)
@spawn put!(encode_ch, encode_text(input))
chain = take!(encode_ch)  # Ready by the time we need it
```

---

## §15 — QUYẾT ĐỊNH ĐÃ ĐÓNG

**KHÔNG MỞ LẠI.** Mọi quyết định dưới đây đã được verify, có lý do, và session sau PHẢI tuân theo.

| # | Quyết định | Chọn | Lý do | Verify status |
|---|-----------|------|-------|---------------|
| D1 | S compose | MAX (union) | S = category index, dominant wins. Nhất quán Rust + 4 tài liệu gốc | ✅ Đóng |
| D2 | Bool type | Confidence <: Real | Float [0,1], threshold 0.5 cho truthiness. Resolve C spec contradiction | ✅ Đóng |
| D3 | Homeostasis | 0.5 tunable | ACT-R/Soar pattern. 0.618 = φ⁻¹ option nhưng 0.5 đơn giản hơn | ✅ Đóng |
| D4 | Decay model | Stretched exp β=φ⁻¹ | Compromise exponential + power-law. τ=78.37h calibrated | ✅ Đóng |
| D5 | QR promotion | BCM adaptive | theta tự điều chỉnh: avg^1.5, clamp [0.3, 0.95] | ✅ Đóng |
| D6 | Learning rate | η=0.236 (φ⁻³) | Design choice, consistent φ family. Tunable. | ✅ Đóng |
| D7 | Encode approach | FORMULA (TÍNH) | Runtime compute từ Unicode properties, không lookup table | ✅ Đóng |
| D8 | Pipeline stages | 8 (merged from 15) | Giảm complexity, merge non-dependent stages | ✅ MỚI |
| D9 | Checkpoints | 3 (reduced from 6) | Chỉ check tại failure = wrong path | ✅ MỚI |
| D10 | Instinct scoring | Normalized [0,1] | Fix incomparable metrics issue | ✅ MỚI |
| D11 | Runtime | Julia | Brain > binary size. Ship > perfection. | ✅ MỚI |
| D12 | Distance weights | (1,1,2,2,4) TUNABLE | Papers không support cụ thể. Cần measure. | ✅ MỚI |
| D13 | Similarity weights | TUNABLE, no citation | Russell/Zajonc KHÔNG nói con số. Engineering default. | ✅ MỚI |
| D14 | Consolidation naming | "idle-phase scheduling" | KHÔNG gọi "circadian". Born 2012 = *Psych Research*. | ✅ MỚI |
| D15 | BuildZone confidence | 0.90 TUNABLE | NARS default ~0.6. 0.90 = engineering choice. | ✅ MỚI |
| D16 | Instinct ordering | Kahneman S1→S2→Meta | CLARION/LIDA/CEUR 2022 precedent. Claim mạnh nhất. | ✅ Verified |

---

## §16 — BLOCKING ISSUES & DEPENDENCIES

### Must resolve BEFORE Phase 1:

| # | Issue | Blocker for | Resolution | Effort |
|---|-------|------------|------------|--------|
| B1 | `build_unicode_flags.jl` chưa viết | Encode pipeline | Write Julia script, parse UnicodeData.txt | 2-3h |
| B2 | NRC-VAD data file location | Valence/Arousal encoding | Copy from ~/Origin/src/nrc_vad_data.h hoặc download TSV gốc | 30min |
| B3 | Vietnamese compound dictionary | Word segmentation | Acquire or build 30K compound list | 1-2h |

### Must resolve BEFORE Phase 2:

| # | Issue | Blocker for | Resolution | Effort |
|---|-------|------------|------------|--------|
| B4 | ShadowVector dims[5:8] = 0 forever | Silk learning quality | Initialize from P_weight hash at startup | 1h |
| B5 | KnowTree capacity planning | Memory usage | Start with 100K, doubling array, Julia handles realloc | Design decision |

### Must resolve BEFORE Phase 3:

| # | Issue | Blocker for | Resolution | Effort |
|---|-------|------------|------------|--------|
| B6 | Dream cross-validation | Dream quality | Add instinct_contradiction check before accepting | 2h |
| B7 | Feedback mechanism | Pipeline Stage 8 | Define explicit feedback API (user says good/bad) | 2h |

### DEFER to Phase 5+:

| # | Issue | Why defer |
|---|-------|-----------|
| D-1 | Concurrency (spawn/channel/select language features) | Julia Tasks handle this natively |
| D-2 | Self-modification beyond parameter tuning | Need working brain first |
| D-3 | CJK proper support (radicals, tones) | MVP = Vietnamese + English |
| D-4 | Module system for Olang DSL | Use Julia modules directly for now |
| D-5 | "Consciousness" claims | Drop entirely. System is metacognitive, not conscious. |

---

## APPENDIX A — REFERENCE CITATIONS (VERIFIED)

| Citation | Correct Reference | Used For |
|----------|-------------------|----------|
| Born & Wilhelm 2012 | *Psychological Research* 76, 192-203. DOI: 10.1007/s00426-011-0335-6 | Dream consolidation metaphor |
| Kahneman 2011 | *Thinking, Fast and Slow*. Farrar, Straus and Giroux | Instinct ordering S1/S2 |
| Conway-Smith & West 2022 | CEUR Workshop Proceedings, Vol-3332 | S1/S2 in AI architectures |
| Wang (NARS) | "Non-Axiomatic Reasoning System" — truth values ⟨f, c⟩ | Confidence model inspiration |
| Ebbinghaus 1885 | *Über das Gedächtnis* | Memory decay baseline |
| Wickelgren 1974 | *J. Mathematical Psychology* 11, 173-186 | Stretched exponential decay |
| Hebb 1949 | *Organization of Behavior* | Hebbian learning rule |
| Russell 1980 | "A circumplex model of affect" — *J. Personality & Social Psychology* | V/A dimensional model (NOT weight justification) |
| NRC-VAD v2 | Mohammad 2018, updated 2025 — 44,728 entries | Emotion lexicon data |

## APPENDIX B — GLOSSARY

| Term | Definition |
|------|-----------|
| Molecule | 16-bit UInt16 encoding 5 semantic dimensions |
| P_weight | Same as Molecule.pw — the raw UInt16 value |
| Chain | Vector{Molecule} — sequence of encoded codepoints |
| KnowTree | Hash-indexed fact storage (256 buckets) |
| Silk | Hebbian learning graph — weighted edges between KT nodes |
| QR | "Qualified Rule" — promoted mature knowledge (permanent) |
| STM | Short-term memory — ring buffer, 4h expiry |
| LCA | Lowest Common Ancestor — compose operation for molecules |
| VP-Tree | Vantage Point Tree — O(log n) nearest-neighbor search |
| ShadowVector | 8-float contextual embedding learned via EMA |
| BCM | Bienenstock-Cooper-Munro — adaptive threshold model |
| IWCM | Incremental Word Co-occurrence Matrix |
| φ | Golden ratio ≈ 1.618034 |
| φ⁻¹ | 0.618034 — used for decay rate |
| φ⁻³ | 0.236068 — used for learning rate |

---

*Document end. Mọi session sau đọc file này trước khi code.*
