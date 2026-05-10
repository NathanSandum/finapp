<script lang="ts">
	import { fmt } from '../../income/lib';

	let {
		operatingTrail,
		investingTrail,
		financingTrail,
		labels
	}: {
		operatingTrail: number[];
		investingTrail: number[];
		financingTrail: number[];
		labels: string[];
	} = $props();

	type Card = {
		title: string;
		values: number[];
		stroke: string;
		verbPositive: string;
		verbNegative: string;
	};

	const cards = $derived<Card[]>([
		{
			title: 'Income',
			values: operatingTrail,
			stroke: 'var(--color-positive-line)',
			verbPositive: 'Earned',
			verbNegative: 'Spent'
		},
		{
			title: 'Investing',
			values: investingTrail,
			stroke: 'var(--color-investing-line)',
			verbPositive: 'Withdrew',
			verbNegative: 'Contributed'
		},
		{
			title: 'Financing',
			values: financingTrail,
			stroke: 'var(--color-financing-line)',
			verbPositive: 'Borrowed',
			verbNegative: 'Repaid'
		}
	]);

	function plot(values: number[], width: number, height: number) {
		const min = Math.min(0, ...values);
		const max = Math.max(0, ...values);
		const range = max - min || 1;
		const stepX = values.length > 1 ? width / (values.length - 1) : 0;
		const path = values
			.map((v, i) => `${(i * stepX).toFixed(1)},${(height - ((v - min) / range) * height).toFixed(1)}`)
			.join(' ');
		const zeroY = height - ((0 - min) / range) * height;
		return { path, zeroY };
	}
</script>

<div class="grid grid-cols-1 gap-3 md:grid-cols-3">
	{#each cards as { title, values, stroke, verbPositive, verbNegative }}
		{@const latest = values[values.length - 1] ?? 0}
		{@const sl = plot(values, 240, 40)}
		<div class="rounded-xl border border-border bg-surface-card p-3">
			<div class="flex items-baseline justify-between">
				<span class="text-xs font-medium text-ink-body">{title}</span>
				<span class="font-mono text-sm font-semibold tabular-nums text-ink">
					<span class="text-[10px] font-bold text-ink-muted">
						{latest >= 0 ? verbPositive : verbNegative}
					</span>
					{fmt(Math.abs(latest))}
				</span>
			</div>
			<svg viewBox="0 0 240 40" class="mt-1 h-10 w-full" preserveAspectRatio="none">
				<line
					x1="0"
					y1={sl.zeroY}
					x2="240"
					y2={sl.zeroY}
					class="stroke-trend-zero"
					stroke-width="0.5"
					stroke-dasharray="2 2"
				/>
				<polyline fill="none" {stroke} stroke-width="1.5" points={sl.path} />
			</svg>
			<div
				class="mt-1 grid font-mono text-[9px] tabular-nums text-ink-faint"
				style:grid-template-columns="repeat({labels.length}, 1fr)"
			>
				{#each labels as label}<span class="text-center">{label}</span>{/each}
			</div>
		</div>
	{/each}
</div>
