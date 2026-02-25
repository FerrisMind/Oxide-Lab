# Task Plan

## Goal
Полностью устранить замечания из `CODE_REVIEW.md` (P0/P1/P2) и добавить недостающие тесты по критичным файлам.

## Phases
1. [completed] P0: убрать дубли listener-ов, устранить dual-state в Chat, оптимизировать history buffer.
2. [completed] P1: разбить Chat.svelte, устранить N+1 в HF search, убрать dynamic imports в hot paths, исправить fallback prompts.
3. [completed] P2: рефактор app-sidebar/session groups, llama-backend overview дубли, выровнять error handling, debounce resize, убрать dead STORAGE_KEY.
4. [completed] Тесты: listener, prompts, local-models, chat-history (базовые критичные сценарии).
5. [completed] Валидация: test/check целевых модулей и итоговый отчет по каждому пункту ревью.

## Risks
- В рабочем дереве есть сторонние изменения, не связанные с этим набором фиксов.

## Errors Encountered
- Ошибка `bind:messages` в `ChatMessages.svelte` (исправлено через `$bindable`).
- Ошибка `$sessions is not defined` в `groupSessionsByDate` (исправлено).
