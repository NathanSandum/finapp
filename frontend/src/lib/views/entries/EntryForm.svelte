<script lang="ts">
	import { cache } from '$lib/api/cache.svelte';
	import type { Account, Anchors } from './loader';
	import { createEntry } from './api';
	import { VERBS, verbConfig, buildVerbPostings, type VerbId } from './verbs';
	import { today } from '../today';
	import SolidButton from '$lib/components/SolidButton.svelte';

	let {
		accounts,
		anchors,
		initialVerb = 'spent',
		initialCounterpartyId = '',
		onClose,
		onCreated
	}: {
		accounts: Account[];
		anchors: Anchors;
		initialVerb?: VerbId;
		initialCounterpartyId?: string;
		onClose: () => void;
		onCreated?: () => void;
	} = $props();

	const chequing = $derived(anchors.cash);

	let verb = $state<VerbId>(initialVerb);
	let counterpartyId = $state(initialCounterpartyId);
	let amountText = $state('');
	let date = $state(today());
	let description = $state('');
	let note = $state('');
	let submitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let fieldErrors = $state<Record<string, string>>({});

	const config = $derived(verbConfig(verb));

	const counterparties = $derived(
		accounts
			.filter((a) => a.account_type === config.counterpartyType && a.id !== chequing.id)
			.sort((a, b) => a.name.localeCompare(b.name))
	);

	const counterpartyName = $derived(
		accounts.find((a) => a.id === counterpartyId)?.name ?? ''
	);
	const defaultDescription = $derived(
		counterpartyName
			? config.counterpartyPreposition
				? `${config.label} ${config.counterpartyPreposition} ${counterpartyName}`
				: `${config.label} ${counterpartyName}`
			: config.label
	);

	$effect(() => {
		if (counterpartyId && !counterparties.some((a) => a.id === counterpartyId)) {
			counterpartyId = '';
		}
	});

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
		if (!counterpartyId) {
			fieldErrors = { counterparty: 'pick a category' };
			return;
		}

		const postings = buildVerbPostings(verb, counterpartyId, chequing.id, cents);

		submitting = true;
		try {
			const result = await createEntry({
				date,
				description: description.trim() || defaultDescription,
				postings,
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
	<div class="flex flex-wrap divide-x divide-border border-y border-ink">
		{#each VERBS as v (v.id)}
			{@const active = v.id === verb}
			<button
				type="button"
				onclick={() => (verb = v.id)}
				class="flex-1 px-3 py-2 text-[11px] font-medium uppercase tracking-[0.16em] transition-colors {active
					? 'bg-ink text-ink-inverse'
					: 'bg-surface-card text-ink-muted hover:text-ink'}"
			>
				{v.label}
			</button>
		{/each}
	</div>

	<div class="grid grid-cols-[auto_1fr] gap-3">
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
		<span class="text-xs uppercase tracking-wide text-ink-muted">
			{config.label}{config.counterpartyPreposition ? ` ${config.counterpartyPreposition}` : ''}
		</span>
		<select
			bind:value={counterpartyId}
			class="border border-border bg-surface-card px-2 py-1.5"
			required
		>
			<option value="" disabled>select…</option>
			{#each counterparties as account (account.id)}
				<option value={account.id}>{account.name}</option>
			{/each}
		</select>
		{#if fieldErrors.counterparty}
			<span class="text-xs text-negative">{fieldErrors.counterparty}</span>
		{/if}
	</label>

	<label class="flex flex-col gap-1">
		<span class="text-xs uppercase tracking-wide text-ink-muted">Description (optional)</span>
		<input
			type="text"
			placeholder={defaultDescription}
			bind:value={description}
			class="border border-border bg-surface-card px-2 py-1.5"
		/>
		{#if fieldErrors.description}
			<span class="text-xs text-negative">{fieldErrors.description}</span>
		{/if}
	</label>

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
		<SolidButton type="submit" disabled={submitting}>
			{submitting ? 'Saving…' : 'Save entry'}
		</SolidButton>
		<button
			type="button"
			onclick={onClose}
			disabled={submitting}
			class="border border-border px-4 py-2 text-xs uppercase tracking-[0.14em] text-ink-body hover:border-ink hover:text-ink disabled:opacity-50"
		>
			Cancel
		</button>
	</div>
</form>
