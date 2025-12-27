<script lang="ts">
  /**
   * Loader Panel Component
   *
   * Model loading settings and status display in a Sheet panel.
   */
  import { t } from '$lib/i18n';
  import { Label } from '$lib/components/ui/label';
  import { Badge } from '$lib/components/ui/badge';
  import { Progress } from '$lib/components/ui/progress';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { cn } from '../../utils';
  import Cpu from 'phosphor-svelte/lib/Cpu';
  import GpuCard from 'phosphor-svelte/lib/GraphicsCard';
  import Check from 'phosphor-svelte/lib/Check';

  interface Props {
    format?: 'gguf' | 'hub_gguf' | 'hub_safetensors' | 'local_safetensors';
    modelPath?: string;
    repoId?: string;
    revision?: string;
    hubGgufFilename?: string;
    ctx_limit_value?: number;
    isLoadingModel?: boolean;
    isUnloadingModel?: boolean;
    isCancelling?: boolean;
    loadingStage?: string;
    loadingProgress?: number;
    unloadingProgress?: number;
    errorText?: string;
    busy?: boolean;
    isLoaded?: boolean;
    use_gpu?: boolean;
    cuda_available?: boolean;
    cuda_build?: boolean;
    avx?: boolean;
    neon?: boolean;
    simd128?: boolean;
    f16c?: boolean;
    split_prompt?: boolean;
    verbose_prompt?: boolean;
    tracing?: boolean;
    onDeviceToggle?: () => void;
    class?: string;
  }

  let {
    format = $bindable('gguf'),
    modelPath = $bindable(''),
    repoId = $bindable(''),
    revision = $bindable(''),
    hubGgufFilename = $bindable(''),
    ctx_limit_value = $bindable(4096),
    isLoadingModel = $bindable(false),
    isUnloadingModel = $bindable(false),
    isCancelling = $bindable(false),
    loadingStage = $bindable(''),
    loadingProgress = $bindable(0),
    unloadingProgress = $bindable(0),
    errorText = $bindable(''),
    busy = $bindable(false),
    isLoaded = $bindable(false),
    use_gpu = $bindable(false),
    cuda_available = $bindable(false),
    cuda_build = $bindable(false),
    avx = $bindable(false),
    neon = $bindable(false),
    simd128 = $bindable(false),
    f16c = $bindable(false),
    split_prompt = $bindable(false),
    verbose_prompt = $bindable(false),
    tracing = $bindable(false),
    onDeviceToggle,
    class: className = '',
  }: Props = $props();

  function toggleDevice() {
    if (onDeviceToggle) {
      onDeviceToggle();
    } else {
      use_gpu = !use_gpu;
    }
  }

  const contextOptions = [2048, 4096, 8192, 16384, 32768];
</script>

