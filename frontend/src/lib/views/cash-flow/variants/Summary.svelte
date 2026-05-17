<script lang="ts">
	import CompositionFlow from '$lib/components/CompositionFlow.svelte';
	import { fmt } from '$lib/format';
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

	type Section = { name: string; total: number };
	const sections = $derived<Section[]>([
		{ name: 'Income', total: operating.total },
		{ name: 'Investing', total: investing.total },
		{ name: 'Financing', total: financing.total }
	]);

	const positiveLines = $derived(
		sections.filter((s) => s.total > 0).map((s) => ({ name: s.name, amount: s.total }))
	);
	const negativeLines = $derived(
		sections.filter((s) => s.total < 0).map((s) => ({ name: s.name, amount: -s.total }))
	);
	const positiveTotal = $derived(positiveLines.reduce((s, l) => s + l.amount, 0));
	const negativeTotal = $derived(negativeLines.reduce((s, l) => s + l.amount, 0));
	const netChange = $derived(closingCash - openingCash);

	const outflowPalette = [
		'bg-outflow-bar-1',
		'bg-outflow-bar-2',
		'bg-outflow-bar-3',
		'bg-outflow-bar-4'
	];
</script>

<div class="space-y-3">
	<CompositionFlow
		positive={{ lines: positiveLines, total: positiveTotal, label: 'Cash In' }}
		negative={{ lines: negativeLines, total: negativeTotal, label: 'Cash Out' }}
		netLabel="Net Change in Cash"
		showNet={false}
		negativePalette={outflowPalette}
		negativeText="text-ink-muted"
		negativeSignPrefix=""
	/>
	<div class="grid grid-cols-3 gap-2 pt-2 text-center">
		<div>
			<p class="text-[10px] uppercase tracking-wide text-ink-muted">Opening</p>
			<p class="font-mono text-base tabular-nums text-ink-body">{fmt(openingCash)}</p>
		</div>
		<div>
			<p class="text-[10px] uppercase tracking-wide text-ink-muted">Change</p>
			<p class="font-mono text-base font-semibold tabular-nums {netChange >= 0 ? 'text-positive' : 'text-negative'}">
				{fmt(netChange)}
			</p>
		</div>
		<div>
			<p class="text-[10px] uppercase tracking-wide text-ink-muted">Closing</p>
			<p class="font-mono text-base tabular-nums text-ink">{fmt(closingCash)}</p>
		</div>
	</div>
</div>
