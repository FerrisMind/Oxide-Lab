### План доработок (по приоритету)

1. Загрузчик моделей из HF Hub (safetensors и GGUF)

- Универсальная загрузка `tokenizer.json`, `config.json`, весов (`model.safetensors` или `model.safetensors.index.json`). Также возможность загрузки GGUF-файлов и выбора уровня квантизации.
- Использовать `VarBuilder::from_mmaped_safetensors`, выбор dtype: GPU → BF16/F16, CPU → F32.
- Кэширование/ревизии через `hf_hub::Api`.

- Зачем: единый способ подключения большинства официальных моделей без ручного скачивания; поддержка больших весов через индекс, mmap-экономия памяти.
- Где посмотреть:
  - candle-examples: `examples/llama/main.rs` (Api/Repo/get + VarBuilder), `src/lib.rs::hub_load_safetensors`.
  - Для GGUF: `examples/quantized-qwen3/main.rs` (`quantized_var_builder::VarBuilder::from_gguf`).
  - в проекте: для GGUF есть локальная загрузка Qwen3 (`src-tauri/src/models/qwen3/*`), универсального загрузчика (в т.ч. из HF Hub) нет.
- Статус: реализовано (добавлены команды загрузки из HF Hub для GGUF и подготовки safetensors: `LoadRequest::hub_gguf` и `LoadRequest::hub_safetensors`; UI расширен поддержкой ввода `repoId`/`revision`/`filename`).

2. Поддержка нескольких архитектур (safetensors и GGUF)

- Добавить LLaMA, Mistral, Gemma, Starcoder2 на базе `candle_transformers`.
- Единый интерфейс инференса и реестр моделей (рядом с текущим Qwen3/GGUF).
- Опция выключения KV-cache, совместимость с семплингом.

- Зачем: охват популярных семейств моделей и совместимость промтов/инфраструктуры.
- Где посмотреть:
  - candle-examples: `examples/llama/main.rs`, `examples/mistral/main.rs`, `examples/gemma/main.rs`, `examples/starcoder2/main.rs`.
  - в проекте: `src-tauri/src/models/qwen3/*` (только Qwen3/GGUF), `src-tauri/src/models/registry.rs` (заглушка архов).
  - Для GGUF: аналогичный путь через `quantized_var_builder::VarBuilder::from_gguf` или собственные GGUF-ридеры по спеке.
- Статус: частично (только Qwen3/GGUF).

3. Автовыбор устройства и dtype

- Авто-детект CUDA/Metal как в `candle_examples::device` с fallback на CPU.
- По умолчанию использовать доступный GPU; переопределение через UI.

- Зачем: лучшая производительность из коробки и корректный выбор числового формата.
- Где посмотреть:
  - candle-examples: `src/lib.rs::device`, выбор dtype в `examples/*/main.rs` (GPU → BF16/F16, CPU → F32).
  - в проекте: `src-tauri/src/core/device.rs` (по умолчанию CPU, без авто BF16/F16 логики).
- Для GGUF: тот же авто-выбор устройства; учитывать ограничения конкретных квантованных весов (если ядро недоступно — падать на CPU).
- Статус: РЕАЛИЗОВАНО ✓ (улучшенная версия с корректной runtime-детекцией CUDA/Metal)

4. Тогглы производительности

- Поддержка `flash-attn` (когда применимо) и `force_dmmv` (CUDA kernel) как опции сборки/рантайма.
- Переключатели в UI.

- Зачем: ускорение attention, контроль CUDA-ядра для стабильности/скорости.
- Где посмотреть:
  - candle-examples: `examples/mistral/main.rs` (`use_flash_attn`, `force_dmmv`), `examples/gemma/main.rs` (`use_flash_attn`), `examples/llama/main.rs` (флаг в конфиге модели).
  - в проекте: отсутствуют.
- Для GGUF: аналогично нужна поддержка работы с flash attention v2; `force_dmmv` возможно релевантен для некоторых CUDA-кернелов, если поддерживаются).
- Статус: не реализовано.

5. Автоформатирование по chat_template

- Извлекать `chat_template` из токенизатора/метаданных и автоматически форматировать prompt.
- Резервные эвристики для известных моделей (если шаблон отсутствует).

- Зачем: корректное форматирование ролей/токенов диалога для chat-моделей без ручной разметки.
- Где посмотреть:
  - candle-examples: `examples/gemma/main.rs` (спец-токены `<start_of_turn>`, `<end_of_turn>`), `examples/quantized-qwen3/main.rs` (шаблон с `
