<script lang="ts">
	import type { JSendFail } from '$lib/tabs/types';

	let { result }: { result: JSendFail } = $props();

	const data = $derived(result.data);
	const isCoded = $derived(
		typeof data === 'object' &&
			data !== null &&
			'code' in data &&
			'message' in data &&
			typeof (data as { code: unknown }).code === 'string' &&
			typeof (data as { message: unknown }).message === 'string'
	);
	const coded = $derived(isCoded ? (data as { code: string; message: string }) : null);
	const fields = $derived(
		isCoded ? [] : Object.entries(data as Record<string, string>)
	);
</script>

<div class="border-l-2 border-amber-400 bg-white py-2 pl-4">
	{#if coded}
		<p class="text-sm text-neutral-900">{coded.message}</p>
	{:else}
		<p class="text-xs uppercase tracking-wide text-neutral-500">Check your input</p>
		<ul class="mt-1 space-y-0.5 text-sm text-neutral-900">
			{#each fields as [field, message] (field)}
				<li><span class="font-mono text-xs text-neutral-500">{field}</span> {message}</li>
			{/each}
		</ul>
	{/if}
</div>
