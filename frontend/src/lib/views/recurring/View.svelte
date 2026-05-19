<script lang="ts">
	import BorderButton from '$lib/components/BorderButton.svelte';
	import RuleForm from './RuleForm.svelte';
	import RulesList from './RulesList.svelte';
	import type { RecurringData } from './loader';

	let { data }: { data: RecurringData } = $props();

	let creating = $state(false);

	const accountNames = $derived.by(() => {
		const map: Record<string, string> = {};
		for (const account of data.accounts) map[account.id] = account.name;
		return map;
	});
</script>

<section class="h-full overflow-y-auto p-6">
	<header class="mb-6 flex items-baseline justify-between">
		<h3 class="text-lg font-semibold text-ink">Recurring rules</h3>
		{#if !creating}
			<BorderButton onclick={() => (creating = true)}>New rule</BorderButton>
		{/if}
	</header>

	{#if creating}
		<div class="mb-6 border border-border bg-surface-card p-4">
			<RuleForm
				accounts={data.accounts}
				anchors={data.anchors}
				onClose={() => (creating = false)}
			/>
		</div>
	{/if}

	<RulesList rules={data.rules} {accountNames} />
</section>
