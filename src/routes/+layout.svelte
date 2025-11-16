<script lang="ts">
  import '../app.css';
  import '$lib/chat/Chat.css';
  // Тема для highlight.js (легкая)
  import 'highlight.js/styles/github.css';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Minus from 'phosphor-svelte/lib/Minus';
  import ArrowsIn from 'phosphor-svelte/lib/ArrowsIn';
  import ArrowsOut from 'phosphor-svelte/lib/ArrowsOut';
  import X from 'phosphor-svelte/lib/X';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import ChatHistory from '$lib/components/ChatHistory.svelte';
  import GGUFUploadArea from '$lib/components/GGUFUploadArea.svelte';
  import { ensureGlobalChatStream } from '$lib/chat/global-stream';
  import Chat from '$lib/chat/Chat.svelte';
  import { showChatHistory } from '$lib/stores/sidebar';

  import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';
  import { pageTabsList, activePageTab } from '$lib/stores/page-tabs.svelte';

  // Импортируем все страницы для постоянного монтирования
  import ApiPage from './api/+page.svelte';
  import ModelsPage from './models/+page.svelte';
  import PerformancePage from './performance/+page.svelte';
  import SettingsPage from './settings/+page.svelte';

  const { children } = $props();

  // Определяем, должен ли отображаться GGUFUploadArea
  let shouldShowGGUFUploadArea = $derived(
    $page.url.pathname === '/' || ($page.url.pathname === '/api' && experimentalFeatures.enabled),
  );

  // Redirect to home if trying to access experimental pages when experimental features are disabled
  $effect(() => {
    if (experimentalFeatures.initialized && !experimentalFeatures.enabled) {
      const experimentalPaths = ['/api', '/performance'];
      if (experimentalPaths.includes($page.url.pathname)) {
        goto('/');
      }
    }
  });

  const appIcon = '/icon.svg';
  let isMaximized = $state(false);
  const appWindow = getCurrentWindow();

  function goHome() {
    goto('/');
  }

  async function toggleMaximize() {
    if (await appWindow.isMaximized()) {
      await appWindow.unmaximize();
      isMaximized = false;
    } else {
      await appWindow.maximize();
      isMaximized = true;
    }
  }

  // Make the entire header draggable
  async function startDragging(event: MouseEvent) {
    // Only start dragging if we're not clicking on an interactive element
    const target = event.target as HTMLElement;
    // Проверяем, не кликнули ли мы на кнопку, input, или элемент внутри них
    if (target.closest('button, input, textarea, select, a, [data-no-drag]')) {
      event.stopPropagation();
      return;
    }
    await appWindow.startDragging();
  }

  onMount(() => {
    // Initialize experimental features store
    void experimentalFeatures.loadState();

    // Ensure background chat stream listener is active across pages
    void ensureGlobalChatStream();
    // Проверяем начальное состояние окна и слушаем resize через Tauri
    const unlistenHolder: { fn: UnlistenFn | null } = { fn: null };
    (async () => {
      isMaximized = await appWindow.isMaximized();

      // Слушаем изменения состояния окна
      unlistenHolder.fn = await appWindow.onResized(() => {
        appWindow.isMaximized().then((maximized: boolean) => {
          isMaximized = maximized;
        });
      });
    })();

    // Sync heights between chat and loader panels to avoid visual mismatch
    const syncHeights = () => {
      try {
        const wrap = document.querySelector('main.wrap');
        if (!wrap) return;
        const chat = wrap.querySelector('.chat');
        const loader = wrap.querySelector('.loader');
        if (chat && loader) {
          // reset loader explicit height first
          if (loader instanceof HTMLElement) {
            loader.style.height = '';
            loader.style.minHeight = '';
          }

          // compute available height inside wrap
          const wrapRect = wrap.getBoundingClientRect();
          const header = document.querySelector('.app-header');
          const _headerH = header ? header.getBoundingClientRect().height : 0;
          const _targetH = wrapRect.height;

          // don't force loader height here — let CSS/flex handle sizing and allow loader internal scroll
        }
      } catch {
        // ignore
      }
    };

    const debounced = () => {
      setTimeout(syncHeights, 50);
    };
    window.addEventListener('resize', debounced);
    const ro = new ResizeObserver(debounced);
    const wrapEl = document.querySelector('main.wrap');
    if (wrapEl) ro.observe(wrapEl);
    // initial sync
    setTimeout(syncHeights, 120);

    return () => {
      if (unlistenHolder.fn) unlistenHolder.fn();
      window.removeEventListener('resize', debounced);
      ro.disconnect();
    };
  });
