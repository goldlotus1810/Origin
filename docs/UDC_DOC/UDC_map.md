# UDC Map — 8,846 Unicode Dimensional Coordinates

> **Bản đồ 5D của HomeOS Olang**
> Nguồn: `Blocks.txt`, `UnicodeData.txt`, `PropList.txt`, `NameAliases.txt`,
> `StandardizedVariants.txt`, `emoji-*.txt`, `PropertyValueAliases.txt`,
> `mapping/NRC-VAD-Lexicon-v2.1` — **KHÔNG tham khảo udc.json**

---

## Sơ đồ gốc

```
                          o{65,536 × 2B = 128 KB}
     ─────────────────────────┼─────────────────────────
     |           |            |            |            |
     S           R            V            A            T
  (Shape)    (Relation)   (Valence)    (Arousal)     (Time)
   SDF         MATH       EMOTICON     EMOTICON     MUSICAL
  14 blk      21 blk     ──17 blk──   ──shared──    7 blk
  1,838 cp    2,563 cp      3,487 cp                  958 cp
     |           |            |            |            |
  P_weight = [S] [R] [V] [A] [T]  = 2 bytes = tọa độ 5D
              4b   4b  3b  3b  2b    SEALED tại bootstrap
```

**Tổng: 59 blocks · 8,846 codepoints = L0 anchor points**

---

## Nguồn dữ liệu × Chiều

| File nguồn | S (Shape) | R (Relation) | V (Valence) | A (Arousal) | T (Time) |
|-------------|-----------|--------------|-------------|-------------|----------|
| `UnicodeData.txt` | Char names: ARROW, BOX DRAWINGS, GEOMETRIC, BRAILLE | Char names: MATHEMATICAL, SUPERSCRIPT, NUMERAL, FRACTION | Char names: FACE WITH, SMILING, HEART, PLAYING CARD | (shared V) | Char names: HEXAGRAM, MUSICAL SYMBOL, BYZANTINE, TETRAGRAM |
| `Blocks.txt` | 14 block ranges (2190→1F8FF) | 21 block ranges (2000→2E7F) | 17 block ranges (2460→1DBFF) | (shared V) | 7 block ranges (4DC0→1D35F) |
| `PropList.txt` | `Pattern_Syntax`, `Other_Math` | `Other_Math`, `ID_Compat_Math_*`, `Other_Lowercase/Uppercase` | `Other_Alphabetic`, `Regional_Indicator` | (shared V) | `Diacritic`, `Other_Grapheme_Extend` |
| `NameAliases.txt` | 2 corrections (Arrows) | 1 correction (WEIERSTRASS) | — | — | 1 correction (BYZANTINE) |
| `StandardizedVariants.txt` | — | 87 variants (chancery, serifs, slant) | — | — | — |
| `emoji-data.txt` | `Emoji`(76), `Extended_Pictographic`(175) | `Emoji`(2) | `Emoji`(1335), `Emoji_Presentation`(1166), `Emoji_Modifier_Base`(132) | (shared V) | — |
| `emoji-test.txt` | 113 fully-qualified (arrow, av-symbol, geometric) | 2 fully-qualified | 3,830 fully-qualified (face, person, animal, transport, flag) | (shared V) | — |
| `PropertyValueAliases.txt` | `gc=So` (Other_Symbol), `gc=Sm` (Math_Symbol) | `gc=Sm`, `gc=Nl` (Letter_Number), `gc=No` (Other_Number) | `gc=So` (Other_Symbol) | (shared V) | `gc=So`, `gc=Mn` (Nonspacing_Mark) |
| `mapping/NRC-VAD-Lexicon` | — | — | 54,801 terms × valence score (-1→+1) | 54,801 terms × arousal score (-1→+1) | — |

---

## P_S — Shape (SDF) · 14 Blocks · 1,838 codepoints

```
P_S ──┬── S.01  Arrows                         [2190..21FF]  112 cp
      │         ├── sub: LEFTWARDS ARROW (×11)
      │         ├── sub: RIGHTWARDS ARROW (×11)
      │         ├── sub: LEFT RIGHT (×8)
      │         ├── sub: DOWNWARDS ARROW (×7)
      │         ├── sub: UPWARDS WHITE (×7)
      │         ├── sub: UPWARDS ARROW (×6)
      │         ├── sub: NORTH WEST / SOUTH EAST (×8)
      │         └── sub: +41 more (UP DOWN, HEAVY, DASHED, WAVE...)
      │         gc: Other_Symbol(85), Math_Symbol(27)
      │         PropList: Pattern_Syntax(112), Other_Math(54)
      │         emoji-data: Emoji(8), Extended_Pictographic(8)
      │         emoji-test: Symbols/arrow ×8
      │
      ├── S.02  Miscellaneous Technical         [2300..23FF]  256 cp
      │         ├── sub: APL FUNCTIONAL (×70)
      │         │        "APL FUNCTIONAL SYMBOL QUAD", "...I-BEAM",
      │         │        "...SQUISH QUAD", "...CIRCLE STAR"
      │         ├── sub: DENTISTRY SYMBOL (×15)
      │         ├── sub: HORIZONTAL SCAN (×4)
      │         ├── sub: BLACK MEDIUM (×4)
      │         ├── sub: LEFT/RIGHT PARENTHESIS (×6)
      │         └── sub: +133 more (KEYBOARD, TOP HALF, HELM, ERASE...)
      │         gc: Other_Symbol(216), Math_Symbol(34)
      │         PropList: Pattern_Syntax(256), Other_Math(9), Deprecated(2)
      │         emoji-data: Emoji(18), Emoji_Presentation(8)
      │         emoji-test: Symbols/av-symbol ×11, Travel/time ×6
      │
      ├── S.03  Box Drawing                     [2500..257F]  128 cp
      │         └── sub: BOX DRAWINGS (×128) — toàn bộ block
      │              "BOX DRAWINGS LIGHT HORIZONTAL",
      │              "BOX DRAWINGS HEAVY VERTICAL",
      │              "BOX DRAWINGS DOUBLE DOWN AND RIGHT"...
      │         gc: Other_Symbol(128)
      │         PropList: Pattern_Syntax(128)
      │
      ├── S.04  Block Elements                  [2580..259F]  32 cp
      │         ├── sub: QUADRANT UPPER (×8)
      │         ├── sub: LOWER ONE / THREE (×4)
      │         ├── sub: LEFT ONE / THREE (×4)
      │         └── sub: UPPER/LOWER HALF, FULL BLOCK, LIGHT/MEDIUM/DARK SHADE
      │         gc: Other_Symbol(32)
      │         PropList: Pattern_Syntax(32)
      │
      ├── S.05  Geometric Shapes                [25A0..25FF]  96 cp
      │         ├── sub: SQUARE WITH (×10)
      │         │        "SQUARE WITH HORIZONTAL FILL",
      │         │        "SQUARE WITH ORTHOGONAL CROSSHATCH FILL"
      │         ├── sub: WHITE SQUARE (×8)
      │         ├── sub: CIRCLE WITH (×7)
      │         ├── sub: WHITE CIRCLE (×5)
      │         ├── sub: BLACK/WHITE UP/DOWN/LEFT/RIGHT-POINTING (×12)
      │         └── sub: +37 more (LOZENGE, DIAMOND, STAR, PENTAGON...)
      │         gc: Other_Symbol(86), Math_Symbol(10)
      │         PropList: Pattern_Syntax(96), Other_Math(33)
      │         emoji-data: Emoji(8), Emoji_Presentation(2)
      │         emoji-test: Symbols/geometric ×6, Symbols/av-symbol ×2
      │
      ├── S.06  Dingbats                        [2700..27BF]  192 cp
      │         ├── sub: DINGBAT NEGATIVE (×20)
      │         │        "DINGBAT NEGATIVE CIRCLED DIGIT ONE"...
      │         ├── sub: DINGBAT CIRCLED (×10)
      │         ├── sub: OPEN CENTRE / HEAVY BLACK (×7)
      │         ├── sub: SHADOWED WHITE (×3)
      │         ├── sub: EIGHT POINTED (×3)
      │         └── sub: +129 more (SCISSORS, PENCIL, ENVELOPE, CROSS...)
      │         gc: Other_Symbol(148), Other_Number(30)
      │         PropList: Pattern_Syntax(162)
      │         emoji-data: Emoji(33), Emoji_Presentation(15), Emoji_Modifier_Base(4)
      │         emoji-test: People&Body/hand-* ×24, Symbols/other ×9, heart ×4
      │
      ├── S.07  Supplemental Arrows-A           [27F0..27FF]  16 cp
      │         ├── sub: LONG RIGHTWARDS (×5)
      │         ├── sub: LONG LEFTWARDS (×4)
      │         ├── sub: LONG LEFT RIGHT (×2)
      │         └── sub: UPWARDS/DOWNWARDS QUADRUPLE, ANTICLOCKWISE/CLOCKWISE GAPPED
      │         gc: Math_Symbol(16)
      │         PropList: Pattern_Syntax(16)
      │
      ├── S.08  Braille Patterns                [2800..28FF]  256 cp
      │         └── sub: BRAILLE PATTERN (×256) — toàn bộ block
      │              "BRAILLE PATTERN BLANK",
      │              "BRAILLE PATTERN DOTS-1",
      │              "BRAILLE PATTERN DOTS-12345678"
      │         gc: Other_Symbol(256)
      │         PropList: Pattern_Syntax(256)
      │
      ├── S.09  Supplemental Arrows-B           [2900..297F]  128 cp
      │         ├── sub: RIGHTWARDS ARROW (×10)
      │         ├── sub: LEFTWARDS/RIGHTWARDS HARPOON (×18)
      │         ├── sub: UPWARDS/DOWNWARDS HARPOON (×12)
      │         ├── sub: RIGHTWARDS TWO-HEADED (×7)
      │         ├── sub: NORTH EAST / SOUTH WEST (×10)
      │         └── sub: +41 more (TRIPLE, DOUBLE, SQUIGGLE, BARB...)
      │         gc: Math_Symbol(128)
      │         PropList: Pattern_Syntax(128)
      │         emoji-data: Emoji(2), Extended_Pictographic(2)
      │
      ├── S.10  Miscellaneous Symbols and Arrows [2B00..2BFF]  256 cp
      │         ├── sub: LEFTWARDS/RIGHTWARDS ARROW (×17)
      │         ├── sub: DOWNWARDS/UPWARDS TRIANGLE-HEADED (×17)
      │         ├── sub: LEFTWARDS/RIGHTWARDS TWO-HEADED (×8)
      │         ├── sub: BLACK CURVED (×8)
      │         ├── sub: RIBBON ARROW (×8)
      │         └── sub: +122 more (NOTCHED, CIRCLED, STAR, PENTAGON...)
      │         gc: Other_Symbol(227), Math_Symbol(27)
      │         PropList: Pattern_Syntax(256)
      │         emoji-data: Emoji(7), Emoji_Presentation(4)
      │
      ├── S.11  Ornamental Dingbats             [1F650..1F67F]  48 cp
      │         ├── sub: HEAVY NORTH/SOUTH (×8)
      │         ├── sub: NORTH/SOUTH WEST/EAST (×12)
      │         ├── sub: SANS-SERIF HEAVY (×3)
      │         └── sub: +19 more (TURNED, HEAVY SALTIRE, STAR...)
      │         gc: Other_Symbol(48)
      │
      ├── S.12  Geometric Shapes Extended       [1F780..1F7FF]  128 cp
      │         ├── sub: VERY HEAVY (×8)
      │         ├── sub: EXTREMELY HEAVY (×6)
      │         ├── sub: WHITE SQUARE (×5)
      │         ├── sub: BLACK TINY (×3)
      │         ├── sub: HEAVY EIGHT (×3)
      │         └── sub: +72 more (LIGHT FOUR, MEDIUM SIX, BOLD...)
      │         gc: Other_Symbol(120)
      │         emoji-data: Extended_Pictographic(21), Emoji(13), Emoji_Presentation(13)
      │         emoji-test: Symbols/geometric ×12
      │
      └── S.13  Supplemental Arrows-C           [1F800..1F8FF]  256 cp
                ├── sub: WIDE-HEADED NORTH/SOUTH (×20)
                ├── sub: RIGHTWARDS/LEFTWARDS ARROW (×15)
                ├── sub: UPWARDS/DOWNWARDS ARROW (×12)
                ├── sub: LEFTWARDS/UPWARDS TRIANGLE-HEADED (×10)
                └── sub: +66 more (HARPOON, LONG, DOUBLE, TRIPLE...)
                gc: Other_Symbol(162), Math_Symbol(9)
                emoji-data: Extended_Pictographic(85)
```

**S tổng: 14 blocks · range 1,838 · actual assigned 1,809**
**gc chủ đạo: `So` (Other_Symbol) + `Sm` (Math_Symbol)**
**PropList chủ đạo: `Pattern_Syntax` — ký hiệu hình học dùng trong pattern matching**

---

## P_R — Relation (MATH) · 21 Blocks · 2,563 codepoints

