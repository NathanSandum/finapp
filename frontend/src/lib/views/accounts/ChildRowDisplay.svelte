<script lang="ts">
	import type { AccountNode } from './tree';
	import BorderButton from '$lib/components/BorderButton.svelte';

	let {
		node,
		badge,
		canRename,
		canDelete,
		busy,
		onSelect,
		onStartEdit,
		onDelete
	}: {
		node: AccountNode;
		badge: string | undefined;
		canRename: boolean;
		canDelete: boolean;
		busy: boolean;
		onSelect: (id: string) => void;
		onStartEdit: () => void;
		onDelete: () => void;
	} = $props();

	const account = $derived(node.account);
	const hasChildren = $derived(node.children.length > 0);
</script>

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
		<BorderButton size="xs" tone="indigo" onclick={onStartEdit} disabled={busy}>rename</BorderButton>
	{/if}
	{#if canDelete}
		<BorderButton size="xs" tone="negative" onclick={onDelete} disabled={busy}>delete</BorderButton>
	{/if}
</div>
