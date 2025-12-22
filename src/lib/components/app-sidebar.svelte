<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { cn } from '$lib/utils';
  import * as Sidebar from '$lib/components/ui/sidebar/index';
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index';
  import { mergeProps } from 'bits-ui';

  // Icons
  import SidebarSimple from 'phosphor-svelte/lib/SidebarSimple';
  import Database from 'phosphor-svelte/lib/Database';
  import Gear from 'phosphor-svelte/lib/Gear';
  import Code from 'phosphor-svelte/lib/Code';
  import ChartLine from 'phosphor-svelte/lib/ChartLine';
  import ArrowCircleDown from 'phosphor-svelte/lib/ArrowCircleDown';
  import Info from 'phosphor-svelte/lib/Info';
  import Plus from 'phosphor-svelte/lib/Plus';
  import Trash from 'phosphor-svelte/lib/Trash';
  import PencilSimple from 'phosphor-svelte/lib/PencilSimple';
  import DotsThree from 'phosphor-svelte/lib/DotsThree';

  import {
    activeDownloads,
    downloadsLoaded,
    ensureDownloadManager,
  } from '$lib/stores/download-manager';
  import { useSidebar } from '$lib/components/ui/sidebar/context.svelte.js';
  import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';
  import { t } from '$lib/i18n';
  import { chatHistory, sortedSessions, currentSession } from '$lib/stores/chat-history';
  import { showChatHistory } from '$lib/stores/sidebar';

  // Navigation items с переводами
  const baseNavigationItems = $derived([
    // Chat удален из навигации, так как он теперь основной контекст или "New Chat"
    // Но "My Stuff" может содержать ссылки на другие разделы.
    // В референсе "Chat" нет в списке "My Stuff", там "Home" или ничего.
    // Оставим Models и Settings здесь.
    {
      id: 'models',
      title: $t('sidebar.navigation.models'),
      icon: Database,
      path: '/models',
    },
    {
      id: 'settings',
      title: $t('sidebar.navigation.settings'),
      icon: Gear,
      path: '/settings',
    },
  ] as const);

  const experimentalNavigationItems = $derived([
    {
      id: 'api',
      title: $t('sidebar.navigation.api'),
      icon: Code,
      path: '/api',
    },
    {
      id: 'performance',
      title: $t('sidebar.navigation.performance'),
      icon: ChartLine,
      path: '/performance',
    },
  ] as const);

  const navigationItems = $derived(
    experimentalFeatures.enabled
      ? [...experimentalNavigationItems, ...baseNavigationItems]
      : baseNavigationItems,
  );

  interface Props {
    onOpenDownloads?: () => void;
    onOpenAbout?: () => void;
  }

  let { onOpenDownloads, onOpenAbout }: Props = $props();

  let currentPath = $derived(page.url.pathname);
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
          if (total === 0) return null;
          return Math.min(1, downloaded / total);
        })(),
  );

  const sidebar = useSidebar();
  let editingSessionId: string | null = $state(null);
  let editingTitle = $state('');

  // При сворачивании сайдбара скрываем историю чатов (чтобы вести себя как лейбл Chats)
  $effect(() => {
    if (sidebar.state === 'collapsed') {
      showChatHistory.set(false);
    }
  });

  // Svelte action for programmatic focus (avoids a11y autofocus warning)
  function focusOnMount(node: HTMLInputElement) {
    node.focus();
    return {};
  }

  onMount(() => {
    void ensureDownloadManager();
  });

  function navigateTo(path: string) {
    if (currentPath === path) return;
    goto(path);
  }

  function handleBrandClick(event: MouseEvent) {
    if (sidebar.state === 'collapsed') {
      event.preventDefault();
      sidebar.toggle();
      return;
    }
    navigateTo('/');
  }

  function handleDownloadsClick() {
    onOpenDownloads?.();
  }

  function handleAboutClick() {
    onOpenAbout?.();
  }

  // Chat Actions
  async function handleNewChat() {
    const _id = await chatHistory.createSession();
    if (currentPath !== '/') goto('/');
  }

  function handleLoadSession(id: string) {
    chatHistory.loadSession(id);
    if (currentPath !== '/') goto('/');
  }

  function handleRenameSession(id: string, newTitle: string) {
    chatHistory.renameSession(id, newTitle);
    editingSessionId = null;
  }

  function startEditing(session: any) {
    editingSessionId = session.id;
    editingTitle = session.title;
  }

  function cancelEdit() {
    editingSessionId = null;
  }

  function handleDeleteSession(id: string) {
    // В реальном приложении лучше спросить подтверждение,
    // но в сайдбаре часто удаляют сразу или через меню.
    // Сделаем через DropdownMenu действие.
    chatHistory.deleteSession(id);
  }
