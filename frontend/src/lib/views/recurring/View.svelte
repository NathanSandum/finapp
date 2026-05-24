<script lang="ts">
	import { slide } from 'svelte/transition';
	import { expoOut } from 'svelte/easing';
	import { accounts, anchors, recurring } from '$lib/data';
	import SolidButton from '$lib/components/SolidButton.svelte';
	import RuleForm from './RuleForm.svelte';
	import RulesList from './RulesList.svelte';
	import { DURATION, panelEnter } from '$lib/utils/motion';

	let creating = $state(false);

	const ruleList = $derived(recurring.value ?? []);
	const accountList = $derived(accounts.value ?? []);
	const anchorData = $derived(anchors.value);

	const accountNames = $derived.by(() => {
		const map: Record<string, string> = {};
		for (const account of accountList) map[account.id] = account.name;
		return map;
	});
</script>

<div class="flex h-full flex-col">
	<section class="glass flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto rounded-2xl p-6">
		<header class="flex items-baseline justify-end">
			{#if !creating}
				<div in:panelEnter={{ offset: 8 }}>
					<SolidButton size="sm" onclick={() => (creating = true)}>+ New rule</SolidButton>
				</div>
			{/if}
		</header>

		{#if creating && anchorData}
			<div
				transition:slide={{ duration: DURATION.base, easing: expoOut }}
				class="glass-faint rounded-xl p-5"
			>
				<RuleForm
					accounts={accountList}
					anchors={anchorData}
					onClose={() => (creating = false)}
				/>
			</div>
		{/if}

		<RulesList rules={ruleList} {accountNames} />
	</section>
</div>
