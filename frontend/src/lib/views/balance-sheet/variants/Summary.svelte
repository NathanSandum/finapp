<script lang="ts">
	import CompositionSection from '$lib/components/CompositionSection.svelte';
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

	const assetPalette = ['bg-positive-bar-1', 'bg-positive-bar-2', 'bg-positive-bar-3', 'bg-positive-bar-4'];
	const liabPalette = ['bg-negative-bar-1', 'bg-negative-bar-2', 'bg-negative-bar-3'];
</script>

<div class="space-y-4">
	<CompositionSection
		title="Assets"
		total={totalAssets}
		lines={assets}
		base={totalAssets}
		palette={assetPalette}
		minLabelFraction={0.08}
		legend
	/>
	<CompositionSection
		title="Liabilities"
		total={totalLiab}
		lines={liabilities}
		base={totalAssets}
		palette={liabPalette}
		minLabelFraction={0.04}
		footnote="Width relative to total assets"
	/>
</div>
