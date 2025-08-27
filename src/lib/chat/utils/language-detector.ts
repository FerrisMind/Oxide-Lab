/**
 * Detects programming language from code content
 * @param code The code content to analyze
 * @returns Detected language name or empty string
 */
export function detectLanguageFromContent(code: string): string {
  const trimmed = code.trim();

  // JavaScript/TypeScript patterns
  if (
    /^(import|export|const|let|var|function|class|\w+\s*=>)/.test(trimmed) ||
    /\.(js|ts|jsx|tsx)$/.test(trimmed) ||
    /console\.log|document\.|window\./.test(trimmed)
  ) {
    return 'javascript';
  }

  // Python patterns
  if (
    /^(def|class|import|from|if __name__|print\()/.test(trimmed) ||
    /\.py$/.test(trimmed) ||
    /:\s*$/.test(trimmed.split('\n')[0])
  ) {
    return 'python';
  }

  // HTML patterns
  if (/^<!DOCTYPE|^<html|^<\w+[^>]*>/.test(trimmed) || /<\/\w+>/.test(trimmed)) {
    return 'html';
  }

  // CSS patterns
  if (/^[.#]?\w+\s*\{/.test(trimmed) || /:\s*[^;]+;/.test(trimmed)) {
    return 'css';
  }

  // JSON patterns
  if (
    (trimmed.startsWith('{') && trimmed.endsWith('}')) ||
    (trimmed.startsWith('[') && trimmed.endsWith(']'))
  ) {
    try {
      JSON.parse(trimmed);
      return 'json';
    } catch {
      // Not valid JSON
    }
  }

  // SQL patterns
  if (
    /^(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP)\s+/i.test(trimmed) ||
    /\bFROM\s+\w+/i.test(trimmed)
  ) {
    return 'sql';
  }

  return '';
}

/**
 * Extracts language from element class names
 * @param codeElement The code element to analyze
 * @returns Extracted language name or empty string
 */
export function extractLanguageFromClassNames(codeElement: Element): string {
  // Check for language class (hljs-*, language-*, etc.)
  const classList = Array.from(codeElement.classList);

  for (const className of classList) {
    if (className.startsWith('hljs-')) {
      // Skip hljs utility classes
      continue;
    }
    if (className.startsWith('language-')) {
      return className.replace('language-', '');
    }
    if (className.startsWith('lang-')) {
      return className.replace('lang-', '');
    }
  }

  // Check parent pre element
  if (codeElement.parentElement) {
    const parentClassList = Array.from(codeElement.parentElement.classList);
    for (const className of parentClassList) {
      if (className.startsWith('language-')) {
        return className.replace('language-', '');
      }
    }
  }

  return '';
}
