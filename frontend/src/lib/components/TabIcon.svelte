<script lang="ts">
	let { id, class: klass = 'h-4 w-4' }: { id: string; class?: string } = $props();
</script>

{#if id === 'overview'}
	<svg class={klass} viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
		<rect x="3" y="3" width="6" height="8" rx="1" />
		<rect x="11" y="3" width="6" height="4" rx="1" />
		<rect x="11" y="9" width="6" height="8" rx="1" />
		<rect x="3" y="13" width="6" height="4" rx="1" />
	</svg>
{:else if id === 'income'}
	<svg class={klass} viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
		<polyline points="3,14 8,9 11,12 17,5" />
		<polyline points="13,5 17,5 17,9" />
	</svg>
{:else if id === 'cash-flow'}
	<svg class={klass} viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
		<path d="M4 7h10l-2-2" />
		<path d="M14 7l-2 2" />
		<path d="M16 13H6l2 2" />
		<path d="M6 13l2-2" />
	</svg>
{:else if id === 'balance-sheet'}
	<svg class={klass} viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
		<path d="M10 3v14" />
		<path d="M4 7l-1 4h6l-1-4" />
		<path d="M16 7l-1 4h6l-1-4" transform="translate(-4 0)" />
		<path d="M5 17h10" />
	</svg>
{:else if id === 'entries'}
	<svg class={klass} viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
		<line x1="6" y1="5" x2="17" y2="5" />
		<line x1="6" y1="10" x2="17" y2="10" />
		<line x1="6" y1="15" x2="17" y2="15" />
		<circle cx="3.5" cy="5" r="0.8" fill="currentColor" stroke="none" />
		<circle cx="3.5" cy="10" r="0.8" fill="currentColor" stroke="none" />
		<circle cx="3.5" cy="15" r="0.8" fill="currentColor" stroke="none" />
	</svg>
{/if}
