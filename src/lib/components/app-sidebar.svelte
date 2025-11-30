<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import * as Sidebar from '$lib/components/ui/sidebar/index';
  import SidebarSimple from 'phosphor-svelte/lib/SidebarSimple';
  import ChatCircle from 'phosphor-svelte/lib/ChatCircle';
  import Database from 'phosphor-svelte/lib/Database';
  import Gear from 'phosphor-svelte/lib/Gear';
  import Code from 'phosphor-svelte/lib/Code';
  import ChartLine from 'phosphor-svelte/lib/ChartLine';
  import ArrowCircleDown from 'phosphor-svelte/lib/ArrowCircleDown';
  import Info from 'phosphor-svelte/lib/Info';
  import {
    activeDownloads,
    downloadsLoaded,
    ensureDownloadManager,
  } from '$lib/stores/download-manager';
  import { useSidebar } from '$lib/components/ui/sidebar/context.svelte.js';
  import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';
  import { t } from '$lib/i18n';

  // Navigation items с переводами
  const baseNavigationItems = $derived([
    {
      id: 'chat',
      title: $t('sidebar.navigation.chat'),
      icon: ChatCircle,
      path: '/',
    },
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
      ? [
          baseNavigationItems[0],
          ...experimentalNavigationItems,
          baseNavigationItems[1],
          baseNavigationItems[2],
        ]
      : baseNavigationItems,
  );

  const dispatch = createEventDispatcher<{
    openDownloads: void;
    openAbout: void;
  }>();

  let currentPath = $derived($page.url.pathname);
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

  onMount(() => {
    void ensureDownloadManager();
  });

  function navigateTo(path: string) {
    if (currentPath === path) return;
    goto(path);
  }

  // When the sidebar is collapsed, repurpose the brand button as the toggle trigger.
  function handleBrandClick(event: MouseEvent) {
    if (sidebar.state === 'collapsed') {
      event.preventDefault();
      sidebar.toggle();
      return;
    }
    navigateTo('/');
  }

  function handleDownloadsClick() {
    dispatch('openDownloads');
  }

  function handleAboutClick() {
    dispatch('openAbout');
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

  <Sidebar.Content class="app-sidebar__content">
    <Sidebar.Group>
      <Sidebar.Menu class="app-sidebar__menu">
        {#each navigationItems as item}
          {@const Icon = item.icon}
          <Sidebar.MenuItem>
            <Sidebar.MenuButton
              tooltipContent={item.title}
              isActive={currentPath === item.path}
              aria-label={item.title}
            >
              {#snippet child({ props })}
                <button
                  type="button"
                  {...props}
                  onclick={() => navigateTo(item.path)}
                  aria-current={currentPath === item.path ? 'page' : undefined}
                >
                  <Icon size={20} weight="regular" />
                  <span>{item.title}</span>
                </button>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
        {/each}
      </Sidebar.Menu>
    </Sidebar.Group>
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
          <Sidebar.MenuButton tooltipContent={$t('sidebar.footer.about')} aria-label={$t('sidebar.footer.about')}>
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
    height: 56px;
    min-height: 56px;
    padding: 0 !important;
    width: 100%;
    justify-content: flex-start !important;
    -webkit-app-region: drag;
  }

  .brand-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    margin-left: 8px;
    flex-shrink: 0;
    border-radius: 8px;
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


  .brand-button:hover {
    background: color-mix(in srgb, var(--sidebar-border) 20%, transparent);
    border-color: var(--sidebar-border);
  }

  .brand-icon {
    width: 28px;
    height: 28px;
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

  .indicator-bar {
    position: relative;
    height: 4px;
    border-radius: 999px;
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
