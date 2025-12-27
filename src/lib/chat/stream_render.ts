/**
 * Stream Render Utilities
 * 
 * Utilities for rendering streaming content to the DOM.
 */

import type { Segment } from '$lib/chat/parser';

/**
 * Get the assistant bubble element by message index.
 */
export function getAssistantBubbleEl(index: number): HTMLElement | null {
    const selector = `[data-message-index="${index}"] [data-message-content]`;
    return document.querySelector(selector);
}

/**
 * Append parsed segments to the assistant bubble element.
 */
export function appendSegments(
    _index: number,
    el: HTMLElement,
    segments: Segment[],
    _isStreaming: boolean
): void {
    for (const seg of segments) {
        switch (seg.kind) {
            case 'text':
                // Append text content
                el.textContent = (el.textContent || '') + seg.data;
                break;
            case 'think_start':
                // Could add think block indicator
                break;
            case 'think_content':
                // Think content - could be rendered differently
                break;
            case 'think_end':
                // Think block ended
                break;
        }
    }
}

/**
 * Finalize streaming for a message.
 */
export function finalizeStreaming(_index: number): void {
    // Mark streaming as complete
    // Could trigger markdown rendering, etc.
}
