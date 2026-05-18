<script lang="ts">
	import NetHero from '$lib/components/NetHero.svelte';
	import StatementView from '$lib/components/StatementView.svelte';
	import {
		computeCashFlow,
		computeNetCashTrail,
		computeSectionTrails,
		defaultTrailCount
	} from './compute';
	import MetricTile from '$lib/components/MetricTile.svelte';
	import type { CashFlowRaw } from './loader';
	import Detail from './variants/Detail.svelte';
	import Summary from './variants/Summary.svelte';
	import Trend from './variants/Trend.svelte';
	import { periodState } from '$lib/views/period-state.svelte';

	let { data }: { data: CashFlowRaw } = $props();

	const cf = $derived(
		computeCashFlow(
			data.entries,
			data.accounts,
			data.anchors,
			periodState.year,
			periodState.month,
			periodState.granularity
		)
	);
	const netTrail = $derived(
		computeNetCashTrail(
			data.entries,
			data.accounts,
			data.anchors,
			periodState.year,
			periodState.month,
			periodState.granularity,
			defaultTrailCount(periodState.granularity)
		)
	);
	const sectionTrails = $derived(
		computeSectionTrails(
			data.entries,
			data.accounts,
			data.anchors,
			periodState.year,
			periodState.month,
			periodState.granularity,
			defaultTrailCount(periodState.granularity)
		)
	);

	const netChange = $derived(cf.closing_cash - cf.opening_cash);
	const priorNet = $derived(
		cf.prior_operating_total + cf.prior_investing_total + cf.prior_financing_total
	);
</script>

<StatementView>
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
			<MetricTile
				label="Income"
				total={cf.operating.total}
				prior={cf.prior_operating_total}
				priorShortLabel={cf.prior_short_label}
				accent="emerald"
				signPrefix={{ positive: 'From', negative: 'To' }}
				verbs={{ positive: 'Earned', negative: 'Spent' }}
			/>
			<MetricTile
				label="Investments"
				total={cf.investing.total}
				prior={cf.prior_investing_total}
				priorShortLabel={cf.prior_short_label}
				accent="sky"
				signPrefix={{ positive: 'From', negative: 'To' }}
				verbs={{ positive: 'Withdrew', negative: 'Contributed' }}
			/>
			<MetricTile
				label="Financing"
				total={cf.financing.total}
				prior={cf.prior_financing_total}
				priorShortLabel={cf.prior_short_label}
				accent="amber"
				signPrefix={{ positive: 'From', negative: 'To' }}
				verbs={{ positive: 'Borrowed', negative: 'Repaid' }}
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
