<script lang="ts">
	import { onMount } from 'svelte';
	import { animate } from 'animejs';
	import { Brain, Lightning, Lock } from 'phosphor-svelte';

	let isThinking = false;
	let thinkingSteps = [
		'Анализ запроса...',
		'Обработка данных...',
		'Генерация ответа...',
		'Оптимизация...',
		'Готово!'
	];
	let currentStep = 0;

	onMount(() => {
		// Continuous neural network animation
		animate('.neuron', {
			scale: [1, 1.5],
			opacity: [0.3, 0.8],
			duration: 1500,
			delay: 100,
			easing: 'easeInOutSine',
			loop: true,
			direction: 'alternate'
		});

		// Pulse connections
		animate('.connection', {
			strokeDashoffset: [0, -20],
			duration: 2000,
			easing: 'easeInOutSine',
			loop: true,
			direction: 'alternate'
		});
	});

	function startThinking() {
		if (isThinking) return;

		isThinking = true;
		currentStep = 0;

		const thinkingInterval = setInterval(() => {
			if (currentStep < thinkingSteps.length - 1) {
				currentStep++;
			} else {
				clearInterval(thinkingInterval);
				setTimeout(() => {
					isThinking = false;
					currentStep = 0;
				}, 1000);
			}
		}, 800);
	}

	function resetThinking() {
		isThinking = false;
		currentStep = 0;
	}
</script>

<section id="thinking-mode" class="thinking-mode">
	<div class="container">
		<h2 class="fade-in">Режим мышления ИИ</h2>
		<p class="section-subtitle fade-in">Наблюдайте за процессом анализа и генерации ответов в реальном времени</p>

		<div class="thinking-container">
			<div class="neural-network">
				<!-- Neural network visualization -->
				<svg class="network-svg" viewBox="0 0 400 300">
					<!-- Neurons -->
					{#each Array(12) as _, i}
						<circle
							class="neuron"
							cx={50 + (i % 4) * 100}
							cy={50 + Math.floor(i / 4) * 100}
							r="8"
							fill="var(--accent-start)"
						/>
					{/each}

					<!-- Connections -->
					<line class="connection" x1="50" y1="50" x2="150" y2="150" stroke="var(--accent-end)" stroke-width="2" stroke-dasharray="5,5" />
					<line class="connection" x1="150" y1="50" x2="250" y2="150" stroke="var(--accent-end)" stroke-width="2" stroke-dasharray="5,5" />
					<line class="connection" x1="250" y1="50" x2="350" y2="150" stroke="var(--accent-end)" stroke-width="2" stroke-dasharray="5,5" />
					<line class="connection" x1="50" y1="150" x2="150" y2="250" stroke="var(--accent-end)" stroke-width="2" stroke-dasharray="5,5" />
					<line class="connection" x1="150" y1="150" x2="250" y2="250" stroke="var(--accent-end)" stroke-width="2" stroke-dasharray="5,5" />
					<line class="connection" x1="250" y1="150" x2="350" y2="250" stroke="var(--accent-end)" stroke-width="2" stroke-dasharray="5,5" />
				</svg>
			</div>

			<div class="thinking-controls">
				<div class="thinking-status">
					<h3>Статус обработки:</h3>
					<p class="current-step">{thinkingSteps[currentStep]}</p>
					<div class="progress-bar">
						<div class="progress-fill" style="width: {(currentStep / (thinkingSteps.length - 1)) * 100}%"></div>
					</div>
				</div>

				<div class="control-buttons">
					<button class="btn btn-primary" on:click={startThinking} disabled={isThinking}>
						{isThinking ? 'Обработка...' : 'Начать мышление'}
					</button>
					<button class="btn btn-secondary" on:click={resetThinking} disabled={!isThinking}>
						Остановить
					</button>
				</div>
			</div>
		</div>

		<div class="features-list">
			<div class="feature-item">
				<div class="feature-icon"><Brain size={24} /></div>
				<h4>Нейронные сети</h4>
				<p>Распределенная обработка на локальном устройстве</p>
			</div>
			<div class="feature-item">
				<div class="feature-icon"><Lightning size={24} /></div>
				<h4>Быстрая обработка</h4>
				<p>Оптимизированные алгоритмы для максимальной скорости</p>
			</div>
			<div class="feature-item">
				<div class="feature-icon"><Lock size={24} /></div>
				<h4>Полная приватность</h4>
				<p>Все данные остаются на вашем устройстве</p>
			</div>
		</div>
	</div>
</section>

<style>
	.thinking-mode {
		background: var(--gray-light);
		padding: 4rem 0;
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

	.thinking-container {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 3rem;
		margin-bottom: 3rem;
		align-items: center;
	}

	.neural-network {
		display: flex;
		justify-content: center;
	}

	.network-svg {
		width: 100%;
		max-width: 400px;
		height: auto;
	}

	.neuron {
		transition: var(--transition);
	}

	.connection {
		opacity: 0.5;
	}

	.thinking-controls {
		display: flex;
		flex-direction: column;
		gap: 2rem;
	}

	.thinking-status h3 {
		font-size: 18px;
		color: var(--primary-color);
		margin-bottom: 0.5rem;
	}

	.current-step {
		font-size: 16px;
		font-weight: 500;
		color: var(--accent-start);
		margin-bottom: 1rem;
	}

	.progress-bar {
		width: 100%;
		height: 8px;
		background: var(--gray-medium);
		border-radius: 4px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background: linear-gradient(to right, var(--accent-start), var(--accent-end));
		transition: width 0.3s ease;
		border-radius: 4px;
	}

	.control-buttons {
		display: flex;
		gap: 1rem;
	}

	.btn {
		padding: 0.75rem 1.5rem;
		border-radius: var(--border-radius);
		font-size: 14px;
		font-weight: 500;
		transition: var(--transition);
	}

	.btn-primary {
		background: linear-gradient(45deg, var(--accent-start), var(--accent-end));
		color: white;
	}

	.btn-primary:hover:not(:disabled) {
		transform: translateY(-2px);
		box-shadow: var(--shadow);
	}

	.btn-secondary {
		background: transparent;
		color: var(--text);
		border: 2px solid var(--gray-medium);
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.features-list {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 2rem;
	}

	.feature-item {
		text-align: center;
		padding: 1.5rem;
		background: white;
		border-radius: var(--border-radius);
		box-shadow: var(--shadow);
		transition: var(--transition);
	}

	.feature-item:hover {
		transform: translateY(-4px);
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
	}

	.feature-icon {
		font-size: 2rem;
		margin-bottom: 1rem;
	}

	.feature-item h4 {
		font-size: 18px;
		color: var(--primary-color);
		margin-bottom: 0.5rem;
	}

	.feature-item p {
		font-size: 14px;
		color: var(--text);
	}

	/* Mobile styles */
	@media (max-width: 767px) {
		.thinking-container {
			grid-template-columns: 1fr;
			gap: 2rem;
		}

		.control-buttons {
			flex-direction: column;
		}

		.features-list {
			grid-template-columns: 1fr;
		}

		h2 {
			font-size: 28px;
		}
	}
</style>
