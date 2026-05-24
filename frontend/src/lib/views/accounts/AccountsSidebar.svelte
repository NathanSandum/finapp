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

<aside class="h-full overflow-y-auto border-r border-border-faint bg-surface">
	{#each subtrees as { key, tree } (key)}
		{@const rootSel = selectedId === tree.account.id}
		<div class="border-b border-border-faint last:border-b-0">
			<button
				type="button"
				onclick={() => onSelect(tree.account.id)}
				class="group relative flex w-full items-center justify-between px-3 py-2.5 text-left transition-colors duration-200 hover:bg-surface-hover-faint"
			>
				<h4
					class="font-ui text-[10px] font-semibold uppercase tracking-[0.12em] {rootSel
						? 'text-ink'
						: ROOT_ACCENT[key]}"
				>
					{ROOT_LABEL[key]}
				</h4>
				<span class="font-mono text-[10px] text-ink-muted">{tree.children.length}</span>
				{#if rootSel}
					<span class="pointer-events-none absolute inset-y-0 left-0 w-[2px] bg-brand"></span>
				{/if}
			</button>
			{#if tree.children.length > 0}
				<div class="pb-2 pl-1 pr-1">
					{#each tree.children as child (child.account.id)}
						<TreeNode node={child} depth={0} {selectedId} {onSelect} />
					{/each}
				</div>
			{/if}
		</div>
	{/each}
</aside>
