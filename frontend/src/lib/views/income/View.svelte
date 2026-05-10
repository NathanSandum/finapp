<script lang="ts">
	import { untrack } from 'svelte';
	import MetricTile from '$lib/components/MetricTile.svelte';
	import NetHero from '$lib/components/NetHero.svelte';
	import StatementView from '$lib/components/StatementView.svelte';
	import {
		computeIncomeData,
		computeIncomeTrails,
		computeNetTrail,
		defaultTrailCount,
		latestActivityMonth,
		shiftPeriod,
		snapToGranularity,
		type Granularity
	} from './compute';
	import {
		totalExpenses,
		totalPriorExpenses,
		totalPriorRevenue,
		totalRevenue
	} from './lib';
	import type { IncomeRaw } from './loader';
	import Detail from './variants/Detail.svelte';
	import Summary from './variants/Summary.svelte';
	import Trend from './variants/Trend.svelte';

	let { data }: { data: IncomeRaw } = $props();

	const initial = untrack(() => latestActivityMonth(data.entries, data.accounts));
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

	const income = $derived(
		computeIncomeData(data.entries, data.accounts, selectedYear, selectedMonth, granularity)
	);
	const trail = $derived(
		computeNetTrail(
			data.entries,
			data.accounts,
			selectedYear,
			selectedMonth,
			granularity,
			defaultTrailCount(granularity)
		)
	);
	const trails = $derived(
		computeIncomeTrails(
			data.entries,
			data.accounts,
			selectedYear,
			selectedMonth,
			granularity,
			defaultTrailCount(granularity)
		)
	);

	const revenueTotal = $derived(totalRevenue(income));
	const expenseTotal = $derived(totalExpenses(income));
	const priorRevenueTotal = $derived(totalPriorRevenue(income));
	const priorExpenseTotal = $derived(totalPriorExpenses(income));
	const netIncome = $derived(revenueTotal - expenseTotal);
	const priorNet = $derived(priorRevenueTotal - priorExpenseTotal);
</script>

<StatementView
	title="Income"
	periodLabel={income.period_label}
	{granularity}
	onPrev={() => shift(-1)}
	onNext={() => shift(1)}
	onGranularity={setGranularity}
>
	{#snippet hero()}
		<NetHero
			label="Net Income · {income.period_label}"
			value={netIncome}
			prior={priorNet}
			trail={trail.values}
			trailLabels={trail.labels}
			gradientId="income-net-grad"
		/>
	{/snippet}

	{#snippet tiles()}
		<div class="grid grid-cols-2 gap-3 border-t border-border">
			<MetricTile
				label="Revenue"
				total={revenueTotal}
				prior={priorRevenueTotal}
				priorShortLabel={income.prior_short_label}
				accent="emerald"
			/>
			<MetricTile
				label="Expenses"
				total={expenseTotal}
				prior={priorExpenseTotal}
				priorShortLabel={income.prior_short_label}
				accent="rose"
			/>
		</div>
	{/snippet}

	{#snippet summary()}
		<Summary
			revenue={income.revenue}
			expenses={income.expenses}
			{revenueTotal}
			{expenseTotal}
		/>
	{/snippet}

	{#snippet detail()}
		<Detail
			revenue={income.revenue}
			expenses={income.expenses}
			{revenueTotal}
			{expenseTotal}
			{priorRevenueTotal}
			{priorExpenseTotal}
			priorShortLabel={income.prior_short_label}
		/>
	{/snippet}

	{#snippet trend()}
		<Trend
			revenue={income.revenue}
			expenses={income.expenses}
			revenueTrails={trails.revenueTrails}
			expenseTrails={trails.expenseTrails}
		/>
	{/snippet}
</StatementView>
