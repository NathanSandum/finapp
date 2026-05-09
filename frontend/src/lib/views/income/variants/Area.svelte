<script lang="ts">
	import NetHero from '$lib/components/NetHero.svelte';
	import PeriodHeader from '$lib/components/PeriodHeader.svelte';
	import {
		fmt,
		pct,
		totalRevenue,
		totalExpenses,
		totalPriorRevenue,
		totalPriorExpenses,
		type IncomeData
	} from '../lib';
	import type { Granularity } from '../compute';

	let {
		data,
		netTrail,
		months,
		granularity,
		onPrev,
		onNext,
		onGranularity
	}: {
		data: IncomeData;
		netTrail: number[];
		months: string[];
		granularity: Granularity;
		onPrev: () => void;
		onNext: () => void;
		onGranularity: (g: Granularity) => void;
	} = $props();

	const revenueTotal = $derived(totalRevenue(data));
	const expenseTotal = $derived(totalExpenses(data));
	const netIncome = $derived(revenueTotal - expenseTotal);
	const priorRevenueTotal = $derived(totalPriorRevenue(data));
	const priorExpenseTotal = $derived(totalPriorExpenses(data));
	const priorNet = $derived(priorRevenueTotal - priorExpenseTotal);
	const savingsRate = $derived(
		revenueTotal > 0 ? ((netIncome / revenueTotal) * 100).toFixed(1) : '0'
	);
</script>

<div class="flex h-full flex-col gap-3 overflow-hidden">
	<PeriodHeader
		title="Income"
		periodLabel={data.period_label}
		{granularity}
		{onPrev}
		{onNext}
		{onGranularity}
	/>

	<NetHero
		label="Net Income · {data.period_label}"
		value={netIncome}
		prior={priorNet}
		priorShortLabel={data.prior_short_label}
		trail={netTrail}
		trailLabels={months}
		gradientId="income-net-grad"
		badgeSuffix="{savingsRate}% saved"
	/>

	<div class="grid min-h-0 flex-1 grid-cols-2 gap-3">
		<section class="flex min-h-0 flex-col rounded-xl border border-neutral-200 bg-white p-4">
			<div class="mb-0.5 flex shrink-0 items-baseline justify-between">
				<h4 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Revenue</h4>
				<span class="font-mono text-base font-semibold tabular-nums text-neutral-900"
					>{fmt(revenueTotal)}</span
				>
			</div>
			<p
				class="mb-2 shrink-0 font-mono text-[10px] tabular-nums {revenueTotal >= priorRevenueTotal
					? 'text-emerald-700'
					: 'text-rose-700'}"
			>
				{pct(revenueTotal, priorRevenueTotal)} vs {data.prior_short_label}
			</p>
			<ul class="min-h-0 flex-1 space-y-1.5 overflow-hidden">
				{#each data.revenue as line}
					<li class="flex items-baseline justify-between text-xs">
						<span class="text-neutral-700">{line.name}</span>
						<span class="font-mono tabular-nums text-neutral-900">{fmt(line.amount)}</span>
					</li>
				{/each}
			</ul>
		</section>

		<section class="flex min-h-0 flex-col rounded-xl border border-neutral-200 bg-white p-4">
			<div class="mb-0.5 flex shrink-0 items-baseline justify-between">
				<h4 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Expenses</h4>
				<span class="font-mono text-base font-semibold tabular-nums text-neutral-900"
					>{fmt(expenseTotal)}</span
				>
			</div>
			<p
				class="mb-2 shrink-0 font-mono text-[10px] tabular-nums {expenseTotal <= priorExpenseTotal
					? 'text-emerald-700'
					: 'text-rose-700'}"
			>
				{pct(expenseTotal, priorExpenseTotal)} vs {data.prior_short_label}
			</p>
			<ul class="min-h-0 flex-1 space-y-1.5 overflow-hidden">
				{#each data.expenses as line}
					{@const fraction = revenueTotal > 0 ? line.amount / revenueTotal : 0}
					<li>
						<div class="flex items-baseline justify-between text-xs">
							<span class="text-neutral-700">{line.name}</span>
							<span class="font-mono tabular-nums text-neutral-900">{fmt(line.amount)}</span>
						</div>
						<div class="mt-0.5 flex items-center gap-2">
							<div class="h-1 flex-1 overflow-hidden rounded-full bg-neutral-100">
								<div class="h-full bg-rose-400" style:width="{fraction * 100}%"></div>
							</div>
							<span class="w-10 text-right font-mono text-[9px] tabular-nums text-neutral-400"
								>{(fraction * 100).toFixed(0)}% rev</span
							>
						</div>
					</li>
				{/each}
			</ul>
		</section>
	</div>
</div>
