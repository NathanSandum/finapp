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

	const revenuePalette = ['bg-positive-bar-1', 'bg-positive-bar-2', 'bg-positive-bar-3', 'bg-positive-bar-4'];
	const expensePalette = ['bg-negative-bar-1', 'bg-negative-bar-2', 'bg-negative-bar-3', 'bg-negative-bar-4'];
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
