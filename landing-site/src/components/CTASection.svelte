<script lang="ts">
  import anime from 'animejs/lib/anime.es.js';
  import { onMount } from 'svelte';

  let sectionEl: HTMLElement | null = null;

  onMount(() => {
    if (!sectionEl) return;
    anime({
      targets: sectionEl.querySelectorAll('.cta__card'),
      opacity: [0, 1],
      translateY: [24, 0],
      delay: anime.stagger(140),
      duration: 520,
      easing: 'easeOutCubic'
    });
  });

  function pulseLinks() {
    anime({
      targets: '.cta__actions a',
      scale: [1, 1.03, 1],
      duration: 420,
      easing: 'easeInOutSine',
      delay: anime.stagger(80)
    });
  }
</script>

<section class="cta" bind:this={sectionEl}>
  <div class="cta__wrap">
    <div class="cta__content">
      <div class="cta__card">
        <h2>Попробуйте Oxide Lab уже сегодня</h2>
        <p>
          Следуйте инструкциям из README репозитория: скачайте подходящий GGUF, выберите его в приложении и начните диалог с ИИ локально. Все вычисления остаются на вашем компьютере.
        </p>
        <div class="cta__actions">
          <a
            class="cta__button"
            href="https://github.com/oxide-lab/Oxide-Lab/blob/main/README-RU.md#-как-начать-пользоваться"
            target="_blank"
            rel="noreferrer"
            onmouseenter={pulseLinks}
          >
            Инструкция по запуску
          </a>
          <a
            class="cta__button cta__button--ghost"
            href="https://github.com/oxide-lab/Oxide-Lab/releases"
            target="_blank"
            rel="noreferrer"
            onmouseenter={pulseLinks}
          >
            Релизы проекта
          </a>
        </div>
      </div>
      <div class="cta__card cta__card--secondary">
        <h3>Что включено</h3>
        <ul>
          <li><strong>Windows 10/11</strong> — полноценная поддержка</li>
          <li><strong>CPU и GPU (CUDA)</strong> — выбор устройств инференса</li>
          <li><strong>MIT лицензия</strong> — открытый и расширяемый код</li>
        </ul>
        <a class="cta__link" href="https://github.com/oxide-lab/Oxide-Lab" target="_blank" rel="noreferrer">Перейти в репозиторий</a>
      </div>
    </div>
  </div>
</section>

<style>
  .cta {
    padding: clamp(4rem, 8vw, 6rem) 0 clamp(5rem, 9vw, 7rem);
  }

  .cta__wrap {
    width: 100%;
    margin: 0;
    padding-inline: clamp(1rem, 6vw, 3.5rem);
    box-sizing: border-box;
  }

  .cta__content {
    display: grid;
    grid-template-columns: minmax(0, 2fr) minmax(260px, 1fr);
    gap: clamp(1.8rem, 4vw, 2.5rem);
  }

  .cta__card {
    border-radius: 1.6rem;
    border: 1px solid rgba(148, 163, 184, 0.2);
    background: linear-gradient(140deg, rgba(15, 23, 42, 0.9), rgba(30, 41, 59, 0.75));
    padding: clamp(2rem, 4vw, 3rem);
    display: grid;
    gap: 1.4rem;
  }

  h2 {
    font-size: clamp(2rem, 3.6vw, 2.6rem);
    color: #f8fafc;
  }

  .cta__card p {
    color: rgba(226, 232, 240, 0.78);
    line-height: 1.7;
  }

  .cta__actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.8rem;
  }

  .cta__button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 999px;
    padding: 0.85rem 1.8rem;
    font-weight: 600;
    background: linear-gradient(120deg, #7c3aed, #c084fc);
    color: #0f172a;
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }

  .cta__button:hover {
    transform: translateY(-2px);
    box-shadow: 0 16px 32px rgba(124, 58, 237, 0.26);
  }

  .cta__button--ghost {
    background: rgba(15, 23, 42, 0.7);
    color: #f1f5f9;
    border: 1px solid rgba(148, 163, 184, 0.3);
  }

  .cta__card--secondary {
    background: linear-gradient(140deg, rgba(91, 33, 182, 0.85), rgba(15, 23, 42, 0.85));
    border-color: rgba(124, 58, 237, 0.45);
    align-content: start;
  }

  .cta__card--secondary h3 {
    color: #f8fafc;
  }

  .cta__card--secondary ul {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    gap: 0.8rem;
    color: rgba(226, 232, 240, 0.85);
  }

  .cta__card--secondary li strong {
    color: #f8fafc;
  }

  .cta__link {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.75rem 1.4rem;
    border-radius: 999px;
    border: 1px solid rgba(226, 232, 240, 0.4);
    color: #f8fafc;
    transition: background 0.2s ease;
  }

  .cta__link:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  @media (max-width: 960px) {
    .cta__content {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 640px) {
    .cta__actions {
      flex-direction: column;
      align-items: stretch;
    }

    .cta__button {
      width: 100%;
    }
  }

  @media (max-width: 540px) {
    .cta {
      padding: 3.6rem 0 4.2rem;
    }

    .cta__card {
      padding: 1.8rem;
      text-align: center;
    }

    .cta__card--secondary ul {
      justify-items: center;
    }

    .cta__link {
      width: 100%;
    }
  }
</style>
