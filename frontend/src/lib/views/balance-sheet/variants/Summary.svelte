<script lang="ts">
	import type { Line } from '../compute';
	import CompositionSection from './CompositionSection.svelte';

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

	const assetPalette = ['bg-emerald-500', 'bg-emerald-400', 'bg-emerald-300', 'bg-emerald-200'];
	const liabPalette = ['bg-rose-500', 'bg-rose-400', 'bg-rose-300'];
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
