# REFERENCES — Academic Foundations of Origin/Nox

> Mục đích: Tổng hợp TẤT CẢ nguồn khoa học. Session sau KHÔNG cần tìm lại.
> Cập nhật: 2026-04-03

---

## 1. SEMANTIC HASHING & COMPACT ENCODING (Validate 16-bit Molecule)

| Paper | Venue/Year | Key Finding | Relevance |
|-------|-----------|-------------|-----------|
| Salakhutdinov & Hinton, "Semantic Hashing" | IJAR 2009 | 32-bit binary codes cho document retrieval, tốc độ O(1) | **Validate:** 16-bit compact encoding hoạt động cho semantic search |
| Tissier et al., "Near-lossless Binarization of Word Embeddings" | AAAI 2019 | Binary word embeddings chỉ mất ~2% accuracy, giảm 97% size | **Validate:** compact binary semantic codes preserve meaning |
| "Semantic Preserving Siamese Autoencoder for Binary Quantization" | ACM 2022 | Siamese autoencoder + Hamming-space projection | Learned binary codes preserve cosine similarity |
| "An Efficient and Robust Semantic Hashing Framework" | ACM TOIS 2023 | **16-bit hash codes**, 19MB storage cho large-scale text | **Trực tiếp validate** 16-bit molecule size |
| "A Survey on Deep Text Hashing" | arXiv:2510.27232, 2025 | Comprehensive survey, confirms viability of compact codes | State-of-art survey |
| Broder, "On the Resemblance and Containment of Documents" | SEQUENCES 1997 | MinHash — LSH cho duplicate detection | Foundation of locality-sensitive hashing |
| Charikar, "Similarity Estimation Techniques from Rounding Algorithms" | STOC 2002 | SimHash — random hyperplane projections | LSH cho cosine similarity |

**Tại sao quan trọng:** Origin's 16-bit molecule nằm TRONG range đã được nghiên cứu. Sự khác biệt: Origin dùng hand-crafted formulas (42) thay vì learned hashing, nhưng principle giống nhau. Thêm vào đó, Origin's effective dimensionality >> 16 bits nhờ: ordering (Zipf), Silk graph, KnowTree position.

---

## 2. EMOTION MODEL & NRC-VAD (V/A Dimensions)

| Paper | Venue/Year | Key Finding | Relevance |
|-------|-----------|-------------|-----------|
| Russell, J.A., "A Circumplex Model of Affect" | J. Personality & Social Psychology, 1980 | 2D emotion space: Valence × Arousal | **Foundation** cho V/A dimensions |
| Mehrabian & Russell, "An Approach to Environmental Psychology" | MIT Press, 1974 | PAD model (Pleasure-Arousal-Dominance) | 3D precursor, Origin dùng V+A (2/3 dims) |
| Mohammad, S., "Obtaining Reliable Human Ratings of V/A/D for 20,000 English Words" | ACL 2018 (aclanthology.org/P18-1017) | NRC-VAD Lexicon v1, best-worst scaling, **20,007 entries gốc** | **Data source** cho Origin: 19,971 entries (sau filter Vietnamese + dedup) |
| Mohammad, S., "NRC VAD Lexicon v2" | arXiv:2503.23547, March 2025 | **55,133 terms** (44,928 unigrams + 10,205 MWEs). 7-point Likert scale (-3 to +3), 9 ratings/term/dim | **UPGRADE available** — hiện Origin dùng v1 (19,971). LƯU Ý: v2 scale ≠ v1 (0-1), cần normalize |
| Mohammad, S., "Breaking Bad: Norms for V/A/D for 10k+ Multiword Expressions" | arXiv:2511.19816, 2025 | VAD cho multiword expressions | Extend sang phrase-level encoding |

**Tại sao quan trọng:** V/A là 2 trong 5 dimensions. NRC-VAD là data source chính. v2 (2025) có gần 3x entries — nên upgrade.

---

## 3. COGNITIVE ARCHITECTURE & NEURO-SYMBOLIC (Brain Pipeline)

