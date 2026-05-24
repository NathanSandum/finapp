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

<aside class="accounts-sidebar h-full overflow-y-auto border-r border-[var(--color-glass-divider)] p-2">
	{#each subtrees as { key, tree } (key)}
		{@const rootSel = selectedId === tree.account.id}
		<div class="mb-2 last:mb-0">
			<button
				type="button"
				onclick={() => onSelect(tree.account.id)}
				class="root-button group relative flex w-full items-center justify-between rounded-lg px-3 py-2.5 text-left transition-all duration-300"
				class:is-selected={rootSel}
			>
				<span class="root-button__sheen pointer-events-none absolute inset-0 rounded-lg"></span>
				<h4
					class="relative font-ui text-[10px] font-semibold uppercase tracking-[0.14em] {rootSel
						? 'text-ink'
						: ROOT_ACCENT[key]}"
				>
					{ROOT_LABEL[key]}
				</h4>
				<span class="relative font-mono text-[10px] text-ink-muted">{tree.children.length}</span>
			</button>
			{#if tree.children.length > 0}
				<div class="mt-1 pl-1">
					{#each tree.children as child (child.account.id)}
						<TreeNode node={child} depth={0} {selectedId} {onSelect} />
					{/each}
				</div>
			{/if}
		</div>
	{/each}
</aside>

<style>
	.accounts-sidebar {
		background: oklch(100% 0 0 / 0.25);
		backdrop-filter: blur(8px);
		-webkit-backdrop-filter: blur(8px);
	}
	.root-button__sheen {
		opacity: 0;
		background: linear-gradient(
			135deg,
			oklch(75% 0.14 268 / 0.1) 0%,
			transparent 70%
		);
		transition: opacity 240ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.root-button:hover .root-button__sheen {
		opacity: 1;
	}
	.root-button.is-selected {
		background: oklch(75% 0.14 268 / 0.16);
		box-shadow:
			inset 0 0 0 1px oklch(58% 0.2 268 / 0.35),
			0 4px 14px -6px oklch(58% 0.2 268 / 0.25);
	}
	.root-button.is-selected .root-button__sheen {
		opacity: 1;
	}
</style>
