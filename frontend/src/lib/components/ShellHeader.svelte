<script lang="ts">
	import { onMount } from 'svelte';
	import GranularityPill from './GranularityPill.svelte';
	import PeriodNav from './PeriodNav.svelte';
	import RollingTitle from './RollingTitle.svelte';
	import { tabs } from '$lib/tabs/registry';
	import { tabState } from '$lib/tabs/state.svelte';
	import { periodState } from '$lib/views/period-state.svelte';
	import { periodLabel } from '$lib/views/period';
	import { entryIntent } from '$lib/views/entries/intent.svelte';

	const currentIndex = $derived(Math.max(0, tabs.findIndex((t) => t.id === tabState.get())));
	const current = $derived(tabs[currentIndex] ?? tabs[0]);
	const showPeriod = $derived(current.hasPeriodHeader && periodState.initialized);
	const label = $derived(
		periodState.initialized
			? periodLabel(periodState.year, periodState.month, periodState.granularity)
			: ''
	);

	const measureEls: (HTMLHeadingElement | undefined)[] = $state(Array(tabs.length).fill(undefined));
	let titleMinWidth = $state(0);

	onMount(() => {
		let max = 0;
		for (const el of measureEls) {
			if (el) max = Math.max(max, el.getBoundingClientRect().width);
		}
		titleMinWidth = Math.ceil(max);
	});

	function newEntry() {
		const verb = current.newEntryVerb;
		if (!verb) return;
		entryIntent.set({ verb, sourceTab: current.id });
		tabState.set('entries');
	}
</script>

<header class="flex shrink-0 items-baseline justify-between gap-3 p-6 pb-0">
	<div class="flex items-baseline gap-3">
		<h3
			class="text-xl font-semibold text-ink"
			style:min-width={titleMinWidth ? `${titleMinWidth}px` : undefined}
		>
			<RollingTitle value={current.title} position={currentIndex} />
		</h3>
		{#if showPeriod}
			<PeriodNav
				label={label}
				onPrev={() => periodState.shift(-1)}
				onNext={() => periodState.shift(1)}
			/>
		{/if}
	</div>
	<div class="title-measure-rack" aria-hidden="true">
		{#each tabs as t, i (t.id)}
			<h3 bind:this={measureEls[i]} class="text-xl font-semibold">{t.title}</h3>
		{/each}
	</div>
	<div class="flex items-baseline gap-3">
		{#if current.newEntryVerb}
			<button
				type="button"
				onclick={newEntry}
				class="bg-surface-inverse px-3 py-1.5 text-xs font-medium uppercase tracking-[0.08em] text-ink-inverse transition-colors hover:bg-surface-inverse-hover"
			>
				+ New
			</button>
		{/if}
		{#if showPeriod}
			<GranularityPill
				granularity={periodState.granularity}
				onSelect={(g) => periodState.setGranularity(g)}
			/>
		{/if}
	</div>
</header>

<style>
	.title-measure-rack {
		position: absolute;
		left: -9999px;
		top: 0;
		visibility: hidden;
		pointer-events: none;
		white-space: nowrap;
	}
	.title-measure-rack h3 {
		display: inline-block;
	}
</style>
