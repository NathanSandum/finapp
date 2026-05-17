<script lang="ts">
	import TabIcon from './TabIcon.svelte';
	import type { Tab } from '$lib/tabs/types';

	let {
		tab,
		active = false,
		blank = false,
		fillDelay = 0,
		onclick,
		el = $bindable<HTMLButtonElement | undefined>(undefined)
	}: {
		tab: Tab;
		active?: boolean;
		blank?: boolean;
		fillDelay?: number;
		onclick?: () => void;
		el?: HTMLButtonElement;
	} = $props();

	const stateClass = $derived(
		blank
			? 'text-ink-faint'
			: active
				? 'text-ink-inverse'
				: 'text-ink-faint hover:bg-surface-inverse-hover/60 hover:text-ink-inverse'
	);
</script>

<button
	bind:this={el}
	type="button"
	onclick={blank ? undefined : onclick}
	tabindex={blank ? -1 : 0}
	aria-hidden={blank ? 'true' : undefined}
	class="relative flex items-center gap-2.5 rounded px-3 py-1.5 text-left text-sm transition-colors {stateClass}"
	class:pointer-events-none={blank}
>
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
</button>
