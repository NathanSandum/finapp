<script lang="ts">
	import TabIcon from './TabIcon.svelte';
	import { tabs } from '$lib/tabs/registry';
	import type { Tab } from '$lib/tabs/types';
	import { todayLabel } from '$lib/views/today';
	import { workspaceState } from '$lib/workspace/state.svelte';
	import WorkspaceSwitcher from '$lib/workspace/WorkspaceSwitcher.svelte';
	import { authState, initials } from '$lib/api/state.svelte';
	import { logout } from '$lib/api/auth';
	import { cache } from '$lib/api/cache.svelte';
	import { goto } from '$app/navigation';

	let { current, onSelect }: { current: Tab; onSelect: (tab: Tab) => void } = $props();

	const dateLabel = todayLabel();
	let switcherOpen = $state(false);
	let switcherEl = $state<HTMLDivElement | null>(null);

	const activeName = $derived(workspaceState.active?.name ?? 'FinApp');
	const initial = $derived(activeName.charAt(0).toUpperCase() || 'F');

	const displayName = $derived(authState.user?.display_name ?? '');
	const userInitials = $derived(displayName ? initials(displayName) : '?');

	async function handleSignOut() {
		await logout();
		authState.clear();
		cache.invalidateAll();
		goto('/signin');
	}

	$effect(() => {
		if (!switcherOpen) return;
		function onPointerDown(event: PointerEvent) {
			if (switcherEl && !switcherEl.contains(event.target as Node)) {
				switcherOpen = false;
			}
		}
		window.addEventListener('pointerdown', onPointerDown);
		return () => {
			window.removeEventListener('pointerdown', onPointerDown);
		};
	});
</script>

<div class="relative" bind:this={switcherEl}>
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
<div class="mt-auto flex items-center gap-2 rounded p-2">
	<div class="flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-br from-avatar-from to-avatar-to text-xs font-semibold text-ink-inverse">
		{userInitials}
	</div>
	<div class="flex flex-1 flex-col leading-tight">
		<span class="text-sm text-ink-inverse-body">{displayName}</span>
	</div>
	<button
		type="button"
		onclick={handleSignOut}
		title="Sign out"
		aria-label="Sign out"
		class="rounded p-1 text-ink-muted hover:bg-surface-inverse-hover hover:text-ink-inverse"
	>
		<svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
			<path fill-rule="evenodd" d="M3 4.75A1.75 1.75 0 014.75 3h6.5A1.75 1.75 0 0113 4.75v2a.75.75 0 01-1.5 0v-2a.25.25 0 00-.25-.25h-6.5a.25.25 0 00-.25.25v10.5c0 .138.112.25.25.25h6.5a.25.25 0 00.25-.25v-2a.75.75 0 011.5 0v2A1.75 1.75 0 0111.25 17h-6.5A1.75 1.75 0 013 15.25V4.75zm12.22 2.97a.75.75 0 011.06 0l2 2a.75.75 0 010 1.06l-2 2a.75.75 0 11-1.06-1.06l.72-.72H8.75a.75.75 0 010-1.5h7.19l-.72-.72a.75.75 0 010-1.06z" clip-rule="evenodd" />
		</svg>
	</button>
</div>
