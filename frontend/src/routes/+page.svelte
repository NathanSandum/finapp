<script lang="ts">
	import AppShell from '$lib/components/AppShell.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import TabPanel from '$lib/components/TabPanel.svelte';
	import { tabs } from '$lib/tabs/registry';
	import { tabState } from '$lib/tabs/state.svelte';
	import { workspaceState } from '$lib/workspace/state.svelte';
	import { listWorkspaces } from '$lib/workspace/api';

	const current = $derived(tabs.find((t) => t.id === tabState.get()) ?? tabs[0]);

	let bootError = $state<string | null>(null);

	$effect(() => {
		if (workspaceState.active) return;
		(async () => {
			const result = await listWorkspaces();
			if (result.status !== 'success') {
				bootError =
					result.status === 'error' ? result.message : 'Failed to load workspaces';
				return;
			}
			workspaceState.setList(result.data.workspaces);
			const initial = workspaceState.pickInitial(result.data.workspaces);
			if (initial) workspaceState.setActive(initial);
			else bootError = 'No workspaces available';
		})();
	});
</script>

{#if workspaceState.active}
	<AppShell>
		{#snippet sidebar()}
			<Sidebar {current} onSelect={(tab) => tabState.set(tab.id)} />
		{/snippet}

		{#snippet main()}
			{#each tabs as tab (tab.id)}
				<TabPanel {tab} active={current.id === tab.id} />
			{/each}
		{/snippet}
	</AppShell>
{:else if bootError}
	<p class="p-6 text-sm text-negative">{bootError}</p>
{:else}
	<p class="p-6 text-sm text-ink-muted">Loading…</p>
{/if}
