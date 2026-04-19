<script lang="ts">
	import { onMount } from 'svelte';
	import * as api from '$lib/api.svelte';
	import IncomeView from '$lib/components/IncomeView.svelte';
	import CashFlowView from '$lib/components/CashFlowView.svelte';
	import BalanceSheetView from '$lib/components/BalanceSheetView.svelte';
	import TransactionsView from '$lib/components/TransactionsView.svelte';
	import ImportView from '$lib/components/ImportView.svelte';

	const tabs = ['Income', 'Cash Flow', 'Balance Sheet', 'Transactions', 'Import'] as const;
	type Tab = (typeof tabs)[number];

	let activeTab: Tab = $state('Income');
	let workspaceId = $state('');
	let loading = $state(true);
	let error = $state('');

	onMount(async () => {
		try {
			await ensureAuth();
			await ensureWorkspace();
		} catch (e: unknown) {
			const err = e as { message?: string };
			error = err.message || 'Failed to connect to backend';
		} finally {
			loading = false;
		}
	});

	async function ensureAuth() {
		if (api.getToken()) {
			try {
				await api.listWorkspaces();
				return;
			} catch {
				api.setToken(null);
			}
		}
		try {
			const data = await api.register('dev@finapp.local', 'Dev User', 'dev');
			api.setToken(data.token);
		} catch {
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
		<!-- Tab sidebar -->
		<aside class="flex w-52 shrink-0 flex-col border-r border-gray-800 bg-gray-900">
			<div class="border-b border-gray-800 px-5 py-4">
				<h1 class="text-base font-semibold text-gray-100">FinApp</h1>
			</div>

			<nav class="flex flex-col py-2">
				{#each tabs as tab}
					<button
						class="px-5 py-2.5 text-left text-sm transition-colors {activeTab === tab
							? 'border-r-2 border-blue-500 bg-blue-950/30 font-medium text-white'
							: 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-300'}"
						onclick={() => (activeTab = tab)}
					>
						{tab}
					</button>
				{/each}
			</nav>
		</aside>

		<!-- Main content -->
		<main class="flex-1 overflow-x-auto p-6">
			{#if activeTab === 'Income'}
				<IncomeView {workspaceId} />
			{:else if activeTab === 'Cash Flow'}
				<CashFlowView {workspaceId} />
			{:else if activeTab === 'Balance Sheet'}
				<BalanceSheetView {workspaceId} />
			{:else if activeTab === 'Transactions'}
				<TransactionsView {workspaceId} />
			{:else if activeTab === 'Import'}
				<ImportView {workspaceId} />
			{/if}
		</main>
	</div>
{/if}
