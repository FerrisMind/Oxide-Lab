<script lang="ts">
  import '../app.css';
  import '$lib/chat/Chat.css';
  // Тема для highlight.js (легкая)
  import 'highlight.js/styles/github.css';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { onMount, tick } from 'svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Minus from 'phosphor-svelte/lib/Minus';
  import ArrowsIn from 'phosphor-svelte/lib/ArrowsIn';
  import ArrowsOut from 'phosphor-svelte/lib/ArrowsOut';
  import X from 'phosphor-svelte/lib/X';
  import CaretDown from 'phosphor-svelte/lib/CaretDown';
  import Check from 'phosphor-svelte/lib/Check';
  import AppSidebar from '$lib/components/app-sidebar.svelte';
  import ChatHistory from '$lib/components/ChatHistory.svelte';
  import { ensureGlobalChatStream } from '$lib/chat/global-stream';
  import Chat from '$lib/chat/Chat.svelte';
  import { showChatHistory } from '$lib/stores/sidebar';
  import DownloadManagerModal from '$lib/components/DownloadManagerModal.svelte';
  import * as SidebarUI from '$lib/components/ui/sidebar/index';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import GithubLogo from 'phosphor-svelte/lib/GithubLogo';

import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';
import { pageTabsList, activePageTab } from '$lib/stores/page-tabs.svelte';
import type { TabId } from '$lib/stores/page-tabs.svelte';
  import { chatState } from '$lib/stores/chat';
  import { folderPath, models, scanFolder } from '$lib/stores/local-models';
  import type { ModelInfo } from '$lib/types/local-models';
  import { derived } from 'svelte/store';
  import ArrowClockwise from 'phosphor-svelte/lib/ArrowClockwise';
  import { modelSelectorSearchEnabled } from '$lib/stores/ui-preferences';
  import { Button } from '$lib/components/ui/button';
  import * as Popover from '$lib/components/ui/popover';
  import * as Command from '$lib/components/ui/command';