</script>

<div class="app-shell">
  <div class="app-header-wrapper" onmousedown={startDragging} role="toolbar" tabindex="0">
    <header class="app-header">
      <button class="brand" onclick={goHome} title="Домой">
        <img src={appIcon} alt="App icon" class="brand-icon" />
      </button>
      <div class="header-center">
        <!-- Page tabs -->
        {#if $pageTabsList.length > 0}
          <nav class="page-tabs" aria-label="Вкладки страницы">
            {#each $pageTabsList as tab}
              <button
                class="page-tab"
                class:active={$activePageTab === tab.id}
                onclick={() => activePageTab.set(tab.id)}
                aria-current={$activePageTab === tab.id ? 'page' : undefined}
              >
                {tab.label}
              </button>
            {/each}
          </nav>
        {/if}
        
        <!-- GGUF upload: всегда смонтирован, скрывается классом -->
        <div class="gguf-host" class:hidden={!shouldShowGGUFUploadArea}>
          <GGUFUploadArea />
        </div>
      </div>
      <div class="window-controls">
        <button type="button" class="win-btn" title="Свернуть" onclick={() => appWindow.minimize()}>
          <Minus size={16} weight="bold" />
        </button>
        <button
          type="button"
          class="win-btn"
          title={isMaximized ? 'Восстановить' : 'Развернуть'}
          onclick={toggleMaximize}
        >
          {#if isMaximized}
            <ArrowsIn size={16} weight="bold" />
          {:else}
            <ArrowsOut size={16} weight="bold" />
          {/if}
        </button>
        <button
          type="button"
          class="win-btn close"
          title="Закрыть"
          onclick={() => appWindow.close()}
        >
          <X size={16} weight="bold" />
        </button>
      </div>
    </header>
  </div>
  <div class="app-body">
    <Sidebar />
    {#if $showChatHistory && $page.url.pathname === '/'}
      <div class="chat-history-panel">
        <ChatHistory />
      </div>
    {/if}
    <main class="app-main">
      <div class="view-switch">
        <!-- Все страницы постоянно смонтированы, переключение через CSS -->
        <div class="page-container" class:active={$page.url.pathname === '/'}>
          <Chat />
        </div>
        {#if experimentalFeatures.enabled}
          <div class="page-container" class:active={$page.url.pathname === '/api'}>
            <ApiPage />
          </div>
          <div class="page-container" class:active={$page.url.pathname === '/performance'}>
            <PerformancePage />
          </div>
        {/if}
        <div class="page-container" class:active={$page.url.pathname === '/models'}>
          <ModelsPage />
        </div>
        <div class="page-container" class:active={$page.url.pathname === '/settings'}>
          <SettingsPage />
        </div>
      </div>
    </main>
  </div>
</div>

<!-- SvelteKit layout must expose a slot; hide it to avoid duplicate rendering -->
<div hidden>{@render children()}</div>

<style>
  /* App shell & header */
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .app-header-wrapper {
    position: relative;
    -webkit-app-region: drag; /* Enable window dragging */
  }

  /* Disable dragging on interactive elements */
  .app-header-wrapper button,
  .app-header-wrapper .gguf-host {
    -webkit-app-region: no-drag;
  }

  .app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    height: 48px;
    background: #1a1a1a;
    border-bottom: 1px solid var(--border-color);
    position: relative;
    z-index: 100;
  }
  .brand {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    cursor: default;
    background: transparent !important;
    border: none !important;
    padding: 4px 8px;
    border-radius: 8px;
    transition: none !important;
    color: var(--text) !important;
    outline: none !important;
    box-shadow: none !important;
    transform: none !important;
  }
  .brand:hover,
  .brand:active,
  .brand:focus {
    background: transparent !important;
    color: var(--text) !important;
    transform: none !important;
    box-shadow: none !important;
    outline: none !important;
  }
  .brand-icon {
    width: 20px;
    height: 20px;
    pointer-events: none;
  }
  .header-center {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1rem;
    max-width: 600px;
    margin: 0 auto;
  }

  .page-tabs {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .page-tab {
    padding: 0.4rem 0.9rem;
    border: none;
    background: transparent;
    color: var(--text);
    cursor: default;
    font-size: 0.9rem;
    border-radius: 8px;
    transition: background 0.2s ease;
    -webkit-app-region: no-drag;
  }

  .page-tab:hover {
    background: rgba(255, 255, 255, 0.1);
    transform: none;
  }

  .page-tab.active {
    background: rgba(255, 255, 255, 0.2);
    font-weight: 600;
    transform: none;
  }
  .gguf-host.hidden {
    display: none;
  }
  .app-body {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    overflow: hidden;
  }
  .app-main {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    overflow: hidden;
    padding: var(--content-gap);
    padding-top: var(--content-gap-top);
  }

  /* Переключение страниц через CSS - все смонтированы одновременно */
  .view-switch {
    position: relative;
    display: flex;
    flex: 1 1 auto;
    min-height: 0;
    width: 100%;
    height: 100%;
  }

  /* Все страницы смонтированы, но скрыты */
  .page-container {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    opacity: 0;
    visibility: hidden;
    pointer-events: none;
    transition:
      opacity 0.15s ease-in-out,
      visibility 0.15s ease-in-out;
    overflow: auto;
  }

  /* Активная страница видима */
  .page-container.active {
    opacity: 1;
    visibility: visible;
    pointer-events: auto;
    z-index: 1;
  }

  /* Адаптивность контента страниц */
  .page-container :global(> *) {
    width: 100%;
    max-width: 100%;
    min-height: 0;
  }
  /* Гарантируем, что корневой main внутри страницы тянется и является flex-контейнером */
  .page-container :global(main) {
    display: flex;
    flex-direction: column;
    flex: 1 1 auto;
    min-width: 0;
    min-height: 0;
    width: 100%;
    max-width: none; /* переопределяем ограничение из медиазапроса */
    margin: 0; /* убираем центрирование при больших ширинах */
    box-sizing: border-box;
  }
  /* Секция внутри main занимает доступное пространство и всю ширину */
  .page-container :global(main > section) {
    flex: 1 1 auto;
    min-width: 0;
    min-height: 0;
    width: 100%;
    box-sizing: border-box;
  }
  /* shift chat content slightly left and give it full height under header */
  /* ensure main wrap fits under header */
  :global(main.wrap) {
    padding: var(--content-gap);
    height: 100%;
    min-height: 0;
    box-sizing: border-box;
    max-height: calc(100vh - 56px);
    overflow: auto;
  }
  /* ensure chat area fills available vertical space */
  :global(.chat) {
    height: 100%;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  /* ensure sidebar keeps fixed width */
  :global(.sidebar) {
    width: 60px;
    min-width: 60px;
    max-width: 6000px;
    flex: 0 0 60px;
  }
  .window-controls {
    display: inline-flex;
    gap: 2px;
    align-items: center;
  }
  .win-btn {
    width: 36px;
    height: 28px;
    padding: 0;
    background: transparent;
    border: none;
    color: var(--text);
    opacity: 0.8;
    cursor: default;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: all 0.2s ease;
  }
  .win-btn:hover {
    background: #f0f0f0;
    color: #212121;
  }
  .win-btn.close:hover {
    background: #e81123;
    color: #212121;
  }

  /* История чатов */
  .chat-history-panel {
    width: 360px;
    min-width: 360px;
    max-width: 500px;
    height: calc(100vh - 80px);
    background: var(--card);
    overflow-y: auto;
    flex-shrink: 0;
    margin-left: 16px;
    margin-top: 16px;
    margin-bottom: 16px;
    border-radius: 14px;
    box-shadow: 0 6px 30px rgb(0 0 0 / 0.05);
  }

  /* Адаптивность для мобильных и планшетов */
  @media (max-width: 1024px) {
    .chat-history-panel {
      width: 300px;
      min-width: 300px;
      margin-left: 8px;
    }
  }

  @media (max-width: 768px) {
    .app-main {
      padding: 0.5rem;
      padding-top: 0.5rem;
    }

    .chat-history-panel {
      position: absolute;
      left: 0;
      top: 0;
      width: 280px;
      min-width: 280px;
      height: 100%;
      margin: 0;
      z-index: 100;
      box-shadow: 2px 0 10px rgba(0, 0, 0, 0.1);
    }

    .page-container {
      padding: 0.5rem;
    }
  }

  @media (max-width: 480px) {
    .app-main {
      padding: 0.25rem;
      padding-top: 0.25rem;
    }

    .page-container {
      padding: 0.25rem;
    }

    .chat-history-panel {
      width: 100%;
      min-width: 100%;
    }
  }

  /* Адаптивность контента для всех размеров */
  @media (min-width: 1440px) {
    .page-container :global(> *) {
      max-width: 1400px;
      margin: 0 auto;
    }
  }

  @media (prefers-color-scheme: dark) {
    .chat-history-panel {
      background: #1a1a1a;
    }
  }
</style>
