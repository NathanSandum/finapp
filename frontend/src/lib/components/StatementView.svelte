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

	<div class="scroll-shadow min-h-0 flex-1 overflow-y-auto">
		{#if mode === 'summary'}
			{@render summary()}
		{:else if mode === 'detail'}
			{@render detail()}
		{:else}
			{@render trend()}
		{/if}
	</div>
</div>

<style>
	.scroll-shadow {
		background:
			linear-gradient(white 30%, rgba(255, 255, 255, 0)),
			linear-gradient(rgba(255, 255, 255, 0), white 70%) 0 100%,
			radial-gradient(farthest-side at 50% 0, rgba(0, 0, 0, 0.14), rgba(0, 0, 0, 0)),
			radial-gradient(farthest-side at 50% 100%, rgba(0, 0, 0, 0.14), rgba(0, 0, 0, 0)) 0 100%;
		background-color: white;
		background-repeat: no-repeat;
		background-size:
			100% 32px,
			100% 32px,
			100% 12px,
			100% 12px;
		background-attachment: local, local, scroll, scroll;
	}
</style>
