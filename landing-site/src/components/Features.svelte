<script lang="ts">
	import { onMount } from 'svelte';
	import { animate } from 'animejs';
	import { ChatCircle, Lock, DeviceMobile, HardDrive, Target, ChartBar, MagnifyingGlass, Lightning, Gear, Palette, Wrench, ChartLine } from 'phosphor-svelte';

	onMount(() => {
		// Stagger animation for feature cards
		animate('.feature-card', {
			translateY: [50, 0],
			opacity: [0, 1],
			duration: 800,
			delay: 200,
			easing: 'easeOutQuad'
		});

		// Hover effects for cards
		const cards = document.querySelectorAll('.feature-card');
		cards.forEach(card => {
			card.addEventListener('mouseenter', () => {
				animate(card, {
					scale: 1.05,
					rotateY: 5,
					boxShadow: '0 20px 40px rgba(0, 0, 0, 0.2)',
					duration: 300,
					easing: 'easeOutQuad'
				});
			});

			card.addEventListener('mouseleave', () => {
				animate(card, {
					scale: 1,
					rotateY: 0,
					boxShadow: '0 4px 12px rgba(0, 0, 0, 0.1)',
					duration: 300,
					easing: 'easeOutQuad'
				});
			});
		});
	});
</script>

<section id="features" class="features">
	<div class="container">
		<h2 class="fade-in">Возможности</h2>
		<p class="section-subtitle fade-in">Расширенные функции для эффективной работы с ИИ</p>

		<div class="features-grid">
			<div class="feature-card fade-in">
				<div class="card-icon"><ChatCircle size={32} /></div>
				<h3>Приватный чат</h3>
				<p>Безопасное общение с ИИ без передачи данных третьим лицам. Все разговоры хранятся локально на вашем устройстве.</p>
				<div class="card-features">
					<span><Lock size={16} /> Шифрование end-to-end</span>
					<span><DeviceMobile size={16} /> Мобильная синхронизация</span>
					<span><HardDrive size={16} /> Локальное хранение</span>
				</div>
			</div>

			<div class="feature-card fade-in">
				<div class="card-icon"><Target size={32} /></div>
				<h3>Режим мышления</h3>
				<p>Визуализация процесса анализа и принятия решений ИИ. Следите за каждым шагом обработки вашего запроса.</p>
				<div class="card-features">
					<span><ChartBar size={16} /> Реальное время</span>
					<span><MagnifyingGlass size={16} /> Детальная аналитика</span>
					<span><Lightning size={16} /> Быстрая обработка</span>
				</div>
			</div>

			<div class="feature-card fade-in">
				<div class="card-icon"><Gear size={32} /></div>
				<h3>Гибкие настройки</h3>
				<p>Полный контроль над поведением ИИ. Настройте параметры под свои нужды и предпочтения.</p>
				<div class="card-features">
					<span><Palette size={16} /> Персонализация</span>
					<span><Wrench size={16} /> Глубокая настройка</span>
					<span><ChartLine size={16} /> Производительность</span>
				</div>
			</div>
		</div>

		<div class="cta-section fade-in">
			<h3>Готовы начать?</h3>
			<p>Попробуйте все возможности Oxide Lab прямо сейчас</p>
			<button class="cta-button">Скачать приложение</button>
		</div>
	</div>
</section>

<style>
	.features {
		padding: 4rem 0;
		background: var(--background);
	}

	h2 {
		text-align: center;
		font-size: 32px;
		color: var(--primary-color);
		margin-bottom: 1rem;
	}

	.section-subtitle {
		text-align: center;
		font-size: 16px;
		color: var(--text);
		margin-bottom: 3rem;
	}

	.features-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
		gap: 2rem;
		margin-bottom: 4rem;
	}

	.feature-card {
		background: white;
		border-radius: var(--border-radius);
		padding: 2rem;
		box-shadow: var(--shadow);
		transition: var(--transition);
		cursor: pointer;
		position: relative;
		overflow: hidden;
	}

	.feature-card::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 4px;
		background: linear-gradient(to right, var(--accent-start), var(--accent-end));
	}

	.card-icon {
		font-size: 3rem;
		margin-bottom: 1rem;
		text-align: center;
	}

	.feature-card h3 {
		font-size: 24px;
		color: var(--primary-color);
		margin-bottom: 1rem;
		text-align: center;
	}

	.feature-card p {
		font-size: 16px;
		color: var(--text);
		margin-bottom: 1.5rem;
		line-height: 1.6;
	}

	.card-features {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.card-features span {
		font-size: 14px;
		color: var(--text);
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.cta-section {
		text-align: center;
		padding: 3rem;
		background: linear-gradient(135deg, var(--gray-light) 0%, white 100%);
		border-radius: var(--border-radius);
		box-shadow: var(--shadow);
	}

	.cta-section h3 {
		font-size: 28px;
		color: var(--primary-color);
		margin-bottom: 0.5rem;
	}

	.cta-section p {
		font-size: 16px;
		color: var(--text);
		margin-bottom: 2rem;
	}

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
		.features-grid {
			grid-template-columns: 1fr;
			gap: 1.5rem;
		}

		.feature-card {
			padding: 1.5rem;
		}

		.card-icon {
			font-size: 2.5rem;
		}

		.feature-card h3 {
			font-size: 20px;
		}

		.cta-section {
			padding: 2rem;
		}

		.cta-section h3 {
			font-size: 24px;
		}
	}
</style>
