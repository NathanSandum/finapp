<script lang="ts">
	import { fmt, pct } from '$lib/views/income/lib';

	type Line = { name: string; amount: number };
	type Accent = 'positive' | 'negative';

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

	type Row = { name: string; amount: number; accent: Accent };
	const ranked = $derived<Row[]>(
		[
			...positive.map((l) => ({ name: l.name, amount: l.amount, accent: 'positive' as const })),
			...negative.map((l) => ({ name: l.name, amount: l.amount, accent: 'negative' as const }))
		].sort((a, b) => b.amount - a.amount)
	);
	const max = $derived(Math.max(...ranked.map((r) => r.amount), 1));
</script>

<div>
	<div class="flex h-8 items-center justify-between border-b border-border">
		<p class="text-[9px] font-semibold uppercase tracking-[0.1em] text-ink-muted">{netLabel}</p>
		<p class="font-mono text-sm font-semibold tabular-nums {net >= 0 ? 'text-positive' : 'text-negative'}">
			{fmt(net)}
			<span class="text-[9px] text-ink-faint">{pct(net, priorNet)} vs {priorShortLabel}</span>
		</p>
	</div>
	<ul>
		{#each ranked as row}
			{@const f = row.amount / max}
			{@const bar = row.accent === 'positive' ? 'bg-positive-line' : 'bg-negative-line'}
			{@const dot = row.accent === 'positive' ? 'bg-positive' : 'bg-negative'}
			<li class="grid h-8 grid-cols-[8px_1fr_auto] items-center gap-2 border-b border-border-faint">
				<span class="h-1.5 w-1.5 {dot}"></span>
				<div class="min-w-0">
					<div class="flex items-baseline justify-between text-[11px]">
						<span class="truncate text-ink-body">{row.name}</span>
					</div>
					<div class="mt-0.5 h-[2px] w-full bg-surface-sunken">
						<div class="h-full {bar}" style:width="{f * 100}%"></div>
					</div>
				</div>
				<span class="font-mono text-[11px] tabular-nums text-ink">{fmt(row.amount)}</span>
			</li>
		{/each}
	</ul>
</div>
