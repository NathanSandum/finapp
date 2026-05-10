<script lang="ts">
	import type { JSendFail } from '$lib/tabs/types';

	let { result }: { result: JSendFail } = $props();

	const data = $derived(result.data);
	const isSingleMessage = $derived(
		Object.keys(data).length === 1 && typeof data.message === 'string'
	);
	const fields = $derived(isSingleMessage ? [] : Object.entries(data));
</script>

<div class="border-l-2 border-financing-border bg-surface-card py-2 pl-4">
	{#if isSingleMessage}
		<p class="text-sm text-ink">{data.message}</p>
	{:else}
		<p class="text-xs uppercase tracking-wide text-ink-muted">Check your input</p>
		<ul class="mt-1 space-y-0.5 text-sm text-ink">
			{#each fields as [field, message] (field)}
				<li><span class="font-mono text-xs text-ink-muted">{field}</span> {message}</li>
			{/each}
		</ul>
	{/if}
</div>
