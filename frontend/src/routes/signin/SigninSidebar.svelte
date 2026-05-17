<script lang="ts">
	import { tabs } from '$lib/tabs/registry';
	import SidebarWorkspacePill from '$lib/components/SidebarWorkspacePill.svelte';
	import SidebarTabRow from '$lib/components/SidebarTabRow.svelte';
	import SidebarUserPill from '$lib/components/SidebarUserPill.svelte';
	import SidebarDate from '$lib/components/SidebarDate.svelte';
</script>

<aside class="flex flex-col bg-surface-inverse p-3 text-ink-inverse-body select-none">
	<SidebarWorkspacePill blank />
	<SidebarDate />
	<nav class="flex flex-col gap-0.5">
		{#each tabs as tab (tab.id)}
			<SidebarTabRow {tab} blank />
		{/each}
	</nav>
	<div class="mt-auto">
		<SidebarUserPill blank />
	</div>
</aside>
