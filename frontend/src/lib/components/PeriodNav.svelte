<script lang="ts">
	let {
		label,
		onPrev,
		onNext
	}: {
		label: string;
		onPrev: () => void;
		onNext: () => void;
	} = $props();
</script>

<div class="flex items-center gap-1 text-xs text-ink-muted">
	<button
		onclick={onPrev}
		class="rounded px-1.5 py-0.5 hover:bg-surface-hover"
		aria-label="Previous period">‹</button
	>
	<span class="min-w-[110px] text-center">{label}</span>
	<button
		onclick={onNext}
		class="rounded px-1.5 py-0.5 hover:bg-surface-hover"
		aria-label="Next period">›</button
	>
</div>
