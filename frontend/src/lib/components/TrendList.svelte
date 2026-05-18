<script lang="ts" module>
	export type TrendAccent = 'positive' | 'negative' | 'investing' | 'financing';
	export type TrendRow = {
		name: string;
		amount: number;
		trail: number[];
		accent: TrendAccent;
	};
</script>

<script lang="ts">
	import TrendRowView from './TrendRow.svelte';

	let {
		rows,
		labels,
		keys
	}: { rows: TrendRow[]; labels: string[]; keys: string[] } = $props();

	const STROKE: Record<TrendAccent, string> = {
		positive: 'stroke-positive-line',
		negative: 'stroke-negative-line',
		investing: 'stroke-investing-line',
		financing: 'stroke-financing-line'
	};
	const TXT: Record<TrendAccent, string> = {
		positive: 'text-positive',
		negative: 'text-negative',
		investing: 'text-investing',
		financing: 'text-financing'
	};
</script>

<table class="w-full text-[11px]">
	<thead>
		<tr class="h-8 border-b border-border text-[9px] uppercase tracking-[0.1em] text-ink-muted">
			<th class="text-left font-semibold">Account</th>
			<th class="text-left font-semibold">Trend</th>
			<th class="text-right font-semibold">Now</th>
		</tr>
	</thead>
	<tbody>
		{#each rows as row (row.name)}
			<TrendRowView {row} {labels} {keys} stroke={STROKE[row.accent]} txt={TXT[row.accent]} />
		{/each}
	</tbody>
</table>
