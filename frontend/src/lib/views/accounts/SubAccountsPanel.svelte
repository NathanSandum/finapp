<script lang="ts">
	import type { AccountNode } from './tree';
	import ChildRow from './ChildRow.svelte';
	import AddChildInput from './AddChildInput.svelte';
	import BorderButton from '$lib/components/BorderButton.svelte';

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
		<h4 class="text-[11px] font-semibold uppercase tracking-[0.08em] text-ink-muted">Sub-accounts</h4>
		{#if !adding}
			<BorderButton onclick={onStartAdd}>
				<span class="text-sm leading-none">+</span>
				<span>new</span>
			</BorderButton>
		{/if}
	</header>
	{#if selected.children.length > 0}
		<ul class="divide-y divide-border-faint border border-border-faint">
			{#each selected.children as c (c.account.id)}
				<li>
					<ChildRow node={c} {onSelect} />
				</li>
			{/each}
		</ul>
	{:else if !adding}
		<p class="text-[12px] text-ink-muted">No sub-accounts.</p>
	{/if}
	{#if adding}
		<div class="mt-2 border border-border-faint px-3 py-2">
			<AddChildInput
				busy={addBusy}
				onCommit={onCommitAdd}
				onCancel={onCancelAdd}
			/>
		</div>
	{/if}
	{#if addError}
		<p class="mt-1 text-[11px] text-negative">{addError}</p>
	{/if}
</div>
