import { mount, unmount } from "svelte";
import Eye from "phosphor-svelte/lib/Eye";
import EyeSlash from "phosphor-svelte/lib/EyeSlash";
import { renderMarkdownToSafeHtml } from "$lib/chat/markdown";
import { getCodeMirrorRenderer, cleanupRenderer } from "$lib/chat/codemirror-renderer";
import { enableExternalLinks } from "$lib/chat/external-links";
import type { BubbleCtx } from "./bubble_ctx";

function hasMarkdownFeatures(text: string): boolean {
  const t = text ?? "";
  if (!t) return false;
  return (
    /(^|\n)\s{0,3}#{1,6}\s+/m.test(t) || // заголовки
    /(^|\n)\s{0,3}[-*+]\s+/.test(t) || // марк. списки
    /(^|\n)\s{0,3}\d+\.\s+/.test(t) || // нум. списки
    /```|~~~/.test(t) || // кодовые блоки
    /`[^`\n]+`/.test(t) || // инлайн-код
    /!\[[^\]]*\]\([^\)]+\)/.test(t) || // изображения
    /\[[^\]]+\]\([^\)]+\)/.test(t) || // ссылки
    /(^|\n)\|[^\n]*\|/.test(t) || // таблицы
    /(^|\n)>\s+/.test(t) // цитаты
  );
}

export function ensureMarkdownContainer(ctx: BubbleCtx, bubble: HTMLDivElement): BubbleCtx {
  if (ctx.lastKind !== "text" || !ctx.mdEl) {
    ctx.mdEl = document.createElement("div");
    ctx.mdEl.className = "md-stream";

    // controls
    const controls = document.createElement("div");
    controls.className = "md-controls";
    const toggleBtn = document.createElement("button");
    toggleBtn.type = "button";
    toggleBtn.className = "md-toggle";
    const eyeHost = document.createElement("span");
    eyeHost.className = "md-eye-host";
    toggleBtn.appendChild(eyeHost);
    controls.appendChild(toggleBtn);

    // content containers
    const contentEl = document.createElement("div");
    contentEl.className = "md-content";
    const rawEl = document.createElement("pre");
    rawEl.className = "md-raw";

    // assemble
    ctx.mdEl.appendChild(controls);
    ctx.mdEl.appendChild(contentEl);
    ctx.mdEl.appendChild(rawEl);
    bubble.appendChild(ctx.mdEl);

    // mount eye icon
    ctx.mdEyeHost = eyeHost;
    ctx.mdEyeIcon = mount(Eye, { target: eyeHost, props: { size: 16, weight: "regular" } });

    // toggle handler
    toggleBtn.addEventListener("click", () => {
      const showingRaw = ctx.mdEl?.classList.toggle("show-raw") ?? false;
      try { if (ctx.mdEyeIcon) unmount(ctx.mdEyeIcon); } catch {}
      if (ctx.mdEyeHost) {
        ctx.mdEyeIcon = mount(showingRaw ? EyeSlash : Eye, { target: ctx.mdEyeHost, props: { size: 16, weight: "regular" } });
      }
    });

    ctx.mdControlsEl = controls;
    ctx.mdToggleBtn = toggleBtn;
    // По умолчанию скрываем контрол до появления markdown-признаков
    try { (ctx.mdControlsEl as HTMLElement).style.display = "none"; } catch {}
    ctx.mdContentEl = contentEl;
    ctx.mdRawEl = rawEl;
    ctx.mdText = "";
  }
  return ctx;
}

export function appendMarkdownText(ctx: BubbleCtx, text: string): BubbleCtx {
  const normalized = text.replace(/\r/g, "");
  ctx.mdText += normalized;
  if (ctx.mdContentEl) {
    ctx.mdContentEl.innerHTML = renderMarkdownToSafeHtml(ctx.mdText);
    
    // Enable external link handling
    enableExternalLinks(ctx.mdContentEl);
    
    // Apply CodeMirror rendering to code blocks
    try {
      if (!ctx.codeMirrorWatching) {
        const renderer = getCodeMirrorRenderer(ctx.mdContentEl);
        renderer.startWatching(ctx.mdContentEl);
        ctx.codeMirrorWatching = true;
      }
    } catch (error) {
      console.error('Failed to apply CodeMirror rendering:', error);
    }
  }
  if (ctx.mdRawEl) {
    ctx.mdRawEl.textContent = ctx.mdText;
  }
  // Показываем/скрываем кнопку-глаз только если есть элементы Markdown
  try {
    if (ctx.mdControlsEl) {
      (ctx.mdControlsEl as HTMLElement).style.display = hasMarkdownFeatures(ctx.mdText) ? "flex" : "none";
    }
  } catch {}
  ctx.lastKind = "text";
  return ctx;
}


