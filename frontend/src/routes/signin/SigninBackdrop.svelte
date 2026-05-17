<script lang="ts">
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';
	import { buildLine } from '$lib/views/income/lib';
	import { signinTrail } from '$lib/data/signinTrail';
	import { prefersReducedMotion } from '$lib/utils/motion';

	const sparkline = buildLine(signinTrail, 800, 360);

	let pathEl = $state<SVGPolylineElement | null>(null);
	let drawn = $state(false);

	onMount(() => {
		if (!pathEl) return;
		if (prefersReducedMotion()) {
			drawn = true;
			return;
		}
		const length = pathEl.getTotalLength();
		pathEl.style.strokeDasharray = `${length}`;
		pathEl.style.strokeDashoffset = `${length}`;
		pathEl.getBoundingClientRect();
		pathEl.style.transition = 'stroke-dashoffset 1400ms cubic-bezier(0.22, 1, 0.36, 1)';
		pathEl.style.strokeDashoffset = '0';
		const onEnd = () => {
			drawn = true;
			pathEl?.removeEventListener('transitionend', onEnd);
		};
		pathEl.addEventListener('transitionend', onEnd);
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
	{#if drawn}
		<polygon
			fill="url(#signinFill)"
			points={sparkline.area}
			transform="translate(0,40)"
			in:fade={{ duration: 600, easing: cubicOut }}
		/>
	{/if}
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
