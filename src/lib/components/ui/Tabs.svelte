<script lang="ts">
  /**
   * Reusable Tabs component
   * Provides accessible horizontal tabs with keyboard navigation
   */

  interface Tab {
    id: string;
    label: string;
  }

  let {
    tabs,
    activeTab = $bindable(''),
    children,
  }: {
    tabs: Tab[];
    activeTab: string;
    children: any;
  } = $props();

  // Handle tab selection
  function selectTab(tabId: string) {
    activeTab = tabId;
  }

  // Handle keyboard navigation
  function handleKeydown(event: KeyboardEvent, currentIndex: number) {
    let newIndex = currentIndex;

    switch (event.key) {
      case 'ArrowLeft':
        event.preventDefault();
        newIndex = currentIndex > 0 ? currentIndex - 1 : tabs.length - 1;
        break;
      case 'ArrowRight':
        event.preventDefault();
        newIndex = currentIndex < tabs.length - 1 ? currentIndex + 1 : 0;
        break;
      case 'Home':
        event.preventDefault();
        newIndex = 0;
        break;
      case 'End':
        event.preventDefault();
        newIndex = tabs.length - 1;
        break;
      default:
        return;
    }

    // Select new tab and focus
    selectTab(tabs[newIndex].id);
    const tabButton = document.querySelector(
      `[data-tab-id="${tabs[newIndex].id}"]`,
    ) as HTMLButtonElement;
    tabButton?.focus();
  }
</script>

<div class="tabs-container">
  <!-- Tab list -->
  <div class="tabs-list" role="tablist" aria-label="Вкладки менеджера моделей">
    {#each tabs as tab, index (tab.id)}
      <button
        type="button"
        role="tab"
        data-tab-id={tab.id}
        class="tab"
        class:active={activeTab === tab.id}
        aria-selected={activeTab === tab.id}
        aria-controls={`tabpanel-${tab.id}`}
        tabindex={activeTab === tab.id ? 0 : -1}
        onclick={() => selectTab(tab.id)}
        onkeydown={(e) => handleKeydown(e, index)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <!-- Tab panels -->
  <div class="tabs-content">
    {@render children()}
  </div>
</div>

<style>
  .tabs-container {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .tabs-list {
    display: flex;
    gap: 0.25rem;
    padding: 0.5rem 0.5rem 0;
    background: var(--bg);
    border-bottom: 2px solid var(--border-color);
    overflow-x: auto;
    overflow-y: hidden;
  }

  /* Скроллбар для горизонтальной прокрутки */
  .tabs-list::-webkit-scrollbar {
    height: 6px;
  }

  .tabs-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .tabs-list::-webkit-scrollbar-thumb {
    background: rgba(179, 205, 224, 0.5);
    border-radius: 3px;
  }

  .tabs-list::-webkit-scrollbar-thumb:hover {
    background: rgba(179, 205, 224, 0.7);
  }

  .tab {
    position: relative;
    padding: 0.75rem 1.5rem;
    border: none;
    background: transparent;
    color: var(--muted);
    font-size: 0.9375rem;
    font-weight: 500;
    cursor: default;
    transition: all 0.2s ease;
    white-space: nowrap;
    border-radius: 8px 8px 0 0;
  }

  .tab::after {
    content: '';
    position: absolute;
    bottom: -2px;
    left: 0;
    right: 0;
    height: 2px;
    background: transparent;
    transition: background-color 0.2s ease;
  }

  .tab:hover {
    background: rgba(52, 152, 219, 0.05);
    color: var(--text);
  }

  .tab:focus {
    outline: 2px solid var(--accent, #3498db);
    outline-offset: 2px;
  }

  .tab.active {
    background: rgba(52, 152, 219, 0.1);
    color: var(--accent, #3498db);
    font-weight: 600;
  }

  .tab.active::after {
    background: var(--accent, #3498db);
  }

  .tabs-content {
    flex: 1;
    overflow: hidden;
    background: var(--bg);
  }

  /* Адаптивность для мобильных */
  @media (max-width: 1024px) {
    .tab {
      padding: 0.6875rem 1.25rem;
      font-size: 0.9rem;
    }
  }

  @media (max-width: 768px) {
    .tab {
      padding: 0.625rem 1rem;
      font-size: 0.875rem;
    }

    .tabs-list {
      padding: 0.375rem 0.375rem 0;
      gap: 0.125rem;
    }

    .tabs-content {
      border-radius: 0;
    }
  }

  @media (max-width: 480px) {
    .tab {
      padding: 0.5rem 0.875rem;
      font-size: 0.8125rem;
      border-radius: 6px 6px 0 0;
    }

    .tabs-list {
      padding: 0.25rem 0.25rem 0;
      gap: 0.0625rem;
    }

    .tab:focus {
      outline-offset: 1px;
    }
  }

  /* Адаптация для широких экранов */
  @media (min-width: 1440px) {
    .tabs-list {
      padding: 0.625rem 1rem 0;
    }

    .tab {
      padding: 0.875rem 1.75rem;
    }
  }
</style>
