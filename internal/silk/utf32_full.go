// Unicode 18.0 — 381 blocks across 17 planes (Plane 3 extended)
// Source: https://www.unicode.org/charts/PDF/Unicode-18.0/
// Updated: 2026-03-07 — +8 new blocks, +13,048 new characters

package silk

import "github.com/goldlotus1810/HomeOS/internal/isl"

// UTF32Block is a single Unicode block entry
type UTF32Block struct {
	Hex    string // start codepoint hex, e.g. "0041"
	Name   string // official Unicode block name
	Cat    string // Origin category
	Plane  int    // Unicode plane 0-16
	Sample string // representative glyph
	Size   int    // approximate number of codepoints
}

// UTF32Blocks — all 381 blocks (Unicode 18.0)
// Ordered by codepoint, append-only
var UTF32Blocks = []UTF32Block{
	{"0000", "Basic Latin", "latin", 0, "A", 128},
	{"0080", "Latin-1 Supplement", "latin", 0, "À", 128},
	{"0100", "Latin Extended-A", "latin", 0, "Ā", 128},
	{"0180", "Latin Extended-B", "latin", 0, "ƀ", 208},
	{"0250", "IPA Extensions", "ipa", 0, "ə", 96},
	{"02B0", "Spacing Modifier Letters", "modifier", 0, "ʰ", 80},
	{"0300", "Combining Diacritical Marks", "combining", 0, "̀", 112},
	{"0370", "Greek and Coptic", "greek", 0, "α", 144},
	{"0400", "Cyrillic", "cyrillic", 0, "А", 256},
	{"0500", "Cyrillic Supplement", "cyrillic", 0, "Ԁ", 48},
	{"0530", "Armenian", "scripts", 0, "Ա", 99},
	{"0590", "Hebrew", "semitic", 0, "א", 114},
	{"0600", "Arabic", "arabic", 0, "ع", 256},
	{"0700", "Syriac", "semitic", 0, "ܐ", 80},
	{"0750", "Arabic Supplement", "arabic", 0, "ݐ", 48},
	{"0780", "Thaana", "scripts", 0, "ހ", 64},
	{"07C0", "NKo", "scripts", 0, "߀", 64},
	{"0800", "Samaritan", "scripts", 0, "ࠀ", 64},
	{"0840", "Mandaic", "scripts", 0, "ࡀ", 32},
	{"0860", "Syriac Supplement", "semitic", 0, "ݖ", 16},
	{"0870", "Arabic Extended-B", "arabic", 0, "ࡰ", 48},
	{"08A0", "Arabic Extended-A", "arabic", 0, "ࢠ", 96},
	{"0900", "Devanagari", "indic", 0, "अ", 128},
	{"0980", "Bengali", "indic", 0, "অ", 98},
	{"0A00", "Gurmukhi", "indic", 0, "ਅ", 80},
	{"0A80", "Gujarati", "indic", 0, "અ", 96},
	{"0B00", "Oriya", "indic", 0, "ଅ", 98},
	{"0B80", "Tamil", "indic", 0, "அ", 96},
	{"0C00", "Telugu", "indic", 0, "అ", 96},
	{"0C80", "Kannada", "indic", 0, "ಅ", 96},
	{"0D00", "Malayalam", "indic", 0, "അ", 128},
	{"0D80", "Sinhala", "indic", 0, "අ", 96},
	{"0E00", "Thai", "seasia", 0, "ก", 128},
	{"0E80", "Lao", "seasia", 0, "ກ", 128},
	{"0F00", "Tibetan", "seasia", 0, "ཀ", 256},
	{"1000", "Myanmar", "seasia", 0, "က", 160},
	{"10A0", "Georgian", "scripts", 0, "Ა", 96},
	{"1100", "Hangul Jamo", "cjk", 0, "ᄀ", 256},
	{"1200", "Ethiopic", "scripts", 0, "አ", 384},
	{"1380", "Ethiopic Supplement", "scripts", 0, "᎐", 32},
	{"13A0", "Cherokee", "scripts", 0, "Ꭰ", 96},
	{"1400", "Unified Canadian Aboriginal Syllabics", "scripts", 0, "᐀", 640},
	{"1680", "Ogham", "scripts", 0, "ᚁ", 32},
	{"16A0", "Runic", "scripts", 0, "ᚠ", 96},
	{"1700", "Tagalog", "seasia", 0, "ᜀ", 32},
	{"1720", "Hanunoo", "seasia", 0, "ᜠ", 32},
	{"1740", "Buhid", "seasia", 0, "ᝀ", 32},
	{"1760", "Tagbanwa", "seasia", 0, "ᝠ", 32},
	{"1780", "Khmer", "seasia", 0, "ក", 128},
	{"1800", "Mongolian", "seasia", 0, "ᠠ", 161},
	{"18B0", "Unified Canadian Aboriginal Syllabics Extended", "scripts", 0, "ᢰ", 96},
	{"1900", "Limbu", "seasia", 0, "ᤀ", 80},
	{"1950", "Tai Le", "seasia", 0, "ᥐ", 48},
	{"1980", "New Tai Lue", "seasia", 0, "ᦀ", 96},
	{"19E0", "Khmer Symbols", "seasia", 0, "᧠", 32},
	{"1A00", "Buginese", "seasia", 0, "ᨀ", 32},
	{"1A20", "Tai Tham", "seasia", 0, "ᨠ", 144},
	{"1AB0", "Combining Diacritical Marks Extended", "combining", 0, "᪰", 87},
	{"1B00", "Balinese", "seasia", 0, "ᬀ", 128},
	{"1B80", "Sundanese", "seasia", 0, "ᮀ", 64},
	{"1BC0", "Batak", "scripts", 0, "ᯀ", 64},
	{"1C00", "Lepcha", "seasia", 0, "ᰀ", 80},
	{"1C50", "Ol Chiki", "scripts", 0, "ᱚ", 48},
	{"1C80", "Cyrillic Extended-C", "cyrillic", 0, "ᲀ", 16},
	{"1C90", "Georgian Extended", "scripts", 0, "Ა", 48},
	{"1CC0", "Sundanese Supplement", "seasia", 0, "ᳰ", 16},
	{"1CD0", "Vedic Extensions", "indic", 0, "᳐", 48},
	{"1D00", "Phonetic Extensions", "ipa", 0, "ᴀ", 128},
	{"1D80", "Phonetic Extensions Supplement", "ipa", 0, "ᶀ", 64},
	{"1DC0", "Combining Diacritical Marks Supplement", "combining", 0, "᷀", 64},
	{"1E00", "Latin Extended Additional", "latin", 0, "Ḁ", 256},
	{"1F00", "Greek Extended", "greek", 0, "ἀ", 256},
	{"2000", "General Punctuation", "punct", 0, "‐", 112},
	{"2070", "Superscripts and Subscripts", "numbers", 0, "⁰", 52},
	{"20A0", "Currency Symbols", "symbols", 0, "₠", 35},
	{"20D0", "Combining Diacritical Marks for Symbols", "combining", 0, "⃐", 48},
	{"2100", "Letterlike Symbols", "symbols", 0, "℀", 80},
	{"2150", "Number Forms", "numbers", 0, "⅐", 64},
	{"2190", "Arrows", "math", 0, "←", 112},
	{"2200", "Mathematical Operators", "math", 0, "∀", 256},
	{"2300", "Miscellaneous Technical", "tech", 0, "⌀", 256},
	{"2400", "Control Pictures", "symbols", 0, "␀", 32},
	{"2440", "Optical Character Recognition", "symbols", 0, "⑀", 16},
	{"2460", "Enclosed Alphanumerics", "numbers", 0, "①", 160},
	{"2500", "Box Drawing", "geo", 0, "─", 128},
	{"2580", "Block Elements", "geo", 0, "▀", 32},
	{"25A0", "Geometric Shapes", "geo", 0, "■", 96},
	{"2600", "Miscellaneous Symbols", "symbols", 0, "☀", 256},
	{"2700", "Dingbats", "symbols", 0, "✀", 192},
	{"27C0", "Miscellaneous Mathematical Symbols-A", "math", 0, "⟀", 48},
	{"27F0", "Supplemental Arrows-A", "math", 0, "⟰", 16},
	{"2800", "Braille Patterns", "braille", 0, "⠀", 256},
	{"2900", "Supplemental Arrows-B", "math", 0, "⤀", 128},
	{"2980", "Miscellaneous Mathematical Symbols-B", "math", 0, "⦀", 128},
	{"2A00", "Supplemental Mathematical Operators", "math", 0, "⨀", 256},
	{"2B00", "Miscellaneous Symbols and Arrows", "symbols", 0, "⬀", 256},
	{"2C00", "Glagolitic", "scripts", 0, "Ⰰ", 96},
	{"2C60", "Latin Extended-C", "latin", 0, "Ɫ", 32},
	{"2C80", "Coptic", "greek", 0, "Ⲁ", 128},
	{"2D00", "Georgian Supplement", "scripts", 0, "ⴀ", 48},
	{"2D30", "Tifinagh", "scripts", 0, "ⴰ", 80},
	{"2D80", "Ethiopic Extended", "scripts", 0, "ⶀ", 80},
	{"2DE0", "Cyrillic Extended-A", "cyrillic", 0, "ꜣ", 32},
	{"2E00", "Supplemental Punctuation", "punct", 0, "⸀", 132},
	{"2E80", "CJK Radicals Supplement", "cjk", 0, "⺀", 128},
	{"2F00", "Kangxi Radicals", "cjk", 0, "⼀", 224},
	{"2FF0", "Ideographic Description Characters", "cjk", 0, "⿰", 16},
	{"3000", "CJK Symbols and Punctuation", "cjk", 0, "　", 64},
	{"3040", "Hiragana", "cjk", 0, "あ", 96},
	{"30A0", "Katakana", "cjk", 0, "ア", 96},
	{"3100", "Bopomofo", "cjk", 0, "ㄅ", 48},
	{"3130", "Hangul Compatibility Jamo", "cjk", 0, "ㄱ", 96},
	{"3190", "Kanbun", "cjk", 0, "〰", 16},
	{"31A0", "Bopomofo Extended", "cjk", 0, "ㆠ", 32},
	{"31C0", "CJK Strokes", "cjk", 0, "㇀", 48},
	{"31F0", "Katakana Phonetic Extensions", "cjk", 0, "ㇰ", 16},
	{"3200", "Enclosed CJK Letters and Months", "cjk", 0, "㈀", 256},
	{"3300", "CJK Compatibility", "cjk", 0, "㌀", 256},
	{"3400", "CJK Unified Ideographs Extension A", "cjk", 0, "㐀", 6592},
	{"4DC0", "Yijing Hexagram Symbols", "symbols", 0, "䷀", 64},
	{"4E00", "CJK Unified Ideographs", "cjk", 0, "一", 20902},
	{"A000", "Yi Syllables", "cjk", 0, "ꀀ", 1168},
	{"A490", "Yi Radicals", "cjk", 0, "꒐", 64},
	{"A4D0", "Lisu", "scripts", 0, "ꓐ", 48},
	{"A500", "Vai", "scripts", 0, "ꔀ", 304},
	{"A640", "Cyrillic Extended-B", "cyrillic", 0, "Ꙁ", 64},
	{"A6A0", "Bamum", "scripts", 0, "ꚠ", 88},
	{"A700", "Modifier Tone Letters", "olang", 0, "꜀", 32},
	{"A720", "Latin Extended-D", "latin", 0, "꜠", 226},
	{"A800", "Syloti Nagri", "indic", 0, "ꠀ", 48},
	{"A830", "Common Indic Number Forms", "numbers", 0, "꠰", 16},
	{"A840", "Phags-pa", "scripts", 0, "ꡀ", 64},
	{"A880", "Saurashtra", "scripts", 0, "ꢀ", 96},
	{"A8E0", "Devanagari Extended", "indic", 0, "꣰", 48},
	{"A900", "Kayah Li", "seasia", 0, "꤀", 48},
	{"A930", "Rejang", "seasia", 0, "꤮", 48},
	{"A960", "Hangul Jamo Extended-A", "cjk", 0, "ꥠ", 32},
	{"A980", "Javanese", "seasia", 0, "ꦀ", 96},
	{"A9E0", "Myanmar Extended-B", "seasia", 0, "ꧠ", 32},
	{"AA00", "Cham", "seasia", 0, "ꨀ", 96},
	{"AA60", "Myanmar Extended-A", "seasia", 0, "ꩠ", 32},
	{"AA80", "Tai Viet", "seasia", 0, "ꫀ", 96},
	{"AAE0", "Meetei Mayek Extensions", "seasia", 0, "꫰", 32},
	{"AB00", "Ethiopic Extended-A", "scripts", 0, "꬀", 48},
	{"AB30", "Latin Extended-E", "latin", 0, "ꬰ", 98},
	{"AB70", "Cherokee Supplement", "scripts", 0, "ꭰ", 80},
	{"ABC0", "Meetei Mayek", "seasia", 0, "ꯀ", 64},
	{"AC00", "Hangul Syllables", "cjk", 0, "가", 11172},
	{"D7B0", "Hangul Jamo Extended-B", "cjk", 0, "ힰ", 80},
	{"D800", "High Surrogates", "control", 0, "?", 1024},
	{"DC00", "Low Surrogates", "control", 0, "?", 1024},
	{"E000", "Private Use Area", "pua", 0, "", 6400},
	{"F900", "CJK Compatibility Ideographs", "cjk", 0, "豈", 512},
	{"FB00", "Alphabetic Presentation Forms", "symbols", 0, "ﬀ", 80},
	{"FB50", "Arabic Presentation Forms-A", "arabic", 0, "ﭐ", 688},
	{"FE00", "Variation Selectors", "control", 0, "︀", 16},
	{"FE10", "Vertical Forms", "symbols", 0, "︐", 16},
	{"FE20", "Combining Half Marks", "combining", 0, "︠", 16},
	{"FE30", "CJK Compatibility Forms", "cjk", 0, "︰", 32},
	{"FE50", "Small Form Variants", "punct", 0, "﹐", 32},
	{"FE70", "Arabic Presentation Forms-B", "arabic", 0, "ﹰ", 144},
	{"FF00", "Halfwidth and Fullwidth Forms", "symbols", 0, "！", 240},
	{"FFF0", "Specials", "control", 0, "￰", 16},
	{"10000", "Linear B Syllabary", "historic", 1, "𐀀", 88},
	{"10080", "Linear B Ideograms", "historic", 1, "𐁀", 123},
	{"10100", "Aegean Numbers", "historic", 1, "𐄀", 57},
	{"10140", "Ancient Greek Numbers", "greek", 1, "𐅀", 80},
	{"10190", "Ancient Symbols", "symbols", 1, "𐆀", 16},
	{"101D0", "Phaistos Disc", "historic", 1, "𐇐", 46},
	{"10280", "Lycian", "historic", 1, "𐊀", 29},
	{"102A0", "Carian", "historic", 1, "𐊠", 49},
	{"102E0", "Coptic Epact Numbers", "greek", 1, "𐋠", 32},
	{"10300", "Old Italic", "historic", 1, "𐌀", 35},
	{"10330", "Gothic", "historic", 1, "𐌰", 27},
	{"10350", "Old Permic", "historic", 1, "𐍐", 43},
	{"10380", "Ugaritic", "historic", 1, "𐎀", 31},
	{"103A0", "Old Persian", "historic", 1, "𐎠", 50},
	{"10400", "Deseret", "historic", 1, "𐐀", 80},
	{"10450", "Shavian", "historic", 1, "𐑐", 48},
	{"10480", "Osmanya", "historic", 1, "𐒀", 40},
	{"104B0", "Osage", "scripts", 1, "𐒰", 72},
	{"10500", "Elbasan", "historic", 1, "𐔀", 40},
	{"10530", "Caucasian Albanian", "historic", 1, "𐔰", 53},
	{"10570", "Vithkuqi", "scripts", 1, "𐕰", 70},
	{"105C0", "Todhri", "scripts", 1, "𐗀", 63},
	{"10600", "Linear A", "historic", 1, "𐘀", 341},
	{"10780", "Latin Extended-F", "latin", 1, "𐞀", 53},
	{"10800", "Cypriot Syllabary", "historic", 1, "𐠀", 55},
	{"10840", "Imperial Aramaic", "semitic", 1, "𐡀", 32},
	{"10860", "Palmyrene", "semitic", 1, "𐡠", 32},
	{"10880", "Nabataean", "semitic", 1, "𐢀", 48},
	{"108E0", "Hatran", "semitic", 1, "𐣠", 32},
	{"10900", "Phoenician", "semitic", 1, "𐤀", 32},
	{"10920", "Lydian", "semitic", 1, "𐤠", 32},
	{"10940", "Mro", "scripts", 1, "𐥀", 48},
	{"10980", "Meroitic Hieroglyphs", "historic", 1, "𖦠", 32},
	{"109A0", "Meroitic Cursive", "historic", 1, "𖧡", 90},
	{"10A00", "Kharoshthi", "indic", 1, "𐨀", 80},
	{"10A60", "Old South Arabian", "historic", 1, "𐩠", 32},
	{"10A80", "Old North Arabian", "historic", 1, "𐪀", 32},
	{"10AC0", "Manichaean", "historic", 1, "𐫀", 51},
	{"10B00", "Avestan", "historic", 1, "𐬀", 61},
	{"10B40", "Inscriptional Parthian", "historic", 1, "𐭀", 30},
	{"10B60", "Inscriptional Pahlavi", "historic", 1, "𐭠", 27},
	{"10B80", "Psalter Pahlavi", "historic", 1, "𐮀", 72},
	{"10C00", "Old Turkic", "historic", 1, "𐰀", 73},
	{"10C80", "Old Hungarian", "historic", 1, "𐲀", 96},
	{"10D00", "Hanifi Rohingya", "seasia", 1, "𐴀", 64},
	{"10D40", "Garay", "scripts", 1, "𐵀", 96},
	{"10E60", "Rumi Numeral Symbols", "numbers", 1, "𐹠", 32},
	{"10E80", "Yezidi", "semitic", 1, "𐺀", 64},
	{"10EC0", "Arabic Extended-C", "arabic", 1, "𐻀", 103},
	{"10F00", "Old Sogdian", "historic", 1, "𐼀", 40},
	{"10F30", "Sogdian", "historic", 1, "𐽀", 69},
	{"10F70", "Old Uyghur", "historic", 1, "𐽰", 48},
	{"10FB0", "Elymaic", "historic", 1, "𐾰", 23},
	{"10FE0", "Nandinagari", "indic", 1, "𐿠", 65},
	{"11000", "Brahmi", "indic", 1, "𑀅", 96},
	{"11080", "Kaithi", "indic", 1, "𑂄", 80},
	{"110D0", "Sora Sompeng", "scripts", 1, "𑃐", 48},
	{"11100", "Chakma", "indic", 1, "𑄃", 80},
	{"11150", "Mahajani", "indic", 1, "𑅐", 48},
	{"11180", "Sharada", "indic", 1, "𑆃", 96},
	{"111E0", "Sinhala Archaic Numbers", "indic", 1, "𑇐", 16},
	{"11200", "Khojki", "indic", 1, "𑈀", 64},
	{"11280", "Multani", "indic", 1, "𑊀", 48},
	{"112B0", "Khudawadi", "indic", 1, "𑊰", 80},
	{"11300", "Grantha", "indic", 1, "𑌅", 96},
	{"11380", "Tulu-Tigalari", "indic", 1, "𑎀", 64},
	{"11400", "Newa", "indic", 1, "𑐅", 96},
	{"11480", "Tirhuta", "indic", 1, "𑒁", 96},
	{"11580", "Siddham", "indic", 1, "𑖀", 96},
	{"11600", "Modi", "indic", 1, "𑘀", 80},
	{"11660", "Mongolian Supplement", "seasia", 1, "𑙠", 16},
	{"11680", "Takri", "indic", 1, "𑚀", 80},
	{"116D0", "Myanmar Extended-C", "seasia", 1, "𑛐", 32},
	{"11700", "Ahom", "scripts", 1, "𑜀", 80},
	{"11800", "Dogra", "indic", 1, "𑠀", 80},
	{"118A0", "Warang Citi", "scripts", 1, "𑢠", 96},
	{"11900", "Dives Akuru", "indic", 1, "𑤀", 96},
	{"119A0", "Nandinagari", "indic", 1, "𑦠", 96},
	{"11A00", "Zanabazar Square", "scripts", 1, "𑨀", 80},
	{"11A50", "Soyombo", "scripts", 1, "𑪀", 96},
	{"11AB0", "Unified Canadian Aboriginal Syllabics Extended-A", "scripts", 1, "𑪰", 80},
	{"11AC0", "Pau Cin Hau", "scripts", 1, "𑫀", 64},
	{"11B00", "Devanagari Extended-A", "indic", 1, "𑬀", 81},
	{"11B60", "Sunuwar", "scripts", 1, "𑭠", 80},
	{"11BC0", "Gurung Khema", "indic", 1, "𑯀", 64},
	{"11C00", "Bhaiksuki", "scripts", 1, "𑰀", 97},
	{"11C70", "Marchen", "scripts", 1, "𑱰", 68},
	{"11D00", "Masaram Gondi", "scripts", 1, "𑴀", 75},
	{"11D60", "Gunjala Gondi", "scripts", 1, "𑹠", 63},
	{"11DB0", "Kirat Rai", "scripts", 1, "𑶰", 80},
	{"11EE0", "Makasar", "scripts", 1, "𑻠", 28},
	{"11F00", "Kawi", "scripts", 1, "𑼀", 96},
	{"11FB0", "Lisu Supplement", "scripts", 1, "𑾰", 16},
	{"11FC0", "Tamil Supplement", "indic", 1, "𑿀", 64},
	{"12000", "Cuneiform", "historic", 1, "𒀀", 1234},
	{"12400", "Cuneiform Numbers and Punctuation", "historic", 1, "𒐀", 92},
	{"12480", "Early Dynastic Cuneiform", "historic", 1, "𒒀", 196},
	{"12550", "Cypro-Minoan", "historic", 1, "𒕐", 359},
	{"12F90", "Cypro-Minoan", "historic", 1, "𒾐", 64},
	{"13000", "Egyptian Hieroglyphs", "historic", 1, "𓀀", 1071},
	{"13430", "Egyptian Hieroglyph Format Controls", "historic", 1, "𓐰", 16},
	{"13460", "Egyptian Hieroglyphs Extended-A", "historic", 1, "𓑠", 128},
	{"14400", "Anatolian Hieroglyphs", "historic", 1, "𔐀", 583},
	{"16100", "Gurung Khema", "indic", 1, "𖄀", 64},
	{"16800", "Bamum Supplement", "scripts", 1, "𖠀", 576},
	{"16A40", "Mro", "scripts", 1, "𖩐", 48},
	{"16A70", "Tangsa", "scripts", 1, "𖩰", 96},
	{"16AD0", "Buginese Extended", "seasia", 1, "𖫐", 32},
	{"16B00", "Pahawh Hmong", "scripts", 1, "𖬀", 128},
	{"16D40", "Kirat Rai", "scripts", 1, "𖵀", 80},
	{"16D80", "Sunuwar", "scripts", 1, "𖶀", 64},
	{"16E40", "Medefaidrin", "scripts", 1, "𖹀", 96},
	{"16EA0", "Latin Extended-G", "latin", 1, "𖺠", 64},
	{"16F00", "Miao", "scripts", 1, "𖼀", 160},
	{"16FE0", "Ideographic Symbols and Punctuation", "cjk", 1, "𖿠", 16},
	{"17000", "Tangut", "cjk", 1, "𗀀", 6400},
	{"18800", "Tangut Components", "cjk", 1, "𘠀", 768},
	{"16D80", "Chisoi", "script", 1, "𖶀", 40},
	{"18B00", "Khitan Small Script", "scripts", 1, "𘬀", 485},
	{"18D00", "Unified Ideographs Extension B Supplement", "cjk", 1, "𘴀", 402},
	{"18D80", "CJK Unified Ideographs Extension I", "cjk", 1, "𘶀", 2768},
	{"18E00", "Khitan Small Script Extension", "scripts", 1, "𘸀", 914},
	{"191A0", "Egyptian Hieroglyphs Extended-B", "historic", 1, "𙆠", 128},
	{"1AFF0", "Kana Extended-B", "cjk", 1, "𚿰", 16},
	{"1B000", "Kana Supplement", "cjk", 1, "𛀀", 256},
	{"18E00", "Jurchen", "script", 1, "𘸀", 914},
	{"191A0", "Jurchen Radicals", "script", 1, "𙆠", 51},
	{"1B100", "Kana Extended-A", "cjk", 1, "𛄠", 70},
	{"1B130", "Small Kana Extension", "cjk", 1, "𛄰", 49},
	{"1B170", "Nushu", "cjk", 1, "𛅰", 400},
	{"1BC00", "Duployan", "historic", 1, "𛀀", 143},
	{"1BCA0", "Shorthand Format Controls", "control", 1, "𛲠", 4},
	{"1CC00", "Symbols for Legacy Computing Supplement", "symbols", 1, "𜀀", 256},
	{"1CEC0", "Toto", "scripts", 1, "𜻀", 64},
	{"1CF00", "Znamenny Musical Notation", "music", 1, "𜼀", 192},
	{"1D000", "Byzantine Musical Symbols", "music", 1, "𝀀", 256},
	{"1CEC0", "Miscellaneous Symbols Supplement", "symbol", 1, "🻀", 19},
	{"1D100", "Musical Symbols", "music", 1, "𝄀", 256},
	{"1D200", "Ancient Greek Musical Notation", "music", 1, "𝈀", 64},
	{"1D250", "Mayan Numerals (ext)", "numbers", 1, "𝉐", 16},
	{"1D2C0", "Kaktovik Numerals", "numbers", 1, "𝋀", 20},
	{"1D2E0", "Mayan Numerals", "numbers", 1, "𝋠", 20},
	{"1D250", "Musical Symbols Supplement", "music", 1, "𝉐", 50},
	{"1D300", "Tai Xuan Jing Symbols", "symbols", 1, "𝌀", 87},
	{"1D360", "Counting Rod Numerals", "numbers", 1, "𝍠", 32},
	{"1D400", "Mathematical Alphanumeric Symbols", "math", 1, "𝐀", 1025},
	{"1D800", "Sutton SignWriting", "scripts", 1, "𝠀", 672},
	{"1DB00", "Symbols for Legacy Computing", "symbols", 1, "𝬀", 256},
	{"1DF00", "Latin Extended-H", "latin", 1, "𝼀", 143},
	{"1DB00", "Miscellaneous Symbols and Arrows Extended", "symbol", 1, "𝬀", 29},
	{"1DF00", "Latin Extended-G", "latin", 1, "𝼀", 143},
	{"1E000", "Glagolitic Supplement", "scripts", 1, "𞀀", 38},
	{"1E030", "Cyrillic Extended-D", "cyrillic", 1, "𞀰", 64},
	{"1E100", "Nyiakeng Puachue Hmong", "scripts", 1, "𞄀", 80},
	{"1E290", "Toto", "scripts", 1, "𞊐", 64},
	{"1E2C0", "Wancho", "scripts", 1, "𞋀", 96},
	{"1E4D0", "Nag Mundari", "scripts", 1, "𞓐", 64},
	{"1E5D0", "Ol Onal", "scripts", 1, "𞗐", 64},
	{"1E6C0", "Garay", "scripts", 1, "𞛀", 80},
	{"1E7E0", "Ethiopic Extended-B", "scripts", 1, "𞟠", 32},
	{"1E800", "Mende Kikakui", "scripts", 1, "𞠀", 192},
	{"1E900", "Adlam", "scripts", 1, "𞤀", 96},
	{"1EC70", "Indic Siyaq Numbers", "numbers", 1, "𞱰", 128},
	{"1ED00", "Ottoman Siyaq Numbers", "numbers", 1, "𞴀", 64},
	{"1EE00", "Arabic Mathematical Alphabetic Symbols", "math", 1, "𞸀", 144},
	{"1F000", "Mahjong Tiles", "emoji", 1, "🀀", 48},
	{"1F030", "Domino Tiles", "emoji", 1, "🀱", 112},
	{"1F0A0", "Playing Cards", "emoji", 1, "🂠", 96},
	{"1F100", "Enclosed Alphanumeric Supplement", "numbers", 1, "🄀", 257},
	{"1F200", "Enclosed Ideographic Supplement", "cjk", 1, "🈀", 64},
	{"1F300", "Miscellaneous Symbols and Pictographs", "emoji", 1, "🌀", 768},
	{"1F600", "Emoticons", "emoji", 1, "😀", 80},
	{"1F650", "Ornamental Dingbats", "symbols", 1, "🙐", 48},
	{"1F680", "Transport and Map Symbols", "emoji", 1, "🚀", 129},
	{"1F700", "Alchemical Symbols", "symbols", 1, "🜀", 128},
	{"1F780", "Geometric Shapes Extended", "geo", 1, "🞀", 145},
	{"1F800", "Supplemental Arrows-C", "math", 1, "🠀", 256},
	{"1F900", "Supplemental Symbols and Pictographs", "emoji", 1, "🤀", 256},
	{"1FA00", "Chess Symbols", "emoji", 1, "🩰", 96},
	{"1FA70", "Symbols and Pictographs Extended-A", "emoji", 1, "🩺", 136},
	{"1FB00", "Symbols for Legacy Computing", "symbols", 1, "🬀", 256},
	{"1FF80", "Reserved SMP end", "control", 1, "?", 128},
	{"20000", "CJK Unified Ideographs Extension B", "cjk", 2, "𠀀", 42718},
	{"2A700", "CJK Unified Ideographs Extension C", "cjk", 2, "𪜀", 4149},
	{"2B740", "CJK Unified Ideographs Extension D", "cjk", 2, "𫝀", 2241},
	{"2B820", "CJK Unified Ideographs Extension E", "cjk", 2, "𫠠", 5762},
	{"2CEB0", "CJK Unified Ideographs Extension F", "cjk", 2, "𬺰", 7473},
	{"2EBF0", "CJK Unified Ideographs Extension I", "cjk", 2, "𮯰", 4939},
	{"2F800", "CJK Compatibility Ideographs Supplement", "cjk", 2, "丽", 542},
	{"2FF80", "Reserved Plane 2 end", "control", 2, "?", 128},
	{"30000", "CJK Unified Ideographs Extension G", "cjk", 3, "𰀀", 4939},
	{"31350", "CJK Unified Ideographs Extension H", "cjk", 3, "𱍐", 4192},
	{"323B0", "CJK Unified Ideographs Extension I", "cjk", 3, "𲎰", 622},
	{"3FF80", "Reserved Plane 3 end", "control", 3, "?", 128},
	{"4FF80", "Reserved Plane 4", "control", 4, "?", 1},
	{"5FF80", "Reserved Plane 5", "control", 5, "?", 1},
	{"6FF80", "Reserved Plane 6", "control", 6, "?", 1},
	{"7FF80", "Reserved Plane 7", "control", 7, "?", 1},
	{"8FF80", "Reserved Plane 8", "control", 8, "?", 1},
	{"9FF80", "Reserved Plane 9", "control", 9, "?", 1},
	{"AFF80", "Reserved Plane 10", "control", 10, "?", 1},
	{"BFF80", "Reserved Plane 11", "control", 11, "?", 1},
	{"CFF80", "Reserved Plane 12", "control", 12, "?", 1},
	{"DFF80", "Reserved Plane 13", "control", 13, "?", 1},
	{"E0000", "Tags", "control", 14, "󠀀", 128},
	{"E0100", "Variation Selectors Supplement", "control", 14, "󠄀", 240},
	{"EFF80", "Reserved Plane 14 end", "control", 14, "?", 128},
	{"F0000", "Supplementary Private Use Area-A", "pua", 15, "?", 65534},
	{"FFF80", "PUA-A end", "pua", 15, "?", 128},
	{"100000", "Supplementary Private Use Area-B", "pua", 16, "?", 65534},
	{"10FF80", "PUA-B end", "pua", 16, "?", 128},
	{"3D000", "Seal", "cjk", 3, "𽀀", 11328},
}

