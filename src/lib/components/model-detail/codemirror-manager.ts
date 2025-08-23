/**
 * CodeMirror management for model descriptions
 */
import { getCodeMirrorRenderer } from '$lib/chat/codemirror-renderer';
import { enableExternalLinks } from '$lib/chat/external-links';

export interface CodeMirrorManager {
  setupCodeMirror(element: HTMLElement, description: string | undefined): void;
  cleanup(): void;
}

export function createCodeMirrorManager(): CodeMirrorManager {
  let codeMirrorRenderer: any;
  let isDescriptionWatched = false;

  function cleanupCodeMirror() {
    if (codeMirrorRenderer && isDescriptionWatched) {
      try {
        codeMirrorRenderer.stopWatching();
      } catch {}
    }
    isDescriptionWatched = false;
  }

  return {
    setupCodeMirror(element: HTMLElement, description: string | undefined) {
      if (element && description) {
        // Clean up previous CodeMirror setup
        cleanupCodeMirror();
        
        // Enable external links for the description content
        enableExternalLinks(element);
        
        try {
          if (!codeMirrorRenderer) {
            codeMirrorRenderer = getCodeMirrorRenderer();
          }
          codeMirrorRenderer.startWatching(element);
          isDescriptionWatched = true;
        } catch (error) {
          console.error('Failed to apply CodeMirror to model description:', error);
        }
      }
    },

    cleanup() {
      cleanupCodeMirror();
    }
  };
}