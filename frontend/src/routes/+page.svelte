<script lang="ts">
	import AppShell from '$lib/components/AppShell.svelte';
	import TabPanel from '$lib/components/TabPanel.svelte';
	import { tabs } from '$lib/tabs/registry';

	let current = $state(tabs[0]);
</script>

<AppShell>
	{#snippet header()}
		<h1 class="text-lg font-semibold">FinApp</h1>
	{/snippet}

	{#snippet sidebar()}
		<nav class="flex flex-col gap-1">
			{#each tabs as tab (tab.id)}
				<button
					type="button"
					onclick={() => (current = tab)}
					class="rounded px-3 py-2 text-left text-sm transition-colors {current.id === tab.id
						? 'bg-neutral-900 text-white'
						: 'hover:bg-neutral-100'}"
				>
					{tab.label}
				</button>
			{/each}
		</nav>
	{/snippet}

	{#snippet main()}
		{#each tabs as tab (tab.id)}
			<TabPanel {tab} active={current.id === tab.id} />
		{/each}
	{/snippet}
</AppShell>
