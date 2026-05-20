<script lang="ts">
	import type { Account } from '$lib/views/entries/loader';
	import type { VerbConfig } from '$lib/views/entries/verbs';
	import { INPUT, MONO, LABEL, ERROR } from './field-classes';

	let {
		amountText = $bindable(),
		counterpartyId = $bindable(),
		counterparties,
		config,
		amountError,
		counterpartyError
	}: {
		amountText: string;
		counterpartyId: string;
		counterparties: Account[];
		config: VerbConfig;
		amountError?: string;
		counterpartyError?: string;
	} = $props();
</script>

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
		{#if amountError}<span class={ERROR}>{amountError}</span>{/if}
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
		{#if counterpartyError}<span class={ERROR}>{counterpartyError}</span>{/if}
	</label>
</div>
