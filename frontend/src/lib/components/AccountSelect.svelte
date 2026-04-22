<script lang="ts">
	import type { Account } from '$lib/types';

	let {
		accounts,
		accountId = $bindable(''),
		newAccountName = $bindable('')
	}: {
		accounts: Account[];
		accountId: string;
		newAccountName: string;
	} = $props();

	function handleChange(event: Event) {
		const value = (event.target as HTMLSelectElement).value;
		if (value === '__new__') {
			accountId = '';
			if (!newAccountName) {
				newAccountName = '';
			}
		} else {
			accountId = value;
			newAccountName = '';
		}
	}
</script>

<div class="inline-flex flex-col gap-1">
	<select
		class="rounded border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-200"
		value={accountId || '__new__'}
		onchange={handleChange}
	>
		<option value="__new__">+ Create new</option>
		{#each accounts as account}
			<option value={account.id}>{account.name}</option>
		{/each}
	</select>
	{#if !accountId}
		<input
			type="text"
			bind:value={newAccountName}
			placeholder="Account name"
			class="rounded border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-200"
		/>
	{/if}
</div>
