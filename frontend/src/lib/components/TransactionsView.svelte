<script lang="ts">
	import * as api from '$lib/api.svelte';
	import { onMount } from 'svelte';

	type Account = { id: string; name: string; account_type: string; currency: string };
	type Transaction = {
		id: string;
		account: string;
		date: string;
		description: string;
		amount: number;
		cleared: boolean;
		line_items: { id: string; category: string; amount: number; note: string | null }[];
	};

	let { workspaceId }: { workspaceId: string } = $props();

	let accounts: Account[] = $state([]);
	let selectedAccount: Account | null = $state(null);
	let transactions: Transaction[] = $state([]);
	let clearedBalance = $state(0);
	let loading = $state(true);

	onMount(async () => {
		try {
			const data = await api.listAccounts(workspaceId);
			accounts = data.accounts;
			if (accounts.length > 0) {
				await selectAccount(accounts[0]);
			}
		} catch {
			// no data yet
		} finally {
			loading = false;
		}
	});

	async function selectAccount(acct: Account) {
		selectedAccount = acct;
		const [txnData, balData] = await Promise.all([
			api.listTransactions(workspaceId, acct.id),
			api.getBalance(workspaceId, acct.id)
		]);
		transactions = txnData.transactions;
		clearedBalance = balData.cleared_balance;
	}

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
		<h2 class="text-lg font-semibold text-gray-100">Transactions</h2>
	</header>

	{#if loading}
		<div class="flex min-h-64 items-center justify-center text-gray-600">Loading...</div>
	{:else}
		<!-- Account selector -->
		<div class="mb-4 flex items-center gap-3">
			{#each accounts as acct}
				<button
					class="rounded-lg border px-3 py-1.5 text-sm transition-colors {selectedAccount?.id === acct.id
						? 'border-blue-700 bg-blue-950/50 text-white'
						: 'border-gray-800 text-gray-400 hover:border-gray-700 hover:text-gray-300'}"
					onclick={() => selectAccount(acct)}
				>
					{acct.name}
				</button>
			{/each}
		</div>

		{#if selectedAccount}
			<div class="mb-4 flex items-baseline justify-between">
				<span class="text-sm text-gray-600"
					>{selectedAccount.account_type} &middot; {selectedAccount.currency}</span
				>
				<div class="text-right">
					<span class="mr-2 text-[11px] font-semibold uppercase tracking-wide text-gray-600"
						>Cleared</span
					>
					<span
						class="text-lg font-semibold tabular-nums {clearedBalance < 0
							? 'text-red-400'
							: 'text-emerald-400'}"
					>
						{formatAmount(clearedBalance)}
					</span>
				</div>
			</div>

			{#if transactions.length === 0}
				<div class="flex min-h-48 items-center justify-center text-gray-600">
					No transactions yet
				</div>
			{:else}
				<table class="w-full">
					<thead>
						<tr
							class="border-b border-gray-800 text-left text-[11px] font-semibold uppercase tracking-wide text-gray-600"
						>
							<th class="px-3 py-2">Date</th>
							<th class="px-3 py-2">Description</th>
							<th class="px-3 py-2 text-right">Amount</th>
							<th class="px-3 py-2">Status</th>
						</tr>
					</thead>
					<tbody>
						{#each transactions as txn}
							<tr class="border-b border-gray-900 hover:bg-gray-900/50">
								<td class="px-3 py-2.5 text-sm tabular-nums text-gray-500">{txn.date}</td>
								<td class="px-3 py-2.5 text-sm text-gray-300"
									>{txn.description || '\u2014'}</td
								>
								<td
									class="px-3 py-2.5 text-right text-sm font-medium tabular-nums {txn.amount < 0
										? 'text-red-400'
										: 'text-emerald-400'}"
								>
									{formatAmount(txn.amount)}
								</td>
								<td class="px-3 py-2.5">
									{#if txn.cleared}
										<span
											class="rounded-full bg-emerald-950 px-2 py-0.5 text-[11px] font-medium text-emerald-400"
											>Cleared</span
										>
									{:else}
										<span
											class="rounded-full bg-yellow-950 px-2 py-0.5 text-[11px] font-medium text-yellow-400"
											>Pending</span
										>
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		{:else}
			<div class="flex min-h-48 items-center justify-center text-gray-600">
				No accounts yet
			</div>
		{/if}
	{/if}
</div>
