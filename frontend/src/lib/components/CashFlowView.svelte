<script lang="ts">
	import * as api from '$lib/api.svelte';
	import type { Quarter } from '$lib/period';
	import type { MultiPeriodIncomeStatement } from '$lib/types';
	import ReportShell from './ReportShell.svelte';
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

<ReportShell {loading} hasData={!!statement}>
	{#if statement}
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
</ReportShell>
