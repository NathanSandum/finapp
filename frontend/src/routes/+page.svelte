<script lang="ts">
	import { onMount } from 'svelte';
	import * as api from '$lib/api.svelte';

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

	let workspaceId = $state('');
	let accounts: Account[] = $state([]);
	let selectedAccount: Account | null = $state(null);
	let transactions: Transaction[] = $state([]);
	let clearedBalance = $state(0);
	let loading = $state(true);
	let error = $state('');

	onMount(async () => {
		try {
			await ensureAuth();
			await ensureWorkspace();
			await loadAccounts();
		} catch (e: unknown) {
			const err = e as { message?: string };
			error = err.message || 'Failed to connect to backend';
		} finally {
			loading = false;
		}
	});

	async function ensureAuth() {
		if (api.getToken()) {
			// Check if token is still valid
			try {
				await api.listWorkspaces();
				return;
			} catch {
				api.setToken(null);
			}
		}
		// Auto-register a dev user
		try {
			const data = await api.register('dev@finapp.local', 'Dev User', 'dev');
			api.setToken(data.token);
		} catch {
			// Already registered — log in
			const data = await api.login('dev@finapp.local', 'dev');
			api.setToken(data.token);
		}
	}

	async function ensureWorkspace() {
		const wsData = await api.listWorkspaces();
		if (wsData.workspaces.length === 0) {
			const created = await api.createWorkspace('My Finances');
			workspaceId = created.id;
		} else {
			workspaceId = wsData.workspaces[0].id;
		}
	}

	async function loadAccounts() {
		const data = await api.listAccounts(workspaceId);
		accounts = data.accounts;
		if (accounts.length > 0 && !selectedAccount) {
			await selectAccount(accounts[0]);
		}
	}

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

{#if loading}
	<div class="flex min-h-screen items-center justify-center bg-gray-950 text-gray-500">
		Loading...
	</div>
{:else if error}
	<div class="flex min-h-screen items-center justify-center bg-gray-950">
		<div class="rounded-xl border border-red-900 bg-red-950/50 px-6 py-4 text-sm text-red-400">
			{error}
		</div>
	</div>
{:else}
	<div class="flex min-h-screen bg-gray-950">
		<!-- Sidebar -->
		<aside class="flex w-60 shrink-0 flex-col border-r border-gray-800 bg-gray-900">
			<div class="border-b border-gray-800 px-5 py-4">
				<h1 class="text-base font-semibold text-gray-100">FinApp</h1>
			</div>

			<div class="px-5 pt-4 pb-2 text-[11px] font-semibold uppercase tracking-wide text-gray-600">
				Accounts
			</div>

			{#if accounts.length === 0}
				<p class="px-5 text-sm text-gray-600">No accounts yet</p>
			{:else}
				<nav class="flex flex-col">
					{#each accounts as acct}
						<button
							class="flex flex-col items-start px-5 py-2 text-left text-sm {selectedAccount?.id ===
							acct.id
								? 'bg-blue-950/50 text-white'
								: 'text-gray-400 hover:bg-gray-800/50'}"
							onclick={() => selectAccount(acct)}
						>
							<span class="font-medium">{acct.name}</span>
							<span
								class="text-[11px] capitalize {selectedAccount?.id === acct.id
									? 'text-blue-400'
									: 'text-gray-600'}">{acct.account_type}</span
							>
						</button>
					{/each}
				</nav>
			{/if}
		</aside>

		<!-- Main content -->
		<main class="flex-1 overflow-x-auto p-6">
			{#if selectedAccount}
				<header class="mb-6 flex items-start justify-between border-b border-gray-800 pb-4">
					<div>
						<h2 class="text-lg font-semibold text-gray-100">{selectedAccount.name}</h2>
						<span class="text-sm capitalize text-gray-600"
							>{selectedAccount.account_type} &middot; {selectedAccount.currency}</span
						>
					</div>
					<div class="text-right">
						<span
							class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
							>Cleared balance</span
						>
						<span
							class="text-2xl font-semibold {clearedBalance < 0
								? 'text-red-400'
								: 'text-emerald-400'}"
						>
							{formatAmount(clearedBalance)}
						</span>
					</div>
				</header>

				{#if transactions.length === 0}
					<div class="flex min-h-64 items-center justify-center text-gray-600">
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
										class="px-3 py-2.5 text-right text-sm font-medium tabular-nums {txn.amount <
										0
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
				<div class="flex min-h-64 items-center justify-center text-gray-600">
					Select or create an account to get started
				</div>
			{/if}
		</main>
	</div>
{/if}
