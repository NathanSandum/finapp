<script lang="ts">
	import AccountRow from './AccountRow.svelte';
	import { buildSubtree, countDescendants, ROOT_ACCENT, ROOT_LABEL, ROOT_ORDER } from './tree';
	import type { AccountsData } from './loader';

	let { data }: { data: AccountsData } = $props();

	const subtrees = $derived(
		ROOT_ORDER.map((key) => ({
			key,
			tree: buildSubtree(data.anchors[key].id, data.accounts)
		}))
	);
</script>

<div class="flex h-full flex-col gap-4">
	<p class="text-xs uppercase tracking-[0.08em] text-ink-muted">Click name to rename</p>
	<div class="grid min-h-0 flex-1 grid-cols-1 gap-3 overflow-y-auto md:grid-cols-2 xl:grid-cols-3">
		{#each subtrees as { key, tree } (key)}
			<section class="flex flex-col border border-border bg-surface-card">
				<header class="flex items-baseline justify-between border-b border-border-faint px-4 py-2">
					<h3 class="font-display text-sm font-semibold uppercase tracking-[0.08em] {ROOT_ACCENT[key]}">
						{ROOT_LABEL[key]}
					</h3>
					<span class="font-mono text-[11px] text-ink-muted">
						{countDescendants(tree)} accounts
					</span>
				</header>
				<div class="flex-1 px-3 py-2">
					<AccountRow node={tree} />
				</div>
			</section>
		{/each}
	</div>
</div>