| Paper | Venue/Year | Key Finding | Relevance |
|-------|-----------|-------------|-----------|
| Anderson, J.R. & Lebiere, C., "The Atomic Components of Thought" | LEA, 1998 | ACT-R base-level: B_i = ln(Σ t_j^(-d)), d=0.5. Full: A_i = B_i + Σ W_j × S_ji | **Closest model** cho Silk activation |
| Anderson, J.R., "How Can the Human Mind Occur in the Physical Universe?" | OUP, 2007 | ACT-R 6.0 architecture overview | Textbook reference |
| Petrov, A., "Computationally Efficient Approximation of Base-Level Learning" | ICCM, 2006 | Approx: B ≈ ln(n) - d×ln(L) (used in our actr_activation()) | Practical implementation |
| Pei Wang, NARS | 1995-present | Non-Axiomatic Reasoning: truth = <frequency, confidence> | Revision-based belief updating → QR maturity |
| "Neuro-Symbolic AI in 2024: A Systematic Review" | arXiv:2501.05435, Jan 2025 | 63% research on Learning & Inference, 44% Knowledge Rep | Origin sits at intersection |
| "Cognitive LLMs: Towards Integrating Cognitive Architectures and LLMs" | arXiv:2408.09176, 2024 | Combining ACT-R/Soar with LLMs | Validates hybrid approach |
| Wu et al., "Comparing LLMs for Prompt-Enhanced ACT-R and Soar" | AAAI Symposium, 2024 | Practical integration of cognitive arch + LLMs | Shows field is active |
| "A Review of Neuro-Symbolic AI" | ScienceDirect, 2025 | Advanced cognitive systems need symbolic + statistical | Confirms Origin's direction |

**Tại sao quan trọng:** Origin's brain pipeline (KnowTree + Silk + Pipeline + Instincts + Dream) là cognitive architecture. ACT-R/SOAR đã nghiên cứu 40+ năm. Neuro-symbolic là hướng hot nhất AI 2024-2025.

---

## 4. HEBBIAN LEARNING & NEURAL PLASTICITY (Silk)

| Paper | Venue/Year | Key Finding | Relevance |
|-------|-----------|-------------|-----------|
| Hebb, D.O., "The Organization of Behavior" | 1949 | "Fire together, wire together" | **Foundation** cho Silk learning |
| Oja, E., "Simplified Neuron Model as PCA" | J. Math Biology, 1982 | Normalized Hebbian rule (prevents divergence) | Silk weight normalization: Δw = η×x×y×(1-w²/w_max²) |
| Bienenstock, Cooper & Munro (BCM) | J. Neuroscience, 1982 | Sliding threshold homeostasis | Origin's adaptive QR threshold |
| Bi & Poo, "Synaptic Modifications: Dependence on Spike Timing" | J. Neuroscience, 1998 | STDP — empirical proof of directional learning | Directional Silk edges |

---

## 5. MEMORY DECAY & FORGETTING (Stretched Exponential)

| Paper | Venue/Year | Key Finding | Relevance |
|-------|-----------|-------------|-----------|
| Ebbinghaus, H., "Über das Gedächtnis" | 1885 | Forgetting curve: R(t) = e^(-t/S) | **Foundation** cho decay model |
| Wickelgren, W.A., "Single-trace Fragility Theory of Memory Dynamics" | Memory & Cognition, 1974, Vol.2, pp.775-780 | Power-law forgetting: m(t) = λ(1+βt)^(-ψ) where m=memory strength | Better empirical fit than pure exponential |
| Wixted & Ebbesen, "On the Form of Forgetting" | **Psychological Science**, Vol.2, pp.409-415, 1991 | Confirmed power-law across 6 paradigms (recall, recognition, pigeon matching) | Origin combines exp + power → stretched exponential β=φ⁻¹ |
| Kohlrausch, R. | Annalen der Physik, 1854 | Original stretched exponential: φ(t) = exp(-(t/τ)^β) | **Foundation** cho KWW function |
| Williams & Watts | Trans. Faraday Soc., 1970 | Applied KWW to dielectric relaxation | Name origin "KWW stretched exponential" |
| Phillips, J.C. | Reports on Progress in Physics, 1996 | β threshold at 1/2: different PDF behavior above/below | **Validates** β=0.618 > 0.5 boundary |
| Hasani, Lechner, Amini, Rus & Grosu | AAAI 2021 (arXiv:2006.04439, 2020) | Liquid Time-constant Networks: ODE with varying τ(t) coupled to hidden state. NOTE: closed-form solution is from SEPARATE paper: Nature Machine Intelligence 2022 (arXiv:2106.13898) | **Inspiration** cho Liquid Weights (MOL §15) |
| Murre & Dros, "Replication and Analysis of Ebbinghaus' Forgetting Curve" | PLOS ONE, 2015 (PMC4492928) | Memory Chain Model: 2 stores + consolidation | **Foundation** cho 3-layer decay (VM §5.6) |

**GHI CHÚ β range:** KWW β thường 0 < β < 1. Threshold β=1/2 có ý nghĩa toán học (Phillips 1996). β=0.618 nằm trong vùng "stretched" (>0.5, <1), phù hợp với heterogeneous memory model. Không có paper nào dùng đúng β=φ⁻¹ cho memory — đây là **design choice** của Origin, không phải empirical fit.

