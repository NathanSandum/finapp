<script lang="ts">
	import { prefersReducedMotion } from '$lib/utils/motion';
	import type { Granularity } from '$lib/views/period';

	let {
		label,
		granularity,
		onPrev,
		onNext
	}: {
		label: string;
		granularity: Granularity;
		onPrev: () => void;
		onNext: () => void;
	} = $props();

	type Axis = 'x' | 'y';
	type Step = { key: number; before: string; after: string; axis: Axis; direction: 1 | -1 };

	const rankOf = (g: Granularity) => (g === 'month' ? 0 : g === 'quarter' ? 1 : 2);

	let step = $state<Step>({ key: 0, before: '', after: label, axis: 'x', direction: 1 });
	let settled = $state(true);
	let prevLabel = label;
	let prevGranularity = granularity;
	let pendingDir: 1 | -1 = 1;
	let nextKey = 1;

	$effect(() => {
		const curr = label;
		const currGran = granularity;
		if (curr === prevLabel) return;

		let axis: Axis;
		let direction: 1 | -1;
		if (currGran !== prevGranularity) {
			axis = 'y';
			direction = rankOf(currGran) > rankOf(prevGranularity) ? -1 : 1;
		} else {
			axis = 'x';
			direction = pendingDir;
		}

		const reduce = prefersReducedMotion();
		step = { key: nextKey++, before: prevLabel, after: curr, axis, direction };
		prevLabel = curr;
		prevGranularity = currGran;
		if (reduce) {
			settled = true;
		} else {
			settled = false;
			requestAnimationFrame(() => requestAnimationFrame(() => (settled = true)));
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
	<span class="reel min-w-[110px]" class:vertical={step.axis === 'y'} aria-label={label}>
		{#key step.key}
			<span
				class="track"
				class:settled
				class:back={step.direction === 1}
				class:vertical={step.axis === 'y'}
			>
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
	.track.vertical {
		flex-direction: column;
		width: 100%;
		height: 200%;
	}
	.track.settled {
		transform: translateX(-50%);
	}
	.track.back {
		transform: translateX(-50%);
	}
	.track.back.settled {
		transform: translateX(0);
	}
	.track.vertical.settled {
		transform: translateY(-50%);
	}
	.track.vertical.back {
		transform: translateY(-50%);
	}
	.track.vertical.back.settled {
		transform: translateY(0);
	}
	.cell {
		display: block;
		width: 50%;
		text-align: center;
		white-space: nowrap;
	}
	.track.vertical .cell {
		width: 100%;
		height: 50%;
		line-height: 1.2em;
	}
	@media (prefers-reduced-motion: reduce) {
		.track {
			transition: transform 0ms;
		}
	}
</style>
