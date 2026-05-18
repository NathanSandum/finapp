import { untrack } from 'svelte';
import { Tween } from 'svelte/motion';
import { cubicOut } from 'svelte/easing';
import { prefersReducedMotion } from '$lib/utils/motion';
import { inferKeyGranularity, parentKeyInGran, type Granularity } from '$lib/views/period';

export type TrailSlot = {
	key: string;
	pos: Tween<{ x: number; y: number }>;
};

function granRank(g: Granularity | undefined): number {
	if (g === 'year') return 0;
	if (g === 'quarter') return 1;
	if (g === 'month') return 2;
	return -1;
}

function timeOrder(key: string): number {
	const g = inferKeyGranularity(key);
	if (g === 'year') return parseInt(key) * 12 + 5.5;
	if (g === 'quarter') {
		const y = parseInt(key.slice(0, 4));
		const q = parseInt(key.slice(6));
		return y * 12 + (q - 1) * 3 + 1;
	}
	if (g === 'month') {
		const y = parseInt(key.slice(0, 4));
		const m = parseInt(key.slice(5, 7));
		return y * 12 + (m - 1) + 0.5;
	}
	return 0;
}

function layoutFor(values: number[], width: number, height: number): { x: number; y: number }[] {
	if (values.length === 0) return [];
	const min = Math.min(...values);
	const max = Math.max(...values);
	const range = max - min || 1;
	const stepX = width / values.length;
	return values.map((v, i) => ({
		x: (i + 0.5) * stepX,
		y: height - ((v - min) / range) * height
	}));
}

export class TrailSlots {
	slots = $state<TrailSlot[]>([]);
	#duration: number;

	constructor(duration = 520) {
		this.#duration = duration;
	}

	destroy() {}

	sync(keys: string[], values: number[], width: number, height: number) {
		untrack(() => this.#syncImpl(keys, values, width, height));
	}

	#syncImpl(keys: string[], values: number[], width: number, height: number) {
		const duration = this.#duration;
		const layout = layoutFor(values, width, height);
		const newLayoutByKey = new Map(keys.map((k, i) => [k, layout[i]]));
		const newKeySet = new Set(keys);
		const reduce = prefersReducedMotion();
		const stepX = keys.length > 0 ? width / keys.length : width;

		if (keys.length === 0) {
			this.slots = [];
			return;
		}

		if (this.slots.length === 0) {
			this.slots = keys.map((k, i) => ({
				key: k,
				pos: new Tween(layout[i], { duration, easing: cubicOut })
			}));
			return;
		}

		const oldSlots = this.slots;
		const oldSlotByKey = new Map(oldSlots.map((s) => [s.key, s]));
		const oldGran = inferKeyGranularity(oldSlots[0].key);
		const newGran = inferKeyGranularity(keys[0]);
		const granChange = !!(oldGran && newGran && oldGran !== newGran);

		const nextSlots: TrailSlot[] = [];

		if (granChange) {
			const isMerge = granRank(newGran) < granRank(oldGran);
			const oldStartTime = timeOrder(oldSlots[0].key);
			const oldEndTime = timeOrder(oldSlots[oldSlots.length - 1].key);
			const oldStartX = oldSlots[0].pos.current.x;
			const oldEndX = oldSlots[oldSlots.length - 1].pos.current.x;
			const oldRange = oldEndTime - oldStartTime || 1;

			const timeInterpStart = (k: string, targetY: number) => {
				const t = timeOrder(k);
				if (t < oldStartTime) return { x: -stepX, y: targetY };
				if (t > oldEndTime) return { x: width + stepX, y: targetY };
				const f = (t - oldStartTime) / oldRange;
				const x = oldStartX + (oldEndX - oldStartX) * f;
				let y = targetY;
				for (let j = 0; j < oldSlots.length - 1; j++) {
					const a = oldSlots[j];
					const b = oldSlots[j + 1];
					if (x >= a.pos.current.x && x <= b.pos.current.x) {
						const u = b.pos.current.x === a.pos.current.x
							? 0
							: (x - a.pos.current.x) / (b.pos.current.x - a.pos.current.x);
						y = a.pos.current.y + (b.pos.current.y - a.pos.current.y) * u;
						break;
					}
				}
				return { x, y };
			};

			for (let i = 0; i < keys.length; i++) {
				const k = keys[i];
				const target = newLayoutByKey.get(k)!;
				let startPos: { x: number; y: number };

				if (isMerge) {
					startPos = timeInterpStart(k, target.y);
				} else {
					const parent = parentKeyInGran(k, oldGran!);
					const parentSlot = parent ? oldSlotByKey.get(parent) : undefined;
					startPos = parentSlot
						? { x: parentSlot.pos.current.x, y: parentSlot.pos.current.y }
						: timeInterpStart(k, target.y);
				}

				const slot: TrailSlot = {
					key: k,
					pos: new Tween(startPos, { duration, easing: cubicOut })
				};
				if (reduce) slot.pos.set(target, { duration: 0 });
				else slot.pos.target = target;
				nextSlots.push(slot);
			}

			this.slots = nextSlots;
			return;
		}

		const survivorKeys = keys.filter((k) => oldSlotByKey.has(k));
		const sortedSurvivorKeys = [...survivorKeys].sort();
		const minSurvivor = sortedSurvivorKeys[0];
		const lastOldKey = oldSlots[oldSlots.length - 1].key;

		const survivors = survivorKeys
			.map((k) => oldSlotByKey.get(k)!)
			.sort((a, b) => a.pos.current.x - b.pos.current.x);
		const leftEdgeY = survivors[0]?.pos.current.y ?? layout[0]?.y;
		const rightEdgeY =
			survivors[survivors.length - 1]?.pos.current.y ?? layout[layout.length - 1]?.y;

		for (let i = 0; i < keys.length; i++) {
			const k = keys[i];
			const target = newLayoutByKey.get(k)!;
			const existing = oldSlotByKey.get(k);

			if (existing) {
				if (reduce) existing.pos.set(target, { duration: 0 });
				else existing.pos.target = target;
				nextSlots.push(existing);
				continue;
			}

			const enterRight =
				minSurvivor === undefined ? k > lastOldKey : k >= minSurvivor;
			const startX = enterRight ? width + stepX : -stepX;
			const startY = (enterRight ? rightEdgeY : leftEdgeY) ?? target.y;

			const slot: TrailSlot = {
				key: k,
				pos: new Tween({ x: startX, y: startY }, { duration, easing: cubicOut })
			};
			if (reduce) slot.pos.set(target, { duration: 0 });
			else slot.pos.target = target;
			nextSlots.push(slot);
		}

		this.slots = nextSlots;
	}
}
