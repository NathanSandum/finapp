<script lang="ts">
	import type { Tab } from '$lib/tabs/types';

	let { tab, active }: { tab: Tab; active: boolean } = $props();
</script>

<section class="h-full" style:display={active ? 'block' : 'none'}>
	<tab.view />
</section>
