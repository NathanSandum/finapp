import { untrack } from 'svelte';
import { Tween } from 'svelte/motion';
import { cubicOut } from 'svelte/easing';
import { prefersReducedMotion } from '$lib/utils/motion';
import { inferKeyGranularity, parentKeyInGran, type Granularity } from '$lib/views/period';

export type TrailSlot = {
	key: string;
	pos: Tween<{ x: number; y: number }>;
};

type Point = { x: number; y: number };

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

function layoutFor(values: number[], width: number, height: number): Point[] {
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

function offscreenSideX(slotTime: number, referenceStart: number, stepX: number, width: number) {
	return slotTime < referenceStart ? -stepX : width + stepX;
}

export class TrailSlots {
	slots = $state<TrailSlot[]>([]);
	private duration: number;
	private pendingSwap = 0;

	constructor(duration = 520) {
		this.duration = duration;
	}

	destroy() {
		this.pendingSwap++;
	}

	sync(keys: string[], values: number[], width: number, height: number) {
		untrack(() => this.syncImpl(keys, values, width, height));
	}

	private fresh(keys: string[], layout: Point[]): TrailSlot[] {
		return keys.map((k, i) => ({
			key: k,
			pos: new Tween(layout[i], { duration: this.duration, easing: cubicOut })
		}));
	}

	private applyTarget(slot: TrailSlot, target: Point, reduce: boolean) {
		if (reduce) slot.pos.set(target, { duration: 0 });
		else slot.pos.target = target;
	}

	private syncImpl(keys: string[], values: number[], width: number, height: number) {
		const token = ++this.pendingSwap;
		const layout = layoutFor(values, width, height);

		if (keys.length === 0) {
			this.slots = [];
			return;
		}
		if (this.slots.length === 0) {
			this.slots = this.fresh(keys, layout);
			return;
		}

		const oldSlots = this.slots.filter((s) => !s.key.startsWith('__spawn:'));
		if (oldSlots.length === 0) {
			this.slots = this.fresh(keys, layout);
			return;
		}

		const oldGran = inferKeyGranularity(oldSlots[0].key);
		const newGran = inferKeyGranularity(keys[0]);
		const granChange = !!(oldGran && newGran && oldGran !== newGran);
		const reduce = prefersReducedMotion();
		const stepX = width / keys.length;

		if (granChange && granRank(newGran) < granRank(oldGran)) {
			this.syncMerge(keys, layout, oldSlots, newGran!, stepX, width, reduce, token);
			return;
		}
		if (granChange) {
			this.syncSplit(keys, layout, oldSlots, oldGran!, stepX, width, reduce);
			return;
		}
		this.syncSameGran(keys, layout, oldSlots, stepX, width, reduce);
	}

	private syncMerge(
		keys: string[],
		layout: Point[],
		oldSlots: TrailSlot[],
		newGran: Granularity,
		stepX: number,
		width: number,
		reduce: boolean,
		token: number
	) {
		const newLayoutByKey = new Map(keys.map((k, i) => [k, layout[i]]));
		const newStartTime = timeOrder(keys[0]);
		const oldStartTime = timeOrder(oldSlots[0].key);
		const parentSourceCount = new Map<string, number>();
		const combined: TrailSlot[] = [];

		for (const old of oldSlots) {
			const parent = parentKeyInGran(old.key, newGran);
			const parentLayout = parent ? newLayoutByKey.get(parent) : undefined;
			let target: Point;
			if (parent && parentLayout) {
				target = parentLayout;
				parentSourceCount.set(parent, (parentSourceCount.get(parent) ?? 0) + 1);
			} else {
				const x = offscreenSideX(timeOrder(old.key), newStartTime, stepX, width);
				target = { x, y: old.pos.current.y };
			}
			this.applyTarget(old, target, reduce);
			combined.push(old);
		}

		for (let i = 0; i < keys.length; i++) {
			const newKey = keys[i];
			if ((parentSourceCount.get(newKey) ?? 0) > 0) continue;
			const target = layout[i];
			const startX = offscreenSideX(timeOrder(newKey), oldStartTime, stepX, width);
			const slot: TrailSlot = {
				key: `__spawn:${newKey}`,
				pos: new Tween(
					{ x: startX, y: target.y },
					{ duration: this.duration, easing: cubicOut }
				)
			};
			this.applyTarget(slot, target, reduce);
			combined.push(slot);
		}

		this.slots = combined;
		this.scheduleFinalize(keys, layout, token, reduce);
	}

	private scheduleFinalize(keys: string[], layout: Point[], token: number, reduce: boolean) {
		const finalize = () => {
			if (token !== this.pendingSwap) return;
			this.slots = this.fresh(keys, layout);
		};
		if (reduce) finalize();
		else setTimeout(finalize, this.duration + 16);
	}

	private syncSplit(
		keys: string[],
		layout: Point[],
		oldSlots: TrailSlot[],
		oldGran: Granularity,
		stepX: number,
		width: number,
		reduce: boolean
	) {
		const oldSlotByKey = new Map(oldSlots.map((s) => [s.key, s]));
		const oldStartTime = timeOrder(oldSlots[0].key);
		const oldEndTime = timeOrder(oldSlots[oldSlots.length - 1].key);
		const oldStartX = oldSlots[0].pos.current.x;
		const oldEndX = oldSlots[oldSlots.length - 1].pos.current.x;
		const oldRange = oldEndTime - oldStartTime || 1;

		const timeInterpStart = (k: string, targetY: number): Point => {
			const t = timeOrder(k);
			if (t < oldStartTime) return { x: -stepX, y: targetY };
			if (t > oldEndTime) return { x: width + stepX, y: targetY };
			const f = (t - oldStartTime) / oldRange;
			const x = oldStartX + (oldEndX - oldStartX) * f;
			let y = targetY;
			for (let j = 0; j < oldSlots.length - 1; j++) {
				const a = oldSlots[j];
				const b = oldSlots[j + 1];
				if (x < a.pos.current.x || x > b.pos.current.x) continue;
				const u =
					b.pos.current.x === a.pos.current.x
						? 0
						: (x - a.pos.current.x) / (b.pos.current.x - a.pos.current.x);
				y = a.pos.current.y + (b.pos.current.y - a.pos.current.y) * u;
				break;
			}
			return { x, y };
		};

		const nextSlots: TrailSlot[] = keys.map((k, i) => {
			const target = layout[i];
			const parent = parentKeyInGran(k, oldGran);
			const parentSlot = parent ? oldSlotByKey.get(parent) : undefined;
			const startPos = parentSlot
				? { x: parentSlot.pos.current.x, y: parentSlot.pos.current.y }
				: timeInterpStart(k, target.y);
			const slot: TrailSlot = {
				key: k,
				pos: new Tween(startPos, { duration: this.duration, easing: cubicOut })
			};
			this.applyTarget(slot, target, reduce);
			return slot;
		});

		this.slots = nextSlots;
	}

	private syncSameGran(
		keys: string[],
		layout: Point[],
		oldSlots: TrailSlot[],
		stepX: number,
		width: number,
		reduce: boolean
	) {
		const oldSlotByKey = new Map(oldSlots.map((s) => [s.key, s]));
		const survivorKeys = keys.filter((k) => oldSlotByKey.has(k));
		const minSurvivor = [...survivorKeys].sort()[0];
		const lastOldKey = oldSlots[oldSlots.length - 1].key;

		const survivors = survivorKeys
			.map((k) => oldSlotByKey.get(k)!)
			.sort((a, b) => a.pos.current.x - b.pos.current.x);
		const leftEdgeY = survivors[0]?.pos.current.y ?? layout[0]?.y;
		const rightEdgeY =
			survivors[survivors.length - 1]?.pos.current.y ?? layout[layout.length - 1]?.y;

		const nextSlots: TrailSlot[] = keys.map((k, i) => {
			const target = layout[i];
			const existing = oldSlotByKey.get(k);
			if (existing) {
				this.applyTarget(existing, target, reduce);
				return existing;
			}
			const enterRight = minSurvivor === undefined ? k > lastOldKey : k >= minSurvivor;
			const startX = enterRight ? width + stepX : -stepX;
			const startY = (enterRight ? rightEdgeY : leftEdgeY) ?? target.y;
			const slot: TrailSlot = {
				key: k,
				pos: new Tween(
					{ x: startX, y: startY },
					{ duration: this.duration, easing: cubicOut }
				)
			};
			this.applyTarget(slot, target, reduce);
			return slot;
		});

		this.slots = nextSlots;
	}
}
