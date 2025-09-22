import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { createStreamParser } from '$lib/chat/parser';
import { chatState, chatUiMounted } from '$lib/stores/chat';
import { get as getStore } from 'svelte/store';

let started = false;
let unlisten: UnlistenFn | null = null;
const streamParser = createStreamParser();
let streamBuf = '';
let subscribed = false;

async function startListenerIfNeeded() {
  if (started) return;
  if (getStore(chatUiMounted)) return; // UI active, no need for background listener
  started = true;
  try {
    const commitSegments = (
      segments: ReturnType<typeof streamParser.parse>['segments'],
      uiActive: boolean,
    ) => {
      if (segments.length === 0) return;
      const onlyText = segments
        .filter((s) => s.kind === 'text')
        .map((s) => s.data)
        .join('');
      if (!onlyText) return;
      if (!uiActive) {
        chatState.update((s) => {
          const msgs = s.messages.slice();
          const last = msgs[msgs.length - 1];
          if (last && last.role === 'assistant') {
            last.content = (last.content ?? '') + onlyText;
          } else {
            msgs.push({ role: 'assistant', content: onlyText } as any);
          }
          return { ...s, messages: msgs };
        });
      }
    };

    unlisten = await listen<string>('token', (event) => {
      const token = event.payload ?? '';
      const uiActive = !!getStore(chatUiMounted);
      if (token === '') {
        if (!uiActive) {
          chatState.update((s) => {
            const msgs = s.messages.slice();
            const last = msgs[msgs.length - 1];
            if (!last || last.role !== 'assistant' || last.content !== '') {
              msgs.push({ role: 'assistant', content: '' } as any);
            }
            return { ...s, messages: msgs, busy: true };
          });
        }
        streamBuf = '';
        streamParser.reset();
        return;
      }
      if (token === '[DONE]') {
        const { segments } = streamParser.parse(streamBuf);
        commitSegments(segments, uiActive);
        streamBuf = '';
        streamParser.reset();
        if (!uiActive) chatState.update((s) => ({ ...s, busy: false }));
        return;
      }
      streamBuf += token;
      const { segments, remainder } = streamParser.parse(streamBuf);
      streamBuf = remainder;
      commitSegments(segments, uiActive);
    });
  } catch {}
}

function stopListener() {
  try {
    if (unlisten) unlisten();
  } catch {}
  unlisten = null;
  started = false;
}

export async function ensureGlobalChatStream() {
  if (subscribed) return;
  subscribed = true;
  // react to UI mount/unmount and toggle background listener
  chatUiMounted.subscribe((ui) => {
    if (ui) {
      // UI active — stop background
      stopListener();
    } else {
      // UI hidden — ensure background listener
      void startListenerIfNeeded();
    }
  });
  // Initial check
  await startListenerIfNeeded();
}

export function stopGlobalChatStream() {
  stopListener();
}
