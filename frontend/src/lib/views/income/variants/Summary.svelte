<script lang="ts">
	import CompositionFlow from '$lib/components/CompositionFlow.svelte';
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
</script>

<CompositionFlow
	positive={{ lines: revenue, total: revenueTotal, label: 'In' }}
	negative={{ lines: expenses, total: expenseTotal, label: 'Out' }}
	netLabel="Net"
/>
