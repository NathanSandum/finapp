<script lang="ts">
	import type { OverviewMetrics } from '../compute';

	let {
		metrics,
		entered = true,
		entranceBaseDelay = null
	}: {
		metrics: OverviewMetrics;
		entered?: boolean;
		entranceBaseDelay?: number | null;
	} = $props();

	const SIZE = 200;
	const CENTER = SIZE / 2;
	const RADIUS = 78;
	const STROKE = 12;
	const TRACK_LEN = Math.PI * RADIUS;

	const items = $derived([
		{ label: 'Emergency Fund', ratio: metrics.emergencyFund },
		{ label: 'Savings Rate', ratio: metrics.savingsRate },
		{ label: 'Debt / Income', ratio: metrics.debtToIncome },
		{ label: 'Debt / Assets', ratio: metrics.debtToAssets }
	]);

	const trackPath = `M ${CENTER - RADIUS} ${CENTER} A ${RADIUS} ${RADIUS} 0 0 1 ${CENTER + RADIUS} ${CENTER}`;

	function healthColor(h: number): string {
		if (h >= 0.7) return 'var(--color-positive-line)';
		if (h >= 0.4) return 'var(--color-financing-line)';
		return 'var(--color-negative-line)';
	}

	function fillOffset(health: number): number {
		const clamped = Math.max(0, Math.min(1, health));
		return TRACK_LEN * (1 - clamped);
	}
</script>

<div class="grid grid-cols-4 gap-6">
	{#each items as { label, ratio }, i (label)}
		{@const animating = entranceBaseDelay !== null}
		{@const itemDelay = animating ? entranceBaseDelay! + i * 70 : 0}
		{@const fillDelay = animating ? itemDelay + 60 : 0}
		{@const offset = !animating || entered ? fillOffset(ratio.health) : TRACK_LEN}
		<div
			class="entrance-item flex flex-col items-center px-3 py-3"
			style:--d="{itemDelay}ms"
		>
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
					d={trackPath}
					fill="none"
					stroke={healthColor(ratio.health)}
					stroke-width={STROKE}
					stroke-linecap="round"
					stroke-dasharray={TRACK_LEN}
					stroke-dashoffset={offset}
					style="transition: stroke-dashoffset 820ms cubic-bezier(0.22, 1, 0.36, 1) {fillDelay}ms"
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
