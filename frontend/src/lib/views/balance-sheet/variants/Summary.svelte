<script lang="ts">
	import CompositionFlow from '$lib/components/CompositionFlow.svelte';
	import type { Line } from '../compute';

	let {
		assets,
		liabilities,
		totalAssets,
		totalLiab
	}: {
		assets: Line[];
		liabilities: Line[];
		totalAssets: number;
		totalLiab: number;
	} = $props();
</script>

<CompositionFlow
	positive={{ lines: assets, total: totalAssets, label: 'Own' }}
	negative={{ lines: liabilities, total: totalLiab, label: 'Owe' }}
	negativePalette={['bg-negative-bar-1', 'bg-negative-bar-2', 'bg-negative-bar-3']}
	netLabel="Net Worth"
/>
