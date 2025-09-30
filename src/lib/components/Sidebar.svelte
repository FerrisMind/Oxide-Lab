<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { getVersion } from '@tauri-apps/api/app';
  import { onMount } from 'svelte';
  import ChatCircle from 'phosphor-svelte/lib/ChatCircle';
  import Code from 'phosphor-svelte/lib/Code';
  import Database from 'phosphor-svelte/lib/Database';
  import MagnifyingGlass from 'phosphor-svelte/lib/MagnifyingGlass';
  import ChartLine from 'phosphor-svelte/lib/ChartLine';
  import Gear from 'phosphor-svelte/lib/Gear';

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

  let appVersion = ''; // Will be loaded from Tauri config

  onMount(async () => {
    try {
      appVersion = await getVersion();
    } catch (error) {
      console.error('Failed to get app version:', error);
      appVersion = 'Unknown'; // Fallback if version can't be fetched
    }
  });

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

  <div class="sidebar-footer">
    <div class="footer-info">
      <small>v{appVersion}</small>
    </div>
  </div>
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
  

  
  .sidebar-footer {
    padding: 8px 6px;
    border-top: 1px solid var(--border-color, #e8e6e3);
  }
  .footer-info { text-align: center; color: var(--muted, #6d6a6a); }
  .footer-info small { font-size: 10px; }
  
  /* Dark mode */
  @media (prefers-color-scheme: dark) {
    .sidebar { background: #1a1a1a; border-right-color: #333; }
    .nav-item:hover { background: #333; }
    .nav-item.active { background: rgba(59, 130, 246, 0.12); border-color: rgba(59, 130, 246, 0.24); }
    .sidebar-footer { border-top-color: #333; }
  }
</style>