<script lang="ts">
	import type { Tab } from '$lib/tabs/types';
	import { cache } from '$lib/api/cache.svelte';
	import FailView from './FailView.svelte';
	import ErrorView from './ErrorView.svelte';

	let { tab, active }: { tab: Tab; active: boolean } = $props();

	const result = $derived(cache.get(tab.id));

	$effect(() => {
		if (result === undefined) {
			cache.load(tab.id, tab.loader);
		}
	});
</script>

<section
	class="tab-panel absolute inset-0 overflow-y-auto"
	class:active
	inert={!active}
	aria-hidden={!active}
>
	{#if result === undefined}
		<p class="p-6 text-sm text-ink-muted">Loading…</p>
	{:else if result.status === 'success'}
		<div class="h-full p-6">
			<tab.view data={result.data} />
		</div>
	{:else if result.status === 'fail'}
		<FailView {result} />
	{:else}
		<ErrorView {result} />
	{/if}
</section>

<style>
	.tab-panel {
		opacity: 0;
		transform: translateY(6px);
		pointer-events: none;
		transition:
			opacity 260ms cubic-bezier(0.22, 1, 0.36, 1),
			transform 260ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.tab-panel.active {
		opacity: 1;
		transform: none;
		pointer-events: auto;
	}
	@media (prefers-reduced-motion: reduce) {
		.tab-panel {
			transition: opacity 0ms, transform 0ms;
			transform: none;
		}
	}
</style>