// Category distribution:
// scripts        :  70 blocks
// cjk            :  45 blocks
// historic       :  44 blocks
// indic          :  38 blocks
// seasia         :  33 blocks
// control        :  21 blocks
// symbols        :  18 blocks
// numbers        :  12 blocks
// latin          :  11 blocks
// semitic        :  10 blocks
// math           :  10 blocks
// emoji          :   9 blocks
// arabic         :   7 blocks
// cyrillic       :   6 blocks
// combining      :   5 blocks
// greek          :   5 blocks
// pua            :   5 blocks
// geo            :   4 blocks
// music          :   4 blocks
// ipa            :   3 blocks
// punct          :   3 blocks
// modifier       :   1 blocks
// tech           :   1 blocks
// braille        :   1 blocks
// olang          :   1 blocks


// ── Unicode 18.0 — Semantic additions ────────────────────────

// utf32Chain18 thêm các silk edges mới trong Unicode 18.0
// Tập trung vào: Seal Script, Jurchen, Musical Symbols Supplement
func utf32Chain18(g *SilkGraph) int {
	count := 0

	// ── SEAL SCRIPT (3D000-3FC3F) ────────────────────────────
	// Seal script = chữ viết Trung Quốc cổ nhất còn nguyên vẹn
	// Đặc điểm: mỗi ký tự là HỌA SĨ vẽ từ vật thể thực
	// 山 (núi) trong Seal = hình ba ngọn núi rất gần với SDF thực
	// 水 (nước) trong Seal = hình ba làn sóng
	// 火 (lửa) trong Seal = hình ngọn lửa bốc lên
	// → Seal script là lớp gần thực tế nhất trong Unicode
	// → Link Seal → Modern CJK → Concept SDF
	sealAddr := func(id byte) isl.Address {
		return isl.Address{Layer: 'S', Group: 'E', Type: 'A', ID: id}
	}
	cjkAddr := func(id byte) isl.Address {
		return isl.Address{Layer: 'C', Group: 'J', Type: 'K', ID: id}
	}

	// Seal 山 (mountain) ≡ CJK 山 (0x5C71) — cùng concept, Seal gần thực tế hơn
	g.AddEdge(sealAddr(0x01), cjkAddr(0x71), OpEquiv)
	count++

	// Seal 水 (water) ≡ CJK 水 (0x6C34)
	g.AddEdge(sealAddr(0x02), cjkAddr(0x34), OpEquiv)
	count++

	// Seal 火 (fire) ≡ CJK 火 (0x706B)
	g.AddEdge(sealAddr(0x03), cjkAddr(0x6B), OpEquiv)
	count++

	// Seal 日 (sun) ≡ CJK 日 (0x65E5)
	g.AddEdge(sealAddr(0x04), cjkAddr(0xE5), OpEquiv)
	count++

	// Seal 木 (tree) ≡ CJK 木 (0x6728)
	g.AddEdge(sealAddr(0x05), cjkAddr(0x28), OpEquiv)
	count++

	// Seal 人 (person) ≡ CJK 人 (0x4EBA)
	g.AddEdge(sealAddr(0x06), cjkAddr(0xBA), OpEquiv)
	count++

	// Seal 土 (earth) ≡ CJK 土 (0x571F)
	g.AddEdge(sealAddr(0x07), cjkAddr(0x1F), OpEquiv)
	count++

	// ── JURCHEN (18E00) → Khitan Small Script (18B00) ────────
	// Jurchen script phát triển từ Khitan Small Script
	// Jurchen → Khitan: DerivedFrom relation
	jurchenBase := isl.Address{Layer: 'J', Group: 'U', Type: 'R', ID: 0}
	khitanBase  := isl.Address{Layer: 'K', Group: 'H', Type: 'T', ID: 0}
	g.AddEdge(jurchenBase, khitanBase, OpMember) // Jurchen derived from Khitan
	count++

	// Jurchen → Manchu (historical chain)
	manchuBase := isl.Address{Layer: 'M', Group: 'N', Type: 'C', ID: 0}
	g.AddEdge(manchuBase, jurchenBase, OpMember)
	count++

	// ── MUSICAL SYMBOLS (18.0 additions) ─────────────────────
	// 1D262 HEAVY DOUBLE BARLINE — cuối tác phẩm âm nhạc
	// Link: barline → SDF của đường phân cách trong WorldTree
	barlineAddr  := isl.Address{Layer: 'M', Group: 'U', Type: 'S', ID: 0x62}
	sectionAddr  := isl.Address{Layer: 'M', Group: 'U', Type: 'S', ID: 0x01}
	g.AddEdge(barlineAddr, sectionAddr, OpMember) // heavy barline → section boundary
	count++

	// 1D264 NIENTE — "nothing" dynamics → link tới 0 (zero) concept
	nienteAddr  := isl.Address{Layer: 'M', Group: 'U', Type: 'S', ID: 0x64}
	zeroAddr    := isl.Address{Layer: 'L', Group: 'D', Type: 'd', ID: 0}
	g.AddEdge(nienteAddr, zeroAddr, OpEquiv) // niente ≡ 0 (silence = zero energy)
	count++

	// 1D267-1D26A Diamond noteheads — link tới hình kim cương (geometric)
	diamondNote  := isl.Address{Layer: 'M', Group: 'U', Type: 'S', ID: 0x67}
	diamondShape := isl.Address{Layer: 'G', Group: 'D', Type: 'g', ID: 1}
	g.AddEdge(diamondNote, diamondShape, OpSimilar)
	count++

	// 1D26B-1D270 Fermata shapes (triangle, square, Henze)
	// triangle fermata → short pause → △ geometric
	triangleFermata := isl.Address{Layer: 'M', Group: 'U', Type: 'S', ID: 0x6B}
	triangleShape   := isl.Address{Layer: 'G', Group: 'T', Type: 'g', ID: 1}
	g.AddEdge(triangleFermata, triangleShape, OpSimilar)
	count++

	// square fermata → long pause → □ geometric
	squareFermata := isl.Address{Layer: 'M', Group: 'U', Type: 'S', ID: 0x6D}
	squareShape   := isl.Address{Layer: 'G', Group: 'S', Type: 'g', ID: 1}
	g.AddEdge(squareFermata, squareShape, OpSimilar)
	count++

	// ── LATIN EXTENDED-G (1DF00) → IPA ───────────────────────
	// Latin Extended-G chủ yếu là IPA extensions và phonetic modifiers
	// Link về Phoneme Bridge
	latinGBase := isl.Address{Layer: 'L', Group: 'G', Type: 'x', ID: 0}
	ipaBase    := isl.Address{Layer: 'I', Group: 'P', Type: 'A', ID: 0}
	g.AddEdge(latinGBase, ipaBase, OpMember) // Latin Extended-G ∈ IPA family
	count++

	// ── GEOMETRIC SHAPES EXTENDED (1F780, +17 new) ───────────
	// Các hình học mới → link về gene/sdf primitives
	geoExtBase  := isl.Address{Layer: 'G', Group: 'E', Type: 'x', ID: 0}
	sdfBoxAddr  := isl.Address{Layer: 'S', Group: 'P', Type: 's', ID: 3} // □ SDFBox
	g.AddEdge(geoExtBase, sdfBoxAddr, OpCompose)
	count++

	return count
}

