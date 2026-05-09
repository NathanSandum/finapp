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

<div class="grid h-screen w-screen grid-cols-[240px_1fr] bg-white text-neutral-900">
	<aside class="col-start-1 flex flex-col overflow-y-auto bg-neutral-900 p-3 text-neutral-200">
		{@render sidebar()}
	</aside>
	<main class="col-start-2 overflow-y-auto">
		{@render main()}
	</main>
</div>
