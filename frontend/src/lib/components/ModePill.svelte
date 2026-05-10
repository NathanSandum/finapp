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

	const items: { id: Mode; label: string; glyph: string }[] = [
		{ id: 'summary', label: 'Summary', glyph: '▤' },
		{ id: 'detail', label: 'Detail', glyph: '≡' },
		{ id: 'trend', label: 'Trend', glyph: '∿' }
	];
</script>

<div class="flex justify-center">
	<div class="flex gap-1 rounded-full bg-surface-sunken p-1 text-xs">
		{#each items as item}
			<button
				onclick={() => onSelect(item.id)}
				class="flex items-center gap-1.5 rounded-full px-4 py-1.5 {mode === item.id
					? 'bg-surface-card font-semibold shadow-sm'
					: 'text-ink-muted'}"
			>
				<span aria-hidden="true" class="font-mono text-[10px]">{item.glyph}</span>{item.label}
			</button>
		{/each}
	</div>
</div>
