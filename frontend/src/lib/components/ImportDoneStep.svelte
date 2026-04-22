<script lang="ts">
	let {
		entriesCreated,
		importErrors,
		onreset
	}: {
		entriesCreated: number;
		importErrors: string[];
		onreset: () => void;
	} = $props();
</script>

<div class="mx-auto max-w-lg space-y-4 py-12">
	<div class="rounded-lg border border-emerald-900 bg-emerald-950/50 px-4 py-3 text-center">
		<span class="text-lg font-semibold text-emerald-400">{entriesCreated}</span>
		<span class="text-sm text-emerald-400"> entries created</span>
	</div>

	{#if importErrors.length > 0}
		<div class="rounded-lg border border-red-900 bg-red-950/50 px-4 py-3">
			<h3 class="mb-2 text-sm font-medium text-red-400">
				{importErrors.length} errors
			</h3>
			<div class="max-h-48 space-y-1 overflow-y-auto text-xs text-red-400/80">
				{#each importErrors as error}
					<div>{error}</div>
				{/each}
			</div>
		</div>
	{/if}

	<div class="text-center">
		<button
			class="rounded border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:bg-gray-800"
			onclick={onreset}
		>
			Import another
		</button>
	</div>
</div>
