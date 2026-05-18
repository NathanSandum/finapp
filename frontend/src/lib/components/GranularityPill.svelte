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

	let containerEl = $state<HTMLDivElement | undefined>(undefined);
	const itemEls: (HTMLButtonElement | undefined)[] = $state(Array(items.length).fill(undefined));
	let indicator = $state({ left: 0, width: 0 });
	let indicatorReady = $state(false);

	$effect(() => {
		const id = granularity;
		if (!containerEl) return;
		const idx = items.findIndex((it) => it.id === id);
		const el = itemEls[idx];
		if (!el) return;
		const containerRect = containerEl.getBoundingClientRect();
		const r = el.getBoundingClientRect();
		indicator = { left: r.left - containerRect.left, width: r.width };
		if (!indicatorReady) {
			requestAnimationFrame(() => requestAnimationFrame(() => (indicatorReady = true)));
		}
	});
</script>

<div
	bind:this={containerEl}
	class="relative flex gap-1 rounded-full bg-surface-sunken p-1 text-xs"
>
	{#if indicator.width > 0}
		<span
			class="indicator absolute top-1 bottom-1 rounded-full bg-surface-card shadow-sm"
			class:ready={indicatorReady}
			style:left="{indicator.left}px"
			style:width="{indicator.width}px"
		></span>
	{/if}
	{#each items as item, i (item.id)}
		<button
			bind:this={itemEls[i]}
			onclick={() => onSelect(item.id)}
			class="relative z-10 rounded-full px-3 py-1 transition-colors {granularity === item.id
				? 'font-medium text-ink'
				: 'text-ink-muted'}">{item.label}</button
		>
	{/each}
</div>

<style>
	.indicator {
		transition: none;
	}
	.indicator.ready {
		transition:
			left 320ms cubic-bezier(0.7, 0, 0.25, 1),
			width 320ms cubic-bezier(0.7, 0, 0.25, 1);
	}
	@media (prefers-reduced-motion: reduce) {
		.indicator.ready {
			transition: left 0ms, width 0ms;
		}
	}
</style>
