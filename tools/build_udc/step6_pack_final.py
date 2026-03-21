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

Structure:
  ○{plane{block{char}}}
  where each char has:
    - cp: codepoint (hex)
    - name: Unicode name (= codepoint, EN name is alias)
    - P: packed 16-bit P_weight
    - S, R, V, A, T: individual axis values
    - cat: Unicode General Category
    - cldr: {en: {tts, keywords}, vi: {tts, keywords}}
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
    """Build compact char entry for final JSON."""
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

    # Unicode name
    name = char_data.get("name", "")
    if name:
        entry["name"] = name

    # CLDR aliases (L0: en + vi only)
    cldr = char_data.get("cldr")
    if cldr:
        entry["cldr"] = cldr

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
        "spec": "HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2",
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
                    en_tts = cd.get("cldr", {}).get("en", {}).get("tts", "")
                    vi_tts = cd.get("cldr", {}).get("vi", {}).get("tts", "")
                    print(f"    {ch} U+{cp:04X} P=0x{P:04X}: S={S} R={R} {v_str} {a_str} T={T} | {en_tts} | {vi_tts}")
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

    # Write binary P table (just codepoint → 16-bit P, for embedded)
    bin_path = os.path.join(os.path.dirname(final_path), 'udc_p_table.bin')
    print(f"\n  Writing binary P table {bin_path}...")
    import struct
    # Format: [count:4][entries: (cp:4, P:2) × count]
    entries = []
    for pid, plane in final["planes"].items():
        for bname, block in plane["blocks"].items():
            for cp_hex, cd in block.get("chars", {}).items():
                cp = int(cp_hex, 16)
                entries.append((cp, cd["P"]))
    entries.sort()

    with open(bin_path, 'wb') as f:
        f.write(struct.pack('<I', len(entries)))
        for cp, p in entries:
            f.write(struct.pack('<IH', cp, p))
    bsize = os.path.getsize(bin_path)
    print(f"  → {bsize / 1024:.0f} KB ({len(entries)} entries × 6B)")

    print(f"\n{'='*60}")
    print(f"  FINAL OUTPUT:")
    print(f"    JSON (pretty):  {final_path} ({fsize/1024/1024:.1f} MB)")
    print(f"    JSON (compact): {compact_path} ({csize/1024/1024:.1f} MB)")
    print(f"    Binary P table: {bin_path} ({bsize/1024:.0f} KB)")
    print(f"    Total codepoints: {total_chars}")
    print(f"    Packed entries: {total_packed}")
    print(f"\nStep 6 DONE ✓ — ○{{UTF32-SDF-INTEGRATOR}} v18.0 complete")


if __name__ == '__main__':
    main()
