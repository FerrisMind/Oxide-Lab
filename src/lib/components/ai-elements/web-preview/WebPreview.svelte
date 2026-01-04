<script lang="ts">
	import { cn } from "$lib/utils";
	import { WebPreviewContext, setWebPreviewContext } from "./web-preview-context.svelte.js";

	let {
		defaultUrl = "",
		onUrlChange,
		class: className,
		children,
		...restProps
	}: {
		defaultUrl?: string;
		onUrlChange?: (url: string) => void;
		class?: string;
		children: import("svelte").Snippet;
		[key: string]: any;
	} = $props();

	let context = new WebPreviewContext(defaultUrl, onUrlChange);
	setWebPreviewContext(context);
</script>

<div class={cn("bg-card flex size-full flex-col rounded-3xl border overflow-hidden", className)} {...restProps}>
	{@render children()}
</div>
