<script lang="ts">
	import { onMount } from 'svelte';
	import { buildLine } from '$lib/views/income/lib';
	import { signinTrail } from '$lib/data/signinTrail';
	import { prefersReducedMotion } from '$lib/utils/motion';

	let { exiting = false }: { exiting?: boolean } = $props();

	const sparkline = buildLine(signinTrail, 800, 360);

	let pathEl = $state<SVGPolylineElement | null>(null);
	let revealing = $state(false);
	let pathLength = $state(0);

	onMount(() => {
		if (!pathEl) return;
		if (prefersReducedMotion()) {
			revealing = true;
			return;
		}
		const length = pathEl.getTotalLength();
		pathLength = length;
		pathEl.style.strokeDasharray = `${length}`;
		pathEl.style.strokeDashoffset = `${length}`;
		pathEl.getBoundingClientRect();
		pathEl.style.transition = 'stroke-dashoffset 1400ms cubic-bezier(0.22, 1, 0.36, 1)';
		pathEl.style.strokeDashoffset = '0';
		revealing = true;
	});

	$effect(() => {
		if (!exiting || !pathEl || pathLength === 0) return;
		if (prefersReducedMotion()) return;
		pathEl.style.transition = 'stroke-dashoffset 700ms cubic-bezier(0.4, 0, 1, 1)';
		pathEl.style.strokeDashoffset = `${-pathLength}`;
	});
</script>

<svg
	class="absolute inset-0 h-full w-full"
	viewBox="0 0 800 400"
	preserveAspectRatio="none"
	aria-hidden="true"
>
	<defs>
		<linearGradient id="signinFill" x1="0" y1="0" x2="0" y2="1">
			<stop offset="0%" style:stop-color="var(--color-positive-line)" stop-opacity="0.30" />
			<stop offset="100%" style:stop-color="var(--color-positive-line)" stop-opacity="0" />
		</linearGradient>
	</defs>
	<polygon
		class="signin-fill"
		class:revealing={revealing && !exiting}
		class:exiting
		fill="url(#signinFill)"
		points={sparkline.area}
		transform="translate(0,40)"
	/>
	<polyline
		bind:this={pathEl}
		fill="none"
		class="stroke-positive-line"
		stroke-width="1.25"
		stroke-linecap="round"
		stroke-linejoin="round"
		points={sparkline.path}
		transform="translate(0,40)"
	/>
</svg>

<style>
	.signin-fill {
		clip-path: inset(0 100% 0 0);
		transition: clip-path 1400ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.signin-fill.revealing {
		clip-path: inset(0 0 0 0);
	}
	.signin-fill.exiting {
		clip-path: inset(0 0 0 100%);
		transition: clip-path 700ms cubic-bezier(0.4, 0, 1, 1);
	}
	@media (prefers-reduced-motion: reduce) {
		.signin-fill,
		.signin-fill.revealing,
		.signin-fill.exiting {
			transition-duration: 0ms;
		}
	}
</style>
