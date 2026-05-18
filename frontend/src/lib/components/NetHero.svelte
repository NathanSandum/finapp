<script lang="ts">
	import { untrack } from 'svelte';
	import { Tween } from 'svelte/motion';
	import { cubicOut } from 'svelte/easing';
	import { fmt, pct, shortFmt } from '$lib/format';
	import { prefersReducedMotion } from '$lib/utils/motion';
	import {
		computeAnim,
		computePositions,
		pointsToArea,
		pointsToPath,
		type Pt
	} from '$lib/utils/sparkline';
	import Num from './Num.svelte';

	let {
		label,
		value,
		prior,
		trail,
		trailLabels,
		gradientId
	}: {
		label: string;
		value: number;
		prior: number;
		trail: number[];
		trailLabels: string[];
		gradientId: string;
	} = $props();

	const WIDTH = 560;
	const HEIGHT = 80;

	let prevLabels: string[] = untrack(() => [...trailLabels]);
	let prevPositions: Pt[] = untrack(() => computePositions(trail, WIDTH, HEIGHT));

	const tween = new Tween<Pt[]>(untrack(() => [...prevPositions]), {
		duration: 520,
		easing: cubicOut
	});

	$effect(() => {
		const nextValues = trail;
		const nextLabels = trailLabels;
		const nextPositions = computePositions(nextValues, WIDTH, HEIGHT);
		const sameLabels =
			nextLabels.length === prevLabels.length &&
			nextLabels.every((l, i) => l === prevLabels[i]);
		const sameValues =
			sameLabels && nextPositions.every((p, i) => p.x === prevPositions[i].x && p.y === prevPositions[i].y);
		if (sameValues) return;
		if (prefersReducedMotion()) {
			tween.set(nextPositions, { duration: 0 });
		} else {
			const { start, end } = computeAnim(
				prevLabels,
				prevPositions,
				nextLabels,
				nextPositions,
				WIDTH
			);
			tween.set(start, { duration: 0 });
			tween.target = end;
		}
		prevLabels = [...nextLabels];
		prevPositions = nextPositions;
	});

	const path = $derived(pointsToPath(tween.current));
	const area = $derived(pointsToArea(tween.current, WIDTH, HEIGHT));
	const lastPoint = $derived(tween.current[tween.current.length - 1]);
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
		<defs>
			<linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
				<stop offset="0%" style:stop-color="var(--color-positive-line)" stop-opacity="0.30" />
				<stop offset="100%" style:stop-color="var(--color-positive-line)" stop-opacity="0" />
			</linearGradient>
		</defs>
		<polygon fill="url(#{gradientId})" points={area} transform="translate(0,20)" />
		<polyline
			fill="none"
			class="stroke-positive-line"
			stroke-width="1.25"
			stroke-linecap="round"
			stroke-linejoin="round"
			points={path}
			transform="translate(0,20)"
		/>
		{#if lastPoint}
			<circle cx={lastPoint.x} cy={lastPoint.y + 20} r="3" class="fill-positive-line" />
		{/if}
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
