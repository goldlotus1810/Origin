#!/usr/bin/env python3
"""
Build full UDC P_weight table for ALL Unicode codepoints.
Maps every assigned Unicode character to a 5D molecule [S, R, V, A, T].

Source priority:
1. udc.json hand-curated entries (8,284 chars, highest quality)
2. Block-level defaults from encoder.ol (59 blocks)
3. Unicode category-based defaults (everything else)

Output: json/udc_p_table_full.bin (u16 per codepoint, up to 0x26000)
"""

import json
import unicodedata
import struct
import os

ORIGIN = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def mol_pack(s, r, v, a, t):
    """Pack SRVAT into u16: [S:4][R:4][V:3][A:3][T:2]"""
    return (s & 0xF) << 12 | (r & 0xF) << 8 | (v & 0x7) << 5 | (a & 0x7) << 2 | (t & 0x3)

def load_existing_udc():
    """Load hand-curated entries from udc.json"""
    path = os.path.join(ORIGIN, 'json', 'udc.json')
    with open(path) as f:
        udc = json.load(f)

    entries = {}
    for char in udc.get('characters', []):
        cp = char['codepoint']
        pw = char['physics_logic']['P_weight']
        # P_weight in json is [S_raw, R_raw, V_raw, A_raw, T_raw] with different scaling
        # Need to convert to u16 packed format
        # In the json: S=0..255, R=0..255, V=0..255, A=0..255, T=0..255
        # In u16: S=0..15, R=0..15, V=0..7, A=0..7, T=0..3
        s = min(15, pw[0] >> 4) if len(pw) > 0 else 0
        r = min(15, pw[1] >> 4) if len(pw) > 1 else 0
        v = min(7, pw[2] >> 5) if len(pw) > 2 else 4
        a = min(7, pw[3] >> 5) if len(pw) > 3 else 4
        t = min(3, pw[4] >> 6) if len(pw) > 4 else 0
        entries[cp] = mol_pack(s, r, v, a, t)

    return entries

