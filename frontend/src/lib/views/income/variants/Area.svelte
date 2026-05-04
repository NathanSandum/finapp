<script lang="ts">
	import {
		dummy,
		fmt,
		pct,
		totalRevenue,
		totalExpenses,
		totalPriorRevenue,
		totalPriorExpenses,
		months,
		netTrail,
		buildLine
	} from './dummy';

	const revenueTotal = totalRevenue(dummy);
	const expenseTotal = totalExpenses(dummy);
	const netIncome = revenueTotal - expenseTotal;
	const priorRevenueTotal = totalPriorRevenue(dummy);
	const priorExpenseTotal = totalPriorExpenses(dummy);
	const priorNet = priorRevenueTotal - priorExpenseTotal;
	const savingsRate = revenueTotal > 0 ? ((netIncome / revenueTotal) * 100).toFixed(1) : '0';

	const sortedRevenue = [...dummy.revenue].sort((a, b) => b.amount - a.amount);
	const sortedExpenses = [...dummy.expenses].sort((a, b) => b.amount - a.amount);

	const big = buildLine(netTrail, 560, 80);
</script>

<div class="flex h-full flex-col gap-3 overflow-hidden">
	<header class="flex shrink-0 items-baseline justify-between">
		<div>
			<h3 class="text-xl font-semibold text-neutral-900">Income</h3>
			<p class="text-xs text-neutral-500">April 2026</p>
		</div>
		<div class="flex gap-1 rounded-full bg-neutral-100 p-1 text-xs">
			<button class="rounded-full bg-white px-3 py-1 font-medium shadow-sm">Month</button>
			<button class="px-3 py-1 text-neutral-500">Quarter</button>
			<button class="px-3 py-1 text-neutral-500">Year</button>
		</div>
	</header>

	<section class="relative shrink-0 overflow-hidden rounded-xl border border-neutral-200 bg-white shadow-sm">
		<div class="absolute left-5 top-4 z-10">
			<p class="text-[10px] font-medium uppercase tracking-wide text-neutral-500">Net Income · April 2026</p>
			<p class="mt-0.5 font-mono text-3xl font-semibold tabular-nums text-neutral-900">{fmt(netIncome)}</p>
			<span class="mt-1 inline-block rounded-md bg-emerald-50 px-2 py-0.5 font-mono text-[10px] font-semibold tabular-nums text-emerald-700">{pct(netIncome, priorNet)} vs Mar · {savingsRate}% saved</span>
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
		<div class="grid grid-cols-12 px-5 pb-2 text-[9px] font-mono tabular-nums text-neutral-400">
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
			<p class="mb-2 shrink-0 font-mono text-[10px] tabular-nums {revenueTotal >= priorRevenueTotal ? 'text-emerald-700' : 'text-rose-700'}">{pct(revenueTotal, priorRevenueTotal)} vs Mar</p>
			<ul class="min-h-0 flex-1 space-y-1.5 overflow-hidden">
				{#each sortedRevenue as line}
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
			<p class="mb-2 shrink-0 font-mono text-[10px] tabular-nums {expenseTotal <= priorExpenseTotal ? 'text-emerald-700' : 'text-rose-700'}">{pct(expenseTotal, priorExpenseTotal)} vs Mar</p>
			<ul class="min-h-0 flex-1 space-y-1.5 overflow-hidden">
				{#each sortedExpenses as line}
					{@const fraction = line.amount / revenueTotal}
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
