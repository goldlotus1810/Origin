#!/usr/bin/env python3
"""
Step 5: Compute S (Shape), R (Relation), T (Time) from data.

S (4 bits, 0-15): Shape primitive from block group + category + name patterns
R (4 bits, 0-15): Relation type from math operators + category + description
T (2 bits, 0-3):  Time mode from block + temporal properties

All derived from data — NOT hardcoded values.

Sources:
  - Block group (SDF/MATH/EMOTICON/MUSICAL) → dominant axis
  - Category (Lu/Sm/So/...) → default shape/relation
  - PropList flags → refine
  - Name patterns → SDF primitive identification
  - StandardizedVariants.txt → shape variant info
  - PropertyValueAliases.txt → relation mapping

Input:  /tmp/udc_build/step4_aliases.json
Output: /tmp/udc_build/step5_srt.json
"""

import json
import os
import re
from collections import defaultdict

BASE_DIR = os.path.join(os.path.dirname(__file__), '..', '..')
JSON_DIR = os.path.join(BASE_DIR, 'json')
OUTPUT_DIR = '/tmp/udc_build'

# ─── 18 SDF PRIMITIVES (from spec) ───
# Encoded as 4 bits (0-15), with 16-17 for composite/other
SDF_PRIMITIVES = {
    0:  "SPHERE",
    1:  "BOX",
    2:  "CAPSULE",
    3:  "PLANE",
    4:  "TORUS",
    5:  "ELLIPSOID",
    6:  "CONE",
    7:  "CYLINDER",
    8:  "OCTAHEDRON",
    9:  "PYRAMID",
    10: "HEX_PRISM",
    11: "PRISM",
    12: "ROUND_BOX",
    13: "LINK",
    14: "REVOLVE",
    15: "EXTRUDE",
    # 16: CUT_SPHERE (overflow, use 0 + flag)
    # 17: DEATH_STAR (overflow, use 15 + flag)
}

# ─── RELATION TYPES (from spec) ───
RELATION_TYPES = {
    0:  "IDENTITY",     # self, default for letters/digits
    1:  "MEMBER",       # ∈, ∋
    2:  "SUBSET",       # ⊂, ⊃, ⊆, ⊇
    3:  "EQUALITY",     # =, ≠, ≡, ≢
    4:  "ORDER",        # <, >, ≤, ≥
    5:  "ARITHMETIC",   # +, −, ×, ÷
    6:  "LOGICAL",      # ∧, ∨, ¬, ∀, ∃
    7:  "SET_OP",       # ∪, ∩, ∖, △
    8:  "COMPOSE",      # ∘, ∙
    9:  "CAUSES",       # →, ⇒, ↦
    10: "APPROXIMATE",  # ≈, ≃, ∼
    11: "ORTHOGONAL",   # ⊥, ∦
    12: "AGGREGATE",    # ∫, ∑, ∏
    13: "DIRECTIONAL",  # arrows (←→↑↓)
    14: "BRACKET",      # grouping (({[]})
    15: "INVERSE",      # negation, complement
}

# ─── TIME MODES (from spec, 2 bits) ───
TIME_MODES = {
    0: "TIMELESS",     # static: letters, math symbols
    1: "SEQUENTIAL",   # has temporal order: scripts, arrows
    2: "CYCLICAL",     # repeating: calendar, clock, zodiac
    3: "RHYTHMIC",     # musical: notes, rhythm
}


