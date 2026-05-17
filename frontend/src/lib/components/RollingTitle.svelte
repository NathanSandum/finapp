<script lang="ts">
	import { prefersReducedMotion } from '$lib/utils/motion';

	let {
		value,
		position = 0
	}: {
		value: string;
		position?: number;
	} = $props();

	type Step = { key: number; before: string; after: string; direction: 1 | -1 };

	let step = $state<Step>({ key: 0, before: '', after: value, direction: 1 });
	let rolled = $state(true);
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
		const dir: 1 | -1 = pos >= prevPosition ? -1 : 1;
		step = { key: nextKey++, before: prevValue, after: curr, direction: dir };
		prevValue = curr;
		prevPosition = pos;
		if (reduce) {
			rolled = true;
		} else {
			rolled = false;
			requestAnimationFrame(() => requestAnimationFrame(() => (rolled = true)));
		}
	});
</script>

<span class="rolling-title" aria-label={value}>
	{#key step.key}
		<span class="reel" class:rolled class:down={step.direction === -1}>
			<span class="track">
				{#if step.direction === -1}
					<span class="cell">{step.after || '\u00A0'}</span>
					<span class="cell">{step.before || '\u00A0'}</span>
				{:else}
					<span class="cell">{step.before || '\u00A0'}</span>
					<span class="cell">{step.after || '\u00A0'}</span>
				{/if}
			</span>
		</span>
	{/key}
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
		transition: transform 420ms cubic-bezier(0.7, 0, 0.25, 1);
	}
	.reel.rolled .track {
		transform: translateY(-50%);
	}
	.reel.down .track {
		transform: translateY(-50%);
	}
	.reel.down.rolled .track {
		transform: translateY(0);
	}
	.cell {
		display: block;
		line-height: 1.1;
		height: 1.1em;
		white-space: nowrap;
	}
	@media (prefers-reduced-motion: reduce) {
		.track {
			transition: transform 0ms;
		}
	}
</style>
