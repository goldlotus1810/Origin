#!/usr/bin/env python3
"""
Step 1: Parse UnicodeData.txt + Blocks.txt → base tree structure.

Output: /tmp/udc_build/step1_base.json
Structure: ○{plane{block{char}}}

UTF-32 IS the ○{} tree:
  Plane (5 bits) → Block → Char (16 bits position)
"""

import json
import os
import sys
from collections import defaultdict

# Paths
UNICODE_DATA = os.path.join(os.path.dirname(__file__), '../../json/UnicodeData.txt')
BLOCKS_TXT   = os.path.join(os.path.dirname(__file__), '../../json/Blocks.txt')
OUTPUT_DIR   = '/tmp/udc_build'
OUTPUT_FILE  = os.path.join(OUTPUT_DIR, 'step1_base.json')


def parse_blocks(path):
    """Parse Blocks.txt → list of (start, end, name)"""
    blocks = []
    with open(path, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            # Format: 0000..007F; Basic Latin
            if '..' not in line:
                continue
            range_part, name = line.split(';', 1)
            start_s, end_s = range_part.strip().split('..')
            start = int(start_s, 16)
            end = int(end_s, 16)
            name = name.strip()
            blocks.append((start, end, name))
    blocks.sort(key=lambda b: b[0])
    return blocks


def classify_block_group(block_name, start, end):
    """
    Classify a Unicode block into one of the 4+1 groups.
    Based on the 58 blocks defined in the spec + extension to full UTF-32.

    Returns: "SDF" | "MATH" | "EMOTICON" | "MUSICAL" | "TEXT" | "CJK" | "OTHER"
    """
    bn = block_name.upper()

    # SDF — Shape blocks (13 blocks in spec)
    sdf_patterns = [
        'ARROW', 'BOX DRAWING', 'BLOCK ELEMENT', 'GEOMETRIC',
        'DINGBAT', 'MISC SYMBOLS AND ARROWS', 'MISCELLANEOUS SYMBOLS AND ARROWS',
        'ORNAMENTAL', 'MISCELLANEOUS TECHNICAL', 'BRAILLE',
        'SUPPLEMENTAL ARROWS',
    ]
    for pat in sdf_patterns:
        if pat in bn:
            return "SDF"

    # MATH — Relation blocks (21 blocks in spec)
    math_patterns = [
        'SUPERSCRIPT', 'SUBSCRIPT', 'LETTERLIKE', 'NUMBER FORM',
        'MATHEMATICAL', 'MATH', 'SIYAQ', 'COUNTING ROD',
        'ANCIENT GREEK NUMBER', 'COPTIC EPACT', 'MAYAN NUMERAL',
        'INDIC SIYAQ', 'OTTOMAN SIYAQ', 'KAKTOVIK',
    ]
    for pat in math_patterns:
        if pat in bn:
            return "MATH"

    # EMOTICON — Valence+Arousal blocks (17 blocks in spec)
    emoticon_patterns = [
        'EMOTICON', 'ENCLOSED ALPHANUMERIC', 'MISCELLANEOUS SYMBOLS',
        'MAHJONG', 'DOMINO', 'PLAYING CARD', 'ENCLOSED IDEOGRAPHIC',
        'ENCLOSED SUPPLEMENT', 'MISC SYMBOLS AND PICTOGRAPHS',
        'MISCELLANEOUS SYMBOLS AND PICTOGRAPHS',
        'TRANSPORT', 'ALCHEMICAL', 'CHESS', 'SYMBOLS AND PICTOGRAPHS EXTENDED',
        'SUPPLEMENTAL SYMBOLS AND PICTOGRAPHS',
    ]
    for pat in emoticon_patterns:
        if pat in bn:
            return "EMOTICON"

    # MUSICAL — Time blocks (7 blocks in spec)
    musical_patterns = [
        'YIJING', 'ZNAMENNY', 'BYZANTINE', 'MUSICAL',
        'ANCIENT GREEK MUSICAL', 'TAI XUAN',
    ]
    for pat in musical_patterns:
        if pat in bn:
            return "MUSICAL"

    # Extended classifications for non-spec blocks
    if 'CJK' in bn or 'KANGXI' in bn or 'IDEOGRAPH' in bn:
        return "CJK"
    if 'LATIN' in bn or 'CYRILLIC' in bn or 'GREEK' in bn or 'ARABIC' in bn:
        return "TEXT"
    if 'HANGUL' in bn or 'KATAKANA' in bn or 'HIRAGANA' in bn:
        return "TEXT"
    if 'DEVANAGARI' in bn or 'BENGALI' in bn or 'TAMIL' in bn:
        return "TEXT"
    if 'THAI' in bn or 'TIBETAN' in bn or 'MYANMAR' in bn:
        return "TEXT"
    if 'ETHIOPIC' in bn or 'CHEROKEE' in bn or 'GEORGIAN' in bn:
        return "TEXT"
    if 'HEBREW' in bn or 'SYRIAC' in bn or 'THAANA' in bn:
        return "TEXT"
    if 'GENERAL PUNCTUATION' in bn or 'SPACING' in bn:
        return "TEXT"
    if 'PRIVATE USE' in bn or 'SURROGATE' in bn:
        return "RESERVED"
    if 'TAG' in bn or 'VARIATION SELECTOR' in bn:
        return "CONTROL"
    if 'SPECIALS' in bn or 'HALFWIDTH' in bn:
        return "TEXT"

    return "OTHER"


def parse_unicode_data(path):
    """
    Parse UnicodeData.txt → dict of cp → {name, category, ...}
    Handles range entries (e.g., CJK Unified Ideographs).
    """
    chars = {}
    range_start = None

    with open(path, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            fields = line.split(';')
            cp = int(fields[0], 16)
            name = fields[1]
            category = fields[2]

            # Handle range entries
            if name.endswith(', First>'):
                range_start = cp
                range_name = name.replace(', First>', '').replace('<', '')
                range_cat = category
                continue

            if name.endswith(', Last>') and range_start is not None:
                range_name_clean = name.replace(', Last>', '').replace('<', '')
                # Store range metadata (don't expand all CJK chars individually)
                chars[f"RANGE_{range_start:04X}_{cp:04X}"] = {
                    "type": "range",
                    "start": range_start,
                    "end": cp,
                    "name": range_name,
                    "cat": range_cat,
                    "count": cp - range_start + 1,
                }
                range_start = None
                continue

            # Normal character entry
            if name.startswith('<'):
                # Control characters
                name = name.strip('<>')

            chars[cp] = {
                "name": name,
                "cat": category,
            }

            # Store decomposition if present
            decomp = fields[5]
            if decomp:
                chars[cp]["decomp"] = decomp

    return chars


def find_block_for_cp(cp, blocks):
    """Binary search for block containing codepoint."""
    lo, hi = 0, len(blocks) - 1
    while lo <= hi:
        mid = (lo + hi) // 2
        start, end, _ = blocks[mid]
        if cp < start:
            hi = mid - 1
        elif cp > end:
            lo = mid + 1
        else:
            return blocks[mid]
    return None


def build_tree(chars, blocks):
    """
    Build ○{plane{block{char}}} tree.

    UTF-32 codepoint (21 bits):
      plane: bits 16-20 (5 bits, 0-16)
      position: bits 0-15 (16 bits, 0-65535)
    """
    tree = {}
    stats = defaultdict(int)

    # First, organize blocks by plane
    block_by_plane = defaultdict(list)
    for start, end, bname in blocks:
        plane = start >> 16
        block_by_plane[plane].append({
            "name": bname,
            "range": [start, end],
            "group": classify_block_group(bname, start, end),
            "chars": {},
        })

    # Process individual characters
    for key, info in chars.items():
        if isinstance(key, str) and key.startswith("RANGE_"):
            # Range entry (CJK, Hangul, etc.)
            rinfo = info
            plane = rinfo["start"] >> 16
            block = find_block_for_cp(rinfo["start"], blocks)
            if block:
                bname = block[2]
                # Find or create block in tree
                for b in block_by_plane[plane]:
                    if b["name"] == bname:
                        b["range_entries"] = b.get("range_entries", [])
                        b["range_entries"].append({
                            "start": rinfo["start"],
                            "end": rinfo["end"],
                            "name": rinfo["name"],
                            "cat": rinfo["cat"],
                            "count": rinfo["count"],
                        })
                        stats["range_chars"] += rinfo["count"]
                        break
            continue

        cp = key
        plane = cp >> 16
        block = find_block_for_cp(cp, blocks)

        if block:
            bname = block[2]
            for b in block_by_plane[plane]:
                if b["name"] == bname:
                    # Store char with hex key
                    b["chars"][f"{cp:04X}"] = {
                        "name": info["name"],
                        "cat": info["cat"],
                    }
                    if "decomp" in info:
                        b["chars"][f"{cp:04X}"]["decomp"] = info["decomp"]
                    stats["individual_chars"] += 1
                    break
        else:
            stats["no_block"] += 1

    # Build final tree
    planes = {}
    plane_names = {
        0: "Basic Multilingual Plane (BMP)",
        1: "Supplementary Multilingual Plane (SMP)",
        2: "Supplementary Ideographic Plane (SIP)",
        3: "Tertiary Ideographic Plane (TIP)",
        14: "Supplementary Special-purpose Plane (SSP)",
        15: "Supplementary Private Use Area-A",
        16: "Supplementary Private Use Area-B",
    }

    for plane_id in sorted(block_by_plane.keys()):
        blist = block_by_plane[plane_id]
        plane_data = {
            "name": plane_names.get(plane_id, f"Plane {plane_id}"),
            "plane_id": plane_id,
            "block_count": len(blist),
            "blocks": {},
        }

        # Count chars
        total_chars = 0
        for b in blist:
            total_chars += len(b["chars"])
            for r in b.get("range_entries", []):
                total_chars += r["count"]
        plane_data["char_count"] = total_chars

        for b in blist:
            block_data = {
                "range": b["range"],
                "group": b["group"],
                "char_count": len(b["chars"]),
            }
            if b.get("range_entries"):
                block_data["ranges"] = b["range_entries"]
                block_data["char_count"] += sum(r["count"] for r in b["range_entries"])
            block_data["chars"] = b["chars"]
            plane_data["blocks"][b["name"]] = block_data

        planes[str(plane_id)] = plane_data

    return planes, stats


def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    print("Step 1: Parse UnicodeData.txt + Blocks.txt")
    print("=" * 60)

    # Parse blocks
    blocks_path = os.path.abspath(BLOCKS_TXT)
    print(f"  Parsing Blocks: {blocks_path}")
    blocks = parse_blocks(blocks_path)
    print(f"  → {len(blocks)} blocks")

    # Parse UnicodeData
    udata_path = os.path.abspath(UNICODE_DATA)
    print(f"  Parsing UnicodeData: {udata_path}")
    chars = parse_unicode_data(udata_path)

    # Count
    individual = sum(1 for k in chars if isinstance(k, int))
    ranges = sum(1 for k in chars if isinstance(k, str) and k.startswith("RANGE_"))
    range_total = sum(v["count"] for k, v in chars.items() if isinstance(k, str) and k.startswith("RANGE_"))
    print(f"  → {individual} individual chars + {ranges} ranges ({range_total} chars)")
    print(f"  → Total: {individual + range_total} codepoints")

    # Build tree
    print("\n  Building ○{{plane{{block{{char}}}}}} tree...")
    planes, stats = build_tree(chars, blocks)

    # Summary
    print(f"\n  Stats:")
    for k, v in stats.items():
        print(f"    {k}: {v}")

    print(f"\n  Planes: {len(planes)}")
    for pid, pdata in planes.items():
        print(f"    Plane {pid} ({pdata['name']}): {pdata['block_count']} blocks, {pdata['char_count']} chars")

    # Group stats
    group_stats = defaultdict(int)
    for pid, pdata in planes.items():
        for bname, bdata in pdata["blocks"].items():
            group_stats[bdata["group"]] += bdata["char_count"]

    print(f"\n  Groups:")
    for g in ["SDF", "MATH", "EMOTICON", "MUSICAL", "TEXT", "CJK", "OTHER", "RESERVED", "CONTROL"]:
        if g in group_stats:
            print(f"    {g}: {group_stats[g]} chars")

    # Output
    output = {
        "protocol": "○{UTF32-SDF-INTEGRATOR}",
        "version": "18.0",
        "structure": "○{plane{block{char}}}",
        "step": "1/6 — base tree from UnicodeData.txt + Blocks.txt",
        "total_codepoints": individual + range_total,
        "total_blocks": len(blocks),
        "total_planes": len(planes),
        "group_summary": dict(group_stats),
        "planes": planes,
    }

    print(f"\n  Writing {OUTPUT_FILE}...")
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f:
        json.dump(output, f, ensure_ascii=False, indent=None, separators=(',', ':'))

    fsize = os.path.getsize(OUTPUT_FILE)
    print(f"  → {fsize / 1024 / 1024:.1f} MB")
    print("\nStep 1 DONE ✓")


if __name__ == '__main__':
    main()
