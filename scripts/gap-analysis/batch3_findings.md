# Batch 3 — Embeddings/Quant/Compression/Normalization

Total: 308 fns | DOC: 96 (31%) | INTERNAL: 177 (57%) | **USER_FACING_GAP: 25 (8%)** | UNCERTAIN: 10

## Critical USER_FACING gaps (25)

### Embedding providers (undocumented)
- `embedding/providers/bag_of_words.rs:29` `new()` + `:37` `build_vocabulary()` — BagOfWords provider absent from EMBEDDINGS.md
- `embedding/providers/char_ngram.rs:30` `new()` + `:39` `build_vocabulary()` — CharNGram absent
- `embedding/openai.rs:119` `new()`, `:140` `initialize()`, `:195` `available_models()` — **OpenAI provider entirely undocumented** (NEW finding, not in iter 1)
- `embedding/providers/minilm.rs:58` `load_model_with_id()` — HuggingFace model selection undocumented
- `embedding/providers/bert.rs:60` `load_model_with_id()` — same
- `embedding/fast_tokenizer.rs:72` `from_pretrained()` — tokenizer model selection undocumented

### Quantization
- `quantization/product.rs:17-37` `ProductQuantizationConfig.adaptive_assignment` flag (confirmed iter 1)
- `quantization/product.rs:52` `new()` PQ presets undocumented
- `quantization/product.rs:74` `train()` — training data requirements undocumented

### Compression — **NO DEDICATED DOC EXISTS**
- `compression/config.rs:16-124` builder + presets — undocumented
- `compression/zstd.rs:48,69,80` `new()/fast()/high_compression()` — undocumented
- `compression/lz4.rs:44,54,64` same — undocumented
- → **Action: create `docs/users/guides/COMPRESSION.md`**

### Normalization — **NO DEDICATED DOC EXISTS**
- `normalization/config.rs:47-109` `enabled()/conservative()/moderate()/aggressive()` presets — undocumented
- `normalization/detector.rs:97` `detect()` — content type detection (HTML, JSON, plaintext, PDF) undocumented
- → **Action: create `docs/users/guides/NORMALIZATION.md`**

## Coverage matrices

### Embedding providers
| Provider | Doc? |
|---|---|
| FastEmbed, BM25, TF-IDF, SVD, BERT, MiniLM | ✅ |
| **BagOfWords, CharNGram, OpenAI** | ❌ |

### Quantization modes
| Mode | Doc? |
|---|---|
| Scalar (16/8/4-bit), Binary | ✅ |
| Product (PQ) | ⚠️ partial — `adaptive_assignment` ausente |

### Compression
Nenhuma doc dedicada. 3 algoritmos × 3 presets = 9 knobs sem guia.

### Normalization
Nenhuma doc dedicada. 4 presets de policy + ContentType detection invisíveis.

## INTERNAL counts (não-gaps)
- Embeddings: 94 | Quantization: 62 | Compression: 15 | Normalization: 6