```
P_R ──┬── R.01  Superscripts and Subscripts     [2070..209F]  48 cp
      │         ├── sub: LATIN SUBSCRIPT (×16)
      │         │        "LATIN SUBSCRIPT SMALL LETTER A"...
      │         ├── sub: SUPERSCRIPT LATIN (×2)
      │         ├── sub: SUPERSCRIPT ZERO/FOUR..NINE (×7)
      │         ├── sub: SUBSCRIPT ZERO..NINE (×10)
      │         └── sub: +24 more (PLUS, MINUS, EQUALS, PARENTHESIS...)
      │         gc: Lm(19), Other_Number(17), Math_Symbol(6)
      │         PropList: ID_Compat_Math_Continue(27), Other_Lowercase(18)
      │
      ├── R.02  Letterlike Symbols               [2100..214F]  80 cp
      │         ├── sub: DOUBLE-STRUCK CAPITAL (×9)
      │         │        "DOUBLE-STRUCK CAPITAL C", "...H", "...N",
      │         │        "...P", "...Q", "...R", "...Z"
      │         ├── sub: SCRIPT CAPITAL (×9)
      │         │        "SCRIPT CAPITAL B/E/F/H/I/L/M/R"
      │         ├── sub: BLACK-LETTER CAPITAL (×5)
      │         ├── sub: DOUBLE-STRUCK ITALIC (×5)
      │         ├── sub: SCRIPT SMALL (×4)
      │         └── sub: +43 more (EULER, PLANCK, ANGSTROM, OHM...)
      │         gc: Uppercase_Letter(28), Other_Symbol(27), Lowercase_Letter(14), Math_Symbol(7)
      │         PropList: Other_Math(41), Soft_Dotted(2), Other_ID_Start(2)
      │         StandardizedVariants: 16 (chancery style for SCRIPT CAPITAL B/E/F/H/I/L/M/R/p/r...)
      │         emoji-data: Emoji(2), Extended_Pictographic(2)
      │
      ├── R.03  Number Forms                     [2150..218F]  64 cp
      │         ├── sub: ROMAN NUMERAL (×24)
      │         │        "ROMAN NUMERAL ONE"..."ROMAN NUMERAL TWELVE"
      │         ├── sub: SMALL ROMAN NUMERAL (×16)
      │         ├── sub: VULGAR FRACTION (×16)
      │         │        "VULGAR FRACTION ONE THIRD",
      │         │        "VULGAR FRACTION TWO THIRDS"...
      │         └── sub: TURNED DIGIT (×2), FRACTION NUMERATOR (×1)
      │         gc: Letter_Number(39), Other_Number(17), Other_Symbol(2)
      │         PropList: Other_Lowercase(16), Other_Uppercase(16)
      │
      ├── R.04  Mathematical Operators           [2200..22FF]  256 cp
      │         ├── sub: DOES NOT (×10)
      │         │        "DOES NOT DIVIDE", "DOES NOT CONTAIN AS MEMBER"
      │         ├── sub: ELEMENT OF (×7)
      │         │        "ELEMENT OF", "NOT AN ELEMENT OF",
      │         │        "SMALL ELEMENT OF", "CONTAINS AS MEMBER"
      │         ├── sub: EQUAL TO (×5)
      │         ├── sub: SMALL ELEMENT / CONTAINS (×6)
      │         └── sub: +190 more (INTEGRAL, UNION, INTERSECTION,
      │              SQUARE ROOT, INFINITY, PROPORTIONAL, ANGLE,
      │              FOR ALL, THERE EXISTS, NABLA, PRODUCT, SUM,
      │              LESS-THAN, GREATER-THAN, SUBSET, SUPERSET,
      │              LOGICAL AND, LOGICAL OR, TILDE, NOT SIGN...)
      │         gc: Math_Symbol(256) — 100%
      │         PropList: Pattern_Syntax(256), ID_Compat_Math_*(6)
      │         StandardizedVariants: 23 (serifs, slant, vertical stroke variants)
      │
      ├── R.05  Miscellaneous Mathematical Symbols-A [27C0..27EF]  48 cp
      │         ├── sub: MATHEMATICAL LEFT/RIGHT (×10)
      │         │        "MATHEMATICAL LEFT WHITE SQUARE BRACKET"...
      │         ├── sub: WHITE CONCAVE-SIDED (×3)
      │         ├── sub: SQUARED LOGICAL (×2)
      │         └── sub: +29 more (THREE DIMENSIONAL ANGLE, LONG DIVISION...)
      │         gc: Math_Symbol(36), Open_Punctuation(6), Close_Punctuation(6)
      │         PropList: Pattern_Syntax(48), Other_Math(12)
      │
      ├── R.06  Miscellaneous Mathematical Symbols-B [2980..29FF]  128 cp
      │         ├── sub: MEASURED ANGLE (×9)
      │         ├── sub: Z NOTATION (×6)
      │         ├── sub: EMPTY SET (×4)
      │         ├── sub: CIRCLE WITH (×4)
      │         ├── sub: LEFT/RIGHT SQUARE BRACKET (×6)
      │         └── sub: +81 more (TRIPLE, VERTICAL, ARC, FENCE...)
      │         gc: Math_Symbol(100), Open_Punctuation(14), Close_Punctuation(14)
      │         PropList: Pattern_Syntax(128), Other_Math(28)
      │         StandardizedVariants: 1 (CIRCLED PARALLEL with parallel lines)
      │
      ├── R.07  Supplemental Mathematical Operators [2A00..2AFF]  256 cp
      │         ├── sub: PLUS SIGN (×11)
      │         ├── sub: INTEGRAL WITH (×7)
      │         ├── sub: Z NOTATION (×6)
      │         ├── sub: MULTIPLICATION SIGN (×6)
      │         ├── sub: LOGICAL AND / OR (×12)
      │         └── sub: +115 more (JOIN, MEET, AMALGAMATION, COPRODUCT...)
      │         gc: Math_Symbol(256) — 100%
      │         PropList: Pattern_Syntax(256)
      │         StandardizedVariants: 9 (tall variant, slanted equal, similar slant...)
      │
      ├── R.08  Mathematical Alphanumeric Symbols [1D400..1D7FF]  1024 cp
      │         ├── sub: MATHEMATICAL SANS-SERIF (×344)
      │         │        BOLD/ITALIC/BOLD ITALIC × A-Z, a-z
      │         ├── sub: MATHEMATICAL BOLD (×336)
      │         │        CAPITAL/SMALL × A-Z, a-z, ALPHA-OMEGA
      │         ├── sub: MATHEMATICAL ITALIC (×112)
      │         ├── sub: MATHEMATICAL MONOSPACE (×62)
      │         ├── sub: MATHEMATICAL DOUBLE-STRUCK (×55)
      │         ├── sub: MATHEMATICAL FRAKTUR (×47)
      │         └── sub: MATHEMATICAL SCRIPT (×41)
      │         gc: Lowercase_Letter(493), Uppercase_Letter(444), Decimal_Number(50), Math_Symbol(10)
      │         PropList: Other_Math(987), Soft_Dotted(26), ID_Compat_Math_*(20)
      │         StandardizedVariants: 37 (chancery style for SCRIPT A-Z)
      │
      ├── R.09  Ancient Greek Numbers            [10140..1018F]  80 cp
      │         ├── sub: GREEK ACROPHONIC (×53)
      │         │        "GREEK ACROPHONIC ATTIC ONE DRACHMA",
      │         │        "GREEK ACROPHONIC EPIDAUREAN TWO HUNDRED"
      │         └── sub: GREEK ONE/TWO/THREE... (×26)
      │         gc: Letter_Number(53), Other_Symbol(20), Other_Number(6)
      │
      ├── R.10  Common Indic Number Forms        [A830..A83F]  16 cp
      │         └── sub: NORTH INDIC (×10)
      │              "NORTH INDIC FRACTION ONE QUARTER",
      │              "NORTH INDIC RUPEE MARK"
      │         gc: Other_Number(6), Other_Symbol(3), Currency_Symbol(1)
      │
      ├── R.11  Counting Rod Numerals            [1D360..1D37F]  32 cp
      │         ├── sub: COUNTING ROD (×18)
      │         │        "COUNTING ROD UNIT DIGIT ONE"...
      │         ├── sub: IDEOGRAPHIC TALLY (×5)
      │         └── sub: TALLY MARK (×2)
      │         gc: Other_Number(25)
      │
      ├── R.12  Cuneiform Numbers and Punctuation [12400..1247F]  128 cp
      │         ├── sub: CUNEIFORM NUMERIC (×123)
      │         │        "CUNEIFORM NUMERIC SIGN TWO ASH",
      │         │        "CUNEIFORM NUMERIC SIGN NINE SHAR2"
      │         └── sub: CUNEIFORM PUNCTUATION (×5)
      │         gc: Letter_Number(123), Other_Punctuation(5)
      │         PropList: Terminal_Punctuation(5)
      │
      ├── R.13  General Punctuation              [2000..206F]  112 cp
      │         ├── sub: EN/EM QUAD, SPACE variants (×11)
      │         ├── sub: HYPHEN, DASH, HORIZONTAL BAR (×6)
      │         ├── sub: QUOTATION MARK variants (×12)
      │         ├── sub: BULLET, ELLIPSIS, PER MILLE (×5)
      │         └── sub: +78 more (ZERO WIDTH, WORD JOINER, FRACTION SLASH...)
      │         gc: Format, Dash, Punctuation mix
      │         PropList: Pattern_Syntax, Dash, Quotation_Mark
      │
      ├── R.14  Currency Symbols                 [20A0..20CF]  48 cp
      │         └── sub: currency signs (EURO, POUND, LIRA, RUPEE,
      │              FRANC, WON, DONG, TENGE, RUBLE, MANAT...)
      │         gc: Currency_Symbol
      │
      ├── R.15  Combining Diacritical Marks for Symbols [20D0..20FF]  48 cp
      │         └── sub: COMBINING marks (ENCLOSING CIRCLE, ENCLOSING SCREEN,
      │              LONG SOLIDUS OVERLAY, RING OVERLAY...)
      │         gc: Nonspacing_Mark, Enclosing_Mark
      │
      ├── R.16  Control Pictures                 [2400..243F]  64 cp
      │         └── sub: SYMBOL FOR (×39)
      │              "SYMBOL FOR NULL", "SYMBOL FOR START OF HEADING",
      │              "SYMBOL FOR END OF TEXT", "SYMBOL FOR DELETE"
      │         gc: Other_Symbol
      │
      ├── R.17  Optical Character Recognition    [2440..245F]  32 cp
      │         └── sub: OCR (×11)
      │              "OCR HOOK", "OCR CHAIR", "OCR FORK",
      │              "OCR INVERTED FORK", "OCR BELT BUCKLE"
      │         gc: Other_Symbol
      │
      ├── R.18  Supplemental Punctuation         [2E00..2E7F]  128 cp
      │         ├── sub: LEFT/RIGHT SUBSTITUTION/TRANSPOSITION BRACKET (×8)
      │         ├── sub: DOTTED/RAISED variants (×6)
      │         └── sub: +80 more (EDITORIAL CORONIS, PARAGRAPHOS,
      │              FORKED PARAGRAPHOS, TILDE...)
      │         gc: Punctuation mix
      │         PropList: Pattern_Syntax, Terminal_Punctuation
      │
      ├── R.19  Indic Siyaq Numbers              [1EC70..1ECBF]  80 cp
      │         └── sub: INDIC SIYAQ (×68)
      │              "INDIC SIYAQ NUMBER ONE",
      │              "INDIC SIYAQ LAKH MARK"
      │         gc: Other_Number(66), Other_Symbol(1), Currency_Symbol(1)
      │
      ├── R.20  Ottoman Siyaq Numbers            [1ED00..1ED4F]  80 cp
      │         └── sub: OTTOMAN SIYAQ (×61)
      │              "OTTOMAN SIYAQ NUMBER ONE",
      │              "OTTOMAN SIYAQ MARRATAN"
      │         gc: Other_Number(60), Other_Symbol(1)
      │
      └── R.21  Arabic Mathematical Alphabetic Symbols [1EE00..1EEFF]  256 cp
                └── sub: ARABIC MATHEMATICAL (×143)
                     "ARABIC MATHEMATICAL ALEF",
                     "ARABIC MATHEMATICAL LOOPED LAM",
                     "ARABIC MATHEMATICAL DOUBLE-STRUCK DAD"
                gc: Other_Letter(141), Math_Symbol(2)
                PropList: Other_Math(141)
```

**R tổng: 21 blocks · range 2,563 · actual assigned ~2,717**
**gc chủ đạo: `Sm` (Math_Symbol) + `No` (Other_Number) + `Nl` (Letter_Number)**
**PropList chủ đạo: `Other_Math`, `Pattern_Syntax` — quan hệ toán học logic**
**StandardizedVariants: 87 biến thể (chancery style, serifs, slant, vertical stroke)**

---

## P_V + P_A — Valence + Arousal (EMOTICON) · 17 Blocks · 3,487 codepoints

> **V và A chia sẻ cùng 17 blocks.** Mỗi ký tự có CẢ V lẫn A.
> V = cảm xúc tích cực/tiêu cực (polarity). A = cường độ kích thích (intensity).
> Nguồn V/A: `mapping/NRC-VAD-Lexicon-v2.1` (54,801 terms × valence × arousal × dominance).
> Emoji annotation: `emoji-test.txt` (subgroup = ngữ nghĩa cảm xúc).

