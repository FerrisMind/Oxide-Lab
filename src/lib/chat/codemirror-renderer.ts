import { mount, unmount } from 'svelte';
import { 
  detectLanguageFromContent, 
  extractLanguageFromClassNames 
} from './utils/language-detector';
import { 
  createCodeMirrorToolbar, 
  createCodeMirrorContainer, 
  createEditorContainer 
} from './utils/dom-utils';
import { 
  mountCodeMirrorComponent, 
  cleanupCodeBlock, 
  type CodeBlock 
} from './utils/component-manager';
import { getCodeMirrorRenderer, cleanupRenderer } from './utils/renderer-manager';

export type { CodeBlock };

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
    // Try to extract language from class names first
    const languageFromClass = extractLanguageFromClassNames(codeElement);
    if (languageFromClass) {
      return languageFromClass;
    }

    // Try to detect language from content
    return detectLanguageFromContent(codeElement.textContent || '');
  }

  private replaceWithCodeMirror(preElement: HTMLPreElement, code: string, language: string) {
    // Create container for CodeMirror
    const container = createCodeMirrorContainer();
    
    // Create toolbar with language label and copy button
    const { toolbar } = createCodeMirrorToolbar(language, code);
    
    const editorContainer = createEditorContainer();
    
    container.appendChild(toolbar);
    container.appendChild(editorContainer);
    
    // Replace the pre element
    preElement.parentNode?.replaceChild(container, preElement);

    // Mount CodeMirror component
    try {
      const component = mountCodeMirrorComponent(editorContainer, code, language);

      // Store reference for cleanup
      this.codeBlocks.set(container, {
        element: container,
        code,
        language,
        component
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
    if (block) {
      cleanupCodeBlock(block);
      this.codeBlocks.delete(container);
    }
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

// Export the renderer manager functions
export { getCodeMirrorRenderer, cleanupRenderer };