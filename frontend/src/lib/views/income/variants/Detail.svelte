<script lang="ts">
	import AccountColumn from '$lib/components/AccountColumn.svelte';
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
</script>

<div class="grid grid-cols-2 gap-3">
	<AccountColumn
		title="Revenue"
		lines={revenue}
		total={revenueTotal}
		prior={priorRevenueTotal}
		{priorShortLabel}
		accent="emerald"
		direction="higher"
	/>
	<AccountColumn
		title="Expenses"
		lines={expenses}
		total={expenseTotal}
		prior={priorExpenseTotal}
		{priorShortLabel}
		accent="rose"
		direction="lower"
	/>
</div>
