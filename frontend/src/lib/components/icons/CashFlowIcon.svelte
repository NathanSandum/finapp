<script lang="ts">
	let { class: klass = 'h-4 w-4' }: { class?: string } = $props();
</script>

<svg class={klass} viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
	<path d="M4 7h10l-2-2" />
	<path d="M14 7l-2 2" />
	<path d="M16 13H6l2 2" />
	<path d="M6 13l2-2" />
</svg>
