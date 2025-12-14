<script lang="ts">
  import type { HFModel } from '$lib/services/huggingface';
  import { formatNumber, formatDate } from './formatters';

  interface Props {
    model: HFModel;
  }

  let { model }: Props = $props();
</script>

<div class="model-stats">
  <div class="stat-item">
    <span class="stat-label">Загрузки</span>
    <span class="stat-value">{formatNumber(model.downloads)}</span>
  </div>
  <div class="stat-item">
    <span class="stat-label">Лайки</span>
    <span class="stat-value">{formatNumber(model.likes)}</span>
  </div>
  <div class="stat-item">
    <span class="stat-label">Обновлено</span>
    <span class="stat-value">{formatDate(model.lastModified)}</span>
  </div>
</div>

<style>
  .model-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: var(--space-3); /* 16px */
    margin-bottom: var(--space-4); /* 24px */
    padding: var(--space-3); /* 16px */
    background: var(--panel-alt-bg);
    border-radius: var(--radius); /* 8px → 10px closest */
    border: 1px solid var(--border-color);
  }

  .stat-item {
    text-align: center;
  }

  .stat-label {
    display: block;
    font-size: 0.875rem;
    color: var(--muted);
    margin-bottom: var(--space-2); /* 8px */
  }

  .stat-value {
    display: block;
    font-size: 1.25rem;
    font-weight: var(--font-weight-semibold);
    color: var(--text);
  }
</style>