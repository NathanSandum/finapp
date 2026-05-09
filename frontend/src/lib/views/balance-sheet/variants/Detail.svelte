<script lang="ts">
	import type { Line } from '../compute';
	import AccountColumn from './AccountColumn.svelte';

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
</script>

<div class="grid grid-cols-2 gap-3">
	<AccountColumn
		title="Assets"
		lines={assets}
		total={totalAssets}
		prior={priorAssets}
		{priorShortLabel}
		accent="emerald"
		direction="higher"
	/>
	<AccountColumn
		title="Liabilities"
		lines={liabilities}
		total={totalLiab}
		prior={priorLiab}
		{priorShortLabel}
		accent="rose"
		direction="lower"
	/>
</div>
