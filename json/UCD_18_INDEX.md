# Unicode Character Database 18.0.0 — Index & Reference

> Source: `Json.zip` (giải nén) | Version: Unicode 18.0.0 | Date: 2026-02-05
>
> File này là tài liệu tham chiếu cho thư mục `json/`.
> Mỗi file đều là plain-text từ Unicode Consortium, dùng cho HomeOS `ucd` crate.

---

## Tổng quan cấu trúc

```
json/
├── UnicodeData.txt              ★ CORE — dữ liệu chính mọi codepoint
├── Blocks.txt                   ★ CORE — phân vùng block
├── Scripts.txt                  ★ CORE — script assignment
├── PropList.txt                 ★ CORE — binary properties
├── DerivedCoreProperties.txt    ★ CORE — derived: Math, Alphabetic, ID_Start...
├── PropertyAliases.txt          ★ META — tên viết tắt ↔ tên đầy đủ property
├── PropertyValueAliases.txt     ★ META — giá trị viết tắt ↔ đầy đủ
├── NameAliases.txt              ☆ TÊN  — alias chính thức (correction, control, figment)
├── NamesList.txt                ☆ TÊN  — human-readable, có annotation
├── Index.txt                    ☆ TÊN  — reverse index: tên → codepoint
├── NamedSequences.txt           ☆ TÊN  — named sequences (chuỗi có tên)
├── NamedSequencesProv.txt       ☆ TÊN  — provisional named sequences
├── DerivedAge.txt               ○ PHỤ  — version mà codepoint được thêm
├── emoji/
│   ├── emoji-data.txt           ★ EMOJI — Emoji/Emoji_Presentation/...
│   ├── emoji-variation-sequences.txt  ○ EMOJI — text vs emoji presentation
│   └── ReadMe.txt
├── auxiliary/
│   ├── GraphemeBreakProperty.txt ○ SEGM — grapheme cluster break
│   ├── SentenceBreakProperty.txt ○ SEGM — sentence break
│   └── WordBreakProperty.txt    ○ SEGM — word break
├── extracted/                   ○ DERIVED — extracted từ UnicodeData.txt
│   ├── DerivedGeneralCategory.txt
│   ├── DerivedName.txt
│   ├── DerivedBidiClass.txt
│   ├── DerivedCombiningClass.txt
│   ├── DerivedDecompositionType.txt
│   ├── DerivedEastAsianWidth.txt
│   ├── DerivedJoiningGroup.txt
│   ├── DerivedJoiningType.txt
│   ├── DerivedLineBreak.txt
│   ├── DerivedNumericType.txt
│   └── DerivedNumericValues.txt
├── ArabicShaping.txt            ○ BIDI  — joining type/group (Arabic, Syriac...)
├── BidiBrackets.txt             ○ BIDI  — bracket pairs for bidi
├── BidiMirroring.txt            ○ BIDI  — mirrored glyphs
├── CaseFolding.txt              ○ CASE  — case fold mapping
├── SpecialCasing.txt            ○ CASE  — context-dependent casing
├── CJKRadicals.txt              ○ CJK   — CJK radical mapping
├── CompositionExclusions.txt    ○ NORM  — normalization exclusions
├── NormalizationCorrections.txt ○ NORM  — corrections to normalization
├── DerivedNormalizationProps.txt ○ NORM  — NFC/NFD/NFKC/NFKD properties
├── DoNotEmit.txt                ○ EMIT  — sequences không nên sinh ra
├── EastAsianWidth.txt           ○ WIDTH — display width (Narrow/Wide/...)
├── EmojiSources.txt             ○ EMOJI — legacy emoji source mapping
├── EquivalentUnifiedIdeograph.txt ○ CJK — radical ↔ unified ideograph
├── HangulSyllableType.txt       ○ HANG  — Hangul: L/V/T/LV/LVT
├── IndicPositionalCategory.txt  ○ INDIC — Indic positional
├── IndicSyllabicCategory.txt    ○ INDIC — Indic syllabic
├── Jamo.txt                     ○ HANG  — Jamo short names
├── LineBreak.txt                ○ BREAK — line break class
├── ScriptExtensions.txt         ○ SCRIPT— multi-script codepoints
├── StandardizedVariants.txt     ○ VAR   — standardized variation sequences
├── VerticalOrientation.txt      ○ CJK   — vertical text orientation
└── ReadMe.txt                   — UCD readme
```

