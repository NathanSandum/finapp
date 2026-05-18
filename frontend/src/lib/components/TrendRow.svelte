<script lang="ts">
	import { untrack } from 'svelte';
	import { Tween } from 'svelte/motion';
	import { cubicOut } from 'svelte/easing';
	import { fmt } from '$lib/format';
	import { prefersReducedMotion } from '$lib/utils/motion';
	import {
		computeAnim,
		computePositions,
		pointsToPath,
		type Pt
	} from '$lib/utils/sparkline';
	import Num from './Num.svelte';
	import type { TrendRow } from './TrendList.svelte';

	let {
		row,
		labels,
		stroke,
		txt
	}: {
		row: TrendRow;
		labels: string[];
		stroke: string;
		txt: string;
	} = $props();

	const WIDTH = 200;
	const HEIGHT = 24;

	let prevLabels: string[] = untrack(() => [...labels]);
	let prevPositions: Pt[] = untrack(() => computePositions(row.trail, WIDTH, HEIGHT));

	const tween = new Tween<Pt[]>(untrack(() => [...prevPositions]), {
		duration: 520,
		easing: cubicOut
	});

	$effect(() => {
		const nextValues = row.trail;
		const nextLabels = labels;
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
	const first = $derived(row.trail[0] ?? 0);
	const last = $derived(row.trail[row.trail.length - 1] ?? 0);
	const dir = $derived(last >= first ? '▲' : '▼');
</script>

<tr class="h-8 border-b border-border-faint">
	<td class="text-ink-body">
		<span class="mr-1 font-mono {txt}">{dir}</span>{row.name}
	</td>
	<td>
		<svg viewBox="0 0 {WIDTH} {HEIGHT}" class="h-6 w-full max-w-[200px]">
			<polyline
				fill="none"
				class={stroke}
				stroke-width="1.25"
				stroke-linecap="round"
				stroke-linejoin="round"
				points={path}
			/>
		</svg>
	</td>
	<td class="text-right font-mono font-semibold tabular-nums text-ink"><Num text={fmt(row.amount)} /></td>
</tr>
