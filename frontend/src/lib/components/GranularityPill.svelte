<script lang="ts">
	import type { Granularity } from '$lib/views/period';

	let {
		granularity,
		onSelect
	}: {
		granularity: Granularity;
		onSelect: (next: Granularity) => void;
	} = $props();

	const items: { id: Granularity; label: string }[] = [
		{ id: 'month', label: 'Month' },
		{ id: 'quarter', label: 'Quarter' },
		{ id: 'year', label: 'Year' }
	];
</script>

<div class="flex gap-1 rounded-full bg-neutral-100 p-1 text-xs">
	{#each items as item}
		<button
			onclick={() => onSelect(item.id)}
			class="rounded-full px-3 py-1 {granularity === item.id
				? 'bg-white font-medium shadow-sm'
				: 'text-neutral-500'}">{item.label}</button
		>
	{/each}
</div>
