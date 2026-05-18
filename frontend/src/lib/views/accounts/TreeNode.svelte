<script lang="ts">
	import type { AccountNode } from './tree';
	import Self from './TreeNode.svelte';

	let {
		node,
		depth,
		selectedId,
		onSelect
	}: {
		node: AccountNode;
		depth: number;
		selectedId: string | null;
		onSelect: (id: string) => void;
	} = $props();

	let expanded = $state(depth < 1);
	const hasChildren = $derived(node.children.length > 0);
	const isSelected = $derived(selectedId === node.account.id);
</script>

<div class="flex items-center gap-1" style="padding-left: {depth * 12}px;">
	{#if hasChildren}
		<button
			type="button"
			onclick={() => (expanded = !expanded)}
			class="flex h-6 w-6 items-center justify-center text-[14px] leading-none text-ink-muted hover:text-ink-body"
		>
			{expanded ? '▾' : '▸'}
		</button>
	{:else}
		<span class="inline-block h-6 w-6"></span>
	{/if}
	<button
		type="button"
		onclick={() => onSelect(node.account.id)}
		class="flex-1 truncate px-2 py-0.5 text-left text-[13px] transition-colors {isSelected
			? 'bg-surface-inverse text-ink-inverse'
			: 'hover:bg-surface-hover-faint'}"
	>
		{node.account.name}
	</button>
</div>

{#if expanded && hasChildren}
	{#each node.children as child (child.account.id)}
		<Self node={child} depth={depth + 1} {selectedId} {onSelect} />
	{/each}
{/if}