```
P_V ──┬── E.01  Enclosed Alphanumerics           [2460..24FF]  160 cp
P_A   │         ├── sub: CIRCLED LATIN (×52)
      │         │        "CIRCLED LATIN CAPITAL LETTER A"...
      │         ├── sub: PARENTHESIZED LATIN (×26)
      │         ├── sub: CIRCLED/PARENTHESIZED NUMBER (×22)
      │         ├── sub: NEGATIVE CIRCLED (×11)
      │         ├── sub: CIRCLED DIGIT (×10)
      │         └── sub: +22 more (DOUBLE CIRCLED, CIRCLED HANGUL...)
      │         gc: Other_Number(82), Other_Symbol(78)
      │         PropList: Other_Alphabetic(52), Other_Lowercase(26), Other_Uppercase(26)
      │         emoji-data: Emoji(1)
      │
      ├── E.02  Miscellaneous Symbols            [2600..26FF]  256 cp
      │         ├── sub: TRIGRAM FOR (×8)
      │         │        "TRIGRAM FOR HEAVEN", "TRIGRAM FOR EARTH",
      │         │        "TRIGRAM FOR WATER", "TRIGRAM FOR FIRE"
      │         ├── sub: RECYCLING SYMBOL (×8)
      │         ├── sub: WHITE/BLACK CHESS (×12)
      │         ├── sub: DIGRAM FOR (×4)
      │         ├── sub: BALLOT BOX (×3)
      │         └── sub: +206 more (SUN, CLOUD, UMBRELLA, SNOWMAN,
      │              COMET, TELEPHONE, SKULL, RADIOACTIVE, PEACE,
      │              MALE/FEMALE SIGN, MERCURY, STAR OF DAVID,
      │              YIN YANG, SCALES, ANCHOR, COFFIN, FUNERAL URN...)
      │         gc: Other_Symbol(255), Math_Symbol(1)
      │         PropList: Pattern_Syntax(256), Other_Math(10)
      │         emoji-data: Emoji(83), Emoji_Presentation(31), Emoji_Modifier_Base(2)
      │         emoji-test: person-sport ×19, zodiac ×13, sky&weather ×11,
      │                     tool ×7, hand ×6, religion ×6, game ×5, sport ×4
      │
      ├── E.03  Mahjong Tiles                    [1F000..1F02F]  48 cp
      │         └── sub: MAHJONG TILE (×44)
      │              "MAHJONG TILE EAST WIND",
      │              "MAHJONG TILE RED DRAGON",
      │              "MAHJONG TILE BAMBOO"
      │         gc: Other_Symbol(44)
      │         emoji-data: Extended_Pictographic(5), Emoji(1)
      │         emoji-test: Activities/game ×1
      │
      ├── E.04  Domino Tiles                     [1F030..1F09F]  112 cp
      │         └── sub: DOMINO TILE (×100)
      │              "DOMINO TILE HORIZONTAL-00-00",
      │              "DOMINO TILE VERTICAL-06-06"
      │         gc: Other_Symbol(100)
      │         emoji-data: Extended_Pictographic(12)
      │
      ├── E.05  Playing Cards                    [1F0A0..1F0FF]  96 cp
      │         └── sub: PLAYING CARD (×82)
      │              "PLAYING CARD ACE OF SPADES",
      │              "PLAYING CARD KING OF HEARTS",
      │              "PLAYING CARD BLACK JOKER"
      │         gc: Other_Symbol(82)
      │         emoji-data: Extended_Pictographic(15), Emoji(1)
      │         emoji-test: Activities/game ×1
      │
      ├── E.06  Enclosed Alphanumeric Supplement  [1F100..1F1FF]  256 cp
      │         ├── sub: NEGATIVE SQUARED (×31)
      │         │        "NEGATIVE SQUARED LATIN CAPITAL LETTER A"...
      │         ├── sub: SQUARED LATIN (×27)
      │         ├── sub: PARENTHESIZED LATIN (×26)
      │         ├── sub: NEGATIVE CIRCLED (×26)
      │         ├── sub: REGIONAL INDICATOR SYMBOL (×26) ← cờ quốc gia
      │         │        "REGIONAL INDICATOR SYMBOL LETTER A"...
      │         │        (ghép cặp → 🇻🇳 🇺🇸 🇯🇵...)
      │         └── sub: +62 more (DIGIT, COMMA, IDEOGRAPH...)
      │         gc: Other_Symbol(188), Other_Number(13)
      │         PropList: Other_Alphabetic(78), Other_Uppercase(78), Regional_Indicator(26)
      │         emoji-data: Emoji(41), Emoji_Presentation(37), Emoji_Component(26)
      │         emoji-test: Flags/country-flag ×259, Symbols/alphanum ×15
      │
      ├── E.07  Enclosed Ideographic Supplement   [1F200..1F2FF]  256 cp
      │         ├── sub: SQUARED CJK (×43)
      │         │        "SQUARED CJK UNIFIED IDEOGRAPH-6307" (指)
      │         ├── sub: TORTOISE SHELL (×9)
      │         ├── sub: ROUNDED SYMBOL (×6)
      │         ├── sub: SQUARED KATAKANA (×3)
      │         └── sub: CIRCLED IDEOGRAPH (×2)
      │         gc: Other_Symbol(64)
      │         emoji-data: Extended_Pictographic(207), Emoji(15), Emoji_Presentation(13)
      │         emoji-test: Symbols/alphanum ×15
      │
      ├── E.08  Miscellaneous Symbols and Pictographs [1F300..1F5FF]  768 cp
      │         ├── sub: CLOCK FACE (×24)
      │         │        "CLOCK FACE ONE OCLOCK"..."CLOCK FACE TWELVE-THIRTY"
      │         ├── sub: EMOJI MODIFIER (×5) — FITZPATRICK TYPE-1-2...6
      │         ├── sub: INPUT SYMBOL (×5)
      │         ├── sub: LOWER LEFT/RIGHT (×5)
      │         ├── sub: CLOUD WITH (×4)
      │         └── sub: +666 more — ĐÂY LÀ BLOCK LỚN NHẤT:
      │              🌀 CYCLONE, 🌈 RAINBOW, 🌍 EARTH GLOBE,
      │              🍎 RED APPLE, 🍕 PIZZA, 🎃 JACK-O-LANTERN,
      │              🎄 CHRISTMAS TREE, 🎵 MUSICAL NOTE,
      │              🏠 HOUSE, 🐱 CAT FACE, 🐶 DOG FACE,
      │              👁 EYE, 👑 CROWN, 💎 GEM STONE,
      │              💰 MONEY BAG, 💀 SKULL, 🔥 FIRE,
      │              🔫 PISTOL, 🗡 DAGGER, 🗽 STATUE OF LIBERTY
      │         gc: Other_Symbol(763), Modifier_Symbol(5)
      │         emoji-data: Emoji(637), Emoji_Presentation(559), Emoji_Modifier_Base(56)
      │         emoji-test: person-role ×324, family ×271, person-activity ×220,
      │                     person-sport ×124, person ×108, animal-mammal ×41,
      │                     sky&weather ×33, clothing ×26, time ×25, +67 subgroups
      │
      ├── E.09  Emoticons                        [1F600..1F64F]  80 cp
      │         ├── sub: FACE WITH (×12)
      │         │        "FACE WITH TEARS OF JOY" 😂,
      │         │        "FACE WITH MEDICAL MASK" 😷,
      │         │        "FACE WITH NO GOOD GESTURE" 🙅
      │         ├── sub: SMILING FACE (×9)
      │         │        "SMILING FACE WITH OPEN MOUTH" 😃,
      │         │        "SMILING FACE WITH HEART-SHAPED EYES" 😍
      │         ├── sub: KISSING FACE (×3)
      │         ├── sub: GRINNING FACE (×2)
      │         ├── sub: CAT FACE / SMILING CAT (×4)
      │         └── sub: +49 more (WEARY, ASTONISHED, FLUSHED,
      │              HUSHED, PERSEVERING, DISAPPOINTED, ANGRY,
      │              FEARFUL, CRYING, POUTING, FOLDED HANDS...)
      │         gc: Other_Symbol(80) — 100%
      │         emoji-data: Emoji(80), Emoji_Presentation(80) — 100% emoji
      │         emoji-test: person-gesture ×108, face-concerned ×21,
      │                     face-smiling ×12, hands ×12, face-neutral ×11,
      │                     cat-face ×9, face-affection ×5, face-tongue ×4
      │
      ├── E.10  Transport and Map Symbols        [1F680..1F6FF]  128 cp
      │         ├── sub: HIGH-SPEED TRAIN (×2)
      │         ├── sub: ROCKET, HELICOPTER, STEAM LOCOMOTIVE...
      │         └── sub: +113 more (AIRPLANE, SAILBOAT, SPEEDBOAT,
      │              AUTOMOBILE, BUS, POLICE CAR, AMBULANCE,
      │              TOILET, BATH, BED, FERRY, MOTOR SCOOTER,
      │              STOP SIGN, CONSTRUCTION...)
      │         gc: Other_Symbol(120)
      │         emoji-data: Emoji(107), Emoji_Presentation(94), Emoji_Modifier_Base(6)
      │         emoji-test: person-sport ×54, transport-ground ×45,
      │                     person-activity ×36, person-resting ×12,
      │                     transport-sign ×11, transport-air ×10
      │
      ├── E.11  Alchemical Symbols               [1F700..1F77F]  128 cp
      │         ├── sub: ALCHEMICAL SYMBOL (×116)
      │         │        "ALCHEMICAL SYMBOL FOR GOLD",
      │         │        "ALCHEMICAL SYMBOL FOR SILVER",
      │         │        "ALCHEMICAL SYMBOL FOR MERCURY",
      │         │        "ALCHEMICAL SYMBOL FOR AQUA REGIA"
      │         └── sub: LOT OF, OCCULTATION, LUNAR ECLIPSE... (×12)
      │         gc: Other_Symbol(128)
      │
      ├── E.12  Supplemental Symbols and Pictographs [1F900..1F9FF]  256 cp
      │         ├── sub: FACE WITH (×10)
      │         │        "FACE WITH COWBOY HAT" 🤠,
      │         │        "FACE WITH HAND OVER MOUTH" 🤭
      │         ├── sub: LEFT HALF (×5)
      │         ├── sub: DOWNWARD FACING (×4)
      │         ├── sub: EMOJI COMPONENT (×4)
      │         ├── sub: SMILING FACE (×3)
      │         └── sub: +225 more (BRAIN 🧠, BONE 🦴, TOOTH 🦷,
      │              SUPERHERO 🦸, MERMAID 🧜, ELF 🧝,
      │              YARN 🧶, BROOM 🧹, TEST TUBE 🧪,
      │              CUPCAKE 🧁, LLAMA 🦙, PEACOCK 🦚)
      │         gc: Other_Symbol(256)
      │         emoji-data: Emoji(242), Emoji_Presentation(242), Emoji_Modifier_Base(46)
      │         emoji-test: person-activity ×152, person-role ×150,
      │                     person-fantasy ×145, person-sport ×111,
      │                     family ×66, person ×60, +48 subgroups
      │
      ├── E.13  Chess Symbols                    [1FA00..1FA6F]  112 cp
      │         ├── sub: NEUTRAL CHESS (×30)
      │         ├── sub: WHITE CHESS (×29)
      │         ├── sub: BLACK CHESS (×29)
      │         ├── sub: XIANGQI RED (×7)
      │         └── sub: XIANGQI BLACK (×7)
      │         gc: Other_Symbol(102)
      │         emoji-data: Extended_Pictographic(10)
      │
      ├── E.14  Symbols and Pictographs Extended-A [1FA70..1FAFF]  144 cp
      │         ├── sub: FACE WITH (×4)
      │         └── sub: +119 more (BALLET SHOES, SHORTS, THONG SANDAL,
      │              ACCORDION 🪗, LONG DRUM 🪘, MIRROR 🪞,
      │              WINDOW 🪟, PLUNGER 🪠, MOUSE TRAP 🪤,
      │              ROCK 🪨, WOOD 🪵, LOTUS 🪷, CORAL 🪸,
      │              BEANS 🫘, POURING LIQUID 🫗, BITING LIP 🫦,
      │              HEART HANDS 🫶, MOOSE 🫎, DONKEY 🫏)
      │         gc: Other_Symbol(128)
      │         emoji-data: Emoji(128), Emoji_Presentation(128), Emoji_Modifier_Base(14)
      │         emoji-test: hand-fingers-open ×36, hands ×26, person-role ×18,
      │                     hand-fingers-closed ×12, household ×9, clothing ×8
      │
      ├── E.15  Symbols for Legacy Computing     [1FB00..1FBFF]  256 cp
      │         ├── sub: BOX DRAWINGS (×32)
      │         ├── sub: UPPER RIGHT/LEFT (×26)
      │         ├── sub: LOWER LEFT/RIGHT (×24)
      │         ├── sub: SEGMENTED DIGIT (×10)
      │         │        "SEGMENTED DIGIT ZERO"..."SEGMENTED DIGIT NINE"
      │         └── sub: +120 more (BLOCK SEXTANT, SEPARATED BLOCK QUADRANT,
      │              CHECKER BOARD, SMOOTH MOSAIC...)
      │         gc: Other_Symbol(240), Decimal_Number(10)
      │
      ├── E.16  Miscellaneous Symbols Supplement  [1CEC0..1CEFF]  64 cp
      │         ├── sub: GEOMANTIC FIGURE (×16)
      │         │        "GEOMANTIC FIGURE ACQUISITIO",
      │         │        "GEOMANTIC FIGURE LAETITIA"
      │         ├── sub: ALCHEMICAL SYMBOL (×3)
      │         ├── sub: SQUARE ROOT (×3)
      │         ├── sub: SECTOR WITH (×3)
      │         └── sub: +23 more (MEASURED ANGLE, LEIBNIZIAN...)
      │         gc: Other_Symbol(36), Math_Symbol(17)
      │
      └── E.17  Miscellaneous Symbols and Arrows Extended [1DB00..1DBFF]  256 cp
                ├── sub: LEIBNIZIAN CONGRUENCE-2 (×4)
                ├── sub: LEIBNIZIAN EQUALS (×3)
                ├── sub: LEIBNIZIAN GREATER-THAN (×3)
                ├── sub: LEIBNIZIAN LESS-THAN (×2)
                └── sub: +13 more (INVERTED SQUARE, LEIBNIZIAN SIMILARITY...)
                gc: Math_Symbol(29)
                StandardizedVariants: 1 (CARTESIAN EQUALS SIGN with descender)
```

**V+A tổng: 17 blocks · range 3,487 · actual assigned ~2,821**
**gc chủ đạo: `So` (Other_Symbol) — ký hiệu biểu tượng cảm xúc**
**PropList chủ đạo: `Regional_Indicator`(26), `Other_Alphabetic`(130)**
**emoji-data: 1,335 Emoji, 1,166 Emoji_Presentation, 132 Emoji_Modifier_Base**
**NRC-VAD Lexicon: 54,801 terms (valence -1→+1, arousal -1→+1, dominance -1→+1)**

### V vs A — cách phân biệt trên cùng block

```
V (Valence) = hướng cảm xúc:
  0x00 ← cực tiêu cực (💀 SKULL, ☠ SKULL AND CROSSBONES)
  0x80 ← trung tính    (♾ INFINITY, ⚖ SCALES)
  0xFF ← cực tích cực  (😍 HEART-EYES, 🌈 RAINBOW)

A (Arousal) = cường độ kích thích:
  0x00 ← rất yên tĩnh  (😴 SLEEPING FACE, 🧘 PERSON IN LOTUS POSITION)
  0x80 ← vừa phải      (🙂 SLIGHTLY SMILING FACE)
  0xFF ← cực kích thích (🔥 FIRE, 💥 COLLISION, 😱 FACE SCREAMING IN FEAR)

Ví dụ P_weight [S, R, V, A, T]:
  😂 FACE WITH TEARS OF JOY    → V=0xE0 (rất vui)    A=0xC0 (kích thích cao)
  😴 SLEEPING FACE              → V=0x70 (hơi tích cực) A=0x10 (rất yên tĩnh)
  💀 SKULL                      → V=0x10 (rất tiêu cực) A=0x30 (tĩnh/lạnh)
  🔥 FIRE                       → V=0x90 (hơi tích cực) A=0xF0 (cực mạnh)
```

---

## P_T — Time (MUSICAL) · 7 Blocks · 958 codepoints

> **T = chiều thời gian / nhịp điệu / dao động.**
> Spline — công thức tạo hình dạng âm thanh, bước sóng, sự giao động.
> Musical notation = biểu diễn trực tiếp sóng âm thanh trên trục thời gian.
> Hexagram/Tetragram = nhịp biến đổi (Dịch lý = spline rời rạc 64/81 trạng thái).

```
P_T ──┬── T.01  Yijing Hexagram Symbols         [4DC0..4DFF]  64 cp
      │         └── sub: HEXAGRAM FOR (×64) — toàn bộ block
      │              "HEXAGRAM FOR THE CREATIVE HEAVEN" ䷀ (Càn)
      │              "HEXAGRAM FOR THE RECEPTIVE EARTH" ䷁ (Khôn)
      │              "HEXAGRAM FOR DIFFICULTY AT THE BEGINNING" ䷂
      │              "HEXAGRAM FOR YOUTHFUL FOLLY" ䷃
      │              "HEXAGRAM FOR WAITING" ䷄
      │              "HEXAGRAM FOR CONFLICT" ䷅
      │              "HEXAGRAM FOR THE ARMY" ䷆
      │              ...
      │              "HEXAGRAM FOR BEFORE COMPLETION" ䷾
      │              "HEXAGRAM FOR AFTER COMPLETION" ䷿
      │              → 64 quẻ = 2⁶ trạng thái = spline rời rạc
      │              → mỗi quẻ = 6 hào (âm/dương) = 6-bit phase
      │         gc: Other_Symbol(64) — 100%
      │
      ├── T.02  Znamenny Musical Notation        [1CF00..1CFCF]  208 cp
      │         ├── sub: ZNAMENNY NEUME (×116)
      │         │        "ZNAMENNY NEUME KRYUK",
      │         │        "ZNAMENNY NEUME STOMITSA",
      │         │        "ZNAMENNY NEUME GOLUBCHIK BORZY"
      │         │        → neume = ký hiệu giai điệu Slavonic cổ
      │         │        → mỗi neume = một đường cong pitch trên trục t
      │         ├── sub: ZNAMENNY COMBINING (×64)
      │         │        "ZNAMENNY COMBINING MARK GORAZDO NIZKO S BORFOY"
      │         │        → modifier = biến đổi hình dạng sóng
      │         └── sub: ZNAMENNY PRIZNAK (×5)
      │         gc: Other_Symbol(116), Nonspacing_Mark(69)
      │         PropList: Diacritic(69) — dấu biến âm = frequency modulation
      │
      ├── T.03  Byzantine Musical Symbols        [1D000..1D0FF]  256 cp
      │         └── sub: BYZANTINE MUSICAL (×246)
      │              "BYZANTINE MUSICAL SYMBOL PSILI",
      │              "BYZANTINE MUSICAL SYMBOL OLIGON",
      │              "BYZANTINE MUSICAL SYMBOL PETASTI",
      │              "BYZANTINE MUSICAL SYMBOL ISON",
      │              "BYZANTINE MUSICAL SYMBOL APODERMA"
      │              → Byzantine notation: mỗi ký hiệu = interval/direction
      │              → lên/xuống bao nhiêu bước = slope trên spline
      │         gc: Other_Symbol(246)
      │         NameAliases: 1 correction (FTHORA SKLIRON CHROMA VASIS)
      │
      ├── T.04  Musical Symbols                  [1D100..1D1FF]  256 cp
      │         └── sub: MUSICAL SYMBOL (×256) — toàn bộ block
      │              "MUSICAL SYMBOL SINGLE BARLINE",
      │              "MUSICAL SYMBOL DOUBLE BARLINE",
      │              "MUSICAL SYMBOL FINAL BARLINE",
      │              "MUSICAL SYMBOL G CLEF" (𝄞 treble clef),
      │              "MUSICAL SYMBOL C CLEF",
      │              "MUSICAL SYMBOL F CLEF" (𝄢 bass clef),
      │              "MUSICAL SYMBOL WHOLE NOTE" (𝅝),
      │              "MUSICAL SYMBOL HALF NOTE" (𝅗𝅥),
      │              "MUSICAL SYMBOL QUARTER NOTE" (𝅘𝅥),
      │              "MUSICAL SYMBOL EIGHTH NOTE" (𝅘𝅥𝅮),
      │              "MUSICAL SYMBOL SIXTEENTH NOTE" (𝅘𝅥𝅯),
      │              "MUSICAL SYMBOL CRESCENDO" (𝆒),
      │              "MUSICAL SYMBOL DECRESCENDO" (𝆓),
      │              "MUSICAL SYMBOL FERMATA" (𝆏),
      │              "MUSICAL SYMBOL SEGNO", "MUSICAL SYMBOL CODA"
      │              → Western notation: pitch × duration = waveform
      │              → note value = thời lượng trên trục t
      │              → clef = frequency range
      │              → barline = phase boundary
      │         gc: Other_Symbol(216), Nonspacing_Mark(24), Spacing_Mark(8), Format(8)
      │         PropList: Diacritic(28), Other_Grapheme_Extend(8)
      │
      ├── T.05  Ancient Greek Musical Notation   [1D200..1D24F]  80 cp
      │         ├── sub: GREEK INSTRUMENTAL (×37)
      │         │        "GREEK INSTRUMENTAL NOTATION SYMBOL-1"...
      │         │        → ký hiệu nhạc cụ cổ Hy Lạp
      │         ├── sub: GREEK VOCAL (×29)
      │         │        "GREEK VOCAL NOTATION SYMBOL-1"...
      │         │        → ký hiệu giọng hát
      │         ├── sub: COMBINING GREEK (×3)
      │         └── sub: GREEK MUSICAL (×1)
      │         gc: Other_Symbol(67), Nonspacing_Mark(3)
      │         → vocal vs instrumental = 2 loại sóng trên cùng trục t
      │
      ├── T.06  Musical Symbols Supplement       [1D250..1D28F]  64 cp
      │         └── sub: MUSICAL SYMBOL (×50)
      │              "MUSICAL SYMBOL SORI",
      │              "MUSICAL SYMBOL KORON",
      │              "MUSICAL SYMBOL DOUBLE SHARP ABOVE"
      │              → microtonal symbols = fine-grained frequency
      │              → sori/koron = quarter-tone (Ba Tư)
      │         gc: Other_Symbol(42), Spacing_Mark(6), Nonspacing_Mark(2)
      │         PropList: Other_Grapheme_Extend(6), Diacritic(5)
      │
      └── T.07  Tai Xuan Jing Symbols            [1D300..1D35F]  96 cp
                ├── sub: TETRAGRAM FOR (×81)
                │        "TETRAGRAM FOR CENTRE" 𝌀,
                │        "TETRAGRAM FOR FULLNESS" 𝌁,
                │        "TETRAGRAM FOR MIRED" 𝌂,
                │        "TETRAGRAM FOR COMPLIANCE" 𝍒,
                │        "TETRAGRAM FOR ON THE VERGE" 𝍓,
                │        "TETRAGRAM FOR DIFFICULTIES" 𝍔,
                │        "TETRAGRAM FOR LABOURING" 𝍕,
                │        "TETRAGRAM FOR FOSTERING" 𝍖
                │        → 81 tứ quái = 3⁴ trạng thái
                │        → Thái Huyền Kinh: 3-valued logic × 4 hào
                │        → finer spline than hexagram (64 → 81 states)
                ├── sub: DIGRAM FOR (×5)
                │        "DIGRAM FOR EARTH/HEAVEN/HUMAN"
                └── sub: MONOGRAM FOR (×1)
                gc: Other_Symbol(87)
```

