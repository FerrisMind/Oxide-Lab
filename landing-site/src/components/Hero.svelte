<script>
	import { onMount } from 'svelte';
	import { animate } from 'animejs';

	onMount(() => {
		// Animate particles
		animate('.particle', {
			translateX: () => Math.random() * 200 - 100,
			translateY: () => Math.random() * 100 - 50,
			scale: [0, 1],
			opacity: [0, 0.6],
			duration: 3000,
			delay: 200,
			easing: 'easeOutQuad',
			loop: true,
			direction: 'alternate'
		});

		// Fade in text
		animate('.hero-text', {
			opacity: [0, 1],
			translateY: [-20, 0],
			duration: 1000,
			delay: 500,
			easing: 'easeOutQuad'
		});

		// Glow effect on CTA button
		animate('.cta-button', {
			boxShadow: ['0 0 0 rgba(255, 107, 53, 0)', '0 0 20px rgba(255, 107, 53, 0.5)'],
			duration: 2000,
			easing: 'easeInOutSine',
			loop: true,
			direction: 'alternate'
		});
	});

	let isVisible = false;
</script>

<section class="hero">
	<div class="container">
		<div class="hero-content fade-in">
			<h1 class="hero-text">Ваш личный ИИ. Полностью приватный.</h1>
			<p class="hero-subtitle">Локальная обработка данных без использования серверов. Полная конфиденциальность гарантирована.</p>

			<div class="particles parallax" data-speed="0.3">
				{#each Array(8) as _, i}
					<div class="particle particle-{i}"></div>
				{/each}
			</div>

			<button class="cta-button" on:click={() => window.scrollTo({ top: document.getElementById('features').offsetTop, behavior: 'smooth' })}>
				Узнать больше
			</button>
		</div>
	</div>
</section>

<style>
	.hero {
		min-height: 100vh;
		display: flex;
		align-items: center;
		justify-content: center;
		text-align: center;
		background: linear-gradient(135deg, #ffffff 0%, #f5f5f5 100%);
		position: relative;
		overflow: hidden;
	}

	.hero-content {
		z-index: 2;
		position: relative;
	}

	.hero-text {
		font-size: 48px;
		font-weight: 600;
		color: var(--primary-color);
		margin-bottom: 1rem;
		line-height: 1.2;
	}

	.hero-subtitle {
		font-size: 18px;
		color: var(--text);
		margin-bottom: 3rem;
		max-width: 600px;
		margin-left: auto;
		margin-right: auto;
	}

	.particles {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: 400px;
		height: 400px;
		pointer-events: none;
	}

	.particle {
		position: absolute;
		width: 8px;
		height: 8px;
		background: radial-gradient(circle, var(--accent-start), var(--accent-end));
		border-radius: 50%;
		box-shadow: 0 0 10px rgba(255, 107, 53, 0.5);
	}

	.particle-0 { top: 20%; left: 20%; }
	.particle-1 { top: 30%; left: 70%; }
	.particle-2 { top: 60%; left: 15%; }
	.particle-3 { top: 70%; left: 80%; }
	.particle-4 { top: 10%; left: 50%; }
	.particle-5 { top: 80%; left: 40%; }
	.particle-6 { top: 40%; left: 10%; }
	.particle-7 { top: 50%; left: 90%; }

	.cta-button {
		background: linear-gradient(45deg, var(--accent-start), var(--accent-end));
		color: white;
		padding: 1rem 2rem;
		border-radius: var(--border-radius);
		font-size: 16px;
		font-weight: 500;
		box-shadow: 0 4px 12px rgba(255, 107, 53, 0.3);
		transition: var(--transition);
	}

	.cta-button:hover {
		transform: translateY(-2px);
		box-shadow: 0 6px 20px rgba(255, 107, 53, 0.4);
	}

	/* Mobile styles */
	@media (max-width: 767px) {
		.hero-text {
			font-size: 36px;
		}

		.hero-subtitle {
			font-size: 16px;
			margin-bottom: 2rem;
		}

		.particles {
			width: 300px;
			height: 300px;
		}

		.particle {
			width: 6px;
			height: 6px;
		}
	}

	@media (min-width: 1200px) {
		.hero-text {
			font-size: 56px;
		}

		.particles {
			width: 500px;
			height: 500px;
		}

		.particle {
			width: 10px;
			height: 10px;
		}
	}
</style>
