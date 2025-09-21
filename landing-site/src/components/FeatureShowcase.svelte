<script lang="ts">
  import { reveal } from './reveal';
  import anime from 'animejs/lib/anime.es.js';

  type Feature = {
    title: string;
    summary: string;
    bullets: string[];
    accent: string;
  };

  const features: Feature[] = [
    {
      title: 'Умный чат-интерфейс',
      summary:
        'Современная оболочка для общения с локальной моделью: потоковые ответы, поддержка форматирования текста и кода, быстрые действия по остановке генерации.',
      bullets: [
        'Стриминг ответов в реальном времени',
        'Подсветка кода и структурированных сообщений',
        'Удобная очередь сообщений и прогресс загрузки модели'
      ],
      accent: 'purple'
    },
    {
      title: 'Режим «Размышления»',
      summary:
        'Наблюдайте ход мыслей модели Qwen3 до финального ответа и получайте более обстоятельные решения сложных задач.',
      bullets: [
        'Пошаговый анализ ввода перед ответом',
        'Прозрачный контроль качества генерации',
        'Работает полностью локально без подключения к интернету'
      ],
      accent: 'blue'
    },
    {
      title: 'Гибкие настройки',
      summary:
        'Тонко подстраивайте генерацию под задачу: регулируйте креативность, длину контекста и параметры выборки прямо во время диалога.',
      bullets: [
        'Температура, Top-K, Top-P, Min-P, повторения',
        'Выбор контекста в зависимости от модели и ресурсов',
        'Предустановленные рекомендации для Qwen3'
      ],
      accent: 'emerald'
    }
  ];

  let selectedIndex = $state(0);
  const currentFeature = $derived(features[selectedIndex]);

  function handleSelect(index: number) {
    if (index === selectedIndex) return;
    selectedIndex = index;
    anime({
      targets: '.feature-card__content',
      opacity: [0, 1],
      translateY: [16, 0],
      duration: 360,
      easing: 'easeOutQuad'
    });
  }
</script>

<section id="features" class="feature-panel">
  <div class="feature-panel__wrap">
    <div class="feature-panel__intro" use:reveal={{ translateY: 24 }}>
      <span class="section-tag">Возможности</span>
      <h2>Все, что нужно локальному ИИ-чату</h2>
      <p>
        Функциональность Oxide Lab основана на возможностях из репозитория: потоковые ответы, режим «Размышления», гибкие параметры генерации и простая загрузка моделей Qwen3 в GGUF.
      </p>
    </div>
    <div class="feature-panel__grid">
      <aside>
        {#each features as feature, index}
          <button
            class:selected={index === selectedIndex}
            class={`feature-tab feature-tab--${feature.accent}`}
            onclick={() => handleSelect(index)}
            use:reveal={{ delay: index * 60 }}
          >
            <span>{feature.title}</span>
            <svg aria-hidden="true" width="18" height="18" viewBox="0 0 18 18" fill="none">
              <path
                d="M5 9.5l3 3 6-6"
                stroke="currentColor"
                stroke-width="1.6"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </button>
        {/each}
      </aside>
      <article class={`feature-card feature-card--${currentFeature.accent}`}>
        <div class="feature-card__content">
          <h3>{currentFeature.title}</h3>
          <p>{currentFeature.summary}</p>
          <ul>
            {#each currentFeature.bullets as point, idx}
              <li use:reveal={{ delay: 120 + idx * 60, translateY: 12 }}>{point}</li>
            {/each}
          </ul>
        </div>
      </article>
    </div>
  </div>
</section>

<style>
  .feature-panel {
    display: grid;
    gap: clamp(2.8rem, 5vw, 4rem);
    padding: clamp(4.5rem, 8vw, 6rem) 0;
  }

  .feature-panel__wrap {
    width: 100%;
    margin: 0;
    padding-inline: clamp(1rem, 6vw, 3.5rem);
    box-sizing: border-box;
    display: grid;
    gap: clamp(2rem, 6vw, 3rem);
  }

  .feature-panel__intro {
    display: grid;
    gap: 1rem;
    max-width: min(780px, 88vw);
  }

  .section-tag {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.84rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: rgba(168, 85, 247, 0.9);
  }

  h2 {
    font-size: clamp(2rem, 4vw, 3rem);
    font-weight: 700;
    color: #f8fafc;
  }

  .feature-panel__intro p {
    font-size: 1.05rem;
    line-height: 1.8;
    color: rgba(226, 232, 240, 0.76);
  }

  .feature-panel__grid {
    display: grid;
    grid-template-columns: minmax(280px, 340px) minmax(0, 1fr);
    gap: clamp(1.8rem, 5vw, 2.8rem);
  }

  aside {
    display: grid;
    gap: 0.8rem;
  }

  .feature-tab {
    padding: 1.2rem 1.4rem;
    border-radius: 1rem;
    background: rgba(15, 23, 42, 0.75);
    border: 1px solid rgba(148, 163, 184, 0.18);
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: rgba(226, 232, 240, 0.8);
    font-weight: 600;
    transition: border 0.2s ease, transform 0.2s ease;
  }

  .feature-tab:hover {
    transform: translateY(-2px);
  }

  .feature-tab.selected {
    border-color: rgba(168, 85, 247, 0.55);
    color: #f8fafc;
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.34), rgba(15, 23, 42, 0.85));
  }

  .feature-card {
    border-radius: 1.6rem;
    padding: clamp(2rem, 4vw, 3rem);
    border: 1px solid rgba(148, 163, 184, 0.18);
    background: rgba(15, 23, 42, 0.8);
    position: relative;
    overflow: hidden;
  }

  .feature-card::after {
    content: '';
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: radial-gradient(circle at top right, rgba(168, 85, 247, 0.32), transparent 55%);
    opacity: 0.7;
  }

  .feature-card__content {
    position: relative;
    display: grid;
    gap: 1.3rem;
  }

  .feature-card h3 {
    font-size: 1.8rem;
    color: #f8fafc;
  }

  .feature-card p {
    color: rgba(226, 232, 240, 0.8);
    line-height: 1.7;
  }

  .feature-card ul {
    display: grid;
    gap: 0.7rem;
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .feature-card li {
    display: flex;
    gap: 0.6rem;
    align-items: flex-start;
    color: rgba(226, 232, 240, 0.85);
  }

  .feature-card li::before {
    content: '';
    width: 10px;
    height: 10px;
    margin-top: 0.35rem;
    border-radius: 999px;
    background: currentColor;
    opacity: 0.64;
  }

  .feature-card--purple {
    border-color: rgba(168, 85, 247, 0.4);
  }

  .feature-card--blue {
    border-color: rgba(56, 189, 248, 0.4);
  }

  .feature-card--emerald {
    border-color: rgba(16, 185, 129, 0.4);
  }

  @media (max-width: 980px) {
    .feature-panel__grid {
      grid-template-columns: 1fr;
    }

    aside {
      grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    }

    .feature-card {
      min-height: auto;
    }
  }

  @media (max-width: 640px) {
    aside {
      grid-template-columns: 1fr;
    }

    .feature-tab {
      justify-content: flex-start;
      gap: 0.7rem;
    }
  }

  @media (max-width: 520px) {
    .feature-panel {
      padding: 3.5rem 0;
      gap: 2.2rem;
    }

    .feature-panel__intro p {
      font-size: 0.98rem;
    }

    .feature-card {
      padding: 1.8rem;
    }

    .feature-card li {
      font-size: 0.95rem;
    }
  }
</style>
