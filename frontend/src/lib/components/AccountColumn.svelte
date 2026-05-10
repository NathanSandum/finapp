<script lang="ts">
	import { fmt, pct } from '$lib/views/income/lib';

	type Line = { name: string; amount: number };

	let {
		title,
		lines,
		total,
		prior,
		priorShortLabel,
		accent,
		direction
	}: {
		title: string;
		lines: Line[];
		total: number;
		prior: number;
		priorShortLabel: string;
		accent: 'emerald' | 'rose';
		direction: 'higher' | 'lower';
	} = $props();

	const positive = $derived(direction === 'higher' ? total >= prior : total <= prior);
	const barClass = $derived(accent === 'rose' ? 'bg-negative-bar-2' : 'bg-positive-bar-2');
</script>

<div class="rounded-xl border border-border bg-surface-card p-4">
	<div class="mb-0.5 flex items-baseline justify-between">
		<h4 class="text-xs font-semibold uppercase tracking-wide text-ink-muted">{title}</h4>
		<span class="font-mono text-base font-semibold tabular-nums text-ink">{fmt(total)}</span>
	</div>
	<p
		class="mb-2 font-mono text-[10px] tabular-nums {positive
			? 'text-positive'
			: 'text-negative'}"
	>
		{pct(total, prior)} vs {priorShortLabel}
	</p>
	<ul class="space-y-1.5">
		{#each lines as line}
			{@const fraction = total > 0 ? line.amount / total : 0}
			<li>
				<div class="flex items-baseline justify-between text-xs">
					<span class="text-ink-body">{line.name}</span>
					<span class="font-mono tabular-nums text-ink">{fmt(line.amount)}</span>
				</div>
				<div class="mt-0.5 flex items-center gap-2">
					<div class="h-1 flex-1 overflow-hidden rounded-full bg-surface-sunken">
						<div class="h-full {barClass}" style:width="{fraction * 100}%"></div>
					</div>
					<span class="w-10 text-right font-mono text-[9px] tabular-nums text-ink-faint"
						>{(fraction * 100).toFixed(0)}%</span
					>
				</div>
			</li>
		{/each}
	</ul>
</div>
