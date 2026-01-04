// Follows the best practices established in https://shiki.matsu.io/guide/best-performance
import { createJavaScriptRegexEngine } from 'shiki/engine/javascript';
import { createHighlighterCore } from 'shiki/core';

/**
 * Bundled languages for Oxide Lab:
 * - Rust development (rust, toml)
 * - Web/docs (markdown, mdx, html, css, scss, javascript, typescript, tsx, jsx)
 * - Configs (yaml, json, ini, dockerfile, nginx)
 * - DevOps/CLI (bash, powershell, ansi, diff)
 * - Data/Query (sql, graphql, regex)
 * - Fallback (text for unknown languages)
 */
const bundledLanguages = {
  // Fallback & CLI output (ansi uses markdown as fallback since no dedicated module)
  text: () => import('@shikijs/langs/markdown'),
  ansi: () => import('@shikijs/langs/shellscript'),
  
  // Documentation
  markdown: () => import('@shikijs/langs/markdown'),
  mdx: () => import('@shikijs/langs/mdx'),
  
  // Rust ecosystem
  rust: () => import('@shikijs/langs/rust'),
  toml: () => import('@shikijs/langs/toml'),
  
  // Config files
  yaml: () => import('@shikijs/langs/yaml'),
  json: () => import('@shikijs/langs/json'),
  ini: () => import('@shikijs/langs/ini'),
  
  // Shell & DevOps
  bash: () => import('@shikijs/langs/bash'),
  powershell: () => import('@shikijs/langs/powershell'),
  dockerfile: () => import('@shikijs/langs/dockerfile'),
  nginx: () => import('@shikijs/langs/nginx'),
  
  // Web frontend
  javascript: () => import('@shikijs/langs/javascript'),
  typescript: () => import('@shikijs/langs/typescript'),
  tsx: () => import('@shikijs/langs/tsx'),
  jsx: () => import('@shikijs/langs/jsx'),
  html: () => import('@shikijs/langs/html'),
  css: () => import('@shikijs/langs/css'),
  scss: () => import('@shikijs/langs/scss'),
  
  // Data & Query
  sql: () => import('@shikijs/langs/sql'),
  graphql: () => import('@shikijs/langs/graphql'),
  regex: () => import('@shikijs/langs/regex'),
  
  // Misc
  diff: () => import('@shikijs/langs/diff'),
  svelte: () => import('@shikijs/langs/svelte'),
  python: () => import('@shikijs/langs/python'),
};

/** The languages configured for the highlighter */
export type SupportedLanguage = keyof typeof bundledLanguages;

/** A preloaded highlighter instance. */
export const highlighter = createHighlighterCore({
  themes: [
    import('@shikijs/themes/github-light-default'),
    import('@shikijs/themes/github-dark-default'),
    import('@shikijs/themes/vesper'),
  ],
  langs: Object.entries(bundledLanguages).map(([_, lang]) => lang),
  engine: createJavaScriptRegexEngine(),
});
