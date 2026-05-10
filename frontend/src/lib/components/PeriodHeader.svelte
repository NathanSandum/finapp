<script lang="ts">
	import type { Snippet } from 'svelte';
	import GranularityPill from './GranularityPill.svelte';
	import PeriodNav from './PeriodNav.svelte';
	import type { Granularity } from '$lib/views/period';

	let {
		title,
		periodLabel,
		granularity,
		onPrev,
		onNext,
		onGranularity,
		actions
	}: {
		title: string;
		periodLabel: string;
		granularity: Granularity;
		onPrev: () => void;
		onNext: () => void;
		onGranularity: (g: Granularity) => void;
		actions?: Snippet;
	} = $props();
</script>

<header class="flex shrink-0 items-baseline justify-between gap-3">
	<div class="flex items-baseline gap-3">
		<h3 class="text-xl font-semibold text-ink">{title}</h3>
		<PeriodNav label={periodLabel} {onPrev} {onNext} />
	</div>
	<div class="flex items-baseline gap-3">
		{#if actions}
			{@render actions()}
		{/if}
		<GranularityPill {granularity} onSelect={onGranularity} />
	</div>
</header>
