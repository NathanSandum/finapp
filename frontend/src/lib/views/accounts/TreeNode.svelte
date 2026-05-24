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
		class="relative flex-1 truncate px-2 py-1 text-left text-[12px] transition-colors duration-200 {isSelected
			? 'bg-surface-inverse text-ink-inverse'
			: 'hover:bg-surface-hover-faint'}"
	>
		{node.account.name}
		{#if isSelected}
			<span class="pointer-events-none absolute inset-y-0 left-0 w-[2px] bg-brand"></span>
		{/if}
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
