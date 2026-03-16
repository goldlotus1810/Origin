# Olang — Ngon ngu cua HomeOS

> Olang la ngon ngu lap trinh, suy luan, va sang tao cua HomeOS.
> Moi thu la MolecularChain. Moi phep toan la bien doi chain.

---

## Muc luc

1. [Co ban — Nodes & Compose](#1-co-ban)
2. [Relations — 18 lien ket](#2-relations)
3. [Control flow — if, loop, fn](#3-control-flow)
4. [Arithmetic — QT3 Axiom](#4-arithmetic)
5. [Reasoning — suy luan](#5-reasoning)
6. [Debugging — debug](#6-debugging)
7. [Structure Analysis — phan tich](#7-structure-analysis)
8. [Programming — lap trinh](#8-programming)
9. [Creative Writing — van & tho](#9-creative-writing)
10. [Reference — tham chieu day du](#10-reference)

---

## 1. Co ban

### Node lookup
```olang
fire
```
Tra `fire` trong Registry -> MolecularChain. Moi tu la alias cua 1 Unicode codepoint.

### Compose (LCA)
```olang
fire ∘ water
```
Tim to tien chung gan nhat (Lowest Common Ancestor) -> chain cha.

### Emit (output)
```olang
emit fire;
○ fire;
```
`emit` va `○` tuong duong — xuat chain ra ngoai.

### Let (gan bien)
```olang
let steam = fire ∘ water;
steam ≔ fire ∘ water;
```
Gan ket qua vao bien cuc bo `steam`.

### String literal
```olang
emit "toi buon vi mat viec";
```
String lit -> registry alias lookup.

---

## 2. Relations — 18 RelOps

### Tao edge (relation giua 2 nodes)
```olang
fire ∈ elements        -- fire la thanh vien cua elements
fire → heat            -- fire gay ra heat
fire ≈ sun             -- fire tuong tu sun
fire ⊂ energy          -- fire la tap con cua energy
fire ≡ flame           -- fire dong nhat voi flame
```

### Query (tim nodes co relation)
```olang
fire ∈ ?               -- tim cai gi chua fire?
fire → ?               -- fire gay ra cai gi?
? → fire               -- cai gi gay ra fire?
```

### Chain query (multi-hop)
```olang
🌞 → ? → 🌵            -- mat troi -> [gi do] -> xuong rong
fire → ? → ? → water   -- 3 buoc tu fire den water
```

### Bang day du 18 RelOps

| Ky hieu | Ten       | Byte | Y nghia                    |
|---------|-----------|------|----------------------------|
| `∈`     | Member    | 0x01 | A la thanh vien cua B      |
| `⊂`     | Subset    | 0x02 | A la tap con cua B         |
| `≡`     | Equiv     | 0x03 | A dong nhat B              |
| `⊥`     | Ortho     | 0x04 | A vuong goc / doc lap B    |
| `∘`     | Compose   | 0x05 | LCA(A, B)                  |
| `→`     | Causes    | 0x06 | A gay ra B                 |
| `≈`     | Similar   | 0x07 | A tuong tu B               |
| `←`     | Derived   | 0x08 | A duoc dan xuat tu B       |
| `∪`     | Contains  | 0x09 | A chua B (khong gian)      |
| `∩`     | Intersects| 0x0A | A giao B                   |
| `∖`     | SetMinus  | 0x0B | A tru B                    |
| `↔`     | Bidir     | 0x0C | A <-> B hai chieu          |
| `⟶`     | Flows     | 0x0D | A chay den B (thoi gian)   |
| `⟳`     | Repeats   | 0x0E | A lap lai chu ky B         |
| `↑`     | Resolves  | 0x0F | A giai quyet tai B         |
| `⚡`    | Trigger   | 0x10 | A kich hoat B              |
| `∥`     | Parallel  | 0x11 | A song song B              |
| `∂`     | Context   | none | A trong ngu canh B (LCA)   |

---

## 3. Control Flow

### If / else
```olang
if fire {
    emit "nong qua!";
} else {
    emit "lanh qua!";
}

-- Unicode syntax:
fire ⇒ {
    ○ "nong qua!";
} ⊥ {
    ○ "lanh qua!";
}
```

### Loop
```olang
loop 5 {
    emit fire;
}

-- Unicode syntax:
↻ 5 {
    ○ fire;
}
```

### Function
```olang
fn blend(a, b) {
    let result = a ∘ b;
    emit result;
}

blend(fire, water);
```

---

## 4. Arithmetic — QT3 Axiom

QT3 phan biet 3 muc do chan ly:

### Hypothesis (gia thuyet — chua chung minh)
```olang
1 + 2           -- __hyp_add: chua chac dung
x - y           -- __hyp_sub
3 × 4           -- __hyp_mul
10 ÷ 2          -- __hyp_div
```

### Physical (vat ly — da chung minh)
```olang
mass_a ⧺ mass_b  -- __phys_add + FUSE: da do, da chung minh
total ⊖ used      -- __phys_sub + FUSE
```
Physical ops tu dong goi `FUSE` (QT2: kiem tra huu han).

### Truth (chan ly — chac chan)
```olang
fire == flame     -- __assert_truth: day la su that
```

---

## 5. Reasoning — Suy luan

### Suy luan nhan qua (Causality)
```olang
-- "Mat troi chieu -> xuong rong lon"
🌞 → growth;
growth → 🌵;
-- Hoi: mat troi den xuong rong qua gi?
🌞 → ? → 🌵;
```

### Suy luan tuong tu (Analogy)
```olang
-- A:B :: C:?
-- "fire la gi cua heat, thi ice la gi cua cold?"
let cause_heat = fire ∘ heat;
let analogy = ice ∘ cold;
-- LCA cua 2 cap -> cung loai
cause_heat ∘ analogy;
```

### Suy luan bao ham (Abstraction)
```olang
-- Tu cu the -> truu tuong
fire ⊂ energy;
water ⊂ matter;
-- LCA cua energy va matter?
energy ∘ matter;  -- -> "physical_world" (truu tuong hon)
```

### Phat hien mau thuan (Contradiction)
```olang
-- Hai dieu khong the cung dung
hot ⊥ cold;           -- vuong goc / xung dot
if hot {
    assert cold;       -- ASSERT FAILED: mau thuan!
}
```

### Kiem tra su that (Honesty)
```olang
-- Chi noi khi du bang chung
fire == flame;         -- truth assertion
assert fire;           -- co ton tai khong?
```

---

## 6. Debugging

### Trace — theo doi thuc thi
```olang
trace;                 -- bat/tat trace mode
emit fire ∘ water;     -- moi buoc se in: [trace pc=N op=OP stack=S]
```

### Inspect — xem cau truc chain
```olang
inspect fire;
-- Output: [inspect hash=0x... molecules=1 bytes=5 empty=false]
```

### Assert — kiem tra dieu kien
```olang
assert fire;           -- OK: fire ton tai (non-empty)
assert unknown_thing;  -- [ASSERT FAILED: chain is empty]
```

### TypeOf — phan loai chain
```olang
typeof fire;
-- Output: [typeof 0xABCD = SDF]
typeof fire ∘ water;
-- Output: [typeof 0x1234 = Mixed(SDF+EMOTICON)]
```

### Explain — truy nguon goc
```olang
explain fire;
-- Output: [explain origin of 0x...]
-- Hoi: tai sao chain nay ton tai? Tu UCD nao? Ai tao?
```

### Stats — thong ke he thong
```olang
stats;
-- In: so nodes, edges, STM size, Silk density...
```

---

## 7. Structure Analysis — Phan tich cau truc

### Phan tich quan he
```olang
-- Tim tat ca thanh vien
elements ∈ ?;          -- query: elements chua gi?

-- Tim tat ca nguyen nhan
sadness → ?;           -- sadness gay ra gi?

-- Tim tat ca nguon goc
? → happiness;         -- cai gi gay ra happiness?
```

### Phan tich da buoc
```olang
-- Tu A den B qua moi gian
A → ? → ? → B;

-- Ket hop relation types
A ∈ ? → ? ≈ B;        -- A thuoc gi, cai do gay ra gi tuong tu B?
```

### Context analysis
```olang
-- Tim ngu canh chung
bank ∂ finance;        -- "bank" trong ngu canh "finance" -> tai chinh
bank ∂ river;          -- "bank" trong ngu canh "river" -> bo song
```

### So sanh chains
```olang
let a = fire ∘ heat;
let b = sun ∘ warmth;
-- So sanh: LCA cua 2 chain
a ∘ b;                 -- tim diem chung
a == b;                -- truth check: giong nhau khong?
```

---

## 8. Programming — Lap trinh voi Olang

### Hello World
```olang
emit "Hello, HomeOS!";
```

### Tim kiem va loc
```olang
fn find_causes(thing) {
    emit thing → ?;
}

find_causes(fire);
find_causes(rain);
```

### Xay dung knowledge
```olang
-- Dinh nghia quan he moi
fire ∈ elements;
water ∈ elements;
earth ∈ elements;
air ∈ elements;

-- Tao lien ket
fire → heat;
water → flow;
fire ⊥ water;

-- Truu tuong hoa
let nature = fire ∘ water ∘ earth ∘ air;
emit nature;
```

### Conditional logic
```olang
fn check_safe(input) {
    assert input;                -- phai ton tai
    if input ∈ ? {
        emit "da biet — co trong he thong";
    } else {
        emit "chua biet — can hoc";
        learn input;
    }
}
```

### Iterative processing
```olang
fn reinforce(concept) {
    -- Lap 5 lan de tang Hebbian weight
    loop 5 {
        emit concept;
        concept ∘ concept;       -- self-compose
    }
}

reinforce(fire);
```

### Dream cycle
```olang
-- Hoc xong, trigger consolidation
learn "toi yeu nhac";
learn "nhac lam toi vui";
learn "vui la cam xuc tot";
dream;                          -- STM -> cluster -> QR
```

---

## 9. Creative Writing — Van & Tho

### Van xuoi — Olang lam "nao" xu ly
```olang
-- HomeOS hoc van:
learn "Buoi sang thuc day, nhin qua cua so, mat troi len.";
learn "Anh sang vang roi xuong manh dat, am ap nhu vong tay me.";

-- HomeOS phan tich:
-- sentence_affect -> V=+0.45, A=0.30
-- "am ap" + "me" co_activate strong
-- ConversationCurve -> Reinforcing
```

### Tho — cam xuc amplify qua Silk
```olang
-- Tho la chuoi cam xuc, moi tu -> chain, moi chain -> emotion
learn "Dem nay trang sang";
learn "Nho ai noi dau day";
learn "Gio lua mat canh hoa";
learn "Roi vao long ta buon";

-- Silk amplification:
-- "dem" + "trang" co_activate V=+0.10
-- "nho" + "dau" co_activate V=-0.55
-- "buon" + "nho" + "dem" -> composite V=-0.65 (AMPLIFY, khong trung binh)
-- ConversationCurve: f' < -0.15 -> Supportive tone
```

### Viet tho bang Olang
```olang
fn poem(line1, line2, line3, line4) {
    learn line1;
    learn line2;
    learn line3;
    learn line4;
    -- Moi dong -> learn -> STM -> Silk connections
    -- Cam xuc tu dong amplify qua co-activation
    -- Dream de consolidate thanh ky uc dai han
    dream;
}

poem(
    "Mat troi len",
    "Hoa no ven duong",
    "Gio thoang huong thom",
    "Long ta nhe nhang"
);
```

### Narrative structure
```olang
-- Van ban co cau truc: nhan vat -> hanh dong -> ket qua
let nhan_vat = "em be" ∘ "buon";
let hanh_dong = nhan_vat → "khoc";
let ket_qua = hanh_dong → "me den" → "om";

-- Chain cau chuyen:
learn "Em be buon, khoc mot minh.";
learn "Me nghe tieng khoc, chay den om.";
learn "Nuoc mat kho, nu cuoi no tren moi.";

-- Silk se lien ket: buon -> khoc -> me -> om -> cuoi
-- Emotional arc: V=-0.6 -> V=-0.8 -> V=+0.3 -> V=+0.7
-- ConversationCurve nhan ra arc recovery -> Celebratory
```

---

## 10. Reference

### Keywords & Unicode equivalents

| Keyword  | Unicode | Ghi chu                  |
|----------|---------|--------------------------|
| `let`    | `≔`     | Gan bien                 |
| `fn`     | (none)  | Dinh nghia ham           |
| `if`     | `⇒`     | Dieu kien                |
| `else`   | `⊥`     | Nhanh else               |
| `loop`   | `↻`     | Lap                      |
| `emit`   | `○`     | Xuat                     |

### System Commands

| Command   | Co arg? | Chuc nang                           |
|-----------|---------|-------------------------------------|
| `dream`   | Khong   | Trigger Dream cycle (STM -> QR)     |
| `stats`   | Khong   | Thong ke he thong                   |
| `fuse`    | Khong   | QT2: kiem tra huu han               |
| `trace`   | Khong   | Bat/tat execution tracing           |
| `learn`   | Co      | Hoc tu van ban                      |
| `seed`    | Co      | Seed L0 nodes                       |
| `inspect` | Co      | Xem cau truc chain                  |
| `assert`  | Co      | Kiem tra chain non-empty             |
| `typeof`  | Co      | Phan loai chain (SDF/MATH/...)      |
| `explain` | Co      | Truy nguon goc chain                |
| `why`     | Co      | Giai thich ket noi giua 2 chains    |
| `health`  | Khong   | Health check                        |
| `status`  | Khong   | Status report                       |
| `help`    | Khong   | Help                                |

### IR Opcodes (26 total)

**Stack:** PUSH, LOAD, LCA, DUP, POP, SWAP
**Relation:** EDGE(rel), QUERY(rel)
**Control:** CALL, RET, JMP, JZ, LOOP
**Output:** EMIT, HALT
**System:** DREAM, STATS, NOP
**State:** STORE, LOAD_LOCAL
**QT2:** FUSE
**Debug:** TRACE, INSPECT, ASSERT, TYPEOF, WHY, EXPLAIN

### QT Axioms

| QT  | Nguyen ly                    | Opcode/Op          |
|-----|------------------------------|---------------------|
| QT1 | ○ la nguon goc               | EMIT (○)            |
| QT2 | ∞ la sai, ∞-1 moi dung      | FUSE                |
| QT3 | +/- = gia thuyet, ⧺/⊖ = vat ly, == = chan ly | __hyp_*, __phys_*, __assert_truth |

### Pipeline xu ly text thuong

```
Input -> runtime::process_text()
      -> infer_context()        -- dieu kien bien
      -> sentence_affect()      -- raw emotion
      -> ctx.apply()            -- scale theo ngu canh
      -> estimate_intent()      -- Crisis/Learn/Command/Chat
      -> SecurityGate.check()   -- DUNG neu nguy hiem
      -> learning.process_one() -- Encode -> STM -> Silk
      -> ConversationCurve      -- chon tone phu hop
      -> render response        -- tra loi
```

### Anti-patterns

```olang
-- SAI: trung binh cam xuc
let avg = (buon + vui) ÷ 2;

-- DUNG: amplify qua Silk
buon ∘ vui;  -- LCA tu nhien, Silk edges amplify

-- SAI: hardcode chain
let x = 0x01020304;

-- DUNG: tu UCD
fire;  -- registry lookup -> encode_codepoint

-- SAI: skip SecurityGate
-- (Gate LUON chay truoc moi thu)

-- SAI: delete data
-- (Append-only: KHONG BAO GIO xoa)
```

---

## Vi du tong hop

```olang
-- === HomeOS: Hoc va suy luan ===

-- 1. Seed knowledge
fire ∈ elements;
water ∈ elements;
fire → heat;
water → cool;
fire ⊥ water;

-- 2. Hoc tu van ban
learn "Lua chay nong, nuoc lam mat";
learn "Lua va nuoc doi lap nhau";

-- 3. Inspect what we know
inspect fire;
typeof fire;

-- 4. Suy luan
fire → ?;                     -- fire gay ra gi? -> heat
? ⊥ fire;                     -- cai gi doi lap fire? -> water
let opposite = fire ∘ water;  -- LCA cua doi lap -> "elements"

-- 5. Kiem tra
assert fire;                   -- fire ton tai? -> OK
assert opposite;               -- opposite ton tai? -> OK
fire == flame;                 -- fire chinh la flame? -> truth

-- 6. Debug
trace;
explain fire;

-- 7. Consolidate
dream;

-- 8. Report
stats;
```
