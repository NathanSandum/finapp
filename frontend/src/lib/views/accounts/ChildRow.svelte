<script lang="ts">
	import { cache } from '$lib/api/cache.svelte';
	import { invalidateAll as invalidateData } from '$lib/data';
	import { deleteAccount, renameAccount } from './api';
	import { load as loadAccounts } from './loader';
	import { ROLE_BADGE, type AccountNode } from './tree';
	import NameEditor from './NameEditor.svelte';
	import ChildRowDisplay from './ChildRowDisplay.svelte';

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
			invalidateData();
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
			invalidateData();
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
		<ChildRowDisplay
			{node}
			{badge}
			{canRename}
			{canDelete}
			{busy}
			{onSelect}
			onStartEdit={() => (editing = true)}
			onDelete={commitDelete}
		/>
	{/if}
</div>

{#if error}
	<p class="px-3 pb-1 text-[11px] text-negative">{error}</p>
{/if}
