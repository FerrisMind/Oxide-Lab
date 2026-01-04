<script lang="ts">
	import {
		WebPreview,
		WebPreviewNavigation,
		WebPreviewNavigationButton,
		WebPreviewUrl,
		WebPreviewBody,
		WebPreviewConsole,
		type LogEntry,
	} from "$lib/components/ai-elements/web-preview/index.js";
	import ArrowLeft from "phosphor-svelte/lib/ArrowLeft";
	import ArrowRight from "phosphor-svelte/lib/ArrowRight";
	import ArrowClockwise from "phosphor-svelte/lib/ArrowClockwise";
	import ArrowSquareOut from "phosphor-svelte/lib/ArrowSquareOut";
	import Code from "phosphor-svelte/lib/Code";

	// Dev-only guard: redirect to home in production
	if (!import.meta.env.DEV && typeof window !== "undefined") {
		window.location.href = "/";
	}

	let sampleLogs: LogEntry[] = [
		{
			level: "log",
			message: "Page loaded successfully",
			timestamp: new Date(Date.now() - 10000),
		},
		{
			level: "warn",
			message: "Warning: Deprecated API used",
			timestamp: new Date(Date.now() - 5000),
		},
		{
			level: "error",
			message: "Error: Failed to load resource",
			timestamp: new Date(),
		},
	];

	let currentUrl = $state("https://svelte.dev");
	let showSrcdoc = $state(false);

	const sampleHtml = `
<!DOCTYPE html>
<html>
<head>
	<style>
		body { 
			font-family: system-ui, sans-serif; 
			padding: 2rem; 
			background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
			color: white;
			min-height: 100vh;
			margin: 0;
			display: flex;
			flex-direction: column;
			align-items: center;
			justify-content: center;
		}
		h1 { margin-bottom: 1rem; }
		p { opacity: 0.9; }
	</style>
</head>
<body>
	<h1>🎉 Generated HTML Preview</h1>
	<p>This content is rendered via srcdoc attribute</p>
	<p>Useful for AI-generated artifacts</p>
</body>
</html>`;

	function handleUrlChange(url: string) {
		console.log("URL changed to:", url);
	}

	function goBack() {
		console.log("Going back");
	}

	function goForward() {
		console.log("Going forward");
	}

	function refresh() {
		console.log("Refreshing page");
	}

	function openExternal() {
		if (currentUrl) {
			window.open(currentUrl, "_blank");
		}
	}

	function toggleSrcdoc() {
		showSrcdoc = !showSrcdoc;
	}
</script>

{#if import.meta.env.DEV}
	<div class="mx-auto w-full max-w-4xl space-y-6 p-6">
		<div class="space-y-2">
			<h1 class="text-3xl font-bold">Web Preview Component</h1>
			<p class="text-muted-foreground">
				Dev-only demo page for testing the web-preview component
			</p>
		</div>

		<div class="flex gap-2">
			<button
				class="bg-primary text-primary-foreground hover:bg-primary/90 rounded-md px-4 py-2 text-sm font-medium"
				onclick={toggleSrcdoc}
			>
				{showSrcdoc ? "Show External URL" : "Show srcdoc HTML"}
			</button>
		</div>

		<WebPreview defaultUrl={currentUrl} onUrlChange={handleUrlChange} class="h-[500px]">
			<WebPreviewNavigation>
				<WebPreviewNavigationButton onclick={goBack} tooltip="Go Back">
					<ArrowLeft class="h-4 w-4" />
				</WebPreviewNavigationButton>

				<WebPreviewNavigationButton onclick={goForward} tooltip="Go Forward">
					<ArrowRight class="h-4 w-4" />
				</WebPreviewNavigationButton>

				<WebPreviewNavigationButton onclick={refresh} tooltip="Refresh">
					<ArrowClockwise class="h-4 w-4" />
				</WebPreviewNavigationButton>

				<WebPreviewUrl bind:value={currentUrl} />

				<WebPreviewNavigationButton onclick={toggleSrcdoc} tooltip="Toggle srcdoc mode">
					<Code class="h-4 w-4" />
				</WebPreviewNavigationButton>

				<WebPreviewNavigationButton onclick={openExternal} tooltip="Open in New Tab">
					<ArrowSquareOut class="h-4 w-4" />
				</WebPreviewNavigationButton>
			</WebPreviewNavigation>

			{#if showSrcdoc}
				<WebPreviewBody srcdoc={sampleHtml} />
			{:else}
				<WebPreviewBody />
			{/if}

			<WebPreviewConsole logs={sampleLogs} />
		</WebPreview>

		<div class="bg-muted rounded-lg p-4">
			<h2 class="mb-2 font-semibold">Component Features:</h2>
			<ul class="text-muted-foreground list-inside list-disc space-y-1 text-sm">
				<li>External URL loading via <code>src</code> prop</li>
				<li>Inline HTML via <code>srcdoc</code> prop (click Code button to toggle)</li>
				<li>Parametrizable <code>sandbox</code> (default: no allow-same-origin for security)</li>
				<li>Collapsible console with log/warn/error styling</li>
				<li>Svelte Context API for state management</li>
			</ul>
		</div>
	</div>
{:else}
	<div class="flex h-screen items-center justify-center">
		<p class="text-muted-foreground">Page not found</p>
	</div>
{/if}
