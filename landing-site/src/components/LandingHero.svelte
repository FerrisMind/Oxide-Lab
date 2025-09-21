<script lang="ts">
  import anime from 'animejs/lib/anime.es.js';
  import { onMount } from 'svelte';

  let heroEl: HTMLElement | null = null;
  let orbitCanvas: HTMLCanvasElement | null = null;

  const highlights = [
    'Локальный ИИ-чат без отправки данных',
    'Режим «Размышления» для глубоких ответов',
    'Полный контроль параметров генерации'
  ];

  let highlightIndex = $state(0);
  const activeHighlight = $derived(highlights[highlightIndex]);

  onMount(() => {
    if (!heroEl) return;

    const timeline = anime
      .timeline({ autoplay: true })
      .add({
        targets: heroEl.querySelectorAll('.hero-line'),
        translateY: [24, 0],
        opacity: [0, 1],
        easing: 'easeOutExpo',
        duration: 650,
        delay: anime.stagger(120)
      })
      .add(
        {
          targets: heroEl.querySelectorAll('.hero-badge, .hero-cta'),
          opacity: [0, 1],
          translateY: [16, 0],
          easing: 'easeOutCirc',
          duration: 500,
          delay: anime.stagger(80)
        },
        '-=300'
      );

    const interval = setInterval(() => {
      highlightIndex = (highlightIndex + 1) % highlights.length;
    }, 3600);

    drawOrbit();
    const backgroundInterval = setInterval(drawOrbit, 5400);

    return () => {
      timeline.pause();
      clearInterval(interval);
      clearInterval(backgroundInterval);
    };
  });

  $effect(() => {
    if (!heroEl) return;

    const highlightEl = heroEl.querySelector('.hero-highlight');
    if (!highlightEl) return;

    anime({
      targets: highlightEl,
      opacity: [0, 1],
      translateY: [12, 0],
      duration: 440,
      easing: 'easeOutQuad'
    });
  });

  function drawOrbit() {
    if (!orbitCanvas) return;
    const ctx = orbitCanvas.getContext('2d');
    if (!ctx) return;

    const { width, height } = orbitCanvas;
    ctx.clearRect(0, 0, width, height);

    const particles = Array.from({ length: 16 }, (_, index) => ({
      radius: 48 + index * 8,
      alpha: 0.08 + index * 0.02
    }));

    for (const { radius, alpha } of particles) {
      ctx.beginPath();
      ctx.arc(width / 2, height / 2, radius, 0, Math.PI * 2);
      ctx.strokeStyle = `rgba(150, 115, 255, ${alpha.toFixed(3)})`;
      ctx.lineWidth = 1.4;
      ctx.stroke();
    }
  }

  $effect(() => {
    if (!heroEl) return;
    const cards = heroEl.querySelectorAll('.metric-card');
    if (!cards.length) return;

    const staggered = anime({
      targets: cards,
      opacity: [0, 1],
      translateY: [22, 0],
      delay: anime.stagger(160, { start: 420 }),
      duration: 480,
      easing: 'easeOutBack'
    });

    return () => staggered.pause();
  });
</script>

<section class="hero" bind:this={heroEl}>
  <div class="hero__glow"></div>
  <canvas class="hero__orbit" width="560" height="560" bind:this={orbitCanvas} aria-hidden="true"></canvas>
  <div class="hero__inner">
    <div class="hero__content">
      <span class="hero-badge">Oxide Lab Desktop</span>
      <h1>
        <span class="hero-line">Соберите свою экосистему</span>
        <span class="hero-line">ИИ-инструментов и данных</span>
        <span class="hero-line hero-highlight">{activeHighlight}</span>
      </h1>
      <p class="hero-tagline">
        Oxide Lab — локальное приложение для Windows, которое запускает модели Qwen3 в формате GGUF прямо на вашем компьютере и показывает ход размышлений ИИ перед ответом.
      </p>
      <div class="hero-cta">
        <a class="primary" href="#discover">Попробовать демо</a>
        <a class="secondary" href="#features">Посмотреть возможности</a>
      </div>
      <div class="hero-metrics">
        <article class="metric-card">
          <span class="metric"><strong>100%</strong></span>
          <span class="label">Вычисления только локально</span>
        </article>
        <article class="metric-card">
          <span class="metric"><strong>Qwen3</strong></span>
          <span class="label">Поддержка моделей в GGUF</span>
        </article>
        <article class="metric-card">
          <span class="metric"><strong>MIT</strong></span>
          <span class="label">Открытый исходный код</span>
        </article>
      </div>
    </div>
  </div>
