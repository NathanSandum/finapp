<script lang="ts">
	import { prefersReducedMotion } from '$lib/utils/motion';

	let {
		value,
		position = 0
	}: {
		value: string;
		position?: number;
	} = $props();

	type Mode = 'up' | 'down' | 'none';
	type Step = { key: number; before: string; after: string; mode: Mode };

	let step = $state<Step>({ key: 0, before: '', after: value, mode: 'none' });
	let prevValue = value;
	let prevPosition = position;
	let nextKey = 1;

	$effect(() => {
		const curr = value;
		const pos = position;
		if (curr === prevValue) {
			prevPosition = pos;
			return;
		}
		const reduce = prefersReducedMotion();
		const mode: Mode = reduce ? 'none' : pos >= prevPosition ? 'up' : 'down';
		step = { key: nextKey++, before: prevValue, after: curr, mode };
		prevValue = curr;
		prevPosition = pos;
	});
</script>

<span class="rolling-title" aria-label={value}>
	<span class="reel">
		{#key step.key}
			<span class="track mode-{step.mode}">
				{#if step.mode === 'down'}
					<span class="cell">{step.after || '\u00A0'}</span>
					<span class="cell">{step.before || '\u00A0'}</span>
				{:else}
					<span class="cell">{step.before || '\u00A0'}</span>
					<span class="cell">{step.after || '\u00A0'}</span>
				{/if}
			</span>
		{/key}
	</span>
</span>

<style>
	.rolling-title {
		display: inline-block;
		line-height: 1.1;
	}
	.reel {
		display: inline-block;
		overflow: hidden;
		line-height: 1.1;
		height: 1.1em;
		vertical-align: top;
	}
	.track {
		display: flex;
		flex-direction: column;
		will-change: transform;
	}
	.cell {
		display: block;
		line-height: 1.1;
		height: 1.1em;
		white-space: nowrap;
	}

	.track.mode-none .cell:first-child {
		display: none;
	}
	.track.mode-up {
		animation: roll-up 520ms cubic-bezier(0.32, 0.72, 0, 1) both;
	}
	.track.mode-down {
		animation: roll-down 520ms cubic-bezier(0.32, 0.72, 0, 1) both;
	}

	@keyframes roll-up {
		from {
			transform: translate3d(0, 0, 0);
		}
		to {
			transform: translate3d(0, -1.1em, 0);
		}
	}
	@keyframes roll-down {
		from {
			transform: translate3d(0, -1.1em, 0);
		}
		to {
			transform: translate3d(0, 0, 0);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.track.mode-up,
		.track.mode-down {
			animation: none;
		}
		.track .cell:first-child {
			display: none;
		}
	}
</style>
