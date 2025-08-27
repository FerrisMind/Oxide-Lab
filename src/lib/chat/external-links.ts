import { openUrl } from '@tauri-apps/plugin-opener';

/**
 * Utility function to make all links in a container open in the system's default browser
 * instead of within the application.
 *
 * This function should be called after markdown content is rendered and inserted into the DOM.
 *
 * @param container - The HTML element containing the rendered markdown content
 */
export function enableExternalLinks(container: HTMLElement): void {
  if (!container) return;

  // Find all anchor tags with href attributes
  const links = container.querySelectorAll('a[href]') as NodeListOf<HTMLAnchorElement>;

  links.forEach((link) => {
    const href = link.getAttribute('href');
    if (!href) return;

    // Skip anchor links (internal page navigation)
    if (href.startsWith('#')) return;

    // Skip mailto links (they should be handled by the system)
    if (href.startsWith('mailto:')) {
      link.addEventListener('click', async (e) => {
        e.preventDefault();
        try {
          await openUrl(href);
        } catch (error) {
          console.error('Failed to open email link:', error);
        }
      });
      return;
    }

    // Handle all other external links (http, https, etc.)
    if (href.startsWith('http://') || href.startsWith('https://') || href.startsWith('ftp://')) {
      link.addEventListener('click', async (e) => {
        e.preventDefault();
        try {
          await openUrl(href);
        } catch (error) {
          console.error('Failed to open external link:', error);
        }
      });
      return;
    }

    // Handle relative URLs by converting them to absolute URLs
    // For model descriptions, these might be relative to the model repository
    if (!href.startsWith('/') && !href.includes('://')) {
      link.addEventListener('click', async (e) => {
        e.preventDefault();
        try {
          // Try to construct a full URL if possible
          const baseUrl = link.closest('[data-base-url]')?.getAttribute('data-base-url');
          const fullUrl = baseUrl ? new URL(href, baseUrl).toString() : href;
          await openUrl(fullUrl);
        } catch (error) {
          console.error('Failed to open relative link:', error);
        }
      });
    }
  });
}

/**
 * Enhanced version that also handles dynamically added content.
 * Uses MutationObserver to watch for new links being added to the container.
 *
 * @param container - The HTML element to watch for new markdown content
 * @returns A cleanup function to stop watching
 */
export function enableExternalLinksWithWatcher(container: HTMLElement): () => void {
  // Process existing links
  enableExternalLinks(container);

  // Watch for new links being added
  const observer = new MutationObserver((mutations) => {
    mutations.forEach((mutation) => {
      mutation.addedNodes.forEach((node) => {
        if (node.nodeType === Node.ELEMENT_NODE) {
          const element = node as HTMLElement;

          // If the added node is a link itself
          if (element.tagName === 'A' && element.hasAttribute('href')) {
            enableExternalLinks(element.parentElement || container);
          }

          // If the added node contains links
          const links = element.querySelectorAll?.('a[href]');
          if (links && links.length > 0) {
            enableExternalLinks(element);
          }
        }
      });
    });
  });

  observer.observe(container, {
    childList: true,
    subtree: true,
  });

  // Return cleanup function
  return () => {
    observer.disconnect();
  };
}
