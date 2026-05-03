<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		header,
		sidebar,
		main
	}: {
		header: Snippet;
		sidebar: Snippet;
		main: Snippet;
	} = $props();
</script>

<div class="grid h-screen w-screen grid-cols-[240px_1fr] grid-rows-[56px_1fr] bg-neutral-50 text-neutral-900">
	<header class="col-span-2 row-start-1 flex items-center border-b border-neutral-200 bg-white px-4">
		{@render header()}
	</header>
	<aside class="col-start-1 row-start-2 overflow-y-auto border-r border-neutral-200 bg-white p-2">
		{@render sidebar()}
	</aside>
	<main class="col-start-2 row-start-2 overflow-y-auto p-6">
		{@render main()}
	</main>
</div>
