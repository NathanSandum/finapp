<script lang="ts">
	import { fly } from 'svelte/transition';
	import { accounts } from '$lib/data';
	import { ROLE_BADGE, type AccountNode } from './tree';
	import NameEditor from './NameEditor.svelte';
	import ChildRowDisplay from './ChildRowDisplay.svelte';
	import { DURATION } from '$lib/utils/motion';

	let {
		node,
		onSelect
	}: {
		node: AccountNode;
		onSelect: (id: string) => void;
	} = $props();

	const account = $derived(node.account);
	const isAnchor = $derived(account.role !== null);
	const hasChildren = $derived(node.children.length > 0);
	const canDelete = $derived(!isAnchor && !hasChildren);
	const canRename = $derived(!isAnchor);
	const badge = $derived(account.role ? ROLE_BADGE[account.role] : undefined);

	let editing = $state(false);
	let confirming = $state(false);
	let busy = $state(false);
	let error = $state<string | null>(null);

	async function commitRename(next: string) {
		const trimmed = next.trim();
		if (!trimmed || trimmed === account.name) {
			editing = false;
			return;
		}
		busy = true;
		error = null;
		const result = await accounts.rename(account.id, trimmed);
		busy = false;
		if (result.status === 'success') {
			editing = false;
		} else {
			error =
				result.status === 'fail'
					? String(Object.values(result.data)[0] ?? 'rename failed')
					: result.message;
		}
	}

	async function commitDelete() {
		if (!canDelete) return;
		busy = true;
		error = null;
		const result = await accounts.remove(account.id);
		busy = false;
		if (result.status !== 'success') {
			error =
				result.status === 'fail'
					? String(Object.values(result.data)[0] ?? 'delete failed')
					: result.message;
		}
		confirming = false;
	}
</script>

<div class="child-row group relative flex items-center gap-2 rounded-lg px-3 py-2 text-[13px] transition-all duration-300">
	<span class="child-row__sheen pointer-events-none absolute inset-0 rounded-lg"></span>
	{#if editing}
		<NameEditor
			initial={account.name}
			{busy}
			onCommit={commitRename}
			onCancel={() => (editing = false)}
		/>
	{:else if confirming}
		<span class="flex-1 text-[11px] text-ink-muted">Delete "{account.name}"?</span>
		<button
			type="button"
			onclick={() => (confirming = false)}
			disabled={busy}
			class="border border-border bg-surface-card px-1.5 py-0.5 text-[10px] uppercase tracking-[0.06em] text-ink-body hover:border-ink-faint disabled:opacity-50"
		>
			cancel
		</button>
		<button
			type="button"
			onclick={commitDelete}
			disabled={busy}
			class="border border-negative-border bg-negative-tint px-1.5 py-0.5 text-[10px] uppercase tracking-[0.06em] text-negative-strong hover:border-negative-strong disabled:opacity-50"
		>
			{busy ? '…' : 'confirm'}
		</button>
	{:else}
		<ChildRowDisplay
			{node}
			{badge}
			{canRename}
			{canDelete}
			{busy}
			{onSelect}
			onStartEdit={() => (editing = true)}
			onDelete={() => (confirming = true)}
		/>
	{/if}
</div>

{#if error}
	<p
		in:fly={{ y: -3, duration: DURATION.fast }}
		class="px-3 pb-1 text-[11px] text-negative"
	>
		{error}
	</p>
{/if}

<style>
	.child-row__sheen {
		opacity: 0;
		background: linear-gradient(
			90deg,
			oklch(75% 0.14 268 / 0.1) 0%,
			transparent 80%
		);
		transition: opacity 240ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.child-row:hover .child-row__sheen {
		opacity: 1;
	}
	.child-row > :global(*) {
		position: relative;
	}
</style>
