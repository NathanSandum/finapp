<script lang="ts">
	import AppShell from '$lib/components/AppShell.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import TabPanel from '$lib/components/TabPanel.svelte';
	import { tabs } from '$lib/tabs/registry';
	import { tabState } from '$lib/tabs/state.svelte';

	const current = $derived(tabs.find((t) => t.id === tabState.get()) ?? tabs[0]);
</script>

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
