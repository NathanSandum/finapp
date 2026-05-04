<script lang="ts">
	import type { JSendFail } from '$lib/tabs/types';

	let { result }: { result: JSendFail } = $props();

	const data = $derived(result.data);
	const isSingleMessage = $derived(
		Object.keys(data).length === 1 && typeof data.message === 'string'
	);
	const fields = $derived(isSingleMessage ? [] : Object.entries(data));
</script>

<div class="border-l-2 border-amber-400 bg-white py-2 pl-4">
	{#if isSingleMessage}
		<p class="text-sm text-neutral-900">{data.message}</p>
	{:else}
		<p class="text-xs uppercase tracking-wide text-neutral-500">Check your input</p>
		<ul class="mt-1 space-y-0.5 text-sm text-neutral-900">
			{#each fields as [field, message] (field)}
				<li><span class="font-mono text-xs text-neutral-500">{field}</span> {message}</li>
			{/each}
		</ul>
	{/if}
</div>
