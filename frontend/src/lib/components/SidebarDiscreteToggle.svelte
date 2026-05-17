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
	class="relative flex items-center gap-2.5 rounded px-3 py-1.5 text-left text-sm transition-colors {blank
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
		<svg class="h-4 w-4" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
			{#if on}
				<path d="M2 10s3-6 8-6c2 0 3.6.96 4.9 2.1M18 10s-3 6-8 6c-2 0-3.6-.96-4.9-2.1" stroke-linecap="round" />
				<path d="M3 3l14 14" stroke-linecap="round" />
			{:else}
				<path d="M2 10s3-6 8-6 8 6 8 6-3 6-8 6-8-6-8-6z" />
				<circle cx="10" cy="10" r="2.5" />
			{/if}
		</svg>
	</span>
	<span
		class="fill-target flex-1"
		class:filled={!blank}
		style:--fill-delay="{fillDelay}ms"
	>Discrete</span>
	<span
		class="fill-target h-3.5 w-6 shrink-0 rounded-full border border-current p-0.5 transition-colors"
		class:bg-brand-soft={!blank && on}
		class:filled={!blank}
		style:--fill-delay="{fillDelay}ms"
	>
		<span
			class="block h-full w-1/2 rounded-full bg-current transition-transform"
			style:transform={on ? 'translateX(100%)' : 'translateX(0)'}
		></span>
	</span>
</button>
