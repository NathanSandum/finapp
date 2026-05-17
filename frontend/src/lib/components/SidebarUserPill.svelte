<script lang="ts">
	let {
		userInitials = '',
		displayName = '',
		open = false,
		blank = false,
		onclick
	}: {
		userInitials?: string;
		displayName?: string;
		open?: boolean;
		blank?: boolean;
		onclick?: () => void;
	} = $props();
</script>

<svelte:element
	this={blank ? 'div' : 'button'}
	type={blank ? undefined : 'button'}
	{onclick}
	aria-haspopup={blank ? undefined : 'menu'}
	aria-expanded={blank ? undefined : open}
	class="flex w-full items-center gap-2 rounded p-2 {blank
		? ''
		: 'text-left transition-colors hover:bg-surface-inverse-hover'} {!blank && open
		? 'bg-surface-inverse-hover'
		: ''}"
>
	<div
		class="flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-br from-avatar-from to-avatar-to text-xs font-semibold text-ink-inverse {blank
			? 'invisible'
			: ''}"
	>
		{userInitials}
	</div>
	<div class="flex flex-1 flex-col leading-tight {blank ? 'invisible' : ''}">
		<span class="text-sm text-ink-inverse-body">{displayName || '\u00a0'}</span>
	</div>
	<span class="text-ink-muted {blank ? 'invisible' : ''} {!blank && open ? 'text-ink-inverse' : ''}">
		<svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
			<path
				d="M10 6a1.5 1.5 0 110-3 1.5 1.5 0 010 3zm0 5.5a1.5 1.5 0 110-3 1.5 1.5 0 010 3zm0 5.5a1.5 1.5 0 110-3 1.5 1.5 0 010 3z"
			/>
		</svg>
	</span>
</svelte:element>
