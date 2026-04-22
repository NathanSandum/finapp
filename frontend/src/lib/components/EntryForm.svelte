<script lang="ts">
	import * as api from '$lib/api.svelte';
	import type { Account, Entry, PostingInput } from '$lib/types';

	let {
		workspaceId,
		accounts,
		entry = null,
		onsave,
		oncancel
	}: {
		workspaceId: string;
		accounts: Account[];
		entry: Entry | null;
		onsave: (entry: Entry) => void;
		oncancel: () => void;
	} = $props();

	type PostingRow = { accountId: string; amount: string; note: string };

	function initialPostings(): PostingRow[] {
		if (entry) {
			return entry.postings.map((posting) => ({
				accountId: posting.account_id,
				amount: (posting.amount / 100).toFixed(2),
				note: posting.note
			}));
		}
		return [
			{ accountId: '', amount: '', note: '' },
			{ accountId: '', amount: '', note: '' }
		];
	}

	function todayString(): string {
		const now = new Date();
		const year = now.getFullYear();
		const month = String(now.getMonth() + 1).padStart(2, '0');
		const day = String(now.getDate()).padStart(2, '0');
		return `${year}-${month}-${day}`;
	}

	let date: string = $state(entry?.date ?? todayString());
	let description: string = $state(entry?.description ?? '');
	let cleared: boolean = $state(entry?.cleared ?? false);
	let postings: PostingRow[] = $state(initialPostings());
	let error: string = $state('');
	let saving: boolean = $state(false);

	let balanceCents = $derived(
		postings.reduce((sum, posting) => {
			const parsed = parseFloat(posting.amount);
			if (isNaN(parsed)) return sum;
			return sum + Math.round(parsed * 100);
		}, 0)
	);

	let isBalanced = $derived(balanceCents === 0);

	function addPosting() {
		postings.push({ accountId: '', amount: '', note: '' });
	}

	function removePosting(index: number) {
		postings.splice(index, 1);
	}

	function dollarsToCents(amount: string): number {
		return Math.round(parseFloat(amount) * 100);
	}

	async function handleSubmit() {
		error = '';

		if (!date) {
			error = 'Date is required';
			return;
		}

		const hasEmptyAccount = postings.some((posting) => !posting.accountId);
		if (hasEmptyAccount) {
			error = 'All postings must have an account selected';
			return;
		}

		const hasEmptyAmount = postings.some((posting) => !posting.amount || isNaN(parseFloat(posting.amount)));
		if (hasEmptyAmount) {
			error = 'All postings must have a valid amount';
			return;
		}

		if (!isBalanced) {
			error = 'Postings must balance to zero';
			return;
		}

		if (postings.length < 2) {
			error = 'At least two postings are required';
			return;
		}

		const postingInputs: PostingInput[] = postings.map((posting) => ({
			account_id: posting.accountId,
			amount: dollarsToCents(posting.amount),
			note: posting.note || undefined
		}));

		saving = true;
		try {
			let result: Entry;
			if (entry) {
				result = await api.updateEntry(workspaceId, entry.id, {
					date,
					description,
					cleared,
					postings: postingInputs
				});
			} else {
				result = await api.createEntry(workspaceId, date, description, postingInputs, cleared);
			}
			onsave(result);
		} catch (catchError: unknown) {
			const errorObject = catchError as { error?: string };
			error = errorObject?.error ?? 'Failed to save entry';
		} finally {
			saving = false;
		}
	}
</script>

<div class="mb-4 rounded border border-gray-700 bg-gray-900 p-4">
	<h3 class="mb-3 text-sm font-semibold text-gray-200">
		{entry ? 'Edit Entry' : 'New Entry'}
	</h3>

	{#if error}
		<div class="mb-3 rounded border border-red-800 bg-red-950 px-3 py-2 text-xs text-red-400">
			{error}
		</div>
	{/if}

	<div class="mb-3 flex flex-wrap gap-3">
		<div class="flex flex-col gap-1">
			<label class="text-[10px] uppercase tracking-wider text-gray-500" for="entry-date">Date</label>
			<input
				id="entry-date"
				type="date"
				bind:value={date}
				class="rounded border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-200"
			/>
		</div>
		<div class="flex flex-1 flex-col gap-1">
			<label class="text-[10px] uppercase tracking-wider text-gray-500" for="entry-description">Description</label>
			<input
				id="entry-description"
				type="text"
				bind:value={description}
				placeholder="Description"
				class="rounded border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-200"
			/>
		</div>
		<div class="flex items-end gap-2">
			<label class="flex items-center gap-1.5 text-xs text-gray-400">
				<input type="checkbox" bind:checked={cleared} class="rounded" />
				Cleared
			</label>
		</div>
	</div>

	<div class="mb-2">
		<div class="mb-1 grid grid-cols-[1fr_100px_1fr_24px] gap-2 text-[10px] uppercase tracking-wider text-gray-500">
			<span>Account</span>
			<span>Amount ($)</span>
			<span>Note</span>
			<span></span>
		</div>
		{#each postings as posting, index}
			<div class="mb-1 grid grid-cols-[1fr_100px_1fr_24px] gap-2">
				<select
					bind:value={posting.accountId}
					class="rounded border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-200"
				>
					<option value="">Select account</option>
					{#each accounts as account}
						<option value={account.id}>{account.name}</option>
					{/each}
				</select>
				<input
					type="text"
					bind:value={posting.amount}
					placeholder="0.00"
					class="rounded border border-gray-700 bg-gray-800 px-2 py-1 text-right text-sm tabular-nums text-gray-200"
				/>
				<input
					type="text"
					bind:value={posting.note}
					placeholder="Note"
					class="rounded border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-200"
				/>
				<button
					class="text-xs text-gray-600 hover:text-red-400"
					onclick={() => removePosting(index)}
					type="button"
				>
					&#10005;
				</button>
			</div>
		{/each}
	</div>

	<div class="mb-3 flex items-center justify-between">
		<button
			type="button"
			class="text-xs text-gray-400 hover:text-gray-200"
			onclick={addPosting}
		>
			+ Add posting
		</button>
		<span class="text-xs tabular-nums {isBalanced ? 'text-emerald-400' : 'text-red-400'}">
			Balance: {(balanceCents / 100).toFixed(2)}
		</span>
	</div>

	<div class="flex gap-2">
		<button
			type="button"
			class="rounded bg-blue-600 px-3 py-1 text-sm text-white hover:bg-blue-500 disabled:opacity-50"
			disabled={saving || !isBalanced}
			onclick={handleSubmit}
		>
			{saving ? 'Saving...' : entry ? 'Update' : 'Create'}
		</button>
		<button
			type="button"
			class="rounded border border-gray-700 px-3 py-1 text-sm text-gray-400 hover:text-gray-200"
			onclick={oncancel}
		>
			Cancel
		</button>
	</div>
</div>
