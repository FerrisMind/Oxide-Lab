<script lang="ts">
	import { cn } from "$lib/utils";

	let {
		class: className,
		children,
		...restProps
	}: {
		class?: string;
		children: import("svelte").Snippet;
		[key: string]: any;
	} = $props();
</script>

<div class={cn("preview-nav", className)} {...restProps}>
	{@render children()}
</div>

<style>
	.preview-nav {
		display: flex;
		align-items: center;
		gap: 0.25rem;
		padding: 0.5rem;
		border-bottom: 1px solid var(--border);
		background: var(--background);
		position: relative;
		z-index: 10;
		border-radius: 1.5rem 1.5rem 0 0;
	}
</style>
