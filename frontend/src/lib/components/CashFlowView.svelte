<script lang="ts">
	import * as api from '$lib/api.svelte';
	import { formatAmount } from '$lib/format';
	import { currentMonthPeriod } from '$lib/period';
	import type { AccountBalance, IncomeStatement } from '$lib/types';
	import { onMount } from 'svelte';

	let { workspaceId }: { workspaceId: string } = $props();

	let statement: IncomeStatement | null = $state(null);
	let loading = $state(true);

	const { periodStart, periodEnd } = currentMonthPeriod();

	onMount(async () => {
		try {
			statement = await api.getIncomeStatement(workspaceId, periodStart, periodEnd);
		} catch {
			// no data yet
		} finally {
			loading = false;
		}
	});

	// Combine revenue + expense items for table
	let allItems = $derived.by(() => {
		if (!statement) return [];
		return [
			...statement.revenue.map((r: AccountBalance) => ({ ...r, type: 'revenue' as const })),
			...statement.expenses.map((e: AccountBalance) => ({ ...e, type: 'expense' as const }))
		];
	});
</script>

<div>
	<header class="mb-6 border-b border-gray-800 pb-4">
		<h2 class="text-lg font-semibold text-gray-100">Cash Flow</h2>
		<span class="text-sm text-gray-600">{periodStart} to {periodEnd}</span>
	</header>

	{#if loading}
		<div class="flex min-h-64 items-center justify-center text-gray-600">Loading...</div>
	{:else if !statement}
		<div class="flex min-h-64 items-center justify-center text-gray-600">
			No data available
		</div>
	{:else}
		<!-- Summary cards -->
		<div class="mb-6 grid grid-cols-3 gap-4">
			<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
				<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
					>Revenue</span
				>
				<span class="text-xl font-semibold tabular-nums text-emerald-400"
					>{formatAmount(statement.total_revenue)}</span
				>
			</div>
			<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
				<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
					>Expenses</span
				>
				<span class="text-xl font-semibold tabular-nums text-red-400"
					>{formatAmount(statement.total_expenses)}</span
				>
			</div>
			<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
				<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
					>Net Income</span
				>
				<span
					class="text-xl font-semibold tabular-nums {statement.net_income >= 0 ? 'text-emerald-400' : 'text-red-400'}"
					>{formatAmount(statement.net_income)}</span
				>
			</div>
		</div>

		{#if allItems.length === 0}
			<div class="flex min-h-48 items-center justify-center text-gray-600">
				No activity this period
			</div>
		{:else}
			<table class="w-full">
				<thead>
					<tr
						class="border-b border-gray-800 text-left text-[11px] font-semibold uppercase tracking-wide text-gray-600"
					>
						<th class="px-3 py-2">Account</th>
						<th class="px-3 py-2">Type</th>
						<th class="px-3 py-2 text-right">Amount</th>
					</tr>
				</thead>
				<tbody>
					{#each allItems as item}
						<tr class="border-b border-gray-900 hover:bg-gray-900/50">
							<td class="px-3 py-2.5 text-sm text-gray-300">{item.name}</td>
							<td class="px-3 py-2.5 text-sm capitalize text-gray-500">{item.type}</td>
							<td
								class="px-3 py-2.5 text-right text-sm font-medium tabular-nums {item.type === 'revenue' ? 'text-emerald-400' : 'text-red-400'}"
							>
								{formatAmount(item.balance)}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	{/if}
</div>
