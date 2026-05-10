<script lang="ts">
	import RankedFlow from '$lib/components/RankedFlow.svelte';
	import type { Line } from '../compute';

	let {
		assets,
		liabilities,
		totalAssets,
		totalLiab,
		priorAssets,
		priorLiab,
		priorShortLabel
	}: {
		assets: Line[];
		liabilities: Line[];
		totalAssets: number;
		totalLiab: number;
		priorAssets: number;
		priorLiab: number;
		priorShortLabel: string;
	} = $props();

	const net = $derived(totalAssets - totalLiab);
	const priorNet = $derived(priorAssets - priorLiab);
</script>

<RankedFlow
	positive={assets}
	negative={liabilities}
	{net}
	{priorNet}
	{priorShortLabel}
	netLabel="Net Worth"
/>
