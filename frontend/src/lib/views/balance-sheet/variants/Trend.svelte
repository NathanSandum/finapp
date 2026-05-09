<script lang="ts">
	import { buildLine, fmt } from '../../income/lib';
	import type { Line } from '../compute';

	let {
		assets,
		trails
	}: {
		assets: Line[];
		trails: Map<string, number[]>;
	} = $props();
</script>

<div class="grid grid-cols-2 gap-3">
	{#each assets as line}
		{@const trail = trails.get(line.name) ?? [line.amount]}
		{@const sl = buildLine(trail, 240, 40)}
		<div class="rounded-xl border border-neutral-200 bg-white p-3">
			<div class="flex items-baseline justify-between">
				<span class="text-xs font-medium text-neutral-700">{line.name}</span>
				<span class="font-mono text-sm font-semibold tabular-nums text-neutral-900"
					>{fmt(line.amount)}</span
				>
			</div>
			<svg viewBox="0 0 240 40" class="mt-1 h-10 w-full" preserveAspectRatio="none">
				<polyline fill="none" stroke="rgb(16 185 129)" stroke-width="1.5" points={sl.path} />
			</svg>
		</div>
	{/each}
</div>
