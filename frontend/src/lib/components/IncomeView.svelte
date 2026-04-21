<script lang="ts">
	import * as api from '$lib/api.svelte';
	import { onMount } from 'svelte';

	type AccountBalance = { account: string; name: string; balance: number };
	type IncomeStatement = {
		start_date: string;
		end_date: string;
		revenue: AccountBalance[];
		expenses: AccountBalance[];
		total_revenue: number;
		total_expenses: number;
		net_income: number;
	};

	let { workspaceId }: { workspaceId: string } = $props();

	let statement: IncomeStatement | null = $state(null);
	let loading = $state(true);

	// Current month boundaries
	const now = new Date();
	const periodStart = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-01`;
	const lastDay = new Date(now.getFullYear(), now.getMonth() + 1, 0);
	const periodEnd = `${lastDay.getFullYear()}-${String(lastDay.getMonth() + 1).padStart(2, '0')}-${String(lastDay.getDate()).padStart(2, '0')}`;

	onMount(async () => {
		try {
			statement = await api.getIncomeStatement(workspaceId, periodStart, periodEnd);
		} catch {
			// no data yet
		} finally {
			loading = false;
		}
	});

	function formatAmount(cents: number): string {
		const abs = Math.abs(cents);
		const dollars = Math.floor(abs / 100);
		const remainder = abs % 100;
		const sign = cents < 0 ? '-' : '';
		return `${sign}$${dollars.toLocaleString()}.${String(remainder).padStart(2, '0')}`;
	}
</script>

<div>
	<header class="mb-6 border-b border-gray-800 pb-4">
		<h2 class="text-lg font-semibold text-gray-100">Income Statement</h2>
		<span class="text-sm text-gray-600">{periodStart} to {periodEnd}</span>
	</header>

	{#if loading}
		<div class="flex min-h-64 items-center justify-center text-gray-600">Loading...</div>
	{:else if !statement}
		<div class="flex min-h-64 items-center justify-center text-gray-600">
			No data available
		</div>
	{:else}
		<!-- Net income highlight -->
		<div class="mb-6 rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
			<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
				>Net Income</span
			>
			<span
				class="text-2xl font-semibold tabular-nums {statement.net_income >= 0 ? 'text-emerald-400' : 'text-red-400'}"
				>{formatAmount(statement.net_income)}</span
			>
		</div>

		<!-- Revenue section -->
		<div class="mb-6">
			<h3 class="mb-2 text-[11px] font-semibold uppercase tracking-wide text-gray-600">
				Revenue
			</h3>
			{#if statement.revenue.length === 0}
				<p class="px-3 text-sm text-gray-600">No revenue this period</p>
			{:else}
				<table class="w-full">
					<tbody>
						{#each statement.revenue as item}
							<tr class="border-b border-gray-900 hover:bg-gray-900/50">
								<td class="px-3 py-2 text-sm text-gray-300">{item.name}</td>
								<td
									class="px-3 py-2 text-right text-sm font-medium tabular-nums text-emerald-400"
								>
									{formatAmount(item.balance)}
								</td>
							</tr>
						{/each}
					</tbody>
					<tfoot>
						<tr class="border-t border-gray-700">
							<td class="px-3 py-2 text-sm font-semibold text-gray-300">Total Revenue</td
							>
							<td
								class="px-3 py-2 text-right text-sm font-semibold tabular-nums text-emerald-400"
							>
								{formatAmount(statement.total_revenue)}
							</td>
						</tr>
					</tfoot>
				</table>
			{/if}
		</div>

		<!-- Expenses section -->
		<div class="mb-6">
			<h3 class="mb-2 text-[11px] font-semibold uppercase tracking-wide text-gray-600">
				Expenses
			</h3>
			{#if statement.expenses.length === 0}
				<p class="px-3 text-sm text-gray-600">No expenses this period</p>
			{:else}
				<table class="w-full">
					<tbody>
						{#each statement.expenses as item}
							<tr class="border-b border-gray-900 hover:bg-gray-900/50">
								<td class="px-3 py-2 text-sm text-gray-300">{item.name}</td>
								<td
									class="px-3 py-2 text-right text-sm font-medium tabular-nums text-red-400"
								>
									{formatAmount(item.balance)}
								</td>
							</tr>
						{/each}
					</tbody>
					<tfoot>
						<tr class="border-t border-gray-700">
							<td class="px-3 py-2 text-sm font-semibold text-gray-300"
								>Total Expenses</td
							>
							<td
								class="px-3 py-2 text-right text-sm font-semibold tabular-nums text-red-400"
							>
								{formatAmount(statement.total_expenses)}
							</td>
						</tr>
					</tfoot>
				</table>
			{/if}
		</div>
	{/if}
</div>
