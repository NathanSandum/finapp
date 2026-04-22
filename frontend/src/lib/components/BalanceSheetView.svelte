<script lang="ts">
	import * as api from '$lib/api.svelte';
	import type { Quarter } from '$lib/period';
	import type { MultiPeriodBalanceSheet } from '$lib/types';
	import ReportTable from './ReportTable.svelte';

	let { workspaceId, quarter }: { workspaceId: string; quarter: Quarter } = $props();

	let sheet: MultiPeriodBalanceSheet | null = $state(null);
	let loading = $state(true);

	$effect(() => {
		loading = true;
		sheet = null;
		api
			.getMultiPeriodBalanceSheet(workspaceId, quarter)
			.then((data) => (sheet = data))
			.catch(() => {})
			.finally(() => (loading = false));
	});
</script>

{#if loading}
	<div class="flex min-h-64 items-center justify-center text-gray-600">Loading...</div>
{:else if !sheet}
	<div class="flex min-h-64 items-center justify-center text-gray-600">No data available</div>
{:else}
	<ReportTable
		periods={sheet.periods}
		sections={[
			{
				title: 'Assets',
				rows: sheet.assets,
				totals: sheet.total_assets
			},
			{
				title: 'Liabilities',
				rows: sheet.liabilities,
				totals: sheet.total_liabilities
			},
			{
				title: 'Equity',
				rows: sheet.equity,
				totals: sheet.total_equity
			}
		]}
		bottomLine={{
			label: 'Liabilities + Equity + Net Income',
			values: sheet.total_liabilities_and_equity
		}}
	/>
{/if}
