<script lang="ts">
	import { cache } from '$lib/api/cache.svelte';
	import { deleteAccount, renameAccount } from './api';
	import { load as loadAccounts } from './loader';
	import { ROLE_BADGE, type AccountNode } from './tree';
	import NameEditor from './NameEditor.svelte';

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
		const result = await renameAccount(account.id, trimmed);
		busy = false;
		if (result.status === 'success') {
			cache.invalidateAllExcept('accounts');
			await cache.refresh('accounts', loadAccounts);
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
		if (!confirm(`Delete "${account.name}"?`)) return;
		busy = true;
		error = null;
		const result = await deleteAccount(account.id);
		busy = false;
		if (result.status === 'success') {
			cache.invalidateAllExcept('accounts');
			await cache.refresh('accounts', loadAccounts);
		} else {
			error =
				result.status === 'fail'
					? String(Object.values(result.data)[0] ?? 'delete failed')
					: result.message;
		}
	}
</script>

<div class="group flex items-center gap-2 px-3 py-2 text-sm hover:bg-surface-hover-faint">
	{#if editing}
		<NameEditor
			initial={account.name}
			{busy}
			onCommit={commitRename}
			onCancel={() => (editing = false)}
		/>
	{:else}
		<button
			type="button"
			onclick={() => onSelect(account.id)}
			class="flex-1 truncate text-left text-ink"
		>
			{account.name}
		</button>
		{#if badge}
			<span class="bg-surface-sunken px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-[0.06em] text-ink-muted">
				{badge}
			</span>
		{/if}
		{#if hasChildren}
			<span class="font-mono text-[10px] text-ink-muted">{node.children.length} ›</span>
		{/if}
		<div class="flex items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100">
			{#if canRename}
				<button
					type="button"
					onclick={() => (editing = true)}
					disabled={busy}
					class="border border-border bg-surface-card px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-[0.06em] text-ink-body hover:border-indigo-500 hover:text-indigo-500 disabled:opacity-50"
				>
					rename
				</button>
			{/if}
			{#if canDelete}
				<button
					type="button"
					onclick={commitDelete}
					disabled={busy}
					class="border border-border bg-surface-card px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-[0.06em] text-ink-body hover:border-negative-strong hover:text-negative-strong disabled:opacity-50"
				>
					×
				</button>
			{/if}
		</div>
	{/if}
</div>

{#if error}
	<p class="px-3 pb-1 text-[11px] text-negative">{error}</p>
{/if}
