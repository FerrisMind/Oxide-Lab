<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';
  // remove version display; add About modal instead
  import ChatCircle from 'phosphor-svelte/lib/ChatCircle';
  import Code from 'phosphor-svelte/lib/Code';
  import Database from 'phosphor-svelte/lib/Database';
  import ChartLine from 'phosphor-svelte/lib/ChartLine';
  import Gear from 'phosphor-svelte/lib/Gear';
  import Info from 'phosphor-svelte/lib/Info';
  import ArrowCircleDown from 'phosphor-svelte/lib/ArrowCircleDown';
  import GithubLogo from 'phosphor-svelte/lib/GithubLogo';
  import SidebarSimple from 'phosphor-svelte/lib/SidebarSimple';
  import { openUrl } from '@tauri-apps/plugin-opener';

  import DownloadManagerModal from './DownloadManagerModal.svelte';
  import {
    activeDownloads,
    downloadsLoaded,
    ensureDownloadManager,
  } from '$lib/stores/download-manager';

  const baseNavigationItems = [
    {
      id: 'chat',
      title: 'Чат с моделью',
      icon: ChatCircle,
      path: '/',
      description: 'Интерактивный чат с LLM',
    },
    {
      id: 'models',
      title: 'Менеджер моделей',
      icon: Database,
      path: '/models',
      description: 'Управление загруженными моделями',
    },
    {
      id: 'settings',
      title: 'Настройки',
      icon: Gear,
      path: '/settings',
      description: 'Настройки приложения',
    },
  ];

  const experimentalNavigationItems = [
    {
      id: 'api',
      title: 'API работа',
      icon: Code,
      path: '/api',
      description: 'Работа с моделью через API',
    },
    {
      id: 'performance',
      title: 'Производительность',
      icon: ChartLine,
      path: '/performance',
      description: 'Мониторинг производительности и метрик',
    },
  ];

  // Computed navigation items based on experimental features
  let navigationItems = $derived(
    experimentalFeatures.enabled
      ? [
          baseNavigationItems[0], // Чат с моделью
          ...experimentalNavigationItems, // Все экспериментальные функции
          baseNavigationItems[1], // Настройки
        ]
      : baseNavigationItems,
  );

  let showAbout = $state(false);
  let appVersion = $state('0.13.0');
  let showDownloadManager = $state(false);
  let hasActiveDownloads = $derived($downloadsLoaded && $activeDownloads.length > 0);
  let aggregateDownloadProgress = $derived(
    !$downloadsLoaded || !$activeDownloads.length
      ? null
      : (() => {
          let total = 0;
          let downloaded = 0;
          for (const job of $activeDownloads) {
            if (typeof job.total_bytes === 'number' && job.total_bytes > 0) {
              total += job.total_bytes;
              downloaded += Math.min(job.downloaded_bytes, job.total_bytes);
            } else {
              return null;
            }
          }
          if (total === 0) {
            return null;
          }
          return Math.min(1, downloaded / total);
        })(),
  );

  onMount(() => {
    void ensureDownloadManager();
  });

  function toggleAbout() {
    showAbout = !showAbout;
    if (showAbout) {
      loadAppVersion();
    }
  }

  function handleBackdropKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' || event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      toggleAbout();
    }
  }

  async function loadAppVersion() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const appInfo = (await invoke('get_app_info')) as { version: string };
      appVersion = appInfo.version;
    } catch (error) {
      console.warn('Не удалось получить версию приложения:', error);
      appVersion = '0.13.0'; // Fallback версия
    }
  }

  // Фокус-трап для модального окна
  let modalElement = $state<HTMLDivElement | null>(null);

  $effect(() => {
    if (!showAbout || !modalElement) return;

    const node = modalElement;
    const focusableElements = Array.from(
      node.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
      ),
    );
    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];

    const handleKeydown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        toggleAbout();
        return;
      }

      if (event.key !== 'Tab' || focusableElements.length === 0) {
        return;
      }

      const activeElement = document.activeElement as HTMLElement | null;

      if (event.shiftKey) {
        if (activeElement === firstElement) {
          event.preventDefault();
          lastElement?.focus();
        }
      } else if (activeElement === lastElement) {
        event.preventDefault();
        firstElement?.focus();
      }
    };

    node.addEventListener('keydown', handleKeydown);

    if (firstElement) {
      firstElement.focus();
    } else {
      node.focus();
    }

    return () => {
      node.removeEventListener('keydown', handleKeydown);
    };
  });

  function navigateTo(path: string) {
    goto(path);
  }

  let currentPath = $derived($page.url.pathname);
