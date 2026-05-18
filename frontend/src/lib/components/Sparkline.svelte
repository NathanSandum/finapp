<script lang="ts" module>
	export type SparklineSlot = {
		key: string;
		pos: import('svelte/motion').Tween<{ x: number; y: number }>;
		exiting: boolean;
	};
</script>

<script lang="ts">
	import { onDestroy, untrack } from 'svelte';
	import { Tween } from 'svelte/motion';
	import { cubicOut } from 'svelte/easing';
	import { prefersReducedMotion } from '$lib/utils/motion';

	let {
		values,
		keys,
		width,
		height,
		lineClass,
		strokeWidth = 1.25,
		showArea = false,
		gradientId,
		showMarker = false,
		markerClass,
		duration = 520,
		yOffset = 0
	}: {
		values: number[];
		keys: string[];
		width: number;
		height: number;
		lineClass: string;
		strokeWidth?: number;
		showArea?: boolean;
		gradientId?: string;
		showMarker?: boolean;
		markerClass?: string;
		duration?: number;
		yOffset?: number;
	} = $props();

	function layoutFor(vs: number[]): { x: number; y: number }[] {
		if (vs.length === 0) return [];
		const min = Math.min(...vs);
		const max = Math.max(...vs);
		const range = max - min || 1;
		const stepX = width / vs.length;
		return vs.map((v, i) => ({
			x: (i + 0.5) * stepX,
			y: height - ((v - min) / range) * height
		}));
	}

	let slots = $state<SparklineSlot[]>(
		untrack(() => {
			const layout = layoutFor(values);
			return keys.map((key, i) => ({
				key,
				pos: new Tween(layout[i], { duration, easing: cubicOut }),
				exiting: false
			}));
		})
	);

	let prevKeys: string[] = untrack(() => [...keys]);
	const timers = new Set<ReturnType<typeof setTimeout>>();

	onDestroy(() => {
		for (const t of timers) clearTimeout(t);
	});

	$effect(() => {
		const nextKeys = keys;
		const nextValues = values;
		const layout = layoutFor(nextValues);
		const newLayoutByKey = new Map(nextKeys.map((k, i) => [k, layout[i]]));
		const newKeySet = new Set(nextKeys);

		const liveSlots = slots.filter((s) => !s.exiting);
		const oldKeySet = new Set(liveSlots.map((s) => s.key));

		let firstSurvivorNewIdx = -1;
		let lastSurvivorNewIdx = -1;
		for (let i = 0; i < nextKeys.length; i++) {
			if (oldKeySet.has(nextKeys[i])) {
				if (firstSurvivorNewIdx === -1) firstSurvivorNewIdx = i;
				lastSurvivorNewIdx = i;
			}
		}

		let firstSurvivorOldIdx = -1;
		let lastSurvivorOldIdx = -1;
		for (let i = 0; i < liveSlots.length; i++) {
			if (newKeySet.has(liveSlots[i].key)) {
				if (firstSurvivorOldIdx === -1) firstSurvivorOldIdx = i;
				lastSurvivorOldIdx = i;
			}
		}

		let newOnRight = true;
		if (firstSurvivorNewIdx === -1 && liveSlots.length > 0 && nextKeys.length > 0) {
			newOnRight = nextKeys[0] > liveSlots[liveSlots.length - 1].key;
		}

		const enteringLeft: { key: string; idx: number }[] = [];
		const enteringRight: { key: string; idx: number }[] = [];
		if (firstSurvivorNewIdx === -1) {
			for (let i = 0; i < nextKeys.length; i++) {
				if (newOnRight) enteringRight.push({ key: nextKeys[i], idx: i });
				else enteringLeft.push({ key: nextKeys[i], idx: i });
			}
		} else {
			for (let i = 0; i < firstSurvivorNewIdx; i++)
				enteringLeft.push({ key: nextKeys[i], idx: i });
			for (let i = lastSurvivorNewIdx + 1; i < nextKeys.length; i++)
				enteringRight.push({ key: nextKeys[i], idx: i });
		}

		const exitingLeft: SparklineSlot[] = [];
		const exitingRight: SparklineSlot[] = [];
		if (firstSurvivorOldIdx === -1) {
			if (newOnRight) exitingLeft.push(...liveSlots);
			else exitingRight.push(...liveSlots);
		} else {
			for (let i = 0; i < firstSurvivorOldIdx; i++) exitingLeft.push(liveSlots[i]);
			for (let i = lastSurvivorOldIdx + 1; i < liveSlots.length; i++)
				exitingRight.push(liveSlots[i]);
		}

		const sameKeys =
			nextKeys.length === prevKeys.length && nextKeys.every((k, i) => k === prevKeys[i]);
		if (sameKeys) {
			let anyChange = false;
			for (const slot of liveSlots) {
				const target = newLayoutByKey.get(slot.key);
				if (!target) continue;
				const tgt = slot.pos.target;
				if (Math.abs(target.x - tgt.x) > 0.001 || Math.abs(target.y - tgt.y) > 0.001) {
					anyChange = true;
					break;
				}
			}
			if (!anyChange) {
				prevKeys = [...nextKeys];
				return;
			}
		}

		const reduce = prefersReducedMotion();
		const stepX = nextKeys.length > 0 ? width / nextKeys.length : width;

		for (const slot of liveSlots) {
			const target = newLayoutByKey.get(slot.key);
			if (!target) continue;
			if (reduce) slot.pos.set(target, { duration: 0 });
			else slot.pos.target = target;
		}

		for (let i = 0; i < exitingLeft.length; i++) {
			const slot = exitingLeft[i];
			slot.exiting = true;
			const offX = (-(exitingLeft.length - i) + 0.5) * stepX;
			const target = { x: offX, y: slot.pos.current.y };
			if (reduce) slot.pos.set(target, { duration: 0 });
			else slot.pos.target = target;
		}

		for (let i = 0; i < exitingRight.length; i++) {
			const slot = exitingRight[i];
			slot.exiting = true;
			const offX = width + (i + 0.5) * stepX;
			const target = { x: offX, y: slot.pos.current.y };
			if (reduce) slot.pos.set(target, { duration: 0 });
			else slot.pos.target = target;
		}

		const newSlots: SparklineSlot[] = [];
		const leftEdgeY = liveSlots[0]?.pos.current.y;
		const rightEdgeY = liveSlots[liveSlots.length - 1]?.pos.current.y;
		for (let i = 0; i < enteringLeft.length; i++) {
			const e = enteringLeft[i];
			const target = newLayoutByKey.get(e.key)!;
			const startX = (-(enteringLeft.length - i) + 0.5) * stepX;
			const startY = leftEdgeY ?? target.y;
			const slot: SparklineSlot = {
				key: e.key,
				pos: new Tween({ x: startX, y: startY }, { duration, easing: cubicOut }),
				exiting: false
			};
			if (reduce) slot.pos.set(target, { duration: 0 });
			else slot.pos.target = target;
			newSlots.push(slot);
		}
		for (let i = 0; i < enteringRight.length; i++) {
			const e = enteringRight[i];
			const target = newLayoutByKey.get(e.key)!;
			const startX = width + (i + 0.5) * stepX;
			const startY = rightEdgeY ?? target.y;
			const slot: SparklineSlot = {
				key: e.key,
				pos: new Tween({ x: startX, y: startY }, { duration, easing: cubicOut }),
				exiting: false
			};
			if (reduce) slot.pos.set(target, { duration: 0 });
			else slot.pos.target = target;
			newSlots.push(slot);
		}

		if (newSlots.length > 0) slots = [...slots, ...newSlots];

		if (exitingLeft.length > 0 || exitingRight.length > 0) {
			const toRemove = new Set([...exitingLeft, ...exitingRight]);
			if (reduce) {
				slots = slots.filter((s) => !toRemove.has(s));
			} else {
				const timer = setTimeout(() => {
					timers.delete(timer);
					slots = slots.filter((s) => !toRemove.has(s));
				}, duration + 50);
				timers.add(timer);
			}
		}

		prevKeys = [...nextKeys];
	});

	const sortedSlots = $derived([...slots].sort((a, b) => a.pos.current.x - b.pos.current.x));
	const path = $derived(
		sortedSlots
			.map((s) => `${s.pos.current.x.toFixed(1)},${s.pos.current.y.toFixed(1)}`)
			.join(' ')
	);
	const area = $derived.by(() => {
		if (sortedSlots.length === 0) return `0,${height} ${width.toFixed(1)},${height}`;
		const first = sortedSlots[0].pos.current;
		const last = sortedSlots[sortedSlots.length - 1].pos.current;
		return `${first.x.toFixed(1)},${height} ${path} ${last.x.toFixed(1)},${height}`;
	});
	const lastLive = $derived.by(() => {
		for (let i = sortedSlots.length - 1; i >= 0; i--) {
			if (!sortedSlots[i].exiting) return sortedSlots[i].pos.current;
		}
		return undefined;
	});
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
{#if showMarker && lastLive}
	<circle cx={lastLive.x} cy={lastLive.y + yOffset} r="3" class={markerClass} />
{/if}
