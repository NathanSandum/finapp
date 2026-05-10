<script lang="ts">
	import { fmt, pct } from '$lib/views/income/lib';
	import { computeOverview } from './compute';
	import type { OverviewRaw } from './loader';
	import Ratios from './variants/Ratios.svelte';
	import { entryIntent } from '$lib/views/entries/intent.svelte';
	import { tabState } from '$lib/tabs/state.svelte';

	let { data }: { data: OverviewRaw } = $props();

	const metrics = $derived(computeOverview(data.entries, data.accounts));

	function newEntry() {
		entryIntent.set({ verb: 'spent', sourceTab: 'overview' });
		tabState.set('entries');
	}
</script>

<div class="flex h-full flex-col gap-3 overflow-hidden">
	<header class="flex shrink-0 items-baseline justify-between gap-3">
		<div class="flex items-baseline gap-3">
			<h3 class="text-xl font-semibold text-ink">Overview</h3>
			<span class="font-mono text-xs uppercase tracking-wider text-ink-muted">
				as of {metrics.asOfLabel}
			</span>
		</div>
		<button
			type="button"
			onclick={newEntry}
			class="bg-surface-inverse px-3 py-1.5 text-xs font-medium uppercase tracking-[0.08em] text-ink-inverse transition-colors hover:bg-surface-inverse-hover"
		>
			+ New
		</button>
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
