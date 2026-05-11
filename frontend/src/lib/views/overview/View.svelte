<script lang="ts">
	import { untrack } from 'svelte';
	import PeriodHeader from '$lib/components/PeriodHeader.svelte';
	import { fmt, pct } from '$lib/views/income/lib';
	import {
		latestActivityPeriod,
		periodLabel,
		shiftPeriod,
		snapToGranularity,
		type Granularity
	} from '$lib/views/period';
	import { computeOverview } from './compute';
	import type { OverviewRaw } from './loader';
	import Ratios from './variants/Ratios.svelte';
	import { entryIntent } from '$lib/views/entries/intent.svelte';
	import { tabState } from '$lib/tabs/state.svelte';

	let { data }: { data: OverviewRaw } = $props();

	const initial = untrack(() => latestActivityPeriod(data.entries));
	let selectedYear = $state(initial.year);
	let selectedMonth = $state(initial.month);
	let granularity = $state<Granularity>('month');

	function shift(direction: 1 | -1) {
		const next = shiftPeriod(selectedYear, selectedMonth, granularity, direction);
		selectedYear = next.year;
		selectedMonth = next.month;
	}

	function setGranularity(next: Granularity) {
		const snapped = snapToGranularity(selectedYear, selectedMonth, next);
		selectedYear = snapped.year;
		selectedMonth = snapped.month;
		granularity = next;
	}

	const metrics = $derived(
		computeOverview(data.entries, data.accounts, data.anchors, selectedYear, selectedMonth, granularity)
	);

	function newEntry() {
		entryIntent.set({ verb: 'spent', sourceTab: 'overview' });
		tabState.set('entries');
	}
</script>

<div class="flex h-full flex-col gap-3 overflow-hidden">
	<PeriodHeader
		title="Overview"
		periodLabel={periodLabel(selectedYear, selectedMonth, granularity)}
		{granularity}
		onPrev={() => shift(-1)}
		onNext={() => shift(1)}
		onGranularity={setGranularity}
	>
		{#snippet actions()}
			<button
				type="button"
				onclick={newEntry}
				class="bg-surface-inverse px-3 py-1.5 text-xs font-medium uppercase tracking-[0.08em] text-ink-inverse transition-colors hover:bg-surface-inverse-hover"
			>
				+ New
			</button>
		{/snippet}
	</PeriodHeader>

	<section class="grid grid-cols-2 gap-12">
		<div>
			<p class="text-[10px] font-medium uppercase tracking-[0.1em] text-ink-muted">Net Worth</p>
			<p class="mt-2 font-mono text-5xl font-semibold tabular-nums text-ink">
				{fmt(metrics.netWorth)}
			</p>
			<p class="mt-1 font-mono text-xs tabular-nums text-positive">
				{pct(metrics.netWorth, metrics.priorNetWorth)} vs prior
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
