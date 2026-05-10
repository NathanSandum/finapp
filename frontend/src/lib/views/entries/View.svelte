<script lang="ts">
	import { untrack } from 'svelte';
	import PeriodHeader from '$lib/components/PeriodHeader.svelte';
	import {
		periodBounds,
		periodLabel,
		shiftPeriod,
		snapToGranularity,
		type Granularity
	} from '$lib/views/period';
	import TwoPaneLayout from './TwoPaneLayout.svelte';
	import type { EntriesData, Entry } from './loader';

	let { data }: { data: EntriesData } = $props();

	function latestActivityPeriod(entries: Entry[]): { year: number; month: number } {
		let latest: string | null = null;
		for (const entry of entries) {
			if (latest === null || entry.date > latest) latest = entry.date;
		}
		if (latest) {
			const [y, m] = latest.split('-');
			return { year: Number(y), month: Number(m) };
		}
		const now = new Date();
		return { year: now.getUTCFullYear(), month: now.getUTCMonth() + 1 };
	}

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

	const bounds = $derived(periodBounds(selectedYear, selectedMonth, granularity));
	const filteredEntries = $derived(
		data.entries.filter((e) => e.date >= bounds.start && e.date <= bounds.end)
	);
</script>

<div class="flex h-full flex-col gap-4">
	<PeriodHeader
		title="Entries"
		periodLabel={periodLabel(selectedYear, selectedMonth, granularity)}
		{granularity}
		onPrev={() => shift(-1)}
		onNext={() => shift(1)}
		onGranularity={setGranularity}
	/>
	<TwoPaneLayout entries={filteredEntries} accountNames={data.accountNames} />
</div>
