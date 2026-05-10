<script lang="ts">
	import { buildLine, fmt } from '$lib/views/income/lib';

	export type TrendAccent = 'positive' | 'negative' | 'investing' | 'financing';
	export type TrendRow = {
		name: string;
		amount: number;
		trail: number[];
		accent: TrendAccent;
	};

	let { rows }: { rows: TrendRow[] } = $props();

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
		{#each rows as row}
			{@const sl = buildLine(row.trail, 200, 24)}
			{@const first = row.trail[0] ?? 0}
			{@const last = row.trail[row.trail.length - 1] ?? 0}
			{@const dir = last >= first ? '▲' : '▼'}
			{@const stroke = STROKE[row.accent]}
			{@const txt = TXT[row.accent]}
			<tr class="h-8 border-b border-border-faint">
				<td class="text-ink-body">
					<span class="mr-1 font-mono {txt}">{dir}</span>{row.name}
				</td>
				<td>
					<svg viewBox="0 0 200 24" class="h-6 w-full max-w-[200px]">
						<polyline
							fill="none"
							class={stroke}
							stroke-width="1.25"
							stroke-linecap="round"
							stroke-linejoin="round"
							points={sl.path}
						/>
					</svg>
				</td>
				<td class="text-right font-mono font-semibold tabular-nums text-ink">{fmt(row.amount)}</td>
			</tr>
		{/each}
	</tbody>
</table>
