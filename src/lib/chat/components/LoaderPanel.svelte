<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import DeviceSelector from './loader/DeviceSelector.svelte';
  import ContextLengthSelector from './loader/ContextLengthSelector.svelte';
  import HubModelForm from './loader/HubModelForm.svelte';
  import LoadingStatus from './loader/LoadingStatus.svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';

  const dispatch = createEventDispatcher();

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
    supports_text: _supports_text = false,
    supports_image: _supports_image = false,
    supports_audio: _supports_audio = false,
    supports_video: _supports_video = false,
    onMainAction: _onMainAction = undefined,
    children,
  } = $props();
  let tokensDump = $state('');
  let dumpUnlisten: UnlistenFn | null = null;
  onMount(async () => {
    try {
      dumpUnlisten = await listen<string>('prompt_tokens_dump', (e) => {
        tokensDump = String(e.payload || '');
      });
    } catch {}
  });
  onDestroy(() => {
    if (dumpUnlisten) dumpUnlisten();
    dumpUnlisten = null;
  });
</script>

<section class="loader">
  <!-- format selection buttons removed (GGUF / HF Hub) — upload controls moved to header -->
  <!-- Панель индикаторов модальностей удалена по требованию -->

  {#if format === 'gguf' || format === 'local_safetensors'}
    <DeviceSelector
      bind:use_gpu
      bind:cuda_available
      bind:cuda_build
      on:device-toggle={(e) => dispatch('device-toggle', e.detail)}
    />

    <!-- thinking toggle removed -->

    <ContextLengthSelector bind:ctx_limit_value />

    <div class="param">
      <div class="row">
        <input id="opt-split" type="checkbox" bind:checked={split_prompt} />
        <label for="opt-split">Split prompt</label>
      </div>
    </div>

    <div class="param">
      <div class="row">
        <input id="opt-verbose" type="checkbox" bind:checked={verbose_prompt} />
        <label for="opt-verbose">Verbose prompt</label>
      </div>
    </div>

    <div class="param">
      <div class="row">
        <input id="opt-tracing" type="checkbox" bind:checked={tracing} />
        <label for="opt-tracing">Chrome tracing</label>
      </div>
    </div>

    <div class="param">
      <div class="row" style="gap:8px; flex-wrap: wrap;">
        <span>CPU:</span>
        <span class="chip" class:active={avx}>AVX</span>
        <span class="chip" class:active={neon}>NEON</span>
        <span class="chip" class:active={simd128}>SIMD128</span>
        <span class="chip" class:active={f16c}>F16C</span>
      </div>
    </div>

    <LoadingStatus
      bind:isLoadingModel
      bind:isCancelling
      bind:loadingStage
      bind:loadingProgress
      bind:errorText
    />
  {:else if format === 'hub_gguf'}
    <DeviceSelector
      bind:use_gpu
      bind:cuda_available
      bind:cuda_build
      on:device-toggle={(e) => dispatch('device-toggle', e.detail)}
    />

    <HubModelForm bind:repoId bind:revision bind:hubGgufFilename />

    <ContextLengthSelector bind:ctx_limit_value />

    <div class="param">
      <div class="row">
        <input id="opt-split" type="checkbox" bind:checked={split_prompt} />
        <label for="opt-split">Split prompt</label>
      </div>
    </div>

    <div class="param">
      <div class="row">
        <input id="opt-verbose" type="checkbox" bind:checked={verbose_prompt} />
        <label for="opt-verbose">Verbose prompt</label>
      </div>
    </div>

    <div class="param">
      <div class="row">
        <input id="opt-tracing" type="checkbox" bind:checked={tracing} />
        <label for="opt-tracing">Chrome tracing</label>
      </div>
    </div>

    <div class="param">
      <div class="row" style="gap:8px; flex-wrap: wrap;">
        <span>CPU:</span>
        <span class="chip" class:active={avx}>AVX</span>
        <span class="chip" class:active={neon}>NEON</span>
        <span class="chip" class:active={simd128}>SIMD128</span>
        <span class="chip" class:active={f16c}>F16C</span>
      </div>
    </div>

    <LoadingStatus
      bind:isLoadingModel
      bind:isCancelling
      bind:loadingStage
      bind:loadingProgress
      bind:errorText
    />
  {:else}
    <DeviceSelector
      bind:use_gpu
      bind:cuda_available
      bind:cuda_build
      on:device-toggle={(e) => dispatch('device-toggle', e.detail)}
    />

    <HubModelForm bind:repoId bind:revision isSafetensors={true} />

    <ContextLengthSelector bind:ctx_limit_value />

    <div class="param">
      <div class="row">
        <input id="opt-split" type="checkbox" bind:checked={split_prompt} />
        <label for="opt-split">Split prompt</label>
      </div>
    </div>

    <div class="param">
      <div class="row">
        <input id="opt-verbose" type="checkbox" bind:checked={verbose_prompt} />
        <label for="opt-verbose">Verbose prompt</label>
      </div>
    </div>

    <div class="param">
      <div class="row">
        <input id="opt-tracing" type="checkbox" bind:checked={tracing} />
        <label for="opt-tracing">Chrome tracing</label>
      </div>
    </div>

    <div class="param">
      <div class="row" style="gap:8px; flex-wrap: wrap;">
        <span>CPU:</span>
        <span class="chip" class:active={avx}>AVX</span>
        <span class="chip" class:active={neon}>NEON</span>
        <span class="chip" class:active={simd128}>SIMD128</span>
        <span class="chip" class:active={f16c}>F16C</span>
      </div>
    </div>

    <LoadingStatus
      bind:isLoadingModel
      bind:isCancelling
      bind:loadingStage
      bind:loadingProgress
      bind:errorText
    />
  {/if}
  {@render children?.default?.()}
  {#if verbose_prompt && tokensDump}
    <div class="param">
      <div class="head">Prompt tokens</div>
      <pre class="dump">{tokensDump}</pre>
    </div>
  {/if}
</section>

<style>
  .chip {
    padding: 2px 8px;
    border-radius: 10px;
    border: 1px solid #777;
    color: #777;
    font-size: 12px;
  }
  .chip.active {
    border-color: #2a7;
    color: #2a7;
  }
  .dump {
    max-height: 160px;
    overflow: auto;
    background: #1114;
    padding: 8px;
    border-radius: 6px;
  }
</style>
