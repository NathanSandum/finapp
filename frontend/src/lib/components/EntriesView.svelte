<script lang="ts">
	import * as api from '$lib/api.svelte';
	import { formatAmount } from '$lib/format';
	import type { Quarter } from '$lib/period';
	import { quarterMonths, quarterPeriodLabel } from '$lib/period';
	import type { Account, Entry } from '$lib/types';

	let { workspaceId, quarter }: { workspaceId: string; quarter: Quarter } = $props();

	let entries: Entry[] = $state([]);
	let accounts: Account[] = $state([]);
	let loading = $state(true);

	let accountNames = $derived(
		Object.fromEntries(accounts.map((account) => [account.id, account.name]))
	);

	$effect(() => {
		loading = true;
		entries = [];
		const period = quarterPeriodLabel(quarter);
		Promise.all([
			api.getPeriodEntries(workspaceId, quarter.year, period),
			api.listAccounts(workspaceId)
		])
			.then(([entryData, accountData]) => {
				entries = entryData.entries;
				accounts = accountData.accounts;
			})
			.catch((error) => console.error('EntriesView fetch failed:', error))
			.finally(() => (loading = false));
	});

	async function handleDelete(entry: Entry) {
		try {
			await api.deleteEntry(workspaceId, entry.id);
			entries = entries.filter((existing) => existing.id !== entry.id);
		} catch {
			// backend will return error if not authorized
		}
	}
</script>

{#if loading}
	<div class="flex min-h-64 items-center justify-center text-gray-600">Loading...</div>
{:else if entries.length === 0}
	<div class="flex min-h-64 items-center justify-center text-gray-600">No entries</div>
{:else}
	<table class="w-full border-collapse font-mono text-xs">
		<thead>
			<tr>
				<th
					class="border border-gray-700 bg-gray-900 px-2 py-1.5 text-left text-[10px] font-semibold uppercase tracking-wider text-gray-400"
				>
					Date
				</th>
				<th
					class="border border-gray-700 bg-gray-900 px-2 py-1.5 text-left text-[10px] font-semibold uppercase tracking-wider text-gray-400"
				>
					Description
				</th>
				<th
					class="border border-gray-700 bg-gray-900 px-2 py-1.5 text-left text-[10px] font-semibold uppercase tracking-wider text-gray-400"
				>
					Account
				</th>
				<th
					class="border border-gray-700 bg-gray-900 px-2 py-1.5 text-right text-[10px] font-semibold uppercase tracking-wider text-gray-400"
				>
					Debit
				</th>
				<th
					class="border border-gray-700 bg-gray-900 px-2 py-1.5 text-right text-[10px] font-semibold uppercase tracking-wider text-gray-400"
				>
					Credit
				</th>
				<th
					class="border border-gray-700 bg-gray-900 px-2 py-1.5 text-center text-[10px] font-semibold uppercase tracking-wider text-gray-400"
				>
					Status
				</th>
				<th
					class="w-8 border border-gray-700 bg-gray-900 px-2 py-1.5 text-[10px] font-semibold uppercase tracking-wider text-gray-400"
				></th>
			</tr>
		</thead>
		<tbody>
			{#each entries as entry}
				{#each entry.postings as posting, index}
					<tr>
						{#if index === 0}
							<td
								class="border border-gray-700 px-2 py-1 tabular-nums text-gray-400"
								rowspan={entry.postings.length}
							>
								{entry.date}
							</td>
							<td
								class="border border-gray-700 px-2 py-1 text-gray-300"
								rowspan={entry.postings.length}
							>
								{entry.description || '\u2014'}
							</td>
						{/if}
						<td class="border border-gray-700 px-2 py-1 text-gray-300">
							{accountNames[posting.account_id] || posting.account_id}
						</td>
						<td class="border border-gray-700 px-2 py-1 text-right tabular-nums text-emerald-400">
							{#if posting.amount > 0}
								{formatAmount(posting.amount)}
							{/if}
						</td>
						<td class="border border-gray-700 px-2 py-1 text-right tabular-nums text-red-400">
							{#if posting.amount < 0}
								{formatAmount(Math.abs(posting.amount))}
							{/if}
						</td>
						{#if index === 0}
							<td
								class="border border-gray-700 px-2 py-1 text-center text-[10px]"
								rowspan={entry.postings.length}
							>
								{#if entry.cleared}
									<span class="text-emerald-500">&#10003;</span>
								{:else}
									<span class="text-yellow-500">&#9679;</span>
								{/if}
							</td>
							<td
								class="border border-gray-700 px-2 py-1 text-center"
								rowspan={entry.postings.length}
							>
								<button
									class="text-[10px] text-gray-600 hover:text-red-400"
									onclick={() => handleDelete(entry)}
								>
									&#10005;
								</button>
							</td>
						{/if}
					</tr>
				{/each}
			{/each}
		</tbody>
	</table>
{/if}
