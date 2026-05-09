<script lang="ts">
	let { class: klass = 'h-4 w-4' }: { class?: string } = $props();
</script>

<svg class={klass} viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
	<rect x="3" y="3" width="6" height="8" rx="1" />
	<rect x="11" y="3" width="6" height="4" rx="1" />
	<rect x="11" y="9" width="6" height="8" rx="1" />
	<rect x="3" y="13" width="6" height="4" rx="1" />
</svg>
