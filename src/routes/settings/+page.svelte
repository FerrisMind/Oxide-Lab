<script lang="ts">
  /**
   * Settings Page
   * 
   * Complete application settings including precision policy, threads, STT, 
   * model selector, experimental features, performance monitor, and language.
   */
  import { onMount, tick } from 'svelte';
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Label } from '$lib/components/ui/label';
  import { Badge } from '$lib/components/ui/badge';
  import { Input } from '$lib/components/ui/input';
  import { Spinner } from '$lib/components/ui/spinner';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import Gear from 'phosphor-svelte/lib/Gear';
  import Cpu from 'phosphor-svelte/lib/Cpu';
  import Globe from 'phosphor-svelte/lib/Globe';
  import Flask from 'phosphor-svelte/lib/Flask';
  import Info from 'phosphor-svelte/lib/Info';
  import Check from 'phosphor-svelte/lib/Check';
  import Microphone from 'phosphor-svelte/lib/Microphone';
  import MagnifyingGlass from 'phosphor-svelte/lib/MagnifyingGlass';
  import ChartBar from 'phosphor-svelte/lib/ChartBar';
  import FolderOpen from 'phosphor-svelte/lib/FolderOpen';
  import DownloadSimple from 'phosphor-svelte/lib/DownloadSimple';
  import Warning from 'phosphor-svelte/lib/Warning';
  import { t, locale, setLocale, loadTranslations, type SupportedLocale } from '$lib/i18n';
  import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';
  import { modelSelectorSearchEnabled } from '$lib/stores/ui-preferences';
  import type { PrecisionPolicy } from '$lib/types';
  import type { SttModelSource, SttSettings } from '$lib/types/stt';
  import PerformanceMonitor from '$lib/components/PerformanceMonitor.svelte';

  // ─────────────────────────────────────────────────────────────
  // State
  // ─────────────────────────────────────────────────────────────

  // Precision Policy
  let currentPolicy = $state<PrecisionPolicy>({ Default: null });
  let policyLoading = $state(true);
  let policyError = $state<string | null>(null);

  // Thread Limit
  const hardwareConcurrency = typeof navigator !== 'undefined' ? navigator.hardwareConcurrency || 4 : 4;
  let threadLimit = $state<number | null>(null);
  let threadSliderValue = $state(hardwareConcurrency);
  let threadLimitLoading = $state(true);
  let threadLimitError = $state<string | null>(null);

  // STT Settings
  let sttSettings = $state<SttSettings>({ source: 'bundled', custom_dir: null });
  let sttLoading = $state(true);
  let sttError = $state<string | null>(null);
  let sttSource = $state<SttModelSource>('bundled');
  let sttCustomDir = $state('');
  let sttRepoId = $state('lmz/candle-whisper');
  let sttRevision = $state('main');
  let sttModelFilename = $state('model-tiny-q80.gguf');
  let sttTokenizerFilename = $state('tokenizer-tiny.json');
  let sttConfigFilename = $state('config-tiny.json');
  let sttDownloadLoading = $state(false);

  // Experimental Features
  let experimentalEnabled = $state(false);

  // Model Selector Search
  let modelSearchEnabled = $state(true);

  // Languages
  const languages: { value: SupportedLocale; label: string }[] = [
    { value: 'en', label: 'English' },
    { value: 'ru', label: 'Русский' },
    { value: 'pt-BR', label: 'Português (Brasil)' },
  ];

  // ─────────────────────────────────────────────────────────────
  // Precision Policy
  // ─────────────────────────────────────────────────────────────

  async function loadPrecisionPolicy() {
    policyLoading = true;
    policyError = null;
    try {
      // TODO: Integrate with Tauri backend
      const { invoke } = await import('@tauri-apps/api/core');
      currentPolicy = await invoke<PrecisionPolicy>('get_precision_policy');
    } catch (err) {
      policyError = `Failed to load precision policy: ${err}`;
      console.error(err);
    } finally {
      policyLoading = false;
    }
  }

  async function savePrecisionPolicy(policy: PrecisionPolicy) {
    policyLoading = true;
    policyError = null;
    try {
      // TODO: Integrate with Tauri backend
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_precision_policy', { policy });
      currentPolicy = policy;
    } catch (err) {
      policyError = `Failed to save precision policy: ${err}`;
      console.error(err);
    } finally {
      policyLoading = false;
    }
  }

  function selectPolicy(policyType: 'Default' | 'MemoryEfficient' | 'MaximumPrecision') {
    const policyMap: Record<string, PrecisionPolicy> = {
      'Default': { Default: null },
      'MemoryEfficient': { MemoryEfficient: null },
      'MaximumPrecision': { MaximumPrecision: null },
    };
    savePrecisionPolicy(policyMap[policyType]);
  }

  function isPolicySelected(policyType: 'Default' | 'MemoryEfficient' | 'MaximumPrecision'): boolean {
    if (typeof currentPolicy === 'string') return currentPolicy === policyType;
    if (typeof currentPolicy === 'object' && currentPolicy !== null) {
      return policyType in currentPolicy;
    }
    return false;
  }

  // ─────────────────────────────────────────────────────────────
  // Thread Limit
  // ─────────────────────────────────────────────────────────────

  async function loadThreadLimit() {
    threadLimitLoading = true;
    threadLimitError = null;
    try {
      // TODO: Integrate with Tauri backend
      const { invoke } = await import('@tauri-apps/api/core');
      const saved = await invoke<number | null>('get_rayon_thread_limit');
      threadLimit = saved;
      threadSliderValue = saved ?? hardwareConcurrency;
    } catch (err) {
      threadLimitError = `Failed to load thread limit: ${err}`;
      console.error(err);
    } finally {
      threadLimitLoading = false;
    }
  }

  async function applyThreadLimit(limit: number | null) {
    threadLimitLoading = true;
    threadLimitError = null;
    try {
      // TODO: Integrate with Tauri backend
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_rayon_thread_limit', { limit });
      threadLimit = limit;
      threadSliderValue = limit ?? hardwareConcurrency;
    } catch (err) {
      threadLimitError = `Failed to save thread limit: ${err}`;
      console.error(err);
    } finally {
      threadLimitLoading = false;
    }
  }

  // ─────────────────────────────────────────────────────────────
  // STT Settings
  // ─────────────────────────────────────────────────────────────

  async function loadSttSettings() {
    sttLoading = true;
    sttError = null;
    try {
      // TODO: Integrate with Tauri backend
      const { getSttSettings } = await import('$lib/services/stt-service');
      sttSettings = await getSttSettings();
      sttSource = sttSettings.source;
      sttCustomDir = sttSettings.custom_dir ?? '';
    } catch (err) {
      sttError = `Failed to load STT settings: ${err}`;
      console.error(err);
    } finally {
      sttLoading = false;
    }
  }

  async function updateSttSettings(next: SttSettings) {
    sttError = null;
    try {
      // TODO: Integrate with Tauri backend
      const { setSttSettings } = await import('$lib/services/stt-service');
      await setSttSettings(next);
      sttSettings = next;
      sttSource = next.source;
      sttCustomDir = next.custom_dir ?? '';
    } catch (err) {
      sttError = `Failed to save STT settings: ${err}`;
      console.error(err);
    }
  }

  async function setSttSource(nextSource: SttModelSource) {
    if (nextSource === 'custom' && !sttCustomDir) {
      sttError = $t('settings.stt.errors.customDirRequired') || 'Custom directory required';
      return;
    }
    await updateSttSettings({
      source: nextSource,
      custom_dir: nextSource === 'custom' ? sttCustomDir : null,
    });
  }

  async function pickSttDirectory() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected === 'string' && selected.length > 0) {
        sttCustomDir = selected;
        await updateSttSettings({ source: 'custom', custom_dir: selected });
      }
    } catch (err) {
      console.error('Failed to pick directory:', err);
    }
  }

  async function handleSttDownload() {
    sttDownloadLoading = true;
    sttError = null;
    try {
      const { downloadSttModel } = await import('$lib/services/stt-service');
      const { message } = await import('@tauri-apps/plugin-dialog');
      const response = await downloadSttModel({
        repo_id: sttRepoId,
        revision: sttRevision || null,
        model_filename: sttModelFilename,
        tokenizer_filename: sttTokenizerFilename,
        config_filename: sttConfigFilename,
      });
      sttCustomDir = response.model_dir;
      await updateSttSettings({ source: 'custom', custom_dir: response.model_dir });
      await message($t('settings.stt.download.success') || 'Model downloaded successfully', {
        title: $t('settings.stt.download.title') || 'STT Model',
        kind: 'info',
      });
    } catch (err) {
      sttError = `Failed to download STT model: ${err}`;
      console.error(err);
    } finally {
      sttDownloadLoading = false;
    }
  }

  // ─────────────────────────────────────────────────────────────
  // Experimental Features
  // ─────────────────────────────────────────────────────────────

  async function handleExperimentalToggle(enabled: boolean) {
    try {
      await experimentalFeatures.setEnabled(enabled);
      experimentalEnabled = enabled;
      await tick();
    } catch (err) {
      console.error('Failed to toggle experimental features:', err);
      experimentalEnabled = experimentalFeatures.enabled;
    }
  }

  // ─────────────────────────────────────────────────────────────
  // Model Selector
  // ─────────────────────────────────────────────────────────────

  function handleModelSearchToggle(enabled: boolean) {
    modelSelectorSearchEnabled.set(enabled);
  }

  // ─────────────────────────────────────────────────────────────
  // Language
  // ─────────────────────────────────────────────────────────────

  async function handleLanguageChange(lang: SupportedLocale) {
    await setLocale(lang);
    await loadTranslations(lang);
  }

  // ─────────────────────────────────────────────────────────────
  // Lifecycle
  // ─────────────────────────────────────────────────────────────

  onMount(async () => {
    await Promise.all([
      loadPrecisionPolicy(),
      loadThreadLimit(),
      loadSttSettings(),
    ]);
  });

  // Sync with stores
  $effect(() => {
    if (experimentalFeatures.initialized) {
      experimentalEnabled = experimentalFeatures.enabled;
    }
  });

  $effect(() => {
    const unsubscribe = modelSelectorSearchEnabled.subscribe((value) => {
      modelSearchEnabled = value;
    });
    return unsubscribe;
  });
