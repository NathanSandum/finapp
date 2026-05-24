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

<div class="glass-faint rounded-xl p-2">
	<table class="trend-list w-full text-[11px]">
		<thead>
			<tr
				class="h-8 border-b border-[var(--color-glass-divider)] text-[9px] uppercase tracking-[0.12em] text-ink-muted"
			>
				<th class="px-2 text-left font-semibold">Account</th>
				<th class="px-2 text-left font-semibold">Trend</th>
				<th class="px-2 text-right font-semibold">Now</th>
			</tr>
		</thead>
		<tbody>
			{#each rows as row (row.name)}
				<TrendRowView {row} {labels} {keys} stroke={STROKE[row.accent]} txt={TXT[row.accent]} />
			{/each}
		</tbody>
	</table>
</div>

<style>
	.trend-list :global(tbody tr) {
		transition: background 240ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.trend-list :global(tbody tr:hover) {
		background: linear-gradient(
			90deg,
			oklch(75% 0.14 268 / 0.08) 0%,
			transparent 80%
		);
	}
	.trend-list :global(tbody td) {
		padding-left: 0.5rem;
		padding-right: 0.5rem;
	}
</style>
