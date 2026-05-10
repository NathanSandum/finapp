<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		sidebar,
		main
	}: {
		sidebar: Snippet;
		main: Snippet;
	} = $props();
</script>

<div class="grid h-screen w-screen grid-cols-[240px_1fr] bg-surface-card text-ink">
	<aside class="col-start-1 flex flex-col overflow-y-auto bg-surface-inverse p-3 text-ink-inverse-body">
		{@render sidebar()}
	</aside>
	<main class="col-start-2 overflow-y-auto">
		{@render main()}
	</main>
</div>
