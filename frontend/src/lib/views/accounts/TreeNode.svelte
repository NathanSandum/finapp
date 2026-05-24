<script lang="ts">
	import { slide } from 'svelte/transition';
	import { expoOut } from 'svelte/easing';
	import { flip } from 'svelte/animate';
	import type { AccountNode } from './tree';
	import Self from './TreeNode.svelte';
	import { DURATION, rowEnter, rowExit } from '$lib/utils/motion';

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

<div class="flex items-center gap-1" style="padding-left: {depth * 14}px;">
	{#if hasChildren}
		<button
			type="button"
			onclick={() => (expanded = !expanded)}
			class="flex h-6 w-6 items-center justify-center text-[12px] leading-none text-ink-muted transition-transform duration-200 hover:text-ink-body"
			style="transform: rotate({expanded ? 90 : 0}deg);"
			aria-label={expanded ? 'collapse' : 'expand'}
		>
			▸
		</button>
	{:else}
		<span class="inline-block h-6 w-6"></span>
	{/if}
	<button
		type="button"
		onclick={() => onSelect(node.account.id)}
		class="tree-leaf relative flex-1 truncate rounded-md px-2 py-1 text-left text-[12px] transition-all duration-300"
		class:is-selected={isSelected}
	>
		<span class="tree-leaf__sheen pointer-events-none absolute inset-0 rounded-md"></span>
		<span class="relative text-ink">{node.account.name}</span>
	</button>
</div>

{#if expanded && hasChildren}
	<div transition:slide={{ duration: DURATION.base, easing: expoOut }}>
		{#each node.children as child (child.account.id)}
			<div animate:flip={{ duration: DURATION.base }} in:rowEnter out:rowExit>
				<Self node={child} depth={depth + 1} {selectedId} {onSelect} />
			</div>
		{/each}
	</div>
{/if}

<style>
	.tree-leaf__sheen {
		opacity: 0;
		background: linear-gradient(
			90deg,
			oklch(75% 0.14 268 / 0.1) 0%,
			transparent 80%
		);
		transition: opacity 240ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.tree-leaf:hover .tree-leaf__sheen {
		opacity: 1;
	}
	.tree-leaf.is-selected {
		background: oklch(75% 0.14 268 / 0.18);
		box-shadow: inset 0 0 0 1px oklch(58% 0.2 268 / 0.4);
	}
	.tree-leaf.is-selected .tree-leaf__sheen {
		opacity: 1;
	}
</style>
