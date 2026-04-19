<script lang="ts">
	import * as api from '$lib/api.svelte';
	import { onMount } from 'svelte';

	type SummaryItem = {
		category: string;
		actual: number;
		baseline: number | null;
		variance: number | null;
		variance_pct: number | null;
	};

	let { workspaceId }: { workspaceId: string } = $props();

	let summary: SummaryItem[] = $state([]);
	let loading = $state(true);

	const now = new Date();
	const periodStart = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-01`;
	const nextMonth = new Date(now.getFullYear(), now.getMonth() + 1, 0);
	const periodEnd = `${nextMonth.getFullYear()}-${String(nextMonth.getMonth() + 1).padStart(2, '0')}-${String(nextMonth.getDate()).padStart(2, '0')}`;

	onMount(async () => {
		try {
			const data = await api.baselineSummary(workspaceId, periodStart, periodEnd);
			summary = data.summary;
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

	let totalActual = $derived(summary.reduce((sum, item) => sum + item.actual, 0));
	let totalBaseline = $derived(
		summary.reduce((sum, item) => sum + (item.baseline ?? 0), 0)
	);
	let netCashFlow = $derived(totalActual);
</script>

<div>
	<header class="mb-6 border-b border-gray-800 pb-4">
		<h2 class="text-lg font-semibold text-gray-100">Cash Flow</h2>
		<span class="text-sm text-gray-600">Inflows and outflows for {periodStart}</span>
	</header>

	{#if loading}
		<div class="flex min-h-64 items-center justify-center text-gray-600">Loading...</div>
	{:else if summary.length === 0}
		<div class="flex min-h-64 items-center justify-center text-gray-600">
			No transactions this period
		</div>
	{:else}
		<div class="mb-6 grid grid-cols-3 gap-4">
			<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
				<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
					>Budget</span
				>
				<span class="text-xl font-semibold tabular-nums text-gray-300"
					>{formatAmount(totalBaseline)}</span
				>
			</div>
			<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
				<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
					>Actual</span
				>
				<span class="text-xl font-semibold tabular-nums text-gray-300"
					>{formatAmount(totalActual)}</span
				>
			</div>
			<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
				<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
					>Net Cash Flow</span
				>
				<span
					class="text-xl font-semibold tabular-nums {netCashFlow >= 0 ? 'text-emerald-400' : 'text-red-400'}"
					>{formatAmount(netCashFlow)}</span
				>
			</div>
		</div>

		<table class="w-full">
			<thead>
				<tr
					class="border-b border-gray-800 text-left text-[11px] font-semibold uppercase tracking-wide text-gray-600"
				>
					<th class="px-3 py-2">Category</th>
					<th class="px-3 py-2 text-right">Baseline</th>
					<th class="px-3 py-2 text-right">Actual</th>
					<th class="px-3 py-2 text-right">Variance</th>
				</tr>
			</thead>
			<tbody>
				{#each summary as item}
					<tr class="border-b border-gray-900 hover:bg-gray-900/50">
						<td class="px-3 py-2.5 text-sm text-gray-300">{item.category}</td>
						<td class="px-3 py-2.5 text-right text-sm tabular-nums text-gray-500">
							{item.baseline !== null ? formatAmount(item.baseline) : '\u2014'}
						</td>
						<td class="px-3 py-2.5 text-right text-sm font-medium tabular-nums text-gray-300">
							{formatAmount(item.actual)}
						</td>
						<td
							class="px-3 py-2.5 text-right text-sm font-medium tabular-nums {item.variance !== null && item.variance >= 0 ? 'text-emerald-400' : 'text-red-400'}"
						>
							{item.variance !== null ? formatAmount(item.variance) : '\u2014'}
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{/if}
</div>
