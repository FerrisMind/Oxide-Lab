import { mount, unmount } from 'svelte';
import Copy from 'phosphor-svelte/lib/Copy';
import Check from 'phosphor-svelte/lib/Check';

/**
 * Creates a toolbar element for CodeMirror with language label and copy button
 * @param language The programming language
 * @param code The code content
 * @returns Toolbar element and copy button
 */
export function createCodeMirrorToolbar(
  language: string,
  code: string,
): {
  toolbar: HTMLDivElement;
  copyButton: HTMLButtonElement;
  languageLabel: HTMLSpanElement;
} {
  // Create toolbar container
  const toolbar = document.createElement('div');
  toolbar.className = 'codemirror-toolbar';

  // Create language label
  const languageLabel = document.createElement('span');
  languageLabel.className = 'codemirror-language';
  languageLabel.textContent = language || 'text';

  // Create copy button
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
    props: { size: 16, weight: 'regular' },
  });

  copyButton.addEventListener('click', () => {
    navigator.clipboard
      .writeText(code)
      .then(() => {
        // Replace with check icon
        if (currentIcon) {
          try {
            unmount(currentIcon);
          } catch {}
        }
        currentIcon = mount(Check, {
          target: iconContainer,
          props: { size: 16, weight: 'regular' },
        });

        setTimeout(() => {
          // Replace back with copy icon
          if (currentIcon) {
            try {
              unmount(currentIcon);
            } catch {}
          }
          currentIcon = mount(Copy, {
            target: iconContainer,
            props: { size: 16, weight: 'regular' },
          });
        }, 1000);
      })
      .catch(() => {
        // Fallback for older browsers
        const textArea = document.createElement('textarea');
        textArea.value = code;
        document.body.appendChild(textArea);
        textArea.select();
        document.execCommand('copy');
        document.body.removeChild(textArea);

        // Replace with check icon
        if (currentIcon) {
          try {
            unmount(currentIcon);
          } catch {}
        }
        currentIcon = mount(Check, {
          target: iconContainer,
          props: { size: 16, weight: 'regular' },
        });

        setTimeout(() => {
          // Replace back with copy icon
          if (currentIcon) {
            try {
              unmount(currentIcon);
            } catch {}
          }
          currentIcon = mount(Copy, {
            target: iconContainer,
            props: { size: 16, weight: 'regular' },
          });
        }, 1000);
      });
  });

  toolbar.appendChild(languageLabel);
  toolbar.appendChild(copyButton);

  return { toolbar, copyButton, languageLabel };
}

/**
 * Creates a container for CodeMirror component
 * @returns Container element
 */
export function createCodeMirrorContainer(): HTMLDivElement {
  const container = document.createElement('div');
  container.className = 'codemirror-container';
  return container;
}

/**
 * Creates an editor container for CodeMirror component
 * @returns Editor container element
 */
export function createEditorContainer(): HTMLDivElement {
  const editorContainer = document.createElement('div');
  editorContainer.className = 'codemirror-editor';
  return editorContainer;
}

/**
 * Copies text to clipboard with fallback for older browsers
 * @param text Text to copy
 */
export function copyToClipboard(text: string): void {
  navigator.clipboard.writeText(text).catch(() => {
    // Fallback for older browsers
    const textArea = document.createElement('textarea');
    textArea.value = text;
    document.body.appendChild(textArea);
    textArea.select();
    document.execCommand('copy');
    document.body.removeChild(textArea);
  });
}