<section class={cn('loader-panel space-y-6', className)}>
  <!-- Device Selector -->
  <div class="space-y-3">
    <Label class="text-sm font-medium">{$t('common.loader.device') || 'Device'}</Label>
    <div class="flex gap-2">
      <button
        type="button"
        class={cn(
          'flex-1 flex items-center justify-center gap-2 p-3 rounded-lg border transition-all',
          !use_gpu
            ? 'border-primary bg-primary/10 text-primary'
            : 'border-border hover:border-muted-foreground',
        )}
        onclick={() => {
          use_gpu = false;
        }}
      >
        <Cpu class="size-5" />
        <span>CPU</span>
        {#if !use_gpu}
          <Check class="size-4 ml-auto" />
        {/if}
      </button>
      <button
        type="button"
        class={cn(
          'flex-1 flex items-center justify-center gap-2 p-3 rounded-lg border transition-all',
          !cuda_available && !cuda_build && 'opacity-50 cursor-not-allowed',
          use_gpu
            ? 'border-primary bg-primary/10 text-primary'
            : 'border-border hover:border-muted-foreground',
        )}
        onclick={toggleDevice}
        disabled={!cuda_available && !cuda_build}
      >
        <GpuCard class="size-5" />
        <span>GPU</span>
        {#if use_gpu}
          <Check class="size-4 ml-auto" />
        {/if}
      </button>
    </div>
    {#if !cuda_available && !cuda_build}
      <p class="text-xs text-muted-foreground">
        {$t('common.loader.gpuNotAvailable') || 'GPU not available (CUDA not detected)'}
      </p>
    {/if}
  </div>

  <!-- Context Length -->
  <div class="space-y-3">
    <Label class="text-sm font-medium"
      >{$t('common.loader.contextLength') || 'Context Length'}</Label
    >
    <div class="flex flex-wrap gap-2">
      {#each contextOptions as option}
        <button
          type="button"
          class={cn(
            'px-3 py-1.5 rounded-md text-sm border transition-all',
            ctx_limit_value === option
              ? 'border-primary bg-primary/10 text-primary font-medium'
              : 'border-border hover:border-muted-foreground',
          )}
          onclick={() => {
            ctx_limit_value = option;
          }}
        >
          {option.toLocaleString()}
        </button>
      {/each}
    </div>
  </div>

  <!-- CPU Features -->
  <div class="space-y-3">
    <Label class="text-sm font-medium">{$t('common.loader.cpuFeatures') || 'CPU Features'}</Label>
    <div class="flex flex-wrap gap-2">
      <Badge variant={avx ? 'default' : 'outline'} class={cn(!avx && 'opacity-50')}>AVX</Badge>
      <Badge variant={neon ? 'default' : 'outline'} class={cn(!neon && 'opacity-50')}>NEON</Badge>
      <Badge variant={simd128 ? 'default' : 'outline'} class={cn(!simd128 && 'opacity-50')}>
        SIMD128
      </Badge>
      <Badge variant={f16c ? 'default' : 'outline'} class={cn(!f16c && 'opacity-50')}>F16C</Badge>
    </div>
  </div>

  <!-- Advanced Options -->
  <div class="space-y-3">
    <Label class="text-sm font-medium"
      >{$t('common.loader.advancedOptions') || 'Advanced Options'}</Label
    >
    <div class="space-y-3">
      <div class="flex items-center gap-2">
        <Checkbox id="split-prompt" bind:checked={split_prompt} />
        <Label for="split-prompt" class="text-sm cursor-pointer">
          {$t('common.loader.splitPrompt') || 'Split prompt'}
        </Label>
      </div>
      <div class="flex items-center gap-2">
        <Checkbox id="verbose-prompt" bind:checked={verbose_prompt} />
        <Label for="verbose-prompt" class="text-sm cursor-pointer">
          {$t('common.loader.verbosePrompt') || 'Verbose prompt'}
        </Label>
      </div>
      <div class="flex items-center gap-2">
        <Checkbox id="tracing" bind:checked={tracing} />
        <Label for="tracing" class="text-sm cursor-pointer">
          {$t('common.loader.chromeTracing') || 'Chrome tracing'}
        </Label>
      </div>
    </div>
  </div>

  <!-- Loading Status -->
  {#if isLoadingModel || isUnloadingModel}
    <div class="space-y-3 p-4 rounded-lg border bg-muted/50">
      <div class="flex items-center justify-between">
        <span class="text-sm font-medium">
          {#if isLoadingModel}
            {$t('common.loader.loading') || 'Loading model...'}
          {:else}
            {$t('common.loader.unloading') || 'Unloading model...'}
          {/if}
        </span>
        {#if loadingStage}
          <Badge variant="outline">{loadingStage}</Badge>
        {/if}
      </div>
      <Progress value={isLoadingModel ? loadingProgress : unloadingProgress} class="h-2" />
      {#if isCancelling}
        <p class="text-xs text-muted-foreground">
          {$t('common.loader.cancelling') || 'Cancelling...'}
        </p>
      {/if}
    </div>
  {/if}

  <!-- Error Display -->
  {#if errorText}
    <div class="p-4 rounded-lg border border-destructive/50 bg-destructive/10">
      <p class="text-sm text-destructive">{errorText}</p>
    </div>
  {/if}

  <!-- Model Info -->
  {#if isLoaded && modelPath}
    <div class="p-4 rounded-lg border bg-muted/30">
      <div class="flex items-center gap-2 mb-2">
        <Badge variant="default">{$t('common.loader.loaded') || 'Loaded'}</Badge>
      </div>
      <p class="text-xs text-muted-foreground truncate">{modelPath}</p>
    </div>
  {/if}
</section>
