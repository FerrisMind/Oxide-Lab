import { invoke } from "@tauri-apps/api/core";
import { open, message } from "@tauri-apps/plugin-dialog";
import type { ChatControllerCtx } from "./types";
import { createStreamListener } from "./listener";
import { buildQwenPromptForChat } from "$lib/chat/prompts";

export function createActions(ctx: ChatControllerCtx) {
  const stream = createStreamListener(ctx);

  function cancelLoading() {
    ctx.isCancelling = true;
    ctx.loadingStage = "cancelling";
    setTimeout(() => {
      ctx.isLoadingModel = false;
      ctx.loadingProgress = 0;
      ctx.loadingStage = "";
      ctx.isCancelling = false;
      ctx.busy = false;
    }, 500);
  }

  async function loadGGUF() {
    if (!ctx.modelPath || !ctx.tokenizerPath) {
      await message("Укажите путь к .gguf и tokenizer.json", { title: "Загрузка модели", kind: "warning" });
      return;
    }
    ctx.isLoadingModel = true;
    ctx.loadingProgress = 0;
    ctx.loadingStage = "model";
    ctx.busy = true;
    ctx.isLoaded = false;
    ctx.errorText = "";
    try {
      const modelLoadInterval = setInterval(() => {
        if (ctx.isCancelling) return;
        if (ctx.loadingProgress < 60) ctx.loadingProgress += Math.random() * 8 + 2;
      }, 150);
      await stream.ensureListener();
      const context_length = Math.max(1, Math.floor(ctx.ctx_limit_value));
      const n_gpu_layers = Math.max(0, Math.floor((ctx as any).n_gpu_layers ?? 0));
      console.log("[load] frontend params", { context_length, n_gpu_layers });
      if (ctx.isCancelling) return;
      ctx.loadingStage = "model";
      ctx.loadingProgress = 30;
      // Выбираем устройство автоматически: если есть CUDA — будет использовано, иначе CPU
      await invoke("load_model", { req: { format: "gguf", model_path: ctx.modelPath, tokenizer_path: ctx.tokenizerPath, context_length, n_gpu_layers, device: { kind: "auto" }, fallback_to_cpu_on_oom: true } });
      if (ctx.isCancelling) return;
      ctx.loadingStage = "tokenizer";
      ctx.loadingProgress = 70;
      const tokenizerLoadInterval = setInterval(() => {
        if (ctx.isCancelling) return;
        if (ctx.loadingProgress < 95) ctx.loadingProgress += Math.random() * 3 + 1;
      }, 100);
      await new Promise((r) => setTimeout(r, 800));
      if (ctx.isCancelling) return;
      clearInterval(modelLoadInterval);
      clearInterval(tokenizerLoadInterval);
      ctx.loadingStage = "complete";
      ctx.loadingProgress = 100;
      await new Promise((r) => setTimeout(r, 500));
      if (ctx.isCancelling) return;
      ctx.isLoaded = true;
    } catch (e) {
      const err = String(e ?? "Unknown error");
      ctx.errorText = err;
      try { await message(err, { title: "Ошибка загрузки модели", kind: "error" }); } catch {}
    } finally {
      ctx.isLoadingModel = false;
      ctx.loadingProgress = 0;
      ctx.loadingStage = "";
      ctx.busy = false;
    }
  }

  async function unloadGGUF() {
    if (ctx.busy || !ctx.isLoaded) return;
    ctx.isUnloadingModel = true;
    ctx.unloadingProgress = 0;
    ctx.busy = true;
    ctx.errorText = "";
    try {
      const unloadInterval = setInterval(() => {
        if (ctx.unloadingProgress < 80) ctx.unloadingProgress += Math.random() * 15 + 5;
      }, 100);
      await invoke("unload_model");
      ctx.unloadingProgress = 100;
      clearInterval(unloadInterval);
      await new Promise((r) => setTimeout(r, 300));
      ctx.isLoaded = false;
      ctx.messages = [];
      ctx.errorText = "Модель и токенизатор успешно выгружены из памяти";
      setTimeout(() => { if (ctx.errorText === "Модель и токенизатор успешно выгружены из памяти") ctx.errorText = ""; }, 3000);
    } catch (e) {
      const err = String(e ?? "Unknown error");
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
      await message("Сначала загрузите модель и токенизатор", { title: "Модель не загружена", kind: "warning" });
      return;
    }
    // В UI не показываем управляющие теги — очищаем их из отображаемого сообщения
    const textUi = text.replace(/^\s*\/(?:think|no_think)\b[ \t]*/i, "");
    const msgs = ctx.messages;
    msgs.push({ role: "user", content: textUi } as any);
    msgs.push({ role: "assistant", content: "" } as any);
    ctx.messages = msgs;
    ctx.prompt = "";
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
      let hist = msgs[msgs.length - 1]?.role === "assistant" && msgs[msgs.length - 1]?.content === ""
        ? msgs.slice(0, -1)
        : msgs.slice();
      // В историю для промпта подмешиваем управляющий тег только для последнего пользовательского сообщения
      for (let i = hist.length - 1; i >= 0; i--) {
        const m: any = hist[i];
        if (m && m.role === "user") {
          const hasCtrl = /^\s*\/(think|no_think)\b/i.test(m.content ?? "");
          if (!hasCtrl) {
            const controlPrefix = ctx.enable_thinking ? "/think " : "/no_think ";
            hist = hist.slice();
            hist[i] = { ...m, content: controlPrefix + (m.content ?? "") };
          }
          break;
        }
      }
      const chatPrompt = buildQwenPromptForChat(hist as any);
      console.log("[infer] frontend params", {
        use_custom_params: ctx.use_custom_params,
        temperature: ctx.use_custom_params && ctx.temperature_enabled ? ctx.temperature : null,
        top_k: ctx.use_custom_params && ctx.top_k_enabled ? Math.max(1, Math.floor(ctx.top_k_value)) : null,
        top_p: ctx.use_custom_params && ctx.top_p_enabled ? (ctx.top_p_value > 0 && ctx.top_p_value <= 1 ? ctx.top_p_value : 0.9) : null,
        min_p: ctx.use_custom_params && ctx.min_p_enabled ? (ctx.min_p_value > 0 && ctx.min_p_value <= 1 ? ctx.min_p_value : 0.05) : null,
        context_length: undefined,
        repeat_penalty: ctx.use_custom_params && ctx.repeat_penalty_enabled ? ctx.repeat_penalty_value : null,
      });
      await invoke("generate_stream", {
        req: {
          prompt: chatPrompt,
          use_custom_params: ctx.use_custom_params,
          temperature: ctx.use_custom_params && ctx.temperature_enabled ? ctx.temperature : null,
          top_p: ctx.use_custom_params && ctx.top_p_enabled ? (ctx.top_p_value > 0 && ctx.top_p_value <= 1 ? ctx.top_p_value : 0.9) : null,
          top_k: ctx.use_custom_params && ctx.top_k_enabled ? Math.max(1, Math.floor(ctx.top_k_value)) : null,
          min_p: ctx.use_custom_params && ctx.min_p_enabled ? (ctx.min_p_value > 0 && ctx.min_p_value <= 1 ? ctx.min_p_value : 0.05) : null,
          context_length: undefined,
          repeat_penalty: ctx.use_custom_params && ctx.repeat_penalty_enabled ? ctx.repeat_penalty_value : null,
          repeat_last_n: 64,
        },
      });
    } catch (e) {
      const err = String(e ?? "Unknown error");
      const msgs = ctx.messages;
      const last = msgs[msgs.length - 1];
      if (last && last.role === "assistant" && last.content === "") {
        last.content = `Ошибка: ${err}`;
        ctx.messages = msgs;
      }
      try { await message(err, { title: "Ошибка генерации", kind: "error" }); } catch {}
    } finally {
      ctx.busy = false;
    }
  }

  async function stopGenerate() {
    try { await invoke("cancel_generation"); } catch {}
  }

  async function pickModel() {
    const selected = await open({ multiple: false, filters: [{ name: "GGUF", extensions: ["gguf"] }] });
    if (typeof selected === "string") ctx.modelPath = selected;
  }

  async function pickTokenizer() {
    const selected = await open({ multiple: false, filters: [{ name: "JSON", extensions: ["json"] }] });
    if (typeof selected === "string") ctx.tokenizerPath = selected;
  }

  function destroy() {
    stream.destroy();
  }

  return { cancelLoading, loadGGUF, unloadGGUF, handleSend, generateFromHistory, stopGenerate, pickModel, pickTokenizer, destroy };
}


