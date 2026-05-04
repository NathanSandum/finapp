<script lang="ts">
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
	const savingsRate = $derived(revenueTotal > 0 ? ((netIncome / revenueTotal) * 100).toFixed(1) : '0');

	const barW = 28;
	const barGap = 12;
	const barChartH = 70;
	const barMin = $derived(Math.min(...netTrail));
	const barMax = $derived(Math.max(...netTrail));
	const barRange = $derived(barMax - barMin || 1);
</script>

<div class="space-y-6">
	<header class="flex items-baseline justify-between">
		<div class="flex items-baseline gap-3">
			<h3 class="text-2xl font-semibold text-neutral-900">Income</h3>
			<div class="flex items-center gap-1 text-sm text-neutral-500">
				<button onclick={onPrev} class="rounded px-1.5 py-0.5 hover:bg-neutral-100" aria-label="Previous period">‹</button>
				<span class="min-w-[100px] text-center">{data.period_label}</span>
				<button onclick={onNext} class="rounded px-1.5 py-0.5 hover:bg-neutral-100" aria-label="Next period">›</button>
			</div>
		</div>
		<div class="flex gap-1 rounded-full bg-neutral-100 p-1 text-xs">
			<button
				onclick={() => onGranularity('month')}
				class="rounded-full px-3 py-1 {granularity === 'month' ? 'bg-white font-medium shadow-sm' : 'text-neutral-500'}"
			>Month</button>
			<button
				onclick={() => onGranularity('quarter')}
				class="rounded-full px-3 py-1 {granularity === 'quarter' ? 'bg-white font-medium shadow-sm' : 'text-neutral-500'}"
			>Quarter</button>
			<button
				onclick={() => onGranularity('year')}
				class="rounded-full px-3 py-1 {granularity === 'year' ? 'bg-white font-medium shadow-sm' : 'text-neutral-500'}"
			>Year</button>
		</div>
	</header>

	<section class="grid grid-cols-[1fr_auto] items-end gap-8 rounded-2xl border border-neutral-200 bg-white p-7 shadow-sm">
		<div>
			<p class="text-xs font-medium uppercase tracking-wide text-neutral-500">Net Income · {data.period_label}</p>
			<p class="mt-2 font-mono text-5xl font-semibold tabular-nums text-neutral-900">{fmt(netIncome)}</p>
			<div class="mt-3 flex items-center gap-3 text-sm">
				<span class="rounded-md bg-emerald-50 px-2 py-0.5 font-mono text-xs font-semibold tabular-nums text-emerald-700">{pct(netIncome, priorNet)}</span>
				<span class="text-neutral-500">vs {data.prior_label} · {savingsRate}% saved</span>
			</div>
		</div>
		<div>
			<svg width={(barW + barGap) * netTrail.length} height={barChartH + 16} class="overflow-visible">
				{#each netTrail as v, i}
					{@const h = ((v - barMin) / barRange) * (barChartH - 4) + 4}
					{@const x = i * (barW + barGap)}
					{@const y = barChartH - h}
					{@const isCurrent = i === netTrail.length - 1}
					<rect {x} {y} width={barW} height={h} rx="2" fill={isCurrent ? 'rgb(16 185 129)' : 'rgb(229 231 235)'} />
					<text x={x + barW / 2} y={barChartH + 12} text-anchor="middle" class="fill-neutral-400 font-mono" font-size="9">{months[i]}</text>
				{/each}
			</svg>
		</div>
	</section>

	<div class="grid grid-cols-2 gap-4">
		<section class="rounded-xl border border-neutral-200 bg-white p-5">
			<div class="mb-1 flex items-baseline justify-between">
				<h4 class="text-sm font-semibold uppercase tracking-wide text-neutral-500">Revenue</h4>
				<span class="font-mono text-lg font-semibold tabular-nums text-neutral-900">{fmt(revenueTotal)}</span>
			</div>
			<p class="mb-3 font-mono text-xs tabular-nums {revenueTotal >= priorRevenueTotal ? 'text-emerald-700' : 'text-rose-700'}">{pct(revenueTotal, priorRevenueTotal)} vs {data.prior_short_label}</p>
			<ul class="space-y-2">
				{#each data.revenue as line}
					<li class="flex items-baseline justify-between text-sm">
						<span class="text-neutral-700">{line.name}</span>
						<span class="font-mono tabular-nums text-neutral-900">{fmt(line.amount)}</span>
					</li>
				{/each}
			</ul>
		</section>

		<section class="rounded-xl border border-neutral-200 bg-white p-5">
			<div class="mb-1 flex items-baseline justify-between">
				<h4 class="text-sm font-semibold uppercase tracking-wide text-neutral-500">Expenses</h4>
				<span class="font-mono text-lg font-semibold tabular-nums text-neutral-900">{fmt(expenseTotal)}</span>
			</div>
			<p class="mb-3 font-mono text-xs tabular-nums {expenseTotal <= priorExpenseTotal ? 'text-emerald-700' : 'text-rose-700'}">{pct(expenseTotal, priorExpenseTotal)} vs {data.prior_short_label}</p>
			<ul class="space-y-2">
				{#each data.expenses as line}
					{@const fraction = expenseTotal > 0 ? line.amount / expenseTotal : 0}
					<li>
						<div class="flex items-baseline justify-between text-sm">
							<span class="text-neutral-700">{line.name}</span>
							<span class="font-mono tabular-nums text-neutral-900">{fmt(line.amount)}</span>
						</div>
						<div class="mt-1 flex items-center gap-2">
							<div class="h-1 flex-1 overflow-hidden rounded-full bg-neutral-100">
								<div class="h-full bg-rose-400" style:width="{fraction * 100}%"></div>
							</div>
							<span class="w-10 text-right font-mono text-[10px] tabular-nums text-neutral-400">{(fraction * 100).toFixed(0)}% exp</span>
						</div>
					</li>
				{/each}
			</ul>
		</section>
	</div>
</div>
