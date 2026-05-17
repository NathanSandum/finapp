<script lang="ts">
	import TabIcon from './TabIcon.svelte';
	import { tabs } from '$lib/tabs/registry';
	import type { Tab } from '$lib/tabs/types';
	import { todayLabel } from '$lib/views/today';
	import { workspaceState } from '$lib/workspace/state.svelte';
	import WorkspaceSwitcher from '$lib/workspace/WorkspaceSwitcher.svelte';
	import { authState, initials } from '$lib/api/state.svelte';
	import UserMenu from './UserMenu.svelte';

	let { current, onSelect }: { current: Tab; onSelect: (tab: Tab) => void } = $props();

	const dateLabel = todayLabel();
	let switcherOpen = $state(false);
	let userMenuOpen = $state(false);

	const activeName = $derived(workspaceState.active?.name ?? 'FinApp');
	const initial = $derived(activeName.charAt(0).toUpperCase() || 'F');

	const displayName = $derived(authState.user?.display_name ?? '');
	const userInitials = $derived(displayName ? initials(displayName) : '?');
</script>

<div class="relative">
	<button
		type="button"
		onclick={() => (switcherOpen = !switcherOpen)}
		class="flex w-full items-center justify-between rounded px-2 py-1.5 text-left transition-colors {switcherOpen
			? 'bg-surface-inverse-hover'
			: 'hover:bg-surface-inverse-hover'}"
	>
		<span class="flex items-center gap-2">
			<span class="flex h-6 w-6 items-center justify-center rounded bg-brand text-xs font-bold text-ink">
				{initial}
			</span>
			<span class="text-sm font-medium text-ink-inverse">{activeName}</span>
		</span>
		<svg
			class="h-4 w-4 text-ink-muted transition-transform {switcherOpen ? 'rotate-180' : ''}"
			viewBox="0 0 20 20"
			fill="currentColor"
			aria-hidden="true"
		>
			<path fill-rule="evenodd" d="M5 8l5 5 5-5H5z" clip-rule="evenodd" />
		</svg>
	</button>
	{#if switcherOpen}
		<WorkspaceSwitcher onClose={() => (switcherOpen = false)} />
	{/if}
</div>
<p class="mb-4 px-2 pt-0.5 font-mono text-[10px] uppercase tracking-[0.1em] text-ink-muted">
	{dateLabel}
</p>
<nav class="flex flex-col gap-0.5">
	{#each tabs as tab (tab.id)}
		<button
			type="button"
			onclick={() => onSelect(tab)}
			class="relative flex items-center gap-2.5 rounded px-3 py-1.5 text-left text-sm transition-colors {current.id === tab.id
				? 'text-ink-inverse'
				: 'text-ink-faint hover:bg-surface-inverse-hover/60 hover:text-ink-inverse'}"
		>
			{#if current.id === tab.id}
				<span class="absolute inset-y-1 -left-1 w-0.5 rounded-r bg-brand-soft"></span>
			{/if}
			<TabIcon id={tab.id} class="h-4 w-4 shrink-0" />
			{tab.label}
		</button>
	{/each}
</nav>
<div class="relative mt-auto">
	<button
		type="button"
		onclick={() => (userMenuOpen = !userMenuOpen)}
		aria-haspopup="menu"
		aria-expanded={userMenuOpen}
		class="flex w-full items-center gap-2 rounded p-2 text-left transition-colors hover:bg-surface-inverse-hover {userMenuOpen ? 'bg-surface-inverse-hover' : ''}"
	>
		<div class="flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-br from-avatar-from to-avatar-to text-xs font-semibold text-ink-inverse">
			{userInitials}
		</div>
		<div class="flex flex-1 flex-col leading-tight">
			<span class="text-sm text-ink-inverse-body">{displayName}</span>
		</div>
		<span class="text-ink-muted {userMenuOpen ? 'text-ink-inverse' : ''}">
			<svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
				<path d="M10 6a1.5 1.5 0 110-3 1.5 1.5 0 010 3zm0 5.5a1.5 1.5 0 110-3 1.5 1.5 0 010 3zm0 5.5a1.5 1.5 0 110-3 1.5 1.5 0 010 3z" />
			</svg>
		</span>
	</button>
	{#if userMenuOpen}
		<UserMenu onClose={() => (userMenuOpen = false)} />
	{/if}
</div>
