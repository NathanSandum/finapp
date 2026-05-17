<script lang="ts">
	import { buildLine } from '$lib/views/income/lib';
	import { fmt, pct, shortFmt } from '$lib/format';

	let {
		label,
		value,
		prior,
		trail,
		trailLabels,
		gradientId
	}: {
		label: string;
		value: number;
		prior: number;
		trail: number[];
		trailLabels: string[];
		gradientId: string;
	} = $props();

	const sparkline = $derived(buildLine(trail, 560, 80));
	const min = $derived(Math.min(...trail));
	const max = $derived(Math.max(...trail));
</script>

<section class="relative shrink-0 overflow-hidden border border-border bg-surface-card">
	<div class="absolute left-5 top-4 z-10">
		<p class="text-[10px] font-medium uppercase tracking-wide text-ink-muted">{label}</p>
		<p
			class="mt-0.5 flex items-baseline gap-2 font-mono text-3xl font-semibold tabular-nums text-ink"
		>
			{fmt(value)}
			<span class="text-[13px] font-semibold text-positive">{pct(value, prior)}</span>
		</p>
	</div>
	<svg width="100%" viewBox="0 0 560 100" class="overflow-visible" preserveAspectRatio="none">
		<defs>
			<linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
				<stop offset="0%" style:stop-color="var(--color-positive-line)" stop-opacity="0.30" />
				<stop offset="100%" style:stop-color="var(--color-positive-line)" stop-opacity="0" />
			</linearGradient>
		</defs>
		<polygon fill="url(#{gradientId})" points={sparkline.area} transform="translate(0,20)" />
		<polyline
			fill="none"
			class="stroke-positive-line"
			stroke-width="1.25"
			stroke-linecap="round"
			stroke-linejoin="round"
			points={sparkline.path}
			transform="translate(0,20)"
		/>
		{#if sparkline.points.length > 0}
			<circle
				cx={sparkline.points[sparkline.points.length - 1].x}
				cy={sparkline.points[sparkline.points.length - 1].y + 20}
				r="3"
				class="fill-positive-line"
			/>
		{/if}
	</svg>
	<div
		class="pointer-events-none absolute right-2 top-4 flex h-[80px] flex-col justify-between font-mono text-[9px] tabular-nums text-ink-faint"
	>
		<span>{shortFmt(max)}</span>
		<span>{shortFmt(min)}</span>
	</div>
	<div
		class="grid px-5 pb-2 font-mono text-[9px] tabular-nums text-ink-faint"
		style:grid-template-columns="repeat({trailLabels.length}, 1fr)"
	>
		{#each trailLabels as label}<span class="text-center">{label}</span>{/each}
	</div>
</section>
