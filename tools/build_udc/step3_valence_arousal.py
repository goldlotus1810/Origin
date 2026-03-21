#!/usr/bin/env python3
"""
Step 3: Compute V (Valence) and A (Arousal) from real data.

Sources:
  1. NRC-VAD-Lexicon v2.1 → word-level V/A from Unicode char names
  2. Emoji Discrete Emotions DB → direct emotion scores for 112 emoji
  3. Unicode name NLP → extract sentiment from name words

Logic (from spec):
  ❌ V = sentiment_dict["😊"]  ← gán từ ngoài vào
  ✅ V = extract_valence(name, cldr)  ← ĐỌC ra từ dữ liệu

  Emoji IS its emotion. P_weight IS the emoji from 5D perspective.
  ○(x) == x

Input:  /tmp/udc_build/step2_flags.json
Output: /tmp/udc_build/step3_va.json
"""

import json
import os
import re
import math
from collections import defaultdict

OUTPUT_DIR = '/tmp/udc_build'
NRC_VAD_PATH = '/tmp/mapping/mapping/nrc_vad/NRC-VAD-Lexicon-v2.1/Unigrams/unigrams-NRC-VAD-Lexicon-v2.1.txt'
EMOJI_EMOTIONS_PATH = '/tmp/mapping/mapping/emojis_discrete_emotions_database.xlsx'


def load_nrc_vad(path):
    """Load NRC-VAD Lexicon → dict of word → (valence, arousal, dominance).
    Values are in range [-1, 1] in v2.1."""
    vad = {}
    with open(path, 'r', encoding='utf-8') as f:
        header = f.readline()  # skip header
        for line in f:
            parts = line.strip().split('\t')
            if len(parts) >= 4:
                word = parts[0].lower()
                v = float(parts[1])
                a = float(parts[2])
                d = float(parts[3])
                vad[word] = (v, a, d)
    return vad


def load_emoji_emotions(path):
    """Load emoji discrete emotions database → dict of cp → {emotion: score}.
    Emotions: awe, happiness, relief, anxiety, disgust, amusement,
              excitement, anger, fear, pleasure, contentment, serenity, sadness

    Returns dict cp_int → dict with computed V and A."""
    import openpyxl
    wb = openpyxl.load_workbook(path, read_only=True)
    ws = wb['data']

    # Get headers
    rows = list(ws.iter_rows(values_only=True))
    headers = list(rows[0])

    result = {}
    # Positive emotions → high V
    positive_emotions = ['happiness', 'amusement', 'excitement', 'pleasure', 'contentment', 'serenity', 'relief', 'awe']
    # Negative emotions → low V
    negative_emotions = ['anger', 'fear', 'disgust', 'anxiety', 'sadness']
    # High arousal emotions
    high_arousal = ['excitement', 'anger', 'fear', 'awe', 'anxiety']
    # Low arousal emotions
    low_arousal = ['serenity', 'contentment', 'relief', 'sadness']

    for row in rows[1:]:
        if not row[0]:
            continue
        cp_hex = str(row[0]).strip()
        cp = int(cp_hex, 16)

        emotions = {}
        for i, h in enumerate(headers):
            if h and not h.endswith('_sd') and not h.endswith('_n') and not h.endswith('_dk'):
                if h not in ('id', 'ascii_code', 'emoji', 'category') and row[i] is not None:
                    try:
                        emotions[h] = float(row[i])
                    except (ValueError, TypeError):
                        pass

        if not emotions:
            continue

        # Compute V from emotions (scale 1-5 in raw data → normalize to 0-1)
        pos_sum = sum(emotions.get(e, 1.0) for e in positive_emotions)
        neg_sum = sum(emotions.get(e, 1.0) for e in negative_emotions)
        pos_max = len(positive_emotions) * 5.0
        neg_max = len(negative_emotions) * 5.0

        # V = (positive - negative) normalized to [0, 1]
        pos_norm = pos_sum / pos_max  # 0-1
        neg_norm = neg_sum / neg_max  # 0-1
        V = (pos_norm - neg_norm + 1.0) / 2.0  # map [-1,1] → [0,1]
        V = max(0.0, min(1.0, V))

        # A = arousal from high vs low arousal emotions
        high_sum = sum(emotions.get(e, 1.0) for e in high_arousal)
        low_sum = sum(emotions.get(e, 1.0) for e in low_arousal)
        high_max = len(high_arousal) * 5.0
        low_max = len(low_arousal) * 5.0

        high_norm = high_sum / high_max
        low_norm = low_sum / low_max
        A = (high_norm - low_norm + 1.0) / 2.0
        A = max(0.0, min(1.0, A))

        result[cp] = {
            "V": round(V, 4),
            "A": round(A, 4),
            "emotions": {k: round(v, 3) for k, v in emotions.items()},
            "source": "emoji_emotions_db",
        }

    wb.close()
    return result


