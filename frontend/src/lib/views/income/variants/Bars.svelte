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
		netTrail
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

	const sixMo = netTrail.slice(-6);
	const sixMoLabels = months.slice(-6);
	const barW = 28;
	const barGap = 12;
	const barChartH = 70;
	const barMin = Math.min(...sixMo);
	const barMax = Math.max(...sixMo);
	const barRange = barMax - barMin || 1;
</script>

<div class="space-y-6">
	<header class="flex items-baseline justify-between">
		<div>
			<h3 class="text-2xl font-semibold text-neutral-900">Income</h3>
			<p class="text-sm text-neutral-500">April 2026</p>
		</div>
		<div class="flex gap-1 rounded-full bg-neutral-100 p-1 text-xs">
			<button class="rounded-full bg-white px-3 py-1 font-medium shadow-sm">Month</button>
			<button class="px-3 py-1 text-neutral-500">Quarter</button>
			<button class="px-3 py-1 text-neutral-500">Year</button>
		</div>
	</header>

	<section class="grid grid-cols-[1fr_auto] items-end gap-8 rounded-2xl border border-neutral-200 bg-white p-7 shadow-sm">
		<div>
			<p class="text-xs font-medium uppercase tracking-wide text-neutral-500">Net Income · April 2026</p>
			<p class="mt-2 font-mono text-5xl font-semibold tabular-nums text-neutral-900">{fmt(netIncome)}</p>
			<div class="mt-3 flex items-center gap-3 text-sm">
				<span class="rounded-md bg-emerald-50 px-2 py-0.5 font-mono text-xs font-semibold tabular-nums text-emerald-700">{pct(netIncome, priorNet)}</span>
				<span class="text-neutral-500">vs March · {savingsRate}% saved</span>
			</div>
		</div>
		<div>
			<svg width={(barW + barGap) * sixMo.length} height={barChartH + 16} class="overflow-visible">
				{#each sixMo as v, i}
					{@const h = ((v - barMin) / barRange) * (barChartH - 4) + 4}
					{@const x = i * (barW + barGap)}
					{@const y = barChartH - h}
					{@const isCurrent = i === sixMo.length - 1}
					<rect {x} {y} width={barW} height={h} rx="2" fill={isCurrent ? 'rgb(16 185 129)' : 'rgb(229 231 235)'} />
					<text x={x + barW / 2} y={barChartH + 12} text-anchor="middle" class="fill-neutral-400 font-mono" font-size="9">{sixMoLabels[i]}</text>
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
			<p class="mb-3 font-mono text-xs tabular-nums {revenueTotal >= priorRevenueTotal ? 'text-emerald-700' : 'text-rose-700'}">{pct(revenueTotal, priorRevenueTotal)} vs Mar</p>
			<ul class="space-y-2">
				{#each sortedRevenue as line}
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
			<p class="mb-3 font-mono text-xs tabular-nums {expenseTotal <= priorExpenseTotal ? 'text-emerald-700' : 'text-rose-700'}">{pct(expenseTotal, priorExpenseTotal)} vs Mar</p>
			<ul class="space-y-2">
				{#each sortedExpenses as line}
					{@const fraction = line.amount / expenseTotal}
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
