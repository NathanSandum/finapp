<script lang="ts">
	let {
		userInitials = '',
		displayName = '',
		open = false,
		blank = false,
		fillDelay = 0,
		onclick
	}: {
		userInitials?: string;
		displayName?: string;
		open?: boolean;
		blank?: boolean;
		fillDelay?: number;
		onclick?: () => void;
	} = $props();
</script>

{#snippet body()}
	<div
		class="fill-target flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-br from-avatar-from to-avatar-to text-xs font-semibold text-ink-inverse"
		class:filled={!blank}
		style:--fill-delay="{fillDelay}ms"
	>
		{userInitials}
	</div>
	<div
		class="fill-target flex flex-1 flex-col leading-tight"
		class:filled={!blank}
		style:--fill-delay="{fillDelay}ms"
	>
		<span class="text-sm text-ink-inverse-body">{displayName || '\u00a0'}</span>
	</div>
	<span
		class="fill-target text-ink-muted {!blank && open ? 'text-ink-inverse' : ''}"
		class:filled={!blank}
		style:--fill-delay="{fillDelay}ms"
	>
		<svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
			<path
				d="M10 6a1.5 1.5 0 110-3 1.5 1.5 0 010 3zm0 5.5a1.5 1.5 0 110-3 1.5 1.5 0 010 3zm0 5.5a1.5 1.5 0 110-3 1.5 1.5 0 010 3z"
			/>
		</svg>
	</span>
{/snippet}

{#if blank}
	<div class="flex w-full items-center gap-2 rounded p-2">
		{@render body()}
	</div>
{:else}
	<button
		type="button"
		{onclick}
		aria-haspopup="menu"
		aria-expanded={open}
		class="flex w-full items-center gap-2 rounded p-2 text-left transition-colors hover:bg-surface-inverse-hover {open
			? 'bg-surface-inverse-hover'
			: ''}"
	>
		{@render body()}
	</button>
{/if}
