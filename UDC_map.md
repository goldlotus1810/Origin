# UDC Map — 9,584 Unicode Dimensional Coordinates

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
  13 blk      21 blk     ──17 blk──   ──shared──    7 blk
  1,904 cp    3,088 cp      3,568 cp                1,024 cp
     |           |            |            |            |
  P_weight = [S] [R] [V] [A] [T]  = 2 bytes = tọa độ 5D
              4b   4b  3b  3b  2b    SEALED tại bootstrap
```

**Tổng: 58 blocks · 9,584 codepoints = L0 anchor points**

---

## Nguồn dữ liệu × Chiều

| File nguồn | S (Shape) | R (Relation) | V (Valence) | A (Arousal) | T (Time) |
|-------------|-----------|--------------|-------------|-------------|----------|
| `UnicodeData.txt` | Char names: ARROW, BOX DRAWINGS, GEOMETRIC, BRAILLE | Char names: MATHEMATICAL, SUPERSCRIPT, NUMERAL, FRACTION | Char names: FACE WITH, SMILING, HEART, PLAYING CARD | (shared V) | Char names: HEXAGRAM, MUSICAL SYMBOL, BYZANTINE, TETRAGRAM |
| `Blocks.txt` | 13 block ranges (2190→1F8FF) | 21 block ranges (2000→2E7F) | 17 block ranges (2460→1DBFF) | (shared V) | 7 block ranges (4DC0→1D35F) |
| `PropList.txt` | `Pattern_Syntax`, `Other_Math` | `Other_Math`, `ID_Compat_Math_*`, `Other_Lowercase/Uppercase` | `Other_Alphabetic`, `Regional_Indicator` | (shared V) | `Diacritic`, `Other_Grapheme_Extend` |
| `NameAliases.txt` | 2 corrections (Arrows) | 1 correction (WEIERSTRASS) | — | — | 1 correction (BYZANTINE) |
| `StandardizedVariants.txt` | — | 87 variants (chancery, serifs, slant) | — | — | — |
| `emoji-data.txt` | `Emoji`(76), `Extended_Pictographic`(175) | `Emoji`(2) | `Emoji`(1335), `Emoji_Presentation`(1166), `Emoji_Modifier_Base`(132) | (shared V) | — |
| `emoji-test.txt` | 113 fully-qualified (arrow, av-symbol, geometric) | 2 fully-qualified | 3,830 fully-qualified (face, person, animal, transport, flag) | (shared V) | — |
| `PropertyValueAliases.txt` | `gc=So` (Other_Symbol), `gc=Sm` (Math_Symbol) | `gc=Sm`, `gc=Nl` (Letter_Number), `gc=No` (Other_Number) | `gc=So` (Other_Symbol) | (shared V) | `gc=So`, `gc=Mn` (Nonspacing_Mark) |
| `mapping/NRC-VAD-Lexicon` | — | — | 54,801 terms × valence score (-1→+1) | 54,801 terms × arousal score (-1→+1) | — |

---

## P_S — Shape (SDF) · 13 Blocks · 1,904 codepoints

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

**S tổng: 13 blocks · range 1,904 · actual assigned 1,809**
**gc chủ đạo: `So` (Other_Symbol) + `Sm` (Math_Symbol)**
**PropList chủ đạo: `Pattern_Syntax` — ký hiệu hình học dùng trong pattern matching**

---

## P_R — Relation (MATH) · 21 Blocks · 3,088 codepoints

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

**R tổng: 21 blocks · range 3,088 · actual assigned ~2,717**
**gc chủ đạo: `Sm` (Math_Symbol) + `No` (Other_Number) + `Nl` (Letter_Number)**
**PropList chủ đạo: `Other_Math`, `Pattern_Syntax` — quan hệ toán học logic**
**StandardizedVariants: 87 biến thể (chancery style, serifs, slant, vertical stroke)**

---

## P_V + P_A — Valence + Arousal (EMOTICON) · 17 Blocks · 3,568 codepoints

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

**V+A tổng: 17 blocks · range 3,568 · actual assigned ~2,821**
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

## P_T — Time (MUSICAL) · 7 Blocks · 1,024 codepoints

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

**T tổng: 7 blocks · range 1,024 · actual assigned 958**
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
                          o{9,584 L0 anchors}
     ─────────────────────────┼─────────────────────────
     |           |            |            |            |
     S           R            V            A            T
   13 blk      21 blk       ──17 blk──                7 blk
   1,904       3,088          3,568                   1,024
   1,809*      2,717*         2,821*                    958*
     |           |            |            |            |
  Pattern     Other_Math    Emoji       NRC-VAD      Diacritic
  _Syntax     Math_Symbol   Emoji_Pres  Lexicon      Other_
  So+Sm       Sm+No+Nl      So          54,801terms  Grapheme
                                         V/A scores   Extend

  * actual assigned (UnicodeData.txt) vs range (Blocks.txt)
  * Δ = 9,584 - 8,305 = 1,279 unassigned slots trong ranges

Nguồn:
  UnicodeData.txt          → char names (TEXT tên ký tự = tên node)
  Blocks.txt               → block ranges (58 blocks)
  PropList.txt             → Pattern_Syntax, Other_Math, Diacritic...
  NameAliases.txt          → 4 corrections
  StandardizedVariants.txt → 87 glyph variants (chủ yếu MATH)
  emoji-data.txt           → Emoji/Presentation/Modifier_Base flags
  emoji-test.txt           → semantic subgroups (face, person, animal...)
  PropertyValueAliases.txt → gc=Sm/So/No/Nl/Mn category mapping
  PropertyAliases.txt      → property name → abbreviation
  mapping/NRC-VAD-Lexicon  → 54,801 terms × (valence, arousal, dominance)
```
