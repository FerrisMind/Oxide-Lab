<script lang="ts">
	import { Tooltip as TooltipPrimitive } from "bits-ui";
	import { cn } from "$lib/utils.js";

let {
	ref = $bindable(null),
	class: className,
	sideOffset = 4,
	side = "top",
	children,
	...restProps
}: TooltipPrimitive.ContentProps = $props();
</script>

<TooltipPrimitive.Portal>
	<TooltipPrimitive.Content
		bind:ref
		data-slot="tooltip-content"
		{sideOffset}
		{side}
		class={cn(
			"z-50 grid place-items-center rounded-lg bg-[color:var(--tooltip-bg)] px-8 py-2.5 min-w-[6.5rem] min-h-[1.75rem] text-[0.68rem] text-center leading-[1.05rem] font-medium tracking-[-0.04em] text-[color:var(--tooltip-fg)] shadow-[var(--tooltip-shadow)] animate-in fade-in-0 zoom-in-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-end-2 data-[side=right]:slide-in-from-start-2 data-[side=top]:slide-in-from-bottom-2 origin-(--bits-tooltip-content-transform-origin) relative overflow-visible",
			className
		)}
		{...restProps}
	>
		{@render children?.()}
	</TooltipPrimitive.Content>
</TooltipPrimitive.Portal>