**Ký hiệu:** ★ = HomeOS dùng trực tiếp | ☆ = tham chiếu quan trọng | ○ = phụ trợ / chuyên ngành

---

## Đã lọc bỏ (không cần)

| File | Lý do | Size |
|------|-------|------|
| `BidiTest.txt` | Conformance test data (7.6M) | 7.6 MB |
| `BidiCharacterTest.txt` | Conformance test data (6.6M) | 6.6 MB |
| `NormalizationTest.txt` | Conformance test data (2.8M) | 2.8 MB |
| `auxiliary/*Test*.txt` | Break algorithm test data | ~4.5 MB |
| `auxiliary/*Test*.html` | HTML version test data | ~1 MB |
| `NamesList.html` | HTML duplicate (có .txt) | 36 KB |
| `USourceGlyphs.pdf` | PDF glyph images | 2.1 MB |
| `USourceRSChart.pdf` | PDF radical-stroke chart | 2.0 MB |
| `SealSources.txt` | Seal script sources (historical) | 1.8 MB |
| `TangutSources.txt` | Tangut sources (historical) | 388 KB |
| `JurchenSources.txt` | Jurchen sources (historical) | 80 KB |
| `NushuSources.txt` | Nushu sources (historical) | 24 KB |
| `Unikemet.txt` | Egyptian hieroglyph sources | 1.5 MB |
| `USourceData.txt` | CJK Unified source data | 252 KB |

---

## Phân loại theo chức năng HomeOS

### A. CORE — Nền tảng 5 chiều (build.rs đọc)

| File | Dùng cho | Cú pháp |
|------|---------|---------|
| **UnicodeData.txt** | `encode_codepoint()` — Molecule từ mọi ký tự | `CP;Name;Gc;Ccc;Bc;Decomp;Num6;Num7;Num8;Mirror;Old;Comment;Upper;Lower;Title` (15 field, `;` sep) |
| **Blocks.txt** | Xác định block → nhóm Shape/Math/Emoticon/Musical | `Start..End ; Block_Name` |
| **Scripts.txt** | Script assignment → language detection | `CP_range ; Script_Name # comment` |
| **PropList.txt** | Binary properties (White_Space, Dash, Math...) | `CP_range ; Property # Gc [count] Name` |
| **DerivedCoreProperties.txt** | Derived: Math, Alphabetic, ID_Start, Lowercase... | `CP_range ; Property # Gc [count] Name` |

### B. EMOJI — Chiều Valence+Arousal

| File | Dùng cho | Cú pháp |
|------|---------|---------|
| **emoji/emoji-data.txt** | Emoji property flags | `CP_range ; Property # Version [count] (glyph) Name` |
| **emoji/emoji-variation-sequences.txt** | Text/emoji presentation | `CP FE0E/FE0F ; style ; # Version Name` |
| **EmojiSources.txt** | Legacy mapping (JIS, ARIB) | `CP ; JIS ; ARIB` |

### C. META — Property dictionary

| File | Dùng cho | Cú pháp |
|------|---------|---------|
| **PropertyAliases.txt** | Short ↔ Long name cho properties | `short ; long ; alias...` |
| **PropertyValueAliases.txt** | Property value aliases | `property ; short_value ; long_value ; alias...` |

### D. TÊN & INDEX — Tên node, alias

| File | Dùng cho | Cú pháp |
|------|---------|---------|
| **NameAliases.txt** | Alias chính thức (5 loại) | `CP ; Alias ; Type` (correction/control/alternate/figment/abbreviation) |
| **NamesList.txt** | Human-readable: annotations, cross-ref, subheads | Custom: `@@` block, `@` subhead, `CP TAB Name`, `= alias`, `* note`, `x cross-ref` |
| **Index.txt** | Reverse lookup: tên → codepoint | `NAME TAB CP` (tab-separated) |
| **NamedSequences.txt** | Named character sequences | `Name ; CP CP CP...` (space-separated codepoints) |

### E. SEGMENTATION — Word/Sentence/Grapheme break

| File | Dùng cho | Cú pháp |
|------|---------|---------|
| **auxiliary/GraphemeBreakProperty.txt** | Grapheme cluster boundaries | `CP_range ; GBP_value # Gc Name` |
| **auxiliary/SentenceBreakProperty.txt** | Sentence boundaries | `CP_range ; SBP_value # Gc Name` |
| **auxiliary/WordBreakProperty.txt** | Word boundaries | `CP_range ; WBP_value # Gc Name` |