def compute_S(name, cat, flags, group, block_name):
    """
    Compute Shape (S) from Unicode data.
    S is the SDF primitive that best describes the visual shape.

    Derived from:
    - Block name patterns → geometric shapes
    - Character name → shape keywords
    - Category → default shape
    - Emoji group → face/body/nature shapes
    """
    name_up = name.upper()
    bn_up = block_name.upper()

    # ─── 1. Geometric shapes (direct SDF match from name) ───
    shape_keywords = {
        'SPHERE': 0, 'CIRCLE': 0, 'RING': 0, 'DOT': 0, 'BALL': 0,
        'ROUND': 0, 'DISC': 0, 'FACE': 0, 'SUN': 0, 'MOON': 0,
        'BOX': 1, 'SQUARE': 1, 'RECTANGLE': 1, 'BLOCK': 1, 'FULL': 1,
        'CAPSULE': 2, 'LOZENGE': 2, 'PILL': 2, 'ROUNDED': 2, 'OVAL': 5,
        'LINE': 3, 'HORIZONTAL': 3, 'BAR': 3, 'DASH': 3, 'RULE': 3,
        'TORUS': 4, 'DONUT': 4,
        'ELLIPSE': 5, 'EGG': 5, 'OVOID': 5,
        'CONE': 6, 'TRIANGLE': 6, 'WEDGE': 6, 'POINTED': 6,
        'CHEVRON': 6, 'ARROWHEAD': 6,
        'CYLINDER': 7, 'BARREL': 7, 'CAN': 7,
        'OCTAGON': 8, 'DIAMOND': 8,
        'PYRAMID': 9,
        'HEXAGON': 10, 'HEX': 10, 'HONEYCOMB': 10,
        'PRISM': 11, 'CUBE': 11,
        'STAR': 12, 'ASTERISK': 12, 'SPARKLE': 12,
        'LINK': 13, 'CHAIN': 13, 'INFINITY': 13,
        'ARROW': 14, 'POINTER': 14, 'DIRECTION': 14, 'CURSOR': 14,
        'CROSS': 15, 'PLUS': 15, 'MARK': 15,
    }

    # Check name for shape keywords
    for keyword, s_val in shape_keywords.items():
        if keyword in name_up:
            return s_val

    # ─── 2. Emoji shape from group ───
    # (already stored as emoji_group/emoji_subgroup in step2)

    # ─── 3. Category defaults ───
    cat_shapes = {
        'Lu': 1,   # uppercase = box-like (blocks of text)
        'Ll': 0,   # lowercase = round (softer)
        'Lt': 1,   # titlecase = box
        'Lm': 3,   # modifier = line
        'Lo': 0,   # other letter = sphere
        'Mn': 3,   # non-spacing mark = line/plane
        'Mc': 3,   # combining mark = line
        'Me': 1,   # enclosing mark = box
        'Nd': 1,   # digit = box-like
        'Nl': 1,   # number letter
        'No': 0,   # number other
        'Pc': 3,   # connector punct = line
        'Pd': 3,   # dash = line
        'Ps': 14,  # open bracket = curved
        'Pe': 14,  # close bracket = curved
        'Pi': 14,  # initial quote
        'Pf': 14,  # final quote
        'Po': 0,   # other punct = dot
        'Sm': 15,  # math symbol = cross/operator
        'Sc': 1,   # currency = box
        'Sk': 3,   # modifier symbol = line
        'So': 0,   # other symbol = sphere (default)
        'Zs': 3,   # space = plane
        'Zl': 3,
        'Zp': 3,
        'Cc': 3,   # control = plane
        'Cf': 3,
    }
    return cat_shapes.get(cat, 0)


