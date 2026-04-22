<script lang="ts">
	import * as api from '$lib/api.svelte';
	import type { Quarter } from '$lib/period';
	import type { MultiPeriodIncomeStatement } from '$lib/types';
	import ReportTable from './ReportTable.svelte';

	let { workspaceId, quarter }: { workspaceId: string; quarter: Quarter } = $props();

	let statement: MultiPeriodIncomeStatement | null = $state(null);
	let loading = $state(true);

	$effect(() => {
		loading = true;
		statement = null;
		api
			.getMultiPeriodIncomeStatement(workspaceId, quarter)
			.then((data) => (statement = data))
			.catch(() => {})
			.finally(() => (loading = false));
	});
</script>

{#if loading}
	<div class="flex min-h-64 items-center justify-center text-gray-600">Loading...</div>
{:else if !statement}
	<div class="flex min-h-64 items-center justify-center text-gray-600">No data available</div>
{:else}
	<ReportTable
		periods={statement.periods}
		sections={[
			{
				title: 'Revenue',
				rows: statement.revenue,
				totals: statement.total_revenue
			},
			{
				title: 'Expenses',
				rows: statement.expenses,
				totals: statement.total_expenses
			}
		]}
		bottomLine={{
			label: 'Net Cash Flow',
			values: statement.net_income
		}}
	/>
{/if}