def load_existing_bin():
    """Load existing P_weight binary table"""
    path = os.path.join(ORIGIN, 'json', 'udc_p_table.bin')
    data = open(path, 'rb').read()
    entries = {}
    for cp in range(len(data) // 2):
        pw = data[cp*2] | (data[cp*2+1] << 8)
        if pw > 0:
            entries[cp] = pw
    return entries

# Unicode category → SRVAT defaults
# Based on the linguistic/semantic nature of each category
CATEGORY_DEFAULTS = {
    # Letters
    'Lu': mol_pack(0, 0, 4, 5, 2),   # Uppercase letter: neutral V, slightly high A
    'Ll': mol_pack(0, 0, 4, 4, 2),   # Lowercase letter: neutral
    'Lt': mol_pack(0, 0, 4, 5, 2),   # Titlecase
    'Lm': mol_pack(0, 0, 4, 3, 2),   # Modifier letter
    'Lo': mol_pack(0, 0, 4, 4, 2),   # Other letter (CJK, etc)

    # Marks
    'Mn': mol_pack(0, 0, 4, 3, 1),   # Non-spacing mark (diacritics)
    'Mc': mol_pack(0, 0, 4, 3, 1),   # Spacing mark
    'Me': mol_pack(0, 0, 4, 3, 1),   # Enclosing mark

    # Numbers
    'Nd': mol_pack(1, 0, 4, 4, 0),   # Decimal digit: S=1 (shape)
    'Nl': mol_pack(1, 0, 4, 3, 0),   # Letter number (Roman numerals)
    'No': mol_pack(1, 0, 4, 3, 0),   # Other number (fractions, etc)

    # Punctuation
    'Pc': mol_pack(3, 4, 4, 2, 1),   # Connector (underscore)
    'Pd': mol_pack(3, 3, 4, 2, 0),   # Dash
    'Ps': mol_pack(3, 4, 4, 3, 1),   # Open punctuation (brackets)
    'Pe': mol_pack(3, 4, 4, 3, 1),   # Close punctuation
    'Pi': mol_pack(3, 4, 4, 3, 1),   # Initial quote
    'Pf': mol_pack(3, 4, 4, 3, 1),   # Final quote
    'Po': mol_pack(3, 3, 4, 2, 0),   # Other punctuation

    # Symbols
    'Sm': mol_pack(0, 4, 4, 4, 1),   # Math symbol: R=4 (relation)
    'Sc': mol_pack(0, 5, 4, 4, 0),   # Currency: R=5 (currency)
    'Sk': mol_pack(0, 0, 4, 3, 0),   # Modifier symbol
    'So': mol_pack(0, 0, 5, 5, 1),   # Other symbol

    # Separators
    'Zs': mol_pack(3, 3, 4, 0, 0),   # Space
    'Zl': mol_pack(3, 3, 4, 0, 0),   # Line separator
    'Zp': mol_pack(3, 3, 4, 0, 0),   # Paragraph separator

    # Other
    'Cc': mol_pack(0, 0, 4, 0, 0),   # Control
    'Cf': mol_pack(0, 0, 4, 0, 0),   # Format
    'Co': mol_pack(0, 0, 4, 4, 2),   # Private use
    'Cs': mol_pack(0, 0, 4, 4, 2),   # Surrogate
    'Cn': mol_pack(0, 0, 4, 4, 0),   # Unassigned
}

# Special block overrides (higher quality than pure category)
BLOCK_OVERRIDES = {
    # Math operators — high R (relation)
    (0x2200, 0x22FF): mol_pack(0, 4, 4, 4, 1),    # Mathematical Operators
    (0x2A00, 0x2AFF): mol_pack(0, 4, 4, 4, 1),    # Supplemental Math
    (0x27C0, 0x27EF): mol_pack(0, 4, 4, 4, 1),    # Misc Math Symbols-A
    (0x2980, 0x29FF): mol_pack(0, 4, 4, 4, 1),    # Misc Math Symbols-B

    # Arrows — high S (shape)
    (0x2190, 0x21FF): mol_pack(1, 5, 4, 4, 2),
    (0x27F0, 0x27FF): mol_pack(1, 5, 4, 4, 2),    # Supplemental Arrows-A
    (0x2900, 0x297F): mol_pack(1, 5, 4, 4, 2),    # Supplemental Arrows-B

    # Box Drawing / Geometric
    (0x2500, 0x257F): mol_pack(1, 2, 4, 2, 0),
    (0x2580, 0x259F): mol_pack(1, 1, 4, 2, 0),
    (0x25A0, 0x25FF): mol_pack(0, 0, 4, 3, 0),

    # Emoticons — high V/A
    (0x1F600, 0x1F64F): mol_pack(0, 0, 6, 6, 2),
    (0x1F300, 0x1F5FF): mol_pack(0, 0, 6, 5, 2),
    (0x1F680, 0x1F6FF): mol_pack(7, 5, 5, 5, 2),
    (0x1F900, 0x1F9FF): mol_pack(0, 0, 5, 5, 1),

    # CJK Unified Ideographs
    (0x4E00, 0x9FFF): mol_pack(0, 0, 4, 4, 2),
    (0x3400, 0x4DBF): mol_pack(0, 0, 4, 4, 2),    # CJK Ext A

    # Musical
    (0x1D100, 0x1D1FF): mol_pack(0, 0, 5, 5, 3),

    # Latin Extended — same as lowercase
    (0x0080, 0x024F): mol_pack(0, 0, 4, 4, 2),
    (0x1E00, 0x1EFF): mol_pack(0, 0, 4, 4, 2),    # Latin Extended Additional (Vietnamese)

    # Arabic
    (0x0600, 0x06FF): mol_pack(0, 0, 4, 4, 2),
    # Devanagari
    (0x0900, 0x097F): mol_pack(0, 0, 4, 4, 2),
    # Thai
    (0x0E00, 0x0E7F): mol_pack(0, 0, 4, 4, 2),
    # Hangul
    (0xAC00, 0xD7AF): mol_pack(0, 0, 4, 4, 2),
    # Hiragana/Katakana
    (0x3040, 0x309F): mol_pack(0, 0, 4, 4, 2),
    (0x30A0, 0x30FF): mol_pack(0, 0, 4, 4, 2),

    # Dingbats
    (0x2700, 0x27BF): mol_pack(8, 0, 5, 4, 1),
    # Misc Symbols
    (0x2600, 0x26FF): mol_pack(0, 0, 5, 5, 1),
    # Technical
    (0x2300, 0x23FF): mol_pack(7, 4, 4, 3, 2),
    # Letterlike
    (0x2100, 0x214F): mol_pack(0, 2, 4, 3, 1),

    # Braille
    (0x2800, 0x28FF): mol_pack(1, 0, 4, 3, 0),
    # Currency
    (0x20A0, 0x20CF): mol_pack(0, 5, 4, 4, 0),
}

def get_block_override(cp):
    for (start, end), pw in BLOCK_OVERRIDES.items():
        if start <= cp <= end:
            return pw
    return None

def main():
    print("Loading existing UDC data...")
    udc_entries = load_existing_udc()
    bin_entries = load_existing_bin()
    print(f"  udc.json: {len(udc_entries)} entries")
    print(f"  udc_p_table.bin: {len(bin_entries)} entries")

    # Build full table
    # Cover up to codepoint 0x26000 (155,648) to include all common Unicode
    MAX_CP = 0x26000
    table = [0] * MAX_CP

    stats = {'udc_json': 0, 'bin_existing': 0, 'block_override': 0, 'category': 0, 'zero': 0}

    for cp in range(MAX_CP):
        # Priority 1: existing bin table (includes udc.json compiled data)
        if cp in bin_entries and bin_entries[cp] > 0:
            table[cp] = bin_entries[cp]
            stats['bin_existing'] += 1
            continue

        # Priority 2: block override
        pw = get_block_override(cp)
        if pw and pw > 0:
            table[cp] = pw
            stats['block_override'] += 1
            continue

        # Priority 3: Unicode category
        try:
            name = unicodedata.name(chr(cp), None)
            if name:
                cat = unicodedata.category(chr(cp))
                pw = CATEGORY_DEFAULTS.get(cat, 0)
                if pw > 0:
                    table[cp] = pw
                    stats['category'] += 1
                    continue
        except:
            pass

        stats['zero'] += 1

    # Count non-zero
    nonzero = sum(1 for pw in table if pw > 0)

    print(f"\nFull table: {MAX_CP} entries")
    print(f"  From bin (existing):  {stats['bin_existing']}")
    print(f"  From block override:  {stats['block_override']}")
    print(f"  From Unicode category: {stats['category']}")
    print(f"  Zero (unassigned):    {stats['zero']}")
    print(f"  Total non-zero:       {nonzero} ({nonzero*100//MAX_CP}%)")

    # Write binary table
    out_path = os.path.join(ORIGIN, 'json', 'udc_p_table_full.bin')
    with open(out_path, 'wb') as f:
        for pw in table:
            f.write(struct.pack('<H', pw & 0xFFFF))

    out_size = os.path.getsize(out_path)
    print(f"\nWritten: {out_path}")
    print(f"  Size: {out_size} bytes ({out_size//1024}KB)")
    print(f"  Entries: {MAX_CP}")
    print(f"  Coverage: {nonzero}/{MAX_CP} = {nonzero*100//MAX_CP}%")

    # Verify some known values
    print(f"\nVerification:")
    test_chars = [
        (ord('a'), 'a'),
        (ord('A'), 'A'),
        (0x2202, '∂ partial'),
        (0x2207, '∇ nabla'),
        (0x222B, '∫ integral'),
        (0x2211, '∑ sum'),
        (0x1F600, '😀 grinning'),
        (0x4E00, '一 CJK first'),
    ]
    for cp, name in test_chars:
        if cp < MAX_CP:
            pw = table[cp]
            s = (pw >> 12) & 0xF
            r = (pw >> 8) & 0xF
            v = (pw >> 5) & 0x7
            a = (pw >> 2) & 0x7
            t = pw & 0x3
            print(f"  U+{cp:04X} {name}: pw={pw} S={s} R={r} V={v} A={a} T={t}")
        else:
            print(f"  U+{cp:04X} {name}: out of range")

if __name__ == '__main__':
    main()
