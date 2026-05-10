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
</script>

<div class="space-y-4">
	<section>
		<h4 class="mb-2 text-xs font-semibold uppercase tracking-wide text-ink-muted">Revenue</h4>
		<div class="grid grid-cols-2 gap-3">
			{#each revenue as line}
				{@const trail = revenueTrails.get(line.name) ?? [line.amount]}
				{@const sl = buildLine(trail, 240, 40)}
				<div class="rounded-xl border border-border bg-surface-card p-3">
					<div class="flex items-baseline justify-between">
						<span class="text-xs font-medium text-ink-body">{line.name}</span>
						<span class="font-mono text-sm font-semibold tabular-nums text-ink"
							>{fmt(line.amount)}</span
						>
					</div>
					<svg viewBox="0 0 240 40" class="mt-1 h-10 w-full" preserveAspectRatio="none">
						<polyline fill="none" class="stroke-positive-line" stroke-width="1.5" points={sl.path} />
					</svg>
				</div>
			{/each}
		</div>
	</section>

	<section>
		<h4 class="mb-2 text-xs font-semibold uppercase tracking-wide text-ink-muted">Expenses</h4>
		<div class="grid grid-cols-2 gap-3">
			{#each expenses as line}
				{@const trail = expenseTrails.get(line.name) ?? [line.amount]}
				{@const sl = buildLine(trail, 240, 40)}
				<div class="rounded-xl border border-border bg-surface-card p-3">
					<div class="flex items-baseline justify-between">
						<span class="text-xs font-medium text-ink-body">{line.name}</span>
						<span class="font-mono text-sm font-semibold tabular-nums text-ink"
							>{fmt(line.amount)}</span
						>
					</div>
					<svg viewBox="0 0 240 40" class="mt-1 h-10 w-full" preserveAspectRatio="none">
						<polyline fill="none" class="stroke-negative-line" stroke-width="1.5" points={sl.path} />
					</svg>
				</div>
			{/each}
		</div>
	</section>
</div>
