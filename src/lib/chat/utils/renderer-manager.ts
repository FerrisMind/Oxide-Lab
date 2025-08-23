import { CodeMirrorRenderer } from '../codemirror-renderer';

// Container-specific renderer management
const containerRenderers = new Map<HTMLElement, CodeMirrorRenderer>();

/**
 * Gets or creates a CodeMirrorRenderer for a container
 * @param container Container element (optional)
 * @returns CodeMirrorRenderer instance
 */
export function getCodeMirrorRenderer(container?: HTMLElement): CodeMirrorRenderer {
  if (!container) {
    // For backward compatibility, return a new renderer
    return new CodeMirrorRenderer();
  }
  
  let renderer = containerRenderers.get(container);
  if (!renderer) {
    renderer = new CodeMirrorRenderer();
    containerRenderers.set(container, renderer);
  }
  return renderer;
}

/**
 * Cleans up a renderer for a container
 * @param container Container element
 */
export function cleanupRenderer(container: HTMLElement): void {
  const renderer = containerRenderers.get(container);
  if (renderer) {
    renderer.destroy();
    containerRenderers.delete(container);
  }
}