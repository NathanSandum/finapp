<script lang="ts">
	import TabPanel from '$lib/components/TabPanel.svelte';
	import ShellHeader from '$lib/components/ShellHeader.svelte';
	import { tabs } from '$lib/tabs/registry';
	import { tabState } from '$lib/tabs/state.svelte';
	import { workspaceState } from '$lib/workspace/state.svelte';
	import { bootWorkspace, bootState } from '$lib/workspace/boot.svelte';
	import { fetchMe } from '$lib/auth/session';
	import { goto } from '$app/navigation';

	const current = $derived(tabs.find((t) => t.id === tabState.get()) ?? tabs[0]);
	const booted = $derived(bootState.ready);

	let bootError = $state<string | null>(null);

	$effect(() => {
		if (bootState.ready) return;
		(async () => {
			const me = await fetchMe();
			if (me.status !== 'success') {
				goto('/signin');
				return;
			}
			const result = await bootWorkspace();
			if (result.status === 'fail') {
				bootError =
					typeof result.data.message === 'string' ? result.data.message : 'Failed to load workspace';
			} else if (result.status === 'error') {
				bootError = result.message;
			}
		})();
	});
</script>

<main class="flex flex-col overflow-hidden">
	{#if booted}
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
