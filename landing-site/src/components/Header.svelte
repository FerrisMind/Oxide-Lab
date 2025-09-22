<script>
	import { onMount } from 'svelte';
	import { animate } from 'animejs';

	onMount(() => {
		// Wait for DOM to be fully loaded
		setTimeout(() => {
			// Pulsing logo animation
			animate('.logo', {
				scale: 1.05,
				duration: 2000,
				easing: 'easeInOutSine',
				direction: 'alternate',
				loop: true
			});
		}, 100);

		// Smooth scroll for navigation links
		const links = document.querySelectorAll('.nav-link');
		links.forEach(link => {
			link.addEventListener('click', (e) => {
				e.preventDefault();
				const target = document.querySelector(link.getAttribute('href'));
				if (target) {
					target.scrollIntoView({ behavior: 'smooth' });
				}
			});
		});
	});
</script>

<header class="header">
	<nav class="nav">
		<ul class="nav-list">
			<li><a href="#features" class="nav-link">Возможности</a></li>
			<li><a href="#thinking-mode" class="nav-link">Режим мышления</a></li>
			<li><a href="#specifications" class="nav-link">Технические характеристики</a></li>
			<li><a href="#footer" class="nav-link">Контакты</a></li>
		</ul>
	</nav>
	<div class="logo">
		<span>Oxide Lab</span>
	</div>
</header>

<style>
	.header {
		position: fixed;
		top: 0;
		left: 0;
		width: 100%;
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1rem;
		background: rgba(255, 255, 255, 0.95);
		backdrop-filter: blur(10px);
		z-index: 100;
		transition: var(--transition);
	}

	.nav-list {
		display: flex;
		list-style: none;
		margin: 0;
		padding: 0;
		gap: 2rem;
	}

	.nav-link {
		font-size: 14px;
		font-weight: 500;
		color: var(--text);
		position: relative;
	}

	.nav-link::after {
		content: '';
		position: absolute;
		bottom: -4px;
		left: 0;
		width: 0;
		height: 2px;
		background: linear-gradient(to right, var(--accent-start), var(--accent-end));
		transition: width 0.3s ease;
	}

	.nav-link:hover::after {
		width: 100%;
	}

	.logo {
		font-size: 1.5rem;
		font-weight: 600;
		color: var(--primary-color);
		cursor: pointer;
	}

	/* Mobile styles */
	@media (max-width: 767px) {
		.header {
			flex-direction: column;
			gap: 1rem;
			padding: 0.5rem;
		}

		.nav-list {
			flex-direction: column;
			align-items: center;
			gap: 1rem;
		}

		.nav-link {
			font-size: 16px;
		}

		.logo {
			font-size: 1.2rem;
		}
	}
</style>