**T tổng: 7 blocks · range 958 · actual assigned 958**
**gc chủ đạo: `So` (Other_Symbol) + `Mn` (Nonspacing_Mark)**
**PropList chủ đạo: `Diacritic`(102) — biến âm = frequency/amplitude modulation**

### T = Spline trên trục thời gian

```
T biểu diễn bước sóng / nhịp / giao động:

  Level 0 (Static)   ← Hexagram/Tetragram (trạng thái tĩnh, snapshot)
  Level 1 (Slow)     ← Whole note 𝅝, Fermata 𝆏 (kéo dài)
  Level 2 (Medium)   ← Quarter note 𝅘𝅥, Half note 𝅗𝅥
  Level 3 (Fast)     ← Eighth 𝅘𝅥𝅮, Sixteenth 𝅘𝅥𝅯 (nhanh)
  Level 4 (Instant)  ← Grace note, Staccato (tức thì)

Spline interpretation:
  Neume (Znamenny/Byzantine)  → melodic contour = shape of wave
  Note value (Western)        → duration = wavelength
  Clef                        → frequency register
  Diacritic                   → modulation (vibrato, trill, bend)
  Hexagram 6-bit / Tetragram 4-trit → discrete phase states

  f(t) = Σ [note_value × pitch × modulation]
       = chuỗi spline knots trên trục thời gian
```

---

## Tổng kết

```
                          o{8,846 L0 anchors}
     ─────────────────────────┼─────────────────────────
     |           |            |            |            |
     S           R            V            A            T
   14 blk      21 blk       ──17 blk──                7 blk
   1,838       2,563          3,487                     958
   1,809*      2,717*         2,821*                    958*
     |           |            |            |            |
  Pattern     Other_Math    Emoji       NRC-VAD      Diacritic
  _Syntax     Math_Symbol   Emoji_Pres  Lexicon      Other_
  So+Sm       Sm+No+Nl      So          54,801terms  Grapheme
                                         V/A scores   Extend

  * actual assigned (UnicodeData.txt) vs range (Blocks.txt)
  * Δ = 8,846 - 8,305 = 541 unassigned slots trong ranges

Nguồn:
  UnicodeData.txt          → char names (TEXT tên ký tự = tên node)
  Blocks.txt               → block ranges (59 blocks)
  PropList.txt             → Pattern_Syntax, Other_Math, Diacritic...
  NameAliases.txt          → 4 corrections
  StandardizedVariants.txt → 87 glyph variants (chủ yếu MATH)
  emoji-data.txt           → Emoji/Presentation/Modifier_Base flags
  emoji-test.txt           → semantic subgroups (face, person, animal...)
  PropertyValueAliases.txt → gc=Sm/So/No/Nl/Mn category mapping
  PropertyAliases.txt      → property name → abbreviation
  mapping/NRC-VAD-Lexicon  → 54,801 terms × (valence, arousal, dominance)
```

---

## PHẦN 2: Danh sách TEXT trích xuất từ Unicode source files

> **Nguồn**: UnicodeData.txt, PropList.txt, PropertyAliases.txt, PropertyValueAliases.txt,
> NameAliases.txt, StandardizedVariants.txt, emoji-test.txt, emoji-data.txt,
> emoji-sequences.txt, emoji-zwj-sequences.txt, mapping/NRC-VAD-Lexicon-v2.1
>
> **Quy tắc gom**: từ giống nhau → 1 từ. Cụm từ giống nhau → 1 cụm từ. Câu giống nhau → 1 câu.

### Tổng quan

| Chiều | Tổng từ | Từ riêng | Tổng cụm từ | Cụm từ riêng | Câu (4+ từ) |
|-------|---------|----------|-------------|-------------|------------|
| **S** (Shape) | 480 | 81 | 1,809 | 1,809 | 1,095 |
| **R** (Relation) | 790 | 291 | 3,784 | 3,784 | 2,076 |
| **V** (Valence) | 28,682 | 7,990 | 13,852 | 3,319 | 0 |
| **A** (Arousal) | 29,155 | 8,472 | 11,174 | 641 | 0 |
| **T** (Time) | 713 | 433 | 958 | 958 | 791 |
| **V∩A** (shared) | 20,546 | — | 10,533 | — | — |

---

### S — Shape (SDF) · Từ vựng hình dạng

**81 từ riêng biệt** (chỉ xuất hiện trong S, không trong R/V/A/T):
```
  ac
  admetos
  alternative
  apl
  apollon
  arrowhead
  arrowheads
  arta
  astrological
  backslanted
  benzene
  binovile
  bisected
  bisecting
  bud
  conical
  continuous
  counterbore
  crosshatch
  cupido
  cylindricity
  del
  dentistry
  dimension
  drafting
  ellipse
  eris
  exponent
  extend
  extension
  fisheye
  florette
  flowing
  fuse
  gapped
  grapheme
  hades
  hellschreiber
  helm
  hysteresis
  kronos
  lobe
  location
  maltese
  metrical
  monostable
  nessus
  novile
  option
  paired
  passed
  pentagon
  petalled
  pholus
  poseidon
  propeller
  quintile
  recorder
  rectilinear
  rhombus
  runout
  scan
  sedna
  selena
  sentagon
  shouldered
  spoked
  taper
  then
  transpluto
  trapezium
  tredecile
  trifoliate
  underline
  undo
  variation
  viewdata
  vigintile
  vulcanus
  zeus
  zilde
```

**480 từ tổng** (bao gồm từ chung với các chiều khác):
```
  above, ac, admetos, airplane, alarm, all, almost, alpha, alternative, ampersand, and, angle
  anticlockwise, apl, apollon, arc, arrow, arrowhead, arrowheads, arrows, arta, asterisk, astraea, astrological
  backslanted, backslash, ballot, bar, barb, bars, base, bell, below, bent, benzene, beside
  between, binovile, bisected, bisecting, black, blade, blank, block, blue, board, bold, bottom
  box, bracket, braille, breve, broken, brown, bud, bullet, bullseye, but, caret, ceiling
  centre, centred, check, checker, chevron, circle, circled, circling, circular, clear, clock, clockwise
  closed, colon, comma, composition, compressed, conical, containing, continuous, corner, corners, counterbore, countersink
  crop, cross, crosshatch, crossing, cube, cupido, curly, current, curved, curving, cusp, cylindricity
  dark, dash, dashed, david, decimal, del, delta, dentistry, dependent, diaeresis, diagonal, diameter
  diamond, digit, dimension, dingbat, direct, directly, discontinuous, divide, division, dot, dotted, double
  doubled, down, downwards, drafting, drawing, drawings, drive, earth, east, eight, eighth, eighths
  eject, electric, electrical, electronics, ellipse, emphasis, enter, envelope, epsilon, equal, equals, equilateral
  erase, eris, et, exclamation, exponent, extend, extension, extremely, eye, falling, fill, fish
  fisheye, fist, five, flatness, flattened, floor, floral, florette, flowing, foot, for, form
  four, from, frown, full, functional, fuse, gapped, geometric, grapheme, greek, green, ground
  group, hades, half, hand, harpoon, head, headed, heart, heavy, hellschreiber, helm, hexagon
  hollow, hook, hooked, horizontal, hourglass, house, hygiea, hysteresis, in, indicator, infinity, insertion
  inside, integral, interest, interrobang, intersection, inverse, iota, isosceles, joined, jot, key, keyboard
  kronos, large, latin, leaf, left, leftwards, ligature, light, line, lines, lobe, location
  long, loop, low, lower, lozenge, maltese, mark, medium, metrical, middle, minus, monostable
  moon, multiplication, narrow, negative, neptune, nessus, newline, next, nib, nine, north, northwest
  not, notch, notched, novile, number, oblique, observer, octagon, of, omega, on, one
  open, operator, option, or, orange, origin, ornament, orthogonal, outlined, oval, over, overbar
  overlapping, page, paired, paragraph, parallelogram, parenthesis, passed, pattern, pause, pedestal, pencil, pentagon
  pentaseme, perspective, petalled, pholus, piece, pinwheel, place, plus, pluto, point, pointed, pointer
  pointing, poseidon, position, power, previous, print, projective, propeller, proserpina, purple, quad, quadrant
  quadruple, quarter, quarters, question, quilt, quintile, quotation, quote, radical, raised, record, recorder
  rectangle, rectilinear, red, return, reverse, reversed, rho, rhombus, ribbon, right, rightwards, ring
  rising, rocket, rotated, round, rounded, runout, russian, safety, saltire, sand, scan, scissors
  screen, script, section, sector, sedna, segment, selector, selena, semicircle, semicircular, semicolon, sentagon
  separated, separator, seven, shade, shaded, shadowed, shaft, shell, shoe, short, shorts, shouldered
  sign, single, six, sixteen, slanted, slash, sleep, slightly, slope, small, smile, snowflake
  solid, solidus, south, sparkle, sparkles, spoked, square, squared, squares, squat, squiggle, squish
  star, stem, stile, stop, stopwatch, straightness, stress, stroke, subset, summation, superset, swash
  symbol, symmetry, syntax, tab, tack, tail, tape, taper, target, telephone, ten, tetraseme
  the, then, thin, third, thirds, three, through, tight, tilde, timer, tiny, tip
  to, top, tortoise, total, transparent, transpluto, trapezium, tredecile, triangle, trifoliate, triple, triseme
  true, turned, twelve, two, type, uncertainty, underbar, underline, undo, united, up, upper
  upwards, vane, variation, vertical, very, victory, viewdata, vigintile, vine, vulcanus, wall, watch
  wave, wavy, west, white, width, with, within, writing, yellow, zeus, zigzag, zilde
```

**Cụm từ riêng S** (1809 cụm, hiển thị 80 mẫu):
```
  AC CURRENT
  ADMETOS
  AIRPLANE
  ALARM CLOCK
  ALL AROUND-PROFILE
  ALTERNATIVE KEY SYMBOL
  ANTICLOCKWISE CLOSED CIRCLE ARROW
  ANTICLOCKWISE GAPPED CIRCLE ARROW
  ANTICLOCKWISE OPEN CIRCLE ARROW
  ANTICLOCKWISE TOP SEMICIRCLE ARROW
  ANTICLOCKWISE TRIANGLE-HEADED BOTTOM U-SHAPED ARROW
  ANTICLOCKWISE TRIANGLE-HEADED LEFT U-SHAPED ARROW
  ANTICLOCKWISE TRIANGLE-HEADED OPEN CIRCLE ARROW
  ANTICLOCKWISE TRIANGLE-HEADED RIGHT U-SHAPED ARROW
  ANTICLOCKWISE TRIANGLE-HEADED TOP U-SHAPED ARROW
  APL FUNCTIONAL SYMBOL ALPHA
  APL FUNCTIONAL SYMBOL ALPHA UNDERBAR
  APL FUNCTIONAL SYMBOL BACKSLASH BAR
  APL FUNCTIONAL SYMBOL CIRCLE BACKSLASH
  APL FUNCTIONAL SYMBOL CIRCLE DIAERESIS
  APL FUNCTIONAL SYMBOL CIRCLE JOT
  APL FUNCTIONAL SYMBOL CIRCLE STAR
  APL FUNCTIONAL SYMBOL CIRCLE STILE
  APL FUNCTIONAL SYMBOL CIRCLE UNDERBAR
  APL FUNCTIONAL SYMBOL COMMA BAR
  APL FUNCTIONAL SYMBOL DEL DIAERESIS
  APL FUNCTIONAL SYMBOL DEL STILE
  APL FUNCTIONAL SYMBOL DEL TILDE
  APL FUNCTIONAL SYMBOL DELTA STILE
  APL FUNCTIONAL SYMBOL DELTA UNDERBAR
  APL FUNCTIONAL SYMBOL DIAMOND UNDERBAR
  APL FUNCTIONAL SYMBOL DOWN CARET TILDE
  APL FUNCTIONAL SYMBOL DOWN SHOE STILE
  APL FUNCTIONAL SYMBOL DOWN TACK JOT
  APL FUNCTIONAL SYMBOL DOWN TACK UNDERBAR
  APL FUNCTIONAL SYMBOL DOWNWARDS VANE
  APL FUNCTIONAL SYMBOL EPSILON UNDERBAR
  APL FUNCTIONAL SYMBOL GREATER-THAN DIAERESIS
  APL FUNCTIONAL SYMBOL I-BEAM
  APL FUNCTIONAL SYMBOL IOTA
  APL FUNCTIONAL SYMBOL IOTA UNDERBAR
  APL FUNCTIONAL SYMBOL JOT DIAERESIS
  APL FUNCTIONAL SYMBOL JOT UNDERBAR
  APL FUNCTIONAL SYMBOL LEFT SHOE STILE
  APL FUNCTIONAL SYMBOL LEFTWARDS VANE
  APL FUNCTIONAL SYMBOL OMEGA
  APL FUNCTIONAL SYMBOL OMEGA UNDERBAR
  APL FUNCTIONAL SYMBOL QUAD
  APL FUNCTIONAL SYMBOL QUAD BACKSLASH
  APL FUNCTIONAL SYMBOL QUAD CIRCLE
  APL FUNCTIONAL SYMBOL QUAD COLON
  APL FUNCTIONAL SYMBOL QUAD DEL
  APL FUNCTIONAL SYMBOL QUAD DELTA
  APL FUNCTIONAL SYMBOL QUAD DIAMOND
  APL FUNCTIONAL SYMBOL QUAD DIVIDE
  APL FUNCTIONAL SYMBOL QUAD DOWN CARET
  APL FUNCTIONAL SYMBOL QUAD DOWNWARDS ARROW
  APL FUNCTIONAL SYMBOL QUAD EQUAL
  APL FUNCTIONAL SYMBOL QUAD GREATER-THAN
  APL FUNCTIONAL SYMBOL QUAD JOT
  APL FUNCTIONAL SYMBOL QUAD LEFTWARDS ARROW
  APL FUNCTIONAL SYMBOL QUAD LESS-THAN
  APL FUNCTIONAL SYMBOL QUAD NOT EQUAL
  APL FUNCTIONAL SYMBOL QUAD QUESTION
  APL FUNCTIONAL SYMBOL QUAD RIGHTWARDS ARROW
  APL FUNCTIONAL SYMBOL QUAD SLASH
  APL FUNCTIONAL SYMBOL QUAD UP CARET
  APL FUNCTIONAL SYMBOL QUAD UPWARDS ARROW
  APL FUNCTIONAL SYMBOL QUOTE QUAD
  APL FUNCTIONAL SYMBOL QUOTE UNDERBAR
  APL FUNCTIONAL SYMBOL RHO
  APL FUNCTIONAL SYMBOL RIGHTWARDS VANE
  APL FUNCTIONAL SYMBOL SEMICOLON UNDERBAR
  APL FUNCTIONAL SYMBOL SLASH BAR
  APL FUNCTIONAL SYMBOL SQUISH QUAD
  APL FUNCTIONAL SYMBOL STAR DIAERESIS
  APL FUNCTIONAL SYMBOL STILE TILDE
  APL FUNCTIONAL SYMBOL TILDE DIAERESIS
  APL FUNCTIONAL SYMBOL UP CARET TILDE
  APL FUNCTIONAL SYMBOL UP SHOE JOT
  ... +1729 cụm từ nữa
```

