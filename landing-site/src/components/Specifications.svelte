<script>
	import { onMount } from 'svelte';
	import { animate } from 'animejs';

	onMount(() => {
		// Animate spec items on scroll
		const observer = new IntersectionObserver((entries) => {
			entries.forEach(entry => {
				if (entry.isIntersecting) {
					animate(entry.target, {
						translateX: [50, 0],
						opacity: [0, 1],
						duration: 800,
						easing: 'easeOutQuad'
					});
				}
			});
		});

		document.querySelectorAll('.spec-item').forEach(item => {
			observer.observe(item);
		});
	});
</script>

<section id="specifications" class="specifications">
	<div class="container">
		<h2 class="fade-in">Технические характеристики</h2>
		<p class="section-subtitle fade-in">Системные требования и поддерживаемые модели</p>

		<div class="specs-grid">
			<div class="spec-category">
				<h3>Системные требования</h3>
				<div class="spec-list">
					<div class="spec-item">
						<div class="spec-label">Операционная система</div>
						<div class="spec-value">Windows 10+, macOS 11+, Linux</div>
					</div>
					<div class="spec-item">
						<div class="spec-label">Процессор</div>
						<div class="spec-value">Intel i5 / AMD Ryzen 5 или выше</div>
					</div>
					<div class="spec-item">
						<div class="spec-label">Оперативная память</div>
						<div class="spec-value">8 ГБ минимум, 16 ГБ рекомендуется</div>
					</div>
					<div class="spec-item">
						<div class="spec-label">Место на диске</div>
						<div class="spec-value">2 ГБ для приложения + модели</div>
					</div>
					<div class="spec-item">
						<div class="spec-label">GPU</div>
						<div class="spec-value">NVIDIA GTX 1060+ / AMD RX 580+ (опционально)</div>
					</div>
				</div>
			</div>

			<div class="spec-category">
				<h3>Поддерживаемые модели</h3>
				<div class="spec-list">
					<div class="spec-item">
						<div class="spec-label">Qwen3 GGUF</div>
						<div class="spec-value">7B, 14B, 32B параметры</div>
					</div>
					<div class="spec-item">
						<div class="spec-label">Mistral 7B</div>
						<div class="spec-value">Instruct и Chat версии</div>
					</div>
					<div class="spec-item">
						<div class="spec-label">Llama 2</div>
						<div class="spec-value">7B, 13B, 70B параметры</div>
					</div>
					<div class="spec-item">
						<div class="spec-label">CodeLlama</div>
						<div class="spec-value">7B, 13B, 34B для программирования</div>
					</div>
					<div class="spec-item">
						<div class="spec-label">Phi-2</div>
						<div class="spec-value">2.7B параметров (Microsoft)</div>
					</div>
				</div>
			</div>
		</div>

		<div class="performance-info">
			<h3>Производительность</h3>
			<div class="performance-grid">
				<div class="performance-item">
					<div class="performance-value">~50</div>
					<div class="performance-label">токенов/секунду</div>
					<div class="performance-desc">на CPU (Intel i7)</div>
				</div>
				<div class="performance-item">
					<div class="performance-value">~200</div>
					<div class="performance-label">токенов/секунду</div>
					<div class="performance-desc">на GPU (RTX 4070)</div>
				</div>
				<div class="performance-item">
					<div class="performance-value">99.9%</div>
					<div class="performance-label">аптайм</div>
					<div class="performance-desc">локальная обработка</div>
				</div>
			</div>
		</div>
	</div>
</section>

<style>
	.specifications {
		padding: 4rem 0;
		background: var(--gray-light);
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

	.specs-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
		gap: 3rem;
		margin-bottom: 4rem;
	}

	.spec-category h3 {
		font-size: 24px;
		color: var(--primary-color);
		margin-bottom: 2rem;
		padding-bottom: 0.5rem;
		border-bottom: 3px solid var(--accent-start);
	}

	.spec-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.spec-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1rem;
		background: white;
		border-radius: var(--border-radius);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
		transition: var(--transition);
	}

	.spec-item:hover {
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
		transform: translateY(-2px);
	}

	.spec-label {
		font-weight: 500;
		color: var(--text);
	}

	.spec-value {
		color: var(--accent-start);
		font-weight: 500;
		text-align: right;
	}

	.performance-info {
		background: white;
		padding: 3rem;
		border-radius: var(--border-radius);
		box-shadow: var(--shadow);
	}

	.performance-info h3 {
		text-align: center;
		font-size: 28px;
		color: var(--primary-color);
		margin-bottom: 2rem;
	}

	.performance-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 2rem;
	}

	.performance-item {
		text-align: center;
		padding: 2rem 1rem;
		background: var(--gray-light);
		border-radius: var(--border-radius);
	}

	.performance-value {
		font-size: 2.5rem;
		font-weight: 600;
		color: var(--accent-start);
		margin-bottom: 0.5rem;
	}

	.performance-label {
		font-size: 1.2rem;
		font-weight: 500;
		color: var(--primary-color);
		margin-bottom: 0.5rem;
	}

	.performance-desc {
		font-size: 14px;
		color: var(--text);
	}

	/* Mobile styles */
	@media (max-width: 767px) {
		.specs-grid {
			grid-template-columns: 1fr;
			gap: 2rem;
		}

		.spec-item {
			flex-direction: column;
			align-items: flex-start;
			gap: 0.5rem;
		}

		.spec-value {
			text-align: left;
		}

		.performance-grid {
			grid-template-columns: 1fr;
		}

		.performance-value {
			font-size: 2rem;
		}
	}
</style>
