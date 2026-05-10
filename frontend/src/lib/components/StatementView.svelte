<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { Granularity } from '$lib/views/period';
	import ModePill, { type Mode } from './ModePill.svelte';
	import PeriodHeader from './PeriodHeader.svelte';

	let {
		title,
		periodLabel,
		granularity,
		onPrev,
		onNext,
		onGranularity,
		hero,
		tiles,
		summary,
		detail,
		trend
	}: {
		title: string;
		periodLabel: string;
		granularity: Granularity;
		onPrev: () => void;
		onNext: () => void;
		onGranularity: (g: Granularity) => void;
		hero: Snippet;
		tiles: Snippet;
		summary: Snippet;
		detail: Snippet;
		trend: Snippet;
	} = $props();

	let mode = $state<Mode>('summary');
</script>

<div class="flex h-full flex-col gap-3 overflow-hidden">
	<PeriodHeader {title} {periodLabel} {granularity} {onPrev} {onNext} {onGranularity} />

	{@render hero()}
	{@render tiles()}

	<ModePill {mode} onSelect={(next) => (mode = next)} />

	<div class="min-h-0 flex-1 overflow-y-auto">
		{#if mode === 'summary'}
			{@render summary()}
		{:else if mode === 'detail'}
			{@render detail()}
		{:else}
			{@render trend()}
		{/if}
	</div>
</div>
