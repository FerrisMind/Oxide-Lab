import type { ChatControllerCtx } from '$lib/chat/controller/types';
import { createActions } from '$lib/chat/controller/actions';

export function createChatController(ctx: ChatControllerCtx) {
  return createActions(ctx);
}
