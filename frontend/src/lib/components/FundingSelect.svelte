<script lang="ts">
	import type { Account } from '$lib/types';

	let {
		accounts,
		fundingAccountId = $bindable('')
	}: {
		accounts: Account[];
		fundingAccountId: string;
	} = $props();
</script>

<select
	class="rounded border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-200"
	bind:value={fundingAccountId}
>
	{#each accounts as account}
		<option value={account.id}>{account.name}</option>
	{/each}
</select>
