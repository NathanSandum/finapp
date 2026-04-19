<script lang="ts">
	import * as api from '$lib/api.svelte';
	import { onMount } from 'svelte';

	type Account = { id: string; name: string; account_type: string; currency: string };
	type AccountWithBalance = Account & { cleared_balance: number };

	let { workspaceId }: { workspaceId: string } = $props();

	let accounts: AccountWithBalance[] = $state([]);
	let loading = $state(true);

	onMount(async () => {
		try {
			const data = await api.listAccounts(workspaceId);
			const withBalances = await Promise.all(
				data.accounts.map(async (acct: Account) => {
					const bal = await api.getBalance(workspaceId, acct.id);
					return { ...acct, cleared_balance: bal.cleared_balance };
				})
			);
			accounts = withBalances;
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

	let totalBalance = $derived(accounts.reduce((sum, a) => sum + a.cleared_balance, 0));
</script>

<div>
	<header class="mb-6 border-b border-gray-800 pb-4">
		<h2 class="text-lg font-semibold text-gray-100">Balance Sheet</h2>
		<div class="mt-1 flex items-baseline gap-2">
			<span class="text-sm text-gray-600">Net worth</span>
			<span
				class="text-2xl font-semibold tabular-nums {totalBalance >= 0 ? 'text-emerald-400' : 'text-red-400'}"
			>
				{formatAmount(totalBalance)}
			</span>
		</div>
	</header>

	{#if loading}
		<div class="flex min-h-64 items-center justify-center text-gray-600">Loading...</div>
	{:else if accounts.length === 0}
		<div class="flex min-h-64 items-center justify-center text-gray-600">
			No accounts yet
		</div>
	{:else}
		<table class="w-full">
			<thead>
				<tr
					class="border-b border-gray-800 text-left text-[11px] font-semibold uppercase tracking-wide text-gray-600"
				>
					<th class="px-3 py-2">Account</th>
					<th class="px-3 py-2">Type</th>
					<th class="px-3 py-2 text-right">Cleared Balance</th>
				</tr>
			</thead>
			<tbody>
				{#each accounts as acct}
					<tr class="border-b border-gray-900 hover:bg-gray-900/50">
						<td class="px-3 py-2.5 text-sm font-medium text-gray-300">{acct.name}</td>
						<td class="px-3 py-2.5 text-sm capitalize text-gray-500">{acct.account_type}</td>
						<td
							class="px-3 py-2.5 text-right text-sm font-medium tabular-nums {acct.cleared_balance >= 0 ? 'text-emerald-400' : 'text-red-400'}"
						>
							{formatAmount(acct.cleared_balance)}
						</td>
					</tr>
				{/each}
			</tbody>
			<tfoot>
				<tr class="border-t border-gray-700">
					<td class="px-3 py-2.5 text-sm font-semibold text-gray-300" colspan="2">Total</td>
					<td
						class="px-3 py-2.5 text-right text-sm font-semibold tabular-nums {totalBalance >= 0 ? 'text-emerald-400' : 'text-red-400'}"
					>
						{formatAmount(totalBalance)}
					</td>
				</tr>
			</tfoot>
		</table>
	{/if}
</div>
