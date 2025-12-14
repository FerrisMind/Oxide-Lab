<script lang="ts">
  /**
   * Language Switcher Component
   * 
   * Компонент для переключения языка интерфейса.
   * Синхронизирует выбор с Tauri Store и backend.
   */

  import { onMount } from 'svelte';
  import { locale, setLocale, syncLocaleWithBackend, SUPPORTED_LOCALES } from '$lib/i18n';
  import { page } from '$app/stores';
  import { invoke } from '@tauri-apps/api/core';
import Globe from 'phosphor-svelte/lib/Globe';
import Check from 'phosphor-svelte/lib/CheckCircle';
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
  import { Button } from '$lib/components/ui/button';
  import { cn } from '$lib/utils';

  // Названия языков для отображения
  const languageNames: Record<string, string> = {
    en: 'English',
    ru: 'Русский',
    'pt-BR': 'Português (BR)',
  };

  let currentLocale = $state<string>('en');
  let currentPath = $derived($page.url.pathname);
  let isOpen = $state(false);

  // Синхронизация с store
  $effect(() => {
    currentLocale = $locale;
  });

  onMount(async () => {
    // Загружаем сохраненную локаль из backend
    try {
      const savedLocale = await invoke<string>('get_locale');
      if (savedLocale && SUPPORTED_LOCALES.includes(savedLocale as any)) {
        await setLocale(savedLocale as any, currentPath);
        await syncLocaleWithBackend(savedLocale as any);
      }
    } catch (error) {
      console.warn('[LanguageSwitcher] Failed to load locale from backend:', error);
    }
  });

  async function handleLocaleChange(newLocale: string) {
    if (!SUPPORTED_LOCALES.includes(newLocale as any)) {
      console.warn(`[LanguageSwitcher] Unsupported locale: ${newLocale}`);
      return;
    }

    isOpen = false;
    // Устанавливаем локаль и загружаем переводы
    await setLocale(newLocale as any, currentPath);
    // Синхронизируем с backend
    await syncLocaleWithBackend(newLocale as any);
    
    // Принудительно обновляем текущую локаль для реактивности
    // Это гарантирует, что все компоненты обновятся с новыми переводами
    if (import.meta.env.DEV) {
      console.log(`[LanguageSwitcher] Locale changed to: ${newLocale}`);
    }
  }
</script>

<DropdownMenu.Root bind:open={isOpen}>
  <DropdownMenu.Trigger>
    <Button
      variant="ghost"
      size="sm"
      class="gap-2"
      aria-label="Switch language"
    >
      <Globe size={16} weight="regular" />
      <span>{languageNames[currentLocale] || currentLocale}</span>
    </Button>
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end" class="w-48">
    {#each SUPPORTED_LOCALES as loc}
      <DropdownMenu.Item
        class={cn('flex items-center justify-between', currentLocale === loc && 'bg-accent')}
        onclick={() => handleLocaleChange(loc)}
      >
        <span>{languageNames[loc] || loc}</span>
        {#if currentLocale === loc}
          <Check size={16} weight="bold" class="text-accent-foreground" />
        {/if}
      </DropdownMenu.Item>
    {/each}
  </DropdownMenu.Content>
</DropdownMenu.Root>

<style>
  :global([data-slot='dropdown-menu-item']) {
    cursor: pointer;
  }
</style>

