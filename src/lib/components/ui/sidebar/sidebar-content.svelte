<script lang="ts">
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/components/ai-elements/markdown/utils/utils";

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLElement>> = $props();
</script>

<div
	bind:this={ref}
	data-slot="sidebar-content"
	data-sidebar="content"
	class={cn(
		"flex min-h-0 flex-1 flex-col gap-2 overflow-auto custom-scrollbar group-data-[collapsible=icon]:overflow-hidden",
		className
	)}
	{...restProps}
>
	{@render children?.()}
</div>
