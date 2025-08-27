import { mount, unmount } from 'svelte';
import CodeMirror from '$lib/components/CodeMirror.svelte';

/**
 * Interface for CodeBlock information
 */
export interface CodeBlock {
  element: HTMLElement;
  code: string;
  language: string;
  component?: any;
  iconComponent?: any;
}

/**
 * Mounts a CodeMirror component
 * @param target Target element to mount to
 * @param code Code content
 * @param language Programming language
 * @returns Mounted component instance
 */
export function mountCodeMirrorComponent(target: HTMLElement, code: string, language: string): any {
  return mount(CodeMirror, {
    target,
    props: {
      code: code,
      language: language,
      readonly: true,
      theme: 'auto',
      showLineNumbers: true,
      wrap: true,
    },
  });
}

/**
 * Unmounts a Svelte component safely
 * @param component Component to unmount
 */
export function safelyUnmountComponent(component: any): void {
  try {
    unmount(component);
  } catch (error) {
    console.error('Failed to unmount component:', error);
  }
}

/**
 * Cleans up a code block by unmounting its components
 * @param block CodeBlock to cleanup
 */
export function cleanupCodeBlock(block: CodeBlock): void {
  if (block.component) {
    safelyUnmountComponent(block.component);
  }
  if (block.iconComponent) {
    safelyUnmountComponent(block.iconComponent);
  }
}
