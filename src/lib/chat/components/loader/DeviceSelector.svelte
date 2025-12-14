<script lang="ts">
  import Cpu from 'phosphor-svelte/lib/Cpu';
  import GraphicsCard from 'phosphor-svelte/lib/GraphicsCard';

  interface Props {
    use_gpu?: boolean;
    cuda_available?: boolean;
    cuda_build?: boolean;
    onDeviceToggle?: (detail: { checked: boolean }) => void;
  }

  let {
    use_gpu = $bindable(false),
    cuda_available = $bindable(false),
    cuda_build = $bindable(false),
    onDeviceToggle,
  }: Props = $props();
</script>

<div class="param">
  <div class="row" style="align-items:center; gap: var(--space-3); margin-bottom: var(--space-2);">
    <label for="device-toggle">Устройство инференса</label>
    <div class="segmented-toggle" title={!cuda_build ? 'Сборка без CUDA' : ''}>
      <button
        type="button"
        class="segment"
        class:active={!use_gpu}
        aria-label="Процессор"
        onclick={() => onDeviceToggle?.({ checked: false })}
      >
        <Cpu size={18} />
        <span>ЦП</span>
      </button>
      <button
        type="button"
        class="segment"
        class:active={use_gpu}
        disabled={!cuda_build}
        aria-label="Графический процессор"
        onclick={() => onDeviceToggle?.({ checked: true })}
        title={!cuda_build
          ? 'Сборка без CUDA'
          : !cuda_available
            ? 'Попытка переключить CUDA (может завершиться ошибкой)'
            : 'GPU (CUDA)'}
      >
        <GraphicsCard size={18} />
        <span>ГП</span>
      </button>
    </div>
  </div>
</div>
