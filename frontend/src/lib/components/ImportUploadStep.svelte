<script lang="ts">
	let {
		year = $bindable(new Date().getFullYear()),
		fileError,
		onfileupload
	}: {
		year: number;
		fileError: string;
		onfileupload: (event: Event) => void;
	} = $props();
</script>

<div class="mx-auto max-w-lg space-y-4">
	<div>
		<label class="mb-1 block text-sm text-gray-400" for="year-input">Year</label>
		<input
			id="year-input"
			type="number"
			bind:value={year}
			class="w-24 rounded border border-gray-700 bg-gray-900 px-3 py-2 text-sm text-gray-200"
		/>
	</div>
	<div>
		<label class="mb-1 block text-sm text-gray-400" for="csv-input">CSV File</label>
		<input
			id="csv-input"
			type="file"
			accept=".csv"
			onchange={onfileupload}
			class="text-sm text-gray-400 file:mr-3 file:rounded file:border-0 file:bg-gray-800 file:px-3 file:py-2 file:text-sm file:text-gray-300 hover:file:bg-gray-700"
		/>
	</div>
	{#if fileError}
		<p class="text-sm text-red-400">{fileError}</p>
	{/if}
</div>
