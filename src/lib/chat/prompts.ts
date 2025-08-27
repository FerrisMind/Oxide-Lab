import { sanitizeForPrompt } from '$lib/chat/sanitize';
import { invoke } from '@tauri-apps/api/core';
import type { ChatMessage } from '$lib/chat/types';

/**
 * Формирует промпт в формате Qwen чата.
 * Поддерживает управляющие команды пользователя: /think и /no_think.
 * Если активен /no_think, добавляется пустой блок <think>...</think> для явного отключения CoT.
 */
export async function buildPromptWithChatTemplate(history: ChatMessage[]): Promise<string> {
  // Определяем управляющий префикс (/think или /no_think) на основе последнего
  // пользовательского сообщения — это нужно и для шаблона, и для fallback.
  let control: 'think' | 'no_think' | null = null;
  const lastUser = [...history].reverse().find((m) => m.role === 'user');
  if (lastUser) {
    const t = lastUser.content.trim();
    if (/^\s*\/no_think\b/i.test(t)) control = 'no_think';
    else if (/^\s*\/think\b/i.test(t)) control = 'think';
  }

  // Пытаемся получить chat_template из бэкенда (если токенизатор содержит его)
  const tpl = await invoke<string | null>('get_chat_template').catch(() => null);
  console.log('[template] requested from backend, present=', !!tpl);
  if (tpl && typeof tpl === 'string' && tpl.trim().length > 0) {
    // Рендерим на бэкенде через minijinja, чтобы поддержать нативные шаблоны
    const hist = history.map((m) => ({ role: m.role, content: sanitizeForPrompt(m.content) }));
    let out = await invoke<string>('render_prompt', { messages: hist });
    console.log('[template] applied (backend render), prefix=', out.slice(0, 160));
    // Если шаблон вернулся снаружи и пользователь указал /no_think, гарантируем
    // наличие пустого блока <think>...</think>, если его нет в шаблоне.
    if (control === 'no_think' && !(out.includes('<think>') && out.includes('</think>'))) {
      out = out + '\n<think>\n\n</think>\n\n';
    }
    return out;
  }

  // Fallback: Qwen-совместимый формат
  let text = '';

  // Системный промпт не используется — следуем только истории и управляющим тегам

  for (const m of history) {
    const clean = sanitizeForPrompt(m.content);
    if (m.role === 'user') {
      let payload = clean;
      // Убираем только префикс управляющей команды, сохраняя остальной текст пользователя
      payload = payload.replace(/^\s*\/(?:no_think|think)\b[ \t]*/i, '').trim();
      text += `<|im_start|>user\n${payload}<|im_end|>\n`;
    } else {
      text += `<|im_start|>assistant\n${clean}<|im_end|>\n`;
    }
  }

  // Открыть ассистента для ответа текущего шага
  text += `<|im_start|>assistant\n`;

  // Официальный способ жёстко отключить размышления — вставить пустой блок <think>...</think>
  if (control === 'no_think') {
    text += `<think>\n\n</think>\n\n`;
  }

  return text;
}
