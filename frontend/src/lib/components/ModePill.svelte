<script lang="ts" module>
	export type Mode = 'summary' | 'detail' | 'trend';
</script>

<script lang="ts">
	let {
		mode,
		onSelect
	}: {
		mode: Mode;
		onSelect: (next: Mode) => void;
	} = $props();

	const items: { id: Mode; label: string }[] = [
		{ id: 'summary', label: 'Summary' },
		{ id: 'detail', label: 'Detail' },
		{ id: 'trend', label: 'Trend' }
	];
</script>

<div class="flex justify-end gap-5 border-b border-border pb-1 text-[11px] uppercase tracking-[0.16em]">
	{#each items as item (item.id)}
		<button
			onclick={() => onSelect(item.id)}
			class="relative pb-1 transition-colors {mode === item.id
				? 'text-ink'
				: 'text-ink-muted hover:text-ink-body'}"
		>
			{item.label}
			{#if mode === item.id}
				<span class="absolute -bottom-[5px] left-0 right-0 h-px bg-ink"></span>
			{/if}
		</button>
	{/each}
</div>
