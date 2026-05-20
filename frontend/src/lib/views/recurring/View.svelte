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

<section class="flex h-full flex-col gap-6 overflow-y-auto">
	<header class="flex items-end justify-between border-t-2 border-ink pt-4">
		<div>
			<p class="text-[10px] uppercase tracking-[0.22em] text-ink-faint">Schedule</p>
			<h1 class="font-display text-2xl font-medium tracking-tight text-ink">Recurring rules</h1>
		</div>
		{#if !creating}
			<BorderButton onclick={() => (creating = true)}>New rule</BorderButton>
		{/if}
	</header>

	{#if creating}
		<div class="border border-border bg-surface-card p-6">
			<RuleForm
				accounts={data.accounts}
				anchors={data.anchors}
				onClose={() => (creating = false)}
			/>
		</div>
	{/if}

	<RulesList rules={data.rules} {accountNames} />
</section>
