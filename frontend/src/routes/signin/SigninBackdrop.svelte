<script lang="ts">
	import { buildLine } from '$lib/views/income/lib';
	import { signinTrail } from '$lib/data/signinTrail';

	const sparkline = buildLine(signinTrail, 800, 360);
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
	<polygon fill="url(#signinFill)" points={sparkline.area} transform="translate(0,40)" />
	<polyline
		fill="none"
		class="stroke-positive-line"
		stroke-width="1.25"
		stroke-linecap="round"
		stroke-linejoin="round"
		points={sparkline.path}
		transform="translate(0,40)"
	/>
</svg>