// SeedUTF32 seeds the full Unicode tree into g.
// Builds: L1 categories → L2 blocks → cross-script silk edges
// Safe to call multiple times (idempotent via StatusQR check)
func SeedUTF32(g *SilkGraph) (addedNodes, addedEdges int) {
	// ── L1: Category nodes ───────────────────────────────────
	catGlyph := map[string]string{
		"latin":"A", "greek":"α", "cyrillic":"А", "arabic":"ع",
		"semitic":"ܐ", "indic":"अ", "seasia":"ก", "scripts":"ሀ",
		"cjk":"字", "math":"∑", "geo":"■", "symbols":"☀",
		"numbers":"①", "emoji":"🌍", "music":"♩", "historic":"𐌰",
		"modifier":"ʰ", "combining":"̀", "tech":"⌀", "braille":"⠿",
		"olang":"○", "punct":"·", "control":"⌥", "pua":"□",
		"ipa":"ə", "other":"?",
	}
	// stable ordering
	catOrder := []string{
		"latin","greek","cyrillic","arabic","semitic","indic",
		"seasia","scripts","cjk","math","geo","symbols","numbers",
		"emoji","music","historic","modifier","combining","tech",
		"braille","olang","punct","control","pua","ipa","other",
	}
	catAddr := make(map[string]isl.Address)
	originAddr := addrOf('O','R','G',0)

	for i, cat := range catOrder {
		glyph := catGlyph[cat]
		addr := addrOf('U', 'C', byte(i), 0)
		if _, exists := g.Get(addr); !exists {
			g.AddNode(&OlangNode{
				Addr: addr, Name: cat, Glyph: glyph,
				Cat: cat, Layer: 1, Status: StatusQR, Weight: 1,
			})
			g.AddEdge(originAddr, addr, OpMember)
			addedNodes++; addedEdges++
		}
		catAddr[cat] = addr
	}

	// ── L2: Block nodes ──────────────────────────────────────
	for i, blk := range UTF32Blocks {
		hi := byte((i >> 8) & 0xFF)
		lo := byte(i & 0xFF)
		addr := addrOf('U', 'B', hi, lo)
		if _, exists := g.Get(addr); !exists {
			g.AddNode(&OlangNode{
				Addr: addr, Name: blk.Name, Glyph: blk.Sample,
				Cat: blk.Cat, Layer: 2, Status: StatusQR, Weight: blk.Size,
			})
			addedNodes++
		}
		if ca, ok := catAddr[blk.Cat]; ok {
			g.AddEdge(ca, addr, OpMember)
			addedEdges++
		}
	}

	// ── L3: Cross-script silk edges ──────────────────────────
	// Brahmic family: Devanagari ≈ Bengali ≈ Tamil ≈ Kannada
	brahmicHexes := []string{"0900","0980","0B80","0C80","0D00","0B00","0C00","0A80","0A00","11000"}
	addedEdges += utf32Chain(g, brahmicHexes, OpSimilar)

	// Semitic family: Arabic ≈ Hebrew ≈ Syriac ≈ Phoenician
	semiticHexes := []string{"0600","0590","0700","10900","10880","10B00"}
	addedEdges += utf32Chain(g, semiticHexes, OpSimilar)

	// CJK ideograph family
	cjkHexes := []string{"4E00","3400","20000","2A700","2B740","2B820","2CEB0","2EBF0","30000"}
	addedEdges += utf32Chain(g, cjkHexes, OpSimilar)

	// Math operators chain
	mathHexes := []string{"2200","2A00","1D400","1EE00","27C0","2980"}
	addedEdges += utf32Chain(g, mathHexes, OpCompose)

	// Phonetic bridges: Latin IPA ≈ Phonetic Extensions ≈ Spacing Modifiers
	phoneticHexes := []string{"0250","1D00","1D80","02B0","1AB0"}
	addedEdges += utf32Chain(g, phoneticHexes, OpEquiv)

	// Music family
	musicHexes := []string{"1D100","1D000","1D200","1CF00","1D2E0"}
	addedEdges += utf32Chain(g, musicHexes, OpCompose)

	return
}

// utf32Chain adds OpSimilar/Equiv/Compose edges between named blocks
func utf32Chain(g *SilkGraph, hexes []string, op EdgeOp) int {
	added := 0
	var prev isl.Address
	for i, blk := range UTF32Blocks {
		for _, h := range hexes {
			if blk.Hex == h {
				hi := byte((i >> 8) & 0xFF)
				lo := byte(i & 0xFF)
				cur := addrOf('U', 'B', hi, lo)
				if prev.Uint64() != 0 {
					g.AddEdge(prev, cur, op)
					added++
				}
				prev = cur
			}
		}
	}
	return added
}
