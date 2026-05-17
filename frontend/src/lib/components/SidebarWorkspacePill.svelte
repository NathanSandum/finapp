<script lang="ts">
	let {
		initial = '',
		name = '',
		open = false,
		blank = false,
		fillDelay = 0,
		onclick
	}: {
		initial?: string;
		name?: string;
		open?: boolean;
		blank?: boolean;
		fillDelay?: number;
		onclick?: () => void;
	} = $props();

	const hoverClass = $derived(
		blank ? '' : open ? 'bg-surface-inverse-hover' : 'hover:bg-surface-inverse-hover'
	);
</script>

<button
	type="button"
	onclick={blank ? undefined : onclick}
	tabindex={blank ? -1 : 0}
	aria-hidden={blank ? 'true' : undefined}
	class="flex w-full items-center justify-between rounded px-2 py-1.5 text-left transition-colors {hoverClass}"
	class:pointer-events-none={blank}
>
	<span class="flex items-center gap-2">
		<span
			class="fill-target flex h-6 w-6 items-center justify-center rounded bg-brand text-xs font-bold text-ink"
			class:filled={!blank}
			style:--fill-delay="{fillDelay}ms"
		>
			{initial}
		</span>
		<span
			class="fill-target text-sm font-medium text-ink-inverse"
			class:filled={!blank}
			style:--fill-delay="{fillDelay}ms"
		>{name}</span>
	</span>
	<svg
		class="fill-target h-4 w-4 text-ink-muted transition-transform {open ? 'rotate-180' : ''}"
		class:filled={!blank}
		style:--fill-delay="{fillDelay}ms"
		viewBox="0 0 20 20"
		fill="currentColor"
		aria-hidden="true"
	>
		<path fill-rule="evenodd" d="M5 8l5 5 5-5H5z" clip-rule="evenodd" />
	</svg>
</button>
