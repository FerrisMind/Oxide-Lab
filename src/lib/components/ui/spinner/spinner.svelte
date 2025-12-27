<script lang="ts">
	import { cn } from "../../../utils.js";
	import CircleNotch from "phosphor-svelte/lib/CircleNotch";
	import type { ComponentProps } from "svelte";

	let { class: className, ...restProps }: ComponentProps<typeof CircleNotch> = $props();
</script>

<CircleNotch
	role="status"
	aria-label="Loading"
	class={cn("size-4 animate-spin", className)}
	{...restProps}
/>
