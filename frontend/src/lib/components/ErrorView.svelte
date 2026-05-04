<script lang="ts">
	import type { JSendError } from '$lib/tabs/types';

	let { result }: { result: JSendError } = $props();
</script>

<div class="rounded-lg border border-rose-300 bg-rose-50 p-6">
	<h3 class="mb-2 text-sm font-semibold text-rose-900">Server error</h3>
	<p class="text-sm text-rose-900">{result.message}</p>
	{#if result.code}
		<p class="mt-1 font-mono text-xs text-rose-700">code: {result.code}</p>
	{/if}
</div>
