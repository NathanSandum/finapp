<script lang="ts">
	import { fmt } from '$lib/views/income/lib';

	type Line = { name: string; amount: number };
	type Side = { lines: Line[]; total: number; label: string };

	let {
		positive,
		negative,
		netLabel,
		positivePalette = ['bg-positive-bar-1', 'bg-positive-bar-2', 'bg-positive-bar-3', 'bg-positive-bar-4'],
		negativePalette = ['bg-negative-bar-1', 'bg-negative-bar-2', 'bg-negative-bar-3', 'bg-negative-bar-4']
	}: {
		positive: Side;
		negative: Side;
		netLabel: string;
		positivePalette?: string[];
		negativePalette?: string[];
	} = $props();

	const net = $derived(positive.total - negative.total);
	const max = $derived(Math.max(positive.total, 1));
	const negativeFrac = $derived(negative.total / max);
	const netFrac = $derived(Math.max(0, net) / max);
</script>

<div class="space-y-3">
	<div class="flex h-7 w-full overflow-hidden">
		{#each positive.lines as line, i}
			{@const f = positive.total > 0 ? line.amount / positive.total : 0}
			<div
				class="{positivePalette[i % positivePalette.length]} flex items-center justify-center text-[9px] font-medium text-white"
				style:width="{f * 100}%"
			>
				{#if f > 0.1}{line.name}{/if}
			</div>
		{/each}
	</div>
	<div class="flex items-center gap-2 text-[10px]">
		<span class="font-semibold uppercase tracking-[0.1em] text-positive">{positive.label}</span>
		<div class="flex-1 border-t border-dashed border-border"></div>
		<span class="font-mono tabular-nums text-ink">{fmt(positive.total)}</span>
	</div>

	<div class="relative h-7 w-full bg-surface-sunken">
		<div class="absolute inset-y-0 left-0 flex" style:width="{negativeFrac * 100}%">
			{#each negative.lines as line, i}
				{@const f = negative.total > 0 ? line.amount / negative.total : 0}
				<div
					class="{negativePalette[i % negativePalette.length]} flex items-center justify-center text-[9px] font-medium text-white"
					style:width="{f * 100}%"
				>
					{#if f > 0.15}{line.name}{/if}
				</div>
			{/each}
		</div>
		<div
			class="absolute inset-y-0 bg-positive-line/40"
			style:left="{negativeFrac * 100}%"
			style:width="{netFrac * 100}%"
		></div>
	</div>
	<div class="flex items-center gap-2 text-[10px]">
		<span class="font-semibold uppercase tracking-[0.1em] text-negative">{negative.label}</span>
		<div class="flex-1 border-t border-dashed border-border"></div>
		<span class="font-mono tabular-nums text-negative">−{fmt(negative.total)}</span>
	</div>

	<div class="flex items-baseline justify-between border-t border-border pt-2">
		<span class="text-[10px] font-semibold uppercase tracking-[0.12em] text-ink-muted">{netLabel}</span>
		<span class="font-mono text-lg font-semibold tabular-nums {net >= 0 ? 'text-positive' : 'text-negative'}">
			{fmt(net)}
		</span>
	</div>
</div>
