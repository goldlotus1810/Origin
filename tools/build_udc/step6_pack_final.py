#!/usr/bin/env python3
"""
Step 6: Pack P_weight into 16 bits + produce final JSON.

P = [S:4][R:4][V:3][A:3][T:2] = 16 bits

  S ← 4 bits (0-15) — SDF primitive
  R ← 4 bits (0-15) — Relation type
  V ← 3 bits (0-7)  — Valence quantized from float
  A ← 3 bits (0-7)  — Arousal quantized from float
  T ← 2 bits (0-3)  — Time mode

Output: json/udc_utf32.json — the final ○{} tree

KEY PRINCIPLE:
  Name IS the codepoint (UTF-32 address). English name is just an alias.
  ○(x) == x → the codepoint IS the node.
  "SPACE" is alias. "0020" is name.

Structure:
  ○{plane{block{char}}}
  JSON key = codepoint hex = THE NAME (e.g., "1F525")
  where each char has:
    - P: packed 16-bit P_weight
    - S, R, V, A, T: individual axis values
    - cat: Unicode General Category
    - aliases: {en: {name, tts, keywords}, vi: {tts, keywords}}
    - flags: property flags
    - emoji_group/subgroup: emoji classification
"""

import json
import os
import math
from collections import defaultdict

OUTPUT_DIR = '/tmp/udc_build'
FINAL_OUTPUT = os.path.join(os.path.dirname(__file__), '..', '..', 'json', 'udc_utf32.json')


def quantize_v(v_float):
    """Quantize V float [0,1] → 3 bits [0,7]"""
    return max(0, min(7, round(v_float * 7.0)))


def quantize_a(a_float):
    """Quantize A float [0,1] → 3 bits [0,7]"""
    return max(0, min(7, round(a_float * 7.0)))


def pack_p(S, R, V_q, A_q, T):
    """Pack into 16 bits: [S:4][R:4][V:3][A:3][T:2]"""
    S = S & 0xF
    R = R & 0xF
    V_q = V_q & 0x7
    A_q = A_q & 0x7
    T = T & 0x3
    return (S << 12) | (R << 8) | (V_q << 5) | (A_q << 2) | T


def unpack_p(p16):
    """Unpack 16 bits → (S, R, V, A, T)"""
    S = (p16 >> 12) & 0xF
    R = (p16 >> 8) & 0xF
    V = (p16 >> 5) & 0x7
    A = (p16 >> 2) & 0x7
    T = p16 & 0x3
    return S, R, V, A, T


def build_compact_char(char_data, cp):
    """Build compact char entry for final JSON.

    KEY: The JSON key (codepoint hex) IS the name.
    English Unicode name → aliases.en.name (it's just an alias).
    ○(x) == x → codepoint IS the node identity.
    """
    V_float = char_data.get("V", 0.5)
    A_float = char_data.get("A", 0.4)
    S = char_data.get("S", 0)
    R = char_data.get("R", 0)
    T = char_data.get("T", 0)

    V_q = quantize_v(V_float)
    A_q = quantize_a(A_float)
    P = pack_p(S, R, V_q, A_q, T)

    entry = {
        "P": P,
        "S": S,
        "R": R,
        "V": round(V_float, 3),
        "A": round(A_float, 3),
        "T": T,
        "cat": char_data.get("cat", ""),
    }

    # ── Aliases ──
    # Name = codepoint (the JSON key). Everything else is alias.
    # Unicode English name, CLDR tts/keywords → all go into aliases.
    aliases = {}

    # EN aliases: Unicode name + CLDR
    unicode_name = char_data.get("name", "")
    cldr = char_data.get("cldr", {})
    en_cldr = cldr.get("en", {})

    en_alias = {}
    if unicode_name:
        en_alias["name"] = unicode_name  # "FIRE", "SPACE", etc. = alias
    if en_cldr.get("tts"):
        en_alias["tts"] = en_cldr["tts"]
    if en_cldr.get("keywords"):
        en_alias["keywords"] = en_cldr["keywords"]
    if en_alias:
        aliases["en"] = en_alias

    # VI aliases: CLDR
    vi_cldr = cldr.get("vi", {})
    vi_alias = {}
    if vi_cldr.get("tts"):
        vi_alias["tts"] = vi_cldr["tts"]
    if vi_cldr.get("keywords"):
        vi_alias["keywords"] = vi_cldr["keywords"]
    if vi_alias:
        aliases["vi"] = vi_alias

    # Name aliases from NameAliases.txt
    name_aliases = char_data.get("name_aliases")
    if name_aliases:
        aliases["unicode_aliases"] = name_aliases

    if aliases:
        entry["aliases"] = aliases

    # Emoji info
    eg = char_data.get("emoji_group")
    if eg:
        entry["emoji_group"] = eg
    es = char_data.get("emoji_subgroup")
    if es:
        entry["emoji_subgroup"] = es

    # Flags (compact)
    flags = char_data.get("flags")
    if flags:
        entry["flags"] = flags

    # VA source (for debugging/verification)
    va_src = char_data.get("va_source")
    if va_src and va_src != "category_default":
        entry["va_src"] = va_src

    return entry


