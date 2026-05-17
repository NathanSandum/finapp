<script lang="ts">
	import { tabs } from '$lib/tabs/registry';
	import type { Tab } from '$lib/tabs/types';
	import { workspaceState } from '$lib/workspace/state.svelte';
	import WorkspaceSwitcher from '$lib/workspace/WorkspaceSwitcher.svelte';
	import { authState, initials } from '$lib/api/state.svelte';
	import UserMenu from './UserMenu.svelte';
	import SidebarWorkspacePill from './SidebarWorkspacePill.svelte';
	import SidebarTabRow from './SidebarTabRow.svelte';
	import SidebarUserPill from './SidebarUserPill.svelte';
	import SidebarDate from './SidebarDate.svelte';

	let { current, onSelect }: { current: Tab; onSelect: (tab: Tab) => void } = $props();

	let switcherOpen = $state(false);
	let userMenuOpen = $state(false);

	const activeName = $derived(workspaceState.active?.name ?? 'FinApp');
	const initial = $derived(activeName.charAt(0).toUpperCase() || 'F');

	const displayName = $derived(authState.user?.display_name ?? '');
	const userInitials = $derived(displayName ? initials(displayName) : '?');
</script>

<div class="relative">
	<SidebarWorkspacePill
		{initial}
		name={activeName}
		open={switcherOpen}
		onclick={() => (switcherOpen = !switcherOpen)}
	/>
	{#if switcherOpen}
		<WorkspaceSwitcher onClose={() => (switcherOpen = false)} />
	{/if}
</div>
<SidebarDate />
<nav class="flex flex-col gap-0.5">
	{#each tabs as tab (tab.id)}
		<SidebarTabRow {tab} active={current.id === tab.id} onclick={() => onSelect(tab)} />
	{/each}
</nav>
<div class="relative mt-auto">
	<SidebarUserPill
		{userInitials}
		{displayName}
		open={userMenuOpen}
		onclick={() => (userMenuOpen = !userMenuOpen)}
	/>
	{#if userMenuOpen}
		<UserMenu onClose={() => (userMenuOpen = false)} />
	{/if}
</div>