### F. BIDI — Bidirectional text

| File | Dùng cho | Cú pháp |
|------|---------|---------|
| **ArabicShaping.txt** | Joining type & group | `CP ; Name ; Joining_Type ; Joining_Group` |
| **BidiBrackets.txt** | Paired brackets | `CP ; Paired_CP ; Type` (o=open/c=close) |
| **BidiMirroring.txt** | Mirrored glyphs | `CP ; Mirrored_CP # Name` |

### G. CASE — Case mapping

| File | Dùng cho | Cú pháp |
|------|---------|---------|
| **CaseFolding.txt** | Case fold (lowercase for comparison) | `CP ; Status ; Mapping ; # Name` (C/F/S/T) |
| **SpecialCasing.txt** | Context-dependent casing | `CP ; Lower ; Title ; Upper ; Condition? ; # Name` |

### H. NORMALIZATION

| File | Dùng cho | Cú pháp |
|------|---------|---------|
| **DerivedNormalizationProps.txt** | NFC_QC, NFD_QC, NFKC_QC... | `CP_range ; Property_Value # Name` |
| **CompositionExclusions.txt** | Excluded from canonical composition | `CP # Name` |
| **NormalizationCorrections.txt** | Historical corrections | `CP ; Old ; New ; Version` |

### I. EXTRACTED — Derived từ UnicodeData.txt

| File | Nội dung | Cú pháp |
|------|---------|---------|
| **extracted/DerivedGeneralCategory.txt** | General Category (Lu, Ll, Sm...) | `CP_range ; Gc # count Name` |
| **extracted/DerivedName.txt** | Full character names | `CP_range ; Name` |
| **extracted/DerivedBidiClass.txt** | Bidi class | `CP_range ; Bc` |
| **extracted/DerivedCombiningClass.txt** | Canonical Combining Class | `CP_range ; Ccc` |
| **extracted/DerivedDecompositionType.txt** | Decomposition type | `CP_range ; Dt` |
| **extracted/DerivedEastAsianWidth.txt** | East Asian Width | `CP_range ; Ea` |
| **extracted/DerivedJoiningGroup.txt** | Arabic joining group | `CP_range ; Jg` |
| **extracted/DerivedJoiningType.txt** | Arabic joining type | `CP_range ; Jt` |
| **extracted/DerivedLineBreak.txt** | Line break class | `CP_range ; Lb` |
| **extracted/DerivedNumericType.txt** | Numeric type (Decimal/Digit/Numeric) | `CP_range ; Nt` |
| **extracted/DerivedNumericValues.txt** | Numeric values | `CP_range ; Nv # Nt Name` |

### J. CHUYÊN NGÀNH — Ít dùng cho HomeOS core

| File | Nội dung |
|------|---------|
| **LineBreak.txt** | Line break class per codepoint |
| **EastAsianWidth.txt** | Display width (Narrow/Wide/Fullwidth/Halfwidth) |
| **HangulSyllableType.txt** | Hangul syllable decomposition (L/V/T/LV/LVT) |
| **Jamo.txt** | Jamo short names (28 entries) |
| **CJKRadicals.txt** | CJK radical ↔ unified ideograph |
| **EquivalentUnifiedIdeograph.txt** | Radical → equivalent CJK ideograph |
| **IndicPositionalCategory.txt** | Indic consonant/vowel positional category |
| **IndicSyllabicCategory.txt** | Indic syllabic category |
| **ScriptExtensions.txt** | Multi-script codepoints |
| **StandardizedVariants.txt** | Standardized variation sequences |
| **VerticalOrientation.txt** | CJK vertical text orientation |
| **DoNotEmit.txt** | Sequences không nên sinh ra trong text mới |
| **DerivedAge.txt** | Unicode version khi codepoint được thêm |

---

## Cú pháp chung UCD

### Comment & blank line
```
# Dòng bắt đầu bằng # = comment, bỏ qua
# Dòng trống = bỏ qua
```

### Format phổ biến: Property file
```
# Dạng 1: single codepoint
0041          ; Property_Value  # General_Category  CHARACTER NAME

# Dạng 2: range
0041..005A    ; Property_Value  # General_Category  [count]  FIRST..LAST

# Phần sau # là comment (informational), không parse
```

