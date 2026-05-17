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

<span class="num">{parsed.head}{#each chars as ch, i (i)}<span class="ch" class:collapsed={on && i >= 3}><span class="real" class:hidden={on}>{ch}</span><span class="dot" class:shown={on} aria-hidden="true">•</span></span>{/each}</span>

<style>
	.num {
		white-space: nowrap;
	}
	.ch {
		display: inline-flex;
		align-items: baseline;
		position: relative;
		vertical-align: baseline;
		width: 1ch;
		overflow: hidden;
		transition: width 200ms ease;
	}
	.ch.collapsed {
		width: 0;
		transition: width 200ms 200ms ease;
	}
	.real {
		display: inline-block;
		opacity: 1;
		transition: opacity 200ms 200ms ease;
	}
	.real.hidden {
		opacity: 0;
		transition: opacity 200ms ease;
	}
	.dot {
		position: absolute;
		left: 0;
		top: 0;
		opacity: 0;
		transition: opacity 200ms 200ms ease;
	}
	.dot.shown {
		opacity: 1;
		transition: opacity 200ms ease;
	}
</style>
