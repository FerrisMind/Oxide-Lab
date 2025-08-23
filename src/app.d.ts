declare module '@tauri-apps/plugin-dialog' {
  export interface DialogFilter { name: string; extensions: string[] }
  export interface OpenDialogOptions {
    title?: string; filters?: DialogFilter[]; defaultPath?: string; multiple?: boolean;
    directory?: boolean; recursive?: boolean; canCreateDirectories?: boolean;
  }
  export interface SaveDialogOptions { title?: string; filters?: DialogFilter[]; defaultPath?: string; canCreateDirectories?: boolean }
  export interface MessageDialogOptions { title?: string; kind?: 'info' | 'warning' | 'error'; okLabel?: string }
  export interface ConfirmDialogOptions { title?: string; kind?: 'info' | 'warning' | 'error'; okLabel?: string; cancelLabel?: string }
  export type OpenDialogReturn<T extends OpenDialogOptions> =
    T['directory'] extends true ? (T['multiple'] extends true ? string[] | null : string | null)
    : (T['multiple'] extends true ? string[] | null : string | null);

  export function open<T extends OpenDialogOptions>(options?: T): Promise<OpenDialogReturn<T>>;
  export function save(options?: SaveDialogOptions): Promise<string | null>;
  export function message(message: string, options?: string | MessageDialogOptions): Promise<void>;
  export function ask(message: string, options?: string | ConfirmDialogOptions): Promise<boolean>;
  export function confirm(message: string, options?: string | ConfirmDialogOptions): Promise<boolean>;
}

declare module '@tauri-apps/plugin-opener' {
  export function openUrl(url: string | URL, openWith?: string): Promise<void>;
  export function openPath(path: string, openWith?: string): Promise<void>;
  export function revealItemInDir(path: string): Promise<unknown>;
}

declare module '$lib/chat/Chat.svelte' {
  import type { SvelteComponentTyped } from 'svelte';
  export default class Chat extends SvelteComponentTyped<Record<string, never>> {}
}

declare module 'phosphor-svelte' {
  import type { SvelteComponentTyped } from 'svelte';
  export class GearSix extends SvelteComponentTyped<{ size?: number | string; weight?: string; style?: string }>{}
  export class UploadSimple extends SvelteComponentTyped<{ size?: number | string; weight?: string; style?: string }>{}
  export class PaperPlaneRight extends SvelteComponentTyped<{ size?: number | string; weight?: string; style?: string }>{}
  export class ChatsCircle extends SvelteComponentTyped<{ size?: number | string; weight?: string; style?: string }>{}
  export class ArrowSquareOut extends SvelteComponentTyped<{ size?: number | string; weight?: string; style?: string }>{}
}


