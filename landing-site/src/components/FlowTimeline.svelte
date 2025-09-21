<script lang="ts">
  import { onMount } from 'svelte';
  import anime from 'animejs/lib/anime.es.js';
  import { reveal } from './reveal';

  const stages = [
    {
      id: 'download',
      title: 'Скачайте модель',
      body: 'Загрузите Qwen3 в формате GGUF и соответствующий tokenizer.json с Hugging Face или из другого доверенного каталога.',
      metric: 'Формат: .gguf + tokenizer'
    },
    {
      id: 'load',
      title: 'Загрузите в Oxide Lab',
      body: 'Выберите файлы модели и токенизатора в приложении, настройте параметры инференса при необходимости и запустите загрузку.',
      metric: 'Настройка за несколько шагов'
    },
    {
      id: 'chat',
      title: 'Начните диалог',
      body: 'Введите запрос, при необходимости включите режим «Размышления», регулируйте температуру и другие параметры прямо в чате.',
      metric: 'Ответы в реальном времени'
    }
  ];

  let timelineEl: HTMLElement | null = null;
  let stagePointer = $state(0);

  onMount(() => {
    if (!timelineEl) return;
    const links = timelineEl.querySelectorAll('.stage-line');
    anime({
      targets: links,
      strokeDashoffset: [anime.setDashoffset, 0],
      easing: 'easeOutCubic',
      duration: 1200,
      delay: anime.stagger(220)
    });

    const cycle = setInterval(() => {
      stagePointer = (stagePointer + 1) % stages.length;
    }, 5200);

    return () => clearInterval(cycle);
  });
</script>

<section class="flow" id="discover" bind:this={timelineEl}>
  <div class="flow__wrap">
    <header>
      <span class="section-tag">Как начать</span>
      <h2>Поднимите локальный ИИ за три шага</h2>
      <p>
        Oxide Lab следует простой схеме из README: подготовьте модель, загрузите её в приложение и сразу же общайтесь локально без подключения к интернету.
      </p>
    </header>
    <div class="flow__stages">
      {#each stages as stage, index}
        <article class:selected={index === stagePointer} use:reveal={{ delay: index * 110 }}>
          <div class="stage__label">
            <span>0{index + 1}</span>
            <strong>{stage.title}</strong>
          </div>
          <p>{stage.body}</p>
          <span class="stage__metric">{stage.metric}</span>
        </article>
      {/each}
    </div>
    <svg class="flow__diagram" viewBox="0 0 640 160" role="presentation" aria-hidden="true">
      <defs>
        <linearGradient id="lineGradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stop-color="#7c3aed" />
          <stop offset="100%" stop-color="#c084fc" />
        </linearGradient>
      </defs>
      <path class="stage-line" d="M40 80C140 80 140 40 240 40C340 40 340 120 440 120C540 120 540 60 640 60" stroke="url(#lineGradient)" stroke-width="4" fill="transparent" stroke-linecap="round" stroke-dasharray="12 8" />
      <circle class="stage-node" cx="40" cy="80" r="8" />
      <circle class="stage-node" cx="240" cy="40" r="8" />
      <circle class="stage-node" cx="440" cy="120" r="8" />
      <circle class="stage-node" cx="640" cy="60" r="8" />
    </svg>
  </div>
</section>

<style>
  .flow {
    display: grid;
    gap: clamp(2.5rem, 6vw, 3.5rem);
    padding: clamp(4rem, 7vw, 6rem) 0;
  }

  .flow__wrap {
    width: 100%;
    margin: 0;
    padding-inline: clamp(1rem, 6vw, 3.5rem);
    box-sizing: border-box;
    display: grid;
    gap: clamp(2rem, 6vw, 3rem);
  }

  header {
    display: grid;
    gap: 1rem;
    max-width: min(780px, 88vw);
  }

  header h2 {
    font-size: clamp(2.1rem, 4vw, 3.1rem);
    color: #f8fafc;
  }

  header p {
    font-size: 1.05rem;
    color: rgba(226, 232, 240, 0.78);
    line-height: 1.8;
  }

  .flow__stages {
    display: grid;
    gap: 1.4rem;
  }

  article {
    padding: 1.6rem;
    border-radius: 1.4rem;
    border: 1px solid rgba(148, 163, 184, 0.2);
    background: rgba(15, 23, 42, 0.7);
    display: grid;
    gap: 0.9rem;
    transition: border 0.2s ease, transform 0.2s ease;
  }

  article.selected {
    border-color: rgba(124, 58, 237, 0.55);
    transform: translateY(-4px);
    box-shadow: 0 20px 32px rgba(88, 28, 135, 0.2);
  }

  .stage__label {
    display: flex;
    gap: 0.9rem;
    align-items: baseline;
    color: rgba(165, 180, 252, 0.85);
  }

  .stage__label span {
    font-family: 'IBM Plex Mono', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
    font-size: 0.82rem;
    letter-spacing: 0.12em;
  }

  .stage__label strong {
    font-size: 1.4rem;
    color: #f8fafc;
  }

  article p {
    color: rgba(226, 232, 240, 0.78);
    line-height: 1.6;
  }

  .stage__metric {
    font-size: 0.9rem;
    color: rgba(168, 85, 247, 0.95);
    font-weight: 600;
  }

  .flow__diagram {
    width: 100%;
  }

  .stage-node {
    fill: #a78bfa;
  }

  @media (max-width: 820px) {
    .flow__diagram {
      display: none;
    }
  }

  @media (max-width: 560px) {
    .flow {
      padding: 3.5rem 0;
    }

    header h2 {
      font-size: 2.2rem;
    }

    .flow__stages {
      gap: 1rem;
    }

    article {
      padding: 1.3rem;
    }

    .stage__label {
      flex-wrap: wrap;
      gap: 0.4rem;
    }
  }
</style>
