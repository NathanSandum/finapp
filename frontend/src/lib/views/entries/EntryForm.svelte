<script lang="ts">
	import { cache } from '$lib/api/cache.svelte';
	import type { Account } from './loader';
	import { createEntry } from './api';

	let {
		accounts,
		onClose,
		onCreated
	}: {
		accounts: Account[];
		onClose: () => void;
		onCreated?: () => void;
	} = $props();

	function todayISO(): string {
		const d = new Date();
		const y = d.getFullYear();
		const m = String(d.getMonth() + 1).padStart(2, '0');
		const day = String(d.getDate()).padStart(2, '0');
		return `${y}-${m}-${day}`;
	}

	let date = $state(todayISO());
	let description = $state('');
	let debitAccountId = $state('');
	let creditAccountId = $state('');
	let amountText = $state('');
	let note = $state('');
	let submitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let fieldErrors = $state<Record<string, string>>({});

	const sortedAccounts = $derived([...accounts].sort((a, b) => a.name.localeCompare(b.name)));

	function parseAmount(text: string): number | null {
		const cleaned = text.replace(/[^0-9.\-]/g, '');
		if (!cleaned) return null;
		const value = Number(cleaned);
		if (!Number.isFinite(value) || value <= 0) return null;
		return Math.round(value * 100);
	}

	async function submit(event: Event) {
		event.preventDefault();
		errorMessage = null;
		fieldErrors = {};

		const cents = parseAmount(amountText);
		if (cents === null) {
			fieldErrors = { amount: 'enter a positive amount' };
			return;
		}
		if (!debitAccountId || !creditAccountId) {
			fieldErrors = { account: 'pick both accounts' };
			return;
		}
		if (debitAccountId === creditAccountId) {
			fieldErrors = { account: 'debit and credit must differ' };
			return;
		}

		submitting = true;
		try {
			const result = await createEntry({
				date,
				description: description.trim(),
				debitAccountId,
				creditAccountId,
				amountCents: cents,
				note: note.trim() || undefined
			});

			if (result.status === 'success') {
				cache.invalidateAll();
				onCreated?.();
				onClose();
			} else if (result.status === 'fail') {
				fieldErrors = result.data ?? {};
			} else {
				errorMessage = result.message;
			}
		} finally {
			submitting = false;
		}
	}
</script>

<form onsubmit={submit} class="flex flex-col gap-4 text-sm">
	<div class="grid grid-cols-2 gap-3">
		<label class="flex flex-col gap-1">
			<span class="text-xs uppercase tracking-wide text-ink-muted">Date</span>
			<input
				type="date"
				bind:value={date}
				class="border border-border bg-surface-card px-2 py-1.5 font-mono"
				required
			/>
		</label>
		<label class="flex flex-col gap-1">
			<span class="text-xs uppercase tracking-wide text-ink-muted">Amount</span>
			<input
				type="text"
				inputmode="decimal"
				placeholder="0.00"
				bind:value={amountText}
				class="border border-border bg-surface-card px-2 py-1.5 font-mono tabular-nums"
				required
			/>
			{#if fieldErrors.amount}
				<span class="text-xs text-negative">{fieldErrors.amount}</span>
			{/if}
		</label>
	</div>

	<label class="flex flex-col gap-1">
		<span class="text-xs uppercase tracking-wide text-ink-muted">Description</span>
		<input
			type="text"
			bind:value={description}
			class="border border-border bg-surface-card px-2 py-1.5"
			required
		/>
		{#if fieldErrors.description}
			<span class="text-xs text-negative">{fieldErrors.description}</span>
		{/if}
	</label>

	<div class="grid grid-cols-2 gap-3">
		<label class="flex flex-col gap-1">
			<span class="text-xs font-semibold uppercase tracking-[0.08em] text-positive">Debit</span>
			<select
				bind:value={debitAccountId}
				class="border border-border bg-surface-card px-2 py-1.5"
				required
			>
				<option value="" disabled>select account</option>
				{#each sortedAccounts as account (account.id)}
					<option value={account.id}>{account.name}</option>
				{/each}
			</select>
		</label>
		<label class="flex flex-col gap-1">
			<span class="text-xs font-semibold uppercase tracking-[0.08em] text-negative-strong">Credit</span>
			<select
				bind:value={creditAccountId}
				class="border border-border bg-surface-card px-2 py-1.5"
				required
			>
				<option value="" disabled>select account</option>
				{#each sortedAccounts as account (account.id)}
					<option value={account.id}>{account.name}</option>
				{/each}
			</select>
		</label>
	</div>
	{#if fieldErrors.account}
		<span class="-mt-2 text-xs text-negative">{fieldErrors.account}</span>
	{/if}

	<label class="flex flex-col gap-1">
		<span class="text-xs uppercase tracking-wide text-ink-muted">Note (optional)</span>
		<input
			type="text"
			bind:value={note}
			class="border border-border bg-surface-card px-2 py-1.5"
		/>
	</label>

	{#if errorMessage}
		<p class="border border-negative-border bg-negative-tint px-3 py-2 text-xs text-negative-strong">
			{errorMessage}
		</p>
	{/if}

	<div class="flex gap-2 pt-2">
		<button
			type="submit"
			disabled={submitting}
			class="bg-surface-inverse px-4 py-2 text-sm font-medium text-ink-inverse transition-colors hover:bg-surface-inverse-hover disabled:opacity-50"
		>
			{submitting ? 'Saving…' : 'Save entry'}
		</button>
		<button
			type="button"
			onclick={onClose}
			disabled={submitting}
			class="border border-border bg-surface-card px-4 py-2 text-sm text-ink-body hover:bg-surface-hover-faint disabled:opacity-50"
		>
			Cancel
		</button>
	</div>
</form>
