<script lang="ts">
	import type { TrailSlots } from './trailSlots.svelte';

	let {
		trailSlots,
		keys,
		labels,
		viewBoxWidth,
		labelClass = ''
	}: {
		trailSlots: TrailSlots;
		keys: string[];
		labels: string[];
		viewBoxWidth: number;
		labelClass?: string;
	} = $props();

	const labelByKey = $derived(new Map(keys.map((k, i) => [k, labels[i] ?? ''])));
</script>

<div class="relative h-3 w-full">
	{#each trailSlots.slots as s (s.key)}
		<span
			class="absolute top-0 -translate-x-1/2 whitespace-nowrap {labelClass}"
			style:left="{(s.pos.current.x / viewBoxWidth) * 100}%"
		>
			{labelByKey.get(s.key) ?? ''}
		</span>
	{/each}
</div>
