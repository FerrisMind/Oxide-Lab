/**
 * Icon management utilities for model detail components
 */
import { mount, unmount } from 'svelte';
import Robot from 'phosphor-svelte/lib/Robot';
import Download from 'phosphor-svelte/lib/Download';
import Heart from 'phosphor-svelte/lib/Heart';
import ArrowSquareOut from 'phosphor-svelte/lib/ArrowSquareOut';

export interface IconManager {
  mountEmptyIcon(element: HTMLElement): void;
  mountActionIcons(downloadEl: HTMLElement, heartEl: HTMLElement, hfEl: HTMLElement): void;
  cleanup(): void;
}

export function createIconManager(): IconManager {
  let robotIcon: any;
  let downloadIcon: any;
  let heartIcon: any;
  let huggingFaceIcon: any;

  function safeMountIcon(Component: any, target: HTMLElement, props: any, currentIcon?: any) {
    if (currentIcon) {
      try {
        unmount(currentIcon);
      } catch {}
    }
    return mount(Component, { target, props });
  }

  function safeUnmountIcon(icon: any) {
    if (icon) {
      try {
        unmount(icon);
      } catch {}
      return null;
    }
    return null;
  }

  return {
    mountEmptyIcon(element: HTMLElement) {
      robotIcon = safeMountIcon(Robot, element, { size: 64, weight: 'regular' }, robotIcon);
    },

    mountActionIcons(downloadEl: HTMLElement, heartEl: HTMLElement, hfEl: HTMLElement) {
      if (downloadEl) {
        downloadIcon = safeMountIcon(
          Download,
          downloadEl,
          { size: 16, weight: 'regular' },
          downloadIcon,
        );
      }
      if (heartEl) {
        heartIcon = safeMountIcon(Heart, heartEl, { size: 16, weight: 'regular' }, heartIcon);
      }
      if (hfEl) {
        huggingFaceIcon = safeMountIcon(
          ArrowSquareOut,
          hfEl,
          { size: 16, weight: 'regular' },
          huggingFaceIcon,
        );
      }
    },

    cleanup() {
      robotIcon = safeUnmountIcon(robotIcon);
      downloadIcon = safeUnmountIcon(downloadIcon);
      heartIcon = safeUnmountIcon(heartIcon);
      huggingFaceIcon = safeUnmountIcon(huggingFaceIcon);
    },
  };
}
