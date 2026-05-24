import { cubicOut, expoOut } from 'svelte/easing';
import { crossfade } from 'svelte/transition';
import type { TransitionConfig } from 'svelte/transition';

export function prefersReducedMotion(): boolean {
	if (typeof window === 'undefined') return false;
	return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

export const DURATION = { fast: 180, base: 240, slow: 320 };

const reduced: TransitionConfig = { duration: 0 };

type RowOptions = { duration?: number; delay?: number };

export function rowEnter(_: Element, options: RowOptions = {}): TransitionConfig {
	if (prefersReducedMotion()) return reduced;
	const { duration = DURATION.base, delay = 0 } = options;
	return {
		duration,
		delay,
		easing: expoOut,
		css: (t) => `opacity: ${t}; transform: translateY(${(1 - t) * 8}px);`
	};
}

export function rowExit(_: Element, options: RowOptions = {}): TransitionConfig {
	if (prefersReducedMotion()) return reduced;
	const { duration = DURATION.fast, delay = 0 } = options;
	return {
		duration,
		delay,
		easing: cubicOut,
		css: (t) => `opacity: ${t}; transform: translateX(${(1 - t) * 16}px);`
	};
}

type PanelOptions = { duration?: number; delay?: number; offset?: number };

export function panelEnter(_: Element, options: PanelOptions = {}): TransitionConfig {
	if (prefersReducedMotion()) return reduced;
	const { duration = DURATION.slow, delay = 0, offset = 12 } = options;
	return {
		duration,
		delay,
		easing: expoOut,
		css: (t) => `opacity: ${t}; transform: translateX(${(1 - t) * offset}px);`
	};
}

export function panelExit(_: Element, options: PanelOptions = {}): TransitionConfig {
	if (prefersReducedMotion()) return reduced;
	const { duration = DURATION.fast, delay = 0, offset = 12 } = options;
	return {
		duration,
		delay,
		easing: cubicOut,
		css: (t) => `opacity: ${t}; transform: translateX(${-(1 - t) * offset}px);`
	};
}

export function expandHeight(node: Element, options: { duration?: number } = {}): TransitionConfig {
	if (prefersReducedMotion()) return reduced;
	const { duration = DURATION.base } = options;
	const height = (node as HTMLElement).getBoundingClientRect().height;
	return {
		duration,
		easing: expoOut,
		css: (t) =>
			`opacity: ${t}; max-height: ${t * height}px; overflow: hidden;`
	};
}

/// Shared crossfade keyed by item id. Use `send` on the outgoing element and
/// `receive` on the incoming element to morph between two locations.
export const [crossfadeSend, crossfadeReceive] = crossfade({
	duration: DURATION.slow,
	easing: expoOut,
	fallback(node, _params, intro) {
		return intro ? rowEnter(node) : rowExit(node);
	}
});