**Câu riêng S** (1095 câu, hiển thị 50 mẫu):
```
  ANTICLOCKWISE CLOSED CIRCLE ARROW
  ANTICLOCKWISE GAPPED CIRCLE ARROW
  ANTICLOCKWISE OPEN CIRCLE ARROW
  ANTICLOCKWISE TOP SEMICIRCLE ARROW
  ANTICLOCKWISE TRIANGLE-HEADED BOTTOM U-SHAPED ARROW
  ANTICLOCKWISE TRIANGLE-HEADED LEFT U-SHAPED ARROW
  ANTICLOCKWISE TRIANGLE-HEADED OPEN CIRCLE ARROW
  ANTICLOCKWISE TRIANGLE-HEADED RIGHT U-SHAPED ARROW
  ANTICLOCKWISE TRIANGLE-HEADED TOP U-SHAPED ARROW
  APL FUNCTIONAL SYMBOL ALPHA
  APL FUNCTIONAL SYMBOL ALPHA UNDERBAR
  APL FUNCTIONAL SYMBOL BACKSLASH BAR
  APL FUNCTIONAL SYMBOL CIRCLE BACKSLASH
  APL FUNCTIONAL SYMBOL CIRCLE DIAERESIS
  APL FUNCTIONAL SYMBOL CIRCLE JOT
  APL FUNCTIONAL SYMBOL CIRCLE STAR
  APL FUNCTIONAL SYMBOL CIRCLE STILE
  APL FUNCTIONAL SYMBOL CIRCLE UNDERBAR
  APL FUNCTIONAL SYMBOL COMMA BAR
  APL FUNCTIONAL SYMBOL DEL DIAERESIS
  APL FUNCTIONAL SYMBOL DEL STILE
  APL FUNCTIONAL SYMBOL DEL TILDE
  APL FUNCTIONAL SYMBOL DELTA STILE
  APL FUNCTIONAL SYMBOL DELTA UNDERBAR
  APL FUNCTIONAL SYMBOL DIAMOND UNDERBAR
  APL FUNCTIONAL SYMBOL DOWN CARET TILDE
  APL FUNCTIONAL SYMBOL DOWN SHOE STILE
  APL FUNCTIONAL SYMBOL DOWN TACK JOT
  APL FUNCTIONAL SYMBOL DOWN TACK UNDERBAR
  APL FUNCTIONAL SYMBOL DOWNWARDS VANE
  APL FUNCTIONAL SYMBOL EPSILON UNDERBAR
  APL FUNCTIONAL SYMBOL GREATER-THAN DIAERESIS
  APL FUNCTIONAL SYMBOL I-BEAM
  APL FUNCTIONAL SYMBOL IOTA
  APL FUNCTIONAL SYMBOL IOTA UNDERBAR
  APL FUNCTIONAL SYMBOL JOT DIAERESIS
  APL FUNCTIONAL SYMBOL JOT UNDERBAR
  APL FUNCTIONAL SYMBOL LEFT SHOE STILE
  APL FUNCTIONAL SYMBOL LEFTWARDS VANE
  APL FUNCTIONAL SYMBOL OMEGA
  APL FUNCTIONAL SYMBOL OMEGA UNDERBAR
  APL FUNCTIONAL SYMBOL QUAD
  APL FUNCTIONAL SYMBOL QUAD BACKSLASH
  APL FUNCTIONAL SYMBOL QUAD CIRCLE
  APL FUNCTIONAL SYMBOL QUAD COLON
  APL FUNCTIONAL SYMBOL QUAD DEL
  APL FUNCTIONAL SYMBOL QUAD DELTA
  APL FUNCTIONAL SYMBOL QUAD DIAMOND
  APL FUNCTIONAL SYMBOL QUAD DIVIDE
  APL FUNCTIONAL SYMBOL QUAD DOWN CARET
  ... +1045 câu nữa
```
---

### R — Relation (MATH) · Từ vựng quan hệ

**291 từ riêng biệt** (chỉ xuất hiện trong R):
```
  acrophonic, actually, addressed, ain, aktieselskab, alef, alphabetic, amalgamation, ancora, angstrom
  antirestriction, approaches, approximate, approximately, arabic, aroura, artabe, assyrian, asterisks, asterism
  asymptotically, austral, because, beh, bitcoin, buru, cada, capitulum, carriage, carystian
  cedi, celsius, centered, circumflex, conjugate, contain, contains, contoured, coproduct, cornish
  coronis, corresponds, counting, crossbar, cruzeiro, cuneiform, cyrenaic, dalet, definition, delimiter
  delphic, descender, device, differential, digamma, dimensional, directional, dirham, divides, domain
  dong, dotless, drachma, drachmas, elamite, elevatus, em, embedding, en, enclosing
  epidaurean, equiangular, estimates, eta, euler, exists, facsimile, fahrenheit, feh, fifth
  fifths, fifty, formatting, forty, fraktur, fullwidth, gal, gamma, geometrically, geshu
  ghain, gimel, gleich, gramma, guarani, gur, ha, hah, heh, heraeum
  hermionian, hermitian, homothetic, hryvnia, hyphen, hypodiastole, identical, ideographic, ilimmu, imin
  increases, indic, indiction, intercalate, interpolation, intersecting, jeem, kaf, kappa, karor
  karoran, kavyka, kelvin, khah, kip, kurrent, kyathos, lakh, lakhan, lam
  lamda, lari, limmu, litra, livre, looped, manat, marratan, mathematical, matrix
  medieval, meem, members, messenian, metretes, mill, mille, min, miny, mnas
  models, modulo, monospace, mu, multimap, multiset, nabla, naira, nand, naxian
  negated, nested, nigidaesh, nigidamin, nomisma, nonforking, nor, nu, numeric, numero
  obelos, obol, obols, ocr, ohm, omani, omicron, ounkia, outer, outline
  overline, paragraphos, paragraphus, paraphrase, patty, perpendicular, peseta, peso, phi, pilcrow
  planck, plethron, precede, precedes, preceding, prefixed, proof, proportion, prove, psi
  punctus, qaf, quadcolon, quantity, quaternion, questioned, ratio, recording, reh, rial
  righthand, rim, riyal, roman, roundhand, ruble, rufiyaa, rupee, schema, schwa
  seen, semidirect, serifs, shapes, shaping, sharu, sheqel, sibe, sigma, signs
  sine, sinusoid, sixteenths, sixths, siyaq, slant, sloping, som, spesmilo, stacked
  staters, stratian, strokes, subgroup, subjoined, substitute, substitution, succeeds, superimposed, surface
  synchronous, tah, tailed, talents, tally, tatweel, tau, teh, tenge, tens
  tenu, thal, theh, therefore, theta, tick, tironian, tournois, transversal, tricolon
  troezenian, tryblion, tugrik, turkish, uae, una, underdot, undertie, upsilon, using
  ussu, variant, vee, versicle, waw, won, xestes, xor, yeh, zah
  zain
```

**790 từ tổng**:
```
  above, accent, account, acknowledge, acrophonic, activate, actually, acute, addressed, ain, aktieselskab, alef
  aligned, all, almost, alpha, alphabetic, alternate, amalgamation, amount, ampersand, an, ancora, and
  angle, angstrom, annuity, anticlockwise, antirestriction, application, approaches, approximate, approximately, arabic, arc, arm
  around, aroura, arrow, artabe, as, ash, assertion, assyrian, asterisk, asterisks, asterism, asymptotically
  at, attic, austral, average, backslash, backspace, bag, bank, bar, barb, base, because
  beh, bell, below, belt, beside, bet, beta, between, big, binary, binding, bitcoin
  black, blank, block, board, bold, bottom, bow, bowtie, box, bracket, branch, bridge
  buckle, bullet, bumpy, buru, but, by, cada, cancel, cap, capital, capitulum, care
  caret, carriage, carystian, cedi, celsius, centered, centre, centred, chair, chancery, character, check
  checker, chi, circle, circled, circulation, circumflex, cjk, clockwise, close, closed, colon, combining
  comma, commercial, compatibility, complement, composition, congruent, conjugate, consecutive, constant, contain, containing, contains
  contour, contoured, control, coproduct, copyright, corner, cornish, coronis, corresponds, counting, cross, crossbar
  cruzeiro, cube, cuneiform, cup, curly, currency, curve, curved, customer, cyrenaic, dad, dagger
  dal, dalet, dash, data, definition, degree, degrees, delete, delimiter, delphic, delta, descender
  device, diaeresis, diagonal, diamond, difference, differential, digamma, digit, dimensional, directional, dirham, dish
  divide, divided, divider, divides, division, does, domain, dong, dot, dotless, dots, dotted
  double, down, downwards, drachma, drachmas, early, editorial, eight, eighth, eighths, eighty, elamite
  element, elevatus, eleven, ellipsis, em, embedding, empty, en, enclosing, end, ending, enquiry
  epidaurean, epsilon, equal, equals, equiangular, equivalent, escape, estimated, estimates, et, eta, euler
  euro, excess, exclamation, exist, exists, expanded, facsimile, factor, fahrenheit, falling, feed, feh
  fence, fifth, fifths, fifty, figure, file, final, finite, first, five, flattened, flower
  following, foot, for, force, forces, fork, forked, forking, form, formatting, forty, four
  fourth, fraction, fraktur, franc, french, from, full, fullwidth, function, gal, gamma, geometric
  geometrically, german, geshu, ghain, gimel, gleich, gramma, greek, group, guarani, guard, gur
  ha, hah, hair, half, harpoon, heading, heh, heraeum, hermionian, hermitian, high, homothetic
  hook, horizontal, hourglass, hryvnia, hundred, hyphen, hyphenation, hypodiastole, identical, identification, ideographic, idle
  ilimmu, image, imin, in, including, incomplete, increases, increment, indian, indic, indiction, infinity
  information, inhibit, initial, insertion, inside, integral, integration, intercalate, interior, interpolation, interrobang, intersecting
  intersection, inverted, invisible, iota, isolate, italic, jeem, join, joined, joiner, kaf, kappa
  karor, karoran, kavyka, kelvin, keycap, khah, kip, kurrent, kyathos, lakh, lakhan, lam
  lamda, large, larger, lari, late, latin, lazy, leader, left, leftwards, leg, letter
  ligature, limit, limmu, line, lines, link, lira, litra, livre, logical, long, looped
  low, lower, lozenge, manat, mark, marker, marratan, math, mathematical, matrix, measured, medieval
  medium, meem, member, members, membership, messenian, metretes, middle, midline, mill, mille, min
  minus, miny, mnas, models, modifier, modulo, monospace, mu, much, multimap, multiplication, multiset
  nabla, naira, nand, narrow, national, naxian, negated, negation, negative, neither, nested, new
  newline, nigidaesh, nigidamin, nine, ninety, ninth, nominal, nomisma, nonforking, noon, nor, nordic
  normal, north, not, notation, nu, null, number, numeral, numerator, numeric, numero, obelos
  oblique, obol, obols, ocr, of, ohm, old, omani, omega, omicron, omission, one
  open, opening, operator, or, original, ottoman, ounce, ounkia, out, outer, outline, over
  overbar, overlapping, overlay, overline, override, palm, paragraph, paragraphos, paragraphus, parallel, paraphrase, parenthesis
  part, partial, path, patty, penny, per, perpendicular, peseta, peso, phi, pi, pilcrow
  piping, pitchfork, placeholder, planck, plethron, plus, point, pointing, pole, pop, precede, precedes
  preceding, prefixed, prescription, prime, product, projection, proof, property, proportion, proportional, prove, psi
  punctuation, punctus, qaf, quad, quadcolon, quadruple, quantity, quarter, quarters, quaternion, question, questioned
  quill, quotation, raised, range, ratio, record, recording, rectangular, reference, reh, relation, relational
  response, return, reverse, reversed, rho, rial, right, righthand, rightwards, rim, ring, rising
  riyal, rod, roman, root, rotated, roundhand, ruble, rufiyaa, rule, rupee, sad, samaritan
  saudi, schema, schwa, screen, script, scruple, second, section, seen, semicircular, semicolon, semidirect
  separator, serifs, service, set, seven, seventh, seventy, shade, shapes, shaping, sharu, sheen
  shell, sheqel, shift, short, shuffle, sibe, sideways, sigma, sign, signs, similar, sine
  single, sinusoid, six, sixteenth, sixteenths, sixth, sixths, sixty, siyaq, slant, slanted, slash
  sloping, small, smaller, smash, solidus, som, sound, source, space, spesmilo, spherical, spot
  square, squared, squares, stacked, star, stark, start, staters, stem, stenographic, stop, stratian
  stretched, strictly, stroke, strokes, strong, style, subgroup, subject, subjoined, subscript, subset, substitute
  substitution, succeed, succeeds, sum, summation, superimposed, superscript, superset, surface, suspension, swapping, swung
  symbol, symmetric, synchronous, tabulation, tack, tah, tailed, take, talent, talents, tall, tally
  tatweel, tau, tee, teh, telephone, ten, tenge, tens, tenth, tenu, terminal, text
  thal, than, the, theh, there, therefore, thermodynamic, thespian, theta, thin, third, thirds
  thirty, thousand, three, through, tick, tie, tilde, times, tiny, tironian, to, top
  tortoise, touching, tournois, trade, transmission, transposition, transversal, triangle, triangular, tricolon, triple, troezenian
  true, tryblion, tugrik, turkish, turned, turnstile, twelve, twenty, two, type, uae, una
  under, underbar, underdot, undertie, union, unit, up, upper, upsilon, upturn, upward, upwards
  using, ussu, variant, vector, vee, verse, versicle, vertical, vertically, very, volume, vulgar
  wave, waw, white, wide, width, wiggly, with, won, word, wreath, xestes, xi
  xor, ya, year, yeh, yet, zah, zain, zero, zeta, zigzag
```

