<script lang="ts">
	import { fmt, pct } from '$lib/views/income/lib';
	import { computeOverview } from './compute';
	import type { OverviewRaw } from './loader';
	import Ratios from './variants/Ratios.svelte';

	let { data }: { data: OverviewRaw } = $props();

	const metrics = $derived(computeOverview(data.entries, data.accounts));
</script>

<div class="px-8 py-8">
	<header class="mb-8 flex items-baseline justify-between">
		<h1 class="font-display text-2xl font-semibold text-ink">Overview</h1>
		<p class="font-mono text-[11px] uppercase tracking-wider text-ink-muted">
			as of {metrics.asOfLabel}
		</p>
	</header>

	<section class="mb-12 grid grid-cols-2 gap-12">
		<div>
			<p class="text-[10px] font-medium uppercase tracking-[0.1em] text-ink-muted">Net Worth</p>
			<p class="mt-2 font-mono text-5xl font-semibold tabular-nums text-ink">
				{fmt(metrics.netWorth)}
			</p>
			<p class="mt-1 font-mono text-xs tabular-nums text-positive">
				{pct(metrics.netWorth, metrics.priorNetWorth)} vs last month
			</p>
		</div>
		<div>
			<p class="text-[10px] font-medium uppercase tracking-[0.1em] text-ink-muted">Income · 12 mo</p>
			<p class="mt-2 font-mono text-5xl font-semibold tabular-nums text-ink">
				{fmt(metrics.income12mo)}
			</p>
			<p class="mt-1 font-mono text-xs tabular-nums text-positive">
				{pct(metrics.income12mo, metrics.priorIncome12mo)} vs prior 12 mo
			</p>
		</div>
	</section>

	<Ratios {metrics} />
</div>
