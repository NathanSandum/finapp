<script lang="ts">
	import TwoPaneLayout from './TwoPaneLayout.svelte';
	import { fixtureEntries } from './fixture';
</script>

<div class="flex flex-col gap-4">
	<header>
		<h2 class="text-2xl font-semibold">Entries</h2>
	</header>
	<TwoPaneLayout entries={fixtureEntries} />
</div>
