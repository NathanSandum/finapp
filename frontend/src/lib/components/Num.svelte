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
</script>

<span class="num">
	<span class="head">{parsed.head}</span><span class="body" class:on={discreteState.on}
		>{#each chars as ch, i (i)}<span class="ch" class:collapse={i >= 3}
				><span class="layer real">{ch}</span><span class="layer dot" aria-hidden="true">•</span
				></span
			>{/each}</span
	>
</span>

<style>
	.num {
		display: inline-flex;
		align-items: baseline;
		white-space: nowrap;
	}
	.body {
		display: inline-flex;
		align-items: baseline;
	}
	.ch {
		display: inline-block;
		position: relative;
		width: 1ch;
		overflow: hidden;
		transition: width 200ms ease;
	}
	.body.on .ch.collapse {
		width: 0;
		transition: width 200ms 200ms ease;
	}
	.layer {
		display: inline-block;
	}
	.layer.real {
		opacity: 1;
		transition: opacity 200ms 200ms ease;
	}
	.body.on .layer.real {
		opacity: 0;
		transition: opacity 200ms ease;
	}
	.layer.dot {
		position: absolute;
		left: 0;
		top: 0;
		opacity: 0;
		transition: opacity 200ms 200ms ease;
	}
	.body.on .layer.dot {
		opacity: 1;
		transition: opacity 200ms ease;
	}
</style>
