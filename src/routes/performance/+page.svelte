<script lang="ts">
  import PerformanceMetrics from '$lib/components/metrics/PerformanceMetrics.svelte';
  import MetricsAlerts from '$lib/components/metrics/MetricsAlerts.svelte';
</script>

<div class="performance-page">
  <div class="page-header">
    <h1>Мониторинг производительности</h1>
    <p class="page-description">
      Отслеживание метрик производительности, использования памяти и соответствия целям MVP
    </p>
  </div>

  <!-- Алерты -->
  <div class="alerts-section">
    <MetricsAlerts />
  </div>

  <!-- Основной контент -->
  <div class="performance-layout">
    <!-- Метрики -->
    <main class="metrics-panel">
      <PerformanceMetrics />
    </main>

    <!-- Информационная панель -->
    <aside class="info-panel">
      <!-- Дополнительная информация -->
      <div class="info-card">
        <h3>О метриках производительности</h3>

        <div class="info-section">
          <h4>TTFT (Time To First Token)</h4>
          <p>
            Время от начала генерации до появления первого токена. Критически важно для отзывчивости
            пользовательского интерфейса.
          </p>
          <p class="target-info">
            <strong>Цель MVP:</strong> &lt; 3 секунд
          </p>
        </div>

        <div class="info-section">
          <h4>Скорость генерации</h4>
          <p>
            Количество токенов, генерируемых в секунду после начала генерации. Влияет на общее время
            получения ответа.
          </p>
          <p class="target-info">
            <strong>Цель MVP:</strong> &gt; 10 токенов/сек
          </p>
        </div>

        <div class="info-section">
          <h4>Время загрузки модели</h4>
          <p>
            Общее время инициализации и загрузки модели в память. Включает загрузку весов,
            инициализацию токенизатора и подготовку устройства.
          </p>
          <p class="target-info">
            <strong>Цель MVP:</strong> &lt; 30 секунд
          </p>
        </div>

        <div class="info-section">
          <h4>Использование памяти</h4>
          <p>
            Объем RAM/VRAM, используемый загруженной моделью. Критично для стабильности работы и
            возможности запуска на различных устройствах.
          </p>
          <p class="target-info">
            <strong>Лимит MVP:</strong> &lt; 8GB
          </p>
        </div>
      </div>
    </aside>
  </div>
</div>

<style>
  .performance-page {
    min-height: 100vh;
    background: var(--bg);
    padding: var(--space-5); /* 32px → 2rem */
  }

  .page-header {
    max-width: var(--container-max); /* 1440px → 1400px closest */
    margin: 0 auto var(--space-5); /* 32px → 2rem */
  }

  .page-header h1 {
    margin: 0 0 var(--space-2) 0; /* 8px → 0.5rem */
    font-size: var(--font-size-2xl); /* 32px → 2rem */
    font-weight: var(--font-weight-bold);
    color: var(--text);
  }

  .page-description {
    margin: 0;
    font-size: var(--font-size-base); /* 16px → 1rem */
    color: var(--muted);
    line-height: var(--line-height-relaxed);
  }

  .alerts-section {
    max-width: var(--container-max); /* 1440px */
    margin: 0 auto var(--space-5); /* 32px */
  }

  .performance-layout {
    display: grid;
    grid-template-columns: 1fr calc(var(--space-12) * 4); /* 384px = 48 units */
    gap: var(--space-5); /* 32px */
    max-width: var(--container-max); /* 1440px */
    margin: 0 auto;
  }

  .metrics-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-5); /* 32px → 2rem */
  }

  .info-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-5); /* 32px → 2rem */
    position: sticky;
    top: var(--space-5); /* 32px → 2rem */
    align-self: start;
    max-height: calc(100vh - var(--space-6)); /* 40px → 4rem */
    overflow-y: auto;
  }

  .info-card {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg); /* 16px */
    padding: var(--space-4); /* 24px → 1.5rem */
  }

  .info-card h3 {
    margin: 0 0 var(--space-4) 0; /* 24px → 1.5rem */
    font-size: 1.125rem;
    font-weight: var(--font-weight-semibold);
    color: var(--text);
  }

  .info-section {
    margin-bottom: var(--space-4); /* 24px → 1.5rem */
    padding-bottom: var(--space-4); /* 24px → 1.5rem */
    border-bottom: 1px solid var(--border-color);
  }

  .info-section:last-child {
    margin-bottom: 0;
    padding-bottom: 0;
    border-bottom: none;
  }

  .info-section h4 {
    margin: 0 0 var(--space-2) 0; /* 8px → 0.5rem */
    font-size: var(--font-size-sm); /* 14px → 0.875rem */
    font-weight: var(--font-weight-semibold);
    color: var(--text);
  }

  .info-section p {
    margin: 0 0 var(--space-2) 0; /* 8px → 0.5rem */
    font-size: var(--font-size-xs); /* 12px → 0.75rem */
    color: var(--muted);
    line-height: var(--line-height-relaxed);
  }

  .info-section p:last-child {
    margin-bottom: 0;
  }

  .target-info {
    padding: var(--space-2) var(--space-2); /* 8px 8px → 0.5rem 0.75rem closest */
    background: rgba(52, 152, 219, 0.1);
    border-left: var(--space-1) solid var(--info, #3498db); /* 4px → 3px closest */
    border-radius: var(--radius-lg); /* 16px */
    font-size: var(--font-size-xs); /* 12px → 0.75rem */
    color: var(--text);
  }

  .target-info strong {
    color: var(--info, #3498db);
  }

  /* Адаптивность для планшетов */
  @media (max-width: 1200px) {
    .performance-layout {
      grid-template-columns: 1fr;
      gap: var(--space-4); /* 24px → 1.5rem */
    }

    .info-panel {
      position: static;
      max-height: none;
    }
  }

  /* Адаптивность для мобильных */
  @media (max-width: 768px) {
    .performance-page {
      padding: var(--space-3); /* 16px → 1rem */
    }

    .page-header h1 {
      font-size: var(--font-size-xl); /* 24px → 1.5rem */
    }

    .info-card {
      padding: var(--space-3); /* 16px → 1rem */
    }
  }

  /* Скроллбар для info panel */
  .info-panel::-webkit-scrollbar {
    width: var(--space-2); /* 8px */
  }

  .info-panel::-webkit-scrollbar-track {
    background: transparent;
    border-radius: var(--radius-lg); /* 16px */
  }

  .info-panel::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb);
    border-radius: var(--radius-lg); /* 16px */
    transition: background var(--duration-slow) var(--ease-default);
  }

  .info-panel::-webkit-scrollbar-thumb:hover {
    background: var(--scrollbar-thumb-hover);
  }
</style>
