<script lang="ts">
	import { discreteState } from '$lib/ui/discrete.svelte';

	let {
		blank = false,
		fillDelay = 0
	}: {
		blank?: boolean;
		fillDelay?: number;
	} = $props();

	const on = $derived(discreteState.on);
</script>

<button
	type="button"
	onclick={blank ? undefined : () => discreteState.toggle()}
	tabindex={blank ? -1 : 0}
	aria-hidden={blank ? 'true' : undefined}
	aria-pressed={blank ? undefined : on}
	class="flex items-center gap-2.5 rounded px-3 py-1.5 text-left text-sm transition-colors {blank
		? 'text-ink-faint'
		: on
			? 'text-ink-inverse'
			: 'text-ink-faint hover:bg-surface-inverse-hover/60 hover:text-ink-inverse'}"
	class:pointer-events-none={blank}
>
	<span
		class="fill-target inline-flex shrink-0"
		class:filled={!blank}
		style:--fill-delay="{fillDelay}ms"
	>
		<svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
			{#if on}
				<path d="M2.5 10c1.5-2.5 4.3-5 7.5-5 1.4 0 2.7.5 3.8 1.2L4.3 16.5 3 15.2zM17.5 10c-1.5 2.5-4.3 5-7.5 5-1 0-2-.2-2.9-.6l9.7-9.7c.2.4.5.8.7 1.3z" />
			{:else}
				<path d="M10 5c-4.2 0-7 4-7.8 5C3 11 5.8 15 10 15s7-4 7.8-5C17 9 14.2 5 10 5zm0 8a3 3 0 110-6 3 3 0 010 6z" />
			{/if}
		</svg>
	</span>
	<span
		class="fill-target flex-1"
		class:filled={!blank}
		style:--fill-delay="{fillDelay}ms"
	>Discrete</span>
	<span
		class="fill-target relative inline-flex h-5 w-9 shrink-0 items-center rounded-full transition-colors {on
			? 'bg-brand-deep'
			: 'bg-ink-faint/25'}"
		class:filled={!blank}
		style:--fill-delay="{fillDelay}ms"
	>
		<span
			class="absolute h-4 w-4 rounded-full bg-white shadow-sm transition-transform"
			style:transform={on ? 'translateX(18px)' : 'translateX(2px)'}
		></span>
	</span>
</button>
