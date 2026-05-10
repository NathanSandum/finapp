<script lang="ts">
	import type { OverviewMetrics } from '../compute';

	let { metrics }: { metrics: OverviewMetrics } = $props();

	const SIZE = 200;
	const CENTER = SIZE / 2;
	const RADIUS = 78;
	const STROKE = 12;

	const items = $derived([
		{ label: 'Emergency Fund', ratio: metrics.emergencyFund },
		{ label: 'Savings Rate', ratio: metrics.savingsRate },
		{ label: 'Debt / Income', ratio: metrics.debtToIncome },
		{ label: 'Debt / Assets', ratio: metrics.debtToAssets }
	]);

	function arcPath(health: number): string {
		const startAngle = 180;
		const endAngle = 180 - 180 * health;
		const startRad = (startAngle * Math.PI) / 180;
		const endRad = (endAngle * Math.PI) / 180;
		const x1 = CENTER + Math.cos(startRad) * RADIUS;
		const y1 = CENTER + Math.sin(startRad) * RADIUS;
		const x2 = CENTER + Math.cos(endRad) * RADIUS;
		const y2 = CENTER + Math.sin(endRad) * RADIUS;
		const largeArc = health > 0.5 ? 1 : 0;
		return `M ${x1.toFixed(1)} ${y1.toFixed(1)} A ${RADIUS} ${RADIUS} 0 ${largeArc} 1 ${x2.toFixed(1)} ${y2.toFixed(1)}`;
	}

	const trackPath = `M ${CENTER - RADIUS} ${CENTER} A ${RADIUS} ${RADIUS} 0 0 1 ${CENTER + RADIUS} ${CENTER}`;

	function healthColor(h: number): string {
		if (h >= 0.7) return 'var(--color-positive-line)';
		if (h >= 0.4) return 'var(--color-financing-line)';
		return 'var(--color-negative-line)';
	}
</script>

<div class="grid grid-cols-4 gap-6">
	{#each items as { label, ratio } (label)}
		<div class="flex flex-col items-center px-3 py-3">
			<p class="text-[10px] font-medium uppercase tracking-[0.1em] text-ink-muted">{label}</p>
			<svg width={SIZE} height={SIZE * 0.62} viewBox="0 0 {SIZE} {SIZE * 0.62}" class="mt-1">
				<path
					d={trackPath}
					fill="none"
					stroke="var(--color-border-faint)"
					stroke-width={STROKE}
					stroke-linecap="butt"
				/>
				<path
					d={arcPath(ratio.health)}
					fill="none"
					stroke={healthColor(ratio.health)}
					stroke-width={STROKE}
					stroke-linecap="round"
				/>
				<text
					x={CENTER}
					y={CENTER - 6}
					text-anchor="middle"
					dominant-baseline="middle"
					class="fill-ink"
					font-size="24"
					font-family="var(--font-mono)"
					font-weight="600"
				>
					{ratio.display}
				</text>
			</svg>
			<p class="font-mono text-[9px] tabular-nums text-ink-faint">target {ratio.target}</p>
		</div>
	{/each}
</div>
