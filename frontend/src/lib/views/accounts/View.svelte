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

<div class="grid h-full grid-cols-[320px_1fr] border border-border bg-surface-card">
	<AccountsSidebar {subtrees} {selectedId} onSelect={select} />

	{#if selected && selectedTrail}
		<AccountDetail
			{selected}
			trail={selectedTrail}
			entries={data.entries}
			onSelect={select}
			onAfterDelete={(nextId) => (selectedId = nextId)}
		/>
	{:else}
		<section class="h-full overflow-y-auto p-6">
			<p class="text-sm text-ink-muted">Select an account on the left.</p>
		</section>
	{/if}
</div>
