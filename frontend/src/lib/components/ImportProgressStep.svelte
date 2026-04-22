<script lang="ts">
	let {
		importProgress,
		importTotal,
		entriesCreated
	}: {
		importProgress: number;
		importTotal: number;
		entriesCreated: number;
	} = $props();
</script>

<div class="mx-auto max-w-lg space-y-4 py-12">
	<div class="text-center text-sm text-gray-400">
		Importing... {importProgress} / {importTotal}
	</div>
	<div class="h-2 overflow-hidden rounded-full bg-gray-800">
		<div
			class="h-full rounded-full bg-blue-500 transition-all"
			style="width: {importTotal > 0 ? (importProgress / importTotal) * 100 : 0}%"
		></div>
	</div>
	<div class="text-center text-sm text-gray-500">{entriesCreated} entries created</div>
</div>
