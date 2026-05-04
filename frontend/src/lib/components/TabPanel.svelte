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

<section class="h-full" style:display={active ? 'block' : 'none'}>
	{#if result === undefined}
		<p class="text-sm text-neutral-500">Loading…</p>
	{:else if result.status === 'success'}
		<tab.view data={result.data} />
	{:else if result.status === 'fail'}
		<FailView {result} />
	{:else}
		<ErrorView {result} />
	{/if}
</section>
