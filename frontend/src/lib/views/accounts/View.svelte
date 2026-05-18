<script lang="ts">
	import {
		buildSubtree,
		ROLE_BADGE,
		ROOT_ACCENT,
		ROOT_LABEL,
		ROOT_ORDER,
		type AccountNode,
		type RootKey
	} from './tree';
	import { load as loadAccounts, type AccountsData } from './loader';
	import TreeNode from './TreeNode.svelte';
	import ChildRow from './ChildRow.svelte';
	import AddChildInput from './AddChildInput.svelte';
	import { cache } from '$lib/api/cache.svelte';
	import { createAccount, deleteAccount } from './api';
	import { fmt } from '$lib/format';
	import Num from '$lib/components/Num.svelte';
	import { accountsSelection } from './selection.svelte';
	import { onDestroy } from 'svelte';
	import { periodState } from '$lib/views/period-state.svelte';
	import { periodBounds, trailLabel } from '$lib/views/period';

	function periodPrefix(year: number, month: number, granularity: 'month' | 'quarter' | 'year'): string {
		if (granularity === 'year') return `${year}`;
		return trailLabel(year, month, granularity);
	}

	let { data }: { data: AccountsData } = $props();

	let selectedId = $state<string | null>(null);
	let adding = $state(false);
	let addError = $state<string | null>(null);
	let addBusy = $state(false);
	let deleteBusy = $state(false);
	let deleteError = $state<string | null>(null);

	const subtrees = $derived(
		ROOT_ORDER.map((key) => ({
			key,
			tree: buildSubtree(data.anchors[key].id, data.accounts)
		}))
	);

	function pathTo(id: string): { key: RootKey; nodes: AccountNode[] } | null {
		for (const { key, tree } of subtrees) {
			const trail: AccountNode[] = [];
			function walk(n: AccountNode): boolean {
				trail.push(n);
				if (n.account.id === id) return true;
				for (const c of n.children) if (walk(c)) return true;
				trail.pop();
				return false;
			}
			if (walk(tree)) return { key, nodes: trail };
		}
		return null;
	}

	const selectedTrail = $derived(selectedId ? pathTo(selectedId) : null);
	const selected = $derived(selectedTrail ? selectedTrail.nodes[selectedTrail.nodes.length - 1] : null);

	$effect(() => {
		accountsSelection.set(selected ? selected.account : null);
	});

	onDestroy(() => accountsSelection.set(null));

	const selectedRoot = $derived(selectedTrail?.key);
	const badge = $derived(selected?.account.role ? ROLE_BADGE[selected.account.role] : undefined);
	const selectedCanDelete = $derived(
		!!selected && selected.account.role === null && selected.children.length === 0
	);
	const selectedParent = $derived(
		selectedTrail && selectedTrail.nodes.length > 1
			? selectedTrail.nodes[selectedTrail.nodes.length - 2]
			: null
	);

	function collectIds(node: AccountNode, out: Set<string>) {
		out.add(node.account.id);
		for (const child of node.children) collectIds(child, out);
	}

	const RECENT_LIMIT = 10;
	const displaySign = $derived(
		selected && ['expense', 'revenue', 'equity'].includes(selected.account.account_type) ? -1 : 1
	);

	const bounds = $derived(
		periodBounds(periodState.year, periodState.month, periodState.granularity)
	);
	const periodValueLabel = $derived(
		periodPrefix(periodState.year, periodState.month, periodState.granularity)
	);
	const periodValue = $derived.by(() => {
		if (!selected) return 0;
		const ids = new Set<string>();
		collectIds(selected, ids);
		const isCumulative = ['asset', 'liability', 'equity'].includes(selected.account.account_type);
		let amount = 0;
		for (const entry of data.entries) {
			if (isCumulative) {
				if (entry.date > bounds.end) continue;
			} else {
				if (entry.date < bounds.start || entry.date > bounds.end) continue;
			}
			for (const posting of entry.postings) {
				if (ids.has(posting.account_id)) amount += posting.amount;
			}
		}
		return amount * displaySign;
	});
	const recentEntries = $derived.by(() => {
		if (!selected) return [] as { id: string; date: string; description: string; amount: number }[];
		const ids = new Set<string>();
		collectIds(selected, ids);
		return data.entries
			.map((entry) => {
				let amount = 0;
				let touched = false;
				for (const posting of entry.postings) {
					if (ids.has(posting.account_id)) {
						amount += posting.amount;
						touched = true;
					}
				}
				return touched
					? { id: entry.id, date: entry.date, description: entry.description, amount: amount * displaySign }
					: null;
			})
			.filter((entry): entry is NonNullable<typeof entry> => entry !== null)
			.sort((a, b) => (a.date < b.date ? 1 : a.date > b.date ? -1 : 0))
			.slice(0, RECENT_LIMIT);
	});

	function select(id: string) {
		selectedId = id;
		adding = false;
		addError = null;
		deleteError = null;
	}

	async function commitDeleteSelected() {
		if (!selected || !selectedCanDelete) return;
		if (!confirm(`Delete "${selected.account.name}"?`)) return;
		deleteBusy = true;
		deleteError = null;
		const result = await deleteAccount(selected.account.id);
		deleteBusy = false;
		if (result.status === 'success') {
			const nextId = selectedParent?.account.id ?? null;
			cache.invalidateAllExcept('accounts');
			await cache.refresh('accounts', loadAccounts);
			selectedId = nextId;
		} else {
			deleteError =
				result.status === 'fail'
					? String(Object.values(result.data)[0] ?? 'delete failed')
					: result.message;
		}
	}

	async function commitAdd(name: string) {
		if (!selected) return;
		const trimmed = name.trim();
		if (!trimmed) {
			adding = false;
			return;
		}
		addBusy = true;
		addError = null;
		const result = await createAccount({
			name: trimmed,
			parent_id: selected.account.id,
			currency: selected.account.currency
		});
		addBusy = false;
		if (result.status === 'success') {
			cache.invalidateAllExcept('accounts');
			await cache.refresh('accounts', loadAccounts);
			adding = false;
		} else {
			addError =
				result.status === 'fail'
					? String(Object.values(result.data)[0] ?? 'create failed')
					: result.message;
		}
	}