---

## 6. CONCEPTUAL SPACES & LINGUISTIC UNIVERSALS (5D Model)

| Paper | Venue/Year | Key Finding | Relevance |
|-------|-----------|-------------|-----------|
| Gardenfors, P., "Conceptual Spaces" | MIT Press, 2000 | Knowledge = regions in quality dimensions | **Theoretical framework** cho 5D molecular space |
| Berlin & Kay, "Basic Color Terms" | 1969 | Universal color taxonomy across all languages | Validates linguistic universals → S/R dimensions |
| Wierzbicka, A., "Semantics: Primes and Universals" | OUP, 1996 | 65 semantic primes across all languages | Basis cho R (Relation) dimension |
| Greenberg, J.H., "Universals of Language" | MIT Press, 1963 | 45 language universals | Validates dimensional approach to semantics |

**Tại sao quan trọng:** Origin's 5D model (SRVAT) KHÔNG tùy tiện. Nó grounded trong Conceptual Spaces framework + linguistic universals research. Gardenfors shows rằng quality dimensions + convex regions = viable knowledge representation.

---

## 7. SIGNED DISTANCE FIELDS (S Dimension)

| Source | Type | Key Contribution | Relevance |
|--------|------|-----------------|-----------|
| Inigo Quilez, "Distance Functions" | Blog/Tutorials, 2008+ | 18 SDF primitives + boolean ops + analytical gradients | **Direct source** cho 18 SDF trong Origin |
| Hart, "Sphere Tracing" | Visual Computer, 1996 | SDF raymarching algorithm | Foundation of SDF computing |
| Park et al., "DeepSDF" | CVPR 2019 | Neural SDF for 3D shape representation | Shows SDF captures categorical structure |
| Sitzmann et al., "MetaSDF" | NeurIPS 2020 | Meta-learning SDF → implicit class encoding | SDF network weights encode abstract categories |

**NOVEL trong Origin:** Applying SDF to SEMANTICS (not just geometry). Quilez SDF = shape only. Origin extends: Logic (R), Emotion (V/A), Time (T). **Không tìm thấy precedent** trong published literature. Đây là genuine novel contribution.

---

## 8. ARTIFICIAL CHEMISTRY (Molecular Metaphor)

| Paper | Venue/Year | Key Finding | Relevance |
|-------|-----------|-------------|-----------|
| Dittrich, Ziegler & Banzhaf, "Artificial Chemistries — A Review" | Artificial Life, 2001 | Molecular systems: molecules + reaction rules + algorithm | **Foundation** cho "molecule as knowledge unit" |
| "Chemical Reservoir Computation in Self-Organizing CRN" | **Nature**, 2024 | CRN performs classification + time-series prediction | Validates "molecule computes" principle |
| "Complex Chemical Reaction Networks for Future Information Processing" | Frontiers in Neuroscience, 2024 | CRNs: nonlinear, energy-efficient, parallelizable | Molecular approach = viable computing paradigm |
| "On Reaction Network Implementations of Neural Networks" | J. Royal Society Interface, 2021 | Maps neural nets onto chemical reaction networks | CRN proven efficiently Turing-universal |

**Tại sao quan trọng:** Origin's molecular metaphor (encode → compose → react → decay) không phải thơ — đây là Artificial Chemistry, một lĩnh vực nghiên cứu thật, có paper Nature 2024.

---

## 9. GOLDEN RATIO (φ⁻¹) IN AI

| Paper | Venue/Year | Key Finding | Relevance |
|-------|-----------|-------------|-----------|
| Jaeger, S. (NIH/NLM), "The Golden Ratio in Machine Learning" | IEEE AIPR, 2021 | φ derived from information theory as optimal learning rate | **Lý thuyết justify** cho φ⁻¹ decay + threshold |
| Jaeger, S., "The Golden Ratio of Learning and Momentum" | arXiv:2006.04751, 2020 | Double-Bayesian model → φ emerges as optimal | Validated on digit recognition |
| "Golden Ratio Organization in Human EEG" | Frontiers in Human Neuroscience, 2026 | Brain oscillation frequencies follow φ geometric series | Biological φ pattern in real neural data |
| "A Promising Approach Using Fibonacci Optimization" | Scientific Reports, 2023 | Fibonacci-based optimization algorithms | Nature paper (Scientific Reports) |

**Tại sao quan trọng:** Origin dùng φ⁻¹ = 0.618 cho decay, QR threshold, Dream trigger. Jaeger (NIH) chứng minh φ emerges from information theory — không phải numerology.

