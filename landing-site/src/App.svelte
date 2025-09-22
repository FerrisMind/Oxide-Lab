<script>
	import { onMount } from 'svelte';
	import Header from './components/Header.svelte';
	import Hero from './components/Hero.svelte';
	import ThinkingMode from './components/ThinkingMode.svelte';
	import Features from './components/Features.svelte';
	import Specifications from './components/Specifications.svelte';
	import Footer from './components/Footer.svelte';
	import './styles/global.css';

	onMount(() => {
		// Parallax scroll effect
		let ticking = false;

		function updateParallax() {
			const scrolled = window.pageYOffset;
			const parallaxElements = document.querySelectorAll('.parallax');

			parallaxElements.forEach(el => {
				const speed = el.dataset.speed || 0.5;
				const yPos = -(scrolled * speed);
				el.style.transform = `translateY(${yPos}px)`;
			});

			ticking = false;
		}

		function requestTick() {
			if (!ticking) {
				requestAnimationFrame(updateParallax);
				ticking = true;
			}
		}

		window.addEventListener('scroll', requestTick);

		// Intersection Observer for fade-in animations
		const observer = new IntersectionObserver((entries) => {
			entries.forEach(entry => {
				if (entry.isIntersecting) {
					entry.target.classList.add('visible');
				}
			});
		}, { threshold: 0.1 });

		document.querySelectorAll('.fade-in').forEach(el => {
			observer.observe(el);
		});

		return () => {
			window.removeEventListener('scroll', requestTick);
		};
	});
</script>

<main>
	<Header />
	<Hero />
	<ThinkingMode />
	<Features />
	<Specifications />
	<Footer />
</main>

<style>
	main {
		overflow-x: hidden;
	}
</style>