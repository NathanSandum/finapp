<script lang="ts">
	import type { JSendError } from '$lib/tabs/types';
	import { friendlyError } from '$lib/api/error-messages';

	let { result }: { result: JSendError } = $props();

	const message = $derived(friendlyError(result));
</script>

<div class="border-l-2 border-negative-border bg-surface-card py-2 pl-4">
	<p class="text-xs uppercase tracking-wide text-ink-muted">Server error</p>
	<p class="text-sm text-ink">{message}</p>
</div>
