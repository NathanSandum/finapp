<script lang="ts">
	import { prefersReducedMotion } from '$lib/utils/motion';
	import Reel from './RollingTitleReel.svelte';

	let { value }: { value: string } = $props();

	type Slot = { key: number; before: string; after: string; animate: boolean };

	let slots = $state<Slot[]>([]);
	let prevValue = '';
	let nextKey = 0;

	$effect(() => {
		const curr = value;
		if (curr === prevValue && slots.length > 0) return;
		const reduce = prefersReducedMotion();
		const len = Math.max(prevValue.length, curr.length);
		const next: Slot[] = [];
		for (let i = 0; i < len; i++) {
			const a = prevValue[i] ?? '';
			const b = curr[i] ?? '';
			if (a === b) {
				next.push({ key: slots[i]?.key ?? nextKey++, before: a, after: b, animate: false });
			} else {
				next.push({ key: nextKey++, before: a, after: b, animate: !reduce });
			}
		}
		slots = next;
		prevValue = curr;
	});
</script>

<span class="rolling-title inline-flex" aria-label={value}>
	{#each slots as slot, i (slot.key)}
		<Reel
			before={slot.before}
			after={slot.after}
			delay={slot.animate ? i * 28 : 0}
			animate={slot.animate}
		/>
	{/each}
</span>

<style>
	.rolling-title {
		line-height: 1.1;
	}
</style>
