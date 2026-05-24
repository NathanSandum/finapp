<script lang="ts">
	import { fly } from 'svelte/transition';
	import { expoOut } from 'svelte/easing';
	import { entries } from '$lib/data';
	import type { Account, Anchors } from '$lib/data';
	import { VERBS, verbConfig, buildVerbPostings, type VerbId } from './verbs';
	import { today } from '../today';
	import SolidButton from '$lib/components/SolidButton.svelte';
	import BorderButton from '$lib/components/BorderButton.svelte';
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
		onCreated?: (id: string) => void;
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
			const result = await entries.create({
				date,
				description: description.trim() || defaultDescription,
				postings,
				note: note.trim() || undefined
			});

			if (result.status === 'success') {
				onCreated?.(result.data.id);
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
	<div class="verb-strip relative grid grid-cols-6 gap-1 rounded-xl p-1">
		{#each VERBS as v, i (v.id)}
			{@const active = v.id === verb}
			<button
				type="button"
				onclick={() => (verb = v.id)}
				in:fly={{ y: 4, duration: DURATION.fast, delay: i * 20, easing: expoOut }}
				class="relative z-10 rounded-lg px-2 py-2 text-[11px] font-medium uppercase tracking-[0.1em] transition-colors duration-300 {active
					? 'text-ink'
					: 'text-ink-muted hover:text-ink-body'}"
			>
				{#if active}
					<span
						in:fly={{ duration: DURATION.base, easing: expoOut }}
						class="verb-pill absolute inset-0 -z-10 rounded-lg"
					></span>
				{/if}
				{v.label}
			</button>
		{/each}
	</div>

	<div class="grid grid-cols-[auto_1fr] gap-3">
		<label class="flex flex-col gap-1.5">
			<span class="font-ui text-[10px] uppercase tracking-[0.12em] text-ink-muted">Date</span>
			<input
				type="date"
				bind:value={date}
				class="glass-input font-mono text-[13px]"
				required
			/>
		</label>
		<label class="flex flex-col gap-1.5">
			<span class="font-ui text-[10px] uppercase tracking-[0.12em] text-ink-muted">Amount</span>
			<input
				type="text"
				inputmode="decimal"
				placeholder="0.00"
				bind:value={amountText}
				class="glass-input font-mono text-[13px] tabular-nums"
				required
			/>
			{#if fieldErrors.amount}
				<span in:fly={{ y: -3, duration: DURATION.fast }} class="text-[11px] text-negative">
					{fieldErrors.amount}
				</span>
			{/if}
		</label>
	</div>

	<label class="flex flex-col gap-1.5">
		<span class="font-ui text-[10px] uppercase tracking-[0.12em] text-ink-muted">
			{config.label}{config.counterpartyPreposition ? ` ${config.counterpartyPreposition}` : ''}
		</span>
		<select
			bind:value={counterpartyId}
			class="glass-input text-[13px]"
			required
		>
			<option value="" disabled>select…</option>
			{#each counterparties as account (account.id)}
				<option value={account.id}>{account.name}</option>
			{/each}
		</select>
		{#if fieldErrors.counterparty}
			<span in:fly={{ y: -3, duration: DURATION.fast }} class="text-[11px] text-negative">
				{fieldErrors.counterparty}
			</span>
		{/if}
	</label>

	<label class="flex flex-col gap-1.5">
		<span class="font-ui text-[10px] uppercase tracking-[0.12em] text-ink-muted">
			Description
		</span>
		<input
			type="text"
			placeholder={defaultDescription}
			bind:value={description}
			class="glass-input text-[13px]"
		/>
		{#if fieldErrors.description}
			<span in:fly={{ y: -3, duration: DURATION.fast }} class="text-[11px] text-negative">
				{fieldErrors.description}
			</span>
		{/if}
	</label>

	<label class="flex flex-col gap-1.5">
		<span class="font-ui text-[10px] uppercase tracking-[0.12em] text-ink-muted">Note</span>
		<input type="text" bind:value={note} class="glass-input text-[13px]" />
	</label>

	{#if errorMessage}
		<p
			in:fly={{ y: -3, duration: DURATION.base }}
			class="rounded-lg border border-negative-border bg-negative-tint px-3 py-2 text-[11px] text-negative-strong"
		>
			{errorMessage}
		</p>
	{/if}

	<div class="flex gap-2 pt-1">
		<SolidButton type="submit" disabled={submitting}>
			{submitting ? 'Saving…' : 'Save entry'}
		</SolidButton>
		<BorderButton onclick={onClose} disabled={submitting}>cancel</BorderButton>
	</div>
</form>

<style>
	.verb-strip {
		background: oklch(100% 0 0 / 0.4);
		backdrop-filter: blur(12px);
		-webkit-backdrop-filter: blur(12px);
		box-shadow: inset 0 0 0 1px oklch(100% 0 0 / 0.5);
	}
	.verb-pill {
		background: linear-gradient(
			135deg,
			oklch(75% 0.14 268 / 0.22) 0%,
			oklch(100% 0 0 / 0.85) 100%
		);
		box-shadow:
			inset 0 0 0 1px oklch(58% 0.2 268 / 0.4),
			0 4px 14px -6px oklch(58% 0.2 268 / 0.3);
	}
</style>
