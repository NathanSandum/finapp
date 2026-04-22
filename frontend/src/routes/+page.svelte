<script lang="ts">
	import { onMount } from 'svelte';
	import * as api from '$lib/api.svelte';
	import { currentQuarter, type Quarter } from '$lib/period';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import QuarterSelector from '$lib/components/QuarterSelector.svelte';
	import IncomeStatementView from '$lib/components/IncomeStatementView.svelte';

	import BalanceSheetView from '$lib/components/BalanceSheetView.svelte';
	import EntriesView from '$lib/components/EntriesView.svelte';
	import SettingsView from '$lib/components/SettingsView.svelte';

	type View = 'Income Statement' | 'Balance Sheet' | 'Cash Flow' | 'Entries' | 'Settings';

	let activeView: View = $state('Income Statement');
	let quarter: Quarter = $state(currentQuarter());
	let workspaceId = $state('');
	let loading = $state(true);
	let error = $state('');
	let entryActions: { openNewForm: () => void } | null = $state(null);

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
				<div class="flex items-center gap-2">
					{#if activeView === 'Entries'}
						<button
							class="inline-flex h-6 w-6 items-center justify-center rounded text-lg text-gray-400 hover:bg-gray-800 hover:text-blue-400"
							onclick={() => entryActions?.openNewForm()}
						>
							+
						</button>
					{/if}
					{#if activeView !== 'Settings'}
						<QuarterSelector bind:quarter />
					{/if}
				</div>
			</div>

			<!-- Content -->
			<main class="flex-1 overflow-auto">
				{#if activeView === 'Income Statement'}
					<IncomeStatementView {workspaceId} {quarter} />
				{:else if activeView === 'Cash Flow'}
					<div class="flex min-h-64 items-center justify-center text-gray-600">
						Coming soon
					</div>
				{:else if activeView === 'Balance Sheet'}
					<BalanceSheetView {workspaceId} {quarter} />
				{:else if activeView === 'Entries'}
					<EntriesView {workspaceId} {quarter} onregister={(actions) => (entryActions = actions)} />
				{:else if activeView === 'Settings'}
					<SettingsView {workspaceId} />
				{/if}
			</main>
		</div>
	</div>
{/if}