**Cụm từ riêng R** (3784 cụm, hiển thị 80 mẫu):
```
  ACCOUNT OF
  ACTIVATE ARABIC FORM SHAPING
  ACTIVATE SYMMETRIC SWAPPING
  ACUTE ANGLE
  ADDRESSED TO THE SUBJECT
  AKTIESELSKAB
  ALEF SYMBOL
  ALL EQUAL TO
  ALMOST EQUAL OR EQUAL TO
  ALMOST EQUAL TO
  ALMOST EQUAL TO WITH CIRCUMFLEX ACCENT
  AMALGAMATION OR COPRODUCT
  AND WITH DOT
  ANGLE
  ANGLE WITH S INSIDE
  ANGLE WITH UNDERBAR
  ANGSTROM SIGN
  ANTICLOCKWISE CONTOUR INTEGRAL
  ANTICLOCKWISE INTEGRATION
  APPROACHES THE LIMIT
  APPROXIMATELY BUT NOT ACTUALLY EQUAL TO
  APPROXIMATELY EQUAL OR EQUAL TO
  APPROXIMATELY EQUAL TO
  APPROXIMATELY EQUAL TO OR THE IMAGE OF
  ARABIC MATHEMATICAL AIN
  ARABIC MATHEMATICAL ALEF
  ARABIC MATHEMATICAL BEH
  ARABIC MATHEMATICAL DAD
  ARABIC MATHEMATICAL DAL
  ARABIC MATHEMATICAL DOTLESS BEH
  ARABIC MATHEMATICAL DOTLESS FEH
  ARABIC MATHEMATICAL DOTLESS NOON
  ARABIC MATHEMATICAL DOTLESS QAF
  ARABIC MATHEMATICAL DOUBLE-STRUCK AIN
  ARABIC MATHEMATICAL DOUBLE-STRUCK BEH
  ARABIC MATHEMATICAL DOUBLE-STRUCK DAD
  ARABIC MATHEMATICAL DOUBLE-STRUCK DAL
  ARABIC MATHEMATICAL DOUBLE-STRUCK FEH
  ARABIC MATHEMATICAL DOUBLE-STRUCK GHAIN
  ARABIC MATHEMATICAL DOUBLE-STRUCK HAH
  ARABIC MATHEMATICAL DOUBLE-STRUCK JEEM
  ARABIC MATHEMATICAL DOUBLE-STRUCK KHAH
  ARABIC MATHEMATICAL DOUBLE-STRUCK LAM
  ARABIC MATHEMATICAL DOUBLE-STRUCK MEEM
  ARABIC MATHEMATICAL DOUBLE-STRUCK NOON
  ARABIC MATHEMATICAL DOUBLE-STRUCK QAF
  ARABIC MATHEMATICAL DOUBLE-STRUCK REH
  ARABIC MATHEMATICAL DOUBLE-STRUCK SAD
  ARABIC MATHEMATICAL DOUBLE-STRUCK SEEN
  ARABIC MATHEMATICAL DOUBLE-STRUCK SHEEN
  ARABIC MATHEMATICAL DOUBLE-STRUCK TAH
  ARABIC MATHEMATICAL DOUBLE-STRUCK TEH
  ARABIC MATHEMATICAL DOUBLE-STRUCK THAL
  ARABIC MATHEMATICAL DOUBLE-STRUCK THEH
  ARABIC MATHEMATICAL DOUBLE-STRUCK WAW
  ARABIC MATHEMATICAL DOUBLE-STRUCK YEH
  ARABIC MATHEMATICAL DOUBLE-STRUCK ZAH
  ARABIC MATHEMATICAL DOUBLE-STRUCK ZAIN
  ARABIC MATHEMATICAL FEH
  ARABIC MATHEMATICAL GHAIN
  ARABIC MATHEMATICAL HAH
  ARABIC MATHEMATICAL INITIAL AIN
  ARABIC MATHEMATICAL INITIAL BEH
  ARABIC MATHEMATICAL INITIAL DAD
  ARABIC MATHEMATICAL INITIAL FEH
  ARABIC MATHEMATICAL INITIAL GHAIN
  ARABIC MATHEMATICAL INITIAL HAH
  ARABIC MATHEMATICAL INITIAL HEH
  ARABIC MATHEMATICAL INITIAL JEEM
  ARABIC MATHEMATICAL INITIAL KAF
  ARABIC MATHEMATICAL INITIAL KHAH
  ARABIC MATHEMATICAL INITIAL LAM
  ARABIC MATHEMATICAL INITIAL MEEM
  ARABIC MATHEMATICAL INITIAL NOON
  ARABIC MATHEMATICAL INITIAL QAF
  ARABIC MATHEMATICAL INITIAL SAD
  ARABIC MATHEMATICAL INITIAL SEEN
  ARABIC MATHEMATICAL INITIAL SHEEN
  ARABIC MATHEMATICAL INITIAL TEH
  ARABIC MATHEMATICAL INITIAL THEH
  ... +3704 cụm từ nữa
```

**Câu riêng R** (2076 câu, hiển thị 50 mẫu):
```
  ACTIVATE ARABIC FORM SHAPING
  ADDRESSED TO THE SUBJECT
  ALMOST EQUAL OR EQUAL TO
  ALMOST EQUAL TO WITH CIRCUMFLEX ACCENT
  ANGLE WITH S INSIDE
  APPROXIMATELY BUT NOT ACTUALLY EQUAL TO
  APPROXIMATELY EQUAL OR EQUAL TO
  APPROXIMATELY EQUAL TO OR THE IMAGE OF
  ARABIC MATHEMATICAL DOTLESS BEH
  ARABIC MATHEMATICAL DOTLESS FEH
  ARABIC MATHEMATICAL DOTLESS NOON
  ARABIC MATHEMATICAL DOTLESS QAF
  ARABIC MATHEMATICAL DOUBLE-STRUCK AIN
  ARABIC MATHEMATICAL DOUBLE-STRUCK BEH
  ARABIC MATHEMATICAL DOUBLE-STRUCK DAD
  ARABIC MATHEMATICAL DOUBLE-STRUCK DAL
  ARABIC MATHEMATICAL DOUBLE-STRUCK FEH
  ARABIC MATHEMATICAL DOUBLE-STRUCK GHAIN
  ARABIC MATHEMATICAL DOUBLE-STRUCK HAH
  ARABIC MATHEMATICAL DOUBLE-STRUCK JEEM
  ARABIC MATHEMATICAL DOUBLE-STRUCK KHAH
  ARABIC MATHEMATICAL DOUBLE-STRUCK LAM
  ARABIC MATHEMATICAL DOUBLE-STRUCK MEEM
  ARABIC MATHEMATICAL DOUBLE-STRUCK NOON
  ARABIC MATHEMATICAL DOUBLE-STRUCK QAF
  ARABIC MATHEMATICAL DOUBLE-STRUCK REH
  ARABIC MATHEMATICAL DOUBLE-STRUCK SAD
  ARABIC MATHEMATICAL DOUBLE-STRUCK SEEN
  ARABIC MATHEMATICAL DOUBLE-STRUCK SHEEN
  ARABIC MATHEMATICAL DOUBLE-STRUCK TAH
  ARABIC MATHEMATICAL DOUBLE-STRUCK TEH
  ARABIC MATHEMATICAL DOUBLE-STRUCK THAL
  ARABIC MATHEMATICAL DOUBLE-STRUCK THEH
  ARABIC MATHEMATICAL DOUBLE-STRUCK WAW
  ARABIC MATHEMATICAL DOUBLE-STRUCK YEH
  ARABIC MATHEMATICAL DOUBLE-STRUCK ZAH
  ARABIC MATHEMATICAL DOUBLE-STRUCK ZAIN
  ARABIC MATHEMATICAL INITIAL AIN
  ARABIC MATHEMATICAL INITIAL BEH
  ARABIC MATHEMATICAL INITIAL DAD
  ARABIC MATHEMATICAL INITIAL FEH
  ARABIC MATHEMATICAL INITIAL GHAIN
  ARABIC MATHEMATICAL INITIAL HAH
  ARABIC MATHEMATICAL INITIAL HEH
  ARABIC MATHEMATICAL INITIAL JEEM
  ARABIC MATHEMATICAL INITIAL KAF
  ARABIC MATHEMATICAL INITIAL KHAH
  ARABIC MATHEMATICAL INITIAL LAM
  ARABIC MATHEMATICAL INITIAL MEEM
  ARABIC MATHEMATICAL INITIAL NOON
  ... +2026 câu nữa
```
---

### V∩A — Shared EMOTICON pool (Valence + Arousal chung)

**20,546 từ chung V∩A** (hiển thị 150 mẫu):
```
  ab, abacus, abandoner, abase, abased, abasement, abash, abashedly, abashment, abatable
  abate, abdicate, abdication, abdicator, abduct, abducted, abductee, abduction, abductor, aberrant
  aberration, aberrational, abet, abhorrence, abhorrent, abhorring, abider, abiding, abjection, abjectly
  abjectness, ablaze, ably, abnormality, aboard, abolisher, abolition, abolitionary, abolitionism, abolitionist
  abominable, abominableness, abominably, abominate, abomination, abominator, abort, abortion, abortionist, abortive
  abortively, about, above, aboveboard, abrasively, abrasiveness, abrupt, abruption, abruptly, abruptness
  abscond, absconder, absence, absent, absentee, absentminded, absentmindedness, absentness, absoluteness, absolvable
  absorbability, absorbable, absorbency, abstention, abstinent, abstinently, abstractionism, absurd, absurdness, abundance
  abuse, abused, abuser, abusive, abusively, abusiveness, abuzz, abysmal, abysmally, abyssal
  academically, academy, acapulco, acceleration, accelerative, acceleratory, accentuation, accept, acceptability, acceptable
  acceptableness, acceptably, acceptance, acceptant, acceptor, accessorize, accident, accidental, accidentally, accidently
  acclaimer, acclamation, acclamatory, acclimate, acclimation, acclimatization, acclimatize, accommodation, accommodative, accommodator
  accompanier, accompanist, accomplish, accomplisher, accomplishment, accord, accordance, accordant, accordion, accost
  account, accountable, accredit, accreditation, accrual, accumulativeness, accurateness, accurse, accursed, accursedness
  accusation, accusative, accusatively, accusatorial, accusatorially, accusatory, accuse, accused, accuser, accusing
  ... +20396 từ nữa
```

**10,533 cụm từ chung V∩A** (hiển thị 80 mẫu):
```
  1st place medal
  2nd place medal
  3rd place medal
  A button (blood type)
  AB button (blood type)
  ABACUS
  ACCORDION
  ADHESIVE BANDAGE
  ADI SHAKTI
  ADMISSION TICKETS
  ADULT
  AERIAL TRAMWAY
  AIRPLANE ARRIVING
  AIRPLANE DEPARTURE
  ALARM BELL SYMBOL
  ALCHEMICAL SYMBOL FOR AIR
  ALCHEMICAL SYMBOL FOR ALEMBIC
  ALCHEMICAL SYMBOL FOR ALKALI
  ALCHEMICAL SYMBOL FOR ALKALI-2
  ALCHEMICAL SYMBOL FOR ALUM
  ALCHEMICAL SYMBOL FOR AMALGAM
  ALCHEMICAL SYMBOL FOR ANTIMONY ORE
  ALCHEMICAL SYMBOL FOR AQUA REGIA
  ALCHEMICAL SYMBOL FOR AQUA REGIA-2
  ALCHEMICAL SYMBOL FOR AQUA VITAE
  ALCHEMICAL SYMBOL FOR AQUA VITAE-2
  ALCHEMICAL SYMBOL FOR AQUAFORTIS
  ALCHEMICAL SYMBOL FOR ARSENIC
  ALCHEMICAL SYMBOL FOR ASHES
  ALCHEMICAL SYMBOL FOR AURIPIGMENT
  ALCHEMICAL SYMBOL FOR BATH OF MARY
  ALCHEMICAL SYMBOL FOR BATH OF VAPOURS
  ALCHEMICAL SYMBOL FOR BISMUTH ORE
  ALCHEMICAL SYMBOL FOR BLACK SULFUR
  ALCHEMICAL SYMBOL FOR BORAX
  ALCHEMICAL SYMBOL FOR BORAX-2
  ALCHEMICAL SYMBOL FOR BORAX-3
  ALCHEMICAL SYMBOL FOR BRICK
  ALCHEMICAL SYMBOL FOR CADUCEUS
  ALCHEMICAL SYMBOL FOR CALX
  ALCHEMICAL SYMBOL FOR CAPUT MORTUUM
  ALCHEMICAL SYMBOL FOR CINNABAR
  ALCHEMICAL SYMBOL FOR COPPER ANTIMONIATE
  ALCHEMICAL SYMBOL FOR COPPER ORE
  ALCHEMICAL SYMBOL FOR CROCUS OF COPPER
  ALCHEMICAL SYMBOL FOR CROCUS OF COPPER-2
  ALCHEMICAL SYMBOL FOR CROCUS OF IRON
  ALCHEMICAL SYMBOL FOR CRUCIBLE
  ALCHEMICAL SYMBOL FOR CRUCIBLE-2
  ALCHEMICAL SYMBOL FOR CRUCIBLE-3
  ALCHEMICAL SYMBOL FOR CRUCIBLE-4
  ALCHEMICAL SYMBOL FOR CRUCIBLE-5
  ALCHEMICAL SYMBOL FOR DAY-NIGHT
  ALCHEMICAL SYMBOL FOR DISSOLVE
  ALCHEMICAL SYMBOL FOR DISSOLVE-2
  ALCHEMICAL SYMBOL FOR DISTILL
  ALCHEMICAL SYMBOL FOR EARTH
  ALCHEMICAL SYMBOL FOR FIRE
  ALCHEMICAL SYMBOL FOR GOLD
  ALCHEMICAL SYMBOL FOR GUM
  ALCHEMICAL SYMBOL FOR HALF DRAM
  ALCHEMICAL SYMBOL FOR HALF OUNCE
  ALCHEMICAL SYMBOL FOR HORSE DUNG
  ALCHEMICAL SYMBOL FOR HOUR
  ALCHEMICAL SYMBOL FOR IRON ORE
  ALCHEMICAL SYMBOL FOR IRON ORE-2
  ALCHEMICAL SYMBOL FOR IRON-COPPER ORE
  ALCHEMICAL SYMBOL FOR LEAD ORE
  ALCHEMICAL SYMBOL FOR LODESTONE
  ALCHEMICAL SYMBOL FOR MARCASITE
  ALCHEMICAL SYMBOL FOR MERCURY SUBLIMATE
  ALCHEMICAL SYMBOL FOR MERCURY SUBLIMATE-2
  ALCHEMICAL SYMBOL FOR MERCURY SUBLIMATE-3
  ALCHEMICAL SYMBOL FOR MONTH
  ALCHEMICAL SYMBOL FOR MOON-JUPITER
  ALCHEMICAL SYMBOL FOR NIGHT
  ALCHEMICAL SYMBOL FOR NITRE
  ALCHEMICAL SYMBOL FOR OIL
  ALCHEMICAL SYMBOL FOR OIL INVERTED
  ALCHEMICAL SYMBOL FOR PHILOSOPHERS SULFUR
  ... +10453 cụm từ nữa
```

