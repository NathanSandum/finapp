<script lang="ts">
	import MetricTile from '$lib/components/MetricTile.svelte';
	import NetHero from '$lib/components/NetHero.svelte';
	import StatementView from '$lib/components/StatementView.svelte';
	import {
		computeIncomeData,
		computeIncomeTrails,
		computeNetTrail,
		defaultTrailCount
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
	import { periodState } from '$lib/views/period-state.svelte';

	let { data }: { data: IncomeRaw } = $props();

	const income = $derived(
		computeIncomeData(
			data.entries,
			data.accounts,
			periodState.year,
			periodState.month,
			periodState.granularity
		)
	);
	const trail = $derived(
		computeNetTrail(
			data.entries,
			data.accounts,
			periodState.year,
			periodState.month,
			periodState.granularity,
			defaultTrailCount(periodState.granularity)
		)
	);
	const trails = $derived(
		computeIncomeTrails(
			data.entries,
			data.accounts,
			periodState.year,
			periodState.month,
			periodState.granularity,
			defaultTrailCount(periodState.granularity)
		)
	);

	const revenueTotal = $derived(totalRevenue(income));
	const expenseTotal = $derived(totalExpenses(income));
	const priorRevenueTotal = $derived(totalPriorRevenue(income));
	const priorExpenseTotal = $derived(totalPriorExpenses(income));
	const netIncome = $derived(revenueTotal - expenseTotal);
	const priorNet = $derived(priorRevenueTotal - priorExpenseTotal);
</script>

<StatementView>
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
		<div class="mx-auto h-px w-[95%] bg-border"></div>
		<div class="grid grid-cols-2 gap-3">
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
