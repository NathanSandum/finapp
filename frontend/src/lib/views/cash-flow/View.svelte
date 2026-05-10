<script lang="ts">
	import { untrack } from 'svelte';
	import NetHero from '$lib/components/NetHero.svelte';
	import StatementView from '$lib/components/StatementView.svelte';
	import {
		computeCashFlow,
		computeNetCashTrail,
		computeSectionTrails,
		defaultTrailCount,
		latestActivityPeriod,
		shiftPeriod,
		snapToGranularity,
		type Granularity
	} from './compute';
	import CashFlowTile from './CashFlowTile.svelte';
	import type { CashFlowRaw } from './loader';
	import Detail from './variants/Detail.svelte';
	import Summary from './variants/Summary.svelte';
	import Trend from './variants/Trend.svelte';

	let { data }: { data: CashFlowRaw } = $props();

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

	const cf = $derived(
		computeCashFlow(data.entries, data.accounts, selectedYear, selectedMonth, granularity)
	);
	const netTrail = $derived(
		computeNetCashTrail(
			data.entries,
			data.accounts,
			selectedYear,
			selectedMonth,
			granularity,
			defaultTrailCount(granularity)
		)
	);
	const sectionTrails = $derived(
		computeSectionTrails(
			data.entries,
			data.accounts,
			selectedYear,
			selectedMonth,
			granularity,
			defaultTrailCount(granularity)
		)
	);

	const netChange = $derived(cf.closing_cash - cf.opening_cash);
	const priorNet = $derived(
		cf.prior_operating_total + cf.prior_investing_total + cf.prior_financing_total
	);
</script>

<StatementView
	title="Cash Flow"
	periodLabel={cf.period_label}
	{granularity}
	onPrev={() => shift(-1)}
	onNext={() => shift(1)}
	onGranularity={setGranularity}
>
	{#snippet hero()}
		<NetHero
			label="Net Change in Cash · {cf.period_label}"
			value={netChange}
			prior={priorNet}
			trail={netTrail.values}
			trailLabels={netTrail.labels}
			gradientId="cash-flow-net-grad"
		/>
	{/snippet}

	{#snippet tiles()}
		<div class="mx-auto h-px w-[95%] bg-border"></div>
		<div class="grid grid-cols-3 gap-3">
			<CashFlowTile
				noun="Income"
				total={cf.operating.total}
				prior={cf.prior_operating_total}
				priorShortLabel={cf.prior_short_label}
				accent="emerald"
				verbPositive="Earned"
				verbNegative="Spent"
			/>
			<CashFlowTile
				noun="Investments"
				total={cf.investing.total}
				prior={cf.prior_investing_total}
				priorShortLabel={cf.prior_short_label}
				accent="sky"
				verbPositive="Withdrew"
				verbNegative="Contributed"
			/>
			<CashFlowTile
				noun="Financing"
				total={cf.financing.total}
				prior={cf.prior_financing_total}
				priorShortLabel={cf.prior_short_label}
				accent="amber"
				verbPositive="Borrowed"
				verbNegative="Repaid"
			/>
		</div>
	{/snippet}

	{#snippet summary()}
		<Summary
			operating={cf.operating}
			investing={cf.investing}
			financing={cf.financing}
			openingCash={cf.opening_cash}
			closingCash={cf.closing_cash}
		/>
	{/snippet}

	{#snippet detail()}
		<Detail
			operating={cf.operating}
			investing={cf.investing}
			financing={cf.financing}
		/>
	{/snippet}

	{#snippet trend()}
		<Trend
			operatingTrail={sectionTrails.operating}
			investingTrail={sectionTrails.investing}
			financingTrail={sectionTrails.financing}
			labels={sectionTrails.labels}
		/>
	{/snippet}
</StatementView>
