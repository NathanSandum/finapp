<script lang="ts">
	let { class: klass = 'h-4 w-4' }: { class?: string } = $props();
</script>

<svg
	class={klass}
	viewBox="0 0 20 20"
	fill="none"
	stroke="currentColor"
	stroke-width="1.6"
	stroke-linecap="round"
	stroke-linejoin="round"
	aria-hidden="true"
>
	<line x1="4" y1="4" x2="4" y2="16" />
	<line x1="4" y1="6" x2="9" y2="6" />
	<line x1="4" y1="10" x2="13" y2="10" />
	<line x1="4" y1="14" x2="11" y2="14" />
	<circle cx="15" cy="6" r="0.9" fill="currentColor" stroke="none" />
	<circle cx="15" cy="10" r="0.9" fill="currentColor" stroke="none" />
	<circle cx="13" cy="14" r="0.9" fill="currentColor" stroke="none" />
</svg>
