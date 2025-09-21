// Web Worker for Shiki syntax highlighting
// This worker will handle syntax highlighting off the main thread to prevent UI blocking

let _highlighter: any = null;
const highlighterCache: Record<string, any> = {};

// Function to get a unique key for highlighter cache
function getHighlighterKey(lang: string, theme: string) {
  return `${lang}::${theme}`;
}

// Import Shiki directly in the worker
import { createHighlighter } from 'shiki/bundle/web';

// Function to ensure we have a highlighter for the specified language and theme
async function ensureHighlighter(lang: string, theme: string) {
  const key = getHighlighterKey(lang, theme);
  if (highlighterCache[key]) return highlighterCache[key];

  // Create highlighter with the specified language and theme
  const newHighlighter = await createHighlighter({ langs: [lang], themes: [theme] });
  highlighterCache[key] = newHighlighter;
  return newHighlighter;
}

// Handle messages from the main thread
self.onmessage = async (event) => {
  const { id, code, language, theme } = event.data;

  try {
    // Ensure we have a highlighter
    const shikiHighlighter = await ensureHighlighter(language, theme);

    // Perform syntax highlighting
    const html = shikiHighlighter.codeToHtml(code, { lang: language, theme });

    // Send result back to main thread
    self.postMessage({ id, html, error: null });
  } catch (error: any) {
    // Send error back to main thread
    self.postMessage({ id, html: null, error: error.message || 'Unknown error' });
  }
};

export {};
