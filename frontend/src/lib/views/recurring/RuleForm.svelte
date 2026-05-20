<script lang="ts">
	import { cache } from '$lib/api/cache.svelte';
	import type { Account, Anchors } from '$lib/views/entries/loader';
	import { verbConfig, buildVerbPostings, type VerbId } from '$lib/views/entries/verbs';
	import { today } from '$lib/views/today';
	import SolidButton from '$lib/components/SolidButton.svelte';
	import { type SchedulePresetId } from './schedules';
	import { createRecurring } from './api';
	import { BANNER } from './field-classes';
	import VerbStrip from './VerbStrip.svelte';
	import ScheduleFields from './ScheduleFields.svelte';
	import AmountCounterpartyFields from './AmountCounterpartyFields.svelte';
	import TextFieldRow from './TextFieldRow.svelte';

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
	<VerbStrip bind:verb />
	<ScheduleFields bind:presetId bind:dtstart />
	<AmountCounterpartyFields
		bind:amountText
		bind:counterpartyId
		{counterparties}
		{config}
		amountError={fieldErrors.amount}
		counterpartyError={fieldErrors.counterparty}
	/>
	<TextFieldRow label="Description (optional)" bind:value={description} placeholder={defaultDescription} />
	<TextFieldRow label="Note (optional)" bind:value={note} />

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
