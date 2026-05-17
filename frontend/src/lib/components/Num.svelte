<script lang="ts">
	import { discreteState } from '$lib/ui/discrete.svelte';

	let { text }: { text: string } = $props();

	const parsed = $derived.by(() => {
		const match = text.match(/^([+\-]?)(\$?)(.*)$/);
		return {
			head: (match?.[1] ?? '') + (match?.[2] ?? ''),
			body: match?.[3] ?? text
		};
	});

	const chars = $derived(parsed.body.split(''));
	const on = $derived(discreteState.on);
</script>

<span class="num">{parsed.head}{#each chars as ch, i (i)}{@const survivor = i === 0 || Math.floor((i * 3) / chars.length) > Math.floor(((i - 1) * 3) / chars.length)}<span class="ch" class:collapsed={on && !survivor}><span class="real" class:hidden={on}>{ch}</span><span class="dot" class:shown={on} aria-hidden="true">•</span></span>{/each}</span>

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
	.real {
		display: inline-block;
		transform-origin: center;
		transform: scale(1);
		opacity: 1;
		transition:
			transform 320ms cubic-bezier(0.22, 1, 0.36, 1) 320ms,
			opacity 320ms cubic-bezier(0.22, 1, 0.36, 1) 320ms;
	}
	.real.hidden {
		transform: scale(0.15);
		opacity: 0;
		transition:
			transform 320ms cubic-bezier(0.55, 0, 0.65, 0.4),
			opacity 320ms cubic-bezier(0.55, 0, 0.65, 0.4);
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
</style>
