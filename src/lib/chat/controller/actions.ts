import { invoke } from '@tauri-apps/api/core';
// Экспортируем invoke/пробу CUDA в глобальную область для отладки из DevTools
try {
  (globalThis as any).__invoke = invoke;

  (globalThis as any).__probeCuda = () => invoke('probe_cuda');
} catch {}
import { open, message } from '@tauri-apps/plugin-dialog';
import type { ChatControllerCtx } from './types';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { createStreamListener } from './listener';
import { buildPromptWithChatTemplate } from '$lib/chat/prompts';

export function createActions(ctx: ChatControllerCtx) {
  const stream = createStreamListener(ctx);

  // Реальный прогресс загрузки из Rust
  type LoadProgressEvent = {
    stage: string;
    progress: number;
    message?: string;
    done?: boolean;
    error?: string;
  };
  let loadUnlisten: UnlistenFn | null = null;
  async function ensureLoadProgressListener() {
    if (loadUnlisten) return;
    try {
      loadUnlisten = await listen<LoadProgressEvent>('load_progress', async (e) => {
        const p = e.payload || ({} as any);
        if (typeof p.progress === 'number')
          ctx.loadingProgress = Math.max(0, Math.min(100, Math.floor(p.progress)));
        if (typeof p.stage === 'string') ctx.loadingStage = p.stage;
        if (p.message) ctx.errorText = '';
        if (p.error) ctx.errorText = String(p.error);

        // Если это стартовые стадии — гарантируем, что индикаторы включены
        if (p.stage === 'start') {
          ctx.isLoadingModel = true;
          ctx.busy = true;
          ctx.isLoaded = false;
        }
        if (p.done) {
          ctx.isLoadingModel = false;
          ctx.busy = false;
          if (!p.error && ctx.loadingProgress >= 100) ctx.isLoaded = true;
        }
      });
      // Доп. канал: ранний сигнал о модальностями из backend
      await listen<any>('modality_support', (e) => {
        const m = e.payload || {};
        if (typeof m.text === 'boolean') ctx.supports_text = m.text;
        if (typeof m.image === 'boolean') ctx.supports_image = m.image;
        if (typeof m.audio === 'boolean') ctx.supports_audio = m.audio;
        if (typeof m.video === 'boolean') ctx.supports_video = m.video;
      });
    } catch (err) {
      console.warn('failed to attach load_progress listener', err);
    }
  }

  // Обработчик вложений: НЕ запускает генерацию автоматически.
  // Добавляет содержимое вложения в поле ввода (prompt), чтобы пользователь сам отправил сообщение.
  async function _handleAttachFile(_payload: { filename: string; content: string }) {
    try {
      // Убираем добавление информации о прикрепленных файлах в поле ввода
      // const prefix = `\n[Прикреплён файл: ${payload.filename}]\n\n`;
      // const addition = `${prefix}${payload.content}`;
      // const has = ctx.prompt && ctx.prompt.trim().length > 0;
      // ctx.prompt = has ? `${ctx.prompt}\n\n${addition}` : addition;

      // Просто игнорируем прикрепленные файлы в поле ввода
      return;
    } catch (e) {
      console.error('Attach handler failed', e);
    }
  }

  async function refreshDeviceInfo() {
    try {
      const info = await invoke<any>('get_device_info');
      ctx.cuda_build = !!info?.cuda_build;
      ctx.cuda_available = !!info?.cuda_available;
      ctx.current_device = String(info?.current ?? 'CPU');
      // CPU по умолчанию; если включён тумблер и CUDA доступна — активируем GPU
      ctx.use_gpu = ctx.cuda_available && ctx.current_device === 'CUDA';
    } catch {}
  }

  async function setDeviceByToggle(desired?: boolean) {
    try {
      if (typeof desired !== 'undefined') {
        ctx.use_gpu = !!desired;
      }
      if (ctx.use_gpu) {
        // Пытаемся переключиться на CUDA, даже если предварительная проверка says false,
        // чтобы получить конкретную ошибку из backend (диагностика проблем PATH/DLL)
        await invoke('set_device', { pref: { kind: 'cuda', index: 0 } });
      } else {
        await invoke('set_device', { pref: { kind: 'cpu' } });
      }
      await refreshDeviceInfo();
    } catch (e) {
      console.warn('[device] toggle switch failed', e);
    }
  }

  // Инициализируем информацию об устройстве при старте
  void refreshDeviceInfo();

  function cancelLoading() {
    ctx.isCancelling = true;
    ctx.loadingStage = 'cancelling';
    // Сигнал на бэкенд — реальная отмена загрузки
    try {
      void invoke('cancel_model_loading');
    } catch {}
  }

  async function loadGGUF() {
    ctx.isLoadingModel = true;
    ctx.loadingProgress = 0;
    ctx.loadingStage = 'start';
    ctx.busy = true;
    ctx.isLoaded = false;
    ctx.errorText = '';
    try {
      await ensureLoadProgressListener();
      await stream.ensureListener();
      const context_length = Math.max(1, Math.floor(ctx.ctx_limit_value));
      console.log('[load] frontend params', { context_length, format: ctx.format });
      if (ctx.isCancelling) return;
      // CPU по умолчанию. Если пользователь явно включил GPU и он доступен — переключим на CUDA после загрузки.
      if (ctx.format === 'gguf') {
        if (!ctx.modelPath) {
          await message('Укажите путь к .gguf', { title: 'Загрузка модели', kind: 'warning' });
          return;
        }
        await invoke('load_model', {
          req: {
            format: 'gguf',
            model_path: ctx.modelPath,
            tokenizer_path: null,
            context_length,
            device: ctx.use_gpu ? { kind: 'cuda', index: 0 } : { kind: 'cpu' },
          },
        });
      } else if (ctx.format === 'hub_gguf') {
        if (!ctx.repoId || !ctx.hubGgufFilename) {
          await message('Укажите repoId и имя файла .gguf', {
            title: 'Загрузка из HF Hub',
            kind: 'warning',
          });
          return;
        }
        await invoke('load_model', {
          req: {
            format: 'hub_gguf',
            repo_id: ctx.repoId,
            revision: ctx.revision || null,
            filename: ctx.hubGgufFilename,
            context_length,
            device: ctx.use_gpu ? { kind: 'cuda', index: 0 } : { kind: 'cpu' },
          },
        });
      } else if (ctx.format === 'hub_safetensors') {
        if (!ctx.repoId) {
          await message('Укажите repoId (owner/repo)', {
            title: 'Загрузка из HF Hub',
            kind: 'warning',
          });
          return;
        }
        await invoke('load_model', {
          req: {
            format: 'hub_safetensors',
            repo_id: ctx.repoId,
            revision: ctx.revision || null,
            context_length,
            device: ctx.use_gpu ? { kind: 'cuda', index: 0 } : { kind: 'cpu' },
          },
        });
      } else if (ctx.format === 'local_safetensors') {
        if (!ctx.modelPath) {
          await message('Укажите директорию с моделью safetensors', {
            title: 'Локальная модель',
            kind: 'warning',
          });
          return;
        }
        await invoke('load_model', {
          req: {
            format: 'local_safetensors',
            model_path: ctx.modelPath,
            context_length,
            device: ctx.use_gpu ? { kind: 'cuda', index: 0 } : { kind: 'cpu' },
          },
        });
      }
      await refreshDeviceInfo();
      if (ctx.isCancelling) return;
    } catch (e) {
      const err = String(e ?? 'Unknown error');
      ctx.errorText = err;
      try {
        await message(err, { title: 'Ошибка загрузки модели', kind: 'error' });
      } catch {}
    } finally {
      // Управление состоянием происходит через события load_progress
    }
  }

  async function unloadGGUF() {
    if (ctx.busy || !ctx.isLoaded) return;
    ctx.isUnloadingModel = true;
    ctx.unloadingProgress = 0;
    ctx.busy = true;
    ctx.errorText = '';
    try {
      const unloadInterval = setInterval(() => {
        if (ctx.unloadingProgress < 80) ctx.unloadingProgress += Math.random() * 15 + 5;
      }, 100);
      await invoke('unload_model');
      ctx.unloadingProgress = 100;
      clearInterval(unloadInterval);
      await new Promise((r) => setTimeout(r, 300));
      ctx.isLoaded = false;
      ctx.messages = [];
      ctx.errorText = 'Модель и токенизатор успешно выгружены из памяти';
      setTimeout(() => {
        if (ctx.errorText === 'Модель и токенизатор успешно выгружены из памяти')
          ctx.errorText = '';
      }, 3000);
    } catch (e) {
      const err = String(e ?? 'Unknown error');
      ctx.errorText = err;
    } finally {
      ctx.isUnloadingModel = false;
      ctx.unloadingProgress = 0;
      ctx.busy = false;
    }
  }

  async function handleSend() {
    const text = ctx.prompt.trim();
    if (!text || ctx.busy) return;
    if (!ctx.isLoaded) {
      await message('Сначала загрузите модель и токенизатор', {
        title: 'Модель не загружена',
        kind: 'warning',
      });
      return;
    }
    // Сохраняем явные управляющие команды (/think, /no_think) в истории как есть
    const msgs = ctx.messages;
    msgs.push({ role: 'user', content: text } as any);
    msgs.push({ role: 'assistant', content: '' } as any);
    ctx.messages = msgs;
    ctx.prompt = '';
    queueMicrotask(() => {
      const c = ctx.messagesEl;
      if (!c) return;
      const atBottom = c.scrollTop + c.clientHeight >= c.scrollHeight - 32;
      if (atBottom) c.scrollTop = c.scrollHeight;
    });
    await generateFromHistory();
  }

  async function generateFromHistory() {
    ctx.busy = true;
    try {
      await stream.ensureListener();
      const msgs = ctx.messages;
      let hist =
        msgs[msgs.length - 1]?.role === 'assistant' && msgs[msgs.length - 1]?.content === ''
          ? msgs.slice(0, -1)
          : msgs.slice();
      // Раньше здесь автоматически подмешивался префикс `/think` или `/no_think`
      // на основе флага `enable_thinking`. Флаг удалён из UI — не изменяем
      // историю и позволяем явным управляющим командам (/think, /no_think)
      // оставаться только если пользователь их ввёл самостоятельно.
      const chatPrompt = await buildPromptWithChatTemplate(hist as any);
      console.log('[infer] frontend params', {
        use_custom_params: ctx.use_custom_params,
        temperature: ctx.use_custom_params && ctx.temperature_enabled ? ctx.temperature : null,
        top_k:
          ctx.use_custom_params && ctx.top_k_enabled
            ? Math.max(1, Math.floor(ctx.top_k_value))
            : null,
        top_p:
          ctx.use_custom_params && ctx.top_p_enabled
            ? ctx.top_p_value > 0 && ctx.top_p_value <= 1
              ? ctx.top_p_value
              : 0.9
            : null,
        min_p:
          ctx.use_custom_params && ctx.min_p_enabled
            ? ctx.min_p_value > 0 && ctx.min_p_value <= 1
              ? ctx.min_p_value
              : 0.05
            : null,
        context_length: undefined,
        repeat_penalty:
          ctx.use_custom_params && ctx.repeat_penalty_enabled ? ctx.repeat_penalty_value : null,
      });
      await invoke('generate_stream', {
        req: {
          prompt: chatPrompt,
          use_custom_params: ctx.use_custom_params,
          temperature: ctx.use_custom_params && ctx.temperature_enabled ? ctx.temperature : null,
          top_p:
            ctx.use_custom_params && ctx.top_p_enabled
              ? ctx.top_p_value > 0 && ctx.top_p_value <= 1
                ? ctx.top_p_value
                : 0.9
              : null,
          top_k:
            ctx.use_custom_params && ctx.top_k_enabled
              ? Math.max(1, Math.floor(ctx.top_k_value))
              : null,
          min_p:
            ctx.use_custom_params && ctx.min_p_enabled
              ? ctx.min_p_value > 0 && ctx.min_p_value <= 1
                ? ctx.min_p_value
                : 0.05
              : null,
          context_length: undefined,
          repeat_penalty:
            ctx.use_custom_params && ctx.repeat_penalty_enabled ? ctx.repeat_penalty_value : null,
          repeat_last_n: 64,
        },
      });
    } catch (e) {
      const err = String(e ?? 'Unknown error');
      const msgs = ctx.messages;
      const last = msgs[msgs.length - 1];
      if (last && last.role === 'assistant' && last.content === '') {
        last.content = `Ошибка: ${err}`;
        ctx.messages = msgs;
      }
      try {
        await message(err, { title: 'Ошибка генерации', kind: 'error' });
      } catch {}
    } finally {
      ctx.busy = false;
    }
  }

  async function stopGenerate() {
    try {
      await invoke('cancel_generation');
    } catch {}
  }

  async function pickModel() {
    if (ctx.format === 'gguf') {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'GGUF', extensions: ['gguf'] }],
      });
      if (typeof selected === 'string') ctx.modelPath = selected;
    } else {
      await message(
        'Для загрузки из HF Hub заполните repoId, revision (по желанию) и, для GGUF, имя файла.',
        { title: 'HF Hub', kind: 'info' },
      );
    }
  }

  // Удалён выбор токенизатора: он загружается автоматически из GGUF

  function destroy() {
    stream.destroy();
  }

  return {
    cancelLoading,
    loadGGUF,
    unloadGGUF,
    handleSend,
    handleAttachFile: _handleAttachFile,
    generateFromHistory,
    stopGenerate,
    pickModel,
    destroy,
    refreshDeviceInfo,
    setDeviceByToggle,
    ensureStreamListener: stream.ensureListener,
  };
}
