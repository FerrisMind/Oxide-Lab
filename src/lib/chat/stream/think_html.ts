import { mount } from 'svelte';
import Brain from 'phosphor-svelte/lib/Brain';
import CaretRight from 'phosphor-svelte/lib/CaretRight';
import type { BubbleCtx } from './bubble_ctx';

const THINK_OPEN_EXACT =
  '<details class="cot"><summary><span class="brain-host"></span><span class="cot-label">Рассуждения</span><span class="caret-host"></span></summary><pre class="cot-pre">';
const THINK_OPEN_PREFIX = '<details class="cot"';
const THINK_OPEN_SUFFIX = '<pre class="cot-pre">';
const THINK_CLOSE = '</pre></details>';

export function appendThinkAwareHtml(
  ctx: BubbleCtx,
  bubble: HTMLDivElement,
  chunk: string,
): BubbleCtx {
  if (!chunk) return ctx;
  let s = chunk;
  while (s.length > 0) {
    if (!ctx.inThink) {
      let pos = s.indexOf(THINK_OPEN_EXACT);
      let openLen = THINK_OPEN_EXACT.length;
      if (pos === -1) {
        const prefixPos = s.indexOf(THINK_OPEN_PREFIX);
        if (prefixPos !== -1) {
          const suffixPos = s.indexOf(THINK_OPEN_SUFFIX, prefixPos);
          if (suffixPos !== -1) {
            pos = prefixPos;
            openLen = suffixPos + THINK_OPEN_SUFFIX.length - prefixPos;
          }
        }
      }
      if (pos !== -1) {
        const before = s.slice(0, pos);
        if (before) bubble.insertAdjacentHTML('beforeend', before);
        // Парсер эмитирует открывающий HTML только для непустого блока,
        // поэтому если мы видим открывающий HTML — вставляем его и
        // переводим контекст в состояние inThink.
        const openHtml = s.slice(pos, pos + openLen);
        bubble.insertAdjacentHTML('beforeend', openHtml);
        ctx.inThink = true;
        const pres = bubble.getElementsByClassName('cot-pre');
        ctx.thinkPre = pres[pres.length - 1] as HTMLElement;
        const summaries = bubble.getElementsByTagName('summary');
        ctx.thinkSummary = summaries[summaries.length - 1] as HTMLElement;
        const caretHosts = bubble.getElementsByClassName('caret-host');
        ctx.thinkCaretHost = caretHosts[caretHosts.length - 1] as HTMLElement;
        const brainHosts = bubble.getElementsByClassName('brain-host');
        ctx.thinkBrainHost = brainHosts[brainHosts.length - 1] as HTMLElement;
        if (ctx.thinkSummary && ctx.thinkCaretHost) {
          try {
            ctx.thinkSummary.appendChild(ctx.thinkCaretHost);
          } catch {}
        }
        if (ctx.thinkCaretHost) {
          ctx.thinkCaretIcon = mount(CaretRight, {
            target: ctx.thinkCaretHost as HTMLElement,
            props: { size: 16, weight: 'bold' },
          });
        }
        if (ctx.thinkBrainHost) {
          ctx.thinkBrainIcon = mount(Brain, {
            target: ctx.thinkBrainHost as HTMLElement,
            props: { size: 16, weight: 'regular' },
          });
        }
        s = s.slice(pos + openLen);
        continue;
      }
      bubble.insertAdjacentHTML('beforeend', s);
      s = '';
    } else {
      const pos = s.indexOf(THINK_CLOSE);
      if (pos !== -1) {
        // Закрывающий HTML пришёл — вставляем содержимое (если есть) и закрываем блок.
        const inner = s.slice(0, pos);
        if (inner && ctx.thinkPre) ctx.thinkPre.insertAdjacentHTML('beforeend', inner);
        bubble.insertAdjacentHTML('beforeend', THINK_CLOSE);
        ctx.inThink = false;
        ctx.thinkPre = null;
        s = s.slice(pos + THINK_CLOSE.length);
      } else {
        // Внутри открытого блока без закрытия — добавляем всё во внутренний pre
        if (ctx.thinkPre) {
          ctx.thinkPre.insertAdjacentHTML('beforeend', s);
        }
        s = '';
      }
    }
  }
  return ctx;
}
