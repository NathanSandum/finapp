<script lang="ts">
	import RankedRows, { type RankedRow } from '$lib/components/RankedRows.svelte';
	import { fmt, pct } from '$lib/format';
	import Num from './Num.svelte';

	type Line = { name: string; amount: number };

	let {
		positive,
		negative,
		net,
		priorNet,
		priorShortLabel,
		netLabel
	}: {
		positive: Line[];
		negative: Line[];
		net: number;
		priorNet: number;
		priorShortLabel: string;
		netLabel: string;
	} = $props();

	const ranked = $derived<RankedRow[]>(
		[
			...positive.map((l) => ({
				name: l.name,
				amount: l.amount,
				dot: 'bg-positive',
				bar: 'bg-positive-line'
			})),
			...negative.map((l) => ({
				name: l.name,
				amount: l.amount,
				dot: 'bg-negative',
				bar: 'bg-negative-line'
			}))
		].sort((a, b) => b.amount - a.amount)
	);
</script>

<div>
	<div class="flex h-8 items-center justify-between border-b border-border">
		<p class="text-[9px] font-semibold uppercase tracking-[0.1em] text-ink-muted">{netLabel}</p>
		<p class="font-mono text-sm font-semibold tabular-nums {net >= 0 ? 'text-positive' : 'text-negative'}">
			<Num text={fmt(net)} />
			<span class="text-[9px] text-ink-faint">{pct(net, priorNet)} vs {priorShortLabel}</span>
		</p>
	</div>
	<RankedRows rows={ranked} />
</div>
