<script lang="ts">
	import TrendList, { type TrendRow } from '$lib/components/TrendList.svelte';
	import type { Line } from '../compute';

	let {
		assets,
		liabilities,
		assetTrails,
		liabilityTrails
	}: {
		assets: Line[];
		liabilities: Line[];
		assetTrails: Map<string, number[]>;
		liabilityTrails: Map<string, number[]>;
	} = $props();

	const rows = $derived<TrendRow[]>([
		...assets.map((l) => ({
			name: l.name,
			amount: l.amount,
			trail: assetTrails.get(l.name) ?? [l.amount],
			accent: 'positive' as const
		})),
		...liabilities.map((l) => ({
			name: l.name,
			amount: l.amount,
			trail: liabilityTrails.get(l.name) ?? [l.amount],
			accent: 'negative' as const
		}))
	]);
</script>

<TrendList {rows} />
