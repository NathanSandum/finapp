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
		{ id: 'month', label: 'M' },
		{ id: 'quarter', label: 'Q' },
		{ id: 'year', label: 'Y' }
	];
</script>

<div class="inline-flex divide-x divide-border border border-border bg-surface-card text-[11px]">
	{#each items as item (item.id)}
		<button
			onclick={() => onSelect(item.id)}
			class="px-2.5 py-1 font-medium uppercase tracking-[0.15em] transition-colors {granularity === item.id
				? 'bg-ink text-ink-inverse'
				: 'text-ink-muted hover:text-ink'}"
			aria-pressed={granularity === item.id}>{item.label}</button
		>
	{/each}
</div>
