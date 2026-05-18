<script lang="ts">
	import { onDestroy } from 'svelte';
	import { fmt } from '$lib/format';
	import Num from './Num.svelte';
	import Sparkline from './Sparkline.svelte';
	import type { TrendRow } from './TrendList.svelte';
	import { TrailSlots } from './trailSlots.svelte';

	let {
		row,
		labels: _labels,
		keys,
		stroke,
		txt
	}: {
		row: TrendRow;
		labels: string[];
		keys: string[];
		stroke: string;
		txt: string;
	} = $props();

	const WIDTH = 200;
	const HEIGHT = 24;

	const first = $derived(row.trail[0] ?? 0);
	const last = $derived(row.trail[row.trail.length - 1] ?? 0);
	const dir = $derived(last >= first ? '▲' : '▼');

	const trailSlots = new TrailSlots();
	$effect(() => {
		trailSlots.sync(keys, row.trail, WIDTH, HEIGHT);
	});
	onDestroy(() => trailSlots.destroy());
</script>

<tr class="h-8 border-b border-border-faint">
	<td class="text-ink-body">
		<span class="mr-1 font-mono {txt}">{dir}</span>{row.name}
	</td>
	<td>
		<svg viewBox="0 0 {WIDTH} {HEIGHT}" class="h-6 w-full max-w-[200px]">
			<Sparkline
				{trailSlots}
				{keys}
				width={WIDTH}
				height={HEIGHT}
				lineClass={stroke}
			/>
		</svg>
	</td>
	<td class="text-right font-mono font-semibold tabular-nums text-ink">
		<Num text={fmt(row.amount)} />
	</td>
</tr>
