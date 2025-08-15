<script lang="ts">
  // Заглушка страницы API
</script>

<main class="wrap">
  <div class="api-page">
    <div class="page-header">
      <h1>Работа с моделью через API</h1>
      <p>Здесь будет интерфейс для отправки запросов к модели через API, тестирование эндпоинтов и просмотр ответов.</p>
    </div>
    
    <div class="api-content">
      <div class="card">
        <h2>API Endpoints</h2>
        <p>В разработке...</p>
      </div>
    </div>
  </div>
</main>

<style>
  .api-page {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 16px;
  }

  .page-header {
    text-align: center;
    margin-bottom: 32px;
    padding: 32px 0;
  }

  .page-header h1 {
    font-size: 2.5rem;
    font-weight: 700;
    color: var(--text);
    margin: 0 0 16px 0;
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .page-header p {
    font-size: 1.1rem;
    color: var(--muted);
    margin: 0;
    max-width: 600px;
    margin: 0 auto;
  }

  .api-content {
    margin-top: 32px;
  }

  /* Адаптивность */
  @media (max-width: 768px) {
    .api-page {
      padding: 0 12px;
    }

    .page-header h1 {
      font-size: 2rem;
    }

    .page-header p {
      font-size: 1rem;
    }
  }
</style>