---

## 10. SELF-MODIFICATION & AI SAFETY

| Paper | Venue/Year | Key Finding | Relevance |
|-------|-----------|-------------|-----------|
| Lenat, D., "EURISKO: A Program That Learns New Heuristics" | Artificial Intelligence, 1983 | Self-modification dangerous without bounded safety | **Lesson learned:** Origin has L0 sealed + bounded repair (max 3 iter) |
| Friston, K., "The Free-Energy Principle" | Nature Reviews Neuroscience, 2010 | F = DKL + E[-ln P] ≥ surprise | Homeostasis model: minimize free energy |
| Friston et al., "Active Inference, Curiosity and Insight" | 2017 | Epistemic vs pragmatic value in action selection | Instinct framework basis |
| Parr, Pezzulo & Friston, "Active Inference" | MIT Press, 2022 | Complete theory: mind, brain, behavior | Textbook-level reference |

---

## 11. CHARACTER-LEVEL NLP & UNICODE (Encoding Foundation)

| Paper | Venue/Year | Key Finding | Relevance |
|-------|-----------|-------------|-----------|
| Kim et al., "Character-Aware Neural Language Models" | AAAI 2016 | Character-level CNNs for language modeling | Validates character-level semantic features |
| Bojanowski et al. (FastText), "Enriching Word Vectors with Subword Information" | TACL 2017 | Subword n-gram embeddings capture morphology | Subword = molecular chain analogy |
| "Emergent Semantics Beyond Token Embeddings: Frozen Visual Unicode Representations" | arXiv:2507.04886, 2025 | Unicode visual features as embeddings | Tangential: Unicode VISUAL properties carry semantic info |

**GHI CHU:** Dùng Unicode METADATA (Category, Name, Block) as semantic features là **unexplored** trong published literature. Origin's approach — parse Unicode character names cho keyword flags → encode formulas — không có precedent. Novel contribution.

---

## 12. OTHER ALGORITHMS USED

| Algorithm | Source | Usage in Origin |
|-----------|--------|----------------|
| VP-Tree | Yianilos, SODA 1993 (pp.311-321); also Uhlmann 1991 (independent) | Nearest neighbor search trong KnowTree |
| Cheney GC | Cheney, 1970 | Semi-space garbage collector cho VM |
| Fibonacci hashing | Knuth, TAOCP §6.4 | Dict hash function (φ⁻¹ multiplicative) |
| NaN boxing | LuaJIT, V8, SpiderMonkey | Value representation (8 types in 64 bits) |
| Pratt parser | Pratt, 1973 | Recursive descent for Olang |
| CLONALG | De Castro & Von Zuben, 2002 | Immune-inspired beam search |
| DBSCAN | Ester et al., 1996 | Clustering trong Dream cycle |
| Holographic Reduced Representations | Plate, 1995 | Future: 32×16-bit encoding nếu cần richer recombination |

---

## 13. DATA SOURCES

| Source | Entries | Used For |
|--------|---------|----------|
| UnicodeData.txt (Unicode Consortium) | ~33,000 codepoints | Character names, categories → 42 encode formulas |
| NRC-VAD-Lexicon v1 (Mohammad, ACL 2018) | 19,971 terms | V/A dimension initialization |
| NRC-VAD-Lexicon v2 (Mohammad, 2025) | **55,133 terms** | **UPGRADE AVAILABLE** |
| emoji-data.txt (CLDR) | 3,830 emoji | Emoji subgroup V/A defaults |
| json/udc.json (Lupin, 2026) | 8,284 curated entries | L0 anchor data |
| json/udc_utf32_compact.json | 41,338 aliases | UTF-32 → L0 mapping |

---

## 14. NOVEL CONTRIBUTIONS (Không có precedent trong literature)

1. **SDF applied to semantics** — extending geometric SDF to Logic, Emotion, Time domains
2. **Unicode metadata as semantic features** — parsing character Names for keyword classification
3. **Non-commutative molecular composition** — Zipf-weighted ordering preserves meaning in 16 bits
4. **9,200 Silk types from UDC characters** — relationship types EMERGE from 5D distance, not enum
5. **Stretched exponential decay β=φ⁻¹** — combining Ebbinghaus + Wickelgren + golden ratio
6. **Safety-as-architecture** (L0 sealed, bounded repair) — directly addresses Lenat 1983 failure
7. **Scale-invariant fractal encoding** — same compose operation at char/word/sentence/document level

---

*Document này là COMPLETE REFERENCE. Mọi academic source đã được verify qua internet search và project docs. Session sau đọc file này thay vì tìm lại.*