</script>

<div class="grid h-full grid-cols-[320px_1fr] border border-border bg-surface-card">
	<aside class="h-full overflow-y-auto border-r border-border-faint">
		{#each subtrees as { key, tree } (key)}
			{@const rootSel = selectedId === tree.account.id}
			<div class="border-b border-border-faint last:border-b-0">
				<button
					type="button"
					onclick={() => select(tree.account.id)}
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
							<TreeNode node={child} depth={0} {selectedId} onSelect={select} />
						{/each}
					</div>
				{/if}
			</div>
		{/each}
	</aside>

	<section class="h-full overflow-y-auto p-6">
		{#if selected && selectedTrail && selectedRoot}
			<nav class="mb-2 flex flex-wrap items-center gap-1 font-mono text-[11px] text-ink-muted">
				<span class={ROOT_ACCENT[selectedRoot]}>{ROOT_LABEL[selectedRoot]}</span>
				{#each selectedTrail.nodes.slice(1) as n (n.account.id)}
					<span>/</span>
					<span>{n.account.name}</span>
				{/each}
			</nav>
			<header class="mb-6 flex items-baseline justify-between">
				<div class="flex items-baseline gap-2">
					<h3 class="text-lg font-semibold text-ink">{selected.account.name}</h3>
					{#if badge}
						<span class="bg-surface-sunken px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-[0.06em] text-ink-muted">
							{badge}
						</span>
					{/if}
					{#if selectedCanDelete}
						<button
							type="button"
							onclick={commitDeleteSelected}
							disabled={deleteBusy}
							class="ml-2 border border-border bg-surface-card px-2 py-1 text-[11px] font-medium uppercase tracking-[0.06em] text-ink-body transition-colors hover:border-negative-strong hover:text-negative-strong disabled:opacity-50"
						>
							delete
						</button>
					{/if}
				</div>
				<span class="font-mono text-[11px] text-ink-muted">{selected.children.length} children</span>
			</header>
			<p class="mb-6 flex items-baseline gap-2 text-sm text-ink-muted">
				<span class="font-mono text-[11px] uppercase tracking-[0.06em]">{periodValueLabel} Value:</span>
				<span class="font-mono tabular-nums text-ink {periodValue < 0 ? 'text-negative' : ''}">
					<Num text={fmt(periodValue)} />
				</span>
			</p>
			{#if deleteError}
				<p class="mb-3 text-[11px] text-negative">{deleteError}</p>
			{/if}

			<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
				<div>
					<header class="mb-2 flex items-center justify-between">
						<h4 class="text-[11px] font-semibold uppercase tracking-[0.08em] text-ink-muted">Sub-accounts</h4>
						{#if !adding}
							<button
								type="button"
								onclick={() => {
									adding = true;
									addError = null;
								}}
								class="inline-flex items-center gap-1 border border-border bg-surface-card px-2 py-1 text-[11px] font-medium uppercase tracking-[0.06em] text-ink-body transition-colors hover:border-indigo-500 hover:text-indigo-500"
							>
								<span class="text-sm leading-none">+</span>
								<span>new</span>
							</button>
						{/if}
					</header>
					{#if selected.children.length > 0}
						<ul class="divide-y divide-border-faint border border-border-faint">
							{#each selected.children as c (c.account.id)}
								<li>
									<ChildRow node={c} onSelect={select} />
								</li>
							{/each}
						</ul>
					{:else if !adding}
						<p class="text-[12px] text-ink-muted">No sub-accounts.</p>
					{/if}
					{#if adding}
						<div class="mt-2 border border-border-faint px-3 py-2">
							<AddChildInput
								busy={addBusy}
								onCommit={commitAdd}
								onCancel={() => {
									adding = false;
									addError = null;
								}}
							/>
						</div>
					{/if}
					{#if addError}
						<p class="mt-1 text-[11px] text-negative">{addError}</p>
					{/if}
				</div>

				<div class="flex flex-col gap-4">
					<div>
						<h4 class="mb-2 text-[11px] font-semibold uppercase tracking-[0.08em] text-ink-muted">Recurring</h4>
						<p class="border border-dashed border-border-faint px-3 py-4 text-[12px] text-ink-muted">
							No recurring rules yet.
						</p>
					</div>
					<div>
						<h4 class="mb-2 text-[11px] font-semibold uppercase tracking-[0.08em] text-ink-muted">Recent entries</h4>
						{#if recentEntries.length > 0}
							<ul class="divide-y divide-border-faint border border-border-faint">
								{#each recentEntries as entry (entry.id)}
									<li class="flex items-baseline gap-3 px-3 py-2 text-sm">
										<span class="font-mono text-[11px] text-ink-muted">{entry.date}</span>
										<span class="flex-1 truncate text-ink">{entry.description}</span>
										<span class="font-mono tabular-nums {entry.amount < 0 ? 'text-negative' : 'text-ink-body'}">
											<Num text={fmt(entry.amount)} />
										</span>
									</li>
								{/each}
							</ul>
						{:else}
							<p class="border border-dashed border-border-faint px-3 py-4 text-[12px] text-ink-muted">
								No entries on this account.
							</p>
						{/if}
					</div>
				</div>
			</div>
		{:else}
			<p class="text-sm text-ink-muted">Select an account on the left.</p>
		{/if}
	</section>
</div>
