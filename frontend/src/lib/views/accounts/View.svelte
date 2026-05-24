<script lang="ts">
	import { onDestroy } from 'svelte';
	import { accounts, anchors } from '$lib/data';
	import { buildSubtree, pathTo, ROOT_ORDER } from './tree';
	import AccountsSidebar from './AccountsSidebar.svelte';
	import AccountDetail from './AccountDetail.svelte';
	import { accountsSelection } from './selection.svelte';
	import { panelEnter } from '$lib/utils/motion';

	let selectedId = $state<string | null>(null);

	const accountList = $derived(accounts.value ?? []);
	const anchorData = $derived(anchors.value);

	const subtrees = $derived(
		anchorData
			? ROOT_ORDER.map((key) => ({
					key,
					tree: buildSubtree(anchorData[key].id, accountList)
				}))
			: []
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

<div class="flex h-full flex-col gap-3">
	<header class="flex items-baseline justify-between px-1">
		<div class="flex items-baseline gap-3">
			<h2 class="font-display text-xl font-semibold tracking-tight text-ink">Accounts</h2>
			<span class="font-ui text-[11px] uppercase tracking-[0.1em] text-ink-muted">
				{accountList.length} total
			</span>
		</div>
	</header>

	<div
		class="grid min-h-0 flex-1 grid-cols-[340px_1fr] overflow-hidden border border-border bg-surface-card shadow-[0_1px_0_rgba(0,0,0,0.02)]"
	>
		<AccountsSidebar {subtrees} {selectedId} onSelect={select} />

		<section class="relative overflow-y-auto">
			{#if selected && selectedTrail && anchorData}
				{#key selected.account.id}
					<div in:panelEnter class="h-full">
						<AccountDetail
							{selected}
							trail={selectedTrail}
							anchors={anchorData}
							onSelect={select}
							onAfterDelete={(nextId) => (selectedId = nextId)}
						/>
					</div>
				{/key}
			{:else}
				<div class="flex h-full items-center justify-center text-[12px] text-ink-muted">
					Select an account on the left.
				</div>
			{/if}
		</section>
	</div>
</div>
