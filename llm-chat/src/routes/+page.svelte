<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { open, message } from "@tauri-apps/plugin-dialog";

  let modelPath = "";
  let tokenizerPath = "";
  type ChatMessage = { role: "user" | "assistant"; content: string };
  let prompt = "";
  let messages: ChatMessage[] = [];
  let messagesEl: HTMLDivElement | null = null;
  let busy = false;
  let format: "gguf" = "gguf";
  let isLoaded = false;
  let errorText = "";

  let unlisten: UnlistenFn | null = null;
  async function ensureListener() {
    if (!unlisten) {
      unlisten = await listen<string>("token", (event) => {
        const token = event.payload ?? "";
        if (token === "") {
          const last = messages[messages.length - 1];
          if (!last || last.role !== "assistant" || last.content !== "") {
            messages.push({ role: "assistant", content: "" });
            messages = messages; // триггер перерисовки
          }
          return;
        }
        const last = messages[messages.length - 1];
        if (last && last.role === "assistant") {
          last.content += token;
          messages = messages; // триггер перерисовки
          queueMicrotask(() => {
            if (messagesEl) messagesEl.scrollTop = messagesEl.scrollHeight;
          });
        }
      });
    }
  }

  async function loadGGUF() {
    if (!modelPath) {
      await message("Укажите путь к .gguf", { title: "Загрузка модели", kind: "warning" });
      return;
    }
    busy = true; isLoaded = false; errorText = "";
    try {
      await ensureListener();
      await invoke("load_model", { req: { format: "gguf", model_path: modelPath, tokenizer_path: null } });
      isLoaded = true;
    } catch (e) {
      const err = String(e ?? "Unknown error");
      errorText = err;
      await message(err, { title: "Ошибка загрузки модели", kind: "error" });
    } finally { busy = false; }
  }

  function buildQwenPromptForChat(history: ChatMessage[]): string {
    let text = "";
    // Без системного промпта — только история и управляющие теги
    for (const m of history) {
      if (m.role === "user") {
        const payload = m.content.replace(/^\s*\/(?:no_think|think)\b[ \t]*/i, "").trim();
        text += `<|im_start|>user\n${payload}<|im_end|>\n`;
      } else {
        text += `<|im_start|>assistant\n${m.content}<|im_end|>\n`;
      }
    }
    text += `<|im_start|>assistant\n`;
    return text;
  }

  async function handleSend() {
    const text = prompt.trim();
    if (!text || busy) return;
    if (!isLoaded) {
      await message("Сначала загрузите модель и токенизатор", { title: "Модель не загружена", kind: "warning" });
      return;
    }
    messages.push({ role: "user", content: text });
    messages.push({ role: "assistant", content: "" });
    messages = messages; // триггер перерисовки
    prompt = "";
    queueMicrotask(() => { if (messagesEl) messagesEl.scrollTop = messagesEl.scrollHeight; });
    await generateFromHistory();
  }

  async function generateFromHistory() {
    busy = true;
    try {
      await ensureListener();
      const hist = messages[messages.length - 1]?.role === "assistant" && messages[messages.length - 1]?.content === ""
        ? messages.slice(0, -1)
        : messages.slice();
      const chatPrompt = buildQwenPromptForChat(hist);
      await invoke("generate_stream", {
        req: { prompt: chatPrompt, temperature: 0.8, top_p: null, top_k: null, repeat_penalty: 1.1, repeat_last_n: 64, use_custom_params: true }
      });
    } catch (e) {
      const err = String(e ?? "Unknown error");
      const last = messages[messages.length - 1];
      if (last && last.role === "assistant" && last.content === "") {
        last.content = `Ошибка: ${err}`;
        messages = messages; // триггер перерисовки
      }
      try { await message(err, { title: "Ошибка генерации", kind: "error" }); } catch {}
    } finally { busy = false; }
  }

  async function pickModel() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "GGUF", extensions: ["gguf"] }]
    });
    if (typeof selected === "string") modelPath = selected;
  }

  async function pickTokenizer() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "JSON", extensions: ["json"] }]
    });
    if (typeof selected === "string") tokenizerPath = selected;
  }
</script>

