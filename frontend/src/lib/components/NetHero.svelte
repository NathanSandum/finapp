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

	const WIDTH = 720;
	const HEIGHT = 100;
	const Y_OFFSET = 16;

	const domain = $derived(valueDomain(trail));
	const yOf = (v: number): number => {
		const range = domain.max - domain.min;
		if (range === 0) return HEIGHT / 2;
		return HEIGHT - ((v - domain.min) / range) * HEIGHT;
	};
	const axisTicks = $derived.by(() => {
		if (domain.min === domain.max) {
			return [{ value: domain.max, y: HEIGHT / 2 + Y_OFFSET }];
		}
		const out: { value: number; y: number }[] = [];
		out.push({ value: domain.max, y: yOf(domain.max) + Y_OFFSET });
		if (domain.min < 0 && domain.max > 0) {
			out.push({ value: 0, y: yOf(0) + Y_OFFSET });
		}
		out.push({ value: domain.min, y: yOf(domain.min) + Y_OFFSET });
		return out;
	});

	const trailSlots = new TrailSlots();
	$effect(() => {
		trailSlots.sync(trailKeys, trail, WIDTH, HEIGHT);
	});
	onDestroy(() => trailSlots.destroy());

	const deltaClass = $derived(value >= prior ? 'text-positive' : 'text-negative');
</script>

<section class="relative shrink-0 border-t-2 border-ink pt-4">
	<div class="flex items-start justify-between gap-8">
		<div>
			<p class="text-[10px] font-medium uppercase tracking-[0.22em] text-ink-faint">{label}</p>
			<p class="mt-2 flex items-baseline gap-3">
				<span class="font-display text-5xl font-medium tabular-nums leading-none text-ink">
					<Num text={fmt(value)} />
				</span>
				<span class="font-mono text-sm tabular-nums {deltaClass}">{pct(value, prior)}</span>
			</p>
		</div>
		<div class="self-end pb-1 text-right text-[10px] uppercase tracking-[0.18em] text-ink-faint">
			Trend · 12 mo
		</div>
	</div>

	<svg width="100%" viewBox="0 0 {WIDTH} {HEIGHT + Y_OFFSET + 12}" class="mt-3 overflow-visible" preserveAspectRatio="none">
		<Sparkline
			{trailSlots}
			keys={trailKeys}
			width={WIDTH}
			height={HEIGHT}
			lineClass="stroke-ink"
			strokeWidth={1}
			showArea
			{gradientId}
			showMarker
			markerClass="fill-ink"
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
	<div class="pt-1 font-mono text-[9px] tabular-nums text-ink-faint">
		<AnimatedLabels {trailSlots} keys={trailKeys} labels={trailLabels} viewBoxWidth={WIDTH} />
	</div>
</section>
