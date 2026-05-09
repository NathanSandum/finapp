<script lang="ts">
	import { fmt, pct } from '../../income/lib';
	import type { Line } from '../compute';

	let {
		assets,
		liabilities,
		totalAssets,
		totalLiab,
		priorAssets,
		priorLiab,
		priorShortLabel
	}: {
		assets: Line[];
		liabilities: Line[];
		totalAssets: number;
		totalLiab: number;
		priorAssets: number;
		priorLiab: number;
		priorShortLabel: string;
	} = $props();
</script>

<div class="grid grid-cols-2 gap-3">
	<div class="rounded-xl border border-neutral-200 bg-white p-4">
		<div class="mb-0.5 flex items-baseline justify-between">
			<h4 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Assets</h4>
			<span class="font-mono text-base font-semibold tabular-nums text-neutral-900"
				>{fmt(totalAssets)}</span
			>
		</div>
		<p
			class="mb-2 font-mono text-[10px] tabular-nums {totalAssets >= priorAssets
				? 'text-emerald-700'
				: 'text-rose-700'}"
		>
			{pct(totalAssets, priorAssets)} vs {priorShortLabel}
		</p>
		<ul class="space-y-1.5">
			{#each assets as line}
				{@const fraction = totalAssets > 0 ? line.amount / totalAssets : 0}
				<li>
					<div class="flex items-baseline justify-between text-xs">
						<span class="text-neutral-700">{line.name}</span>
						<span class="font-mono tabular-nums text-neutral-900">{fmt(line.amount)}</span>
					</div>
					<div class="mt-0.5 flex items-center gap-2">
						<div class="h-1 flex-1 overflow-hidden rounded-full bg-neutral-100">
							<div class="h-full bg-emerald-400" style:width="{fraction * 100}%"></div>
						</div>
						<span class="w-10 text-right font-mono text-[9px] tabular-nums text-neutral-400"
							>{(fraction * 100).toFixed(0)}%</span
						>
					</div>
				</li>
			{/each}
		</ul>
	</div>
	<div class="rounded-xl border border-neutral-200 bg-white p-4">
		<div class="mb-0.5 flex items-baseline justify-between">
			<h4 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Liabilities</h4>
			<span class="font-mono text-base font-semibold tabular-nums text-neutral-900"
				>{fmt(totalLiab)}</span
			>
		</div>
		<p
			class="mb-2 font-mono text-[10px] tabular-nums {totalLiab <= priorLiab
				? 'text-emerald-700'
				: 'text-rose-700'}"
		>
			{pct(totalLiab, priorLiab)} vs {priorShortLabel}
		</p>
		<ul class="space-y-1.5">
			{#each liabilities as line}
				{@const fraction = totalLiab > 0 ? line.amount / totalLiab : 0}
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
							>{(fraction * 100).toFixed(0)}%</span
						>
					</div>
				</li>
			{/each}
		</ul>
	</div>
</div>
