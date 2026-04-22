<script lang="ts">
	import { onMount } from 'svelte';
	import * as api from '$lib/api.svelte';
	import { currentQuarter, type Quarter } from '$lib/period';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import QuarterSelector from '$lib/components/QuarterSelector.svelte';
	import IncomeView from '$lib/components/IncomeView.svelte';
	import CashFlowView from '$lib/components/CashFlowView.svelte';
	import BalanceSheetView from '$lib/components/BalanceSheetView.svelte';
	import EntriesView from '$lib/components/EntriesView.svelte';
	import ImportView from '$lib/components/ImportView.svelte';

	type View = 'Income Statement' | 'Balance Sheet' | 'Cash Flow' | 'Entries' | 'Settings';

	let activeView: View = $state('Income Statement');
	let quarter: Quarter = $state(currentQuarter());
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
			workspaceId = created.workspace;
		} else {
			workspaceId = wsData.workspaces[0].workspace;
		}
	}
</script>

{#if loading}
	<div class="flex h-screen items-center justify-center bg-gray-950 text-gray-500">
		Loading...
	</div>
{:else if error}
	<div class="flex h-screen items-center justify-center bg-gray-950">
		<div class="rounded-xl border border-red-900 bg-red-950/50 px-6 py-4 text-sm text-red-400">
			{error}
		</div>
	</div>
{:else}
	<div class="flex h-screen overflow-hidden bg-gray-950">
		<Sidebar bind:activeView />

		<div class="flex flex-1 flex-col overflow-hidden">
			<!-- Top bar -->
			<div class="flex shrink-0 items-center justify-between border-b border-gray-800 px-4 py-2">
				<h2 class="text-sm font-medium text-gray-200">{activeView}</h2>
				{#if activeView !== 'Entries' && activeView !== 'Settings'}
					<QuarterSelector bind:quarter />
				{/if}
			</div>

			<!-- Content -->
			<main class="flex-1 overflow-auto p-4">
				{#if activeView === 'Income Statement'}
					<IncomeView {workspaceId} />
				{:else if activeView === 'Cash Flow'}
					<CashFlowView {workspaceId} />
				{:else if activeView === 'Balance Sheet'}
					<BalanceSheetView {workspaceId} />
				{:else if activeView === 'Entries'}
					<EntriesView {workspaceId} />
				{:else if activeView === 'Settings'}
					<ImportView {workspaceId} />
				{/if}
			</main>
		</div>
	</div>
{/if}