def main():
    print("Step 6: Pack P_weight (16 bits) + produce final JSON")
    print("=" * 60)

    # Load step5
    step5_path = os.path.join(OUTPUT_DIR, 'step5_srt.json')
    print(f"  Loading {step5_path}...")
    with open(step5_path, 'r', encoding='utf-8') as f:
        data = json.load(f)

    # Build final structure
    print(f"\n  Packing P_weight and building final JSON...")

    # ○{} tree metadata
    final = {
        "○": "UTF32-SDF-INTEGRATOR",
        "version": "18.0",
        "spec": "HomeOS_SPEC_v3",
        "structure": "○{plane{block{char}}}",
        "P_layout": "[S:4][R:4][V:3][A:3][T:2] = 16 bits",
        "axes": {
            "S": {"bits": 4, "name": "Shape", "values": data.get("sdf_primitives", {})},
            "R": {"bits": 4, "name": "Relation", "values": data.get("relation_types", {})},
            "V": {"bits": 3, "name": "Valence", "range": "[0.0, 1.0] → quantized [0,7]"},
            "A": {"bits": 3, "name": "Arousal", "range": "[0.0, 1.0] → quantized [0,7]"},
            "T": {"bits": 2, "name": "Time", "values": data.get("time_modes", {})},
        },
        "sources": {
            "UnicodeData": "Unicode 18.0 UnicodeData.txt",
            "Blocks": "Unicode 18.0 Blocks.txt",
            "PropList": "Unicode 18.0 PropList.txt",
            "emoji_data": "Unicode 18.0 emoji-data.txt",
            "emoji_test": "Unicode 18.0 emoji-test.txt",
            "NRC_VAD": "NRC-VAD-Lexicon v2.1 (Saif Mohammad, 2025)",
            "emoji_emotions": "Emoji Discrete Emotions Database (2025)",
            "CLDR": "CLDR 48.2 annotations (en, vi)",
        },
    }

    # Process planes
    planes = {}
    total_chars = 0
    total_packed = 0
    p_distribution = defaultdict(int)

    for pid, plane_data in data["planes"].items():
        plane_entry = {
            "name": plane_data["name"],
            "blocks": {},
        }

        plane_chars = 0
        for bname, block_data in plane_data["blocks"].items():
            block_entry = {
                "range": block_data["range"],
                "group": block_data["group"],
                "chars": {},
            }

            # Range entries (CJK, Hangul, etc.) → store as ranges, not individual chars
            if block_data.get("ranges"):
                block_entry["ranges"] = []
                for r in block_data["ranges"]:
                    # Compute default P for range
                    range_cat = r.get("cat", "Lo")
                    V_float = 0.50
                    A_float = 0.40
                    S = 0  # SPHERE default for CJK
                    R_val = 0  # IDENTITY
                    T = 0  # TIMELESS

                    V_q = quantize_v(V_float)
                    A_q = quantize_a(A_float)
                    P = pack_p(S, R_val, V_q, A_q, T)

                    block_entry["ranges"].append({
                        "start": r["start"],
                        "end": r["end"],
                        "name": r["name"],
                        "cat": range_cat,
                        "count": r["count"],
                        "P_default": P,
                    })
                    plane_chars += r["count"]

            # Individual chars
            for cp_hex, char_data in block_data.get("chars", {}).items():
                cp = int(cp_hex, 16)
                entry = build_compact_char(char_data, cp)
                block_entry["chars"][cp_hex] = entry
                plane_chars += 1
                total_packed += 1
                p_distribution[entry["P"]] += 1

            if block_entry["chars"] or block_entry.get("ranges"):
                plane_entry["blocks"][bname] = block_entry

        plane_entry["char_count"] = plane_chars
        total_chars += plane_chars
        planes[pid] = plane_entry

    final["planes"] = planes
    final["total_codepoints"] = total_chars
    final["total_packed"] = total_packed

    # Emoji sequences (multi-codepoint)
    if "emoji_sequences" in data:
        final["emoji_sequences"] = data["emoji_sequences"]
    if "cldr_sequences" in data:
        final["cldr_sequences"] = data["cldr_sequences"]

    # Summary stats
    print(f"\n  Total codepoints: {total_chars}")
    print(f"  Individually packed: {total_packed}")
    print(f"  Unique P values: {len(p_distribution)}")

    # Top P values
    print(f"\n  Top 10 P values:")
    for p_val, count in sorted(p_distribution.items(), key=lambda x: -x[1])[:10]:
        S, R, V, A, T = unpack_p(p_val)
        print(f"    P=0x{p_val:04X} ({p_val:016b}): S={S} R={R} V={V} A={A} T={T} → {count} chars")

    # Spot check
    print(f"\n  Spot check (final packed P):")
    checks = [
        (0x1F525, "🔥"), (0x1F60A, "😊"), (0x1F494, "💔"),
        (0x2208, "∈"), (0x222B, "∫"), (0x2190, "←"),
        (0x1D11E, "𝄞"), (0x0041, "A"), (0x2764, "❤"),
    ]
    for cp, ch in checks:
        plane_id = str(cp >> 16)
        if plane_id in final["planes"]:
            for bname, block in final["planes"][plane_id]["blocks"].items():
                cp_hex = f"{cp:04X}"
                if cp_hex in block.get("chars", {}):
                    cd = block["chars"][cp_hex]
                    P = cd["P"]
                    S, R, V, A, T = unpack_p(P)
                    v_str = f"V={cd['V']:.2f}→{V}"
                    a_str = f"A={cd['A']:.2f}→{A}"
                    en_name = cd.get("aliases", {}).get("en", {}).get("name", "")
                    en_tts = cd.get("aliases", {}).get("en", {}).get("tts", "")
                    vi_tts = cd.get("aliases", {}).get("vi", {}).get("tts", "")
                    print(f"    {ch} name={cp_hex} P=0x{P:04X}: S={S} R={R} {v_str} {a_str} T={T}")
                    print(f"       aliases: en.name=\"{en_name}\" en.tts=\"{en_tts}\" vi.tts=\"{vi_tts}\"")
                    break

    # Verification: unpack and repack should match
    print(f"\n  Verification: unpack→repack consistency...")
    errors = 0
    for pid, plane in final["planes"].items():
        for bname, block in plane["blocks"].items():
            for cp_hex, cd in block.get("chars", {}).items():
                P = cd["P"]
                S, R, V, A, T = unpack_p(P)
                P2 = pack_p(S, R, V, A, T)
                if P != P2:
                    errors += 1
    print(f"  → {errors} errors (should be 0)")

    # Write final JSON
    final_path = os.path.abspath(FINAL_OUTPUT)
    print(f"\n  Writing {final_path}...")
    with open(final_path, 'w', encoding='utf-8') as f:
        json.dump(final, f, ensure_ascii=False, indent=2)

    fsize = os.path.getsize(final_path)
    print(f"  → {fsize / 1024 / 1024:.1f} MB")

    # Also write compact version (no indent)
    compact_path = final_path.replace('.json', '_compact.json')
    print(f"  Writing compact {compact_path}...")
    with open(compact_path, 'w', encoding='utf-8') as f:
        json.dump(final, f, ensure_ascii=False, indent=None, separators=(',', ':'))
    csize = os.path.getsize(compact_path)
    print(f"  → {csize / 1024 / 1024:.1f} MB")

    # ── Write binary P table — v3.1 hierarchical format ──
    # Spec v3.1: KnowTree = tree L0→L1→L2→L3 (UDC only) + alias table (emoji/UTF-32)
    bin_path = os.path.join(os.path.dirname(final_path), 'udc_p_table.bin')
    print(f"\n  Writing binary P table (v3.1 hierarchical) {bin_path}...")
    import struct

    # ── 1. Classify: UDC blocks (58 blocks = KnowTree) vs alias (everything else) ──
    UDC_GROUPS = {"SDF", "MATH", "EMOTICON", "MUSICAL"}

    # Build hierarchical KnowTree
    # L0: 5 groups (4 UDC groups + RELATION mapped from MATH)
    # L1: 58 blocks within groups
    # L2: sub-ranges within blocks (chars grouped by semantic sub-type)
    # L3: individual UDC chars (9,584 leaf nodes)
    knowtree_l3 = []       # (cp, P, block_name, group)
    alias_entries = []     # (cp, P, closest_l3_index)

    for pid, plane in final["planes"].items():
        for bname, block in plane["blocks"].items():
            group = block.get("group", "")
            is_udc = group in UDC_GROUPS
            for cp_hex, cd in block.get("chars", {}).items():
                cp = int(cp_hex, 16)
                P = cd["P"]
                if is_udc:
                    knowtree_l3.append((cp, P, bname, group))
                else:
                    alias_entries.append((cp, P))

    knowtree_l3.sort()
    alias_entries.sort()

    # Build L1 (blocks) — aggregate P from L3 chars via LCA (mode/weighted avg)
    block_groups = defaultdict(list)  # block_name → [(cp, P)]
    for cp, P, bname, group in knowtree_l3:
        block_groups[(group, bname)].append((cp, P))

    # Build L0 (groups) — aggregate from L1
    group_blocks = defaultdict(list)  # group → [(bname, block_P)]

    knowtree_l1 = []  # (group, block_name, block_P, char_count)
    for (group, bname), chars in sorted(block_groups.items()):
        # LCA: mode of P values (≥60% same → use it, else weighted mean)
        p_values = [p for _, p in chars]
        from collections import Counter
        p_counts = Counter(p_values)
        most_common_p, most_common_count = p_counts.most_common(1)[0]
        if most_common_count / len(p_values) >= 0.6:
            block_P = most_common_p
        else:
            # Weighted average of S,R,V,A,T then repack
            avg_S = round(sum((p >> 12) & 0xF for p in p_values) / len(p_values))
            avg_R = round(sum((p >> 8) & 0xF for p in p_values) / len(p_values))
            avg_V = round(sum((p >> 5) & 0x7 for p in p_values) / len(p_values))
            avg_A = round(sum((p >> 2) & 0x7 for p in p_values) / len(p_values))
            avg_T = round(sum(p & 0x3 for p in p_values) / len(p_values))
            block_P = pack_p(avg_S, avg_R, avg_V, avg_A, avg_T)
        knowtree_l1.append((group, bname, block_P, len(chars)))
        group_blocks[group].append((bname, block_P))

    knowtree_l0 = []  # (group, group_P, block_count)
    for group, blocks in sorted(group_blocks.items()):
        p_values = [bp for _, bp in blocks]
        avg_S = round(sum((p >> 12) & 0xF for p in p_values) / len(p_values))
        avg_R = round(sum((p >> 8) & 0xF for p in p_values) / len(p_values))
        avg_V = round(sum((p >> 5) & 0x7 for p in p_values) / len(p_values))
        avg_A = round(sum((p >> 2) & 0x7 for p in p_values) / len(p_values))
        avg_T = round(sum(p & 0x3 for p in p_values) / len(p_values))
        group_P = pack_p(avg_S, avg_R, avg_V, avg_A, avg_T)
        knowtree_l0.append((group, group_P, len(blocks)))

    # ── 2. Build alias → L3 index mapping ──
    # For each alias, find nearest L3 UDC char by P_weight distance
    l3_sorted_p = [(cp, P) for cp, P, _, _ in knowtree_l3]
    l3_index = {cp: idx for idx, (cp, P, _, _) in enumerate(knowtree_l3)}

    alias_with_index = []
    for cp, P in alias_entries:
        # Simple: find L3 char with closest P (hamming on packed bits)
        best_idx = 0
        best_dist = 0xFFFF
        S_a, R_a, V_a, A_a, T_a = unpack_p(P)
        for idx, (l3_cp, l3_P) in enumerate(l3_sorted_p):
            S_b, R_b, V_b, A_b, T_b = unpack_p(l3_P)
            dist = abs(S_a - S_b) + abs(R_a - R_b) + abs(V_a - V_b) + abs(A_a - A_b) + abs(T_a - T_b)
            if dist < best_dist:
                best_dist = dist
                best_idx = idx
                if dist == 0:
                    break
        alias_with_index.append((cp, P, best_idx))

    # ── 3. Write binary ──
    # Format v3.1:
    #   Header:
    #     [magic:4B "KT31"]
    #     [l0_count:2B][l1_count:2B][l3_count:2B][alias_count:4B]
    #   L0 section: (group_name_len:1B, group_name:NB, P:2B, block_count:2B) × l0_count
    #   L1 section: (group_idx:1B, block_name_len:1B, block_name:NB, P:2B, char_count:2B) × l1_count
    #   L3 section: (cp:4B, P:2B) × l3_count
    #   Alias section: (cp:4B, P:2B, l3_index:2B) × alias_count

    with open(bin_path, 'wb') as f:
        # Header
        f.write(b'KT31')
        f.write(struct.pack('<HHH I',
            len(knowtree_l0), len(knowtree_l1), len(knowtree_l3), len(alias_with_index)))

        # L0 section
        for group, group_P, block_count in knowtree_l0:
            name_bytes = group.encode('utf-8')
            f.write(struct.pack('<B', len(name_bytes)))
            f.write(name_bytes)
            f.write(struct.pack('<HH', group_P, block_count))

        # L1 section
        group_names = [g for g, _, _ in knowtree_l0]
        for group, bname, block_P, char_count in knowtree_l1:
            group_idx = group_names.index(group) if group in group_names else 0
            name_bytes = bname.encode('utf-8')
            f.write(struct.pack('<BB', group_idx, len(name_bytes)))
            f.write(name_bytes)
            f.write(struct.pack('<HH', block_P, char_count))

        # L3 section (UDC chars only)
        for cp, P, _, _ in knowtree_l3:
            f.write(struct.pack('<IH', cp, P))

        # Alias section
        for cp, P, l3_idx in alias_with_index:
            f.write(struct.pack('<IHH', cp, P, l3_idx))

    bsize = os.path.getsize(bin_path)
    print(f"  → {bsize / 1024:.1f} KB")
    print(f"    KnowTree: L0={len(knowtree_l0)} groups, L1={len(knowtree_l1)} blocks, L3={len(knowtree_l3)} chars")
    print(f"    Alias table: {len(alias_with_index)} entries (emoji/UTF-32 → L3 UDC)")

    # Also write legacy flat format for backward compat
    legacy_bin_path = bin_path.replace('.bin', '_legacy.bin')
    print(f"\n  Writing legacy flat binary {legacy_bin_path}...")
    all_entries = [(cp, P) for cp, P, _, _ in knowtree_l3]
    all_entries.extend([(cp, P) for cp, P, _ in alias_with_index])
    all_entries.sort()
    with open(legacy_bin_path, 'wb') as f:
        f.write(struct.pack('<I', len(all_entries)))
        for cp, p in all_entries:
            f.write(struct.pack('<IH', cp, p))
    lsize = os.path.getsize(legacy_bin_path)
    print(f"  → {lsize / 1024:.0f} KB ({len(all_entries)} entries × 6B) [legacy compat]")

    print(f"\n{'='*60}")
    print(f"  FINAL OUTPUT (v3.1 hierarchical):")
    print(f"    JSON (pretty):  {final_path} ({fsize/1024/1024:.1f} MB)")
    print(f"    JSON (compact): {compact_path} ({csize/1024/1024:.1f} MB)")
    print(f"    Binary P table: {bin_path} ({bsize/1024:.1f} KB) [KT31 hierarchical]")
    print(f"    Legacy binary:  {legacy_bin_path} ({lsize/1024:.0f} KB) [flat compat]")
    print(f"    KnowTree: L0={len(knowtree_l0)} → L1={len(knowtree_l1)} → L3={len(knowtree_l3)} UDC chars")
    print(f"    Alias table: {len(alias_with_index)} emoji/UTF-32 → L3 UDC")
    print(f"    Total codepoints: {total_chars}")
    print(f"    Packed entries: {total_packed}")
    print(f"\nStep 6 DONE ✓ — ○{{UTF32-SDF-INTEGRATOR}} v18.0 / Spec v3.1 complete")


if __name__ == '__main__':
    main()
