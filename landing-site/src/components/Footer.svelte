<script lang="ts">
	import { onMount } from 'svelte';
	import { animate } from 'animejs';
	import { Star, ChatCircle, BookOpen, Bug, Lightbulb, Envelope, ArrowDown, Book, HandHeart, Heart } from 'phosphor-svelte';

	onMount(() => {
		// Animate footer elements on load
		animate('.footer-section', {
			translateY: [30, 0],
			opacity: [0, 1],
			duration: 1000,
			delay: 200,
			easing: 'easeOutQuad'
		});

		// GitHub star animation
		animate('.github-stars', {
			textContent: [0, 247],
			round: 1,
			duration: 2000,
			easing: 'easeOutQuad'
		});
	});

	function scrollToTop() {
		window.scrollTo({ top: 0, behavior: 'smooth' });
	}
</script>

<footer id="footer" class="footer">
	<div class="container">
		<div class="footer-content">
			<div class="footer-section fade-in">
				<h3>Oxide Lab</h3>
				<p>Ваш личный ИИ-ассистент с полным контролем над приватностью данных.</p>
				<button class="scroll-to-top" on:click={scrollToTop}>
					<span>↑</span>
				</button>
			</div>

			<div class="footer-section">
				<h4>Сообщество</h4>
				<div class="social-links">
					<a href="https://github.com/FerrisMind/Oxide-Lab" target="_blank" class="social-link">
						<Star size={16} class="github-icon" />
						<span class="github-stars">247</span> GitHub Stars
					</a>
					<a href="https://github.com/FerrisMind/Oxide-Lab/discussions" target="_blank" class="social-link">
						<ChatCircle size={16} /> Обсуждения
					</a>
					<a href="https://github.com/FerrisMind/Oxide-Lab/wiki" target="_blank" class="social-link">
						<BookOpen size={16} /> Документация
					</a>
				</div>
			</div>

			<div class="footer-section">
				<h4>Поддержка</h4>
				<div class="support-links">
					<a href="https://github.com/FerrisMind/Oxide-Lab/issues" target="_blank" class="support-link">
						<Bug size={16} /> Сообщить об ошибке
					</a>
					<a href="https://github.com/FerrisMind/Oxide-Lab/discussions/categories/feature-requests" target="_blank" class="support-link">
						<Lightbulb size={16} /> Предложить функцию
					</a>
					<a href="mailto:support@oxide-lab.com" class="support-link">
						<Envelope size={16} /> Поддержка
					</a>
				</div>
			</div>

			<div class="footer-section">
				<h4>Ресурсы</h4>
				<div class="resource-links">
					<a href="https://github.com/FerrisMind/Oxide-Lab/releases" target="_blank" class="resource-link">
						<ArrowDown size={16} /> Скачать
					</a>
					<a href="https://github.com/FerrisMind/Oxide-Lab/blob/main/README.md" target="_blank" class="resource-link">
						<Book size={16} /> README
					</a>
					<a href="https://github.com/FerrisMind/Oxide-Lab/blob/main/CONTRIBUTING.md" target="_blank" class="resource-link">
						<HandHeart size={16} /> Внести вклад
					</a>
				</div>
			</div>
		</div>

		<div class="footer-bottom">
			<p>&copy; 2025 Oxide Lab. Открытый исходный код под лицензией MIT.</p>
			<p>Создано с <Heart size={16} /> для сообщества приватного ИИ.</p>
		</div>
	</div>
</footer>

<style>
	.footer {
		background: var(--primary-color);
		color: white;
		padding: 4rem 0 2rem;
	}

	.footer-content {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 3rem;
		margin-bottom: 3rem;
	}

	.footer-section h3 {
		font-size: 24px;
		margin-bottom: 1rem;
		color: white;
	}

	.footer-section h4 {
		font-size: 18px;
		margin-bottom: 1rem;
		color: var(--accent-start);
	}

	.footer-section p {
		font-size: 14px;
		line-height: 1.6;
		margin-bottom: 1rem;
		opacity: 0.9;
	}

	.social-links, .support-links, .resource-links {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.social-link, .support-link, .resource-link {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 14px;
		color: rgba(255, 255, 255, 0.8);
		transition: var(--transition);
		text-decoration: none;
	}

	.social-link:hover, .support-link:hover, .resource-link:hover {
		color: var(--accent-start);
		transform: translateX(5px);
	}

	.github-stars {
		font-weight: 600;
		color: var(--accent-start);
	}

	.scroll-to-top {
		position: fixed;
		bottom: 2rem;
		right: 2rem;
		background: var(--accent-start);
		color: white;
		border: none;
		width: 50px;
		height: 50px;
		border-radius: 50%;
		cursor: pointer;
		box-shadow: 0 4px 12px rgba(255, 107, 53, 0.3);
		transition: var(--transition);
		z-index: 1000;
	}

	.scroll-to-top:hover {
		background: var(--accent-end);
		transform: translateY(-2px);
		box-shadow: 0 6px 20px rgba(255, 107, 53, 0.4);
	}

	.scroll-to-top span {
		font-size: 1.5rem;
		line-height: 1;
	}

	.footer-bottom {
		border-top: 1px solid rgba(255, 255, 255, 0.1);
		padding-top: 2rem;
		text-align: center;
	}

	.footer-bottom p {
		font-size: 14px;
		opacity: 0.7;
		margin-bottom: 0.5rem;
	}

	/* Mobile styles */
	@media (max-width: 767px) {
		.footer-content {
			grid-template-columns: 1fr;
			gap: 2rem;
		}

		.scroll-to-top {
			bottom: 1rem;
			right: 1rem;
			width: 40px;
			height: 40px;
		}

		.scroll-to-top span {
			font-size: 1.2rem;
		}
	}
</style>
