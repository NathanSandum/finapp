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
	<div class="mode-pill flex gap-1 rounded-full p-1 text-xs">
		{#each items as item}
			<button
				onclick={() => onSelect(item.id)}
				class="mode-pill__btn relative flex items-center gap-1.5 rounded-full px-4 py-1.5 transition-colors duration-300 {mode ===
				item.id
					? 'mode-pill__btn--active font-semibold text-ink'
					: 'text-ink-muted hover:text-ink-body'}"
			>
				<span aria-hidden="true" class="relative font-mono text-[10px]">{item.glyph}</span>
				<span class="relative">{item.label}</span>
			</button>
		{/each}
	</div>
</div>

<style>
	.mode-pill {
		background: oklch(100% 0 0 / 0.4);
		backdrop-filter: blur(12px);
		-webkit-backdrop-filter: blur(12px);
		box-shadow: inset 0 0 0 1px oklch(100% 0 0 / 0.55);
	}
	.mode-pill__btn--active::before {
		content: '';
		position: absolute;
		inset: 0;
		border-radius: 9999px;
		background: linear-gradient(
			135deg,
			oklch(75% 0.14 268 / 0.22) 0%,
			oklch(100% 0 0 / 0.85) 100%
		);
		box-shadow:
			inset 0 0 0 1px oklch(58% 0.2 268 / 0.35),
			0 4px 14px -6px oklch(58% 0.2 268 / 0.28);
	}
</style>
