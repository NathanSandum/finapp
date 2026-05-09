<script lang="ts">
	import AppShell from '$lib/components/AppShell.svelte';
	import TabPanel from '$lib/components/TabPanel.svelte';
	import TabIcon from '$lib/components/TabIcon.svelte';
	import { tabs } from '$lib/tabs/registry';

	let current = $state(tabs[0]);
</script>

<AppShell>
	{#snippet sidebar()}
		<button
			type="button"
			class="mb-4 flex items-center justify-between rounded px-2 py-1.5 text-left transition-colors hover:bg-neutral-800"
		>
			<span class="flex items-center gap-2">
				<span class="flex h-6 w-6 items-center justify-center rounded bg-emerald-500 text-xs font-bold text-neutral-900">
					F
				</span>
				<span class="text-sm font-medium text-white">FinApp</span>
			</span>
			<svg class="h-4 w-4 text-neutral-500" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
				<path fill-rule="evenodd" d="M5 8l5 5 5-5H5z" clip-rule="evenodd" />
			</svg>
		</button>
		<nav class="flex flex-col gap-0.5">
			{#each tabs as tab (tab.id)}
				<button
					type="button"
					onclick={() => (current = tab)}
					class="relative flex items-center gap-2.5 rounded px-3 py-1.5 text-left text-sm transition-colors {current.id === tab.id
						? 'text-white'
						: 'text-neutral-400 hover:bg-neutral-800/60 hover:text-neutral-100'}"
				>
					{#if current.id === tab.id}
						<span class="absolute inset-y-1 -left-1 w-0.5 rounded-r bg-emerald-400"></span>
					{/if}
					<TabIcon id={tab.id} class="h-4 w-4 shrink-0" />
					{tab.label}
				</button>
			{/each}
		</nav>
		<div class="mt-auto flex items-center gap-2 rounded p-2 hover:bg-neutral-800">
			<div class="flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-br from-indigo-400 to-purple-500 text-xs font-semibold text-white">
				NS
			</div>
			<div class="flex flex-1 flex-col leading-tight">
				<span class="text-sm text-neutral-100">Nathan Sandum</span>
				<span class="text-[11px] text-neutral-500">Personal</span>
			</div>
			<svg class="h-4 w-4 text-neutral-500" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
				<path d="M10 3a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM10 8.5a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM11.5 15.5a1.5 1.5 0 10-3 0 1.5 1.5 0 003 0z" />
			</svg>
		</div>
	{/snippet}

	{#snippet main()}
		{#each tabs as tab (tab.id)}
			<TabPanel {tab} active={current.id === tab.id} />
		{/each}
	{/snippet}
</AppShell>
