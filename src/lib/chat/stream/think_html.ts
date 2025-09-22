import type { BubbleCtx } from './bubble_ctx';
import { THINK_OPEN_TOKEN, THINK_CLOSE_TOKEN } from '../parser/constants';

const thinkState = new Map<string, boolean>();
const THINK_LOADING_LABEL = 'Рассуждения…';
const THINK_READY_LABEL = 'Размышления';
const THINKING_KEY_PREFIX = 'thinking-block-';

function ensureThinkKey(ctx: BubbleCtx) {
  if (!ctx.thinkKey) {
    const suffix = Date.now().toString(36) + '-' + Math.random().toString(36).slice(2, 8);
    ctx.thinkKey = THINKING_KEY_PREFIX + suffix;
  }
}

function applyExpansion(ctx: BubbleCtx, expanded: boolean) {
  ctx.thinkExpanded = expanded;
  if (ctx.thinkKey) {
    thinkState.set(ctx.thinkKey, expanded);
  }
  if (ctx.thinkToggleBtn) {
    ctx.thinkToggleBtn.setAttribute('aria-expanded', expanded ? 'true' : 'false');
  }
  if (ctx.thinkChevronEl) {
    ctx.thinkChevronEl.textContent = expanded ? '▾' : '▸';
  }
  if (ctx.thinkCardEl) {
    ctx.thinkCardEl.classList.toggle('is-expanded', expanded);
  }
  if (ctx.thinkBody) {
    ctx.thinkBody.style.display = expanded ? '' : 'none';
  }
}

function setLoading(ctx: BubbleCtx, loading: boolean) {
  if (ctx.thinkLoaderEl) {
    ctx.thinkLoaderEl.classList.toggle('is-hidden', !loading);
  }
  if (ctx.thinkCardEl) {
    ctx.thinkCardEl.classList.toggle('is-loading', loading);
  }
  if (ctx.thinkLabelEl) {
    ctx.thinkLabelEl.textContent = loading ? THINK_LOADING_LABEL : THINK_READY_LABEL;
  }
}

function attachToggleHandler(ctx: BubbleCtx) {
  if (!ctx.thinkToggleBtn) return;
  if (ctx.thinkToggleHandler) {
    ctx.thinkToggleBtn.removeEventListener('click', ctx.thinkToggleHandler);
  }
  const handler = (event: Event) => {
    event.preventDefault();
    const current = ctx.thinkKey ? thinkState.get(ctx.thinkKey) ?? ctx.thinkExpanded ?? false : ctx.thinkExpanded ?? false;
    applyExpansion(ctx, !current);
  };
  ctx.thinkToggleHandler = handler;
  ctx.thinkToggleBtn.addEventListener('click', handler);
}

function ensureThinkBlock(ctx: BubbleCtx, bubble: HTMLDivElement, loading: boolean) {
  ensureThinkKey(ctx);
  const hasStored = ctx.thinkKey ? thinkState.has(ctx.thinkKey) : false;
  const expanded = hasStored && ctx.thinkKey ? thinkState.get(ctx.thinkKey)! : loading;
  if (ctx.thinkKey) {
    thinkState.set(ctx.thinkKey, expanded);
  }

  if (!ctx.thinkBlock || !ctx.thinkBlock.isConnected) {
    const block = document.createElement('div');
    block.className = 'thinking-block';
    if (ctx.thinkKey) {
      block.dataset.thinking = ctx.thinkKey;
    }

    const card = document.createElement('div');
    card.className = 'thinking-card';
    block.appendChild(card);

    const header = document.createElement('div');
    header.className = 'thinking-header';
    card.appendChild(header);

    const loader = document.createElement('span');
    loader.className = 'thinking-loader';
    header.appendChild(loader);

    const toggle = document.createElement('button');
    toggle.type = 'button';
    toggle.className = 'thinking-toggle';
    header.appendChild(toggle);

    const chevron = document.createElement('span');
    chevron.className = 'thinking-chevron';
    toggle.appendChild(chevron);

    const label = document.createElement('span');
    label.className = 'thinking-title';
    toggle.appendChild(label);

    const body = document.createElement('div');
    body.className = 'thinking-body';
    card.appendChild(body);

    bubble.appendChild(block);

    ctx.thinkBlock = block;
    ctx.thinkCardEl = card;
    ctx.thinkBody = body;
    ctx.thinkToggleBtn = toggle;
    ctx.thinkLoaderEl = loader;
    ctx.thinkChevronEl = chevron;
    ctx.thinkLabelEl = label;

    attachToggleHandler(ctx);
  }

  applyExpansion(ctx, expanded);
  setLoading(ctx, loading);
}

function appendThinkContent(ctx: BubbleCtx, html: string) {
  if (!ctx.thinkBody) return;
  ctx.thinkBody.insertAdjacentHTML('beforeend', html);
}

export function appendThinkAwareHtml(
  ctx: BubbleCtx,
  bubble: HTMLDivElement,
  chunk: string,
): BubbleCtx {
  if (!chunk) return ctx;
  let buffer = chunk;
  while (buffer.length > 0) {
    if (buffer.startsWith(THINK_OPEN_TOKEN)) {
      ensureThinkBlock(ctx, bubble, true);
      ctx.inThink = true;
      buffer = buffer.slice(THINK_OPEN_TOKEN.length);
      continue;
    }
    if (buffer.startsWith(THINK_CLOSE_TOKEN)) {
      setLoading(ctx, false);
      ctx.inThink = false;
      buffer = buffer.slice(THINK_CLOSE_TOKEN.length);
      continue;
    }

    const nextOpen = buffer.indexOf(THINK_OPEN_TOKEN);
    const nextClose = buffer.indexOf(THINK_CLOSE_TOKEN);
    const nextIndexCandidates = [nextOpen, nextClose];
    let nextIndex = -1;
    for (const value of nextIndexCandidates) {
      if (value !== -1 && (nextIndex === -1 || value < nextIndex)) {
        nextIndex = value;
      }
    }

    const textChunk = nextIndex === -1 ? buffer : buffer.slice(0, nextIndex);
    if (textChunk) {
      if (!ctx.thinkBlock || !ctx.thinkBlock.isConnected) {
        ensureThinkBlock(ctx, bubble, ctx.inThink);
      }
      appendThinkContent(ctx, textChunk);
    }

    if (nextIndex === -1) {
      break;
    }
    buffer = buffer.slice(nextIndex);
  }

  return ctx;
}
