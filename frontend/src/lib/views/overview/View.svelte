<script lang="ts">
	import { onMount } from 'svelte';
	import { fmt, pct } from '$lib/format';
	import Num from '$lib/components/Num.svelte';
	import { periodState } from '$lib/views/period-state.svelte';
	import { computeOverview } from './compute';
	import type { OverviewRaw } from './loader';
	import Ratios from './variants/Ratios.svelte';
	import { overviewEntrance } from './entrance';
	import { prefersReducedMotion } from '$lib/utils/motion';

	let { data }: { data: OverviewRaw } = $props();

	const animateEntrance = overviewEntrance.consume() && !prefersReducedMotion();
	let entered = $state(!animateEntrance);

	onMount(() => {
		if (!animateEntrance) return;
		requestAnimationFrame(() => requestAnimationFrame(() => (entered = true)));
	});

	const metrics = $derived(
		computeOverview(
			data.entries,
			data.accounts,
			data.anchors,
			periodState.year,
			periodState.month,
			periodState.granularity
		)
	);

	const nwDelta = $derived(metrics.netWorth - metrics.priorNetWorth);
	const incDelta = $derived(metrics.income12mo - metrics.priorIncome12mo);
</script>

<div class="overview-root flex h-full flex-col gap-10 overflow-y-auto pb-10" class:entered>
	<section class="border-t-2 border-ink pt-8">
		<p class="font-display text-2xl font-medium tracking-tight text-ink">
			A view of {metrics.asOfLabel}.
		</p>
		<p class="mt-1 max-w-prose text-sm text-ink-body">
			Net worth, income, and savings discipline — as of the period in view.
		</p>
	</section>

	<section class="grid grid-cols-2 gap-12">
		<div class="entrance-item border-t border-ink pt-4" style="--d: 90ms">
			<p class="text-[10px] font-medium uppercase tracking-[0.22em] text-ink-faint">Net Worth</p>
			<p class="mt-2 font-display text-6xl font-medium tabular-nums leading-none text-ink">
				<Num text={fmt(metrics.netWorth)} />
			</p>
			<p class="mt-3 text-[11px] uppercase tracking-[0.12em] text-ink-body">
				<span class={nwDelta >= 0 ? 'text-positive' : 'text-negative'}>
					{nwDelta >= 0 ? '▲' : '▼'} {pct(metrics.netWorth, metrics.priorNetWorth)}
				</span>
				<span class="text-ink-faint">vs prior period</span>
			</p>
		</div>
		<div class="entrance-item border-t border-ink pt-4" style="--d: 170ms">
			<p class="text-[10px] font-medium uppercase tracking-[0.22em] text-ink-faint">Income · trailing 12 mo</p>
			<p class="mt-2 font-display text-6xl font-medium tabular-nums leading-none text-ink">
				<Num text={fmt(metrics.income12mo)} />
			</p>
			<p class="mt-3 text-[11px] uppercase tracking-[0.12em] text-ink-body">
				<span class={incDelta >= 0 ? 'text-positive' : 'text-negative'}>
					{incDelta >= 0 ? '▲' : '▼'} {pct(metrics.income12mo, metrics.priorIncome12mo)}
				</span>
				<span class="text-ink-faint">vs prior 12 mo</span>
			</p>
		</div>
	</section>

	<section>
		<h2 class="mb-4 border-b border-border pb-2 font-display text-base font-medium uppercase tracking-[0.2em] text-ink">
			Financial Health
		</h2>
		<Ratios {metrics} {entered} entranceBaseDelay={animateEntrance ? 250 : null} />
	</section>
</div>

<style>
	.overview-root :global(.entrance-item) {
		opacity: 0;
		transform: translateY(8px);
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
			transition:
				opacity 0ms,
				transform 0ms;
		}
	}
</style>
