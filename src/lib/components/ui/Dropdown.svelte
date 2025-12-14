<script lang="ts">
  import { onMount } from 'svelte';

  interface MenuItem {
    label: string;
    onclick: () => void;
    disabled?: boolean;
  }

  interface Props {
    items?: MenuItem[];
    label?: string;
  }

  let { items = [], label = '⋯' }: Props = $props();

  let isOpen = $state(false);
  let dropdownElement: HTMLDivElement | undefined = $state();

  function toggleMenu() {
    isOpen = !isOpen;
  }

  function closeMenu() {
    isOpen = false;
  }

  function handleItemClick(item: MenuItem) {
    if (!item.disabled) {
      item.onclick();
      closeMenu();
    }
  }

  onMount(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownElement && !dropdownElement.contains(event.target as Node)) {
        closeMenu();
      }
    }

    document.addEventListener('click', handleClickOutside);
    return () => document.removeEventListener('click', handleClickOutside);
  });
</script>

<div class="dropdown" bind:this={dropdownElement}>
  <button class="dropdown-trigger" onclick={toggleMenu}>
    {label}
  </button>

  {#if isOpen}
    <div class="dropdown-menu">
      {#each items as item (item.label)}
        <button
          class="dropdown-item"
          disabled={item.disabled}
          onclick={() => handleItemClick(item)}
        >
          {item.label}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .dropdown {
    position: relative;
    display: inline-block;
  }

  .dropdown-trigger {
    width: var(--space-5); /* 32px */
    height: var(--space-5); /* 32px */
    padding: 0;
    border-radius: var(--radius);
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
    cursor: default;
    font-size: 1.25rem;
    font-weight: var(--font-weight-normal);
    line-height: 1;
    transition:
      transform 0.2s ease,
      box-shadow 0.2s ease,
      background 0.2s ease,
      border-color 0.2s ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }

  .dropdown-menu {
    position: absolute;
    top: calc(100% + 0.5rem);
    left: 0;
    background: var(--card);
    border: 1px solid var(--border-color, #d8dee5);
    border-radius: var(--radius-md); /* 12px */
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 100;
    min-width: 200px;
    overflow: visible;
    padding: var(--space-1); /* 4px */
  }

  .dropdown-item {
    display: block;
    width: 100%;
    padding: var(--space-2) var(--space-2); /* 8px 8px → 8px 12px closest */
    border: none;
    background: transparent;
    color: var(--text);
    cursor: pointer;
    font-size: 0.8rem;
    text-align: left;
    transition: background-color 0.2s ease;
    box-sizing: border-box;
    margin: 0;
    border-radius: var(--radius);
  }

  .dropdown-item:hover:not(:disabled) {
    background-color: color-mix(in srgb, var(--accent) 12%, transparent 88%);
    transform: none;
  }

  .dropdown-item:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
