<script lang="ts">
	import GranularityPill from './GranularityPill.svelte';
	import PeriodNav from './PeriodNav.svelte';
	import type { Granularity } from '$lib/views/period';

	let {
		title,
		periodLabel,
		granularity,
		onPrev,
		onNext,
		onGranularity
	}: {
		title: string;
		periodLabel: string;
		granularity: Granularity;
		onPrev: () => void;
		onNext: () => void;
		onGranularity: (g: Granularity) => void;
	} = $props();
</script>

<header class="flex shrink-0 items-baseline justify-between">
	<div class="flex items-baseline gap-3">
		<h3 class="text-xl font-semibold text-ink">{title}</h3>
		<PeriodNav label={periodLabel} {onPrev} {onNext} />
	</div>
	<GranularityPill {granularity} onSelect={onGranularity} />
</header>
