<script lang="ts">
	let { class: klass = 'h-4 w-4' }: { class?: string } = $props();
</script>

<svg class={klass} viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
	<line x1="6" y1="5" x2="17" y2="5" />
	<line x1="6" y1="10" x2="17" y2="10" />
	<line x1="6" y1="15" x2="17" y2="15" />
	<circle cx="3.5" cy="5" r="0.8" fill="currentColor" stroke="none" />
	<circle cx="3.5" cy="10" r="0.8" fill="currentColor" stroke="none" />
	<circle cx="3.5" cy="15" r="0.8" fill="currentColor" stroke="none" />
</svg>
