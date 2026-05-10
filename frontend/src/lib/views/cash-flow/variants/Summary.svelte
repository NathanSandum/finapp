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

	type Row = { label: string; total: number; bar: string };

	const rows = $derived<Row[]>([
		{ label: 'Income', total: operating.total, bar: 'bg-emerald-400' },
		{ label: 'Investing', total: investing.total, bar: 'bg-sky-400' },
		{ label: 'Financing', total: financing.total, bar: 'bg-amber-400' }
	]);

	const maxAbs = $derived(
		Math.max(1, ...rows.map((r) => Math.abs(r.total)))
	);
	const netChange = $derived(closingCash - openingCash);
</script>

<div class="space-y-4">
	<div class="rounded-xl border border-neutral-200 bg-white p-4">
		<div class="mb-3 flex items-baseline justify-between">
			<h4 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">By section</h4>
			<span class="font-mono text-[10px] text-neutral-400">cash out / in</span>
		</div>
		<ul class="space-y-2.5">
			{#each rows as row}
				{@const fraction = Math.abs(row.total) / maxAbs}
				{@const positive = row.total >= 0}
				<li>
					<div class="flex items-baseline justify-between text-xs">
						<span class="text-neutral-700">{row.label}</span>
						<span
							class="font-mono tabular-nums {positive
								? 'text-neutral-900'
								: 'text-rose-700'}"
						>
							{fmt(row.total)}
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
						<div class="h-full w-px bg-neutral-300"></div>
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

	<div class="rounded-xl border border-neutral-200 bg-white p-4">
		<h4 class="mb-2 text-xs font-semibold uppercase tracking-wide text-neutral-500">
			Reconciliation
		</h4>
		<div class="grid grid-cols-3 gap-3 text-center">
			<div>
				<p class="text-[10px] uppercase tracking-wide text-neutral-500">Opening</p>
				<p class="mt-0.5 font-mono text-sm tabular-nums text-neutral-700">{fmt(openingCash)}</p>
			</div>
			<div>
				<p class="text-[10px] uppercase tracking-wide text-neutral-500">Net Δ</p>
				<p
					class="mt-0.5 font-mono text-sm font-semibold tabular-nums {netChange >= 0
						? 'text-emerald-700'
						: 'text-rose-700'}"
				>
					{fmt(netChange)}
				</p>
			</div>
			<div>
				<p class="text-[10px] uppercase tracking-wide text-neutral-500">Closing</p>
				<p class="mt-0.5 font-mono text-sm tabular-nums text-neutral-900">{fmt(closingCash)}</p>
			</div>
		</div>
	</div>
</div>
