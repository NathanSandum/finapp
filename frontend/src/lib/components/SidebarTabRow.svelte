<script lang="ts">
	import TabIcon from './TabIcon.svelte';
	import type { Tab } from '$lib/tabs/types';

	let {
		tab,
		active = false,
		blank = false,
		fillDelay = 0,
		onclick
	}: {
		tab: Tab;
		active?: boolean;
		blank?: boolean;
		fillDelay?: number;
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
	<span
		class="fill-target inline-flex shrink-0"
		class:filled={!blank}
		style:--fill-delay="{fillDelay}ms"
	>
		<TabIcon id={tab.id} class="h-4 w-4" />
	</span>
	<span
		class="fill-target"
		class:filled={!blank}
		style:--fill-delay="{fillDelay}ms"
	>{tab.label}</span>
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
