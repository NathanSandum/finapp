<script lang="ts">
	import { fly } from 'svelte/transition';
	import { expoOut } from 'svelte/easing';
	import { recurring } from '$lib/data';
	import type { Account, Anchors } from '$lib/data';
	import { VERBS, verbConfig, buildVerbPostings, type VerbId } from '$lib/views/entries/verbs';
	import { today } from '$lib/views/today';
	import SolidButton from '$lib/components/SolidButton.svelte';
	import BorderButton from '$lib/components/BorderButton.svelte';
	import { type SchedulePresetId } from './schedules';
	import { BANNER } from './field-classes';
	import ScheduleFields from './ScheduleFields.svelte';
	import AmountCounterpartyFields from './AmountCounterpartyFields.svelte';
	import TextFieldRow from './TextFieldRow.svelte';
	import { DURATION } from '$lib/utils/motion';

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
			const result = await recurring.create({
				description: description.trim() || defaultDescription,
				schedule: presetId,
				dtstart,
				templates
			});

			if (result.status === 'success') {
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

<form onsubmit={submit} class="flex flex-col gap-5 text-sm">
	<div class="relative grid grid-cols-6 gap-1 border border-border bg-surface-sunken p-1">
		{#each VERBS as v, i (v.id)}
			{@const active = v.id === verb}
			<button
				type="button"
				onclick={() => (verb = v.id)}
				in:fly={{ y: 4, duration: DURATION.fast, delay: i * 20, easing: expoOut }}
				class="relative z-10 px-2 py-1.5 text-[11px] font-medium uppercase tracking-[0.08em] transition-colors duration-200 {active
					? 'text-ink'
					: 'text-ink-muted hover:text-ink-body'}"
			>
				{#if active}
					<span
						in:fly={{ duration: DURATION.base, easing: expoOut }}
						class="absolute inset-0 -z-10 bg-surface-card shadow-[0_1px_2px_rgba(0,0,0,0.05)]"
					></span>
				{/if}
				{v.label}
			</button>
		{/each}
	</div>

	<ScheduleFields bind:presetId bind:dtstart />
	<AmountCounterpartyFields
		bind:amountText
		bind:counterpartyId
		{counterparties}
		{config}
		amountError={fieldErrors.amount}
		counterpartyError={fieldErrors.counterparty}
	/>
	<TextFieldRow
		label="Description"
		bind:value={description}
		placeholder={defaultDescription}
	/>
	<TextFieldRow label="Note" bind:value={note} />

	{#if fieldErrors.templates}
		<p in:fly={{ y: -3, duration: DURATION.base }} class={BANNER}>{fieldErrors.templates}</p>
	{/if}
	{#if errorMessage}
		<p in:fly={{ y: -3, duration: DURATION.base }} class={BANNER}>{errorMessage}</p>
	{/if}

	<div class="flex gap-2 pt-1">
		<SolidButton type="submit" disabled={submitting}>
			{submitting ? 'Saving…' : 'Save rule'}
		</SolidButton>
		<BorderButton onclick={onClose} disabled={submitting}>cancel</BorderButton>
	</div>
</form>
