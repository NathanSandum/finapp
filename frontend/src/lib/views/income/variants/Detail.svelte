<script lang="ts">
	import RankedFlow from '$lib/components/RankedFlow.svelte';
	import type { Line } from '../lib';

	let {
		revenue,
		expenses,
		revenueTotal,
		expenseTotal,
		priorRevenueTotal,
		priorExpenseTotal,
		priorShortLabel
	}: {
		revenue: Line[];
		expenses: Line[];
		revenueTotal: number;
		expenseTotal: number;
		priorRevenueTotal: number;
		priorExpenseTotal: number;
		priorShortLabel: string;
	} = $props();

	const net = $derived(revenueTotal - expenseTotal);
	const priorNet = $derived(priorRevenueTotal - priorExpenseTotal);
</script>

<RankedFlow
	positive={revenue}
	negative={expenses}
	{net}
	{priorNet}
	{priorShortLabel}
	netLabel="Net"
/>
