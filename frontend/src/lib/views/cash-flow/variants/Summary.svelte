<script lang="ts">
	import { fmt } from '../../income/lib';
	import type { CashFlowSection } from '../compute';

	let {
		operating,
		investing,
		financing,
		openingCash,
		closingCash
	}: {
		operating: CashFlowSection;
		investing: CashFlowSection;
		financing: CashFlowSection;
		openingCash: number;
		closingCash: number;
	} = $props();

	type Row = {
		label: string;
		total: number;
		bar: string;
		verbPositive: string;
		verbNegative: string;
	};

	const rows = $derived<Row[]>([
		{
			label: 'Income',
			total: operating.total,
			bar: 'bg-positive-bar-2',
			verbPositive: 'Earned',
			verbNegative: 'Spent'
		},
		{
			label: 'Investing',
			total: investing.total,
			bar: 'bg-investing-line',
			verbPositive: 'Withdrew',
			verbNegative: 'Contributed'
		},
		{
			label: 'Financing',
			total: financing.total,
			bar: 'bg-financing-line',
			verbPositive: 'Borrowed',
			verbNegative: 'Repaid'
		}
	]);

	const maxAbs = $derived(
		Math.max(1, ...rows.map((r) => Math.abs(r.total)))
	);
	const netChange = $derived(closingCash - openingCash);
</script>

<div class="space-y-4">
	<div class="rounded-xl border border-border bg-surface-card p-4">
		<div class="mb-3 flex items-baseline justify-between">
			<h4 class="text-xs font-semibold uppercase tracking-wide text-ink-muted">By section</h4>
			<span class="font-mono text-[10px] text-ink-faint">cash out / in</span>
		</div>
		<ul class="space-y-2.5">
			{#each rows as row}
				{@const fraction = Math.abs(row.total) / maxAbs}
				{@const positive = row.total >= 0}
				<li>
					<div class="flex items-baseline justify-between text-xs">
						<span class="text-ink-body">{row.label}</span>
						<span class="font-mono tabular-nums text-ink">
							<span class="font-bold text-ink-muted">
								{positive ? row.verbPositive : row.verbNegative}
							</span>
							{fmt(Math.abs(row.total))}
						</span>
					</div>
					<div class="mt-1 flex h-1.5 items-center">
						<div class="flex h-full w-1/2 justify-end overflow-hidden">
							{#if !positive}
								<div
									class="h-full {row.bar}"
									style:width="{fraction * 100}%"
								></div>
							{/if}
						</div>
						<div class="h-full w-px bg-trend-zero"></div>
						<div class="h-full w-1/2 overflow-hidden">
							{#if positive}
								<div
									class="h-full {row.bar}"
									style:width="{fraction * 100}%"
								></div>
							{/if}
						</div>
					</div>
				</li>
			{/each}
		</ul>
	</div>

	<div class="rounded-xl border border-border bg-surface-card p-4">
		<h4 class="mb-2 text-xs font-semibold uppercase tracking-wide text-ink-muted">
			Reconciliation
		</h4>
		<div class="grid grid-cols-3 gap-3 text-center">
			<div>
				<p class="text-[10px] uppercase tracking-wide text-ink-muted">Opening</p>
				<p class="mt-0.5 font-mono text-sm tabular-nums text-ink-body">{fmt(openingCash)}</p>
			</div>
			<div>
				<p class="text-[10px] uppercase tracking-wide text-ink-muted">Net Δ</p>
				<p
					class="mt-0.5 font-mono text-sm font-semibold tabular-nums {netChange >= 0
						? 'text-positive'
						: 'text-negative'}"
				>
					{fmt(netChange)}
				</p>
			</div>
			<div>
				<p class="text-[10px] uppercase tracking-wide text-ink-muted">Closing</p>
				<p class="mt-0.5 font-mono text-sm tabular-nums text-ink">{fmt(closingCash)}</p>
			</div>
		</div>
	</div>
</div>
