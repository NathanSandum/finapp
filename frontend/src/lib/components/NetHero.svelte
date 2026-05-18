<script lang="ts">
	import { fmt, pct, shortFmt } from '$lib/format';
	import Num from './Num.svelte';
	import Sparkline from './Sparkline.svelte';

	let {
		label,
		value,
		prior,
		trail,
		trailLabels,
		trailKeys,
		gradientId
	}: {
		label: string;
		value: number;
		prior: number;
		trail: number[];
		trailLabels: string[];
		trailKeys: string[];
		gradientId: string;
	} = $props();

	const WIDTH = 560;
	const HEIGHT = 80;

	const min = $derived(trail.length > 0 ? Math.min(...trail) : 0);
	const max = $derived(trail.length > 0 ? Math.max(...trail) : 0);
</script>

<section class="relative shrink-0 overflow-hidden border border-border bg-surface-card">
	<div class="absolute left-5 top-4 z-10">
		<p class="text-[10px] font-medium uppercase tracking-wide text-ink-muted">{label}</p>
		<p
			class="mt-0.5 flex items-baseline gap-2 font-mono text-3xl font-semibold tabular-nums text-ink"
		>
			<Num text={fmt(value)} />
			<span class="text-[13px] font-semibold text-positive">{pct(value, prior)}</span>
		</p>
	</div>
	<svg width="100%" viewBox="0 0 {WIDTH} 100" class="overflow-visible" preserveAspectRatio="none">
		<Sparkline
			values={trail}
			keys={trailKeys}
			width={WIDTH}
			height={HEIGHT}
			lineClass="stroke-positive-line"
			showArea
			{gradientId}
			showMarker
			markerClass="fill-positive-line"
			yOffset={20}
		/>
	</svg>
	<div
		class="pointer-events-none absolute right-2 top-4 flex h-[80px] flex-col justify-between font-mono text-[9px] tabular-nums text-ink-faint"
	>
		<span><Num text={shortFmt(max)} /></span>
		<span><Num text={shortFmt(min)} /></span>
	</div>
	<div
		class="grid pb-2 font-mono text-[9px] tabular-nums text-ink-faint"
		style:grid-template-columns="repeat({trailLabels.length}, 1fr)"
	>
		{#each trailLabels as label}<span class="text-center">{label}</span>{/each}
	</div>
</section>
