<script lang="ts">
	import { getDiscrete } from '$lib/ui/discrete.svelte';

	type Mode = 'up' | 'down' | 'none';
	type Cell = { key: number; before: string; after: string; mode: Mode };

	let { cells }: { cells: Cell[] } = $props();

	const on = $derived(getDiscrete());
</script>

{#each cells as cell, i (cells.length - 1 - i)}{@const survivor =
		i === 0 ||
		Math.floor((i * 3) / cells.length) > Math.floor(((i - 1) * 3) / cells.length)}<span
		class="ch"
		class:collapsed={on && !survivor}
		><span class="reel" class:hidden={on}
			>{#key cell.key}<span class="track mode-{cell.mode}"
					>{#if cell.mode === 'down'}<span class="cellch">{cell.after}</span><span class="cellch"
							>{cell.before}</span
						>{:else}<span class="cellch">{cell.before}</span><span class="cellch"
							>{cell.after}</span
						>{/if}</span
				>{/key}</span
		><span class="dot" class:shown={on} aria-hidden="true">•</span></span
	>{/each}

<style>
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
