<script lang="ts">
	import type { TrailSlots } from './trailSlots.svelte';

	let {
		trailSlots,
		keys,
		width,
		height,
		lineClass,
		strokeWidth = 1.25,
		showArea = false,
		gradientId,
		showMarker = false,
		markerClass,
		showCurrentLine = false,
		currentLineClass,
		yOffset = 0
	}: {
		trailSlots: TrailSlots;
		keys: string[];
		width: number;
		height: number;
		lineClass: string;
		strokeWidth?: number;
		showArea?: boolean;
		gradientId?: string;
		showMarker?: boolean;
		markerClass?: string;
		showCurrentLine?: boolean;
		currentLineClass?: string;
		yOffset?: number;
	} = $props();

	function yAtX(targetX: number, pts: { x: number; y: number }[]): number {
		if (pts.length === 0) return 0;
		if (targetX <= pts[0].x) return pts[0].y;
		if (targetX >= pts[pts.length - 1].x) return pts[pts.length - 1].y;
		for (let i = 0; i < pts.length - 1; i++) {
			const a = pts[i];
			const b = pts[i + 1];
			if (targetX >= a.x && targetX <= b.x) {
				const t = b.x === a.x ? 0 : (targetX - a.x) / (b.x - a.x);
				return a.y + (b.y - a.y) * t;
			}
		}
		return pts[pts.length - 1].y;
	}

	const stepXCurrent = $derived(keys.length > 0 ? width / keys.length : width);
	const sortedSlots = $derived(
		[...trailSlots.slots].sort((a, b) => a.pos.current.x - b.pos.current.x)
	);
	const linePoints = $derived(
		sortedSlots.map((s) => ({ x: s.pos.current.x, y: s.pos.current.y }))
	);
	const path = $derived(
		linePoints.map((p) => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' ')
	);
	const area = $derived.by(() => {
		if (linePoints.length === 0) return `0,${height} ${width.toFixed(1)},${height}`;
		const first = linePoints[0];
		const last = linePoints[linePoints.length - 1];
		return `${first.x.toFixed(1)},${height} ${path} ${last.x.toFixed(1)},${height}`;
	});
	const markers = $derived(
		keys.map((k, i) => {
			const x = (i + 0.5) * stepXCurrent;
			return { key: k, x, y: yAtX(x, linePoints) };
		})
	);
	const currentX = $derived(keys.length > 0 ? width - stepXCurrent / 2 : 0);
</script>

{#if showArea && gradientId}
	<defs>
		<linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
			<stop offset="0%" style:stop-color="var(--color-positive-line)" stop-opacity="0.30" />
			<stop offset="100%" style:stop-color="var(--color-positive-line)" stop-opacity="0" />
		</linearGradient>
	</defs>
	<polygon fill="url(#{gradientId})" points={area} transform="translate(0,{yOffset})" />
{/if}
<polyline
	fill="none"
	class={lineClass}
	stroke-width={strokeWidth}
	stroke-linecap="round"
	stroke-linejoin="round"
	points={path}
	transform="translate(0,{yOffset})"
/>
{#if showCurrentLine && keys.length > 0}
	<line
		x1={currentX}
		x2={currentX}
		y1={yOffset}
		y2={height + yOffset}
		stroke-width="1"
		stroke-dasharray="2 3"
		class={currentLineClass}
	/>
{/if}
{#if showMarker}
	{#each markers as m (m.key)}
		<circle cx={m.x} cy={m.y + yOffset} r="2" class={markerClass} />
	{/each}
{/if}
