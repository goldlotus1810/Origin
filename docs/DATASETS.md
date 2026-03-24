# Datasets — Resources for HomeOS Intelligence

> Saved 2026-03-24. For future integration.

## Multilingual & Sentiment

| Dataset | URL | Use case |
|---------|-----|----------|
| multilingual-sentiment | https://github.com/tyqiangz/multilingual-sentiment-datasets | Expand word_affect table (Vietnamese + 50 languages) |
| xMIND | https://github.com/andreeaiana/xMIND | Multilingual news — knowledge ingestion |
| Cross-Language-Dataset | https://github.com/FerreroJeremy/Cross-Language-Dataset | Cross-language text pairs — translation/understanding |
| Multi2WOZ | https://github.com/umanlp/Multi2WOZ | Multilingual dialog — improve response patterns |
| multiloko | https://github.com/facebookresearch/multiloko | Multilingual evaluation benchmark |

## Priority

1. **multilingual-sentiment** → word_affect expansion (immediate value)
2. **Multi2WOZ** → dialog quality improvement
3. **xMIND** → news knowledge base
4. **multiloko** → benchmark evaluation
5. **Cross-Language-Dataset** → future multilingual support

## Integration path

```
Dataset → download CSV/JSON → convert to .md or .txt
→ learn_file <path> → knowledge store (512 facts max)
→ Or: parse into word_affect table entries (Olang code)
```
