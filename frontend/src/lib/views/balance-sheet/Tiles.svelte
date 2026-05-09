<script lang="ts">
	import { fmt, pct } from '../income/lib';

	let {
		totalAssets,
		priorAssets,
		totalLiab,
		priorLiab,
		priorShortLabel
	}: {
		totalAssets: number;
		priorAssets: number;
		totalLiab: number;
		priorLiab: number;
		priorShortLabel: string;
	} = $props();
</script>

<div class="grid grid-cols-2 gap-3">
	<div class="rounded-lg bg-emerald-50/50 p-3">
		<p class="text-[10px] font-medium uppercase tracking-wide text-emerald-700">Assets</p>
		<p class="mt-1 font-mono text-2xl font-semibold tabular-nums text-neutral-900">
			{fmt(totalAssets)}
		</p>
		<p
			class="mt-0.5 font-mono text-[10px] tabular-nums {totalAssets >= priorAssets
				? 'text-emerald-700'
				: 'text-rose-700'}"
		>
			{pct(totalAssets, priorAssets)} vs {priorShortLabel}
		</p>
	</div>
	<div class="rounded-lg bg-rose-50/50 p-3">
		<p class="text-[10px] font-medium uppercase tracking-wide text-rose-700">Liabilities</p>
		<p class="mt-1 font-mono text-2xl font-semibold tabular-nums text-neutral-900">
			{fmt(totalLiab)}
		</p>
		<p
			class="mt-0.5 font-mono text-[10px] tabular-nums {totalLiab <= priorLiab
				? 'text-emerald-700'
				: 'text-rose-700'}"
		>
			{pct(totalLiab, priorLiab)} vs {priorShortLabel}
		</p>
	</div>
</div>
