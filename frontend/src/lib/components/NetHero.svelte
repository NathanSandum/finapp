<script lang="ts">
	import { onDestroy } from 'svelte';
	import { fmt, pct, shortFmt } from '$lib/format';
	import AnimatedLabels from './AnimatedLabels.svelte';
	import Num from './Num.svelte';
	import Sparkline from './Sparkline.svelte';
	import { TrailSlots } from './trailSlots.svelte';

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

	const trailSlots = new TrailSlots();
	$effect(() => {
		trailSlots.sync(trailKeys, trail, WIDTH, HEIGHT);
	});
	onDestroy(() => trailSlots.destroy());
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
			{trailSlots}
			keys={trailKeys}
			width={WIDTH}
			height={HEIGHT}
			lineClass="stroke-positive-line"
			showArea
			{gradientId}
			showMarker
			markerClass="fill-positive-line"
			showCurrentLine
			currentLineClass="stroke-ink-faint"
			yOffset={20}
		/>
	</svg>
	<div
		class="pointer-events-none absolute right-2 top-4 flex h-[80px] flex-col justify-between font-mono text-[9px] tabular-nums text-ink-faint"
	>
		<span><Num text={shortFmt(max)} /></span>
		<span><Num text={shortFmt(min)} /></span>
	</div>
	<div class="pb-2 font-mono text-[9px] tabular-nums text-ink-faint">
		<AnimatedLabels
			{trailSlots}
			keys={trailKeys}
			labels={trailLabels}
			viewBoxWidth={WIDTH}
		/>
	</div>
</section>
