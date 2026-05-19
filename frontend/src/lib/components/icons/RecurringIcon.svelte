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
	<path d="M4 10a6 6 0 0 1 10.2-4.2" />
	<path d="M14 3v3h-3" />
	<path d="M16 10a6 6 0 0 1-10.2 4.2" />
	<path d="M6 17v-3h3" />
</svg>
