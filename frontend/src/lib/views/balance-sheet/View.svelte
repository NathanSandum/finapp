<script lang="ts">
	import { untrack } from 'svelte';
	import NetHero from '$lib/components/NetHero.svelte';
	import PeriodHeader from '$lib/components/PeriodHeader.svelte';
	import { delta, fmt, pct } from '../income/lib';
	import {
		computeAssetTrails,
		computeBalanceSheet,
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

	type Mode = 'summary' | 'detail' | 'trend';

	const initial = untrack(() => latestActivityPeriod(data.entries, data.accounts));
	let selectedYear = $state(initial.year);
	let selectedMonth = $state(initial.month);
	let granularity = $state<Granularity>('month');
	let mode = $state<Mode>('summary');

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

	const totalAssets = $derived(bs.assets.reduce((s, l) => s + l.amount, 0));
	const totalLiab = $derived(bs.liabilities.reduce((s, l) => s + l.amount, 0));
	const netWorth = $derived(totalAssets - totalLiab);
	const priorAssets = $derived(bs.prior_assets.reduce((s, l) => s + l.amount, 0));
	const priorLiab = $derived(bs.prior_liabilities.reduce((s, l) => s + l.amount, 0));
	const priorNet = $derived(priorAssets - priorLiab);
</script>

<div class="flex flex-col gap-3">
	<PeriodHeader
		title="Balance Sheet"
		periodLabel="As of {bs.as_of_label}"
		{granularity}
		onPrev={() => shift(-1)}
		onNext={() => shift(1)}
		onGranularity={setGranularity}
	/>

	<NetHero
		label="Net Worth · {bs.as_of_label}"
		value={netWorth}
		prior={priorNet}
		priorShortLabel={bs.prior_short_label}
		trail={netTrail.values}
		trailLabels={netTrail.labels}
		gradientId="balance-sheet-net-grad"
		badgeSuffix={delta(netWorth, priorNet)}
	/>

	<div class="grid grid-cols-2 gap-3">
		<div class="rounded-lg bg-emerald-50/50 p-3">
			<p class="text-[10px] font-medium uppercase tracking-wide text-emerald-700">Assets</p>
			<p class="mt-1 font-mono text-2xl font-semibold tabular-nums text-neutral-900">
				{fmt(totalAssets)}
			</p>
			<p
				class="mt-0.5 font-mono text-[10px] tabular-nums {totalAssets >= priorAssets
					? 'text-emerald-700'
					: 'text-rose-700'}"
			>
				{pct(totalAssets, priorAssets)} vs {bs.prior_short_label}
			</p>
		</div>
		<div class="rounded-lg bg-rose-50/50 p-3">
			<p class="text-[10px] font-medium uppercase tracking-wide text-rose-700">Liabilities</p>
			<p class="mt-1 font-mono text-2xl font-semibold tabular-nums text-neutral-900">
				{fmt(totalLiab)}
			</p>
			<p
				class="mt-0.5 font-mono text-[10px] tabular-nums {totalLiab <= priorLiab
					? 'text-emerald-700'
					: 'text-rose-700'}"
			>
				{pct(totalLiab, priorLiab)} vs {bs.prior_short_label}
			</p>
		</div>
	</div>

	<div class="flex justify-center">
		<div class="flex gap-1 rounded-full bg-neutral-100 p-1 text-xs">
			<button
				onclick={() => (mode = 'summary')}
				class="flex items-center gap-1.5 rounded-full px-4 py-1.5 {mode === 'summary'
					? 'bg-white font-semibold shadow-sm'
					: 'text-neutral-500'}"
			>
				<span aria-hidden="true" class="font-mono text-[10px]">▤</span>Summary
			</button>
			<button
				onclick={() => (mode = 'detail')}
				class="flex items-center gap-1.5 rounded-full px-4 py-1.5 {mode === 'detail'
					? 'bg-white font-semibold shadow-sm'
					: 'text-neutral-500'}"
			>
				<span aria-hidden="true" class="font-mono text-[10px]">≡</span>Detail
			</button>
			<button
				onclick={() => (mode = 'trend')}
				class="flex items-center gap-1.5 rounded-full px-4 py-1.5 {mode === 'trend'
					? 'bg-white font-semibold shadow-sm'
					: 'text-neutral-500'}"
			>
				<span aria-hidden="true" class="font-mono text-[10px]">∿</span>Trend
			</button>
		</div>
	</div>

	{#if mode === 'summary'}
		<Summary
			assets={bs.assets}
			liabilities={bs.liabilities}
			{totalAssets}
			{totalLiab}
		/>
	{:else if mode === 'detail'}
		<Detail
			assets={bs.assets}
			liabilities={bs.liabilities}
			{totalAssets}
			{totalLiab}
			{priorAssets}
			{priorLiab}
			priorShortLabel={bs.prior_short_label}
		/>
	{:else}
		<Trend assets={bs.assets} trails={assetTrails.trails} />
	{/if}
</div>
