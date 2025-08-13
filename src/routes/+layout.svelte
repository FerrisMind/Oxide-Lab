<script>
  import "../app.css";
  // Тема для highlight.js (легкая)
  import 'highlight.js/styles/github.css';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import Minus from "phosphor-svelte/lib/Minus";
  import ArrowsIn from "phosphor-svelte/lib/ArrowsIn";
  import ArrowsOut from "phosphor-svelte/lib/ArrowsOut";
  import X from "phosphor-svelte/lib/X";
  
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
    // Проверяем начальное состояние окна
    (async () => {
      const w = (await import('@tauri-apps/api/window')).getCurrentWindow();
      isMaximized = await w.isMaximized();
      
      // Слушаем изменения состояния окна
      const unlisten = await w.onResized(() => {
        w.isMaximized().then(maximized => {
          isMaximized = maximized;
        });
      });
      
      return () => unlisten();
    })();
  });
</script>

<div class="app-shell">
  <header class="app-header" data-tauri-drag-region>
    <button type="button" class="brand" onclick={goHome} title={appName} aria-label={appName}>
      <img src={appIcon} alt="App icon" class="brand-icon" />
      <span class="brand-title">{appName}</span>
    </button>
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

  <main class="app-main">
    <slot />
  </main>
</div>

<style>
  .app-header {
    position: sticky; top: 0; z-index: 100;
    background: var(--card); color: var(--text);
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 8px; border-bottom: 1px solid var(--border-color);
    box-shadow: 0 4px 20px rgba(0,0,0,0.06);
    height: 20px; /* Фиксированная высота для лучшего центрирования */
  }
  .brand { display: inline-flex; align-items: center; gap: 10px; cursor: default; background: transparent; border: none; padding: 4px 8px; border-radius: 8px; }
  .brand-icon { width: 20px; height: 20px; }
  .brand-title { font-weight: 700; letter-spacing: 0.3px; color: var(--text); opacity: 0.9; }
  .app-shell { height: 100dvh; min-height: 100dvh; display: flex; flex-direction: column; }
  .app-main { flex: 1 1 auto; min-height: 0; display: flex; overflow: hidden; }
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