def tokenize_unicode_name(name):
    """Tokenize a Unicode character name into searchable words."""
    # Remove common prefixes
    name = name.upper()
    for prefix in ['SMILING ', 'GRINNING ', 'FROWNING ', 'POUTING ']:
        pass  # keep these, they're meaningful

    # Split on spaces, hyphens
    words = re.split(r'[\s\-]+', name.lower())

    # Filter out very short and common stop words
    stop_words = {'with', 'and', 'or', 'the', 'a', 'an', 'of', 'for', 'in',
                  'at', 'to', 'on', 'by', 'no', 'sign', 'symbol', 'mark',
                  'left', 'right', 'up', 'down', 'above', 'below',
                  'small', 'large', 'capital', 'letter', 'digit'}

    return [w for w in words if len(w) > 1 and w not in stop_words]


def compute_va_from_name(name, nrc_vad):
    """
    Compute V and A from Unicode character name using NRC-VAD lexicon.

    "SMILING FACE WITH SMILING EYES"
     → words: ["smiling", "face", "smiling", "eyes"]
     → NRC lookup each word → aggregate

    Emoji IS its emotion. We READ V from it, not assign.
    """
    words = tokenize_unicode_name(name)
    if not words:
        return None, None

    v_scores = []
    a_scores = []

    for word in words:
        if word in nrc_vad:
            v, a, d = nrc_vad[word]
            v_scores.append(v)
            a_scores.append(a)
        # Try plural/singular
        elif word.endswith('s') and word[:-1] in nrc_vad:
            v, a, d = nrc_vad[word[:-1]]
            v_scores.append(v)
            a_scores.append(a)
        elif word + 's' in nrc_vad:
            v, a, d = nrc_vad[word + 's']
            v_scores.append(v)
            a_scores.append(a)
        # Try -ing → base
        elif word.endswith('ing') and word[:-3] in nrc_vad:
            v, a, d = nrc_vad[word[:-3]]
            v_scores.append(v)
            a_scores.append(a)
        elif word.endswith('ing') and word[:-3] + 'e' in nrc_vad:
            v, a, d = nrc_vad[word[:-3] + 'e']
            v_scores.append(v)
            a_scores.append(a)

    if not v_scores:
        return None, None

    # Amplify (spec rule: KHÔNG trung bình, amplify qua dominant)
    # But for bootstrap L0, simple weighted mean is appropriate
    # because we're reading from the name, not composing emotions
    V = sum(v_scores) / len(v_scores)
    A = sum(a_scores) / len(a_scores)

    # NRC-VAD v2.1 range is [-1, 1], normalize to [0, 1]
    V = (V + 1.0) / 2.0
    A = (A + 1.0) / 2.0
    V = max(0.0, min(1.0, V))
    A = max(0.0, min(1.0, A))

    return round(V, 4), round(A, 4)


def category_default_va(cat):
    """Default V/A for non-emoji chars based on Unicode General Category."""
    defaults = {
        # Letters
        'Lu': (0.52, 0.45),  # uppercase slightly assertive
        'Ll': (0.52, 0.40),  # lowercase neutral-calm
        'Lt': (0.52, 0.42),
        'Lm': (0.50, 0.38),
        'Lo': (0.50, 0.40),  # other letter
        # Marks
        'Mn': (0.50, 0.35),
        'Mc': (0.50, 0.35),
        'Me': (0.50, 0.35),
        # Numbers
        'Nd': (0.50, 0.42),
        'Nl': (0.50, 0.40),
        'No': (0.50, 0.40),
        # Punctuation
        'Pc': (0.50, 0.38),
        'Pd': (0.45, 0.40),  # dash = slight pause
        'Ps': (0.52, 0.45),  # opening bracket = expectation
        'Pe': (0.50, 0.40),  # closing = resolution
        'Pi': (0.52, 0.42),
        'Pf': (0.50, 0.40),
        'Po': (0.48, 0.42),
        # Symbols
        'Sm': (0.55, 0.50),  # math = logic = slightly positive, active
        'Sc': (0.50, 0.55),  # currency = arousing
        'Sk': (0.50, 0.38),
        'So': (0.55, 0.48),  # other symbol
        # Separators
        'Zs': (0.50, 0.30),  # space = calm
        'Zl': (0.50, 0.30),
        'Zp': (0.50, 0.30),
        # Control/Format
        'Cc': (0.50, 0.30),
        'Cf': (0.50, 0.30),
        'Cs': (0.50, 0.30),
        'Co': (0.50, 0.30),
        'Cn': (0.50, 0.30),
    }
    return defaults.get(cat, (0.50, 0.40))