def compute_R(name, cat, flags, group, block_name, cp):
    """
    Compute Relation (R) from Unicode data.

    Derived from:
    - Math operator names → direct relation type
    - Arrow names → directional
    - Bracket category → bracket relation
    - Category → default relation
    """
    name_up = name.upper()

    # ─── 1. Direct math relation from name ───
    if 'MEMBER' in name_up or 'ELEMENT' in name_up or 'CONTAIN' in name_up:
        return 1  # MEMBER
    if 'SUBSET' in name_up or 'SUPERSET' in name_up or 'SUBGROUP' in name_up:
        return 2  # SUBSET
    if 'EQUAL' in name_up or 'IDENTICAL' in name_up or 'CONGRUENT' in name_up:
        return 3  # EQUALITY
    if 'LESS' in name_up or 'GREATER' in name_up or 'PRECEDE' in name_up or 'SUCCEED' in name_up:
        return 4  # ORDER
    if 'PLUS' in name_up or 'MINUS' in name_up or 'MULTIPLY' in name_up or 'DIVISION' in name_up:
        return 5  # ARITHMETIC
    if 'TIMES' in name_up and cat == 'Sm':
        return 5  # ARITHMETIC
    if 'LOGICAL' in name_up or 'FOR ALL' in name_up or 'THERE EXISTS' in name_up:
        return 6  # LOGICAL
    if 'AND' == name_up or 'OR' == name_up or 'NOT' in name_up:
        if cat == 'Sm':
            return 6
    if 'UNION' in name_up or 'INTERSECTION' in name_up or 'COMPLEMENT' in name_up:
        return 7  # SET_OP
    if 'RING OPERATOR' in name_up or 'COMPOSE' in name_up or 'DOT OPERATOR' in name_up:
        return 8  # COMPOSE
    if 'IMPLIES' in name_up or 'RIGHT ARROW' in name_up or 'MAPS TO' in name_up:
        if cat == 'Sm':
            return 9  # CAUSES
    if 'TILDE' in name_up or 'APPROX' in name_up or 'ASYMPTOT' in name_up:
        return 10  # APPROXIMATE
    if 'PERPENDICULAR' in name_up or 'ORTHOGONAL' in name_up or 'NORMAL' in name_up:
        if cat == 'Sm':
            return 11  # ORTHOGONAL
    if 'INTEGRAL' in name_up or 'SUMMATION' in name_up or 'PRODUCT' in name_up:
        if cat == 'Sm':
            return 12  # AGGREGATE
    if 'N-ARY' in name_up:
        return 12  # AGGREGATE

    # ─── 2. Arrow → DIRECTIONAL ───
    if 'ARROW' in name_up and group == 'SDF':
        return 13  # DIRECTIONAL

    # ─── 3. Bracket → BRACKET ───
    if cat in ('Ps', 'Pe', 'Pi', 'Pf'):
        return 14  # BRACKET

    # ─── 4. Negation/inverse ───
    if 'NOT' in name_up and cat == 'Sm':
        return 15  # INVERSE
    if 'REVERSED' in name_up or 'INVERTED' in name_up:
        if cat == 'Sm':
            return 15

    # ─── 5. Category defaults ───
    if group == 'MATH':
        # Default for math block chars not caught above
        if cat == 'Sm':
            return 5  # ARITHMETIC default for math symbols
        return 3  # EQUALITY default for math text

    if group == 'SDF':
        return 13  # DIRECTIONAL for shape blocks (arrows, etc)

    if group == 'EMOTICON':
        return 9  # CAUSES (emoji tend to express causation: fire→hot)

    if group == 'MUSICAL':
        return 8  # COMPOSE (music = composition)

    # Default: IDENTITY (self-referential, no relation)
    return 0


def compute_T(name, cat, flags, group, block_name, cp):
    """
    Compute Time (T) from Unicode data.

    2 bits:
    00 = TIMELESS   — static: letters, basic symbols
    01 = SEQUENTIAL — has temporal order: historical scripts, arrows
    10 = CYCLICAL   — repeating: calendar, clock, zodiac
    11 = RHYTHMIC   — musical: notes, rhythm

    Derived from:
    - Block group (MUSICAL → RHYTHMIC)
    - Name patterns (CLOCK, CALENDAR → CYCLICAL)
    - Category/block → default
    """
    name_up = name.upper()

    # ─── RHYTHMIC: musical blocks ───
    if group == 'MUSICAL':
        return 3

    # ─── CYCLICAL: clock, calendar, zodiac, seasonal ───
    cyclical_keywords = [
        'CLOCK', 'WATCH', 'TIMER', 'HOURGLASS', 'CALENDAR',
        'ZODIAC', 'ARIES', 'TAURUS', 'GEMINI', 'CANCER', 'LEO',
        'VIRGO', 'LIBRA', 'SCORPIO', 'SAGITTARIUS', 'CAPRICORN',
        'AQUARIUS', 'PISCES',
        'SEASON', 'SPRING', 'SUMMER', 'AUTUMN', 'WINTER',
        'SUNRISE', 'SUNSET', 'MOON PHASE',
        'RECYCLING', 'CYCLE', 'REFRESH', 'REPEAT',
        'YIN YANG', 'INFINITY',
    ]
    for kw in cyclical_keywords:
        if kw in name_up:
            return 2

    # CYCLICAL: Yijing hexagrams
    if 'YIJING' in block_name.upper() or 'HEXAGRAM' in name_up:
        return 2

    # ─── SEQUENTIAL: arrows, historical scripts, playing sequence ───
    if 'ARROW' in name_up:
        return 1
    if group == 'SDF' and ('ARROW' in block_name.upper()):
        return 1

    # Historical/ancient scripts → sequential
    historical = ['CUNEIFORM', 'HIEROGLYPH', 'RUNIC', 'OGHAM', 'LINEAR',
                  'PHOENICIAN', 'OLD', 'ANCIENT', 'IMPERIAL']
    for kw in historical:
        if kw in block_name.upper():
            return 1

    # Playing cards, domino, mahjong → sequential (game turns)
    if 'PLAYING' in block_name.upper() or 'DOMINO' in block_name.upper():
        return 1
    if 'MAHJONG' in block_name.upper():
        return 1

    # ─── TIMELESS: everything else ───
    return 0


