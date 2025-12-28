import { MediaQuery } from "svelte/reactivity";

/**
 * Mobile breakpoint for sidebar behavior.
 * On screens <= this width, sidebar opens as Sheet overlay.
 * Must match Tailwind's md breakpoint (768px) for consistency.
 */
const DEFAULT_MOBILE_BREAKPOINT = 768;

export class IsMobile extends MediaQuery {
	constructor(breakpoint: number = DEFAULT_MOBILE_BREAKPOINT) {
		super(`max-width: ${breakpoint - 1}px`);
	}
}
