<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  // remove version display; add About modal instead
  import ChatCircle from 'phosphor-svelte/lib/ChatCircle';
  import Code from 'phosphor-svelte/lib/Code';
  import Database from 'phosphor-svelte/lib/Database';
  import MagnifyingGlass from 'phosphor-svelte/lib/MagnifyingGlass';
  import ChartLine from 'phosphor-svelte/lib/ChartLine';
  import Gear from 'phosphor-svelte/lib/Gear';
  import Info from 'phosphor-svelte/lib/Info';

  const navigationItems = [
    {
      id: 'chat',
      title: 'Чат с моделью',
      icon: ChatCircle,
      path: '/',
      description: 'Интерактивный чат с LLM'
    },
    {
      id: 'api',
      title: 'API работа',
      icon: Code,
      path: '/api',
      description: 'Работа с моделью через API'
    },
    {
      id: 'models',
      title: 'Менеджер моделей',
      icon: Database,
      path: '/models',
      description: 'Управление загруженными моделями'
    },
    {
      id: 'search',
      title: 'Поиск моделей',
      icon: MagnifyingGlass,
      path: '/search',
      description: 'Поиск в Hugging Face Hub'
    },
    {
      id: 'performance',
      title: 'Производительность',
      icon: ChartLine,
      path: '/performance',
      description: 'Мониторинг производительности и метрик'
    },
    {
      id: 'settings',
      title: 'Настройки',
      icon: Gear,
      path: '/settings',
      description: 'Настройки приложения'
    }
  ];

  let showAbout = false;
  function toggleAbout() { showAbout = !showAbout; }

  function navigateTo(path: string) {
    goto(path);
  }

  $: currentPath = $page.url.pathname;
</script>

<aside class="sidebar">
  <nav class="sidebar-nav">
    {#each navigationItems as item}
      <button
        class="nav-item"
        class:active={currentPath === item.path}
        on:click={() => navigateTo(item.path)}
        title={item.title}
      >
        <div class="nav-icon">
          <svelte:component this={item.icon} size={20} weight="regular" />
        </div>
      </button>
    {/each}

  </nav>
  <div class="sidebar-bottom">
    <button class="nav-item" title="О программе" aria-label="О программе" on:click={toggleAbout}>
      <div class="nav-icon"><Info size={20} weight="regular" /></div>
    </button>
  </div>
  {#if showAbout}
    <div class="about-modal" role="dialog" aria-modal="true" aria-labelledby="about-title" tabindex="-1" on:click={toggleAbout} on:keydown={(e) => { if (e.key === 'Escape') toggleAbout(); }}>
      <div class="about-content" role="document" on:mousedown|stopPropagation>
        <h2 id="about-title">О программе</h2>
        <p>Oxide Lab — настольное приложение для локального инференса LLM на Tauri + Svelte.</p>
        <div class="about-actions">
          <button class="close-btn" on:click={toggleAbout}>Закрыть</button>
        </div>
      </div>
    </div>
  {/if}
</aside>

<style>
  /* Sidebar core */
  .sidebar {
    width: 120px; /* expanded width */
    height: 100%;
    background: var(--card);
    border-right: 1px solid var(--border-color, #e8e6e3);
    display: flex;
    flex-direction: column;
    box-shadow: 2px 0 8px rgba(0, 0, 0, 0.05);
    z-index: 50;
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
    padding: 10px;
    border: none;
    background: transparent;
    border-radius: 10px;
     cursor: default;
    transition: all 0.2s ease;
    color: var(--text);
    min-height: 48px;
    position: relative;
    overflow: hidden;
  }
  
  .nav-item:hover { background: var(--border-color, #e8e6e3); }
  
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
  
  .nav-icon { display: flex; align-items: center; justify-content: center; color: inherit; }
  /* Выравниваем фактический размер SVG-иконок для一致ности */
  .nav-icon :global(svg) { width: 20px; height: 20px; }
  
  .sidebar-bottom {
    margin-top: auto;
    padding: 8px 6px;
    border-top: 1px solid var(--border-color, #e8e6e3);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  
  /* Modal */
  .about-modal {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 999;
  }
  .about-content {
    background: var(--card);
    color: var(--text);
    border: 1px solid var(--border-color, #e8e6e3);
    border-radius: 12px;
    padding: 16px;
    width: min(520px, calc(100vw - 32px));
    box-shadow: var(--shadow, 0 4px 20px rgb(0 0 0 / 0.06));
  }
  .about-actions { margin-top: 12px; display: flex; justify-content: flex-end; }
  .close-btn { background: var(--accent); color: #fff; }
  
  /* Dark mode */
  @media (prefers-color-scheme: dark) {
    .sidebar { background: #1a1a1a; border-right-color: #333; }
    .nav-item:hover { background: #333; }
    .nav-item.active { background: rgba(59, 130, 246, 0.12); border-color: rgba(59, 130, 246, 0.24); }
    .sidebar-footer { border-top-color: #333; }
    .about-content { background: #1a1a1a; border-color: #333; }
  }
</style>