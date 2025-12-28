<script lang="ts">
  /**
   * App Sidebar Component
   * Simple sidebar based on shadcn-svelte blocks.
   */
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import * as Sidebar from '$lib/components/ui/sidebar/index';
  import { useSidebar } from '$lib/components/ui/sidebar/context.svelte.js';

  // Phosphor Icons
  import Database from 'phosphor-svelte/lib/Database';
  import Gear from 'phosphor-svelte/lib/Gear';
  import Code from 'phosphor-svelte/lib/Code';
  import ChartLine from 'phosphor-svelte/lib/ChartLine';
  import ArrowCircleDown from 'phosphor-svelte/lib/ArrowCircleDown';
  import Info from 'phosphor-svelte/lib/Info';
  import Plus from 'phosphor-svelte/lib/Plus';
  import SidebarSimple from 'phosphor-svelte/lib/SidebarSimple';

  // Stores
  import {
    activeDownloads,
    downloadsLoaded,
    ensureDownloadManager,
  } from '$lib/stores/download-manager';
  import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';
  import { t, locale } from '$lib/i18n';
  import { chatHistory, sortedSessions, currentSession } from '$lib/stores/chat-history';

  // Get sidebar state
  const sidebar = useSidebar();

  // Get current locale
  let currentLocale = $derived($locale);

  // Navigation items
  const baseNavigationItems = $derived([
    {
      id: 'models',
      title: currentLocale ? $t('sidebar.navigation.models') : 'Model Manager',
      icon: Database,
      path: '/models',
    },
    {
      id: 'settings',
      title: currentLocale ? $t('sidebar.navigation.settings') : 'Settings',
      icon: Gear,
      path: '/settings',
    },
  ] as const);

  const experimentalNavigationItems = $derived([
    {
      id: 'api',
      title: currentLocale ? $t('sidebar.navigation.api') : 'API',
      icon: Code,
      path: '/api',
    },
    {
      id: 'performance',
      title: currentLocale ? $t('sidebar.navigation.performance') : 'Performance',
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

  // Translated labels
  let newChatLabel = $derived(currentLocale ? $t('sidebar.newChat') : 'New Chat');
  let chatsLabel = $derived(currentLocale ? $t('sidebar.chats') : 'Chats');
  let downloadsLabel = $derived(currentLocale ? $t('sidebar.footer.downloads') : 'Downloads');
  let aboutLabel = $derived(currentLocale ? $t('sidebar.footer.about') : 'About');

  onMount(() => {
    void ensureDownloadManager();
  });

  function navigateTo(path: string) {
    if (currentPath === path) return;
    goto(path);
  }

  async function handleNewChat() {
    const _id = await chatHistory.createSession();
    if (currentPath !== '/') goto('/');
  }

  function handleLoadSession(id: string) {
    chatHistory.loadSession(id);
    if (currentPath !== '/') goto('/');
  }

  function handleDownloadsClick() {
    onOpenDownloads?.();
  }

  function handleAboutClick() {
    onOpenAbout?.();
  }
</script>

<Sidebar.Root collapsible="icon">
  <!-- Header with brand -->
  <Sidebar.Header class="!flex-row h-14 items-center justify-between p-2">
    <!-- Brand button -->
    <button
      type="button"
      class="brand-button flex items-center rounded-md py-2 pl-2.5 pr-2.5 size-10 hover:bg-sidebar-accent transition-colors"
      onclick={() => {
        if (sidebar.state === 'collapsed') {
          sidebar.toggle();
        } else {
          goto('/');
        }
      }}
    >
      <span class="brand-icon-wrapper relative w-5 h-5 min-w-5 min-h-5 shrink-0">
        <img src="/icon.svg" alt="Oxide Lab" class="brand-icon-default w-5 h-5 absolute inset-0" />
        <SidebarSimple
          size={20}
          weight="regular"
          class="brand-icon-hover w-5 h-5 absolute inset-0 text-sidebar-foreground"
        />
      </span>
    </button>

    <!-- Toggle button (only when expanded) -->
    {#if sidebar.state !== 'collapsed'}
      <button
        type="button"
        class="flex items-center justify-center rounded-md size-10 hover:bg-sidebar-accent transition-colors text-sidebar-foreground"
        onclick={() => sidebar.toggle()}
      >
        <SidebarSimple size={20} weight="regular" />
      </button>
    {/if}
  </Sidebar.Header>

  <Sidebar.Content>
    <!-- New Chat -->
    <Sidebar.Group>
      <Sidebar.Menu>
        <Sidebar.MenuItem>
          <Sidebar.MenuButton tooltipContent={newChatLabel}>
            {#snippet child({ props })}
              <button {...props} onclick={handleNewChat}>
                <Plus size={16} weight="bold" />
                <span>{newChatLabel}</span>
              </button>
            {/snippet}
          </Sidebar.MenuButton>
        </Sidebar.MenuItem>
      </Sidebar.Menu>
    </Sidebar.Group>

    <!-- Navigation -->
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

    <!-- Chats History (hidden when collapsed) -->
    {#if sidebar.state !== 'collapsed'}
      <Sidebar.Group class="flex-1 min-h-0 flex flex-col">
        <Sidebar.GroupLabel>{chatsLabel}</Sidebar.GroupLabel>
        <Sidebar.GroupContent class="flex-1 min-h-0 overflow-y-auto custom-scrollbar">
          <Sidebar.Menu>
            {#each $sortedSessions as session (session.id)}
              <Sidebar.MenuItem>
                <Sidebar.MenuButton isActive={$currentSession?.id === session.id}>
                  {#snippet child({ props })}
                    <button
                      {...props}
                      onclick={() => handleLoadSession(session.id)}
                      class="w-full text-left truncate"
                    >
                      <span>{session.title || 'Untitled Chat'}</span>
                    </button>
                  {/snippet}
                </Sidebar.MenuButton>
              </Sidebar.MenuItem>
            {/each}
          </Sidebar.Menu>
        </Sidebar.GroupContent>
      </Sidebar.Group>
    {/if}
  </Sidebar.Content>

  <!-- Footer -->
  <Sidebar.Footer>
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <Sidebar.MenuButton tooltipContent={downloadsLabel} isActive={hasActiveDownloads}>
          {#snippet child({ props })}
            <button type="button" {...props} onclick={handleDownloadsClick}>
              <ArrowCircleDown size={16} weight="regular" />
              <span>{downloadsLabel}</span>
            </button>
          {/snippet}
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
      <Sidebar.MenuItem>
        <Sidebar.MenuButton tooltipContent={aboutLabel}>
          {#snippet child({ props })}
            <button type="button" {...props} onclick={handleAboutClick}>
              <Info size={16} weight="regular" />
              <span>{aboutLabel}</span>
            </button>
          {/snippet}
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    </Sidebar.Menu>
  </Sidebar.Footer>
</Sidebar.Root>

<style>
  :global(.brand-icon-hover) {
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  :global(.brand-icon-default) {
    opacity: 1;
    transition: opacity 0.15s ease;
  }

  /* Hover effect only when collapsed */
  :global([data-state='collapsed']) .brand-button:hover :global(.brand-icon-hover) {
    opacity: 1;
  }

  :global([data-state='collapsed']) .brand-button:hover :global(.brand-icon-default) {
    opacity: 0;
  }
</style>
