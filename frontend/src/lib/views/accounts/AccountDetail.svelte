<script lang="ts">
	import { cache } from '$lib/api/cache.svelte';
	import { createAccount, deleteAccount } from './api';
	import { load as loadAccounts } from './loader';
	import {
		ROLE_BADGE,
		ROOT_ACCENT,
		ROOT_LABEL,
		type AccountNode,
		type AccountTrail
	} from './tree';
	import { displaySignOf, periodValueOf, recentEntriesOf } from './aggregate';
	import { periodBounds, periodPrefix } from '$lib/views/period';
	import { periodState } from '$lib/views/period-state.svelte';
	import { fmt } from '$lib/format';
	import Num from '$lib/components/Num.svelte';
	import BorderButton from '$lib/components/BorderButton.svelte';
	import type { Account, Anchors, Entry } from '$lib/views/entries/loader';
	import type { RecurringRule } from '$lib/views/recurring/loader';
	import RulesList from '$lib/views/recurring/RulesList.svelte';
	import RuleForm from '$lib/views/recurring/RuleForm.svelte';
	import SubAccountsPanel from './SubAccountsPanel.svelte';
	import RecentEntriesPanel from './RecentEntriesPanel.svelte';

	const RECENT_LIMIT = 10;

	let {
		selected,
		trail,
		entries,
		accounts,
		anchors,
		recurringRules,
		onSelect,
		onAfterDelete
	}: {
		selected: AccountNode;
		trail: AccountTrail;
		entries: Entry[];
		accounts: Account[];
		anchors: Anchors;
		recurringRules: RecurringRule[];
		onSelect: (id: string) => void;
		onAfterDelete: (nextId: string | null) => void;
	} = $props();

	const badge = $derived(selected.account.role ? ROLE_BADGE[selected.account.role] : undefined);
	const canDelete = $derived(
		selected.account.role === null && selected.children.length === 0
	);
	const parentNode = $derived(trail.nodes.length > 1 ? trail.nodes[trail.nodes.length - 2] : null);

	const displaySign = $derived(displaySignOf(selected.account.account_type));
	const bounds = $derived(
		periodBounds(periodState.year, periodState.month, periodState.granularity)
	);
	const valueLabel = $derived(
		periodPrefix(periodState.year, periodState.month, periodState.granularity)
	);
	const periodValue = $derived(periodValueOf(selected, entries, bounds, displaySign));
	const recentEntries = $derived(recentEntriesOf(selected, entries, displaySign, RECENT_LIMIT));

	let adding = $state(false);
	let addBusy = $state(false);
	let addError = $state<string | null>(null);
	let deleteBusy = $state(false);
	let deleteError = $state<string | null>(null);
	let addingRule = $state(false);

	const accountNames = $derived.by(() => {
		const map: Record<string, string> = {};
		for (const account of accounts) map[account.id] = account.name;
		return map;
	});

	const rulesForAccount = $derived(
		recurringRules.filter((rule) =>
			rule.templates.some((template) => template.account === selected.account.id)
		)
	);

	$effect(() => {
		selected.account.id;
		adding = false;
		addingRule = false;
		addError = null;
		deleteError = null;
	});

	async function commitDelete() {
		if (!canDelete) return;
		if (!confirm(`Delete "${selected.account.name}"?`)) return;
		deleteBusy = true;
		deleteError = null;
		const result = await deleteAccount(selected.account.id);
		deleteBusy = false;
		if (result.status === 'success') {
			const nextId = parentNode?.account.id ?? null;
			cache.invalidateAllExcept('accounts');
			await cache.refresh('accounts', loadAccounts);
			onAfterDelete(nextId);
		} else {
			deleteError =
				result.status === 'fail'
					? String(Object.values(result.data)[0] ?? 'delete failed')
					: result.message;
		}
	}

	async function commitAdd(name: string) {
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

<section class="h-full overflow-y-auto p-6">
	<nav class="mb-2 flex flex-wrap items-center gap-1 font-mono text-[11px] text-ink-muted">
		<span class={ROOT_ACCENT[trail.key]}>{ROOT_LABEL[trail.key]}</span>
		{#each trail.nodes.slice(1) as n (n.account.id)}
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
			{#if canDelete}
				<BorderButton tone="negative" onclick={commitDelete} disabled={deleteBusy} class="ml-2">
					delete
				</BorderButton>
			{/if}
		</div>
		<span class="font-mono text-[11px] text-ink-muted">{selected.children.length} children</span>
	</header>
	<p class="mb-6 flex items-baseline gap-2 text-sm text-ink-muted">
		<span class="font-mono text-[11px] uppercase tracking-[0.06em]">{valueLabel} Value:</span>
		<span class="font-mono tabular-nums text-ink {periodValue < 0 ? 'text-negative' : ''}">
			<Num text={fmt(periodValue)} />
		</span>
	</p>
	{#if deleteError}
		<p class="mb-3 text-[11px] text-negative">{deleteError}</p>
	{/if}

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
		<SubAccountsPanel
			{selected}
			{adding}
			{addBusy}
			{addError}
			{onSelect}
			onStartAdd={() => {
				adding = true;
				addError = null;
			}}
			onCommitAdd={commitAdd}
			onCancelAdd={() => {
				adding = false;
				addError = null;
			}}
		/>

		<div class="flex flex-col gap-4">
			<div>
				<div class="mb-2 flex items-baseline justify-between">
					<h4 class="text-[11px] font-semibold uppercase tracking-[0.08em] text-ink-muted">Recurring</h4>
					{#if !addingRule}
						<BorderButton onclick={() => (addingRule = true)}>New rule</BorderButton>
					{/if}
				</div>
				{#if addingRule}
					<div class="mb-3 border border-border bg-surface-card p-3">
						<RuleForm
							{accounts}
							{anchors}
							initialCounterpartyId={selected.account.id}
							onClose={() => (addingRule = false)}
						/>
					</div>
				{/if}
				<RulesList
					rules={rulesForAccount}
					{accountNames}
					emptyMessage="No recurring rules touching this account."
				/>
			</div>
			<RecentEntriesPanel entries={recentEntries} />
		</div>
	</div>
</section>
