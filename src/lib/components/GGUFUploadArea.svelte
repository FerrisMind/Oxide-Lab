<script lang="ts">
  
  import Binoculars from 'phosphor-svelte/lib/Binoculars';
  import DownloadSimple from 'phosphor-svelte/lib/DownloadSimple';
  import UploadSimple from 'phosphor-svelte/lib/UploadSimple';
  import CircleNotch from 'phosphor-svelte/lib/CircleNotch';
  import { chatState } from '$lib/stores/chat';

  function pickModel() {
    if ((window as any).__oxide?.pickModel) (window as any).__oxide.pickModel();
  }

  function mainAction() {
    const ox = (window as any).__oxide;
    if (!ox) return;
    if ($chatState.isLoadingModel && ox.cancelLoading) return ox.cancelLoading();
    if ($chatState.isLoaded && ox.unloadGGUF) return ox.unloadGGUF();
    if (ox.loadGGUF) return ox.loadGGUF();
  }
</script>

<div class="gguf-upload-area">
  <div class="file-row">
    <div class="input-with-button">
      <input class="gguf-input" placeholder="Выбрать GGUF файл" value={$chatState.modelPath} readonly />
      <button class="inside-btn" type="button" on:click={pickModel} aria-label="Выбрать файл модели"><Binoculars size={16} weight="bold" /></button>
    </div>
    <button class="primary" on:click={mainAction} disabled={$chatState.busy} title="Загрузить/Выгрузить">
      {#if $chatState.isLoadingModel}
        <CircleNotch size={16} class="spinning" /> <span>Загрузка... {Math.round($chatState.loadingProgress)}%</span>
      {:else if $chatState.isUnloadingModel}
        <CircleNotch size={16} class="spinning" /> <span>Выгрузка... {Math.round($chatState.unloadingProgress)}%</span>
      {:else if $chatState.isLoaded}
        <UploadSimple size={16} /> <span>Выгрузить</span>
      {:else}
        <DownloadSimple size={16} /> <span>Загрузить</span>
      {/if}
    </button>
  </div>
</div>

<style>
  .gguf-upload-area { display:inline-flex; align-items:center; gap:12px; height:100%; }
  .file-row { display:flex; gap:8px; align-items:center; height:100%; }
  .input-with-button { display:inline-flex; align-items:center; gap:8px; position: relative; }
  .gguf-input { width:280px; padding:6px 8px; padding-right:44px; border-radius:6px; border:1px solid var(--border-color); background:var(--card); color:var(--text); height:36px; line-height:24px; }
  .inside-btn {
    position: absolute;
    right: 4px;
    top: 50%;
    transform: translateY(-50%) !important; /* override global button hover transform */
    width: 24px !important;
    height: 24px !important;
    padding: 0 !important; /* remove extra internal padding */
    box-sizing: border-box;
    border-radius: 4px;
    border: none;
    background: transparent;
    cursor: default;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 0;
    transition: background 0.12s ease;
  }
  /* фиксируем размер svg-иконки чтобы она не масштабировалась */
  .inside-btn :global(svg) { color: var(--muted); width:16px; height:16px; }
  .inside-btn:hover {
    background: var(--accent);
    transform: translateY(-50%) !important;
  }
  .inside-btn:hover :global(svg) { color: #fff; }
  .inside-btn:active {
    background: color-mix(in srgb, var(--accent) 80%, black 10%);
  }
  .primary {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding:6px 10px;
    border-radius:6px;
    border:none;
    background:var(--primary);
    color:#fff;
    cursor: default;
    white-space: nowrap;
    line-height: 1;
    align-self: center;
    height: 36px;
    transform: none !important;
    margin-top: 0 !important; /* avoid baseline shift from global .primary */
    position: relative;
    top: -1px; /* lower by 1px from previous adjustment */
  }
  /* classic toolbar-like hover: slight lift, subtle shadow, color shift */
  .primary:hover {
    box-shadow: var(--shadow-hover);
    background: var(--accent-2);
    transform: none !important;
  }
  .primary:active {
    box-shadow: none;
    background: var(--primary);
  }
  .primary:disabled { opacity:0.6; cursor:not-allowed; }
</style>


