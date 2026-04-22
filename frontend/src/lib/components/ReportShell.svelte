<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		loading,
		hasData,
		children
	}: {
		loading: boolean;
		hasData: boolean;
		children: Snippet;
	} = $props();
</script>

{#if loading}
	<div class="flex min-h-64 items-center justify-center text-gray-600">Loading...</div>
{:else if !hasData}
	<div class="flex min-h-64 items-center justify-center text-gray-600">No data available</div>
{:else}
	{@render children()}
{/if}
