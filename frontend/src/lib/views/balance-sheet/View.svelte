<script lang="ts">
	import { untrack } from 'svelte';
	import MetricTile from '$lib/components/MetricTile.svelte';
	import NetHero from '$lib/components/NetHero.svelte';
	import StatementView from '$lib/components/StatementView.svelte';
	import {
		computeAssetTrails,
		computeBalanceSheet,
		computeLiabilityTrails,
		computeNetWorthTrail,
		defaultTrailCount,
		latestActivityPeriod,
		shiftPeriod,
		snapToGranularity,
		type Granularity
	} from './compute';
	import type { BalanceSheetRaw } from './loader';
	import Detail from './variants/Detail.svelte';
	import Summary from './variants/Summary.svelte';
	import Trend from './variants/Trend.svelte';

	let { data }: { data: BalanceSheetRaw } = $props();

	const initial = untrack(() => latestActivityPeriod(data.entries, data.accounts));
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

	const bs = $derived(
		computeBalanceSheet(data.entries, data.accounts, selectedYear, selectedMonth, granularity)
	);
	const netTrail = $derived(
		computeNetWorthTrail(
			data.entries,
			data.accounts,
			selectedYear,
			selectedMonth,
			granularity,
			defaultTrailCount(granularity)
		)
	);
	const assetTrails = $derived(
		computeAssetTrails(
			data.entries,
			data.accounts,
			selectedYear,
			selectedMonth,
			granularity,
			defaultTrailCount(granularity)
		)
	);
	const liabilityTrails = $derived(
		computeLiabilityTrails(
			data.entries,
			data.accounts,
			selectedYear,
			selectedMonth,
			granularity,
			defaultTrailCount(granularity)
		)
	);

	const totalAssets = $derived(bs.assets.reduce((s, l) => s + l.amount, 0));
	const totalLiab = $derived(bs.liabilities.reduce((s, l) => s + l.amount, 0));
	const netWorth = $derived(totalAssets - totalLiab);
	const priorAssets = $derived(bs.prior_assets.reduce((s, l) => s + l.amount, 0));
	const priorLiab = $derived(bs.prior_liabilities.reduce((s, l) => s + l.amount, 0));
	const priorNet = $derived(priorAssets - priorLiab);
</script>

<StatementView
	title="Balance Sheet"
	periodLabel="As of {bs.as_of_label}"
	{granularity}
	onPrev={() => shift(-1)}
	onNext={() => shift(1)}
	onGranularity={setGranularity}
>
	{#snippet hero()}
		<NetHero
			label="Net Worth · {bs.as_of_label}"
			value={netWorth}
			prior={priorNet}
			trail={netTrail.values}
			trailLabels={netTrail.labels}
			gradientId="balance-sheet-net-grad"
		/>
	{/snippet}

	{#snippet tiles()}
		<div class="mx-auto h-px w-[95%] bg-border"></div>
		<div class="grid grid-cols-2 gap-3">
			<MetricTile
				label="Assets"
				total={totalAssets}
				prior={priorAssets}
				priorShortLabel={bs.prior_short_label}
				accent="emerald"
			/>
			<MetricTile
				label="Liabilities"
				total={totalLiab}
				prior={priorLiab}
				priorShortLabel={bs.prior_short_label}
				accent="rose"
			/>
		</div>
	{/snippet}

	{#snippet summary()}
		<Summary assets={bs.assets} liabilities={bs.liabilities} {totalAssets} {totalLiab} />
	{/snippet}

	{#snippet detail()}
		<Detail
			assets={bs.assets}
			liabilities={bs.liabilities}
			{totalAssets}
			{totalLiab}
			{priorAssets}
			{priorLiab}
			priorShortLabel={bs.prior_short_label}
		/>
	{/snippet}

	{#snippet trend()}
		<Trend
			assets={bs.assets}
			liabilities={bs.liabilities}
			assetTrails={assetTrails.trails}
			liabilityTrails={liabilityTrails.trails}
		/>
	{/snippet}
</StatementView>
