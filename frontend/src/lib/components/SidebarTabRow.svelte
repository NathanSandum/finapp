<script lang="ts">
	import TabIcon from './TabIcon.svelte';
	import type { Tab } from '$lib/tabs/types';

	let {
		tab,
		active = false,
		blank = false,
		onclick
	}: {
		tab: Tab;
		active?: boolean;
		blank?: boolean;
		onclick?: () => void;
	} = $props();

	const stateClass = $derived(
		active
			? 'text-ink-inverse'
			: 'text-ink-faint hover:bg-surface-inverse-hover/60 hover:text-ink-inverse'
	);
</script>

{#snippet body()}
	{#if active && !blank}
		<span class="absolute inset-y-1 -left-1 w-0.5 rounded-r bg-brand-soft"></span>
	{/if}
	<TabIcon id={tab.id} class="h-4 w-4 shrink-0 {blank ? 'invisible' : ''}" />
	<span class={blank ? 'invisible' : ''}>{tab.label}</span>
{/snippet}

{#if blank}
	<div class="relative flex items-center gap-2.5 rounded px-3 py-1.5 text-sm text-ink-faint">
		{@render body()}
	</div>
{:else}
	<button
		type="button"
		{onclick}
		class="relative flex items-center gap-2.5 rounded px-3 py-1.5 text-left text-sm transition-colors {stateClass}"
	>
		{@render body()}
	</button>
{/if}
