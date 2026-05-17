<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import PeriodHeader from '$lib/components/PeriodHeader.svelte';
	import { fmt, pct } from '$lib/format';
	import Num from '$lib/components/Num.svelte';
	import {
		latestActivityPeriod,
		periodLabel,
		shiftPeriod,
		snapToGranularity,
		type Granularity
	} from '$lib/views/period';
	import { computeOverview } from './compute';
	import type { OverviewRaw } from './loader';
	import Ratios from './variants/Ratios.svelte';
	import { entryIntent } from '$lib/views/entries/intent.svelte';
	import { tabState } from '$lib/tabs/state.svelte';
	import { overviewEntrance } from './entrance';
	import { prefersReducedMotion } from '$lib/utils/motion';

	let { data }: { data: OverviewRaw } = $props();

	const animateEntrance = overviewEntrance.consume() && !prefersReducedMotion();
	let entered = $state(!animateEntrance);

	onMount(() => {
		if (!animateEntrance) return;
		requestAnimationFrame(() => requestAnimationFrame(() => (entered = true)));
	});

	const initial = untrack(() => latestActivityPeriod(data.entries));
	let selectedYear = $state(initial.year);
	let selectedMonth = $state(initial.month);
	let granularity = $state<Granularity>('month');

	function shift(direction: 1 | -1) {
		const next = shiftPeriod(selectedYear, selectedMonth, granularity, direction);
		selectedYear = next.year;
		selectedMonth = next.month;
	}

	function setGranularity(next: Granularity) {
		const snapped = snapToGranularity(selectedYear, selectedMonth, next);
		selectedYear = snapped.year;
		selectedMonth = snapped.month;
		granularity = next;
	}

	const metrics = $derived(
		computeOverview(data.entries, data.accounts, data.anchors, selectedYear, selectedMonth, granularity)
	);

	function newEntry() {
		entryIntent.set({ verb: 'spent', sourceTab: 'overview' });
		tabState.set('entries');
	}
</script>

<div class="overview-root flex h-full flex-col gap-3 overflow-hidden" class:entered>
	<div class="entrance-item" style="--d: 0ms">
		<PeriodHeader
			title="Overview"
			periodLabel={periodLabel(selectedYear, selectedMonth, granularity)}
			{granularity}
			onPrev={() => shift(-1)}
			onNext={() => shift(1)}
			onGranularity={setGranularity}
		>
			{#snippet actions()}
				<button
					type="button"
					onclick={newEntry}
					class="bg-surface-inverse px-3 py-1.5 text-xs font-medium uppercase tracking-[0.08em] text-ink-inverse transition-colors hover:bg-surface-inverse-hover"
				>
					+ New
				</button>
			{/snippet}
		</PeriodHeader>
	</div>

	<section class="grid grid-cols-2 gap-12">
		<div class="entrance-item" style="--d: 90ms">
			<p class="text-[10px] font-medium uppercase tracking-[0.1em] text-ink-muted">Net Worth</p>
			<p class="mt-2 font-mono text-5xl font-semibold tabular-nums text-ink">
				<Num text={fmt(metrics.netWorth)} />
			</p>
			<p class="mt-1 font-mono text-xs tabular-nums text-positive">
				{pct(metrics.netWorth, metrics.priorNetWorth)} vs prior
			</p>
		</div>
		<div class="entrance-item" style="--d: 170ms">
			<p class="text-[10px] font-medium uppercase tracking-[0.1em] text-ink-muted">Income · 12 mo</p>
			<p class="mt-2 font-mono text-5xl font-semibold tabular-nums text-ink">
				<Num text={fmt(metrics.income12mo)} />
			</p>
			<p class="mt-1 font-mono text-xs tabular-nums text-positive">
				{pct(metrics.income12mo, metrics.priorIncome12mo)} vs prior 12 mo
			</p>
		</div>
	</section>

	<Ratios {metrics} {entered} entranceBaseDelay={animateEntrance ? 250 : null} />
</div>

<style>
	.overview-root :global(.entrance-item) {
		opacity: 0;
		transform: translateY(14px);
		transition:
			opacity 480ms cubic-bezier(0.22, 1, 0.36, 1) var(--d, 0ms),
			transform 480ms cubic-bezier(0.22, 1, 0.36, 1) var(--d, 0ms);
	}
	.overview-root.entered :global(.entrance-item) {
		opacity: 1;
		transform: translateY(0);
	}
	@media (prefers-reduced-motion: reduce) {
		.overview-root :global(.entrance-item),
		.overview-root.entered :global(.entrance-item) {
			transition: opacity 0ms, transform 0ms;
		}
	}
</style>
