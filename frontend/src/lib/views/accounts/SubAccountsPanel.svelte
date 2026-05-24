<script lang="ts">
	import { slide } from 'svelte/transition';
	import { expoOut } from 'svelte/easing';
	import { flip } from 'svelte/animate';
	import type { AccountNode } from './tree';
	import ChildRow from './ChildRow.svelte';
	import AddChildInput from './AddChildInput.svelte';
	import BorderButton from '$lib/components/BorderButton.svelte';
	import { DURATION, rowEnter, rowExit } from '$lib/utils/motion';

	let {
		selected,
		adding,
		addBusy,
		addError,
		onSelect,
		onStartAdd,
		onCommitAdd,
		onCancelAdd
	}: {
		selected: AccountNode;
		adding: boolean;
		addBusy: boolean;
		addError: string | null;
		onSelect: (id: string) => void;
		onStartAdd: () => void;
		onCommitAdd: (name: string) => void;
		onCancelAdd: () => void;
	} = $props();
</script>

<div>
	<header class="mb-2 flex items-center justify-between">
		<h4 class="font-ui text-[10px] font-semibold uppercase tracking-[0.12em] text-ink-muted">
			Sub-accounts
		</h4>
		{#if !adding}
			<BorderButton onclick={onStartAdd}>+ new</BorderButton>
		{/if}
	</header>

	{#if selected.children.length > 0}
		<ul class="glass-faint flex flex-col gap-px rounded-xl p-1">
			{#each selected.children as c (c.account.id)}
				<li animate:flip={{ duration: DURATION.base }} in:rowEnter out:rowExit>
					<ChildRow node={c} {onSelect} />
				</li>
			{/each}
		</ul>
	{:else if !adding}
		<p class="glass-faint rounded-xl px-4 py-5 text-center text-[12px] text-ink-muted">
			No sub-accounts.
		</p>
	{/if}

	{#if adding}
		<div
			transition:slide={{ duration: DURATION.base, easing: expoOut }}
			class="glass-faint mt-2 rounded-xl px-3 py-2"
		>
			<AddChildInput busy={addBusy} onCommit={onCommitAdd} onCancel={onCancelAdd} />
		</div>
	{/if}

	{#if addError}
		<p class="mt-1 text-[11px] text-negative">{addError}</p>
	{/if}
</div>