---

### V — Valence riêng (cảm xúc tích cực/tiêu cực)

**7,990 từ chỉ V** (hiển thị 200 mẫu):
```
  abandon, abandoned, abandonment, abashed, abeyance, abhor, abhorrently, abilities, ability, able
  abloom, abnormal, abnormally, abnormity, abound, aboveground, abracadabra, abroad, abrogate, abscess
  absolutely, absolution, absolve, absolvent, absolver, abstinence, abstractionist, absurdist, absurdity, absurdly
  abundant, abundantly, academic, accede, accentuate, acceptances, accepter, access, accessibility, accessible
  accessibly, accessories, accolade, accommodate, accompaniment, accompany, accompanying, accomplishable, accomplished, accordable
  according, accountability, accountably, accredited, accruement, accurate, accurately, accursedly, accusable, acerbic
  acetic, acetylene, achievable, achieve, achieved, achilles, achiness, acidic, acidification, acidly
  acknowledgeable, acknowledged, acknowledgement, acknowledgment, acne, acolyte, acquainted, acquire, acquiring, acquisition
  acquittal, acreage, actor, actual, actuality, adamant, adapt, adaptability, adaptable, adaptation
  add, added, additional, addressable, adeptness, adequately, adhd, adieu, adipose, adjusted
  admin, administrable, admirable, admiral, admiralship, admiration, admired, admiring, admissibility, admissible
  admittance, admitted, adopt, adoptability, adoptable, adoptively, adorable, adore, adoring, adorn
  adornment, adulatory, adulterant, adulteration, advanced, advancement, advancing, advantageous, advantageously, advent
  adverse, advice, advisable, advise, adviser, advisory, aeronautically, aeroplane, aesthetic, aesthetically
  affable, affair, affectation, affectionate, affections, affiliated, affinitive, affinity, affirm, affirmable
  affirmation, affirmative, affirmatively, afflict, afflicted, affliction, affluence, affluent, affordability, affordable
  afro, aftermath, again, aged, agent, ages, aggregate, aggrieved, agh, aghhh
  aglow, agnostic, agreeable, agreed, agreeing, agreement, agriculture, agrochemical, ahead, ailing
  airhead, airworthy, airy, alamo, ale, alienate, align, alimentation, alkalize, all-star
  allegedly, allegiance, allegorist, allergenic, allergies, allergy, alley, alliance, alliteration, alliterative
  ... +7790 từ nữa
```

**3,319 cụm từ chỉ V** (hiển thị 80 mẫu):
```
  a good
  a kickback
  a mere
  a peach
  a pleasure
  a plus
  a sec
  a shame
  a smokescreen
  a student
  a treat
  about face
  above all
  above water
  absolutely not
  academic freedom
  academic year
  according to
  accused of
  achieve success
  achilles heel
  acting out
  ad hominem
  ad nauseam
  add on
  added value
  advance poll
  advent calendar
  agent orange
  agree on
  agree to
  agree with
  agreed upon
  air conditioned
  air duct
  air flow
  air freshener
  air pollution
  air raid
  air time
  alive with
  all clear
  all correct
  all downhill
  all important
  all inclusive
  all nations
  all right
  all round
  all set
  all together
  all wet
  all-star game
  allow for
  allow in
  american dream
  american express
  american flag
  american thanksgiving
  american way
  amusement park
  animal kingdom
  animal lover
  anime series
  any old
  anything but
  apartment building
  april fool
  argue over
  argue with
  art director
  art gallery
  art history
  art museum
  art school
  art teacher
  as possible
  ash tray
  at bay
  at best
  ... +3239 cụm từ nữa
```

---

### A — Arousal riêng (cường độ kích thích)

**8,472 từ chỉ A** (hiển thị 200 mẫu):
```
  abatement, abbot, abbreviation, abbreviator, abdominally, abiotic, abjudicate, abode, abolishable, abolishment
  aboriginality, abovementioned, abridge, abridged, abridgement, abridgment, absenteeism, absently, absentmindedly, absolutism
  absorbedly, absorbedness, absorber, absorptive, abstainer, abstentious, abstract, abstractedly, abstractedness, abstracter
  abstractly, abstractness, abstractor, abutment, abutting, abyss, accelerant, accelerate, accelerated, accelerator
  accelerometer, accessory, acclaim, accordionist, accountancy, accounts, accumulative, accumulatively, accumulator, accustom
  acetate, achromatic, acorn, acoustical, acoustics, acquiesce, acquired, acquirer, acquisitive, acquisitively
  acronymic, across, acrylic, actin, activist, actuarial, actuate, actuation, actuator, acumen
  acupuncture, adamantly, adaptor, additionally, addressbook, addresser, adhere, adherently, adios, adjacent
  adjacently, adjective, adjectively, adjudicator, adjunctive, adjure, adjust, administrant, administrate, administrational
  administratively, admissibly, admitter, adobe, adolescence, adolescently, adrenal, adverbial, advert, advertently
  advertiser, advertize, advisee, aerially, aerobiological, aerobiology, aerodrome, aerodynamic, aerodynamics, aeromechanical
  aeromechanics, aerostatic, affectively, affiche, affix, affixation, affixer, affronted, afire, aflame
  afoot, afore, aforementioned, aforethought, afterburner, afterburning, aftergame, afterhours, aftermarket, afterschool
  aftertime, afterward, afterword, aga, agedness, agencies, agglomeration, aggregative, aggregator, agitated
  agnostically, agnosticism, agribusiness, agriculturalist, agriculturally, agua, ahem, airbed, airboat, aircraft
  aircraftman, aircraftsman, aircrew, airdrop, airfreight, airline, airman, airs, airspeed, airstrip
  airwave, aisle, akimbo, akin, alarmclock, alas, albinism, albino, alchemically, alchemistic
  alchemize, alcove, alehouse, alertness, alfalfa, algae, algebra, algebraically, algorithmic, alikeness
  aliquot, alkaline, alkalinity, alkaloid, allegeable, allegoric, allegorically, allegorize, allergist, alligator
  allocative, allowably, allspice, allude, allusion, allusively, along, alongside, aloud, alphabet
  ... +8272 từ nữa
```

**641 cụm từ chỉ A** (hiển thị 80 mẫu):
```
  a bunch
  a few
  a little
  acting up
  action figure
  action film
  active duty
  active list
  adventure game
  affiliate marketing
  air force
  aircraft carrier
  airline industry
  altar boy
  american revolution
  and so
  arm chair
  arm wrestling
  armed forces
  arrested development
  artificial intelligence
  as if
  as per
  as to
  as usual
  at ease
  at rest
  attention span
  augmented reality
  baby food
  baby oil
  bachelor party
  back in
  back porch
  baseball season
  based on
  basketball game
  basketball league
  basketball player
  basketball season
  bath time
  bath tub
  battle royal
  be full
  be one
  be quiet
  bed bath
  bed frame
  bed in
  bedside table
  beef stew
  bench press
  big band
  big old
  big shot
  black tea
  blend in
  blood test
  blue moon
  blue spruce
  blue suit
  body builder
  body guard
  boiled egg
  booster dose
  booster shot
  boot camp
  boot up
  bottom line
  bounce around
  bow tie
  boxing gloves
  boxing match
  boxing ring
  bragging rights
  brain teaser
  breaking and entering
  breaking news
  brief time
  brown rice
  ... +561 cụm từ nữa
```
---

### T — Time (MUSICAL) · Từ vựng thời gian / nhịp / dao động

**433 từ riêng biệt** (chỉ xuất hiện trong T):
```
  accumulation, agogi, alli, allo, alta, anatrichisma, ano, antifonia, antikenokylisma, antikenoma
  apeso, apli, aploun, apoderma, apodexia, apostrofoi, apostrofos, apothema, apothes, approach
  archaion, argi, argosyntheton, argoteri, aristera, arktiko, arousing, arpeggiato, arseos, attaching
  barline, bassa, before, borzaya, borzy, brace, branching, brevis, byzantine, caesura
  capo, cauldron, chamili, chamilon, chashka, chelnu, chelyustka, chorevma, chroa, chroma
  chronon, chronou, clef, climacus, clinging, clivis, cluster, coda, contention, contrariety
  croix, cum, curlew, darkening, daseia, demestvenny, derbitsa, dexia, deyteros, deyterou
  di, diacritic, diargon, diastoli, diatoniki, diatonon, diesis, difonias, diftoggos, digorgon
  digramma, digrammos, dimming, dipli, diploun, disimou, divergence, dodekata, doit, duda
  dva, dvoechelnaya, dvoechelnokryzhevaya, dvoechelnopovodnaya, dvoetochie, dvumya, dyo, ekfonitikon, ekstrepton, elafron
  enarmonios, enarxis, encounters, endeavour, endofonon, enos, epegerma, eso, eteron, exo
  fanerosis, fermata, fhtora, fingered, fingernails, fita, flexus, fretboard, fthora, fusa
  ga, geniki, gg, glissando, golubchik, gorazdo, gorgi, gorgon, gorgosyntheton, gorgoteri
  gorthmikon, gregorian, gromnaya, gromokryzhevaya, gromopovodnaya, gronthismata, hardness, hauptstimme, henze, hexagram
  ichadin, ichimatos, ichos, ichou, imidiargon, imifonon, imifthora, imifthoron, imiseos, imperfecta
  imperfectum, isakia, ison, kachka, kai, katava, katavasma, kathisti, kato, ke
  kentima, kentimata, khamilo, khokhlom, kievan, klasma, kliton, klyuch, klyuchenepostoyannaya, klyuchenepostoyanny
  klyuchepovodnaya, klyuchepovodny, klyuchevaya, klyuchevoy, kobyla, kontevma, koron, koronis, koufisma, kratima
  kratimata, kratimokoufisma, kratimoyporroon, kremasti, kryuk, kryukovaya, kryzh, kryzhem, kryzhevaya, kufisma
  kupnaya, kylisma, labouring, legetos, legion, leimma, lemoi, lomka, longa, loure
  lygisma, malakon, malo, marcato, marrying, martyria, massing, maxima, measure, mechik
  megali, mesi, meso, meta, metria, mezzo, mikri, mikron, minima, mired
  monofonias, monogrammos, mrachnaya, mrachno, mrachnotikhaya, mrachny, multi, naos, nebenstimme, nemka
  nenano, neo, nepostoyannaya, neume, ni, niente, nizko, notehead, nozhka, oblachko
  oblako, ochkom, okto, oligon, omalon, omet, optionally, organ, osoka, otsechka
  ottava, ou, oxeia, oxeiai, oyranisma, packing, palka, parakalesma, paraklit, paraklitiki
  parestigmenon, parichon, pauk, pelaston, perevodka, perfecta, perfectum, perispomeni, pes, petasma
  petasti, petastokoufisma, piasma, pizzicato, plagios, podatus, podchashie, podchashiem, podvertka, poli
  polkulizmy, polnaya, polupovodnaya, porrectus, povodnaya, povodny, povyshe, preponderance, priznak, prolatione
  prostaya, protos, protovarys, psifistolygisma, psifiston, psifistoparakalesma, psifistosynagma, psili, psilon, putnaya
  quindicesima, rapisma, ravno, razseka, repeated, residence, resupinus, retreat, revma, rinforzando
  ritual, rog, rogom, salzedo, saximata, scandicus, segno, seisma, semibrevis, semiminima
  severance, simansis, skameytsa, skliron, skoba, slozhitie, sori, sorochya, spathi, sprechgesang
  sredne, staccatissimo, staccato, statya, stavros, stavrou, stimme, stopitsa, straggismata, stranno
  strela, subito, subpunctis, svetlaya, svetlo, svetly, synafi, synagma, syndesmos, synevma
  syrma, syrmatiki, tablature, taming, teleia, telous, tempus, tenuto, tessaron, tessera
  tetartimorion, tetartos, tetrafonias, tetragram, tetrapli, tetrasimou, thema, thematismos, thes, theseos
  thita, tikhaya, tikhy, tinagma, tochka, tonal, torculus, tr, treading, tresvetlaya
  tresvetlo, tresvetly, tria, trifonias, trigorgon, trigrammos, trion, tripli, trisimou, tritimorion
  tritos, tromikolygisma, tromikon, tromikoparakalesma, tromikopsifiston, tromikosynagma, tryaska, tryasoglasnaya, tryasopovodnaya, tryasostrelnaya
  tsata, udarka, unstress, vareia, vareiai, varys, vasis, vathy, virga, vocal
  vou, vrakhiya, vysoko, wanderer, xiron, yfen, yfesis, ypokrisis, yporroi, ypsili
  zaderzhka, zakrytaya, zakrytoe, zanozhek, zapyataya, zapyatoy, zapyatymi, zelo, zevok, zmeytsa
  znamenny, zo, zygos
```

