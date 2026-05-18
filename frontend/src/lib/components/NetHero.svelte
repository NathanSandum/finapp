<script lang="ts">
	import { onDestroy } from 'svelte';
	import { fmt, pct, shortFmt } from '$lib/format';
	import AnimatedLabels from './AnimatedLabels.svelte';
	import Num from './Num.svelte';
	import Sparkline from './Sparkline.svelte';
	import { TrailSlots, valueDomain } from './trailSlots.svelte';

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
	const Y_OFFSET = 20;

	const domain = $derived(valueDomain(trail));
	const yOf = (v: number): number => {
		const range = domain.max - domain.min || 1;
		return HEIGHT - ((v - domain.min) / range) * HEIGHT;
	};
	const axisTicks = $derived.by(() => {
		const out: { value: number; y: number }[] = [];
		out.push({ value: domain.max, y: yOf(domain.max) + Y_OFFSET });
		if (domain.min < 0 && domain.max > 0) {
			out.push({ value: 0, y: yOf(0) + Y_OFFSET });
		}
		if (domain.min !== domain.max) {
			out.push({ value: domain.min, y: yOf(domain.min) + Y_OFFSET });
		}
		return out;
	});

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
			yOffset={Y_OFFSET}
		/>
		{#each axisTicks as tick (tick.value)}
			<text
				x={WIDTH - 4}
				y={tick.y}
				text-anchor="end"
				dominant-baseline="middle"
				class="fill-ink-faint"
				font-family="var(--font-mono)"
				font-size="9">{shortFmt(tick.value)}</text
			>
		{/each}
	</svg>
	<div class="pb-2 font-mono text-[9px] tabular-nums text-ink-faint">
		<AnimatedLabels
			{trailSlots}
			keys={trailKeys}
			labels={trailLabels}
			viewBoxWidth={WIDTH}
		/>
	</div>
</section>
