<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import Cpu from "phosphor-svelte/lib/Cpu";
  import GraphicsCard from "phosphor-svelte/lib/GraphicsCard";
  
  const dispatch = createEventDispatcher();
  
  export let use_gpu = false;
  export let cuda_available = false;
  export let cuda_build = false;
</script>

<div class="param">
  <div class="row" style="align-items:center; gap: 12px; margin-bottom: 6px;">
    <label for="device-toggle">Устройство инференса</label>
    <div class="segmented-toggle" title={!cuda_build ? 'Сборка без CUDA' : ''}>
      <button
        type="button"
        class="segment" class:active={!use_gpu}
        aria-label="Процессор"
        on:click={() => dispatch('device-toggle', { checked: false })}
      >
        <Cpu size={18} />
        <span>ЦП</span>
      </button>
      <button
        type="button"
        class="segment" class:active={use_gpu} disabled={!cuda_build}
        aria-label="Графический процессор"
        on:click={() => dispatch('device-toggle', { checked: true })}
        title={!cuda_build ? 'Сборка без CUDA' : (!cuda_available ? 'Попытка переключить CUDA (может завершиться ошибкой)' : 'GPU (CUDA)')}
      >
        <GraphicsCard size={18} />
        <span>ГП</span>
      </button>
    </div>
  </div>
</div>