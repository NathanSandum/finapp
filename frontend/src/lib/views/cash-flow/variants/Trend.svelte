<script lang="ts">
	import TrendList, { type TrendRow } from '$lib/components/TrendList.svelte';

	let {
		operatingTrail,
		investingTrail,
		financingTrail,
		labels
	}: {
		operatingTrail: number[];
		investingTrail: number[];
		financingTrail: number[];
		labels: string[];
	} = $props();

	const rows = $derived<TrendRow[]>([
		{
			name: 'Income',
			amount: operatingTrail[operatingTrail.length - 1] ?? 0,
			trail: operatingTrail,
			accent: 'positive'
		},
		{
			name: 'Investing',
			amount: investingTrail[investingTrail.length - 1] ?? 0,
			trail: investingTrail,
			accent: 'investing'
		},
		{
			name: 'Financing',
			amount: financingTrail[financingTrail.length - 1] ?? 0,
			trail: financingTrail,
			accent: 'financing'
		}
	]);
</script>

<TrendList {rows} {labels} />
