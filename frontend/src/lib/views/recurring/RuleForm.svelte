<script lang="ts">
	import { cache } from '$lib/api/cache.svelte';
	import type { Account, Anchors } from '$lib/views/entries/loader';
	import { VERBS, verbConfig, buildVerbPostings, type VerbId } from '$lib/views/entries/verbs';
	import { today } from '$lib/views/today';
	import SolidButton from '$lib/components/SolidButton.svelte';
	import { SCHEDULE_PRESETS, type SchedulePresetId } from './schedules';
	import { createRecurring } from './api';

	const INPUT = 'border border-border bg-surface-card px-2 py-1.5';
	const MONO = `${INPUT} font-mono tabular-nums`;
	const ERROR = 'text-xs text-negative';
	const BANNER = 'border border-negative-border bg-negative-tint px-3 py-2 text-xs text-negative-strong';
	const LABEL = 'text-xs uppercase tracking-wide text-ink-muted';

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

	let verb = $state<VerbId>(initialVerb);
	let counterpartyId = $state(initialCounterpartyId);
	let amountText = $state('');
	let dtstart = $state(today());
	let presetId = $state<SchedulePresetId>('monthly-first');
	let description = $state('');
	let note = $state('');
	let submitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let fieldErrors = $state<Record<string, string>>({});

	const chequing = $derived(anchors.cash);
	const config = $derived(verbConfig(verb));
	const counterparties = $derived(
		accounts
			.filter((a) => a.account_type === config.counterpartyType && a.id !== chequing.id)
			.sort((a, b) => a.name.localeCompare(b.name))
	);
	const counterpartyName = $derived(accounts.find((a) => a.id === counterpartyId)?.name ?? '');
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
		if (cents === null) return void (fieldErrors = { amount: 'enter a positive amount' });
		if (!counterpartyId) return void (fieldErrors = { counterparty: 'pick an account' });

		const trimmedNote = note.trim() || undefined;
		const templates = buildVerbPostings(verb, counterpartyId, chequing.id, cents).map((p) => ({
			accountId: p.accountId,
			amount: p.amount,
			...(trimmedNote ? { note: trimmedNote } : {})
		}));

		submitting = true;
		try {
			const result = await createRecurring({
				description: description.trim() || defaultDescription,
				schedule: presetId,
				dtstart,
				templates
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
	<div class="flex flex-wrap gap-1 border border-border bg-surface-sunken p-1">
		{#each VERBS as v (v.id)}
			<button
				type="button"
				onclick={() => (verb = v.id)}
				class="flex-1 px-3 py-1.5 text-xs font-medium uppercase tracking-[0.06em] transition-colors {v.id === verb
					? 'bg-surface-card text-ink shadow-sm'
					: 'text-ink-muted hover:text-ink-body'}"
			>
				{v.label}
			</button>
		{/each}
	</div>

	<div class="grid grid-cols-[1fr_auto] gap-3">
		<label class="flex flex-col gap-1">
			<span class={LABEL}>Schedule</span>
			<select bind:value={presetId} class={INPUT} required>
				{#each SCHEDULE_PRESETS as preset (preset.id)}
					<option value={preset.id}>{preset.label}</option>
				{/each}
			</select>
		</label>
		<label class="flex flex-col gap-1">
			<span class={LABEL}>Starts</span>
			<input type="date" bind:value={dtstart} class={MONO} required />
		</label>
	</div>

	<div class="grid grid-cols-[auto_1fr] gap-3">
		<label class="flex flex-col gap-1">
			<span class={LABEL}>Amount</span>
			<input
				type="text"
				inputmode="decimal"
				placeholder="0.00"
				bind:value={amountText}
				class={MONO}
				required
			/>
			{#if fieldErrors.amount}<span class={ERROR}>{fieldErrors.amount}</span>{/if}
		</label>
		<label class="flex flex-col gap-1">
			<span class={LABEL}>
				{config.label}{config.counterpartyPreposition ? ` ${config.counterpartyPreposition}` : ''}
			</span>
			<select bind:value={counterpartyId} class={INPUT} required>
				<option value="" disabled>select…</option>
				{#each counterparties as account (account.id)}
					<option value={account.id}>{account.name}</option>
				{/each}
			</select>
			{#if fieldErrors.counterparty}<span class={ERROR}>{fieldErrors.counterparty}</span>{/if}
		</label>
	</div>

	<label class="flex flex-col gap-1">
		<span class={LABEL}>Description (optional)</span>
		<input type="text" placeholder={defaultDescription} bind:value={description} class={INPUT} />
	</label>

	<label class="flex flex-col gap-1">
		<span class={LABEL}>Note (optional)</span>
		<input type="text" bind:value={note} class={INPUT} />
	</label>

	{#if fieldErrors.templates}<p class={BANNER}>{fieldErrors.templates}</p>{/if}
	{#if errorMessage}<p class={BANNER}>{errorMessage}</p>{/if}

	<div class="flex gap-2 pt-2">
		<SolidButton type="submit" disabled={submitting}>
			{submitting ? 'Saving…' : 'Save rule'}
		</SolidButton>
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
