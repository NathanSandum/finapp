<script lang="ts">
	import type { Account } from '$lib/types';
	import type { ColumnMapping, DateRow } from '$lib/import';
	import { parseDollars, formatAmount } from '$lib/import';

	let {
		mappings,
		accounts,
		startRow,
		dateRows,
		previewStats,
		onimport,
		onback
	}: {
		mappings: ColumnMapping[];
		accounts: Account[];
		startRow: string[];
		dateRows: DateRow[];
		previewStats: { openingCount: number; activityCount: number; newAccountCount: number };
		onimport: () => void;
		onback: () => void;
	} = $props();

	let accountsById = $derived(Object.fromEntries(accounts.map((a) => [a.id, a])));

	let sampleEntries = $derived.by(() => {
		const entries: { date: string; column: string; amount: number }[] = [];
		for (const row of dateRows) {
			for (let i = 0; i < mappings.length; i++) {
				const mapping = mappings[i];
				if (mapping.type === 'skip' || mapping.type === 'opening_balance') continue;
				const cents = parseDollars(row.cells[i + 1] ?? '');
				if (cents === 0) continue;
				entries.push({ date: row.date, column: mapping.column, amount: cents });
				if (entries.length >= 20) return entries;
			}
		}
		return entries;
	});
</script>

<div class="mb-6 grid grid-cols-3 gap-4">
	<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
		<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
			>Opening Balances</span
		>
		<span class="text-xl font-semibold tabular-nums text-gray-200"
			>{previewStats.openingCount} postings</span
		>
	</div>
	<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
		<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
			>Activity Entries</span
		>
		<span class="text-xl font-semibold tabular-nums text-gray-200"
			>{previewStats.activityCount}</span
		>
	</div>
	<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
		<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
			>New Accounts</span
		>
		<span class="text-xl font-semibold tabular-nums text-gray-200"
			>{previewStats.newAccountCount}</span
		>
	</div>
</div>

{#if startRow.length > 0}
	<div class="mb-4">
		<h3 class="mb-2 text-[11px] font-semibold uppercase tracking-wide text-gray-600">
			Opening Balances
		</h3>
		<div class="space-y-1">
			{#each mappings as mapping, index}
				{#if mapping.type === 'opening_balance'}
					{@const cents = parseDollars(startRow[index + 1] ?? '')}
					{#if cents > 0}
						<div class="flex items-center justify-between rounded border border-gray-800 px-3 py-1.5 text-sm">
							<span class="text-gray-300"
								>{mapping.accountId
									? (accountsById[mapping.accountId]?.name ?? mapping.column)
									: mapping.newAccountName}</span
							>
							<span class="font-medium tabular-nums text-emerald-400"
								>{formatAmount(cents)}</span
							>
						</div>
					{/if}
				{/if}
			{/each}
		</div>
	</div>
{/if}

<div class="mb-4">
	<h3 class="mb-2 text-[11px] font-semibold uppercase tracking-wide text-gray-600">
		Sample Activity (first 20)
	</h3>
	<div class="space-y-1">
		{#each sampleEntries as entry}
			<div class="flex items-center justify-between rounded border border-gray-800 px-3 py-1.5 text-sm">
				<div class="flex items-center gap-3">
					<span class="tabular-nums text-gray-500">{entry.date}</span>
					<span class="text-gray-300">{entry.column}</span>
				</div>
				<span class="font-medium tabular-nums text-gray-200"
					>{formatAmount(entry.amount)}</span
				>
			</div>
		{/each}
	</div>
</div>

<div class="mt-4 flex items-center gap-4">
	<button
		class="rounded bg-emerald-600 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-500"
		onclick={onimport}
	>
		Import {previewStats.activityCount + (previewStats.openingCount > 0 ? 1 : 0)} entries
	</button>
	<button
		class="rounded border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:bg-gray-800"
		onclick={onback}
	>
		Back
	</button>
</div>
