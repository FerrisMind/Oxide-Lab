/**
 * Stream Render Utilities
 *
 * Utilities for rendering streaming content to the DOM.
 */

/**
 * Get the assistant bubble element by message index.
 */
export function getAssistantBubbleEl(index: number): HTMLElement | null {
    const selector = `[data-message-index="${index}"] [data-message-content]`;
    return document.querySelector(selector);
}

/**
 * Finalize streaming for a message.
 */
export function finalizeStreaming(_index: number): void {
    // Mark streaming as complete
    // Could trigger markdown rendering, etc.
}
