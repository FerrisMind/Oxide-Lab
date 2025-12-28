<script lang="ts">
	import { cn } from "$lib/components/ai-elements/markdown/utils/utils";
	import { getReasoningContext } from "./reasoning-context.svelte.js";
	import { tick } from "svelte";

	interface Props {
		class?: string;
		children?: import("svelte").Snippet;
	}

	let { class: className = "", children, ...props }: Props = $props();

	let reasoningContext = getReasoningContext();
	
	let wrapperEl: HTMLDivElement | undefined = $state();
	let contentEl: HTMLDivElement | undefined = $state();
	let contentHeight = $state(0);
	let hasOverflow = $state(false);
	
	const COLLAPSED_HEIGHT = 192; // 12rem

	// Measure and update transform
	async function updateTransform() {
		await tick();
		if (!contentEl || !wrapperEl) return;
		
		contentHeight = contentEl.scrollHeight;
		const wrapperHeight = wrapperEl.clientHeight;
		const isCollapsed = !reasoningContext.isOpen;
		const isFinished = !reasoningContext.isStreaming;
		
		if (isCollapsed && !isFinished && contentHeight > wrapperHeight) {
			// Show last portion by translating up
			const translateY = -(contentHeight - wrapperHeight);
			contentEl.style.transform = `translateY(${translateY}px)`;
			hasOverflow = true;
		} else {
			contentEl.style.transform = 'translateY(0)';
			hasOverflow = false;
		}
	}

	// Watch for state changes
	$effect(() => {
		const _ = [reasoningContext.isOpen, reasoningContext.isStreaming, children];
		updateTransform();
	});
	
	// MutationObserver for dynamic content
	$effect(() => {
		if (!contentEl) return;
		
		const observer = new MutationObserver(() => {
			updateTransform();
		});
		
		observer.observe(contentEl, { 
			childList: true, 
			subtree: true, 
			characterData: true 
		});
		
		return () => observer.disconnect();
	});

	// Derived states
	let isCollapsed = $derived(!reasoningContext.isOpen);
	let isFinished = $derived(!reasoningContext.isStreaming);
	let showGradient = $derived(isCollapsed && hasOverflow && !isFinished);
	
	// Calculate max height
	let maxHeight = $derived.by(() => {
		if (isCollapsed) {
			return isFinished ? '0px' : `${COLLAPSED_HEIGHT}px`;
		}
		return contentHeight ? `${contentHeight}px` : 'none';
	});
</script>

<div
	bind:this={wrapperEl}
	class={cn(
		"text-xs text-muted-foreground/75 rounded-md relative ml-6 mt-2",
		"transition-[max-height,opacity] duration-300 ease-in-out",
		isCollapsed ? "overflow-hidden" : "overflow-y-auto",
		className
	)}
	style:max-height={maxHeight}
	style:opacity={isCollapsed && isFinished ? 0 : 1}
	{...props}
>
	<div
		bind:this={contentEl}
		class="transition-transform duration-300 opacity-75 select-text"
	>
		{@render children?.()}
	</div>

	<!-- Gradient overlay for fade effect when collapsed and has overflow (like Ollama) -->
	{#if showGradient}
		<div 
			class="absolute inset-x-0 -top-1 h-8 pointer-events-none bg-gradient-to-b from-background to-transparent"
		></div>
	{/if}
</div>
