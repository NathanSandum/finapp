<script lang="ts">
	import { onMount } from 'svelte';

	let {
		before,
		after,
		delay = 0,
		animate = true
	}: {
		before: string;
		after: string;
		delay?: number;
		animate?: boolean;
	} = $props();

	let rolled = $state(!animate);

	onMount(() => {
		if (!animate) return;
		requestAnimationFrame(() => requestAnimationFrame(() => (rolled = true)));
	});

	function nbsp(c: string) {
		return c === '' || c === ' ' ? '\u00A0' : c;
	}
</script>

<span class="reel" class:rolled aria-hidden="true">
	<span class="track" style="transition-delay: {delay}ms">
		<span class="cell">{nbsp(before)}</span>
		<span class="cell">{nbsp(after)}</span>
	</span>
</span>

<style>
	.reel {
		display: inline-block;
		overflow: hidden;
		line-height: 1.1;
		height: 1.1em;
		vertical-align: top;
	}
	.track {
		display: flex;
		flex-direction: column;
		transition: transform 380ms cubic-bezier(0.7, 0, 0.25, 1);
	}
	.reel.rolled .track {
		transform: translateY(-50%);
	}
	.cell {
		display: block;
		line-height: 1.1;
		height: 1.1em;
	}
	@media (prefers-reduced-motion: reduce) {
		.track {
			transition: transform 0ms;
		}
	}
</style>
