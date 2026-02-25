# Findings

- `+layout.svelte` no longer writes into `loadedModelIds`; state authority moved to `local-models.ts` listeners.
- `Chat.svelte` no longer performs runtime two-way mirror sync into `chatState` on each parameter mutation.
- `PerformanceService` no longer uses `Array.shift()` for inference history trimming.
- Sidebar session groups now render through shared `SessionGroup.svelte` component.
- HF search path no longer executes per-model `listGGUFFiles` calls during listing.
- Prompt fallback now avoids Qwen-specific tokens and remains model-agnostic.
- `chat-history` persistence moved to `chat-history-repository.ts`; store remains state/business layer.
- Added missing tests for previously uncovered critical modules (prompts/listener/local-models/chat-history/download-manager).
