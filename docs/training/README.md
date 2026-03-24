# Training Data for HomeOS

> **Built by Sora (空) — 2026-03-24**
> **Feed to Nox for integration into HomeOS intelligence layer**

## Files

| File | Entries | Source | Purpose |
|------|---------|--------|---------|
| `01_emotion_training.md` | 400 | [multilingual-sentiment-datasets](https://github.com/tyqiangz/multilingual-sentiment-datasets) (591K rows) | Dạy emotion pipeline nhận diện cảm xúc. 7 ngôn ngữ: EN/FR/ES/DE/AR/ZH/JA |
| `02_word_affect.md` | 112 | Curated | Word → sentiment mapping. Vietnamese (36 từ) + English (76 từ). Mở rộng `word_affect[]` |
| `03_world_knowledge.md` | 129 | Curated | Facts: VN geography/history, science, CS, world, food, literature, Origin |
| `04_dialog_patterns.md` | 20 | Curated | Response patterns: heal/learn/chat behaviors |
| `05_about_origin.md` | 18 | Curated | Meta-knowledge: Origin, Olang, HomeOS, AI sessions |

**Total: 661 entries**

## Format Rules

- 1 dòng = 1 fact hoặc 1 example
- Không dấu tiếng Việt (u16 encoding mất diacritics)
- Keyword-rich: mỗi câu >= 2 keywords (từ >= 3 ký tự)
- Max ~180 chars / dòng

## How to Use

```
# Trong HomeOS REPL:
learn_file data/05_about_origin.md     # dạy nó về chính nó
learn_file data/03_world_knowledge.md  # dạy kiến thức thế giới
learn_file data/01_emotion_training.md # dạy cảm xúc (128 facts max/session)
```

## Giới hạn hiện tại

- `__knowledge` max 128 facts / session → cần chia nhỏ
- `__silk` max 64 edges → sẽ saturate nhanh
- Không persistent → mất khi tắt binary
- Keyword exact match → cần fuzzy matching trong tương lai

## Nguồn gốc datasets

| Dataset | URL | Đánh giá |
|---------|-----|----------|
| multilingual-sentiment | github.com/tyqiangz/multilingual-sentiment-datasets | ⭐ BEST FIT — 591K rows, 3 classes, Asian langs |
| MultiLoKo | github.com/facebookresearch/multiloko | ⭐ 250 Q&A × 31 langs, JSONL, cần decrypt |
| Multi2WOZ | github.com/umanlp/Multi2WOZ | ◐ Task-oriented dialog, 5 langs |
| xMIND | github.com/andreeaiana/xMIND | ◐ 130K news × 14 langs, cần download lớn |
| Cross-Language-Dataset | github.com/FerreroJeremy/Cross-Language-Dataset | ○ Parallel corpus, ít giá trị trực tiếp |