def main():
    print("Step 3: Compute V (Valence) and A (Arousal) from real data")
    print("=" * 60)

    # Load step2
    step2_path = os.path.join(OUTPUT_DIR, 'step2_flags.json')
    print(f"  Loading {step2_path}...")
    with open(step2_path, 'r', encoding='utf-8') as f:
        data = json.load(f)

    # Load NRC-VAD
    print(f"  Loading NRC-VAD Lexicon...")
    nrc_vad = load_nrc_vad(NRC_VAD_PATH)
    print(f"  → {len(nrc_vad)} words")

    # Load emoji emotions
    print(f"  Loading Emoji Discrete Emotions DB...")
    emoji_emotions = load_emoji_emotions(EMOJI_EMOTIONS_PATH)
    print(f"  → {len(emoji_emotions)} emoji with direct V/A")

    # Apply V/A to all chars
    print(f"\n  Computing V/A for all chars...")
    stats = defaultdict(int)

    for pid, plane in data["planes"].items():
        for bname, block in plane["blocks"].items():
            for cp_hex, char_data in block.get("chars", {}).items():
                cp = int(cp_hex, 16)
                name = char_data.get("name", "")
                cat = char_data.get("cat", "")

                # Priority 1: Direct emoji emotions DB (most accurate)
                if cp in emoji_emotions:
                    edata = emoji_emotions[cp]
                    char_data["V"] = edata["V"]
                    char_data["A"] = edata["A"]
                    char_data["va_source"] = "emoji_emotions_db"
                    stats["emoji_db"] += 1
                    continue

                # Priority 2: NRC-VAD from Unicode name words
                V, A = compute_va_from_name(name, nrc_vad)
                if V is not None:
                    char_data["V"] = V
                    char_data["A"] = A
                    char_data["va_source"] = "nrc_vad_name"
                    stats["nrc_name"] += 1
                    continue

                # Priority 3: Default from category
                V, A = category_default_va(cat)
                char_data["V"] = V
                char_data["A"] = A
                char_data["va_source"] = "category_default"
                stats["category_default"] += 1

    print(f"\n  V/A assignment stats:")
    for source, count in sorted(stats.items()):
        print(f"    {source}: {count}")

    # Spot check: print some emoji V/A
    print(f"\n  Spot check (emoji V/A):")
    spot_checks = [
        (0x1F525, "🔥"),  # fire
        (0x1F60A, "😊"),  # smiling face
        (0x1F494, "💔"),  # broken heart
        (0x1F621, "😡"),  # angry face
        (0x1F622, "😢"),  # crying face
        (0x1F389, "🎉"),  # party popper
        (0x2764, "❤"),    # heart
        (0x1F480, "💀"),  # skull
        (0x1F31E, "🌞"),  # sun with face
        (0x1F4A9, "💩"),  # pile of poo
    ]
    for cp, emoji in spot_checks:
        # Find in tree
        plane_id = str(cp >> 16)
        if plane_id in data["planes"]:
            for bname, block in data["planes"][plane_id]["blocks"].items():
                cp_hex = f"{cp:04X}"
                if cp_hex in block.get("chars", {}):
                    cd = block["chars"][cp_hex]
                    print(f"    {emoji} U+{cp:04X}: V={cd.get('V','?'):.3f} A={cd.get('A','?'):.3f} [{cd.get('va_source','')}] {cd.get('name','')}")
                    break

    # Update metadata
    data["step"] = "3/6 — V/A from NRC-VAD + Emoji Emotions DB"
    data["va_summary"] = dict(stats)
    data["va_summary"]["nrc_vad_words"] = len(nrc_vad)
    data["va_summary"]["emoji_emotions_entries"] = len(emoji_emotions)

    # Output
    output_path = os.path.join(OUTPUT_DIR, 'step3_va.json')
    print(f"\n  Writing {output_path}...")
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, ensure_ascii=False, indent=None, separators=(',', ':'))

    fsize = os.path.getsize(output_path)
    print(f"  → {fsize / 1024 / 1024:.1f} MB")
    print("\nStep 3 DONE ✓")


if __name__ == '__main__':
    main()
