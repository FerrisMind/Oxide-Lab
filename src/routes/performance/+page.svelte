<script lang="ts">
  import PerformanceMetrics from '$lib/components/metrics/PerformanceMetrics.svelte';
  import MetricsAlerts from '$lib/components/metrics/MetricsAlerts.svelte';
  import MemoryMonitor from '$lib/components/model-manager/MemoryMonitor.svelte';
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
    <!-- Метрики и мониторинг памяти -->
    <main class="metrics-panel">
      <PerformanceMetrics />
      <MemoryMonitor />
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
    padding: 2rem;
  }

  .page-header {
    max-width: 1400px;
    margin: 0 auto 2rem;
  }

  .page-header h1 {
    margin: 0 0 0.5rem 0;
    font-size: 2rem;
    font-weight: 700;
    color: var(--text);
  }

  .page-description {
    margin: 0;
    font-size: 1rem;
    color: var(--muted);
    line-height: 1.6;
  }

  .alerts-section {
    max-width: 1400px;
    margin: 0 auto 2rem;
  }

  .performance-layout {
    display: grid;
    grid-template-columns: 1fr 380px;
    gap: 2rem;
    max-width: 1400px;
    margin: 0 auto;
  }

  .metrics-panel {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  .info-panel {
    display: flex;
    flex-direction: column;
    gap: 2rem;
    position: sticky;
    top: 2rem;
    align-self: start;
    max-height: calc(100vh - 4rem);
    overflow-y: auto;
  }

  .info-card {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
  }

  .info-card h3 {
    margin: 0 0 1.5rem 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text);
  }

  .info-section {
    margin-bottom: 1.5rem;
    padding-bottom: 1.5rem;
    border-bottom: 1px solid var(--border-color);
  }

  .info-section:last-child {
    margin-bottom: 0;
    padding-bottom: 0;
    border-bottom: none;
  }

  .info-section h4 {
    margin: 0 0 0.5rem 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text);
  }

  .info-section p {
    margin: 0 0 0.5rem 0;
    font-size: 0.75rem;
    color: var(--muted);
    line-height: 1.6;
  }

  .info-section p:last-child {
    margin-bottom: 0;
  }

  .target-info {
    padding: 0.5rem 0.75rem;
    background: rgba(52, 152, 219, 0.1);
    border-left: 3px solid var(--info, #3498db);
    border-radius: 4px;
    font-size: 0.75rem;
    color: var(--text);
  }

  .target-info strong {
    color: var(--info, #3498db);
  }

  /* Адаптивность для планшетов */
  @media (max-width: 1200px) {
    .performance-layout {
      grid-template-columns: 1fr;
      gap: 1.5rem;
    }

    .info-panel {
      position: static;
      max-height: none;
    }
  }

  /* Адаптивность для мобильных */
  @media (max-width: 768px) {
    .performance-page {
      padding: 1rem;
    }

    .page-header h1 {
      font-size: 1.5rem;
    }

    .info-card {
      padding: 1rem;
    }
  }

  /* Скроллбар для info panel */
  .info-panel::-webkit-scrollbar {
    width: 8px;
  }

  .info-panel::-webkit-scrollbar-track {
    background: transparent;
    border-radius: 4px;
  }

  .info-panel::-webkit-scrollbar-thumb {
    background: rgba(179, 205, 224, 0.6);
    border-radius: 4px;
    transition: background 0.3s ease;
  }

  .info-panel::-webkit-scrollbar-thumb:hover {
    background: rgba(179, 205, 224, 0.8);
  }
</style>