</script>

<aside class="sidebar">
  <div class="sidebar-top">
    <button class="brand-button" onclick={() => window.location.href = '/'} title="Домой">
      <img src="/icon.svg" alt="App icon" class="brand-icon brand-icon-default" />
      <div class="brand-icon brand-icon-hover">
        <SidebarSimple size={28} weight="regular" />
      </div>
    </button>
  </div>
  <nav class="sidebar-nav">
    {#each navigationItems as item}
      {@const Icon = item.icon}
      <button
        class="nav-item"
        class:active={currentPath === item.path}
        onclick={() => navigateTo(item.path)}
        title={item.title}
      >
        <div class="nav-icon">
          <Icon size={20} weight="regular" />
        </div>
      </button>
    {/each}
  </nav>
  <div class="sidebar-bottom">
    <button
      class="nav-item"
      title="Загрузки"
      aria-label="Загрузки"
      onclick={() => (showDownloadManager = true)}
    >
      <div class="nav-icon"><ArrowCircleDown size={20} weight="regular" /></div>
    </button>
    {#if !showDownloadManager && hasActiveDownloads}
      <div class="download-indicator">
        {#if aggregateDownloadProgress !== null}
          <div class="indicator-bar">
            <div class="indicator-fill" style={`width: ${aggregateDownloadProgress * 100}%`}></div>
          </div>
        {:else}
          <div class="indicator-bar indeterminate">
            <span></span>
          </div>
        {/if}
      </div>
    {/if}
    <button class="nav-item" title="О программе" aria-label="О программе" onclick={toggleAbout}>
      <div class="nav-icon"><Info size={20} weight="regular" /></div>
    </button>
  </div>
  {#if showAbout}
    <div
      bind:this={modalElement}
      class="about-modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="about-title"
      tabindex="-1"
      onclick={(event) => {
        if (event.target === event.currentTarget) {
          toggleAbout();
        }
      }}
      onkeydown={handleBackdropKeydown}
    >
      <div class="about-content" role="document">
        <h2 id="about-title">О программе</h2>
        <div class="about-info">
          <p>
            <strong>Oxide Lab</strong> — настольное приложение для локального инференса LLM на базе современных
            технологий.
          </p>
          <p><strong>Технологии:</strong> Tauri 2 + Svelte 5 + Rust + Candle ML</p>
          <p><strong>Версия:</strong> {appVersion}</p>
        </div>
        <div class="about-actions">
          <button
            class="github-btn"
            onclick={() => openUrl('https://github.com/FerrisMind/Oxide-Lab')}
            aria-label="Открыть GitHub репозиторий"
          >
            <GithubLogo size={16} />
            GitHub
          </button>
          <button
            class="gitverse-btn"
            onclick={() => openUrl('https://gitverse.ru/FerrisMind/Oxide-Lab')}
            aria-label="Открыть GitVerse репозиторий"
          >
            <svg width="16" height="16" viewBox="0 0 256 256" fill="currentColor">
              <path d="m127.94,245.01c59.01,0,106.84-47.84,106.84-106.84,0-13.35-2.45-26.12-6.92-37.91l1.17-12.24c7.5,15.11,11.73,32.14,11.73,50.15,0,62.31-50.51,112.83-112.83,112.83S15.1,200.49,15.1,138.17c0-49.87,32.35-92.17,77.2-107.09-.33,2.32-.87,6.31-.91,6.66-41.02,14.93-70.31,54.25-70.31,100.43,0,59.01,47.84,106.84,106.84,106.84h.02Zm-5.32-213.55c1.76-.09,3.54-.12,5.32-.12,18.5,0,35.92,4.7,51.1,12.99.99-1.48,2.3-3.42,3.37-4.98-16.15-8.92-34.72-14-54.47-14-2.87,0-5.72.1-8.54.32l3.21,5.8h.01Z" fill-rule="evenodd"/>
              <path d="m125.79,185.34c-12.15,1.1-23.13,2.11-31.34.46-27.18-5.48-47.39-26.77-47.39-26.77-.69-.47-.08-1.47.62-1.16,0,0,28.57,14.41,46.77,14.95,18.2.55,24.65-2.57,39.87-12.18,0,0-.47-.55-.77-.47-.63.17-1.41.48-2.39.87-5.26,2.06-16.09,6.31-36.7,3.09-17.76-2.78-34.36-10.88-42.11-15.1-2.9-1.58-4.65-4.61-4.74-7.93l-.15-5.04c0-.23.08-.39.23-.55l10.86-9.05c2.87-2.39,4.9-5.62,5.78-9.25l1.09-4.43c.83-3.36-1.77-6.58-5.23-6.5l-.84.02c-7.3.18-14.51-1.57-20.91-5.08l54.6-.74c2.12-.03,4.16-.8,5.77-2.16l29.13-24.7c6.26-5.32,14.21-8.23,22.42-8.23h14.71c2.44-3.46,5.15-7.62,8.04-12.06.84-1.29,5.51-10.26,6.38-11.59-14.21-7.65-35.39-12.26-52.67-12.26-.81,0-.5,7.94-1.3,7.96,3.37,7.28,6.32,14.95,8.51,21.37h-7.21c-8.53,0-16.83,2.79-23.63,7.95l-13.03,9.88c-.39.31-1.01,0-1.01-.47.34-5-4.92-10.58-4.7-15.75.24-5.56,1.94-21.8,1.94-21.8-37.28,14.78-67.47,57.01-67.47,99.56,0,39.49,24.13,76.71,57.22,93.23,12.74-5.2,22.26-13.14,28.48-18.32,10.03-8.35,19.54-16.27,38.58-20.69,18.37-4.26,33.46-3.97,44.62-2.21-19.18-8.75-42.18-6.65-62.04-4.84h.01Zm106.29-12.51c.79-5.09,4.87-29.76,4.87-35.07,0-7.65-2.08-21.66-3.7-28.82-.51,8.39-6.96,23.99-6.41,32.6.26,4.1,5.09,27.24,5.24,31.29Zm-41.27-113.42c7.17,5.73,13.54,12.41,18.91,19.87-1.63,7.27-3.12,12.64-3.12,12.64l-3.01-3.81c-5.47-6.94-11.84-13.12-18.95-18.37,0,0,2.53-4.71,6.15-10.33h.01Zm-59.75,33.35c-3.42,2.1-5.74,5.74-5.9,10.01-.31,6.9,5.04,12.72,11.95,13.03h.55c6.59,0,12.09-5.19,12.41-11.87.08-1.24-.08-2.4-.31-3.5-5.51-4.66-11.79-6.67-18.7-7.68Zm-37.49.59l32.08-25.68c2.29-1.81.93-2.92.32-2.92-6.29-.05-12.39,2.02-17.35,5.86l-30.81,23.88h12.53c1.18,0,2.33-.4,3.26-1.14h-.01Zm-58.78,32.61v6.17c0,5.92,1.94,11.67,5.51,16.4v-15.36l-5.51-7.21Zm0,0c0-.62-.23-.55,0,0Z" fill-rule="evenodd"/>
              <path d="m240.76,134.06c-.42-12.37-2.9-24.04-7-35.1v-.1c-1.39-3.72-2.97-7.36-4.73-10.89v.05l-1.17,12.24v.02c-.38,3.63-.77,7.2-1.18,10.67-1.16,9.92-.5,20.26.16,30.56.55,8.51,1.09,17,.6,25.2-.36,6.21-1.33,12.24-3.33,18.02-17.31,35.67-53.86,60.26-96.18,60.26-18.79,0-36.45-4.86-51.79-13.38,12.8-5.51,20.79-12.16,28.48-18.56,10.03-8.35,19.54-16.27,38.58-20.69,5.74-1.33,11.16-2.22,16.25-2.76,11.17-1.18,20.69-.66,28.37.55-8.92-4.07-18.65-5.8-28.53-6.29-11.35-.57-22.88.48-33.51,1.45-12.15,1.12-23.13,2.12-31.35.46-14.47-2.92-26.98-10.32-35.49-16.6-3.51-2.58-6.33-4.98-8.36-6.79-1.97-1.78-3.18-3-3.47-3.3-.02-.02-.03-.03-.05-.05l-.02-.02c-.69-.47-.08-1.47.61-1.16,0,0,.03.02.11.06.07.03.16.08.28.14.66.33,2.12,1.04,4.17,1.97,2.46,1.13,5.76,2.6,9.54,4.12,9.77,3.95,22.75,8.36,32.67,8.65,8.72.26,14.75-.32,20.33-2.05,5.36-1.66,10.33-4.4,16.88-8.46.68-.42,1.38-.87,2.1-1.31h.02c.18-.11.36-.23.55-.35,0,0-.47-.55-.77-.47-.62.17-1.41.48-2.39.87-3.3,1.3-8.78,3.45-17.49,4.08-5.19.38-11.52.22-19.22-.99-17.76-2.78-34.36-10.88-42.11-15.11-2.9-1.58-4.65-4.61-4.74-7.93l-.15-5.04c0-.23.08-.39.23-.55l10.86-9.05c2.87-2.38,4.9-5.63,5.78-9.25l1.09-4.43c.66-2.66-.83-5.24-3.2-6.15-.63-.24-1.31-.36-2.03-.35l-.84.02c-7.3.18-14.51-1.57-20.91-5.08l54.6-.74c2.12-.03,4.16-.8,5.77-2.16l29.13-24.7c6.26-5.32,14.21-8.23,22.42-8.23h14.71c2.44-3.46,5.15-7.62,8.04-12.06,1.89-2.92,3.87-5.96,5.92-9.01.89-1.32,2.02-3,3.02-4.45.11-.17.24-.34.35-.51-1.7-.93-3.42-1.83-5.17-2.69-1.84-.9-3.71-1.74-5.6-2.54-.05-.02-.1-.05-.15-.06-.85-.35-1.71-.71-2.57-1.04-.25-.1-.51-.19-.76-.28-.66-.25-1.33-.5-1.99-.74-.33-.12-.67-.24-1-.35-.59-.2-1.18-.41-1.78-.6-.39-.13-.76-.25-1.15-.38-.55-.17-1.1-.34-1.65-.51-.42-.13-.83-.25-1.25-.38-.52-.15-1.06-.3-1.58-.44-.44-.13-.89-.24-1.33-.36-.51-.14-1.04-.26-1.55-.39-.46-.11-.92-.23-1.38-.34-.5-.11-.99-.22-1.49-.33-.99-.22-1.98-.42-2.97-.6-.41-.08-.81-.16-1.22-.23-.57-.1-1.14-.19-1.71-.28-.42-.07-.84-.14-1.26-.2-.57-.08-1.13-.16-1.7-.24-.43-.06-.87-.11-1.31-.17-.56-.07-1.13-.14-1.68-.19-.46-.05-.91-.09-1.36-.14-.55-.06-1.1-.1-1.65-.15-.48-.03-.97-.07-1.45-.1-.52-.03-1.06-.07-1.59-.1-.55-.03-1.09-.05-1.65-.07-.47-.02-.93-.05-1.41-.06-.89-.02-1.79-.03-2.68-.03h-.39c-.69,0-1.38.02-2.06.03h0c-2.16.02-4.3.12-6.43.27l3.21,5.8c3.55,6.88,6.8,14.63,9.38,21.6.74,1.98,1.41,3.91,2.04,5.73h-7.21c-8.53,0-16.83,2.79-23.63,7.95l-13.03,9.88c-.39.31-1.01,0-1.01-.47.02-.24.03-.49.05-.73v-.06c.09-1.43.18-2.88.25-4.33.02-.38.03-.74.06-1.12.16-3.02.28-6.08.42-9.13.34-7.99.68-15.95,1.46-23.05.05-.35.58-4.33.91-6.66v-.02c-2.06.68-4.11,1.42-6.12,2.22v.06C44.6,49.9,15.17,90.56,15.17,138.12c0,38.09,18.88,71.77,47.79,92.2l6.1,4.02c17.15,10.52,37.33,16.6,58.93,16.6,57.96,0,105.71-43.72,112.09-99.97.07-.3.14-.6.2-.91.4-3.88.62-7.82.62-11.81,0-1.42-.03-2.84-.09-4.25l-.05.06Zm-132.15-63.44c3.13-2.43,6.73-4.15,10.53-5.07.69-.17,1.39-.31,2.1-.42.72-.11,1.43-.21,2.16-.26h.03c.84-.07,1.69-.1,2.54-.09.59,0,1.84,1,0,2.65,0,0-.01.01-.02.02-.09.08-.18.16-.3.25l-32.08,25.68c-.92.74-2.07,1.14-3.26,1.14h-12.53l30.81-23.88v-.02Zm-68.31,77.91c-2.85-3.76-4.66-8.19-5.27-12.82-.11-.84-.18-1.69-.22-2.53-.01-.34-.02-.69-.02-1.04v-6.17l5.51,7.21v15.36h0Zm5.73,2.79c.25.26.52.52.8.77-.27.02-.54.07-.8.14v-.9h0Z" />
              <path d="m203.6,88.1l3.01,3.82s.76-2.76,1.79-6.98c3.28-13.43,9.28-41.71,2.32-43.9-9.14-2.87-26.07,28.7-26.07,28.7,6.49,4.79,12.37,10.36,17.5,16.58.49.59.97,1.18,1.45,1.79Z" />
              <path d="m233.27,23.1c-1.84-6.83-5.11-13.72-11.49-16.58-5.72-2.55-11.46-1.64-16.49.83-4.9,2.4-9.51,6.43-13.71,10.95-8.36,8.98-16.42,21.37-23.1,31.65l-.16.25c-2.24,3.45-4.33,6.64-6.24,9.46h-11.71c-3.26,0-6.47.4-9.59,1.15l-1.32-3.87c-3.31-9.75-8.42-22.51-14.18-32.6-2.86-5-6.07-9.69-9.53-13.02-3.27-3.14-8.13-6.32-13.82-4.9-4.77,1.2-8.07,4.43-10.31,8.21-2.19,3.69-3.56,8.24-4.51,12.99-1.88,9.38-2.39,21.23-2.87,32.57v.34c-.24,5.19-.46,10.32-.8,15.22v.39c-.01,2.22,1.08,3.86,2.35,4.89l-18.13,14.05,31.16,3.22,29.13-24.7c6.26-5.32,14.21-8.23,22.42-8.23h14.71c2.44-3.46,5.15-7.62,8.04-12.06,13.57-20.86,31.4-48.25,46.32-41.58,15.44,6.9,11.9,59.6,7.23,99.21l5.68.67c2.33-19.86,4.43-43.26,3.96-62.68-.24-9.67-1.12-18.68-3.05-25.85Zm-107.3,44.31l-.02.02c-.09.08-.18.16-.3.25l-32.08,25.68c-.92.74-2.07,1.14-3.26,1.14h-12.53l30.81-23.88c3.13-2.43,6.73-4.15,10.53-5.07.69-.17,1.39-.31,2.1-.42.72-.11,1.44-.2,2.16-.26h.03c.84-.07,1.69-.1,2.54-.09.59,0,1.84,1,0,2.65v-.02Zm.87-8.61c-8.53,0-16.83,2.79-23.63,7.95l-13.03,9.88c-.39.31-1.01,0-1.01-.47.02-.24.03-.49.05-.73v-.06c.09-1.43.18-2.88.25-4.33.02-.38.05-.74.06-1.12.16-3.02.28-6.08.42-9.13.99-23.17,1.97-45.95,13.37-48.8,9.83-2.46,21.69,22.17,28.68,41.07.74,1.98,1.41,3.91,2.04,5.73h-7.21.01Z" />
            </svg>
            GitVerse
          </button>
          <button
            class="close-btn"
            onclick={(e) => {
              e.stopPropagation();
              toggleAbout();
            }}
            aria-label="Закрыть окно о программе"
          >
            Закрыть
          </button>
        </div>
      </div>
    </div>
  {/if}
  {#if showDownloadManager}
    <DownloadManagerModal on:close={() => (showDownloadManager = false)} />
  {/if}
</aside>

<style>
  /* Sidebar core */
  .sidebar {
    width: 48px; /* adjusted for 32px buttons */
    height: 100%;
    background: var(--card);
    border-right: 1px solid var(--border-color, #e8e6e3);
    display: flex;
    flex-direction: column;
    box-shadow: 2px 0 8px rgba(0, 0, 0, 0.05);
    z-index: 250;
    position: fixed;
    top: 0;
    left: 0;
  }

  .sidebar-top {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 44px;
    padding: 0 6px;
    border-bottom: 1px solid var(--border-color, #e8e6e3);
  }

  .brand-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 8px;
    transition: background 0.2s ease;
    padding: 0;
    position: relative;
  }

  .brand-button:hover {
    background: rgba(255, 255, 255, 0.1);
    transform: none;
  }

  .brand-icon {
    width: 28px;
    height: 28px;
    pointer-events: none;
    position: absolute;
    transition: opacity 0.2s ease;
  }

  .brand-icon-default {
    opacity: 1;
  }

  .brand-icon-hover {
    opacity: 0;
  }

  .brand-button:hover .brand-icon-default {
    opacity: 0;
  }

  .brand-button:hover .brand-icon-hover {
    opacity: 1;
  }

  /* remove collapsed variant */

  /* remove sidebar-header and toggle styles */

  .sidebar-nav {
    flex: 1;
    padding: 8px 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    justify-content: center; /* center icon */
    gap: 0;
    padding: 0;
    border: none;
    background: transparent;
    border-radius: 10px;
    cursor: default;
    transition: all 0.2s ease;
    color: var(--text);
    height: 44px;
    position: relative;
    overflow: hidden;
  }

  .nav-item:hover {
    background: var(--border-color, #e8e6e3);
    transform: none;
  }

  .nav-item.active {
    background: rgba(59, 130, 246, 0.06);
    border: 1px solid rgba(59, 130, 246, 0.12);
    color: #3b82f6;
  }

  .nav-item.active::after {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    height: 100%;
    width: 3px;
    background: #3b82f6;
  }

  .nav-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: inherit;
  }
  /* Выравниваем фактический размер SVG-иконок для一致ности */
  .nav-icon :global(svg) {
    width: 28px;
    height: 28px;
  }

  .sidebar-bottom {
    margin-top: auto;
    padding: 8px 6px;
    border-top: 1px solid var(--border-color, #e8e6e3);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .download-indicator {
    width: 100%;
    padding: 0 8px 4px;
  }

  .indicator-bar {
    position: relative;
    height: 4px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--accent, #3498db) 18%, transparent 82%);
    overflow: hidden;
  }

  .indicator-fill {
    height: 100%;
    background: var(--accent, #3498db);
    border-radius: inherit;
    transition: width 0.2s ease;
  }

  .indicator-bar.indeterminate span {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      90deg,
      transparent 0%,
      color-mix(in srgb, var(--accent, #3498db) 60%, transparent) 50%,
      transparent 100%
    );
    animation: sidebar-progress 1.2s linear infinite;
  }

  @keyframes sidebar-progress {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(100%);
    }
  }

  /* Modal */
  .about-modal {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000; /* Увеличиваем z-index для уверенности */
    backdrop-filter: blur(2px); /* Добавляем размытие фона */
  }
  .about-content {
    background: var(--card);
    color: var(--text);
    border: 1px solid var(--border-color, #e8e6e3);
    border-radius: 12px;
    padding: 16px;
    width: min(520px, calc(100vw - 32px));
    box-shadow: var(--shadow, 0 4px 20px rgb(0 0 0 / 0.06));
    pointer-events: auto;
  }
  .about-info {
    margin-top: 8px;
  }

  .about-info p {
    margin: 6px 0;
    line-height: 1.4;
  }

  .about-actions {
    margin-top: 16px;
    display: flex;
    justify-content: space-between;
  }
  .close-btn {
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: 6px;
    padding: 8px 16px;
    cursor: default;
    font-size: 14px;
    font-weight: 500;
    transition: background-color 0.2s ease;
    min-width: 80px;
  }

  .close-btn:hover {
    background: var(--accent-dark, #2563eb);
    transform: none;
  }

  .close-btn:focus {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .github-btn {
    background: transparent;
    color: var(--text);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 8px 16px;
    cursor: default;
    font-size: 14px;
    font-weight: 500;
    transition: all 0.2s ease;
    min-width: 80px;
    display: flex;
    align-items: center;
    gap: 8px;
    margin-right: 8px;
  }

  .github-btn:hover {
    background: var(--panel-bg);
    border-color: var(--accent);
    transform: none;
  }

  .github-btn:focus {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .gitverse-btn {
    background: transparent;
    color: var(--text);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 8px 16px;
    cursor: default;
    font-size: 14px;
    font-weight: 500;
    transition: all 0.2s ease;
    min-width: 80px;
    display: flex;
    align-items: center;
    gap: 8px;
    margin-right: 8px;
  }

  .gitverse-btn:hover {
    background: var(--panel-bg);
    border-color: var(--accent);
    transform: none;
  }

  .gitverse-btn:focus {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  /* Dark mode */
  @media (prefers-color-scheme: dark) {
    .sidebar {
      background: #1a1a1a;
      border-right-color: #333;
    }
    .nav-item:hover {
      background: #333;
    }
    .nav-item.active {
      background: rgba(59, 130, 246, 0.12);
      border-color: rgba(59, 130, 246, 0.24);
    }
    .about-content {
      background: #1a1a1a;
      border-color: #333;
    }
  }
</style>