**713 từ tổng**:
```
  abundance, abysmal, accent, accumulation, advance, after, aggravation, agogi, all, alli, allo, alta
  anatrichisma, and, ano, antifonia, antikenokylisma, antikenoma, apart, apeso, apli, aploun, apoderma, apodexia
  apostrofoi, apostrofos, apothema, apothes, approach, archaion, argi, argon, argosyntheton, argoteri, aristera, arktiko
  army, arousing, arpeggiato, arrow, arseos, ascent, at, attaching, augmentation, back, barline, barrier
  bassa, beam, before, begin, beginning, behind, below, bend, biting, black, bold, borzaya
  borzy, bow, brace, bracket, branching, breakthrough, breath, breve, brevis, bridge, buzz, byzantine
  caesura, capo, cauldron, centre, chamili, chamilon, change, chashka, chelnu, chelyustka, chorevma, chroa
  chroma, chronon, chronou, circle, clef, climacus, clinging, clivis, closed, closeness, closure, cluster
  coda, combining, coming, common, completion, compliance, conflict, constancy, contact, contemplation, contention, contrariety
  corners, creative, crescendo, croix, cum, curlew, curved, cut, da, dal, damp, darkening
  daseia, dashed, decayed, decisiveness, decrease, decrescendo, defectiveness, degree, deliverance, demestvenny, departure, derbitsa
  development, dexia, deyteros, deyterou, di, diacritic, diamond, diargon, diastoli, diatoniki, diatonon, diesis
  difficulties, difficulty, difonias, diftoggos, digit, digorgon, digram, digramma, digrammos, diminishment, dimming, dipli
  diploun, direction, disimou, dispersion, distortion, divergence, divider, dodekata, doit, dot, dots, double
  doubt, down, drum, duda, duration, duties, dva, dvoechelnaya, dvoechelnokryzhevaya, dvoechelnopovodnaya, dvoetochie, dvumya
  dyo, earth, earthly, ease, eighth, ekfonitikon, ekstrepton, elafron, embellishment, enarmonios, enarxis, encounters
  end, endeavour, endofonon, enlargement, enos, enthusiasm, epegerma, eso, eternity, eteron, ex, exhaustion
  exo, failure, family, fanerosis, fellowship, fermata, fhtora, final, fingered, fingernails, fire, fita
  five, flat, flexus, flight, flip, following, folly, for, forte, fostering, four, fretboard
  fthora, full, fullness, fusa, ga, gathering, geniki, gentle, gg, glissando, going, golubchik
  gorazdo, gorgi, gorgon, gorgosyntheton, gorgoteri, gorthmikon, grace, great, greatness, greek, gregorian, gromnaya
  gromokryzhevaya, gromopovodnaya, gronthismata, guardedness, half, hardness, harmonic, hauptstimme, heaven, heavenly, heavy, heel
  henze, hexagram, holding, human, hundred, ichadin, ichimatos, ichos, ichou, imidiargon, imifonon, imifthora
  imifthoron, imiseos, imperfecta, imperfectum, in, increase, indicator, influence, inner, innocence, instrumental, inverted
  isakia, ison, joy, joyous, kachka, kai, katava, katavasma, kathisti, kato, ke, keeping
  kentima, kentimata, khamilo, khokhlom, kievan, kinship, klasma, kliton, klyuch, klyuchenepostoyannaya, klyuchenepostoyanny, klyuchepovodnaya
  klyuchepovodny, klyuchevaya, klyuchevoy, kobyla, kontevma, koron, koronis, koufisma, kratima, kratimata, kratimokoufisma, kratimoyporroon
  kremasti, kryuk, kryukovaya, kryzh, kryzhem, kryzhevaya, kufisma, kupnaya, kylisma, labouring, lake, law
  left, legetos, legion, leimma, lemoi, light, limitation, lomka, long, longa, loure, low
  lower, lygisma, maiden, malakon, malo, marcato, mark, marrying, martyria, massing, maxima, measure
  mechik, meet, megali, mesi, meso, meta, metria, mezzo, mikri, mikron, minima, mired
  model, modesty, modifier, monofonias, monogram, monogrammos, moon, mountain, mouth, mrachnaya, mrachno, mrachnotikhaya
  mrachny, multi, multiple, musical, nana, naos, natural, nebenstimme, nemka, nenano, neo, nepostoyannaya
  neume, ni, niente, nine, nizko, no, notation, note, notehead, nozhka, null, oblachko
  oblako, obstruction, ochkom, of, okto, oligon, omalon, omet, on, one, opposition, oppression
  optionally, or, organ, ornament, osoka, otsechka, ottava, ou, out, oxeia, oxeiai, oyranisma
  pa, packing, palka, parakalesma, paraklit, paraklitiki, parenthesis, parestigmenon, parichon, pattern, pauk, peace
  pedal, pelaston, penetration, pentaseme, perevodka, perfecta, perfectum, perispomeni, pes, petasma, petasti, petastokoufisma
  phrase, piano, piasma, piece, pizzicato, plagios, plus, podatus, podchashie, podchashiem, podvertka, poli
  polkulizmy, polnaya, polupovodnaya, porrectus, possession, povodnaya, povodny, povyshe, power, preponderance, priznak, progress
  prolatione, prostaya, protos, protovarys, psifistolygisma, psifiston, psifistoparakalesma, psifistosynagma, psili, psilon, purity, pushing
  putnaya, quarter, quindicesima, range, rapisma, ravno, razseka, reach, receptive, recitative, release, repeat
  repeated, residence, resistance, resolution, response, rest, resupinus, retreat, return, reverse, reversed, revma
  revolution, right, rinforzando, ring, rip, ritual, rog, rogom, roll, salzedo, saximata, scandicus
  segno, seisma, semibrevis, semiminima, seven, severance, sharp, short, sign, simansis, single, sinking
  six, sixteenth, skameytsa, skliron, skoba, slash, slozhitie, slur, small, smear, snap, sori
  sorochya, spathi, splitting, sprechgesang, square, sredne, staccatissimo, staccato, staff, standstill, statya, stavros
  stavrou, stem, stigma, still, stimme, stopitsa, stoppage, stove, straggismata, stranno, strela, strength
  stress, stroke, subito, subpunctis, suspension, svetlaya, svetlo, svetly, symbol, synafi, synagma, syndesmos
  synevma, syrma, syrmatiki, system, tablature, taming, teleia, telous, tempus, tenuto, tessaron, tessera
  tetartimorion, tetartos, tetrafonias, tetragram, tetrapli, tetraseme, tetrasimou, the, thema, thematismos, thes, theseos
  thita, thousand, through, thunder, tie, tikhaya, tikhy, time, tinagma, to, tochka, toe
  together, tonal, tone, tongue, top, torculus, tr, treading, tresvetlaya, tresvetlo, tresvetly, tria
  triangle, triangular, trifonias, trigorgon, trigrammos, trion, triple, tripli, triseme, trisimou, tritimorion, tritos
  tromikolygisma, tromikon, tromikoparakalesma, tromikopsifiston, tromikosynagma, truth, tryaska, tryasoglasnaya, tryasopovodnaya, tryasostrelnaya, tsata, turn
  twelfth, two, udarka, unity, unstress, up, upward, vareia, vareiai, varys, vasis, vastness
  vathy, verge, vertical, virga, vocal, void, vou, vrakhiya, vysoko, waiting, wanderer, wasting
  watch, water, well, white, whole, wind, with, work, xiron, yfen, yfesis, youthful
  youthfulness, ypokrisis, yporroi, ypsili, zaderzhka, zakrytaya, zakrytoe, zanozhek, zapyataya, zapyatoy, zapyatymi, zelo
  zevok, zmeytsa, znamenny, zo, zygos
```

**Cụm từ riêng T** (958 cụm, hiển thị 80 mẫu):
```
  BYZANTINE MUSICAL SYMBOL AGOGI ARGI
  BYZANTINE MUSICAL SYMBOL AGOGI ARGOTERI
  BYZANTINE MUSICAL SYMBOL AGOGI GORGI
  BYZANTINE MUSICAL SYMBOL AGOGI GORGOTERI
  BYZANTINE MUSICAL SYMBOL AGOGI MESI
  BYZANTINE MUSICAL SYMBOL AGOGI METRIA
  BYZANTINE MUSICAL SYMBOL AGOGI POLI ARGI
  BYZANTINE MUSICAL SYMBOL AGOGI POLI GORGI
  BYZANTINE MUSICAL SYMBOL ANATRICHISMA
  BYZANTINE MUSICAL SYMBOL ANTIKENOKYLISMA
  BYZANTINE MUSICAL SYMBOL ANTIKENOMA
  BYZANTINE MUSICAL SYMBOL APESO EKFONITIKON
  BYZANTINE MUSICAL SYMBOL APESO EXO NEO
  BYZANTINE MUSICAL SYMBOL APLI
  BYZANTINE MUSICAL SYMBOL APODERMA ARCHAION
  BYZANTINE MUSICAL SYMBOL APODERMA NEO
  BYZANTINE MUSICAL SYMBOL APOSTROFOI SYNDESMOS NEO
  BYZANTINE MUSICAL SYMBOL APOSTROFOI TELOUS ICHIMATOS
  BYZANTINE MUSICAL SYMBOL APOSTROFOS
  BYZANTINE MUSICAL SYMBOL APOSTROFOS DIPLI
  BYZANTINE MUSICAL SYMBOL APOSTROFOS NEO
  BYZANTINE MUSICAL SYMBOL APOTHEMA
  BYZANTINE MUSICAL SYMBOL ARGON
  BYZANTINE MUSICAL SYMBOL ARGOSYNTHETON
  BYZANTINE MUSICAL SYMBOL ARKTIKO DI
  BYZANTINE MUSICAL SYMBOL ARKTIKO GA
  BYZANTINE MUSICAL SYMBOL ARKTIKO KE
  BYZANTINE MUSICAL SYMBOL ARKTIKO NI
  BYZANTINE MUSICAL SYMBOL ARKTIKO PA
  BYZANTINE MUSICAL SYMBOL ARKTIKO VOU
  BYZANTINE MUSICAL SYMBOL ARKTIKO ZO
  BYZANTINE MUSICAL SYMBOL CHAMILI
  BYZANTINE MUSICAL SYMBOL CHAMILON
  BYZANTINE MUSICAL SYMBOL CHOREVMA ARCHAION
  BYZANTINE MUSICAL SYMBOL CHOREVMA NEO
  BYZANTINE MUSICAL SYMBOL CHROA KLITON
  BYZANTINE MUSICAL SYMBOL CHROA SPATHI
  BYZANTINE MUSICAL SYMBOL CHROA ZYGOS
  BYZANTINE MUSICAL SYMBOL DASEIA
  BYZANTINE MUSICAL SYMBOL DIARGON
  BYZANTINE MUSICAL SYMBOL DIASTOLI APLI MEGALI
  BYZANTINE MUSICAL SYMBOL DIASTOLI APLI MIKRI
  BYZANTINE MUSICAL SYMBOL DIASTOLI DIPLI
  BYZANTINE MUSICAL SYMBOL DIASTOLI THESEOS
  BYZANTINE MUSICAL SYMBOL DIESIS APLI DYO DODEKATA
  BYZANTINE MUSICAL SYMBOL DIESIS DIGRAMMOS EX DODEKATA
  BYZANTINE MUSICAL SYMBOL DIESIS MONOGRAMMOS TESSERA DODEKATA
  BYZANTINE MUSICAL SYMBOL DIESIS TETARTIMORION
  BYZANTINE MUSICAL SYMBOL DIESIS TRIGRAMMOS OKTO DODEKATA
  BYZANTINE MUSICAL SYMBOL DIESIS TRITIMORION
  BYZANTINE MUSICAL SYMBOL DIFTOGGOS OU
  BYZANTINE MUSICAL SYMBOL DIGORGON
  BYZANTINE MUSICAL SYMBOL DIGORGON PARESTIGMENON ARISTERA ANO
  BYZANTINE MUSICAL SYMBOL DIGORGON PARESTIGMENON ARISTERA KATO
  BYZANTINE MUSICAL SYMBOL DIGORGON PARESTIGMENON DEXIA
  BYZANTINE MUSICAL SYMBOL DIGRAMMA GG
  BYZANTINE MUSICAL SYMBOL DIPLI
  BYZANTINE MUSICAL SYMBOL DIPLI ARCHAION
  BYZANTINE MUSICAL SYMBOL DYO
  BYZANTINE MUSICAL SYMBOL EKSTREPTON
  BYZANTINE MUSICAL SYMBOL ELAFRON
  BYZANTINE MUSICAL SYMBOL ENARXIS KAI FTHORA VOU
  BYZANTINE MUSICAL SYMBOL ENDOFONON
  BYZANTINE MUSICAL SYMBOL EPEGERMA
  BYZANTINE MUSICAL SYMBOL ETERON ARGOSYNTHETON
  BYZANTINE MUSICAL SYMBOL ETERON PARAKALESMA
  BYZANTINE MUSICAL SYMBOL EXO EKFONITIKON
  BYZANTINE MUSICAL SYMBOL FANEROSIS DIFONIAS
  BYZANTINE MUSICAL SYMBOL FANEROSIS MONOFONIAS
  BYZANTINE MUSICAL SYMBOL FANEROSIS TETRAFONIAS
  BYZANTINE MUSICAL SYMBOL FHTORA SKLIRON CHROMA VASIS
  BYZANTINE MUSICAL SYMBOL FTHORA ARCHAION
  BYZANTINE MUSICAL SYMBOL FTHORA ARCHAION DEYTEROU ICHOU
  BYZANTINE MUSICAL SYMBOL FTHORA DIATONIKI DI
  BYZANTINE MUSICAL SYMBOL FTHORA DIATONIKI KE
  BYZANTINE MUSICAL SYMBOL FTHORA DIATONIKI NANA
  BYZANTINE MUSICAL SYMBOL FTHORA DIATONIKI NI ANO
  BYZANTINE MUSICAL SYMBOL FTHORA DIATONIKI NI KATO
  BYZANTINE MUSICAL SYMBOL FTHORA DIATONIKI PA
  BYZANTINE MUSICAL SYMBOL FTHORA DIATONIKI ZO
  ... +878 cụm từ nữa
```

**Câu riêng T** (791 câu, hiển thị 50 mẫu):
```
  BYZANTINE MUSICAL SYMBOL AGOGI ARGI
  BYZANTINE MUSICAL SYMBOL AGOGI ARGOTERI
  BYZANTINE MUSICAL SYMBOL AGOGI GORGI
  BYZANTINE MUSICAL SYMBOL AGOGI GORGOTERI
  BYZANTINE MUSICAL SYMBOL AGOGI MESI
  BYZANTINE MUSICAL SYMBOL AGOGI METRIA
  BYZANTINE MUSICAL SYMBOL AGOGI POLI ARGI
  BYZANTINE MUSICAL SYMBOL AGOGI POLI GORGI
  BYZANTINE MUSICAL SYMBOL ANATRICHISMA
  BYZANTINE MUSICAL SYMBOL ANTIKENOKYLISMA
  BYZANTINE MUSICAL SYMBOL ANTIKENOMA
  BYZANTINE MUSICAL SYMBOL APESO EKFONITIKON
  BYZANTINE MUSICAL SYMBOL APESO EXO NEO
  BYZANTINE MUSICAL SYMBOL APLI
  BYZANTINE MUSICAL SYMBOL APODERMA ARCHAION
  BYZANTINE MUSICAL SYMBOL APODERMA NEO
  BYZANTINE MUSICAL SYMBOL APOSTROFOI SYNDESMOS NEO
  BYZANTINE MUSICAL SYMBOL APOSTROFOI TELOUS ICHIMATOS
  BYZANTINE MUSICAL SYMBOL APOSTROFOS
  BYZANTINE MUSICAL SYMBOL APOSTROFOS DIPLI
  BYZANTINE MUSICAL SYMBOL APOSTROFOS NEO
  BYZANTINE MUSICAL SYMBOL APOTHEMA
  BYZANTINE MUSICAL SYMBOL ARGON
  BYZANTINE MUSICAL SYMBOL ARGOSYNTHETON
  BYZANTINE MUSICAL SYMBOL ARKTIKO DI
  BYZANTINE MUSICAL SYMBOL ARKTIKO GA
  BYZANTINE MUSICAL SYMBOL ARKTIKO KE
  BYZANTINE MUSICAL SYMBOL ARKTIKO NI
  BYZANTINE MUSICAL SYMBOL ARKTIKO PA
  BYZANTINE MUSICAL SYMBOL ARKTIKO VOU
  BYZANTINE MUSICAL SYMBOL ARKTIKO ZO
  BYZANTINE MUSICAL SYMBOL CHAMILI
  BYZANTINE MUSICAL SYMBOL CHAMILON
  BYZANTINE MUSICAL SYMBOL CHOREVMA ARCHAION
  BYZANTINE MUSICAL SYMBOL CHOREVMA NEO
  BYZANTINE MUSICAL SYMBOL CHROA KLITON
  BYZANTINE MUSICAL SYMBOL CHROA SPATHI
  BYZANTINE MUSICAL SYMBOL CHROA ZYGOS
  BYZANTINE MUSICAL SYMBOL DASEIA
  BYZANTINE MUSICAL SYMBOL DIARGON
  BYZANTINE MUSICAL SYMBOL DIASTOLI APLI MEGALI
  BYZANTINE MUSICAL SYMBOL DIASTOLI APLI MIKRI
  BYZANTINE MUSICAL SYMBOL DIASTOLI DIPLI
  BYZANTINE MUSICAL SYMBOL DIASTOLI THESEOS
  BYZANTINE MUSICAL SYMBOL DIESIS APLI DYO DODEKATA
  BYZANTINE MUSICAL SYMBOL DIESIS DIGRAMMOS EX DODEKATA
  BYZANTINE MUSICAL SYMBOL DIESIS MONOGRAMMOS TESSERA DODEKATA
  BYZANTINE MUSICAL SYMBOL DIESIS TETARTIMORION
  BYZANTINE MUSICAL SYMBOL DIESIS TRIGRAMMOS OKTO DODEKATA
  BYZANTINE MUSICAL SYMBOL DIESIS TRITIMORION
  ... +741 câu nữa
```