</script>

<Sidebar.Root collapsible="icon" variant="sidebar">
  <Sidebar.Header class="app-sidebar__header p-0 gap-0 flex-row h-14 items-center justify-start">
    <button
      class="brand-button"
      class:brand-button--collapsed={sidebar.state === 'collapsed'}
      type="button"
      onclick={handleBrandClick}
      title={$t('sidebar.brand.home')}
    >
      <img src="/icon.svg" alt="Oxide Lab" class="brand-icon brand-icon-default" />
      {#if sidebar.state === 'collapsed'}
        <div class="brand-icon brand-icon-hover">
          <SidebarSimple size={24} weight="regular" />
        </div>
      {/if}
    </button>
    {#if sidebar.state !== 'collapsed'}
      <Sidebar.Trigger class="header-trigger sidebar-header-trigger" data-no-drag />
    {/if}
  </Sidebar.Header>

  <Sidebar.Content class="app-sidebar__content gap-0 overflow-hidden">
    <!-- "New Chat" shortcut when collapsed or main action -->
    <Sidebar.Group>
      <Sidebar.Menu>
        <Sidebar.MenuItem>
          <Sidebar.MenuButton isActive={false} tooltipContent="New Chat">
            {#snippet child({ props })}
              <button {...props} onclick={handleNewChat}>
                <Plus size={16} weight="bold" class="text-sidebar-foreground" />
                <span>New chat</span>
              </button>
            {/snippet}
          </Sidebar.MenuButton>
        </Sidebar.MenuItem>
      </Sidebar.Menu>
    </Sidebar.Group>

    <!-- My Stuff Group -->
    <Sidebar.Group>
      <Sidebar.Menu>
        {#each navigationItems as item}
          {@const Icon = item.icon}
          <Sidebar.MenuItem>
            <Sidebar.MenuButton tooltipContent={item.title} isActive={currentPath === item.path}>
              {#snippet child({ props })}
                <button {...props} onclick={() => navigateTo(item.path)}>
                  <Icon size={16} weight="regular" />
                  <span>{item.title}</span>
                </button>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
        {/each}
      </Sidebar.Menu>
    </Sidebar.Group>

    <!-- Chats History Group (hidden when collapsed) -->
    {#if sidebar.state !== 'collapsed'}
      <Sidebar.Group class="flex-1 min-h-0 flex flex-col p-0">
        <Sidebar.GroupLabel class="chats-label px-2 pt-2">Chats</Sidebar.GroupLabel>
        <Sidebar.GroupContent
          class="flex-1 min-h-0 overflow-y-auto scroll-smooth scrollbar-auto-hide pb-2"
        >
          <Sidebar.Menu class="gap-0.5">
            {#each $sortedSessions as session (session.id)}
              <Sidebar.MenuItem>
                {#if editingSessionId === session.id}
                  <div class="px-2 py-1">
                    <input
                      class="w-full bg-sidebar-input rounded-md px-2 py-1 text-sm text-sidebar-foreground border border-sidebar-border focus:outline-none focus:ring-1 focus:ring-sidebar-ring"
                      bind:value={editingTitle}
                      onblur={() => handleRenameSession(session.id, editingTitle)}
                      onkeydown={(e) => {
                        if (e.key === 'Enter') handleRenameSession(session.id, editingTitle);
                        if (e.key === 'Escape') cancelEdit();
                      }}
                      use:focusOnMount
                    />
                  </div>
                {:else}
                  <Sidebar.MenuButton
                    isActive={$currentSession?.id === session.id}
                    class="chat-session-button"
                  >
                    {#snippet child({ props })}
                      <button
                        {...props}
                        onclick={() => handleLoadSession(session.id)}
                        class={cn(props.class as any, 'flex-1 overflow-hidden text-left truncate')}
                      >
                        <span>{session.title || 'Untitled Chat'}</span>
                      </button>
                    {/snippet}
                  </Sidebar.MenuButton>

                  <DropdownMenu.Root>
                    <DropdownMenu.Trigger>
                      {#snippet child({ props: triggerProps })}
                        <Sidebar.MenuAction
                          showOnHover={$currentSession?.id !== session.id}
                          class={cn(
                            $currentSession?.id === session.id ? '!opacity-100' : '',
                            '!bg-transparent',
                          )}
                        >
                          {#snippet child({ props: actionProps })}
                            <button {...mergeProps(triggerProps, actionProps)}>
                              <DotsThree size={16} weight="bold" />
                            </button>
                          {/snippet}
                        </Sidebar.MenuAction>
                      {/snippet}
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Content align="end">
                      <DropdownMenu.Item onclick={() => startEditing(session)}>
                        <PencilSimple class="mr-2 h-3.5 w-3.5" />
                        <span>Rename</span>
                      </DropdownMenu.Item>
                      <DropdownMenu.Item
                        onclick={() => handleDeleteSession(session.id)}
                        class="text-destructive focus:text-destructive"
                      >
                        <Trash class="mr-2 h-3.5 w-3.5" />
                        <span>Delete</span>
                      </DropdownMenu.Item>
                    </DropdownMenu.Content>
                  </DropdownMenu.Root>
                {/if}
              </Sidebar.MenuItem>
            {/each}
          </Sidebar.Menu>
        </Sidebar.GroupContent>
      </Sidebar.Group>
    {/if}
  </Sidebar.Content>

  <Sidebar.Footer class="app-sidebar__footer">
    <Sidebar.Group>
      <Sidebar.Menu>
        <Sidebar.MenuItem>
          <Sidebar.MenuButton
            tooltipContent={$t('sidebar.footer.downloads')}
            aria-label={$t('sidebar.footer.downloads')}
            isActive={hasActiveDownloads}
          >
            {#snippet child({ props })}
              <button type="button" {...props} onclick={handleDownloadsClick}>
                <ArrowCircleDown size={20} weight="regular" />
                <span>{$t('sidebar.footer.downloads')}</span>
              </button>
            {/snippet}
          </Sidebar.MenuButton>
          {#if hasActiveDownloads}
            <div class="download-indicator" aria-hidden="true">
              {#if aggregateDownloadProgress !== null}
                <div class="indicator-bar">
                  <div
                    class="indicator-fill"
                    style={`width: ${aggregateDownloadProgress * 100}%`}
                  ></div>
                </div>
              {:else}
                <div class="indicator-bar indeterminate">
                  <span></span>
                </div>
              {/if}
            </div>
          {/if}
        </Sidebar.MenuItem>
        <Sidebar.MenuItem>
          <Sidebar.MenuButton
            tooltipContent={$t('sidebar.footer.about')}
            aria-label={$t('sidebar.footer.about')}
          >
            {#snippet child({ props })}
              <button type="button" {...props} onclick={handleAboutClick}>
                <Info size={20} weight="regular" />
                <span>{$t('sidebar.footer.about')}</span>
              </button>
            {/snippet}
          </Sidebar.MenuButton>
        </Sidebar.MenuItem>
      </Sidebar.Menu>
    </Sidebar.Group>
  </Sidebar.Footer>

  <!-- Sidebar.Rail intentionally omitted to keep layout clean -->
</Sidebar.Root>

<style>
  :global([data-slot='sidebar'][data-state='collapsed'] [data-slot='sidebar-inner']) {
    background: var(--sidebar-collapsed-bg);
  }

  :global(.app-sidebar__header) {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    height: var(--space-8); /* 56px */
    min-height: var(--space-8); /* 56px */
    padding: 0 !important;
    width: 100%;
    justify-content: flex-start !important;
    -webkit-app-region: drag;
  }

  :global(.sidebar-header-trigger) {
    margin-left: auto;
    margin-right: 0.5rem;
    -webkit-app-region: no-drag;
    background: transparent;
    color: var(--sidebar-foreground);
    border-color: transparent;
    box-shadow: none;
  }

  :global(.sidebar-header-trigger:hover),
  :global(.sidebar-header-trigger:focus-visible) {
    background: color-mix(in srgb, var(--sidebar-primary) 15%, transparent);
    color: var(--sidebar-primary);
    border-color: color-mix(in srgb, var(--sidebar-primary) 25%, transparent);
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--sidebar-primary) 20%, transparent);
  }

  .brand-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: var(--space-6); /* 40px */
    height: var(--space-6); /* 40px */
    margin-left: var(--space-2); /* 8px */
    flex-shrink: 0;
    border-radius: var(--radius); /* 16px */
    background: transparent;
    border: 1px solid transparent;
    transition:
      background 0.2s ease,
      border-color 0.2s ease;
    position: relative;
    overflow: hidden;
    cursor: pointer;
    -webkit-app-region: no-drag;
  }

  .brand-button:hover {
    background: color-mix(in srgb, var(--sidebar-border) 20%, transparent);
    border-color: var(--sidebar-border);
  }

  .brand-icon {
    width: var(--space-4); /* 24px → 28px closest */
    height: var(--space-4); /* 24px → 28px closest */
    position: absolute;
    inset: 0;
    margin: auto;
    transition: opacity 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
  }

  .brand-icon-hover {
    opacity: 0;
    color: var(--sidebar-primary);
  }

  .brand-button:not(.brand-button--collapsed) .brand-icon-hover {
    display: none;
  }

  .brand-button--collapsed:hover .brand-icon-default {
    opacity: 0;
  }

  .brand-button--collapsed:hover .brand-icon-hover {
    opacity: 1;
  }

  :global(.app-sidebar__content) {
    padding: 0.75rem 0.5rem;
    flex: 1;
  }

  :global(.chat-session-button) {
    width: 100% !important;
  }

  /* Keep highlight active when hovering anywhere in the item (including action buttons) */
  :global([data-slot='sidebar-menu-item']:hover .chat-session-button) {
    background-color: var(--sidebar-accent) !important;
    color: var(--sidebar-accent-foreground) !important;
  }

  :global(.app-sidebar__menu) {
    gap: 0.25rem;
  }

  :global(.app-sidebar__footer) {
    padding: 0.5rem;
  }

  :global([data-sidebar='menu-button']) {
    background: transparent;
    color: var(--sidebar-foreground);
  }

  /* Shift icons left in collapsed mode */
  :global([data-state='collapsed'] [data-sidebar='menu-button'] svg) {
    margin-left: 2px;
  }

  .download-indicator {
    width: 100%;
    padding: 0.25rem 0 0.4rem;
  }

  :global(.chats-label) {
    display: block;
    padding-left: var(--space-3);
    padding-top: var(--space-2);
  }

  .indicator-bar {
    position: relative;
    height: var(--space-2); /* 8px */
    border-radius: var(--radius-full); /* 9999px */
    background: color-mix(in srgb, var(--sidebar-ring) 18%, transparent 82%);
    overflow: hidden;
  }

  .indicator-fill {
    height: 100%;
    border-radius: inherit;
    background: var(--sidebar-ring);
    transition: width 0.2s ease;
  }

  .indicator-bar.indeterminate span {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      90deg,
      transparent 0%,
      color-mix(in srgb, var(--sidebar-ring) 50%, transparent) 50%,
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
</style>
