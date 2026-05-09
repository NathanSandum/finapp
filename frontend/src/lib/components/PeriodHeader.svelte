<script lang="ts">
	import type { Granularity } from '$lib/views/period';

	let {
		title,
		periodLabel,
		granularity,
		onPrev,
		onNext,
		onGranularity
	}: {
		title: string;
		periodLabel: string;
		granularity: Granularity;
		onPrev: () => void;
		onNext: () => void;
		onGranularity: (g: Granularity) => void;
	} = $props();
</script>

<header class="flex shrink-0 items-baseline justify-between">
	<div class="flex items-baseline gap-3">
		<h3 class="text-xl font-semibold text-neutral-900">{title}</h3>
		<div class="flex items-center gap-1 text-xs text-neutral-500">
			<button
				onclick={onPrev}
				class="rounded px-1.5 py-0.5 hover:bg-neutral-100"
				aria-label="Previous period">‹</button
			>
			<span class="min-w-[110px] text-center">{periodLabel}</span>
			<button
				onclick={onNext}
				class="rounded px-1.5 py-0.5 hover:bg-neutral-100"
				aria-label="Next period">›</button
			>
		</div>
	</div>
	<div class="flex gap-1 rounded-full bg-neutral-100 p-1 text-xs">
		<button
			onclick={() => onGranularity('month')}
			class="rounded-full px-3 py-1 {granularity === 'month'
				? 'bg-white font-medium shadow-sm'
				: 'text-neutral-500'}">Month</button
		>
		<button
			onclick={() => onGranularity('quarter')}
			class="rounded-full px-3 py-1 {granularity === 'quarter'
				? 'bg-white font-medium shadow-sm'
				: 'text-neutral-500'}">Quarter</button
		>
		<button
			onclick={() => onGranularity('year')}
			class="rounded-full px-3 py-1 {granularity === 'year'
				? 'bg-white font-medium shadow-sm'
				: 'text-neutral-500'}">Year</button
		>
	</div>
</header>
