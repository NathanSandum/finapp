<script lang="ts">
	import MetricTile from '$lib/components/MetricTile.svelte';
	import NetHero from '$lib/components/NetHero.svelte';
	import StatementView from '$lib/components/StatementView.svelte';
	import {
		computeAssetTrails,
		computeBalanceSheet,
		computeLiabilityTrails,
		computeNetWorthTrail,
		defaultTrailCount
	} from './compute';
	import type { BalanceSheetRaw } from './loader';
	import Detail from './variants/Detail.svelte';
	import Summary from './variants/Summary.svelte';
	import Trend from './variants/Trend.svelte';
	import { periodState } from '$lib/views/period-state.svelte';

	let { data }: { data: BalanceSheetRaw } = $props();

	const bs = $derived(
		computeBalanceSheet(
			data.entries,
			data.accounts,
			periodState.year,
			periodState.month,
			periodState.granularity
		)
	);
	const netTrail = $derived(
		computeNetWorthTrail(
			data.entries,
			data.accounts,
			periodState.year,
			periodState.month,
			periodState.granularity,
			defaultTrailCount(periodState.granularity)
		)
	);
	const assetTrails = $derived(
		computeAssetTrails(
			data.entries,
			data.accounts,
			periodState.year,
			periodState.month,
			periodState.granularity,
			defaultTrailCount(periodState.granularity)
		)
	);
	const liabilityTrails = $derived(
		computeLiabilityTrails(
			data.entries,
			data.accounts,
			periodState.year,
			periodState.month,
			periodState.granularity,
			defaultTrailCount(periodState.granularity)
		)
	);

	const totalAssets = $derived(bs.assets.reduce((s, l) => s + l.amount, 0));
	const totalLiab = $derived(bs.liabilities.reduce((s, l) => s + l.amount, 0));
	const netWorth = $derived(totalAssets - totalLiab);
	const priorAssets = $derived(bs.prior_assets.reduce((s, l) => s + l.amount, 0));
	const priorLiab = $derived(bs.prior_liabilities.reduce((s, l) => s + l.amount, 0));
	const priorNet = $derived(priorAssets - priorLiab);
</script>

<StatementView>
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
			labels={assetTrails.labels}
		/>
	{/snippet}
</StatementView>
