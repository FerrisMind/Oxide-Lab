/**
 * Chat Controller Actions
 * 
 * Implements all chat-related actions using the EngineManager architecture.
 * Engines are started/stopped via Tauri commands, generation uses OpenAI-compatible API.
 */

import type { ChatControllerCtx } from './types';
import { createStreamListener } from './listener';
import { get } from 'svelte/store';
import { t } from '$lib/i18n';
import { chatState } from '$lib/stores/chat';

export function createActions(ctx: ChatControllerCtx) {
    const stream = createStreamListener(ctx);

    // Track which engine is currently active
    let activeEngineId: string | null = null;

    // ─── Engine Management ────────────────────────────────────────

    /**
     * List available engines from engines.json
     */
    async function listEngines(): Promise<Array<{ id: string; name: string; description?: string }>> {
        try {
            const { invoke } = await import('@tauri-apps/api/core');
            return await invoke('list_engines');
        } catch (e) {
            console.warn('[engines] Failed to list engines:', e);
            return [];
        }
    }

    /**
     * Start an engine with a given model path.
     * Replaces the old loadGGUF / load_model flow.
     */
    async function startEngine(engineId?: string, modelPath?: string) {
        ctx.isLoadingModel = true;
        ctx.loadingProgress = 0;
        ctx.loadingStage = 'start';
        ctx.busy = true;
        ctx.isLoaded = false;
        ctx.errorText = '';

        try {
            await stream.ensureListener();
            await ensureEngineStatusListener();

            const { invoke } = await import('@tauri-apps/api/core');

            // If no engine specified, pick first available
            if (!engineId) {
                const engines = await listEngines();
                if (engines.length === 0) {
                    throw new Error('No engines configured. Check engines.json.');
                }
                engineId = engines[0].id;
            }

            const path = modelPath || ctx.modelPath;
            if (!path) {
                const { message } = await import('@tauri-apps/plugin-dialog');
                await message(get(t)('chat.errors.loadModelFirst'), {
                    title: get(t)('chat.loading.title') || 'Engine',
                    kind: 'warning',
                });
                return;
            }

            console.log('[engine] Starting:', engineId, 'with model:', path);

            ctx.loadingStage = 'starting_engine';
            ctx.loadingProgress = 30;

            await invoke('start_engine', {
                engineId,
                modelPath: path,
            });

            activeEngineId = engineId;
            ctx.isLoaded = true;
            ctx.loadingProgress = 100;
            chatState.update(s => ({
                ...s,
                isLoaded: true,
                isLoadingModel: false,
                busy: false,
                loadingProgress: 100,
            }));

            console.log('[engine] Started successfully:', engineId);
        } catch (e) {
            const err = String(e ?? 'Unknown error');
            ctx.errorText = err;
            try {
                const { message } = await import('@tauri-apps/plugin-dialog');
                await message(err, {
                    title: get(t)('chat.errors.loadFailed'),
                    kind: 'error',
                });
            } catch { /* ignore */ }
        } finally {
            ctx.isLoadingModel = false;
            ctx.busy = false;
        }
    }

    /**
     * Stop the active engine. Replaces unloadGGUF.
     */
    async function stopEngine() {
        if (!activeEngineId || ctx.busy) return;

        ctx.isUnloadingModel = true;
        ctx.unloadingProgress = 0;
        ctx.busy = true;
        ctx.errorText = '';

        try {
            const { invoke } = await import('@tauri-apps/api/core');
            ctx.unloadingProgress = 50;

            await invoke('stop_engine', { engineId: activeEngineId });

            ctx.unloadingProgress = 100;
            await new Promise((r) => setTimeout(r, 300));

            ctx.isLoaded = false;
            ctx.messages = [];
            activeEngineId = null;

            const successText = get(t)('chat.loading.unloadSuccess');
            ctx.errorText = successText;
            setTimeout(() => {
                if (ctx.errorText === successText) ctx.errorText = '';
            }, 3000);
        } catch (e) {
            ctx.errorText = String(e ?? 'Unknown error');
        } finally {
            ctx.isUnloadingModel = false;
            ctx.unloadingProgress = 0;
            ctx.busy = false;
        }
    }

    // ─── Engine Status Listener ──────────────────────────────────

    let engineStatusUnlisten: (() => void) | null = null;

    async function ensureEngineStatusListener() {
        if (engineStatusUnlisten) return;
        try {
            const { listen } = await import('@tauri-apps/api/event');
            engineStatusUnlisten = await listen<{
                engine_id: string;
                status: string;
                error?: string;
            }>('engine_status', (e) => {
                const p = e.payload;
                console.log('[engine_status]', p);

                if (p.status === 'ready') {
                    ctx.isLoaded = true;
                    ctx.isLoadingModel = false;
                    ctx.loadingProgress = 100;
                }
                if (p.status === 'error') {
                    ctx.errorText = p.error || 'Engine error';
                    ctx.isLoadingModel = false;
                }
                if (p.status === 'stopped') {
                    ctx.isLoaded = false;
                }
            });
        } catch (err) {
            console.warn('Failed to attach engine_status listener:', err);
        }
    }

    // ─── Chat / Generation ───────────────────────────────────────

    async function handleSend() {
        const text = ctx.prompt.trim();
        if (!text || ctx.busy) return;

        const storeState = get(chatState);
        const isModelLoaded = ctx.isLoaded || storeState.isLoaded;
        if (!isModelLoaded || !activeEngineId) {
            const { message } = await import('@tauri-apps/plugin-dialog');
            await message(get(t)('chat.errors.loadModelFirst'), {
                title: get(t)('chat.errors.modelNotLoaded'),
                kind: 'warning',
            });
            return;
        }

        // Add user message to database
        const { chatHistory } = await import('$lib/stores/chat-history');
        const state = get(chatHistory);
        if (!state.currentSessionId) {
            await chatHistory.createSession(ctx.modelPath, ctx.repoId);
        }

        await chatHistory.addMessage({ role: 'user', content: text });
        await chatHistory.addMessage({ role: 'assistant', content: '', thinking: '' });

        const msgs = ctx.messages;
        msgs.push({ role: 'user', content: text });
        msgs.push({ role: 'assistant', content: '', thinking: '', isThinking: false });
        ctx.messages = msgs;

        ctx.prompt = '';
        await generateFromHistory();
    }

    /**
     * Build OpenAI-compatible messages array from chat history.
     */
    function buildOpenAIMessages(history: Array<{ role: string; content: string }>) {
        return history
            .filter(m => m.content.trim() !== '')
            .map(m => ({ role: m.role, content: m.content }));
    }

    /**
     * Build request with optional sampling parameters.
     */
    function buildGenerateRequest(messages: Array<{ role: string; content: string }>) {
        const req: Record<string, unknown> = {
            model: 'default',
            messages,
            stream: true,
        };

        if (ctx.use_custom_params) {
            if (ctx.temperature_enabled) req.temperature = ctx.temperature;
            if (ctx.top_p_enabled && ctx.top_p_value > 0 && ctx.top_p_value <= 1) {
                req.top_p = ctx.top_p_value;
            }
            if (ctx.top_k_enabled) {
                // top_k is not standard OpenAI but many engines support it
                req.top_k = Math.max(1, Math.floor(ctx.top_k_value));
            }
            if (ctx.repeat_penalty_enabled) {
                req.repetition_penalty = ctx.repeat_penalty_value;
            }
        }

        return req;
    }

    async function generateFromHistory() {
        ctx.busy = true;
        chatState.update(s => ({ ...s, busy: true }));

        try {
            await stream.ensureListener();

            const msgs = ctx.messages;
            const hist = msgs[msgs.length - 1]?.role === 'assistant' && msgs[msgs.length - 1]?.content === ''
                ? msgs.slice(0, -1)
                : msgs.slice();

            const openaiMessages = buildOpenAIMessages(hist);
            const req = buildGenerateRequest(openaiMessages);

            console.log('[generate] Sending to engine:', activeEngineId, 'messages:', openaiMessages.length);

            const { invoke } = await import('@tauri-apps/api/core');
            await invoke('generate', {
                engineId: activeEngineId,
                req,
            });
        } catch (e) {
            const err = String(e ?? 'Unknown error');
            const msgs = ctx.messages;
            const last = msgs[msgs.length - 1];
            if (last && last.role === 'assistant' && last.content === '') {
                last.content = `${get(t)('chat.errors.generationFailed')}: ${err}`;
                ctx.messages = msgs;
            }
            try {
                const { message } = await import('@tauri-apps/plugin-dialog');
                await message(err, { title: get(t)('chat.errors.generationFailed'), kind: 'error' });
            } catch { /* ignore */ }
        } finally {
            ctx.busy = false;
            chatState.update(s => ({ ...s, busy: false }));
        }
    }

    async function stopGenerate() {
        if (!activeEngineId) return;
        console.log('[stopGenerate] called for engine:', activeEngineId);
        try {
            const { invoke } = await import('@tauri-apps/api/core');
            await invoke('cancel_generation', { engineId: activeEngineId });

            // Save partial generation
            const { chatHistory } = await import('$lib/stores/chat-history');
            const state = get(chatHistory);
            if (state.currentSessionId) {
                const msgs = ctx.messages;
                const last = msgs[msgs.length - 1];
                if (last && last.role === 'assistant' && last.content) {
                    await chatHistory.saveAssistantMessage(state.currentSessionId, last.content);
                }
            }
        } catch (err) {
            console.error('[stopGenerate] error:', err);
        }
    }

    // ─── Edit / Regenerate ───────────────────────────────────────

    async function handleEdit(editIndex: number, newContent: string) {
        if (ctx.busy) return;

        const storeState = get(chatState);
        const isModelLoaded = ctx.isLoaded || storeState.isLoaded;
        if (!isModelLoaded || !activeEngineId) {
            const { message } = await import('@tauri-apps/plugin-dialog');
            await message(get(t)('chat.errors.loadModelFirst'), {
                title: get(t)('chat.errors.modelNotLoaded'),
                kind: 'warning',
            });
            return;
        }

        const { chatHistory } = await import('$lib/stores/chat-history');
        const historyState = get(chatHistory);

        const msgs = ctx.messages.slice(0, editIndex + 1);
        if (msgs[editIndex]) {
            msgs[editIndex].content = newContent;
        }

        if (historyState.currentSessionId) {
            await chatHistory.truncateMessages(historyState.currentSessionId, editIndex + 1);
            await chatHistory.updateLastMessage(historyState.currentSessionId, newContent);
            await chatHistory.addMessage({ role: 'assistant', content: '', thinking: '' });
        }

        msgs.push({ role: 'assistant', content: '', html: '', thinking: '', isThinking: false });
        ctx.messages = msgs;

        await generateFromHistory();
    }

    async function handleRegenerate(messageIndex: number) {
        if (ctx.busy) return;

        const storeState = get(chatState);
        const isModelLoaded = ctx.isLoaded || storeState.isLoaded;
        if (!isModelLoaded || !activeEngineId) {
            const { message } = await import('@tauri-apps/plugin-dialog');
            await message(get(t)('chat.errors.loadModelFirst'), {
                title: get(t)('chat.errors.modelNotLoaded'),
                kind: 'warning',
            });
            return;
        }

        let userIndex = messageIndex;
        if (ctx.messages[messageIndex]?.role === 'assistant') {
            userIndex = messageIndex - 1;
        }

        if (userIndex < 0 || ctx.messages[userIndex]?.role !== 'user') {
            console.warn('[regenerate] Could not find user message to regenerate from');
            return;
        }

        const { chatHistory } = await import('$lib/stores/chat-history');
        const historyState = get(chatHistory);

        const msgs = ctx.messages.slice(0, userIndex + 1);

        if (historyState.currentSessionId) {
            await chatHistory.truncateMessages(historyState.currentSessionId, userIndex + 1);
            await chatHistory.addMessage({ role: 'assistant', content: '', thinking: '' });
        }

        msgs.push({ role: 'assistant', content: '', html: '', thinking: '', isThinking: false });
        ctx.messages = msgs;

        await generateFromHistory();
    }

    // ─── File Picker ─────────────────────────────────────────────

    async function pickModel() {
        const { open, message } = await import('@tauri-apps/plugin-dialog');

        const selected = await open({
            multiple: false,
            filters: [{ name: 'Model Files', extensions: ['gguf'] }],
        });
        if (typeof selected === 'string') ctx.modelPath = selected;
    }

    // ─── Lifecycle ───────────────────────────────────────────────

    function destroy() {
        stream.destroy();
        if (engineStatusUnlisten) {
            try { engineStatusUnlisten(); } catch { /* ignore */ }
            engineStatusUnlisten = null;
        }
    }

    return {
        // Engine management (new API)
        listEngines,
        startEngine,
        stopEngine,
        // Chat
        handleSend,
        handleEdit,
        handleRegenerate,
        generateFromHistory,
        stopGenerate,
        // UI
        pickModel,
        destroy,
        ensureStreamListener: stream.ensureListener,
        // Legacy aliases for backward compat
        loadGGUF: startEngine,
        unloadGGUF: stopEngine,
        cancelLoading: () => { ctx.isCancelling = true; },
        refreshDeviceInfo: async () => { },
        setDeviceByToggle: async (_desired?: boolean) => { },
        handleAttachFile: async () => { },
    };
}
