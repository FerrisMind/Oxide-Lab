<script lang="ts">
  interface Props {
    use_custom_params: boolean;
    temperature_enabled: boolean;
    temperature: number;
    top_k_enabled: boolean;
    top_k_value: number;
    top_p_enabled: boolean;
    top_p_value: number;
    min_p_enabled: boolean;
    min_p_value: number;
    repeat_penalty_enabled: boolean;
    repeat_penalty_value: number;
  }

  let {
    use_custom_params = $bindable(),
    temperature_enabled = $bindable(),
    temperature = $bindable(),
    top_k_enabled = $bindable(),
    top_k_value = $bindable(),
    top_p_enabled = $bindable(),
    top_p_value = $bindable(),
    min_p_enabled = $bindable(),
    min_p_value = $bindable(),
    repeat_penalty_enabled = $bindable(),
    repeat_penalty_value = $bindable()
  }: Props = $props();
</script>

<div class="params">
  <div class="param">
    <div class="row">
      <input id="p-custom" type="checkbox" bind:checked={use_custom_params} />
      <label for="p-custom">Пользовательские параметры</label>
    </div>
  </div>
  {#if use_custom_params}
    <div class="param">
      <div class="head">
        <input id="p-temp-enabled" type="checkbox" bind:checked={temperature_enabled} disabled={!use_custom_params} />
        <label for="p-temp-enabled">Температура</label>
        <input class="value-input" type="number" min={0} max={2} step={0.01} bind:value={temperature} disabled={!use_custom_params || !temperature_enabled} />
      </div>
      <div class="slider-row">
        <input id="p-temperature" type="range" min={0} max={2} step={0.01} bind:value={temperature} disabled={!use_custom_params || !temperature_enabled} />
      </div>
    </div>

    <div class="param">
      <div class="head">
        <input id="p-topk-enabled" type="checkbox" bind:checked={top_k_enabled} disabled={!use_custom_params} />
        <label for="p-topk-enabled">Top-K</label>
        <input class="value-input" id="p-topk" type="number" min={1} step={1} bind:value={top_k_value} disabled={!use_custom_params || !top_k_enabled} />
      </div>
    </div>

    <div class="param">
      <div class="head">
        <input id="p-topp-enabled" type="checkbox" bind:checked={top_p_enabled} disabled={!use_custom_params} />
        <label for="p-topp-enabled">Top-P (0..1)</label>
        <input class="value-input" type="number" min={0.01} max={1} step={0.01} bind:value={top_p_value} disabled={!use_custom_params || !top_p_enabled} />
      </div>
      <div class="slider-row">
        <input id="p-topp" type="range" min={0.01} max={1} step={0.01} bind:value={top_p_value} disabled={!use_custom_params || !top_p_enabled} />
      </div>
    </div>

    <div class="param">
      <div class="head">
        <input id="p-minp-enabled" type="checkbox" bind:checked={min_p_enabled} disabled={!use_custom_params} />
        <label for="p-minp-enabled">Min-P (0..1)</label>
        <input class="value-input" type="number" min={0.001} max={1} step={0.001} bind:value={min_p_value} disabled={!use_custom_params || !min_p_enabled} />
      </div>
      <div class="slider-row">
        <input id="p-minp" type="range" min={0.001} max={1} step={0.001} bind:value={min_p_value} disabled={!use_custom_params || !min_p_enabled} />
      </div>
    </div>

    <div class="param">
      <div class="head">
        <input id="p-rp-enabled" type="checkbox" bind:checked={repeat_penalty_enabled} disabled={!use_custom_params} />
        <label for="p-rp-enabled">Repeat penalty</label>
        <input class="value-input" type="number" min={0.8} max={2} step={0.01} bind:value={repeat_penalty_value} disabled={!use_custom_params || !repeat_penalty_enabled} />
      </div>
      <div class="slider-row">
        <input id="p-rp" type="range" min={0.8} max={2} step={0.01} bind:value={repeat_penalty_value} disabled={!use_custom_params || !repeat_penalty_enabled} />
      </div>
    </div>
  {/if}
</div>


