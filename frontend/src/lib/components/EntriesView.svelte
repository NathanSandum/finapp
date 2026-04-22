<script lang="ts">
	import * as api from '$lib/api.svelte';
	import { formatAmount } from '$lib/format';
	import type { Account, Entry } from '$lib/types';
	import { onMount } from 'svelte';

	let { workspaceId }: { workspaceId: string } = $props();

	let entries: Entry[] = $state([]);
	let accounts: Account[] = $state([]);
	let loading = $state(true);

	// Build account name lookup
	let accountNames = $derived(
		Object.fromEntries(accounts.map((a) => [a.id, a.name]))
	);

	onMount(async () => {
		try {
			const [entryData, accountData] = await Promise.all([
				api.listEntries(workspaceId),
				api.listAccounts(workspaceId)
			]);
			entries = entryData.entries;
			accounts = accountData.accounts;
		} catch {
			// no data yet
		} finally {
			loading = false;
		}
	});

	async function handleDelete(entry: Entry) {
		try {
			await api.deleteEntry(workspaceId, entry.id);
			entries = entries.filter((e) => e.id !== entry.id);
		} catch {
			// backend will return error if not authorized
		}
	}
</script>

<div>
	<header class="mb-6 border-b border-gray-800 pb-4">
		<h2 class="text-lg font-semibold text-gray-100">Entries</h2>
		<span class="text-sm text-gray-600">{entries.length} entries</span>
	</header>

	{#if loading}
		<div class="flex min-h-64 items-center justify-center text-gray-600">Loading...</div>
	{:else if entries.length === 0}
		<div class="flex min-h-64 items-center justify-center text-gray-600">
			No entries yet
		</div>
	{:else}
		<div class="space-y-2">
			{#each entries as entry}
				<div class="rounded-lg border border-gray-800 bg-gray-900/50 px-4 py-3">
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-3">
							<span class="text-sm tabular-nums text-gray-500">{entry.date}</span>
							<span class="text-sm font-medium text-gray-300"
								>{entry.description || '\u2014'}</span
							>
							{#if entry.cleared}
								<span
									class="rounded-full bg-emerald-950 px-2 py-0.5 text-[11px] font-medium text-emerald-400"
									>Cleared</span
								>
							{:else}
								<span
									class="rounded-full bg-yellow-950 px-2 py-0.5 text-[11px] font-medium text-yellow-400"
									>Pending</span
								>
							{/if}
						</div>
						<button
							class="text-xs text-gray-600 hover:text-red-400"
							onclick={() => handleDelete(entry)}
						>
							Delete
						</button>
					</div>
					<div class="mt-2 space-y-1">
						{#each entry.postings as posting}
							<div class="flex items-center justify-between text-sm">
								<span class="text-gray-400"
									>{accountNames[posting.account_id] || posting.account_id}</span
								>
								<span
									class="font-medium tabular-nums {posting.amount >= 0 ? 'text-emerald-400' : 'text-red-400'}"
								>
									{formatAmount(posting.amount)}
								</span>
							</div>
						{/each}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
