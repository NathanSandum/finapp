<script lang="ts">
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

	type Mode = 'x-back' | 'x-fwd' | 'y-up' | 'y-down' | 'none';
	type Step = { key: number; before: string; after: string; mode: Mode };

	const rankOf = (g: Granularity) => (g === 'month' ? 0 : g === 'quarter' ? 1 : 2);

	let step = $state<Step>({ key: 0, before: '', after: label, mode: 'none' });
	let prevLabel = label;
	let prevGranularity = granularity;
	let pendingDir: 1 | -1 = 1;
	let nextKey = 1;

	$effect(() => {
		const curr = label;
		const currGran = granularity;
		if (curr === prevLabel) return;

		let mode: Mode;
		if (currGran !== prevGranularity) {
			mode = rankOf(currGran) > rankOf(prevGranularity) ? 'y-up' : 'y-down';
		} else {
			mode = pendingDir === 1 ? 'x-back' : 'x-fwd';
		}

		step = { key: nextKey++, before: prevLabel, after: curr, mode };
		prevLabel = curr;
		prevGranularity = currGran;
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
		class="rounded px-2 py-1 text-base leading-none hover:bg-surface-hover"
		aria-label="Previous period">‹</button
	>
	<span class="reel min-w-[110px]" aria-label={label}>
		{#key step.key}
			<span class="stage mode-{step.mode}">
				<span class="cell before">{step.before}</span>
				<span class="cell after">{step.after}</span>
			</span>
		{/key}
	</span>
	<button
		onclick={handleNext}
		class="rounded px-2 py-1 text-base leading-none hover:bg-surface-hover"
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
		min-width: 110px;
	}
	.stage {
		position: absolute;
		inset: 0;
	}
	.cell {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		white-space: nowrap;
		line-height: 1.2em;
	}

	/* mode-none: only show after, no animation */
	.stage.mode-none .before {
		display: none;
	}

	/* x-fwd: next click. New label enters from right, old leaves to left. */
	.stage.mode-x-fwd .before {
		animation: x-out-left 320ms cubic-bezier(0.7, 0, 0.25, 1) both;
	}
	.stage.mode-x-fwd .after {
		animation: x-in-right 320ms cubic-bezier(0.7, 0, 0.25, 1) both;
	}

	/* x-back: prev click. New label enters from left, old leaves to right. */
	.stage.mode-x-back .before {
		animation: x-out-right 320ms cubic-bezier(0.7, 0, 0.25, 1) both;
	}
	.stage.mode-x-back .after {
		animation: x-in-left 320ms cubic-bezier(0.7, 0, 0.25, 1) both;
	}

	/* y-up: bigger granularity. New label enters from bottom, old leaves to top. */
	.stage.mode-y-up .before {
		animation: y-out-up 320ms cubic-bezier(0.7, 0, 0.25, 1) both;
	}
	.stage.mode-y-up .after {
		animation: y-in-up 320ms cubic-bezier(0.7, 0, 0.25, 1) both;
	}

	/* y-down: smaller granularity. New label enters from top, old leaves to bottom. */
	.stage.mode-y-down .before {
		animation: y-out-down 320ms cubic-bezier(0.7, 0, 0.25, 1) both;
	}
	.stage.mode-y-down .after {
		animation: y-in-down 320ms cubic-bezier(0.7, 0, 0.25, 1) both;
	}

	@keyframes x-in-right {
		from { transform: translateX(100%); }
		to { transform: translateX(0); }
	}
	@keyframes x-out-left {
		from { transform: translateX(0); }
		to { transform: translateX(-100%); }
	}
	@keyframes x-in-left {
		from { transform: translateX(-100%); }
		to { transform: translateX(0); }
	}
	@keyframes x-out-right {
		from { transform: translateX(0); }
		to { transform: translateX(100%); }
	}
	@keyframes y-in-up {
		from { transform: translateY(100%); }
		to { transform: translateY(0); }
	}
	@keyframes y-out-up {
		from { transform: translateY(0); }
		to { transform: translateY(-100%); }
	}
	@keyframes y-in-down {
		from { transform: translateY(-100%); }
		to { transform: translateY(0); }
	}
	@keyframes y-out-down {
		from { transform: translateY(0); }
		to { transform: translateY(100%); }
	}

	@media (prefers-reduced-motion: reduce) {
		.cell {
			animation: none !important;
		}
	}
</style>
