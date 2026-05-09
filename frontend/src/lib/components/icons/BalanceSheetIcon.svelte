<script lang="ts">
	let { class: klass = 'h-4 w-4' }: { class?: string } = $props();
</script>

<svg class={klass} viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
	<path d="M10 3v14" />
	<path d="M4 7l-1 4h6l-1-4" />
	<path d="M16 7l-1 4h6l-1-4" transform="translate(-4 0)" />
	<path d="M5 17h10" />
</svg>
