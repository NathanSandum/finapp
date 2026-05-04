<script lang="ts">
	import type { JSendError } from '$lib/tabs/types';
	import { friendlyError } from '$lib/api/error-messages';

	let { result }: { result: JSendError } = $props();

	const message = $derived(friendlyError(result));
</script>

<div class="border-l-2 border-rose-400 bg-white py-2 pl-4">
	<p class="text-xs uppercase tracking-wide text-neutral-500">Server error</p>
	<p class="text-sm text-neutral-900">{message}</p>
</div>
