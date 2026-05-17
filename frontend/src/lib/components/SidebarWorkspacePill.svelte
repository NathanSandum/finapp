<script lang="ts">
	let {
		initial = '',
		name = '',
		open = false,
		blank = false,
		onclick
	}: {
		initial?: string;
		name?: string;
		open?: boolean;
		blank?: boolean;
		onclick?: () => void;
	} = $props();
</script>

<svelte:element
	this={blank ? 'div' : 'button'}
	type={blank ? undefined : 'button'}
	{onclick}
	class="flex w-full items-center justify-between rounded px-2 py-1.5 {blank
		? ''
		: 'text-left transition-colors'} {!blank && (open ? 'bg-surface-inverse-hover' : 'hover:bg-surface-inverse-hover')}"
>
	<span class="flex items-center gap-2">
		<span
			class="flex h-6 w-6 items-center justify-center rounded bg-brand text-xs font-bold text-ink {blank
				? 'invisible'
				: ''}"
		>
			{initial}
		</span>
		<span class="text-sm font-medium text-ink-inverse {blank ? 'invisible' : ''}">{name}</span>
	</span>
	<svg
		class="h-4 w-4 text-ink-muted transition-transform {blank ? 'invisible' : ''} {open
			? 'rotate-180'
			: ''}"
		viewBox="0 0 20 20"
		fill="currentColor"
		aria-hidden="true"
	>
		<path fill-rule="evenodd" d="M5 8l5 5 5-5H5z" clip-rule="evenodd" />
	</svg>
</svelte:element>
