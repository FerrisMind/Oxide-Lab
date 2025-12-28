<script lang="ts">
	import { cn } from "$lib/components/ai-elements/markdown/utils/utils";
	import { watch } from "runed";
	import { ReasoningContext, setReasoningContext } from "./reasoning-context.svelte";
	import Shimmer from "$lib/components/ai-elements/shimmer/Shimmer.svelte";
	import CaretRight from "phosphor-svelte/lib/CaretRight";
	import CaretDown from "phosphor-svelte/lib/CaretDown";
	import Brain from "phosphor-svelte/lib/Brain";
	import { t } from '$lib/i18n';

	interface Props {
		class?: string;
		isStreaming?: boolean;
		open?: boolean;
		defaultOpen?: boolean;
		onOpenChange?: (open: boolean) => void;
		duration?: number;
		children?: import("svelte").Snippet;
	}

	let {
		class: className = "",
		isStreaming = false,
		open = $bindable(),
		defaultOpen = false,
		onOpenChange,
		duration = $bindable(),
		children,
		...props
	}: Props = $props();

	let MS_IN_S = 1000;

	// Create the reasoning context
	let reasoningContext = new ReasoningContext({
		isStreaming,
		isOpen: open ?? defaultOpen,
		duration: duration ?? 0,
	});

	// Set up controllable state
	let isOpen = $state(open ?? defaultOpen);
	let currentDuration = $state(duration ?? 0);
	let hasUserInteracted = $state(false);
	let startTime = $state<number | null>(null);

	// Sync external props to context
	$effect(() => {
		reasoningContext.isStreaming = isStreaming;
	});

	$effect(() => {
		if (open !== undefined) {
			isOpen = open;
			reasoningContext.isOpen = open;
		}
	});

	$effect(() => {
		if (duration !== undefined) {
			currentDuration = duration;
			reasoningContext.duration = duration;
		}
	});

	// Track duration
	watch(
		() => isStreaming,
		(isStreamingValue) => {
			if (isStreamingValue) {
				if (startTime === null) {
					startTime = Date.now();
				}
				hasUserInteracted = false;
			} else if (startTime !== null) {
				let newDuration = Math.ceil((Date.now() - startTime) / MS_IN_S);
				currentDuration = newDuration;
				reasoningContext.duration = newDuration;
				if (duration !== undefined) {
					duration = newDuration;
				}
				startTime = null;
				
				// Auto-collapse
				if (!hasUserInteracted) {
					isOpen = false;
					reasoningContext.setIsOpen(false);
					onOpenChange?.(false);
				}
			}
		}
	);

	function handleToggle() {
		isOpen = !isOpen;
		reasoningContext.setIsOpen(isOpen);
		hasUserInteracted = true;
		if (open !== undefined) {
			open = isOpen;
		}
		onOpenChange?.(isOpen);
	}

	// Text for trigger (with i18n)
	let triggerText = $derived.by(() => {
		if (isStreaming) return $t('chat.thinking.thinking') || 'Thinking';
		if (currentDuration < 2) return $t('chat.thinking.thoughtMoment') || 'Thought for a moment';
		return $t('chat.thinking.thoughtSeconds', { seconds: currentDuration }) || `Thought for ${currentDuration} seconds`;
	});

	setReasoningContext(reasoningContext);
</script>

<div
	class={cn("not-prose mb-4 flex flex-col w-full", className)}
	data-state={isOpen ? "open" : "closed"}
	{...props}
>
	<!-- Trigger -->
	<button
		type="button"
		class={cn(
			"group/trigger flex items-center gap-1.5 self-start text-sm transition-colors",
			isStreaming || isOpen
				? "text-foreground"
				: "text-muted-foreground hover:text-foreground"
		)}
		onclick={handleToggle}
	>
		<!-- Icon container with hover effect -->
		<span class="relative w-4 h-4 flex items-center justify-center">
			{#if isStreaming}
				<Brain size={16} weight="fill" />
			{:else if isOpen}
				<CaretDown size={16} weight="bold" />
			{:else}
				<!-- Brain visible by default, caret on hover -->
				<Brain 
					size={16} 
					weight="regular" 
					class="absolute transition-opacity opacity-100 group-hover/trigger:opacity-0" 
				/>
				<CaretRight 
					size={16} 
					weight="bold" 
					class="absolute transition-opacity opacity-0 group-hover/trigger:opacity-100" 
				/>
			{/if}
		</span>
		
		{#if isStreaming}
			<Shimmer as="span" duration={2} spread={1.5} content_length={triggerText.length}>
				{triggerText}...
			</Shimmer>
		{:else}
			<span>{triggerText}</span>
		{/if}
	</button>

	<!-- Content -->
	{@render children?.()}
</div>
