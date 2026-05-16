<script lang="ts">
	let { onStart }: { onStart: () => void } = $props();
</script>

<button
	type="button"
	onclick={onStart}
	class="flex h-full w-full items-center gap-2 px-4 text-left font-mono text-[10px] uppercase tracking-widest text-ink-inverse-muted transition-colors hover:bg-surface-overlay-hover hover:text-ink-inverse"
>
	<svg class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
		<path d="M10 4a1 1 0 011 1v4h4a1 1 0 110 2h-4v4a1 1 0 11-2 0v-4H5a1 1 0 110-2h4V5a1 1 0 011-1z" />
	</svg>
	<span>New workspace</span>
</button>
