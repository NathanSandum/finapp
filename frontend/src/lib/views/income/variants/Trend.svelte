<script lang="ts">
	import TrendList, { type TrendRow } from '$lib/components/TrendList.svelte';
	import type { Line } from '../lib';

	let {
		revenue,
		expenses,
		revenueTrails,
		expenseTrails,
		labels
	}: {
		revenue: Line[];
		expenses: Line[];
		revenueTrails: Map<string, number[]>;
		expenseTrails: Map<string, number[]>;
		labels: string[];
	} = $props();

	const rows = $derived<TrendRow[]>([
		...revenue.map((l) => ({
			name: l.name,
			amount: l.amount,
			trail: revenueTrails.get(l.name) ?? [l.amount],
			accent: 'positive' as const
		})),
		...expenses.map((l) => ({
			name: l.name,
			amount: l.amount,
			trail: expenseTrails.get(l.name) ?? [l.amount],
			accent: 'negative' as const
		}))
	]);
</script>

<TrendList {rows} {labels} />
