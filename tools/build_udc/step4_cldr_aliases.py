#!/usr/bin/env python3
"""
Step 4: Extract CLDR annotations (vi/en) → aliases for L0.

L0 lưu: UDC codepoint + en alias + vi alias
Aliases khác (ja, zh...) để ngoài L0-L1.

CLDR annotation format:
  "😊": {"default": ["face", "happy", "smile"], "tts": ["smiling face with smiling eyes"]}
  → en_keywords = ["face", "happy", "smile"]
  → en_tts = "smiling face with smiling eyes"

Input:  /tmp/udc_build/step3_va.json
        /tmp/cldr_en/annotations.json
        /tmp/cldr_vi/annotations.json
Output: /tmp/udc_build/step4_aliases.json
"""

import json
import os
from collections import defaultdict

OUTPUT_DIR = '/tmp/udc_build'
CLDR_EN = '/tmp/cldr_en/annotations.json'
CLDR_VI = '/tmp/cldr_vi/annotations.json'


def load_cldr_annotations(path, lang):
    """Load CLDR annotation file → dict of char_str → {keywords, tts}"""
    with open(path, 'r', encoding='utf-8') as f:
        data = json.load(f)

    annotations = data.get('annotations', {}).get('annotations', {})
    result = {}

    for char_str, info in annotations.items():
        # char_str can be a single emoji or multi-codepoint sequence
        cps = [ord(c) for c in char_str]

        entry = {}
        if 'default' in info:
            entry['keywords'] = info['default']
        if 'tts' in info:
            entry['tts'] = info['tts'][0] if isinstance(info['tts'], list) else info['tts']

        if entry:
            if len(cps) == 1:
                result[cps[0]] = entry
            else:
                # Multi-codepoint sequence — store separately
                key = tuple(cps)
                result[key] = entry

    return result


def main():
    print("Step 4: Extract CLDR annotations (vi/en) → aliases")
    print("=" * 60)

    # Load step3
    step3_path = os.path.join(OUTPUT_DIR, 'step3_va.json')
    print(f"  Loading {step3_path}...")
    with open(step3_path, 'r', encoding='utf-8') as f:
        data = json.load(f)

    # Load CLDR
    print(f"  Loading CLDR EN...")
    cldr_en = load_cldr_annotations(CLDR_EN, 'en')
    en_single = {k: v for k, v in cldr_en.items() if isinstance(k, int)}
    en_multi = {k: v for k, v in cldr_en.items() if isinstance(k, tuple)}
    print(f"  → {len(en_single)} single-cp, {len(en_multi)} multi-cp sequences")

    print(f"  Loading CLDR VI...")
    cldr_vi = load_cldr_annotations(CLDR_VI, 'vi')
    vi_single = {k: v for k, v in cldr_vi.items() if isinstance(k, int)}
    vi_multi = {k: v for k, v in cldr_vi.items() if isinstance(k, tuple)}
    print(f"  → {len(vi_single)} single-cp, {len(vi_multi)} multi-cp sequences")

    # Apply to chars in tree
    print(f"\n  Applying CLDR aliases to chars...")
    stats = defaultdict(int)

    for pid, plane in data["planes"].items():
        for bname, block in plane["blocks"].items():
            for cp_hex, char_data in block.get("chars", {}).items():
                cp = int(cp_hex, 16)

                aliases = {}

                # English
                if cp in en_single:
                    en = en_single[cp]
                    aliases["en"] = {}
                    if 'tts' in en:
                        aliases["en"]["tts"] = en['tts']
                    if 'keywords' in en:
                        aliases["en"]["keywords"] = en['keywords']
                    stats["en"] += 1

                # Vietnamese
                if cp in vi_single:
                    vi = vi_single[cp]
                    aliases["vi"] = {}
                    if 'tts' in vi:
                        aliases["vi"]["tts"] = vi['tts']
                    if 'keywords' in vi:
                        aliases["vi"]["keywords"] = vi['keywords']
                    stats["vi"] += 1

                if aliases:
                    char_data["cldr"] = aliases
                    stats["total"] += 1

    print(f"\n  CLDR alias stats:")
    for k, v in sorted(stats.items()):
        print(f"    {k}: {v}")

    # Store multi-cp CLDR sequences in emoji_sequences
    if "emoji_sequences" in data:
        cldr_seq = {}
        for key, en_info in en_multi.items():
            seq_hex = " ".join(f"{c:04X}" for c in key)
            entry = {"en": en_info}
            if key in vi_multi:
                entry["vi"] = vi_multi[key]
            cldr_seq[seq_hex] = entry

        # Also add vi-only sequences
        for key, vi_info in vi_multi.items():
            seq_hex = " ".join(f"{c:04X}" for c in key)
            if seq_hex not in cldr_seq:
                cldr_seq[seq_hex] = {"vi": vi_info}

        data["cldr_sequences"] = {
            "count": len(cldr_seq),
            "sequences": cldr_seq,
        }
        print(f"  → {len(cldr_seq)} multi-cp CLDR sequences stored")

    # Spot check
    print(f"\n  Spot check (CLDR aliases):")
    checks = [0x1F525, 0x1F60A, 0x1F494, 0x2764, 0x1F389]
    for cp in checks:
        plane_id = str(cp >> 16)
        if plane_id in data["planes"]:
            for bname, block in data["planes"][plane_id]["blocks"].items():
                cp_hex = f"{cp:04X}"
                if cp_hex in block.get("chars", {}):
                    cd = block["chars"][cp_hex]
                    cldr = cd.get("cldr", {})
                    en_tts = cldr.get("en", {}).get("tts", "?")
                    vi_tts = cldr.get("vi", {}).get("tts", "?")
                    print(f"    U+{cp:04X}: EN={en_tts} | VI={vi_tts}")
                    break

    # Update metadata
    data["step"] = "4/6 — CLDR aliases (vi/en) for L0"
    data["cldr_summary"] = dict(stats)

    # Output
    output_path = os.path.join(OUTPUT_DIR, 'step4_aliases.json')
    print(f"\n  Writing {output_path}...")
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, ensure_ascii=False, indent=None, separators=(',', ':'))

    fsize = os.path.getsize(output_path)
    print(f"  → {fsize / 1024 / 1024:.1f} MB")
    print("\nStep 4 DONE ✓")


if __name__ == '__main__':
    main()
