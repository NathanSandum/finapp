<script lang="ts">
	import { buildSubtree, pathTo, ROOT_ORDER } from './tree';
	import type { AccountsData } from './loader';
	import AccountsSidebar from './AccountsSidebar.svelte';
	import AccountDetail from './AccountDetail.svelte';
	import { accountsSelection } from './selection.svelte';
	import { onDestroy } from 'svelte';

	let { data }: { data: AccountsData } = $props();

	let selectedId = $state<string | null>(null);

	const subtrees = $derived(
		ROOT_ORDER.map((key) => ({
			key,
			tree: buildSubtree(data.anchors[key].id, data.accounts)
		}))
	);

	const selectedTrail = $derived(selectedId ? pathTo(subtrees, selectedId) : null);
	const selected = $derived(
		selectedTrail ? selectedTrail.nodes[selectedTrail.nodes.length - 1] : null
	);

	$effect(() => {
		accountsSelection.set(selected ? selected.account : null);
	});

	onDestroy(() => accountsSelection.set(null));

	function select(id: string) {
		selectedId = id;
	}
</script>

<div class="flex h-full flex-col gap-6">
	<header class="border-t-2 border-ink pt-4">
		<p class="text-[10px] uppercase tracking-[0.22em] text-ink-faint">Chart</p>
		<h1 class="font-display text-2xl font-medium tracking-tight text-ink">Accounts</h1>
	</header>

	<div class="grid min-h-0 flex-1 grid-cols-[320px_1fr] overflow-hidden border border-border bg-surface-card">
		<AccountsSidebar {subtrees} {selectedId} onSelect={select} />

		{#if selected && selectedTrail}
			<AccountDetail
				{selected}
				trail={selectedTrail}
				entries={data.entries}
				accounts={data.accounts}
				anchors={data.anchors}
				recurringRules={data.recurringRules}
				onSelect={select}
				onAfterDelete={(nextId) => (selectedId = nextId)}
			/>
		{:else}
			<section class="h-full overflow-y-auto p-8">
				<p class="text-sm text-ink-muted">Select an account on the left.</p>
			</section>
		{/if}
	</div>
</div>