<main class="wrap">
  <header class="title">Oxide Lab</header>
  <section class="loader">
    <div class="switch">
      <button class:active={format === 'gguf'} onclick={() => format = 'gguf'}>GGUF</button>
    </div>

  </section>

  <section class="chat">
    <div class="messages" bind:this={messagesEl}>
      {#if messages.length === 0}
        <div class="empty">Нет сообщений. Напишите что-нибудь…</div>
      {/if}
      {#each messages as m}
        <div class="message {m.role}"><div class="bubble">{m.content}</div></div>
      {/each}
    </div>
    <div class="composer">
      <textarea id="chat-input" rows="4" bind:value={prompt} placeholder="Напишите сообщение..." onkeydown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleSend(); } }}></textarea>
      <button class="primary" onclick={handleSend} disabled={busy || !prompt.trim() || !isLoaded}>Отправить</button>
    </div>
  </section>
</main>

<style>
  :root {
    --bg: #f7f5f2;
    --card: #ffffff;
    --text: #2b2a29;
    --muted: #6d6a6a;
    --accent: #b3cde0; /* пастельный голубой */
    --accent-2: #e2c6ff; /* пастельный лиловый */
    --radius: 12px;
  }

  .wrap {
    min-height: 100dvh;
    height: 100dvh;
    background: linear-gradient(180deg, var(--bg), #f1efec 50%, var(--bg));
    color: var(--text);
    display: grid;
    grid-template-columns: 1fr;
    grid-template-rows: auto auto minmax(0, 1fr);
    gap: 16px;
    padding: 16px;
    width: 100%;
    margin: 0;
    box-sizing: border-box;
  }

  .title {
    font-size: 24px;
    font-weight: 700;
    letter-spacing: 0.5px;
  }

  .loader, .chat {
    background: var(--card);
    border-radius: var(--radius);
    padding: 16px;
    box-shadow: 0 6px 30px rgba(0,0,0,0.05);
  }

  .switch { display: inline-flex; gap: 8px; margin-bottom: 12px; }
  .switch button {
    padding: 8px 12px; border-radius: 10px; border: 1px solid transparent;
    background: #f3f3f3; color: var(--text); cursor: default;
  }
  .switch button.active { background: var(--accent); }

  

  .primary {
    background: var(--accent-2); border: none; border-radius: 12px; padding: 10px 14px;
    cursor: default; color: #3a2f4f; font-weight: 600; margin-top: 6px;
  }

  .file-row { display: flex; gap: 8px; align-items: center; }

  .chat {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .messages {
    margin-top: 8px;
    background: #faf9f7;
    border: 1px dashed #e8e6e3;
    border-radius: 10px;
    padding: 12px;
    flex: 1 1 auto;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
    color: var(--text);
  }
  .message { display: flex; }
  .message.user { justify-content: flex-end; }
  .message.assistant { justify-content: flex-start; }
  .bubble {
    max-width: 80%;
    padding: 10px 12px;
    border-radius: 12px;
    background: #eae6f8;
    color: var(--text);
    white-space: pre-wrap;
  }
  .message.user .bubble { background: var(--accent-2); }
  .message.assistant .bubble { background: #e8f0f8; }
  .composer { display: flex; gap: 8px; margin-top: 8px; }
  .composer textarea {
    flex: 1;
    border: 1px solid #e8e6e3;
    background: #fcfbfa;
    color: var(--text);
    border-radius: 10px;
    padding: 10px 12px;
    outline: none;
    resize: vertical;
  }
  .empty { color: var(--muted); font-size: 12px; }
  .error { margin-top: 8px; color: #a11; font-size: 12px; }

  @media (prefers-color-scheme: dark) {
    :root {
      --bg: #1f1f1f; --card: #262626; --text: #eee; --muted: #aaa; --accent: #5b7c99; --accent-2: #695b99;
    }
    
    .messages { background: #222; border-color: #333; }
    .message.user .bubble { background: #4a3f66; color: #eee; }
    .message.assistant .bubble { background: #2f3640; color: #eee; }
    .composer textarea { background: #2d2d2d; border-color: #3a3a3a; color: #eee; }
  }

  @media (min-width: 960px) {
    .wrap {
      grid-template-columns: 1fr 360px; /* слева чат, справа настройки */
      grid-template-rows: auto minmax(0, 1fr);
      gap: 20px;
      padding: 24px 32px;
    }
    .title { grid-column: 1 / -1; }
    .chat { grid-column: 1; grid-row: 2; min-height: 0; }
    .loader { grid-column: 2; grid-row: 2; min-height: 0; overflow: auto; }
  }
</style>
