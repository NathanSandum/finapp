<script lang="ts">
	import type { JSendFail } from '$lib/tabs/types';

	let { result }: { result: JSendFail } = $props();

	const entries = $derived(Object.entries(result.data));
</script>

<div class="rounded-lg border border-amber-300 bg-amber-50 p-6">
	<h3 class="mb-2 text-sm font-semibold text-amber-900">Request rejected</h3>
	<ul class="space-y-1 text-sm text-amber-900">
		{#each entries as [field, message] (field)}
			<li><span class="font-mono text-xs">{field}</span>: {message}</li>
		{/each}
	</ul>
</div>
