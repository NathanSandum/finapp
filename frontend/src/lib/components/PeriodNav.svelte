<script lang="ts">
	import { prefersReducedMotion } from '$lib/utils/motion';

	let {
		label,
		onPrev,
		onNext
	}: {
		label: string;
		onPrev: () => void;
		onNext: () => void;
	} = $props();

	type Step = { key: number; before: string; after: string; direction: 1 | -1 };

	let step = $state<Step>({ key: 0, before: '', after: label, direction: 1 });
	let slid = $state(true);
	let prevLabel = label;
	let pendingDir: 1 | -1 = 1;
	let nextKey = 1;

	$effect(() => {
		const curr = label;
		if (curr === prevLabel) return;
		const reduce = prefersReducedMotion();
		step = { key: nextKey++, before: prevLabel, after: curr, direction: pendingDir };
		prevLabel = curr;
		if (reduce) {
			slid = true;
		} else {
			slid = false;
			requestAnimationFrame(() => requestAnimationFrame(() => (slid = true)));
		}
	});

	function handlePrev() {
		pendingDir = 1;
		onPrev();
	}
	function handleNext() {
		pendingDir = -1;
		onNext();
	}
</script>

<div class="flex items-center gap-1 text-xs text-ink-muted">
	<button
		onclick={handlePrev}
		class="rounded px-1.5 py-0.5 hover:bg-surface-hover"
		aria-label="Previous period">‹</button
	>
	<span class="reel min-w-[110px]" aria-label={label}>
		{#key step.key}
			<span class="track" class:slid class:back={step.direction === 1}>
				{#if step.direction === 1}
					<span class="cell">{step.after}</span>
					<span class="cell">{step.before}</span>
				{:else}
					<span class="cell">{step.before}</span>
					<span class="cell">{step.after}</span>
				{/if}
			</span>
		{/key}
	</span>
	<button
		onclick={handleNext}
		class="rounded px-1.5 py-0.5 hover:bg-surface-hover"
		aria-label="Next period">›</button
	>
</div>

<style>
	.reel {
		display: inline-block;
		overflow: hidden;
		vertical-align: middle;
		position: relative;
		height: 1.2em;
	}
	.track {
		display: flex;
		flex-direction: row;
		width: 200%;
		transition: transform 320ms cubic-bezier(0.7, 0, 0.25, 1);
	}
	.track.slid {
		transform: translateX(-50%);
	}
	.track.back {
		transform: translateX(-50%);
	}
	.track.back.slid {
		transform: translateX(0);
	}
	.cell {
		display: block;
		width: 50%;
		text-align: center;
		white-space: nowrap;
	}
	@media (prefers-reduced-motion: reduce) {
		.track {
			transition: transform 0ms;
		}
	}
</style>
