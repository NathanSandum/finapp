<script lang="ts">
	import type { Snippet } from 'svelte';
	import ModePill, { type Mode } from './ModePill.svelte';

	let {
		hero,
		tiles,
		summary,
		detail,
		trend
	}: {
		hero: Snippet;
		tiles: Snippet;
		summary: Snippet;
		detail: Snippet;
		trend: Snippet;
	} = $props();

	let mode = $state<Mode>('summary');

	let scrollEl: HTMLDivElement | undefined;
	let atTop = $state(true);
	let atBottom = $state(true);

	function recompute() {
		if (!scrollEl) return;
		atTop = scrollEl.scrollTop <= 0;
		atBottom = scrollEl.scrollTop + scrollEl.clientHeight >= scrollEl.scrollHeight - 1;
	}

	$effect(() => {
		if (!scrollEl) return;
		recompute();
		const ro = new ResizeObserver(recompute);
		ro.observe(scrollEl);
		for (const child of scrollEl.children) ro.observe(child);
		return () => ro.disconnect();
	});

	$effect(() => {
		void mode;
		queueMicrotask(recompute);
	});
</script>

<div class="flex h-full flex-col gap-3 overflow-hidden">
	{@render hero()}
	{@render tiles()}

	<ModePill {mode} onSelect={(next) => (mode = next)} />

	<div class="relative min-h-0 flex-1">
		<div
			bind:this={scrollEl}
			onscroll={recompute}
			class="h-full overflow-y-auto"
		>
			{#if mode === 'summary'}
				{@render summary()}
			{:else if mode === 'detail'}
				{@render detail()}
			{:else}
				{@render trend()}
			{/if}
		</div>
		<div
			aria-hidden="true"
			class="pointer-events-none absolute inset-x-0 top-0 z-10 h-4 bg-gradient-to-b from-black/12 to-transparent transition-opacity duration-150 {atTop
				? 'opacity-0'
				: 'opacity-100'}"
		></div>
		<div
			aria-hidden="true"
			class="pointer-events-none absolute inset-x-0 bottom-0 z-10 h-4 bg-gradient-to-t from-black/12 to-transparent transition-opacity duration-150 {atBottom
				? 'opacity-0'
				: 'opacity-100'}"
		></div>
	</div>
</div>
