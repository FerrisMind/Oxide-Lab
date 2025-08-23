import { mount, unmount, type ComponentConstructorOptions } from 'svelte';
import CodeMirror from '$lib/components/CodeMirror.svelte';
import Copy from 'phosphor-svelte/lib/Copy';
import Check from 'phosphor-svelte/lib/Check';

interface CodeBlock {
  element: HTMLElement;
  code: string;
  language: string;
  component?: any;
  iconComponent?: any;
}

export class CodeMirrorRenderer {
  private codeBlocks: Map<HTMLElement, CodeBlock> = new Map();
  private observer: MutationObserver | null = null;
  private isWatching: boolean = false;
  private container: HTMLElement | null = null;

  constructor() {
    // Initialize observer but don't start watching yet
    this.observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        mutation.addedNodes.forEach((node) => {
          if (node.nodeType === Node.ELEMENT_NODE) {
            this.processElement(node as HTMLElement);
          }
        });
        mutation.removedNodes.forEach((node) => {
          if (node.nodeType === Node.ELEMENT_NODE) {
            this.cleanupElement(node as HTMLElement);
          }
        });
      });
    });
  }

  public startWatching(container: HTMLElement) {
    // If already watching the same container, don't restart
    if (this.isWatching && this.container === container) {
      return;
    }
    
    // Stop watching previous container if any
    if (this.isWatching) {
      this.stopWatching();
    }
    
    this.container = container;
    this.isWatching = true;
    
    if (this.observer) {
      this.observer.observe(container, {
        childList: true,
        subtree: true,
      });
    }
    
    // Process existing elements
    this.processElement(container);
  }

  public stopWatching() {
    if (this.observer) {
      this.observer.disconnect();
    }
    this.isWatching = false;
    this.container = null;
    this.cleanup();
  }

  private processElement(element: HTMLElement) {
    // Find all pre > code elements (markdown code blocks)
    const codeElements = element.querySelectorAll('pre > code');
    
    codeElements.forEach((codeEl) => {
      const preEl = codeEl.parentElement as HTMLPreElement;
      if (!preEl || this.codeBlocks.has(preEl)) return;

      const code = codeEl.textContent || '';
      const language = this.extractLanguage(codeEl);
      
      // Only replace if it's a substantial code block (more than just inline code)
      if (code.trim().length > 10 || code.includes('\n')) {
        this.replaceWithCodeMirror(preEl, code, language);
      }
    });
  }

  private extractLanguage(codeElement: Element): string {
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

    // Try to detect language from content
    return this.detectLanguageFromContent(codeElement.textContent || '');
  }

  private detectLanguageFromContent(code: string): string {
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
    if (
      /^<!DOCTYPE|^<html|^<\w+[^>]*>/.test(trimmed) ||
      /<\/\w+>/.test(trimmed)
    ) {
      return 'html';
    }

    // CSS patterns
    if (
      /^[.#]?\w+\s*\{/.test(trimmed) ||
      /:\s*[^;]+;/.test(trimmed)
    ) {
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

  private replaceWithCodeMirror(preElement: HTMLPreElement, code: string, language: string) {
    // Create container for CodeMirror
    const container = document.createElement('div');
    container.className = 'codemirror-container';
    
    // Create copy button
    const toolbar = document.createElement('div');
    toolbar.className = 'codemirror-toolbar';
    
    const languageLabel = document.createElement('span');
    languageLabel.className = 'codemirror-language';
    languageLabel.textContent = language || 'text';
    
    const copyButton = document.createElement('button');
    copyButton.className = 'codemirror-copy-btn';
    copyButton.title = 'Copy code';
    
    // Create icon container
    const iconContainer = document.createElement('span');
    iconContainer.className = 'codemirror-copy-icon';
    copyButton.appendChild(iconContainer);
    
    // Mount the copy icon
    let currentIcon = mount(Copy, {
      target: iconContainer,
      props: { size: 16, weight: 'regular' }
    });
    
    copyButton.addEventListener('click', () => {
      navigator.clipboard.writeText(code).then(() => {
        // Replace with check icon
        if (currentIcon) {
          try { unmount(currentIcon); } catch {}
        }
        currentIcon = mount(Check, {
          target: iconContainer,
          props: { size: 16, weight: 'regular' }
        });
        
        setTimeout(() => {
          // Replace back with copy icon
          if (currentIcon) {
            try { unmount(currentIcon); } catch {}
          }
          currentIcon = mount(Copy, {
            target: iconContainer,
            props: { size: 16, weight: 'regular' }
          });
        }, 1000);
      }).catch(() => {
        // Fallback for older browsers
        const textArea = document.createElement('textarea');
        textArea.value = code;
        document.body.appendChild(textArea);
        textArea.select();
        document.execCommand('copy');
        document.body.removeChild(textArea);
        
        // Replace with check icon
        if (currentIcon) {
          try { unmount(currentIcon); } catch {}
        }
        currentIcon = mount(Check, {
          target: iconContainer,
          props: { size: 16, weight: 'regular' }
        });
        
        setTimeout(() => {
          // Replace back with copy icon
          if (currentIcon) {
            try { unmount(currentIcon); } catch {}
          }
          currentIcon = mount(Copy, {
            target: iconContainer,
            props: { size: 16, weight: 'regular' }
          });
        }, 1000);
      });
    });

    toolbar.appendChild(languageLabel);
    toolbar.appendChild(copyButton);
    
    const editorContainer = document.createElement('div');
    editorContainer.className = 'codemirror-editor';
    
    container.appendChild(toolbar);
    container.appendChild(editorContainer);
    
    // Replace the pre element
    preElement.parentNode?.replaceChild(container, preElement);

    // Mount CodeMirror component
    try {
      const component = mount(CodeMirror, {
        target: editorContainer,
        props: {
          code: code,
          language: language,
          readonly: true,
          theme: 'auto',
          showLineNumbers: true,
          wrap: true
        }
      });

      // Store reference for cleanup
      this.codeBlocks.set(container, {
        element: container,
        code,
        language,
        component,
        iconComponent: currentIcon
      });
    } catch (error) {
      console.error('Failed to mount CodeMirror component:', error);
      // Fallback to original pre element
      container.parentNode?.replaceChild(preElement, container);
    }
  }

  private cleanupElement(element: HTMLElement) {
    // Find and cleanup any CodeMirror components within the removed element
    this.codeBlocks.forEach((block, containerEl) => {
      if (containerEl === element || element.contains(containerEl)) {
        this.cleanupCodeBlock(containerEl);
      }
    });
  }

  private cleanupCodeBlock(container: HTMLElement) {
    const block = this.codeBlocks.get(container);
    if (block?.component) {
      try {
        unmount(block.component);
      } catch (error) {
        console.error('Failed to unmount CodeMirror component:', error);
      }
    }
    if (block?.iconComponent) {
      try {
        unmount(block.iconComponent);
      } catch (error) {
        console.error('Failed to unmount icon component:', error);
      }
    }
    this.codeBlocks.delete(container);
  }

  private cleanup() {
    this.codeBlocks.forEach((block, container) => {
      this.cleanupCodeBlock(container);
    });
    this.codeBlocks.clear();
  }

  public destroy() {
    this.stopWatching();
    if (this.observer) {
      this.observer.disconnect();
      this.observer = null;
    }
  }
}

// Container-specific renderer management
const containerRenderers = new Map<HTMLElement, CodeMirrorRenderer>();

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

export function cleanupRenderer(container: HTMLElement) {
  const renderer = containerRenderers.get(container);
  if (renderer) {
    renderer.destroy();
    containerRenderers.delete(container);
  }
}