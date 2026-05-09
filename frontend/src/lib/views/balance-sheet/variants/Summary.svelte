<script lang="ts">
	import { fmt } from '../../income/lib';
	import type { Line } from '../compute';

	let {
		assets,
		liabilities,
		totalAssets,
		totalLiab
	}: {
		assets: Line[];
		liabilities: Line[];
		totalAssets: number;
		totalLiab: number;
	} = $props();

	const assetColors = ['bg-emerald-500', 'bg-emerald-400', 'bg-emerald-300', 'bg-emerald-200'];
	const liabColors = ['bg-rose-500', 'bg-rose-400', 'bg-rose-300'];
</script>

<div class="space-y-4">
	<div>
		<div class="mb-1.5 flex items-baseline justify-between">
			<span class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Assets</span>
			<span class="font-mono text-xs tabular-nums text-neutral-500">{fmt(totalAssets)}</span>
		</div>
		<div class="flex h-7 w-full overflow-hidden rounded-md bg-neutral-100">
			{#each assets as line, i}
				{@const fraction = totalAssets > 0 ? line.amount / totalAssets : 0}
				<div
					class="{assetColors[
						i % assetColors.length
					]} flex items-center justify-center text-[10px] font-medium text-white"
					style:width="{fraction * 100}%"
				>
					{#if fraction > 0.08}{line.name}{/if}
				</div>
			{/each}
		</div>
		<ul class="mt-2 grid grid-cols-2 gap-x-6 gap-y-1 text-xs sm:grid-cols-4">
			{#each assets as line, i}
				<li class="flex items-baseline gap-2">
					<span class="h-2 w-2 rounded-sm {assetColors[i % assetColors.length]}"></span>
					<span class="text-neutral-700">{line.name}</span>
					<span class="ml-auto font-mono tabular-nums text-neutral-900">{fmt(line.amount)}</span>
				</li>
			{/each}
		</ul>
	</div>
	<div>
		<div class="mb-1.5 flex items-baseline justify-between">
			<span class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Liabilities</span
			>
			<span class="font-mono text-xs tabular-nums text-neutral-500">{fmt(totalLiab)}</span>
		</div>
		<div class="flex h-7 w-full overflow-hidden rounded-md bg-neutral-100">
			{#each liabilities as line, i}
				{@const fraction = totalAssets > 0 ? line.amount / totalAssets : 0}
				<div
					class="{liabColors[
						i % liabColors.length
					]} flex items-center justify-center text-[10px] font-medium text-white"
					style:width="{fraction * 100}%"
				>
					{#if fraction > 0.04}{line.name}{/if}
				</div>
			{/each}
		</div>
		<p class="mt-1 text-[10px] text-neutral-400">Width relative to total assets</p>
	</div>
</div>