### Format UnicodeData.txt (15 fields, `;` separated)
```
Field  Ý nghĩa                    Ví dụ
─────  ──────────────────────────  ──────
 0     Codepoint (hex)             0041
 1     Name                        LATIN CAPITAL LETTER A
 2     General_Category             Lu
 3     Canonical_Combining_Class    0
 4     Bidi_Class                   L
 5     Decomposition_Mapping        (blank or <type> CP CP...)
 6     Numeric_Value (decimal)      (blank or 0-9)
 7     Numeric_Value (digit)        (blank or 0-9)
 8     Numeric_Value (numeric)      (blank or fraction)
 9     Bidi_Mirrored                Y/N
10     Unicode_1_Name (obsolete)    (blank)
11     ISO_Comment (obsolete)       (blank)
12     Simple_Uppercase_Mapping     (blank or CP)
13     Simple_Lowercase_Mapping     (blank or CP)
14     Simple_Titlecase_Mapping     (blank or CP)

Đặc biệt: Name = "<control>" hoặc "<CJK Ideograph, First>" → range marker
```

### Format NamesList.txt (custom markup)
```
@@   Block_Start  Block_Name  Block_End     — block header
@@+                                         — block note
@    Subhead_text                            — subhead
CP TAB Name                                  — character entry
TAB = Alias                                  — informative alias
TAB * Note                                   — note
TAB x CP Name                                — cross reference
TAB ~ CP Name                                — compatibility decomposition
TAB # Name                                   — compatibility character
TAB : CP                                     — decomposition
```

### Format Index.txt
```
CHARACTER NAME (reversed)  TAB  Codepoint_Hex
# Tên được sắp xếp theo từ khóa đảo ngược
# VD: "A WITH ACUTE, LATIN CAPITAL LETTER" → 00C1
```

### Format NameAliases.txt
```
CP ; Alias_Name ; Type
# Type: correction | control | alternate | figment | abbreviation
# VD: 0000 ; NULL ; control
```

---

## Mapping HomeOS 5 chiều ← UCD files

```
Chiều HomeOS        File UCD chính                   Cách dùng
──────────────────  ──────────────────────────────    ──────────────────────
Shape (S)           Blocks.txt + UnicodeData.txt      Block → ShapeBase (8 primitives)
                    extracted/DerivedGeneralCategory   Gc phân loại hình dạng ký tự
Relation (R)        UnicodeData.txt field[2] (Gc)     General Category → RelationBase
                    PropList.txt (Math property)       Math symbols → relation operators
Valence (V)         emoji/emoji-data.txt              Emoji → valence mapping
                    UnicodeData.txt (name keywords)   Name chứa HEART/SKULL... → V
Arousal (A)         emoji/emoji-data.txt              Emoji_Presentation → arousal level
                    Blocks.txt (Musical blocks)       Musical symbols → arousal
Time (T)            Blocks.txt (Musical Symbols)      ♩♪♫♬ → time dimension
                    UnicodeData.txt (Gc=So,Sm,Sk)     Symbol type → temporal behavior
```

---

## Quick Reference — Dùng nhanh

```bash
# Tìm codepoint theo tên
grep "FIRE" json/UnicodeData.txt

# Tìm tất cả emoji
grep "Emoji_Presentation" json/emoji/emoji-data.txt

# Xem block nào chứa Musical Symbols
grep -i "musical" json/Blocks.txt

# Xem properties của codepoint
grep "^1F525" json/UnicodeData.txt       # 🔥
grep "1F525" json/emoji/emoji-data.txt   # emoji properties
grep "1F525" json/PropList.txt           # binary properties

# Xem tên alias
grep "^0000" json/NameAliases.txt        # NULL aliases

# Đếm tổng codepoints trong UnicodeData.txt
wc -l json/UnicodeData.txt              # ~35,000 entries
```

---

## Ghi chú cho HomeOS build.rs

`ucd/build.rs` hiện đọc `UnicodeData.txt` để sinh bảng lookup lúc compile.
Các file khác trong `json/` là tài liệu tham chiếu — có thể dùng trong tương lai để:

1. **Mở rộng encode_codepoint()** — dùng `DerivedCoreProperties.txt` (Math) + `emoji-data.txt`
2. **Word/sentence segmentation** — dùng `auxiliary/WordBreakProperty.txt`
3. **Script detection** — dùng `Scripts.txt` + `ScriptExtensions.txt`
4. **Case-insensitive alias** — dùng `CaseFolding.txt`
5. **Block-based grouping** — dùng `Blocks.txt` (5 nhóm Unicode → 5 chiều)
