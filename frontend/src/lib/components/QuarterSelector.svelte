<script lang="ts">
	import type { Quarter } from '$lib/period';
	import { previousQuarter, nextQuarter, quarterLabel } from '$lib/period';

	let { quarter = $bindable() }: { quarter: Quarter } = $props();
</script>

<div class="flex items-center gap-3">
	<button
		class="rounded px-2 py-1 text-xs text-gray-400 hover:bg-gray-800 hover:text-gray-200"
		onclick={() => (quarter = previousQuarter(quarter))}
	>
		&larr;
	</button>
	<span class="text-xs font-medium text-gray-200 tabular-nums">{quarterLabel(quarter)}</span>
	<button
		class="rounded px-2 py-1 text-xs text-gray-400 hover:bg-gray-800 hover:text-gray-200"
		onclick={() => (quarter = nextQuarter(quarter))}
	>
		&rarr;
	</button>
</div>
