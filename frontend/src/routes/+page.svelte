<script lang="ts">
	import AppShell from '$lib/components/AppShell.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import TabPanel from '$lib/components/TabPanel.svelte';
	import { tabs } from '$lib/tabs/registry';
	import { page } from '$app/state';

	const initialTab = (() => {
		const id = page.url.searchParams.get('tab');
		return tabs.find((t) => t.id === id) ?? tabs[0];
	})();
	let current = $state(initialTab);
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
