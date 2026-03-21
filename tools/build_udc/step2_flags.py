#!/usr/bin/env python3
"""
Step 2: Parse PropList.txt + emoji-data.txt + NameAliases.txt + StandardizedVariants.txt
        → gán flags cho mỗi codepoint.

Input:  /tmp/udc_build/step1_base.json
Output: /tmp/udc_build/step2_flags.json

Adds to each char:
  - flags: ["Emoji", "Emoji_Presentation", "Math", "White_Space", ...]
  - aliases: from NameAliases.txt
  - variants: from StandardizedVariants.txt
"""

import json
import os
import sys
from collections import defaultdict

BASE_DIR = os.path.join(os.path.dirname(__file__), '..', '..')
JSON_DIR = os.path.join(BASE_DIR, 'json')
OUTPUT_DIR = '/tmp/udc_build'


def parse_prop_list(path):
    """Parse PropList.txt → dict of property_name → set of codepoints."""
    props = defaultdict(set)
    with open(path, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            if ';' not in line:
                continue
            range_part, prop = line.split(';', 1)
            prop = prop.split('#')[0].strip()
            range_part = range_part.strip()

            if '..' in range_part:
                start_s, end_s = range_part.split('..')
                start = int(start_s, 16)
                end = int(end_s, 16)
                for cp in range(start, end + 1):
                    props[prop].add(cp)
            else:
                cp = int(range_part, 16)
                props[prop].add(cp)
    return props


def parse_emoji_data(path):
    """Parse emoji-data.txt → dict of property → set of codepoints."""
    props = defaultdict(set)
    with open(path, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            if ';' not in line:
                continue
            range_part, rest = line.split(';', 1)
            prop = rest.split('#')[0].strip()
            range_part = range_part.strip()

            if '..' in range_part:
                start_s, end_s = range_part.split('..')
                start = int(start_s, 16)
                end = int(end_s, 16)
                for cp in range(start, end + 1):
                    props[prop].add(cp)
            else:
                cp = int(range_part, 16)
                props[prop].add(cp)
    return props


def parse_name_aliases(path):
    """Parse NameAliases.txt → dict of cp → list of (alias, type)."""
    aliases = defaultdict(list)
    with open(path, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            parts = line.split(';')
            if len(parts) >= 3:
                cp = int(parts[0].strip(), 16)
                alias = parts[1].strip()
                atype = parts[2].strip()
                aliases[cp].append({"alias": alias, "type": atype})
    return aliases


def parse_emoji_test(path):
    """Parse emoji-test.txt → dict of cp_sequence → {name, status, group, subgroup}."""
    emojis = {}
    current_group = ""
    current_subgroup = ""

    with open(path, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.rstrip()
            if line.startswith('# group:'):
                current_group = line.split(':', 1)[1].strip()
                continue
            if line.startswith('# subgroup:'):
                current_subgroup = line.split(':', 1)[1].strip()
                continue
            if not line or line.startswith('#'):
                continue

            # Format: codepoints ; status # emoji name
            if ';' not in line:
                continue
            cp_part, rest = line.split(';', 1)
            status_part = rest.split('#', 1)
            status = status_part[0].strip()

            if '#' in rest:
                comment = rest.split('#', 1)[1].strip()
                # Extract version and name
                # Format: "emoji_char E14.0 name"
                parts = comment.split(' ', 2)
                if len(parts) >= 3:
                    emoji_name = parts[2] if len(parts) > 2 else parts[-1]
                else:
                    emoji_name = comment
            else:
                emoji_name = ""

            # Parse codepoint sequence
            cps = []
            for c in cp_part.strip().split():
                cps.append(int(c, 16))

            key = tuple(cps)
            emojis[key] = {
                "status": status,
                "group": current_group,
                "subgroup": current_subgroup,
                "name": emoji_name,
            }

    return emojis


def main():
    print("Step 2: Parse PropList + emoji-data + NameAliases → flags")
    print("=" * 60)

    # Load step1
    step1_path = os.path.join(OUTPUT_DIR, 'step1_base.json')
    print(f"  Loading {step1_path}...")
    with open(step1_path, 'r', encoding='utf-8') as f:
        data = json.load(f)

    # Parse PropList
    proplist_path = os.path.join(JSON_DIR, 'PropList.txt')
    print(f"  Parsing PropList.txt...")
    proplist = parse_prop_list(proplist_path)
    print(f"  → {len(proplist)} properties")
    for p in sorted(proplist.keys()):
        print(f"    {p}: {len(proplist[p])} chars")

    # Parse emoji-data
    emoji_data_path = os.path.join(JSON_DIR, 'emoji', 'emoji-data.txt')
    print(f"\n  Parsing emoji-data.txt...")
    emoji_props = parse_emoji_data(emoji_data_path)
    print(f"  → {len(emoji_props)} emoji properties")
    for p in sorted(emoji_props.keys()):
        print(f"    {p}: {len(emoji_props[p])} chars")

    # Parse NameAliases
    aliases_path = os.path.join(JSON_DIR, 'NameAliases.txt')
    print(f"\n  Parsing NameAliases.txt...")
    name_aliases = parse_name_aliases(aliases_path)
    print(f"  → {len(name_aliases)} chars with aliases")

    # Parse emoji-test.txt
    emoji_test_path = os.path.join(JSON_DIR, 'emoji', 'emoji-test.txt')
    print(f"\n  Parsing emoji-test.txt...")
    emoji_test = parse_emoji_test(emoji_test_path)
    # Extract single-codepoint emojis for direct mapping
    single_emoji = {}
    for key, info in emoji_test.items():
        if len(key) == 1:
            single_emoji[key[0]] = info
    print(f"  → {len(emoji_test)} emoji sequences ({len(single_emoji)} single-cp)")

    # Collect all emoji groups/subgroups
    emoji_groups = set()
    emoji_subgroups = set()
    for info in emoji_test.values():
        emoji_groups.add(info["group"])
        emoji_subgroups.add(info["subgroup"])
    print(f"  → {len(emoji_groups)} groups, {len(emoji_subgroups)} subgroups")

    # Relevant properties we track
    relevant_props = [
        "White_Space", "Dash", "Quotation_Mark", "Terminal_Punctuation",
        "Hex_Digit", "ASCII_Hex_Digit", "Diacritic", "Extender",
        "Ideographic", "Unified_Ideograph",
    ]
    relevant_emoji_props = [
        "Emoji", "Emoji_Presentation", "Emoji_Modifier_Base",
        "Emoji_Modifier", "Emoji_Component", "Extended_Pictographic",
    ]

    # Apply flags to chars in tree
    print(f"\n  Applying flags to chars...")
    flagged_count = 0
    emoji_enriched = 0

    for pid, plane in data["planes"].items():
        for bname, block in plane["blocks"].items():
            for cp_hex, char_data in block.get("chars", {}).items():
                cp = int(cp_hex, 16)

                # Flags from PropList
                flags = []
                for prop in relevant_props:
                    if cp in proplist.get(prop, set()):
                        flags.append(prop)

                # Flags from emoji-data
                for prop in relevant_emoji_props:
                    if cp in emoji_props.get(prop, set()):
                        flags.append(prop)

                if flags:
                    char_data["flags"] = flags
                    flagged_count += 1

                # Name aliases
                if cp in name_aliases:
                    char_data["name_aliases"] = name_aliases[cp]

                # Emoji test info (group/subgroup)
                if cp in single_emoji:
                    einfo = single_emoji[cp]
                    char_data["emoji_group"] = einfo["group"]
                    char_data["emoji_subgroup"] = einfo["subgroup"]
                    char_data["emoji_name"] = einfo["name"]
                    char_data["emoji_status"] = einfo["status"]
                    emoji_enriched += 1

    print(f"  → {flagged_count} chars with flags")
    print(f"  → {emoji_enriched} chars with emoji group/subgroup info")

    # Store emoji sequences (ZWJ, multi-cp) separately
    multi_sequences = {}
    for key, info in emoji_test.items():
        if len(key) > 1:
            seq_hex = " ".join(f"{c:04X}" for c in key)
            multi_sequences[seq_hex] = {
                "codepoints": [f"{c:04X}" for c in key],
                "group": info["group"],
                "subgroup": info["subgroup"],
                "name": info["name"],
                "status": info["status"],
            }

    data["emoji_sequences"] = {
        "count": len(multi_sequences),
        "sequences": multi_sequences,
    }

    # Update metadata
    data["step"] = "2/6 — flags from PropList + emoji-data + NameAliases"
    data["flag_summary"] = {
        "flagged_chars": flagged_count,
        "emoji_enriched": emoji_enriched,
        "emoji_sequences": len(multi_sequences),
        "emoji_groups": sorted(emoji_groups),
        "emoji_subgroups_count": len(emoji_subgroups),
    }

    # Output
    output_path = os.path.join(OUTPUT_DIR, 'step2_flags.json')
    print(f"\n  Writing {output_path}...")
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, ensure_ascii=False, indent=None, separators=(',', ':'))

    fsize = os.path.getsize(output_path)
    print(f"  → {fsize / 1024 / 1024:.1f} MB")
    print("\nStep 2 DONE ✓")


if __name__ == '__main__':
    main()
