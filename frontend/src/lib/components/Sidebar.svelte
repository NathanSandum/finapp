<script lang="ts">
	import { tabs } from '$lib/tabs/registry';
	import { tabState } from '$lib/tabs/state.svelte';
	import { workspaceState } from '$lib/workspace/state.svelte';
	import WorkspaceSwitcher from '$lib/workspace/WorkspaceSwitcher.svelte';
	import { authState, initials } from '$lib/api/state.svelte';
	import UserMenu from './UserMenu.svelte';
	import SidebarWorkspacePill from './SidebarWorkspacePill.svelte';
	import SidebarTabRow from './SidebarTabRow.svelte';
	import SidebarUserPill from './SidebarUserPill.svelte';
	import SidebarDate from './SidebarDate.svelte';
	import SidebarDiscreteToggle from './SidebarDiscreteToggle.svelte';

	let switcherOpen = $state(false);
	let userMenuOpen = $state(false);

	const userBlank = $derived(authState.user === null);
	const workspaceBlank = $derived(workspaceState.active === null);
	const current = $derived(tabs.find((t) => t.id === tabState.get()) ?? tabs[0]);

	let navEl = $state<HTMLElement | undefined>(undefined);
	const rowEls: (HTMLButtonElement | undefined)[] = $state(Array(tabs.length).fill(undefined));
	let indicator = $state({ top: 0, height: 0 });
	let indicatorReady = $state(false);

	$effect(() => {
		const id = current.id;
		void userBlank;
		if (!navEl) return;
		const idx = tabs.findIndex((t) => t.id === id);
		const el = rowEls[idx];
		if (!el) return;
		const navRect = navEl.getBoundingClientRect();
		const r = el.getBoundingClientRect();
		indicator = { top: r.top - navRect.top, height: r.height };
		if (!indicatorReady) {
			requestAnimationFrame(() => requestAnimationFrame(() => (indicatorReady = true)));
		}
	});

	const activeName = $derived(workspaceState.active?.name ?? '');
	const initial = $derived(activeName.charAt(0).toUpperCase() || '');

	const displayName = $derived(authState.user?.display_name ?? '');
	const userInitials = $derived(displayName ? initials(displayName) : '?');
</script>

<div class="relative">
	<SidebarWorkspacePill
		{initial}
		name={activeName}
		open={switcherOpen}
		blank={workspaceBlank}
		fillDelay={80}
		onclick={() => (switcherOpen = !switcherOpen)}
	/>
	{#if switcherOpen}
		<WorkspaceSwitcher onClose={() => (switcherOpen = false)} />
	{/if}
</div>
<SidebarDate />
<nav bind:this={navEl} class="relative flex flex-col gap-0.5">
	<span
		class="tab-indicator pointer-events-none absolute -left-1 w-0.5 rounded-r bg-brand-soft"
		class:ready={indicatorReady && !userBlank}
		style:transform="translateY({indicator.top + 4}px)"
		style:height="{Math.max(0, indicator.height - 8)}px"
		aria-hidden="true"
	></span>
	{#each tabs as tab, i (tab.id)}
		{@const groupChanged = i > 0 && tab.group !== tabs[i - 1].group}
		{#if groupChanged}
			<div
				class="mx-3 my-1 h-px bg-gradient-to-r from-transparent via-border-faint/50 to-transparent"
				aria-hidden="true"
			></div>
		{/if}
		<SidebarTabRow
			{tab}
			bind:el={rowEls[i]}
			active={current.id === tab.id}
			blank={userBlank}
			fillDelay={160 + i * 50}
			onclick={() => tabState.set(tab.id)}
		/>
	{/each}
</nav>
<div class="mt-auto">
	<SidebarDiscreteToggle blank={userBlank} fillDelay={160 + tabs.length * 50} />
</div>
<div class="relative">
	<SidebarUserPill
		{userInitials}
		{displayName}
		open={userMenuOpen}
		blank={userBlank}
		fillDelay={160 + tabs.length * 50}
		onclick={() => (userMenuOpen = !userMenuOpen)}
	/>
	{#if userMenuOpen}
		<UserMenu onClose={() => (userMenuOpen = false)} />
	{/if}
</div>

<style>
	.tab-indicator {
		top: 0;
		opacity: 0;
		transition: opacity 200ms ease;
	}
	.tab-indicator.ready {
		opacity: 1;
		transition:
			transform 320ms cubic-bezier(0.22, 1, 0.36, 1),
			height 320ms cubic-bezier(0.22, 1, 0.36, 1),
			opacity 200ms ease;
	}
	@media (prefers-reduced-motion: reduce) {
		.tab-indicator,
		.tab-indicator.ready {
			transition: opacity 0ms;
		}
	}
</style>