</script>

<div class="h-full overflow-auto p-3 sm:p-4 lg:p-6 custom-scrollbar">
  <div class="max-w-xl sm:max-w-2xl lg:max-w-3xl mx-auto space-y-4 sm:space-y-6">
    <h1 class="text-xl sm:text-2xl font-bold">{$t('settings.title')}</h1>

    <!-- Precision Policy -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <Cpu class="size-5" />
          {$t('settings.precision.title') || 'Precision Policy'}
        </Card.Title>
        <Card.Description>
          {$t('settings.precision.description') || 'Control memory vs precision tradeoff'}
          <br />
          <span class="text-amber-500 text-xs font-medium">
            {$t('settings.precision.warning') || 'Requires model reload to take effect'}
          </span>
        </Card.Description>
      </Card.Header>
      <Card.Content>
        {#if policyLoading}
          <div class="flex justify-center py-4"><Spinner class="size-6" /></div>
        {:else}
          <div class="grid gap-3 sm:grid-cols-3">
            <Button
              variant={isPolicySelected('Default') ? 'default' : 'outline'}
              class="h-auto flex-col py-4 gap-1"
              onclick={() => selectPolicy('Default')}
            >
              <span class="font-semibold">{$t('settings.precision.default') || 'Default'}</span>
              <span class="text-xs opacity-70">{$t('settings.precision.balanced') || 'Balanced'}</span>
            </Button>
            <Button
              variant={isPolicySelected('MemoryEfficient') ? 'default' : 'outline'}
              class="h-auto flex-col py-4 gap-1"
              onclick={() => selectPolicy('MemoryEfficient')}
            >
              <span class="font-semibold">{$t('settings.precision.memoryEfficient') || 'Memory Efficient'}</span>
              <span class="text-xs opacity-70">{$t('settings.precision.lowerRam') || 'Lower RAM'}</span>
            </Button>
            <Button
              variant={isPolicySelected('MaximumPrecision') ? 'default' : 'outline'}
              class="h-auto flex-col py-4 gap-1"
              onclick={() => selectPolicy('MaximumPrecision')}
            >
              <span class="font-semibold">{$t('settings.precision.maximumPrecision') || 'Maximum Precision'}</span>
              <span class="text-xs opacity-70">{$t('settings.precision.bestQuality') || 'Best quality'}</span>
            </Button>
          </div>
        {/if}
        {#if policyError}
          <div class="mt-3 text-sm text-destructive flex items-center gap-2">
            <Warning class="size-4" />{policyError}
          </div>
        {/if}
      </Card.Content>
    </Card.Root>

    <!-- Thread Limit -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <Gear class="size-5" />
          {$t('settings.threads.title') || 'Thread Limit'}
        </Card.Title>
        <Card.Description>{$t('settings.threads.description') || 'Control CPU thread usage'}</Card.Description>
      </Card.Header>
      <Card.Content>
        {#if threadLimitLoading}
          <div class="flex justify-center py-4"><Spinner class="size-6" /></div>
        {:else}
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <Label>{$t('settings.threads.maxThreads') || 'Max threads'}: {threadSliderValue}</Label>
              <Badge variant="outline">{$t('settings.threads.available') || 'Available'}: {hardwareConcurrency}</Badge>
            </div>
            <input
              type="range"
              min="1"
              max={hardwareConcurrency}
              bind:value={threadSliderValue}
              onchange={(e) => applyThreadLimit(parseInt((e.target as HTMLInputElement).value))}
              class="w-full accent-primary"
            />
            <div class="flex items-center justify-between text-sm">
              <Button
                variant="ghost"
                size="sm"
                disabled={threadLimit === null}
                onclick={() => applyThreadLimit(null)}
              >
                {$t('settings.threads.useSystem') || 'Use system default'}
              </Button>
              <span class="text-muted-foreground">
                {threadLimit === null
                  ? $t('settings.threads.automatic') || 'Automatic'
                  : $t('settings.threads.manual') || 'Manual'}
              </span>
            </div>
          </div>
        {/if}
        {#if threadLimitError}
          <div class="mt-3 text-sm text-destructive flex items-center gap-2">
            <Warning class="size-4" />{threadLimitError}
          </div>
        {/if}
      </Card.Content>
    </Card.Root>

    <!-- STT Settings -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <Microphone class="size-5" />
          {$t('settings.stt.title') || 'Speech-to-Text'}
        </Card.Title>
        <Card.Description>{$t('settings.stt.description') || 'Configure voice input model'}</Card.Description>
      </Card.Header>
      <Card.Content class="space-y-4">
        {#if sttLoading}
          <div class="flex justify-center py-4"><Spinner class="size-6" /></div>
        {:else}
          <!-- Source Selection -->
          <div class="flex gap-4">
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="stt-source"
                value="bundled"
                checked={sttSource === 'bundled'}
                onchange={() => setSttSource('bundled')}
                class="accent-primary"
              />
              <span>{$t('settings.stt.sources.bundled') || 'Bundled'}</span>
            </label>
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="stt-source"
                value="custom"
                checked={sttSource === 'custom'}
                onchange={() => setSttSource('custom')}
                class="accent-primary"
              />
              <span>{$t('settings.stt.sources.custom') || 'Custom'}</span>
            </label>
          </div>

          <!-- Custom Path -->
          <div class="flex items-center gap-2">
            <div class="flex-1 px-3 py-2 rounded border bg-muted/30 text-sm font-mono truncate">
              {sttCustomDir || $t('settings.stt.customPathEmpty') || 'No custom path'}
            </div>
            <Button variant="outline" size="sm" onclick={pickSttDirectory}>
              <FolderOpen class="size-4 mr-1" />
              {$t('settings.stt.chooseFolder') || 'Choose'}
            </Button>
          </div>

          <!-- Download Section -->
          <div class="border rounded-lg p-4 space-y-4 bg-muted/20">
            <h4 class="font-medium flex items-center gap-2">
              <DownloadSimple class="size-4" />
              {$t('settings.stt.download.title') || 'Download Model'}
            </h4>
            <div class="grid gap-3 sm:grid-cols-2">
              <div class="space-y-1">
                <Label class="text-xs">{$t('settings.stt.download.repoId') || 'Repo ID'}</Label>
                <Input bind:value={sttRepoId} class="text-sm" />
              </div>
              <div class="space-y-1">
                <Label class="text-xs">{$t('settings.stt.download.revision') || 'Revision'}</Label>
                <Input bind:value={sttRevision} class="text-sm" />
              </div>
              <div class="space-y-1">
                <Label class="text-xs">{$t('settings.stt.download.modelFile') || 'Model File'}</Label>
                <Input bind:value={sttModelFilename} class="text-sm" />
              </div>
              <div class="space-y-1">
                <Label class="text-xs">{$t('settings.stt.download.tokenizerFile') || 'Tokenizer'}</Label>
                <Input bind:value={sttTokenizerFilename} class="text-sm" />
              </div>
              <div class="space-y-1 sm:col-span-2">
                <Label class="text-xs">{$t('settings.stt.download.configFile') || 'Config File'}</Label>
                <Input bind:value={sttConfigFilename} class="text-sm" />
              </div>
            </div>
            <Button onclick={handleSttDownload} disabled={sttDownloadLoading}>
              {#if sttDownloadLoading}
                <Spinner class="size-4 mr-2" />
                {$t('settings.stt.download.loading') || 'Downloading...'}
              {:else}
                <DownloadSimple class="size-4 mr-2" />
                {$t('settings.stt.download.button') || 'Download'}
              {/if}
            </Button>
          </div>
        {/if}
        {#if sttError}
          <div class="text-sm text-destructive flex items-center gap-2">
            <Warning class="size-4" />{sttError}
          </div>
        {/if}
      </Card.Content>
    </Card.Root>

    <!-- Model Selector Settings -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <MagnifyingGlass class="size-5" />
          {$t('settings.modelSelector.title') || 'Model Selector'}
        </Card.Title>
        <Card.Description>{$t('settings.modelSelector.description') || 'Configure model picker behavior'}</Card.Description>
      </Card.Header>
      <Card.Content>
        <label class="flex items-center gap-3 cursor-pointer">
          <Checkbox 
            checked={modelSearchEnabled}
            onCheckedChange={(checked: boolean) => handleModelSearchToggle(checked)}
          />
          <span>{$t('settings.modelSelector.enableSearch') || 'Enable search in model picker'}</span>
        </label>
        <p class="mt-2 text-sm text-muted-foreground">
          {modelSearchEnabled
            ? $t('settings.modelSelector.enabledDescription') || 'Search is enabled in the model selector'
            : $t('settings.modelSelector.disabledDescription') || 'Search is disabled in the model selector'}
        </p>
      </Card.Content>
    </Card.Root>

    <!-- Experimental Features -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <Flask class="size-5" />
          {$t('settings.experimental.title') || 'Experimental Features'}
        </Card.Title>
        <Card.Description>{$t('settings.experimental.description') || 'Try new features before release'}</Card.Description>
      </Card.Header>
      <Card.Content>
        <label class="flex items-center gap-3 cursor-pointer">
          <Checkbox 
            checked={experimentalEnabled}
            onCheckedChange={(checked: boolean) => handleExperimentalToggle(checked)}
          />
          <span>{$t('settings.experimental.enable') || 'Enable experimental features'}</span>
        </label>
        <div class="mt-4 flex items-start gap-2 text-sm text-muted-foreground">
          <Info class="size-4 mt-0.5 flex-shrink-0" />
          <p>{$t('settings.experimental.warning') || 'These features may be unstable'}</p>
        </div>
      </Card.Content>
    </Card.Root>

    <!-- Performance Monitor -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <ChartBar class="size-5" />
          {$t('settings.performance.title') || 'Performance'}
        </Card.Title>
        <Card.Description>{$t('settings.performance.description') || 'Monitor system performance'}</Card.Description>
      </Card.Header>
      <Card.Content>
        <PerformanceMonitor />
      </Card.Content>
    </Card.Root>

    <!-- Language -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <Globe class="size-5" />
          {$t('settings.language.title') || 'Language'}
        </Card.Title>
        <Card.Description>{$t('settings.language.description') || 'Select your preferred language'}</Card.Description>
      </Card.Header>
      <Card.Content>
        <div class="flex gap-2 flex-wrap">
          {#each languages as lang}
            <Button
              variant={$locale === lang.value ? 'default' : 'outline'}
              size="sm"
              class="gap-2"
              onclick={() => handleLanguageChange(lang.value)}
            >
              {#if $locale === lang.value}
                <Check class="size-4" />
              {/if}
              {lang.label}
            </Button>
          {/each}
        </div>
      </Card.Content>
    </Card.Root>

    <!-- About -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <Info class="size-5" />
          {$t('about.title') || 'About'}
        </Card.Title>
      </Card.Header>
      <Card.Content>
        <div class="space-y-2 text-sm">
          <div class="flex justify-between">
            <span class="text-muted-foreground">{$t('about.version') || 'Version'}</span>
            <span>0.13.1</span>
          </div>
          <div class="flex justify-between">
            <span class="text-muted-foreground">{$t('about.license') || 'License'}</span>
            <span>Apache-2.0</span>
          </div>
        </div>
      </Card.Content>
    </Card.Root>
  </div>
</div>
