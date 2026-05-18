<script lang="ts">
	import { ROOT_ACCENT, ROOT_LABEL, type AccountNode, type RootKey } from './tree';
	import TreeNode from './TreeNode.svelte';

	let {
		subtrees,
		selectedId,
		onSelect
	}: {
		subtrees: { key: RootKey; tree: AccountNode }[];
		selectedId: string | null;
		onSelect: (id: string) => void;
	} = $props();
</script>

<aside class="h-full overflow-y-auto border-r border-border-faint">
	{#each subtrees as { key, tree } (key)}
		{@const rootSel = selectedId === tree.account.id}
		<div class="border-b border-border-faint last:border-b-0">
			<button
				type="button"
				onclick={() => onSelect(tree.account.id)}
				class="flex w-full items-center justify-between px-3 py-2 text-left transition-colors {rootSel
					? 'bg-surface-inverse'
					: 'hover:bg-surface-hover-faint'}"
			>
				<h4
					class="font-display text-[11px] font-semibold uppercase tracking-[0.08em] {rootSel
						? 'text-ink-inverse'
						: ROOT_ACCENT[key]}"
				>
					{ROOT_LABEL[key]}
				</h4>
				<span class="font-mono text-[10px] {rootSel ? 'text-ink-inverse-muted' : 'text-ink-muted'}">
					{tree.children.length}
				</span>
			</button>
			{#if tree.children.length > 0}
				<div class="px-1 pb-2">
					{#each tree.children as child (child.account.id)}
						<TreeNode node={child} depth={0} {selectedId} onSelect={onSelect} />
					{/each}
				</div>
			{/if}
		</div>
	{/each}
</aside>
