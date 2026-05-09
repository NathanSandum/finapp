<script lang="ts">
	import { untrack } from 'svelte';
	import NetHero from '$lib/components/NetHero.svelte';
	import PeriodHeader from '$lib/components/PeriodHeader.svelte';
	import { buildLine, delta, fmt, pct } from '../income/lib';
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

	const assetColors = ['bg-emerald-500', 'bg-emerald-400', 'bg-emerald-300', 'bg-emerald-200'];
	const liabColors = ['bg-rose-500', 'bg-rose-400', 'bg-rose-300'];
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
		<div class="space-y-4">
			<div>
				<div class="mb-1.5 flex items-baseline justify-between">
					<span class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Assets</span>
					<span class="font-mono text-xs tabular-nums text-neutral-500">{fmt(totalAssets)}</span>
				</div>
				<div class="flex h-7 w-full overflow-hidden rounded-md bg-neutral-100">
					{#each bs.assets as line, i}
						{@const fraction = totalAssets > 0 ? line.amount / totalAssets : 0}
						<div
							class="{assetColors[
								i % assetColors.length
							]} flex items-center justify-center text-[10px] font-medium text-white"
							style:width="{fraction * 100}%"
						>
							{#if fraction > 0.08}{line.name}{/if}
						</div>
					{/each}
				</div>
				<ul class="mt-2 grid grid-cols-2 gap-x-6 gap-y-1 text-xs sm:grid-cols-4">
					{#each bs.assets as line, i}
						<li class="flex items-baseline gap-2">
							<span class="h-2 w-2 rounded-sm {assetColors[i % assetColors.length]}"></span>
							<span class="text-neutral-700">{line.name}</span>
							<span class="ml-auto font-mono tabular-nums text-neutral-900">{fmt(line.amount)}</span>
						</li>
					{/each}
				</ul>
			</div>
			<div>
				<div class="mb-1.5 flex items-baseline justify-between">
					<span class="text-xs font-semibold uppercase tracking-wide text-neutral-500"
						>Liabilities</span
					>
					<span class="font-mono text-xs tabular-nums text-neutral-500">{fmt(totalLiab)}</span>
				</div>
				<div class="flex h-7 w-full overflow-hidden rounded-md bg-neutral-100">
					{#each bs.liabilities as line, i}
						{@const fraction = totalAssets > 0 ? line.amount / totalAssets : 0}
						<div
							class="{liabColors[
								i % liabColors.length
							]} flex items-center justify-center text-[10px] font-medium text-white"
							style:width="{fraction * 100}%"
						>
							{#if fraction > 0.04}{line.name}{/if}
						</div>
					{/each}
				</div>
				<p class="mt-1 text-[10px] text-neutral-400">Width relative to total assets</p>
			</div>
		</div>
	{:else if mode === 'detail'}
		<div class="grid grid-cols-2 gap-3">
			<div class="rounded-xl border border-neutral-200 bg-white p-4">
				<div class="mb-0.5 flex items-baseline justify-between">
					<h4 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Assets</h4>
					<span class="font-mono text-base font-semibold tabular-nums text-neutral-900"
						>{fmt(totalAssets)}</span
					>
				</div>
				<p
					class="mb-2 font-mono text-[10px] tabular-nums {totalAssets >= priorAssets
						? 'text-emerald-700'
						: 'text-rose-700'}"
				>
					{pct(totalAssets, priorAssets)} vs {bs.prior_short_label}
				</p>
				<ul class="space-y-1.5">
					{#each bs.assets as line}
						{@const fraction = totalAssets > 0 ? line.amount / totalAssets : 0}
						<li>
							<div class="flex items-baseline justify-between text-xs">
								<span class="text-neutral-700">{line.name}</span>
								<span class="font-mono tabular-nums text-neutral-900">{fmt(line.amount)}</span>
							</div>
							<div class="mt-0.5 flex items-center gap-2">
								<div class="h-1 flex-1 overflow-hidden rounded-full bg-neutral-100">
									<div class="h-full bg-emerald-400" style:width="{fraction * 100}%"></div>
								</div>
								<span class="w-10 text-right font-mono text-[9px] tabular-nums text-neutral-400"
									>{(fraction * 100).toFixed(0)}%</span
								>
							</div>
						</li>
					{/each}
				</ul>
			</div>
			<div class="rounded-xl border border-neutral-200 bg-white p-4">
				<div class="mb-0.5 flex items-baseline justify-between">
					<h4 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">
						Liabilities
					</h4>
					<span class="font-mono text-base font-semibold tabular-nums text-neutral-900"
						>{fmt(totalLiab)}</span
					>
				</div>
				<p
					class="mb-2 font-mono text-[10px] tabular-nums {totalLiab <= priorLiab
						? 'text-emerald-700'
						: 'text-rose-700'}"
				>
					{pct(totalLiab, priorLiab)} vs {bs.prior_short_label}
				</p>
				<ul class="space-y-1.5">
					{#each bs.liabilities as line}
						{@const fraction = totalLiab > 0 ? line.amount / totalLiab : 0}
						<li>
							<div class="flex items-baseline justify-between text-xs">
								<span class="text-neutral-700">{line.name}</span>
								<span class="font-mono tabular-nums text-neutral-900">{fmt(line.amount)}</span>
							</div>
							<div class="mt-0.5 flex items-center gap-2">
								<div class="h-1 flex-1 overflow-hidden rounded-full bg-neutral-100">
									<div class="h-full bg-rose-400" style:width="{fraction * 100}%"></div>
								</div>
								<span class="w-10 text-right font-mono text-[9px] tabular-nums text-neutral-400"
									>{(fraction * 100).toFixed(0)}%</span
								>
							</div>
						</li>
					{/each}
				</ul>
			</div>
		</div>
	{:else}
		<div class="grid grid-cols-2 gap-3">
			{#each bs.assets as line}
				{@const trail = assetTrails.trails.get(line.name) ?? [line.amount]}
				{@const sl = buildLine(trail, 240, 40)}
				<div class="rounded-xl border border-neutral-200 bg-white p-3">
					<div class="flex items-baseline justify-between">
						<span class="text-xs font-medium text-neutral-700">{line.name}</span>
						<span class="font-mono text-sm font-semibold tabular-nums text-neutral-900"
							>{fmt(line.amount)}</span
						>
					</div>
					<svg viewBox="0 0 240 40" class="mt-1 h-10 w-full" preserveAspectRatio="none">
						<polyline fill="none" stroke="rgb(16 185 129)" stroke-width="1.5" points={sl.path} />
					</svg>
				</div>
			{/each}
		</div>
	{/if}
</div>
