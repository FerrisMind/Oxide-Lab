<script lang="ts">
  import "../app.css";
  import "$lib/chat/Chat.css";
  // Тема для highlight.js (легкая)
  import 'highlight.js/styles/github.css';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Minus from "phosphor-svelte/lib/Minus";
  import ArrowsIn from "phosphor-svelte/lib/ArrowsIn";
  import ArrowsOut from "phosphor-svelte/lib/ArrowsOut";
  import X from "phosphor-svelte/lib/X";
  import Sidebar from '$lib/components/Sidebar.svelte';
  import GGUFUploadArea from '$lib/components/GGUFUploadArea.svelte';
  import HeaderSearch from '$lib/components/HeaderSearch.svelte';
  import { triggerHeaderSearch } from '$lib/stores/search';
  import { ensureGlobalChatStream } from '$lib/chat/global-stream';
  import Chat from '$lib/chat/Chat.svelte';
  
  // Определяем, должен ли отображаться GGUFUploadArea
  $: shouldShowGGUFUploadArea = $page.url.pathname === '/' || $page.url.pathname === '/api';
  
  // Определяем, должен ли отображаться HeaderSearch
  $: shouldShowHeaderSearch = $page.url.pathname === '/search';
  
  // Обработчик поиска из хедера
  function handleHeaderSearch(event: CustomEvent<{ query: string }>) {
    triggerHeaderSearch(event.detail.query);
  }
  
  const appName = 'Oxide Lab';
  const appIcon = '/icon.svg';
  let isMaximized = false;
  const appWindow = getCurrentWindow();
  
  function goHome() { goto('/'); }
  
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
    if (!target.closest('button, input, [data-tauri-drag-region="false"]')) {
      await appWindow.startDragging();
    }
  }
  
  onMount(() => {
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
      } catch (_e) {
        // ignore
      }
    };

    const debounced = () => { setTimeout(syncHeights, 50); };
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
        <span class="brand-title">{appName}</span>
      </button>
      <div class="header-center">
        <!-- GGUF upload: всегда смонтирован, скрывается классом -->
        <div class="gguf-host" class:hidden={!shouldShowGGUFUploadArea}>
          <GGUFUploadArea />
        </div>

        <!-- Поиск показывается только на вкладке поиска -->
        {#if shouldShowHeaderSearch}
          <HeaderSearch on:search={handleHeaderSearch} />
        {/if}
      </div>
      <div class="window-controls">
        <button type="button" class="win-btn" title="Свернуть" onclick={() => appWindow.minimize()}>
          <Minus size={16} weight="bold" />
        </button>
        <button type="button" class="win-btn" title={isMaximized ? "Восстановить" : "Развернуть"} onclick={toggleMaximize}>
          {#if isMaximized}
            <ArrowsIn size={16} weight="bold" />
          {:else}
            <ArrowsOut size={16} weight="bold" />
          {/if}
        </button>
        <button type="button" class="win-btn close" title="Закрыть" onclick={() => appWindow.close()}>
          <X size={16} weight="bold" />
        </button>
      </div>
    </header>
  </div>
  <div class="app-body">
    <Sidebar />
    <main class="app-main">
      <div class="view-switch">
        <div class="chat-host" class:hidden={$page.url.pathname !== '/'} aria-hidden={$page.url.pathname !== '/'}>
          <Chat />
        </div>
        <div class="route-host" class:hidden={$page.url.pathname === '/'} aria-hidden={$page.url.pathname === '/'}>
          <slot />
        </div>
      </div>
    </main>
  </div>
</div>

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
  
  .app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    height: 48px;
    background: var(--card);
    border-bottom: 1px solid var(--border-color);
    position: relative;
    z-index: 100;
  }
  .brand { 
    display: inline-flex; align-items: center; gap: 10px; 
    cursor: pointer; 
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
  .brand:hover, .brand:active, .brand:focus {
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
  .brand-title { 
    font-weight: 700; 
    letter-spacing: 0.3px; 
    color: var(--text) !important; 
    opacity: 0.9; 
    pointer-events: none;
  }
  .header-center {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    max-width: 600px;
    margin: 0 auto;
  }
  .gguf-host.hidden { display: none; }
  .app-body { flex: 1 1 auto; min-height: 0; display: flex; overflow: hidden; }
  .app-main { flex: 1 1 auto; min-height: 0; display: flex; overflow: hidden; padding: var(--content-gap); padding-top: var(--content-gap-top); }
  .view-switch { position: relative; display: flex; flex: 1 1 auto; min-height: 0; }
  .chat-host, .route-host { flex: 1 1 auto; min-height: 0; display: none; }
  .chat-host:not(.hidden), .route-host:not(.hidden) { display: flex; }
  /* shift chat content slightly left and give it full height under header */
  /* ensure main wrap fits under header */
  :global(main.wrap) { padding: var(--content-gap); height: 100%; min-height: 0; box-sizing: border-box; max-height: calc(100vh - 56px); overflow: auto; }
  /* ensure chat area fills available vertical space */
  :global(.chat) { height: 100%; min-height: 0; display: flex; flex-direction: column; }
  /* ensure sidebar keeps fixed width */
  :global(.sidebar) { width: 60px; min-width: 60px; max-width: 6000px; flex: 0 0 60px; }
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
  .win-btn:hover { background: #f0f0f0; color: #212121; }
  .win-btn.close:hover { background: #e81123; color: #212121; }
</style>


