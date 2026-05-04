<script lang="ts">
	import {
		fmt,
		pct,
		totalRevenue,
		totalExpenses,
		totalPriorRevenue,
		totalPriorExpenses,
		buildLine,
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
	const big = $derived(buildLine(netTrail, 560, 80));
</script>

<div class="flex h-full flex-col gap-3 overflow-hidden">
	<header class="flex shrink-0 items-baseline justify-between">
		<div class="flex items-baseline gap-3">
			<h3 class="text-xl font-semibold text-neutral-900">Income</h3>
			<div class="flex items-center gap-1 text-xs text-neutral-500">
				<button onclick={onPrev} class="rounded px-1.5 py-0.5 hover:bg-neutral-100" aria-label="Previous period">‹</button>
				<span class="min-w-[88px] text-center">{data.period_label}</span>
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

	<section class="relative shrink-0 overflow-hidden rounded-xl border border-neutral-200 bg-white shadow-sm">
		<div class="absolute left-5 top-4 z-10">
			<p class="text-[10px] font-medium uppercase tracking-wide text-neutral-500">Net Income · {data.period_label}</p>
			<p class="mt-0.5 font-mono text-3xl font-semibold tabular-nums text-neutral-900">{fmt(netIncome)}</p>
			<span class="mt-1 inline-block rounded-md bg-emerald-50 px-2 py-0.5 font-mono text-[10px] font-semibold tabular-nums text-emerald-700">{pct(netIncome, priorNet)} vs {data.prior_short_label} · {savingsRate}% saved</span>
		</div>
		<svg width="100%" viewBox="0 0 560 100" class="overflow-visible" preserveAspectRatio="none">
			<defs>
				<linearGradient id="b4d-grad" x1="0" y1="0" x2="0" y2="1">
					<stop offset="0%" stop-color="rgb(16 185 129)" stop-opacity="0.30" />
					<stop offset="100%" stop-color="rgb(16 185 129)" stop-opacity="0" />
				</linearGradient>
			</defs>
			<polygon fill="url(#b4d-grad)" points={big.area} transform="translate(0,20)" />
			<polyline fill="none" stroke="rgb(16 185 129)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" points={big.path} transform="translate(0,20)" />
			<circle cx={big.points[big.points.length - 1].x} cy={big.points[big.points.length - 1].y + 20} r="3" fill="rgb(16 185 129)" />
		</svg>
		<div class="grid px-5 pb-2 text-[9px] font-mono tabular-nums text-neutral-400" style:grid-template-columns="repeat({months.length}, 1fr)">
			{#each months as m}
				<span class="text-center">{m}</span>
			{/each}
		</div>
	</section>

	<div class="grid min-h-0 flex-1 grid-cols-2 gap-3">
		<section class="flex min-h-0 flex-col rounded-xl border border-neutral-200 bg-white p-4">
			<div class="mb-0.5 flex shrink-0 items-baseline justify-between">
				<h4 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Revenue</h4>
				<span class="font-mono text-base font-semibold tabular-nums text-neutral-900">{fmt(revenueTotal)}</span>
			</div>
			<p class="mb-2 shrink-0 font-mono text-[10px] tabular-nums {revenueTotal >= priorRevenueTotal ? 'text-emerald-700' : 'text-rose-700'}">{pct(revenueTotal, priorRevenueTotal)} vs {data.prior_short_label}</p>
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
				<span class="font-mono text-base font-semibold tabular-nums text-neutral-900">{fmt(expenseTotal)}</span>
			</div>
			<p class="mb-2 shrink-0 font-mono text-[10px] tabular-nums {expenseTotal <= priorExpenseTotal ? 'text-emerald-700' : 'text-rose-700'}">{pct(expenseTotal, priorExpenseTotal)} vs {data.prior_short_label}</p>
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
							<span class="w-10 text-right font-mono text-[9px] tabular-nums text-neutral-400">{(fraction * 100).toFixed(0)}% rev</span>
						</div>
					</li>
				{/each}
			</ul>
		</section>
	</div>
</div>