import * as Tabs from '$lib/components/ui/tabs';
  import { cn } from '$lib/utils';

  // Импортируем все страницы для постоянного монтирования
  import ApiPage from './api/+page.svelte';
  import ModelsPage from './models/+page.svelte';
  import PerformancePage from './performance/+page.svelte';
  import SettingsPage from './settings/+page.svelte';

  const { children } = $props();

  // Redirect to home if trying to access experimental pages when experimental features are disabled
  $effect(() => {
    if (experimentalFeatures.initialized && !experimentalFeatures.enabled) {
      const experimentalPaths = ['/api', '/performance'];
      if (experimentalPaths.includes($page.url.pathname)) {
        goto('/');
      }
    }
  });

  const _appIcon = '/icon.svg';
  let isMaximized = $state(false);
  const appWindow = getCurrentWindow();
  let isModelPickerOpen = $state(false);
  let comboboxTrigger = $state<HTMLButtonElement | null>(null);
  let showDownloadManager = $state(false);
  let showAbout = $state(false);
  let appVersion = $state('0.13.0');
  let modalElement = $state<HTMLDivElement | null>(null);
  let isSidebarOpen = $state(false);
  const quickModels = derived(models, ($models) =>
    $models.filter(
      (model: ModelInfo) =>
        model.candle_compatible && Boolean(model.source_repo_name?.trim() || model.name?.trim()),
    ),
  );
  const currentModelPath = derived(chatState, ($chatState) => $chatState.modelPath);
  const pendingModelPath = derived(chatState, ($chatState) => $chatState.pendingModelPath);
  const currentModel = derived(
    [quickModels, currentModelPath],
    ([$quickModels, $currentModelPath]) =>
      $quickModels.find((model: ModelInfo) => model.path === $currentModelPath),
  );
  const currentDisplayName = derived(currentModel, ($currentModel) =>
    formatModelLabel($currentModel),
  );
  const isReloadAvailable = derived([pendingModelPath, currentModelPath], ([$pending, $current]) =>
    Boolean($pending && $pending !== $current),
  );
  const modelSearchEnabled = derived(modelSelectorSearchEnabled, ($value) => $value);

  function currentHeaderTabValue(): TabId | '' {
    return $activePageTab || $pageTabsList[0]?.id || '';
  }

  function handleHeaderTabsChange(nextValue: string) {
    if (!nextValue || nextValue === $activePageTab) {
      return;
    }
    activePageTab.set(nextValue as TabId);
  }

  function formatModelLabel(model: ModelInfo | null | undefined) {
    if (!model) return 'Выберите модель';
    const publisher = model.metadata?.author ?? model.source_repo_id?.split('/')[0] ?? 'local';
    const title = model.name ?? model.source_repo_name ?? 'Без имени';
    return `${publisher}/${title}`;
  }

  function _goHome() {
    goto('/');
  }

  function handleReloadModel() {
    const ox = (window as any).__oxide;
    if (ox?.reloadSelectedModel) {
      ox.reloadSelectedModel();
    }
  }

  function closeModelPicker() {
    isModelPickerOpen = false;
    void tick().then(() => comboboxTrigger?.focus());
  }

  function handleSelectModel(model: ModelInfo) {
    const ox = (window as any).__oxide;
    if (!ox?.loadModelFromManager) {
      console.warn('Модель пока не готова к загрузке; попробуйте позже.');
      closeModelPicker();
      return;
    }
    ox.loadModelFromManager({
      path: model.path,
      format: model.format === 'gguf' ? 'gguf' : 'local_safetensors',
    });
    closeModelPicker();
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

  function toggleAbout() {
    showAbout = !showAbout;
    if (showAbout) {
      void loadAppVersion();
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
      appVersion = '0.13.0';
    }
  }

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

    if ($folderPath) {
      void scanFolder($folderPath).catch((err) =>
        console.warn('Не удалось просканировать локальные модели при старте', err),
      );
    }

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

<SidebarUI.Provider bind:open={isSidebarOpen}>
  <AppSidebar on:openDownloads={() => (showDownloadManager = true)} on:openAbout={toggleAbout} />
  <SidebarUI.Inset class="app-shell">
    <div class="app-header-wrapper" onmousedown={startDragging} role="toolbar" tabindex="0">
      <header class="app-header">
        <div class="header-center">
          <!-- Model dropdown -->
          {#if $page.url.pathname === '/' || $page.url.pathname === '/api'}
            <Popover.Root bind:open={isModelPickerOpen}>
              <Popover.Trigger bind:ref={comboboxTrigger} data-no-drag>
                {#snippet child({ props })}
                  <Button
                    {...props}
                    variant="ghost"
                    class={cn(
                      'model-combobox-trigger',
                      isModelPickerOpen && 'model-combobox-trigger--active',
                    )}
                    role="combobox"
                    aria-expanded={isModelPickerOpen}
                    aria-haspopup="listbox"
                    type="button"
                  >
                    <span class="model-combobox-label">{$currentDisplayName}</span>
                    <CaretDown size={14} />
                  </Button>
                {/snippet}
              </Popover.Trigger>
              <Popover.Content class="model-combobox-content" side="bottom" align="start">
                <Command.Root>
                  {#if $modelSearchEnabled}
                    <Command.Input
                      class="model-combobox-input"
                      placeholder="Поиск модели..."
                      autofocus
                    />
                  {/if}
                  <Command.List class="model-combobox-list">
                    <Command.Empty class="model-combobox-empty">
                      Нет доступных моделей
                    </Command.Empty>
                    <Command.Group>
                      {#each $quickModels as model (model.path)}
                        <Command.Item
                          value={formatModelLabel(model)}
                          onSelect={() => handleSelectModel(model)}
                          class="model-combobox-item"
                        >
                          <Check
                            size={14}
                            weight="bold"
                            class={cn(
                              'model-combobox-check',
                              model.path !== $currentModelPath && 'model-combobox-check--hidden',
                            )}
                          />
                          <div class="model-combobox-item-body">
                            <span class="model-combobox-item-name">
                              {formatModelLabel(model)}
                            </span>
                            <span class="model-combobox-item-meta">
                              {model.architecture ?? 'Неизвестная архитектура'}
                            </span>
                          </div>
                          {#if model.path === $currentModelPath}
                            <span class="model-combobox-item-badge">Текущая</span>
                          {/if}
                        </Command.Item>
                      {/each}
                    </Command.Group>
                  </Command.List>
                </Command.Root>
              </Popover.Content>
            </Popover.Root>
            {#if $isReloadAvailable}
              <button
                type="button"
                class="model-reload-btn"
                onclick={handleReloadModel}
                aria-label="Перезагрузить модель"
              >
                <ArrowClockwise size={16} weight="bold" />
                Перезагрузить модель
              </button>
            {/if}
          {/if}
          <!-- Page tabs -->
          {#if $page.url.pathname === '/models' && $pageTabsList.length > 0}
            <div class="page-tabs" data-no-drag>
              <Tabs.Root
                value={currentHeaderTabValue()}
                class="page-tabs-root"
                onValueChange={handleHeaderTabsChange}
              >
                <Tabs.List class="page-tabs-list" aria-label="Вкладки страницы">
                  {#each $pageTabsList as tab}
                    <Tabs.Trigger class="page-tab" value={tab.id}>
                      {tab.label}
                    </Tabs.Trigger>
                  {/each}
                </Tabs.List>
              </Tabs.Root>
            </div>
          {/if}
        </div>
        <div class="window-controls">
          <button
            type="button"
            class="win-btn"
            title="Свернуть"
            onclick={() => appWindow.minimize()}
          >
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
    <div class="app-body" class:with-history={$showChatHistory && $page.url.pathname === '/'}>
      {#if $showChatHistory && $page.url.pathname === '/'}
        <div class="chat-history-panel">
          <ChatHistory />
        </div>
      {/if}
      <main class="app-main" class:models-compact={$page.url.pathname === '/models'}>
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
  </SidebarUI.Inset>
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
              <path
                d="m127.94,245.01c59.01,0,106.84-47.84,106.84-106.84,0-13.35-2.45-26.12-6.92-37.91l1.17-12.24c7.5,15.11,11.73,32.14,11.73,50.15,0,62.31-50.51,112.83-112.83,112.83S15.1,200.49,15.1,138.17c0-49.87,32.35-92.17,77.2-107.09-.33,2.32-.87,6.31-.91,6.66-41.02,14.93-70.31,54.25-70.31,100.43,0,59.01,47.84,106.84,106.84,106.84h.02Zm-5.32-213.55c1.76-.09,3.54-.12,5.32-.12,18.5,0,35.92,4.7,51.1,12.99.99-1.48,2.3-3.42,3.37-4.98-16.15-8.92-34.72-14-54.47-14-2.87,0-5.72.1-8.54.32l3.21,5.8h.01Z"
                fill-rule="evenodd"
              />
              <path
                d="m125.79,185.34c-12.15,1.1-23.13,2.11-31.34.46-27.18-5.48-47.39-26.77-47.39-26.77-.69-.47-.08-1.47.62-1.16,0,0,28.57,14.41,46.77,14.95,18.2.55,24.65-2.57,39.87-12.18,0,0-.47-.55-.77-.47-.63.17-1.41.48-2.39.87-5.26,2.06-16.09,6.31-36.7,3.09-17.76-2.78-34.36-10.88-42.11-15.1-2.9-1.58-4.65-4.61-4.74-7.93l-.15-5.04c0-.23.08-.39.23-.55l10.86-9.05c2.87-2.39,4.9-5.62,5.78-9.25l1.09-4.43c.83-3.36-1.77-6.58-5.23-6.5l-.84.02c-7.3.18-14.51-1.57-20.91-5.08l54.6-.74c2.12-.03,4.16-.8,5.77-2.16l29.13-24.7c6.26-5.32,14.21-8.23,22.42-8.23h14.71c2.44-3.46,5.15-7.62,8.04-12.06.84-1.29,5.51-10.26,6.38-11.59-14.21-7.65-35.39-12.26-52.67-12.26-.81,0-.5,7.94-1.3,7.96,3.37,7.28,6.32,14.95,8.51,21.37h-7.21c-8.53,0-16.83,2.79-23.63,7.95l-13.03,9.88c-.39.31-1.01,0-1.01-.47.34-5-4.92-10.58-4.7-15.75.24-5.56,1.94-21.8,1.94-21.8-37.28,14.78-67.47,57.01-67.47,99.56,0,39.49,24.13,76.71,57.22,93.23,12.74-5.2,22.26-13.14,28.48-18.32,10.03-8.35,19.54-16.27,38.58-20.69,18.37-4.26,33.46-3.97,44.62-2.21-19.18-8.75-42.18-6.65-62.04-4.84h.01Zm106.29-12.51c.79-5.09,4.87-29.76,4.87-35.07,0-7.65-2.08-21.66-3.7-28.82-.51,8.39-6.96,23.99-6.41,32.6.26,4.1,5.09,27.24,5.24,31.29Zm-41.27-113.42c7.17,5.73,13.54,12.41,18.91,19.87-1.63,7.27-3.12,12.64-3.12,12.64l-3.01-3.81c-5.47-6.94-11.84-13.12-18.95-18.37,0,0,2.53-4.71,6.15-10.33h.01Zm-59.75,33.35c-3.42,2.1-5.74,5.74-5.9,10.01-.31,6.9,5.04,12.72,11.95,13.03h.55c6.59,0,12.09-5.19,12.41-11.87.08-1.24-.08-2.4-.31-3.5-5.51-4.66-11.79-6.67-18.7-7.68Zm-37.49.59l32.08-25.68c2.29-1.81.93-2.92.32-2.92-6.29-.05-12.39,2.02-17.35,5.86l-30.81,23.88h12.53c1.18,0,2.33-.4,3.26-1.14h-.01Zm-58.78,32.61v6.17c0,5.92,1.94,11.67,5.51,16.4v-15.36l-5.51-7.21Zm0,0c0-.62-.23-.55,0,0Z"
                fill-rule="evenodd"
              />
              <path
                d="m240.76,134.06c-.42-12.37-2.9-24.04-7-35.1v-.1c-1.39-3.72-2.97-7.36-4.73-10.89v.05l-1.17,12.24v.02c-.38,3.63-.77,7.2-1.18,10.67-1.16,9.92-.5,20.26.16,30.56.55,8.51,1.09,17,.6,25.2-.36,6.21-1.33,12.24-3.33,18.02-17.31,35.67-53.86,60.26-96.18,60.26-18.79,0-36.45-4.86-51.79-13.38,12.8-5.51,20.79-12.16,28.48-18.56,10.03-8.35,19.54-16.27,38.58-20.69,5.74-1.33,11.16-2.22,16.25-2.76,11.17-1.18,20.69-.66,28.37.55-8.92-4.07-18.65-5.8-28.53-6.29-11.35-.57-22.88.48-33.51,1.45-12.15,1.12-23.13,2.12-31.35.46-14.47-2.92-26.98-10.32-35.49-16.6-3.51-2.58-6.33-4.98-8.36-6.79-1.97-1.78-3.18-3-3.47-3.3-.02-.02-.03-.03-.05-.05l-.02-.02c-.69-.47-.08-1.47.61-1.16,0,0,.03.02.11.06.07.03.16.08.28.14.66.33,2.12,1.04,4.17,1.97,2.46,1.13,5.76,2.6,9.54,4.12,9.77,3.95,22.75,8.36,32.67,8.65,8.72.26,14.75-.32,20.33-2.05,5.36-1.66,10.33-4.4,16.88-8.46.68-.42,1.38-.87,2.1-1.31h.02c.18-.11.36-.23.55-.35,0,0-.47-.55-.77-.47-.62.17-1.41.48-2.39.87-3.3,1.3-8.78,3.45-17.49,4.08-5.19.38-11.52.22-19.22-.99-17.76-2.78-34.36-10.88-42.11-15.11-2.9-1.58-4.65-4.61-4.74-7.93l-.15-5.04c0-.23.08-.39.23-.55l10.86-9.05c2.87-2.38,4.9-5.63,5.78-9.25l1.09-4.43c.66-2.66-.83-5.24-3.2-6.15-.63-.24-1.31-.36-2.03-.35l-.84.02c-7.3.18-14.51-1.57-20.91-5.08l54.6-.74c2.12-.03,4.16-.8,5.77-2.16l29.13-24.7c6.26-5.32,14.21-8.23,22.42-8.23h14.71c2.44-3.46,5.15-7.62,8.04-12.06,1.89-2.92,3.87-5.96,5.92-9.01.89-1.32,2.02-3,3.02-4.45.11-.17.24-.34.35-.51-1.7-.93-3.42-1.83-5.17-2.69-1.84-.9-3.71-1.74-5.6-2.54-.05-.02-.1-.05-.15-.06-.85-.35-1.71-.71-2.57-1.04-.25-.1-.51-.19-.76-.28-.66-.25-1.33-.5-1.99-.74-.33-.12-.67-.24-1-.35-.59-.2-1.18-.41-1.78-.6-.39-.13-.76-.25-1.15-.38-.55-.17-1.1-.34-1.65-.51-.42-.13-.83-.25-1.25-.38-.52-.15-1.06-.3-1.58-.44-.44-.13-.89-.24-1.33-.36-.51-.14-1.04-.26-1.55-.39-.46-.11-.92-.23-1.38-.34-.5-.11-.99-.22-1.49-.33-.99-.22-1.98-.42-2.97-.6-.41-.08-.81-.16-1.22-.23-.57-.1-1.14-.19-1.71-.28-.42-.07-.84-.14-1.26-.2-.57-.08-1.13-.16-1.7-.24-.43-.06-.87-.11-1.31-.17-.56-.07-1.13-.14-1.68-.19-.46-.05-.91-.09-1.36-.14-.55-.06-1.1-.1-1.65-.15-.48-.03-.97-.07-1.45-.1-.52-.03-1.06-.07-1.59-.1-.55-.03-1.09-.05-1.65-.07-.47-.02-.93-.05-1.41-.06-.89-.02-1.79-.03-2.68-.03h-.39c-.69,0-1.38.02-2.06.03h0c-2.16.02-4.3.12-6.43.27л3.21,5.8c3.55,6.88,6.8,14.63,9.38,21.6.74,1.98,1.41,3.91,2.04,5.73h-7.21c-8.53,0-16.83,2.79-23.63,7.95л-13.03,9.88c-.39.31-1.01,0-1.01-.47.02-.24.03-.49.05-.73v-.06c.09-1.43.18-2.88.25-4.33.02-.38.05-.74.06-1.12.16-3.02.28-6.08.42-9.13.34-7.99.68-15.95,1.46-23.05.05-.35.58-4.33.91-6.66v-.02c-2.06.68-4.11,1.42-6.12,2.22v.06C44.6,49.9,15.17,90.56,15.17,138.12c0,38.09,18.88,71.77,47.79,92.2լ6.1,4.02c17.15,10.52,37.33,16.6,58.93,16.6,57.96,0,105.71-43.72,112.09-99.97.07-.3.14-.6.2-.91.4-3.88.62-7.82.62-11.81,0-1.42-.03-2.84-.09-4.25լ-.05.06Zm-132.15-63.44c3.13-2.43,6.73-4.15,10.53-5.07.69-.17,1.39-.31,2.1-.42.72-.11,1.43-.21,2.16-.26h.03c.84-.07,1.69-.1,2.54-.09.59,0,1.84,1,0,2.65,0,0-.01.01-.02.02-.09.08-.18.16-.3.25л-32.08,25.68c-.92.74-2.07,1.14-3.26,1.14h-12.53լ30.81-23.88v-.02Zm-68.31,77.91c-2.85-3.76-4.66-8.19-5.27-12.82-.11-.84-.18-1.69-.22-2.53-.01-.34-.02-.69-.02-1.04v-6.17լ5.51,7.21v15.36h0Zm5.73,2.79c.25.26.52.52.8.77-.27.02-.54.07-.8.14v-.9h0Z"
              />
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

  <!-- SvelteKit layout must expose a slot; hide it to avoid duplicate rendering -->
  <div hidden>{@render children()}</div>
</SidebarUI.Provider>

<style>
  /* App shell & header */
  :global(.app-shell) {
    --model-combobox-bg: transparent;
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    position: relative;
  }

  .app-header-wrapper {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 64px;
    box-sizing: border-box;
    -webkit-app-region: drag; /* Enable window dragging */
    z-index: 200;
  }

  /* Disable dragging on interactive elements */
  .app-header-wrapper button {
    -webkit-app-region: no-drag;
  }

  .app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    height: 56px;
    background: #1a1a1a;
    position: relative;
    z-index: 100;
  }
  .header-center {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 1rem;
    position: relative;
  }

  :global(.header-trigger) {
    -webkit-app-region: no-drag;
  }

  :global(.model-combobox-trigger) {
    min-width: 220px;
    justify-content: space-between;
    gap: 8px;
    padding: 0.35rem 0.75rem;
    background: var(--model-combobox-bg);
    color: #fff;
    border: 1px solid transparent;
    border-radius: 10px;
    -webkit-app-region: no-drag;
    transition:
      background 0.2s ease,
      border 0.2s ease,
      color 0.2s ease;
  }

  :global(.model-combobox-trigger:hover),
  :global(.model-combobox-trigger:focus-visible),
  :global(.model-combobox-trigger--active) {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.2);
    color: #fff;
  }

  :global(.model-combobox-label) {
    font-size: 0.95rem;
    color: inherit;
    flex: 1;
    text-align: center;
    display: block;
  }

  :global(.model-combobox-content) {
    width: 320px;
    padding: 0.5rem;
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 12px;
    box-shadow:
      0 20px 45px rgba(0, 0, 0, 0.35),
      0 4px 18px rgba(0, 0, 0, 0.15);
    z-index: 1200;
    -webkit-app-region: no-drag;
  }

  :global(.model-combobox-input) {
    width: 100%;
    margin-bottom: 0.5rem;
    border-radius: 8px;
    border: none;
    background: var(--model-combobox-bg);
    color: var(--text);
    font-size: 0.85rem;
    padding: 0.55rem 0.75rem;
    outline: none;
  }

  :global(.model-combobox-list) {
    max-height: 320px;
    overflow-y: auto;
  }

  :global(.model-combobox-empty) {
    padding: 0.75rem;
    font-size: 0.85rem;
    color: var(--muted);
  }

  :global(.model-combobox-item) {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 0.5rem;
    border-radius: 8px;
    transition: background 0.15s ease;
  }

  :global(.model-combobox-item:hover) {
    background: rgba(59, 130, 246, 0.08);
  }

  :global(.model-combobox-check) {
    color: var(--accent);
    flex-shrink: 0;
    transition: opacity 0.2s ease;
  }

  :global(.model-combobox-check--hidden) {
    opacity: 0;
  }

  :global(.model-combobox-item-body) {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }

  :global(.model-combobox-item-name) {
    font-weight: 600;
    color: var(--text);
    font-size: 0.9rem;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  :global(.model-combobox-item-meta) {
    color: var(--muted);
    font-size: 0.75rem;
  }

  :global(.model-combobox-item-badge) {
    font-size: 0.7rem;
    line-height: 1;
    padding: 0.2rem 0.5rem;
    border-radius: 9999px;
    background: rgba(59, 130, 246, 0.15);
    color: var(--accent);
    flex-shrink: 0;
  }

  .model-reload-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.4);
    background: rgba(255, 255, 255, 0.05);
    color: inherit;
    font-size: 0.85rem;
    cursor: pointer;
    transition:
      background 0.2s ease,
      border-color 0.2s ease;
  }

  .model-reload-btn:hover {
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.6);
  }

  .page-tabs {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    justify-content: center;
    -webkit-app-region: no-drag;
  }

  :global(.page-tabs-root) {
    display: flex;
    flex-direction: row;
    gap: 0;
  }

  :global(.page-tabs-list) {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0;
    margin: 0;
    background: transparent;
    box-shadow: none;
  }

  :global(.page-tabs .page-tab) {
    flex: 0 0 auto;
    height: auto;
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

  :global(.page-tabs .page-tab:hover) {
    background: rgba(255, 255, 255, 0.1);
  }

  :global(.page-tabs .page-tab[data-state='active']) {
    background: rgba(255, 255, 255, 0.2);
    font-weight: 600;
  }
  .app-body {
    flex: 1 1 auto;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--content-gap);
    overflow: hidden;
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
  }

  .app-body.with-history {
    grid-template-columns: minmax(0, auto) minmax(0, 1fr);
  }

  .app-body.with-history .app-main {
    grid-column: 2;
  }
  .app-main {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    overflow: hidden;
    padding: var(--content-gap);
    padding-top: calc(var(--content-gap-top) + 64px);
  }

  .app-main.models-compact {
    padding-top: 56px;
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
    margin: 16px 0;
    border-radius: 14px;
    box-shadow: 0 6px 30px rgb(0 0 0 / 0.05);
  }

  .about-modal {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(2px);
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
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: background-color 0.2s ease;
    min-width: 80px;
  }

  .close-btn:hover {
    background: var(--accent-2, #2563eb);
  }

  .close-btn:focus {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .github-btn,
  .gitverse-btn {
    background: transparent;
    color: var(--text);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 8px 16px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: all 0.2s ease;
    min-width: 80px;
    display: flex;
    align-items: center;
    gap: 8px;
    margin-right: 8px;
  }

  .github-btn:hover,
  .gitverse-btn:hover {
    background: var(--panel-bg);
    border-color: var(--accent);
  }

  .github-btn:focus,
  .gitverse-btn:focus {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
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
