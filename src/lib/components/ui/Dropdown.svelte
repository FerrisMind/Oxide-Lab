<script lang="ts">
  import { onMount } from 'svelte';

  interface MenuItem {
    label: string;
    onclick: () => void;
    disabled?: boolean;
  }

  export let items: MenuItem[] = [];
  export let label = '⋯';

  let isOpen = false;
  let dropdownElement: HTMLDivElement;

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
      if (
        dropdownElement &&
        !dropdownElement.contains(event.target as Node)
      ) {
        closeMenu();
      }
    }

    document.addEventListener('click', handleClickOutside);
    return () => document.removeEventListener('click', handleClickOutside);
  });
</script>

<div class="dropdown" bind:this={dropdownElement}>
  <button class="dropdown-trigger" on:click={toggleMenu}>
    {label}
  </button>

  {#if isOpen}
    <div class="dropdown-menu">
      {#each items as item (item.label)}
        <button
          class="dropdown-item"
          disabled={item.disabled}
          on:click={() => handleItemClick(item)}
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
    width: 34px;
    height: 34px;
    padding: 0;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
    cursor: default;
    font-size: 1.25rem;
    font-weight: 400;
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

  .dropdown-trigger::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
    transition: left 0.5s ease;
  }

  .dropdown-trigger:not(:disabled):hover {
    transform: none;
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.2);
    box-shadow:
      0 8px 25px rgba(0, 0, 0, 0.15),
      0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .dropdown-trigger:not(:disabled):hover::before {
    left: 100%;
  }

  .dropdown-trigger:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .dropdown-menu {
    position: absolute;
    top: calc(100% + 0.5rem);
    left: 0;
    background: var(--card);
    border: 1px solid var(--border-color, #d8dee5);
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 100;
    min-width: 200px;
    overflow: visible;
    padding: 2px;
  }

  .dropdown-item {
    display: block;
    width: calc(100% - 4px);
    padding: 0.5rem 0.75rem;
    border: none;
    background: transparent;
    color: var(--text);
    cursor: pointer;
    font-size: 0.8rem;
    text-align: left;
    transition: background-color 0.2s ease;
    box-sizing: border-box;
    margin: 2px;
    border-radius: 6px;
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
