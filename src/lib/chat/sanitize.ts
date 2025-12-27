/**
 * Prompt Sanitizer
 * 
 * Sanitizes user input for safe prompt generation.
 */

/**
 * Remove potentially harmful content from user prompts
 */
export function sanitizeForPrompt(text: string): string {
    if (!text) return '';

    // Remove any control characters except newlines and tabs
    let cleaned = text.replace(/[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/g, '');

    // Normalize excessive whitespace but preserve structure
    cleaned = cleaned.replace(/[\t ]+/g, ' ');
    cleaned = cleaned.replace(/\n{4,}/g, '\n\n\n');

    return cleaned.trim();
}
