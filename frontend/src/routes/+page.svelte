<script lang="ts">
	import TabPanel from '$lib/components/TabPanel.svelte';
	import ShellHeader from '$lib/components/ShellHeader.svelte';
	import { tabs } from '$lib/tabs/registry';
	import { tabState } from '$lib/tabs/state.svelte';
	import { workspaceState } from '$lib/workspace/state.svelte';
	import { listWorkspaces } from '$lib/workspace/api';
	import { fetchMe } from '$lib/api/auth';
	import { authState } from '$lib/api/state.svelte';
	import { goto } from '$app/navigation';

	const current = $derived(tabs.find((t) => t.id === tabState.get()) ?? tabs[0]);

	let bootError = $state<string | null>(null);

	$effect(() => {
		if (workspaceState.active) return;
		(async () => {
			const me = await fetchMe();
			if (me.status !== 'success') {
				goto('/signin');
				return;
			}
			authState.set(me.data.user);
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

<main class="flex flex-col overflow-hidden">
	{#if workspaceState.active}
		<ShellHeader />
		<div class="relative flex-1 overflow-hidden">
			{#each tabs as tab (tab.id)}
				<TabPanel {tab} active={current.id === tab.id} />
			{/each}
		</div>
	{:else if bootError}
		<p class="p-6 text-sm text-negative">{bootError}</p>
	{:else}
		<p class="p-6 text-sm text-ink-muted">Loading…</p>
	{/if}
</main>
