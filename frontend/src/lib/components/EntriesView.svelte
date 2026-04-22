<script lang="ts">
	import * as api from '$lib/api.svelte';
	import EntryForm from '$lib/components/EntryForm.svelte';
	import { formatAmount } from '$lib/format';
	import type { Quarter } from '$lib/period';
	import { quarterMonths, quarterPeriodLabel } from '$lib/period';
	import type { Account, Entry } from '$lib/types';

	let { workspaceId, quarter }: { workspaceId: string; quarter: Quarter } = $props();

	let entries: Entry[] = $state([]);
	let accounts: Account[] = $state([]);
	let loading = $state(true);
	let formEntry: Entry | null | 'new' = $state(null);

	let accountNames = $derived(
		Object.fromEntries(accounts.map((account) => [account.id, account.name]))
	);

	const MONTH_NAMES = [
		'January', 'February', 'March', 'April', 'May', 'June',
		'July', 'August', 'September', 'October', 'November', 'December'
	];

	type MonthGroup = {
		label: string;
		entries: Entry[];
	};

	let monthGroups = $derived.by(() => {
		const months = quarterMonths(quarter);
		const groups: MonthGroup[] = months.map((month, index) => {
			const firstMonthOfQuarter = (quarter.quarter - 1) * 3;
			const monthNumber = firstMonthOfQuarter + index + 1;
			const monthPrefix = `${quarter.year}-${String(monthNumber).padStart(2, '0')}`;
			return {
				label: MONTH_NAMES[monthNumber - 1],
				entries: entries.filter((entry) => entry.date.startsWith(monthPrefix))
			};
		});
		return groups;
	});

	let collapsedMonths: Set<string> = $state(new Set());

	function toggleMonth(label: string) {
		if (collapsedMonths.has(label)) {
			collapsedMonths.delete(label);
		} else {
			collapsedMonths.add(label);
		}
		collapsedMonths = new Set(collapsedMonths);
	}

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

	function handleFormSave(savedEntry: Entry) {
		if (formEntry === 'new') {
			entries = [savedEntry, ...entries];
		} else {
			entries = entries.map((existing) =>
				existing.id === savedEntry.id ? savedEntry : existing
			);
		}
		formEntry = null;
	}

	function handleFormCancel() {
		formEntry = null;
	}
</script>

{#if loading}
	<div class="flex min-h-64 items-center justify-center text-gray-600">Loading...</div>
{:else}
	<div class="mb-2 flex justify-end">
		<button
			class="rounded bg-blue-600 px-3 py-1 text-sm text-white hover:bg-blue-500"
			onclick={() => (formEntry = 'new')}
		>
			New Entry
		</button>
	</div>

	{#if formEntry !== null}
		<EntryForm
			{workspaceId}
			{accounts}
			entry={formEntry === 'new' ? null : formEntry}
			onsave={handleFormSave}
			oncancel={handleFormCancel}
		/>
	{/if}

	{#if entries.length === 0}
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
			{#each monthGroups as group}
				{#if group.entries.length > 0}
					<!-- Month header -->
					<tr
						class="cursor-pointer select-none"
						onclick={() => toggleMonth(group.label)}
					>
						<td
							colspan="7"
							class="border border-gray-700 bg-gray-800/50 px-2 py-1 font-bold text-gray-200"
						>
							<span class="inline-block w-3 text-[10px] text-gray-500"
								>{collapsedMonths.has(group.label) ? '▸' : '▾'}</span
							>
							{group.label}
							<span class="ml-2 text-[10px] font-normal text-gray-500"
								>{group.entries.length} entries</span
							>
						</td>
					</tr>

					{#if !collapsedMonths.has(group.label)}
						{#each group.entries as entry}
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
									<td
										class="border border-gray-700 px-2 py-1 text-right tabular-nums text-emerald-400"
									>
										{#if posting.amount > 0}
											{formatAmount(posting.amount)}
										{/if}
									</td>
									<td
										class="border border-gray-700 px-2 py-1 text-right tabular-nums text-red-400"
									>
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
												class="mr-1 text-[10px] text-gray-600 hover:text-blue-400"
												onclick={() => (formEntry = entry)}
											>
												&#9998;
											</button>
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
					{/if}
				{/if}
			{/each}
		</tbody>
	</table>
	{/if}
{/if}
