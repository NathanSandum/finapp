<script lang="ts">
	import type { Granularity } from '$lib/views/period';

	let {
		label,
		isAtCurrent,
		onPrev,
		onNext,
		onGoToCurrent
	}: {
		label: string;
		granularity: Granularity;
		year: number;
		month: number;
		isAtCurrent: boolean;
		onPrev: () => void;
		onNext: () => void;
		onGoToCurrent: () => void;
	} = $props();
</script>

<div class="flex items-center gap-2 text-xs">
	<button
		onclick={onPrev}
		class="px-1.5 leading-none text-ink-muted hover:text-ink"
		aria-label="Previous period">‹</button
	>
	<span class="min-w-[120px] text-center font-display text-sm tracking-tight text-ink">
		{label}
	</span>
	<button
		onclick={onNext}
		class="px-1.5 leading-none text-ink-muted hover:text-ink"
		aria-label="Next period">›</button
	>
	{#if !isAtCurrent}
		<button
			onclick={onGoToCurrent}
			aria-label="Go to current"
			title="Go to current"
			class="ml-1 text-[10px] uppercase tracking-[0.15em] text-ink-muted underline-offset-4 hover:text-ink hover:underline"
		>
			Today
		</button>
	{/if}
</div>
