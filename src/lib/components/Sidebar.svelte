<script lang="ts">
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

  import DownloadManagerModal from './DownloadManagerModal.svelte';

  const baseNavigationItems = [
    {
      id: 'chat',
      title: 'Чат с моделью',
      icon: ChatCircle,
      path: '/',
      description: 'Интерактивный чат с LLM',
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
      id: 'models',
      title: 'Менеджер моделей',
      icon: Database,
      path: '/models',
      description: 'Управление загруженными моделями',
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
    {#if experimentalFeatures.enabled}
      <button
        class="nav-item"
        title="Загрузки"
        aria-label="Загрузки"
        onclick={() => (showDownloadManager = true)}
      >
        <div class="nav-icon"><ArrowCircleDown size={20} weight="regular" /></div>
      </button>
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

  .nav-item:hover {
    background: var(--border-color, #e8e6e3);
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
    width: 20px;
    height: 20px;
  }

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
    justify-content: flex-end;
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
  }

  .close-btn:focus {
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
