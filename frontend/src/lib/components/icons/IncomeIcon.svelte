<script lang="ts">
	let { class: klass = 'h-4 w-4' }: { class?: string } = $props();
</script>

<svg class={klass} viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
	<polyline points="3,14 8,9 11,12 17,5" />
	<polyline points="13,5 17,5 17,9" />
</svg>
