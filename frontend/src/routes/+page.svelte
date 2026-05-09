<script lang="ts">
	import AppShell from '$lib/components/AppShell.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import TabPanel from '$lib/components/TabPanel.svelte';
	import { tabs } from '$lib/tabs/registry';

	let current = $state(tabs[0]);
</script>

<AppShell>
	{#snippet sidebar()}
		<Sidebar {current} onSelect={(tab) => (current = tab)} />
	{/snippet}

	{#snippet main()}
		{#each tabs as tab (tab.id)}
			<TabPanel {tab} active={current.id === tab.id} />
		{/each}
	{/snippet}
</AppShell>