</section>

<style>
  .hero {
    position: relative;
    width: 100%;
    min-height: 100vh;
    padding: clamp(4rem, 10vw, 6rem) 0;
    display: flex;
    align-items: stretch;
    justify-content: center;
    overflow: hidden;
  }

  .hero__glow {
    position: absolute;
    inset: 0;
    margin: auto;
    width: 640px;
    height: 640px;
    border-radius: 999px;
    filter: blur(160px);
    opacity: 0.6;
    background: radial-gradient(circle at 50% 50%, rgba(124, 58, 237, 0.35), transparent 60%);
    pointer-events: none;
  }

  .hero__orbit {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -52%);
    opacity: 0.6;
  }

  .hero__inner {
    position: relative;
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding-inline: clamp(1.5rem, 5vw, 6rem);
    width: 100%;
  }

  .hero__content {
    position: relative;
    width: 100%;
    display: grid;
    gap: clamp(1.5rem, 4vw, 2.5rem);
    text-align: center;
  }

  .hero-badge {
    display: inline-flex;
    align-self: center;
    padding: 0.55rem 1.4rem;
    border-radius: 999px;
    border: 1px solid rgba(148, 163, 184, 0.24);
    background: rgba(30, 41, 59, 0.55);
    font-size: 0.84rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-weight: 600;
    color: #cbd5f5;
  }

  h1 {
    display: grid;
    gap: 0.35rem;
    font-size: clamp(2.5rem, 6vw, 4.65rem);
    line-height: 1.1;
    font-weight: 700;
    color: #f8fafc;
  }

  .hero-highlight {
    color: #a78bfa;
    text-shadow: 0 0 18px rgba(168, 85, 247, 0.45);
  }

  .hero-tagline {
    margin: 0 auto;
    max-width: 100%;
    font-size: clamp(1.05rem, 2.4vw, 1.2rem);
    line-height: 1.7;
    color: #cbd5f5;
  }

  .hero-cta {
    display: flex;
    justify-content: center;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .hero-cta a {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 999px;
    padding: 0.85rem 1.9rem;
    font-weight: 600;
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }

  .hero-cta .primary {
    background: linear-gradient(120deg, #7c3aed, #c084fc);
    color: #111827;
    box-shadow: 0 20px 30px rgba(124, 58, 237, 0.3);
  }

  .hero-cta .secondary {
    border: 1px solid rgba(148, 163, 184, 0.3);
    color: #cbd5f5;
  }

  .hero-cta a:hover {
    transform: translateY(-2px) scale(1.01);
  }

  .hero-metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 1rem;
    margin-top: 0.5rem;
  }

  .metric-card {
    padding: 1.2rem 1.4rem;
    border-radius: 1rem;
    background: rgba(15, 23, 42, 0.8);
    border: 1px solid rgba(148, 163, 184, 0.15);
    backdrop-filter: blur(10px);
    display: grid;
    gap: 0.45rem;
  }

  .metric-card .metric {
    font-size: 1.9rem;
    color: #f8fafc;
  }

  .metric-card .label {
    font-size: 0.95rem;
    color: rgba(226, 232, 240, 0.75);
  }

  @media (max-width: 720px) {
    .hero {
      padding: clamp(3rem, 12vw, 4.5rem) 0;
    }

    .hero__inner {
      padding-inline: clamp(1rem, 6vw, 2.4rem);
      padding-block: clamp(2.5rem, 12vw, 4rem);
    }

    .hero-cta {
      flex-direction: column;
      align-items: center;
    }

    .hero__orbit {
      width: 320px;
      height: 320px;
    }
  }

  @media (max-width: 540px) {
    .hero {
      padding: clamp(2.5rem, 14vw, 3.4rem) 0;
    }

    .hero__inner {
      padding-inline: clamp(1rem, 7vw, 2rem);
      padding-block: clamp(2rem, 14vw, 3.2rem);
    }

    .hero__orbit {
      display: none;
    }

    .hero__content {
      gap: 1.4rem;
    }

    h1 {
      font-size: clamp(2rem, 10vw, 2.8rem);
    }

    .hero-tagline {
      font-size: 1rem;
      line-height: 1.6;
    }

    .hero-metrics {
      grid-template-columns: 1fr;
    }
  }
</style>
