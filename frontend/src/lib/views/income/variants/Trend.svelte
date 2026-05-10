<script lang="ts">
	import { buildLine, fmt, type Line } from '../lib';

	let {
		revenue,
		expenses,
		revenueTrails,
		expenseTrails
	}: {
		revenue: Line[];
		expenses: Line[];
		revenueTrails: Map<string, number[]>;
		expenseTrails: Map<string, number[]>;
	} = $props();

	const revenueStroke = 'rgb(16 185 129)';
	const expenseStroke = 'rgb(244 63 94)';
</script>

<div class="space-y-4">
	<section>
		<h4 class="mb-2 text-xs font-semibold uppercase tracking-wide text-neutral-500">Revenue</h4>
		<div class="grid grid-cols-2 gap-3">
			{#each revenue as line}
				{@const trail = revenueTrails.get(line.name) ?? [line.amount]}
				{@const sl = buildLine(trail, 240, 40)}
				<div class="rounded-xl border border-neutral-200 bg-white p-3">
					<div class="flex items-baseline justify-between">
						<span class="text-xs font-medium text-neutral-700">{line.name}</span>
						<span class="font-mono text-sm font-semibold tabular-nums text-neutral-900"
							>{fmt(line.amount)}</span
						>
					</div>
					<svg viewBox="0 0 240 40" class="mt-1 h-10 w-full" preserveAspectRatio="none">
						<polyline fill="none" stroke={revenueStroke} stroke-width="1.5" points={sl.path} />
					</svg>
				</div>
			{/each}
		</div>
	</section>

	<section>
		<h4 class="mb-2 text-xs font-semibold uppercase tracking-wide text-neutral-500">Expenses</h4>
		<div class="grid grid-cols-2 gap-3">
			{#each expenses as line}
				{@const trail = expenseTrails.get(line.name) ?? [line.amount]}
				{@const sl = buildLine(trail, 240, 40)}
				<div class="rounded-xl border border-neutral-200 bg-white p-3">
					<div class="flex items-baseline justify-between">
						<span class="text-xs font-medium text-neutral-700">{line.name}</span>
						<span class="font-mono text-sm font-semibold tabular-nums text-neutral-900"
							>{fmt(line.amount)}</span
						>
					</div>
					<svg viewBox="0 0 240 40" class="mt-1 h-10 w-full" preserveAspectRatio="none">
						<polyline fill="none" stroke={expenseStroke} stroke-width="1.5" points={sl.path} />
					</svg>
				</div>
			{/each}
		</div>
	</section>
</div>
