<script lang="ts">
	import { untrack } from 'svelte';
	import { getDiscrete } from '$lib/ui/discrete.svelte';
	import { prefersReducedMotion } from '$lib/utils/motion';

	let { text }: { text: string } = $props();

	type Mode = 'up' | 'down' | 'none';
	type Cell = { key: number; before: string; after: string; mode: Mode };

	function parseHead(t: string) {
		const match = t.match(/^([+\-\u2212]?)(\$?)(.*)$/);
		return {
			head: (match?.[1] ?? '') + (match?.[2] ?? ''),
			body: match?.[3] ?? t
		};
	}

	function parseNum(t: string): number {
		const cleaned = t.replace(/\u2212/g, '-').replace(/[^0-9.\-]/g, '');
		const n = parseFloat(cleaned);
		return isNaN(n) ? 0 : n;
	}

	const parsed = $derived(parseHead(text));

	let nextKey = 1;
	let prevBody = untrack(() => parseHead(text).body);
	let prevNum = untrack(() => parseNum(text));
	let cells = $state<Cell[]>(
		untrack(() =>
			parseHead(text)
				.body.split('')
				.map((ch) => ({
					key: nextKey++,
					before: ch,
					after: ch,
					mode: 'none' as Mode
				}))
		)
	);

	$effect(() => {
		const body = parsed.body;
		if (body === prevBody) return;
		const num = parseNum(text);
		const reduce = prefersReducedMotion();
		const dir: Mode = reduce ? 'none' : num >= prevNum ? 'up' : 'down';
		const oldChars = prevBody.split('');
		const newChars = body.split('');
		const oldLen = oldChars.length;
		const newLen = newChars.length;
		cells = newChars.map((ch, j) => {
			const rightOffset = newLen - 1 - j;
			const oldIdx = oldLen - 1 - rightOffset;
			const before = oldIdx >= 0 ? oldChars[oldIdx] : ch;
			const isNew = oldIdx < 0;
			const mode: Mode = isNew || before === ch ? 'none' : dir;
			return { key: nextKey++, before, after: ch, mode };
		});
		prevBody = body;
		prevNum = num;
	});

	const on = $derived(getDiscrete());
</script>

<span class="num"
	>{parsed.head}{#each cells as cell, i (cells.length - 1 - i)}{@const survivor =
			i === 0 ||
			Math.floor((i * 3) / cells.length) > Math.floor(((i - 1) * 3) / cells.length)}<span
			class="ch"
			class:collapsed={on && !survivor}
			><span class="reel" class:hidden={on}
				>{#key cell.key}<span class="track mode-{cell.mode}"
						>{#if cell.mode === 'down'}<span class="cellch">{cell.after}</span><span
								class="cellch">{cell.before}</span
							>{:else}<span class="cellch">{cell.before}</span><span class="cellch"
								>{cell.after}</span
							>{/if}</span
					>{/key}</span
			><span class="dot" class:shown={on} aria-hidden="true">•</span></span
		>{/each}</span
>

<style>
	.num {
		white-space: nowrap;
	}
	.ch {
		display: inline-flex;
		align-items: baseline;
		justify-content: center;
		position: relative;
		vertical-align: baseline;
		width: 1ch;
		overflow: hidden;
		transition: width 320ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.ch.collapsed {
		width: 0;
		transition: width 320ms cubic-bezier(0.22, 1, 0.36, 1) 320ms;
	}
	.reel {
		display: inline-block;
		overflow: hidden;
		line-height: 1.1;
		height: 1.1em;
		width: 1ch;
		text-align: center;
		vertical-align: baseline;
		transform-origin: center;
		transform: scale(1);
		opacity: 1;
		transition:
			transform 320ms cubic-bezier(0.22, 1, 0.36, 1) 320ms,
			opacity 320ms cubic-bezier(0.22, 1, 0.36, 1) 320ms;
	}
	.reel.hidden {
		transform: scale(0.15);
		opacity: 0;
		transition:
			transform 320ms cubic-bezier(0.55, 0, 0.65, 0.4),
			opacity 320ms cubic-bezier(0.55, 0, 0.65, 0.4);
	}
	.track {
		display: flex;
		flex-direction: column;
		will-change: transform;
	}
	.cellch {
		display: block;
		line-height: 1.1;
		height: 1.1em;
		white-space: nowrap;
	}
	.track.mode-none .cellch:first-child {
		display: none;
	}
	.track.mode-up {
		animation: roll-up 520ms cubic-bezier(0.32, 0.72, 0, 1) both;
	}
	.track.mode-down {
		animation: roll-down 520ms cubic-bezier(0.32, 0.72, 0, 1) both;
	}

	@keyframes roll-up {
		from {
			transform: translate3d(0, 0, 0);
		}
		to {
			transform: translate3d(0, -1.1em, 0);
		}
	}
	@keyframes roll-down {
		from {
			transform: translate3d(0, -1.1em, 0);
		}
		to {
			transform: translate3d(0, 0, 0);
		}
	}

	.dot {
		position: absolute;
		left: 0;
		right: 0;
		top: 0;
		text-align: center;
		transform-origin: center;
		transform: scale(0.15);
		opacity: 0;
		transition:
			transform 320ms cubic-bezier(0.55, 0, 0.65, 0.4),
			opacity 320ms cubic-bezier(0.55, 0, 0.65, 0.4);
	}
	.dot.shown {
		transform: scale(1);
		opacity: 1;
		transition:
			transform 320ms cubic-bezier(0.22, 1, 0.36, 1) 320ms,
			opacity 320ms cubic-bezier(0.22, 1, 0.36, 1) 320ms;
	}

	@media (prefers-reduced-motion: reduce) {
		.track.mode-up,
		.track.mode-down {
			animation: none;
		}
		.track .cellch:first-child {
			display: none;
		}
	}
</style>
