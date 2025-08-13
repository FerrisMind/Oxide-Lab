import { sanitizeForPrompt } from "$lib/chat/sanitize";
import type { ChatMessage } from "$lib/chat/types";

/**
 * Формирует промпт в формате Qwen чата.
 * Поддерживает управляющие команды пользователя: /think и /no_think.
 * Если активен /no_think, добавляется пустой блок <think>...</think> для явного отключения CoT.
 */
export function buildQwenPromptForChat(
  history: ChatMessage[]
): string {
  let text = "";

  // Системный промпт не используется — следуем только истории и управляющим тегам

  let control: "think" | "no_think" | null = null;
  const lastUser = [...history].reverse().find((m) => m.role === "user");
  if (lastUser) {
    const t = lastUser.content.trim();
    if (/^\s*\/no_think\b/i.test(t)) control = "no_think";
    else if (/^\s*\/think\b/i.test(t)) control = "think";
  }

  for (const m of history) {
    const clean = sanitizeForPrompt(m.content);
    if (m.role === "user") {
      let payload = clean;
      // Убираем только префикс управляющей команды, сохраняя остальной текст пользователя
      payload = payload.replace(/^\s*\/(?:no_think|think)\b[ \t]*/i, "").trim();
      text += `<|im_start|>user\n${payload}<|im_end|>\n`;
    } else {
      text += `<|im_start|>assistant\n${clean}<|im_end|>\n`;
    }
  }

  // Открыть ассистента для ответа текущего шага
  text += `<|im_start|>assistant\n`;

  // Официальный способ жёстко отключить размышления — вставить пустой блок <think>...</think>
  if (control === "no_think") {
    text += `<think>\n\n</think>\n\n`;
  }

  return text;
}


