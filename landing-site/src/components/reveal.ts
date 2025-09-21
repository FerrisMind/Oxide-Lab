import anime from 'animejs/lib/anime.es.js';

export type RevealOptions = {
  delay?: number;
  translateY?: number;
};

export function reveal(node: HTMLElement, options: RevealOptions = {}) {
  const { delay = 0, translateY = 32 } = options;
  let played = false;

  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting && !played) {
          played = true;
          anime({
            targets: node,
            opacity: [0, 1],
            translateY: [translateY, 0],
            duration: 560,
            delay,
            easing: 'easeOutCubic'
          });
          observer.unobserve(node);
        }
      }
    },
    { threshold: 0.18 }
  );

  observer.observe(node);

  return {
    destroy() {
      observer.disconnect();
    }
  };
}
