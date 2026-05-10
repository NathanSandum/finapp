<script lang="ts">
	import CompositionSection from '$lib/components/CompositionSection.svelte';
	import type { Line } from '../lib';

	let {
		revenue,
		expenses,
		revenueTotal,
		expenseTotal
	}: {
		revenue: Line[];
		expenses: Line[];
		revenueTotal: number;
		expenseTotal: number;
	} = $props();

	const revenuePalette = ['bg-emerald-500', 'bg-emerald-400', 'bg-emerald-300', 'bg-emerald-200'];
	const expensePalette = ['bg-rose-500', 'bg-rose-400', 'bg-rose-300', 'bg-rose-200'];
</script>

<div class="space-y-4">
	<CompositionSection
		title="Revenue"
		total={revenueTotal}
		lines={revenue}
		base={revenueTotal}
		palette={revenuePalette}
		minLabelFraction={0.08}
		legend
	/>
	<CompositionSection
		title="Expenses"
		total={expenseTotal}
		lines={expenses}
		base={revenueTotal}
		palette={expensePalette}
		minLabelFraction={0.04}
		footnote="Width relative to revenue"
	/>
</div>
