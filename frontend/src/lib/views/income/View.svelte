<script lang="ts">
	import { untrack } from 'svelte';
	import Area from './variants/Area.svelte';
	// Alt chart style kept in repo: ./variants/Bars.svelte — swap import + tag below to use it.
	import type { IncomeRaw } from './loader';
	import {
		computeIncomeData,
		computeNetTrail,
		defaultTrailCount,
		latestActivityMonth,
		shiftPeriod,
		snapToGranularity,
		type Granularity
	} from './compute';

	let { data }: { data: IncomeRaw } = $props();

	const initial = untrack(() => latestActivityMonth(data.entries, data.accounts));
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

	const income = $derived(
		computeIncomeData(data.entries, data.accounts, selectedYear, selectedMonth, granularity)
	);
	const trail = $derived(
		computeNetTrail(
			data.entries,
			data.accounts,
			selectedYear,
			selectedMonth,
			granularity,
			defaultTrailCount(granularity)
		)
	);
</script>

<Area
	data={income}
	netTrail={trail.values}
	months={trail.labels}
	{granularity}
	onPrev={() => shift(-1)}
	onNext={() => shift(1)}
	onGranularity={setGranularity}
/>
