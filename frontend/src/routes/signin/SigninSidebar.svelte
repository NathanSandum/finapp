<script lang="ts">
	import { tabs } from '$lib/tabs/registry';
	import { todayLabel } from '$lib/views/today';
	import SidebarWorkspacePill from '$lib/components/SidebarWorkspacePill.svelte';
	import SidebarTabRow from '$lib/components/SidebarTabRow.svelte';
	import SidebarUserPill from '$lib/components/SidebarUserPill.svelte';

	const dateLabel = todayLabel();
</script>

<aside class="flex flex-col bg-surface-inverse p-3 text-ink-inverse-body select-none">
	<SidebarWorkspacePill blank />
	<p class="mb-4 px-2 pt-0.5 font-mono text-[10px] uppercase tracking-[0.1em] text-ink-muted">
		{dateLabel}
	</p>
	<nav class="flex flex-col gap-0.5">
		{#each tabs as tab (tab.id)}
			<SidebarTabRow {tab} blank />
		{/each}
	</nav>
	<div class="mt-auto">
		<SidebarUserPill blank />
	</div>
</aside>
