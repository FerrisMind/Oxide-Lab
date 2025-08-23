<script lang="ts">
  import "../app.css";
  import "$lib/chat/Chat.css";
  // Тема для highlight.js (легкая)
  import 'highlight.js/styles/github.css';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import Minus from "phosphor-svelte/lib/Minus";
  import ArrowsIn from "phosphor-svelte/lib/ArrowsIn";
  import ArrowsOut from "phosphor-svelte/lib/ArrowsOut";
  import X from "phosphor-svelte/lib/X";
  import Sidebar from '$lib/components/Sidebar.svelte';
  import GGUFUploadArea from '$lib/components/GGUFUploadArea.svelte';
  import HeaderSearch from '$lib/components/HeaderSearch.svelte';
  import { triggerHeaderSearch } from '$lib/stores/search';
  
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
  
  function goHome() { goto('/'); }
  
  async function toggleMaximize() {
    const w = (await import('@tauri-apps/api/window')).getCurrentWindow();
    if (await w.isMaximized()) { 
      await w.unmaximize(); 
      isMaximized = false;
    } else { 
      await w.maximize(); 
      isMaximized = true;
    }
  }
  
  onMount(() => {
    // Проверяем начальное состояние окна и слушаем resize через Tauri
    const unlistenHolder: { fn: UnlistenFn | null } = { fn: null };
    (async () => {
      const w = (await import('@tauri-apps/api/window')).getCurrentWindow();
      isMaximized = await w.isMaximized();

      // Слушаем изменения состояния окна
      unlistenHolder.fn = await w.onResized(() => {
        w.isMaximized().then(maximized => {
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
          const headerH = header ? header.getBoundingClientRect().height : 0;
          const targetH = wrapRect.height;

          // don't force loader height here — let CSS/flex handle sizing and allow loader internal scroll
        }
      } catch (e) {
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
  <header class="app-header" data-tauri-drag-region>
    <button class="brand" onclick={goHome} title="Домой" data-tauri-drag-region="false">
      <img src={appIcon} alt="App icon" class="brand-icon" />
      <span class="brand-title">{appName}</span>
    </button>
    <div class="header-center" data-tauri-drag-region="false">
      <!-- GGUF upload показывается только на вкладках чата и API работы -->
      {#if shouldShowGGUFUploadArea}
        <GGUFUploadArea />
      {/if}
      
      <!-- Поиск показывается только на вкладке поиска -->
      {#if shouldShowHeaderSearch}
        <HeaderSearch on:search={handleHeaderSearch} />
      {/if}
    </div>
    <div class="window-controls" data-tauri-drag-region="false">
      <button type="button" class="win-btn" title="Свернуть" onclick={() => import('@tauri-apps/api/window').then(m => m.getCurrentWindow().minimize())}>
        <Minus size={16} weight="bold" />
      </button>
      <button type="button" class="win-btn" title={isMaximized ? "Восстановить" : "Развернуть"} onclick={toggleMaximize}>
        {#if isMaximized}
          <ArrowsIn size={16} weight="bold" />
        {:else}
          <ArrowsOut size={16} weight="bold" />
        {/if}
      </button>
      <button type="button" class="win-btn close" title="Закрыть" onclick={() => import('@tauri-apps/api/window').then(m => m.getCurrentWindow().close())}>
        <X size={16} weight="bold" />
      </button>
    </div>
  </header>

  <div class="app-body">
    <Sidebar />
    <main class="app-main">
      <slot />
    </main>
  </div>
</div>

<style>
  .app-header {
    position: sticky; top: 0; z-index: 100;
    background: var(--card); color: var(--text);
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 8px; border-bottom: 1px solid var(--border-color);
    height: 56px; min-height: 56px; box-sizing: border-box; /* fixed header height */
    box-shadow: 0 4px 20px rgba(0,0,0,0.06);
  }
  .brand { display: inline-flex; align-items: center; gap: 10px; cursor: default; background: transparent; border: none; padding: 4px 8px; border-radius: 8px; }
  .brand-icon { width: 20px; height: 20px; }
  .brand-title { font-weight: 700; letter-spacing: 0.3px; color: var(--text); opacity: 0.9; }
  .header-center {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    max-width: 600px;
    margin: 0 auto;
  }
  .app-shell { height: 100dvh; min-height: 100dvh; display: flex; flex-direction: column; }
  .app-body { flex: 1 1 auto; min-height: 0; display: flex; overflow: hidden; }
  .app-main { flex: 1 1 auto; min-height: 0; display: flex; overflow: hidden; padding: var(--content-gap); padding-top: var(--content-gap-top); }
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


