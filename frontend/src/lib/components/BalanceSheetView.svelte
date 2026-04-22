<script lang="ts">
	import * as api from '$lib/api.svelte';
	import { formatAmount } from '$lib/format';
	import type { BalanceSheet } from '$lib/types';
	import { onMount } from 'svelte';

	let { workspaceId }: { workspaceId: string } = $props();

	let sheet: BalanceSheet | null = $state(null);
	let loading = $state(true);

	const today = new Date().toISOString().slice(0, 10);

	onMount(async () => {
		try {
			sheet = await api.getBalanceSheet(workspaceId, today);
		} catch {
			// no data yet
		} finally {
			loading = false;
		}
	});
</script>

<div>
	<header class="mb-6 border-b border-gray-800 pb-4">
		<h2 class="text-lg font-semibold text-gray-100">Balance Sheet</h2>
		<span class="text-sm text-gray-600">As of {today}</span>
	</header>

	{#if loading}
		<div class="flex min-h-64 items-center justify-center text-gray-600">Loading...</div>
	{:else if !sheet}
		<div class="flex min-h-64 items-center justify-center text-gray-600">
			No data available
		</div>
	{:else}
		<!-- Summary cards -->
		<div class="mb-6 grid grid-cols-3 gap-4">
			<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
				<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
					>Total Assets</span
				>
				<span
					class="text-xl font-semibold tabular-nums {sheet.total_assets >= 0 ? 'text-emerald-400' : 'text-red-400'}"
					>{formatAmount(sheet.total_assets)}</span
				>
			</div>
			<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
				<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
					>Total Liabilities</span
				>
				<span class="text-xl font-semibold tabular-nums text-gray-300"
					>{formatAmount(sheet.total_liabilities)}</span
				>
			</div>
			<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
				<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
					>Net Income</span
				>
				<span
					class="text-xl font-semibold tabular-nums {sheet.net_income >= 0 ? 'text-emerald-400' : 'text-red-400'}"
					>{formatAmount(sheet.net_income)}</span
				>
			</div>
		</div>

		{#each [{ title: 'Assets', items: sheet.assets, total: sheet.total_assets }, { title: 'Liabilities', items: sheet.liabilities, total: sheet.total_liabilities }, { title: 'Equity', items: sheet.equity, total: sheet.total_equity }] as section}
			{#if section.items.length > 0}
				<div class="mb-6">
					<h3
						class="mb-2 text-[11px] font-semibold uppercase tracking-wide text-gray-600"
					>
						{section.title}
					</h3>
					<table class="w-full">
						<tbody>
							{#each section.items as item}
								<tr class="border-b border-gray-900 hover:bg-gray-900/50">
									<td class="px-3 py-2 text-sm text-gray-300">{item.name}</td>
									<td
										class="px-3 py-2 text-right text-sm font-medium tabular-nums {item.balance >= 0 ? 'text-emerald-400' : 'text-red-400'}"
									>
										{formatAmount(item.balance)}
									</td>
								</tr>
							{/each}
						</tbody>
						<tfoot>
							<tr class="border-t border-gray-700">
								<td class="px-3 py-2 text-sm font-semibold text-gray-300"
									>Total {section.title}</td
								>
								<td
									class="px-3 py-2 text-right text-sm font-semibold tabular-nums text-gray-300"
								>
									{formatAmount(section.total)}
								</td>
							</tr>
						</tfoot>
					</table>
				</div>
			{/if}
		{/each}

		<!-- Accounting equation -->
		<div class="rounded-lg border border-gray-800 bg-gray-900/50 px-4 py-3 text-sm text-gray-500">
			Assets ({formatAmount(sheet.total_assets)}) = Liabilities + Equity + Net Income ({formatAmount(sheet.total_liabilities_and_equity)})
		</div>
	{/if}
</div>