def main():
    print("Step 5: Compute S (Shape), R (Relation), T (Time)")
    print("=" * 60)

    # Load step4
    step4_path = os.path.join(OUTPUT_DIR, 'step4_aliases.json')
    print(f"  Loading {step4_path}...")
    with open(step4_path, 'r', encoding='utf-8') as f:
        data = json.load(f)

    # Process all chars
    print(f"\n  Computing S, R, T for all chars...")
    stats = {
        'S': defaultdict(int),
        'R': defaultdict(int),
        'T': defaultdict(int),
    }

    for pid, plane in data["planes"].items():
        for bname, block in plane["blocks"].items():
            group = block.get("group", "OTHER")
            for cp_hex, char_data in block.get("chars", {}).items():
                cp = int(cp_hex, 16)
                name = char_data.get("name", "")
                cat = char_data.get("cat", "")
                flags = char_data.get("flags", [])

                S = compute_S(name, cat, flags, group, bname)
                R = compute_R(name, cat, flags, group, bname, cp)
                T = compute_T(name, cat, flags, group, bname, cp)

                char_data["S"] = S
                char_data["R"] = R
                char_data["T"] = T

                stats['S'][SDF_PRIMITIVES.get(S, f"unknown_{S}")] += 1
                stats['R'][RELATION_TYPES.get(R, f"unknown_{R}")] += 1
                stats['T'][TIME_MODES.get(T, f"unknown_{T}")] += 1

    print(f"\n  S (Shape) distribution:")
    for k, v in sorted(stats['S'].items(), key=lambda x: -x[1])[:10]:
        print(f"    {k}: {v}")

    print(f"\n  R (Relation) distribution:")
    for k, v in sorted(stats['R'].items(), key=lambda x: -x[1])[:10]:
        print(f"    {k}: {v}")

    print(f"\n  T (Time) distribution:")
    for k, v in sorted(stats['T'].items(), key=lambda x: -x[1]):
        print(f"    {k}: {v}")

    # Spot check
    print(f"\n  Spot check:")
    checks = [
        (0x1F525, "🔥"), (0x1F60A, "😊"), (0x25CF, "●"),
        (0x2208, "∈"), (0x222B, "∫"), (0x2190, "←"),
        (0x1D11E, "𝄞"), (0x0041, "A"), (0x0030, "0"),
    ]
    for cp, ch in checks:
        plane_id = str(cp >> 16)
        if plane_id in data["planes"]:
            for bname, block in data["planes"][plane_id]["blocks"].items():
                cp_hex = f"{cp:04X}"
                if cp_hex in block.get("chars", {}):
                    cd = block["chars"][cp_hex]
                    s_name = SDF_PRIMITIVES.get(cd['S'], '?')
                    r_name = RELATION_TYPES.get(cd['R'], '?')
                    t_name = TIME_MODES.get(cd['T'], '?')
                    print(f"    {ch} U+{cp:04X}: S={cd['S']}({s_name}) R={cd['R']}({r_name}) T={cd['T']}({t_name})")
                    break

    # Store lookup tables in metadata
    data["sdf_primitives"] = SDF_PRIMITIVES
    data["relation_types"] = RELATION_TYPES
    data["time_modes"] = TIME_MODES

    # Update metadata
    data["step"] = "5/6 — S/R/T from block + category + name patterns"
    data["srt_summary"] = {
        "S_distribution": {k: v for k, v in sorted(stats['S'].items(), key=lambda x: -x[1])},
        "R_distribution": {k: v for k, v in sorted(stats['R'].items(), key=lambda x: -x[1])},
        "T_distribution": {k: v for k, v in sorted(stats['T'].items(), key=lambda x: -x[1])},
    }

    # Output
    output_path = os.path.join(OUTPUT_DIR, 'step5_srt.json')
    print(f"\n  Writing {output_path}...")
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, ensure_ascii=False, indent=None, separators=(',', ':'))

    fsize = os.path.getsize(output_path)
    print(f"  → {fsize / 1024 / 1024:.1f} MB")
    print("\nStep 5 DONE ✓")


if __name__ == '__main__':
    main()
