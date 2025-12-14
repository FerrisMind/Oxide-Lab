<script lang="ts">
	import { Button } from "$lib/components/ui/button/index.js";
	import { cn } from "$lib/utils.js";
import SidebarSimpleIcon from "phosphor-svelte/lib/SidebarSimple";
	import type { ComponentProps } from "svelte";
	import { useSidebar } from "./context.svelte.js";

	let {
		ref = $bindable(null),
		class: className,
		onclick,
		...restProps
	}: ComponentProps<typeof Button> & {
		onclick?: (e: MouseEvent) => void;
	} = $props();

	const sidebar = useSidebar();
</script>

<Button
	data-sidebar="trigger"
	data-slot="sidebar-trigger"
	variant="ghost"
	size="icon"
	class={cn("size-7", className)}
	type="button"
	onclick={(e) => {
		onclick?.(e);
		sidebar.toggle();
	}}
	{...restProps}
>
	<SidebarSimpleIcon />
	<span class="sr-only">Toggle Sidebar</span>
</Button>
