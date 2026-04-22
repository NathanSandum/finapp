<script lang="ts">
	import type { Account } from '$lib/types';
	import type { MappingType, ColumnMapping, DateRow } from '$lib/import';
	import { findAccountByName } from '$lib/import';
	import AccountSelect from './AccountSelect.svelte';
	import FundingSelect from './FundingSelect.svelte';

	const MAPPING_LABELS: Record<MappingType, string> = {
		skip: 'Skip',
		opening_balance: 'Opening Balance',
		income: 'Income',
		expense: 'Expense',
		deduction: 'Deduction',
		transfer: 'Transfer',
		credit_expense: 'Credit Expense'
	};

	let {
		mappings = $bindable(),
		accounts,
		dateRows,
		startRow,
		mappingErrors,
		onpreview,
		onback
	}: {
		mappings: ColumnMapping[];
		accounts: Account[];
		dateRows: DateRow[];
		startRow: string[];
		mappingErrors: string[];
		onpreview: () => void;
		onback: () => void;
	} = $props();

	let leafAccounts = $derived(accounts.filter((a) => a.parent_id !== null));

	function needsFunding(type: MappingType): boolean {
		return ['income', 'expense', 'deduction', 'transfer', 'credit_expense'].includes(type);
	}

	function updateMappingType(index: number, newType: MappingType) {
		const mapping = mappings[index];
		mapping.type = newType;

		const chequingId = findAccountByName(accounts, 'Chequing')?.id ?? '';
		const creditCardId = findAccountByName(accounts, 'Credit Card')?.id ?? '';

		if (newType === 'credit_expense') {
			mapping.fundingAccountId = creditCardId;
		} else if (['income', 'expense', 'deduction', 'transfer'].includes(newType)) {
			mapping.fundingAccountId = chequingId;
		} else {
			mapping.fundingAccountId = '';
		}

		if (newType === 'transfer') {
			if (!mapping.accountId) {
				mapping.newAccountName = '';
			}
		}
	}
</script>

<div class="mb-4 text-sm text-gray-500">
	{dateRows.length} days parsed{startRow.length > 0 ? ', Start row found' : ''} &middot; {accounts.length} accounts loaded
</div>

<div class="space-y-2">
	{#each mappings as mapping, index}
		<div class="rounded-lg border border-gray-800 bg-gray-900/50 px-4 py-3">
			<div class="flex items-center gap-3">
				<span class="w-48 shrink-0 text-sm font-medium text-gray-200">{mapping.column}</span>
				<select
					class="rounded border border-gray-700 bg-gray-900 px-2 py-1.5 text-sm text-gray-300"
					value={mapping.type}
					onchange={(e) =>
						updateMappingType(
							index,
							(e.target as HTMLSelectElement).value as MappingType
						)}
				>
					{#each Object.entries(MAPPING_LABELS) as [value, label]}
						<option {value}>{label}</option>
					{/each}
				</select>
			</div>

			{#if mapping.type === 'opening_balance'}
				<div class="mt-2 flex items-center gap-2 text-sm text-gray-400">
					<span>Starting balance of</span>
					<AccountSelect
						accounts={leafAccounts}
						bind:accountId={mapping.accountId}
						bind:newAccountName={mapping.newAccountName}
					/>
				</div>
			{:else if mapping.type === 'income'}
				<div class="mt-2 flex items-center gap-2 text-sm text-gray-400">
					<span>Revenue from</span>
					<AccountSelect
						accounts={leafAccounts}
						bind:accountId={mapping.accountId}
						bind:newAccountName={mapping.newAccountName}
					/>
					<span>deposited into</span>
					<FundingSelect
						accounts={leafAccounts}
						bind:fundingAccountId={mapping.fundingAccountId}
					/>
				</div>
			{:else if mapping.type === 'expense' || mapping.type === 'deduction'}
				<div class="mt-2 flex items-center gap-2 text-sm text-gray-400">
					<span>Expense categorized as</span>
					<AccountSelect
						accounts={leafAccounts}
						bind:accountId={mapping.accountId}
						bind:newAccountName={mapping.newAccountName}
					/>
					<span>paid from</span>
					<FundingSelect
						accounts={leafAccounts}
						bind:fundingAccountId={mapping.fundingAccountId}
					/>
				</div>
			{:else if mapping.type === 'credit_expense'}
				<div class="mt-2 flex items-center gap-2 text-sm text-gray-400">
					<span>Expense categorized as</span>
					<AccountSelect
						accounts={leafAccounts}
						bind:accountId={mapping.accountId}
						bind:newAccountName={mapping.newAccountName}
					/>
					<span>charged to</span>
					<FundingSelect
						accounts={leafAccounts}
						bind:fundingAccountId={mapping.fundingAccountId}
					/>
				</div>
			{:else if mapping.type === 'transfer'}
				<div class="mt-2 flex items-center gap-2 text-sm text-gray-400">
					<span>Move money from</span>
					<FundingSelect
						accounts={leafAccounts}
						bind:fundingAccountId={mapping.fundingAccountId}
					/>
					<span>to</span>
					<AccountSelect
						accounts={leafAccounts}
						bind:accountId={mapping.accountId}
						bind:newAccountName={mapping.newAccountName}
					/>
				</div>
			{/if}
		</div>
	{/each}
</div>

{#if mappingErrors.length > 0}
	<div class="mt-4 rounded-lg border border-red-900 bg-red-950/50 px-4 py-3">
		{#each mappingErrors as error}
			<div class="text-sm text-red-400">{error}</div>
		{/each}
	</div>
{/if}

<div class="mt-4 flex items-center gap-4">
	<button
		class="rounded bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 disabled:opacity-40 disabled:cursor-not-allowed"
		onclick={onpreview}
		disabled={mappingErrors.length > 0}
	>
		Preview
	</button>
	<button
		class="rounded border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:bg-gray-800"
		onclick={onback}
	>
		Back
	</button>
</div>
