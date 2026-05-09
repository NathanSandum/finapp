<script lang="ts">
	import { buildLine, fmt, pct } from '$lib/views/income/lib';

	let {
		label,
		value,
		prior,
		priorShortLabel,
		trail,
		trailLabels,
		gradientId,
		badgeSuffix
	}: {
		label: string;
		value: number;
		prior: number;
		priorShortLabel: string;
		trail: number[];
		trailLabels: string[];
		gradientId: string;
		badgeSuffix?: string;
	} = $props();

	const sparkline = $derived(buildLine(trail, 560, 80));
</script>

<section
	class="relative shrink-0 overflow-hidden rounded-xl border border-neutral-200 bg-white shadow-sm"
>
	<div class="absolute left-5 top-4 z-10">
		<p class="text-[10px] font-medium uppercase tracking-wide text-neutral-500">{label}</p>
		<p class="mt-0.5 font-mono text-3xl font-semibold tabular-nums text-neutral-900">{fmt(value)}</p>
		<span
			class="mt-1 inline-block rounded-md bg-emerald-50 px-2 py-0.5 font-mono text-[10px] font-semibold tabular-nums text-emerald-700"
		>
			{pct(value, prior)} vs {priorShortLabel}{#if badgeSuffix} · {badgeSuffix}{/if}
		</span>
	</div>
	<svg width="100%" viewBox="0 0 560 100" class="overflow-visible" preserveAspectRatio="none">
		<defs>
			<linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
				<stop offset="0%" stop-color="rgb(16 185 129)" stop-opacity="0.30" />
				<stop offset="100%" stop-color="rgb(16 185 129)" stop-opacity="0" />
			</linearGradient>
		</defs>
		<polygon fill="url(#{gradientId})" points={sparkline.area} transform="translate(0,20)" />
		<polyline
			fill="none"
			stroke="rgb(16 185 129)"
			stroke-width="2"
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
				fill="rgb(16 185 129)"
			/>
		{/if}
	</svg>
	<div
		class="grid px-5 pb-2 font-mono text-[9px] tabular-nums text-neutral-400"
		style:grid-template-columns="repeat({trailLabels.length}, 1fr)"
	>
		{#each trailLabels as label}<span class="text-center">{label}</span>{/each}
	</div>
</section>
