<script lang="ts">
	import TwoPaneLayout from './TwoPaneLayout.svelte';
	import type { EntriesData } from './loader';

	let { data }: { data: EntriesData } = $props();
</script>

<div class="flex flex-col gap-4">
	<header>
		<h2 class="text-2xl font-semibold">Entries</h2>
	</header>
	<TwoPaneLayout entries={data.entries} accountNames={data.accountNames} />
</div>
