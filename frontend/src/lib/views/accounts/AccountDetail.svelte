<script lang="ts">
	import { fly, slide } from 'svelte/transition';
	import { expoOut } from 'svelte/easing';
	import { accounts, entries, recurring } from '$lib/data';
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
	import type { Anchors } from '$lib/data';
	import RulesList from '$lib/views/recurring/RulesList.svelte';
	import RuleForm from '$lib/views/recurring/RuleForm.svelte';
	import SubAccountsPanel from './SubAccountsPanel.svelte';
	import RecentEntriesPanel from './RecentEntriesPanel.svelte';
	import { DURATION } from '$lib/utils/motion';

	const RECENT_LIMIT = 10;

	let {
		selected,
		trail,
		anchors,
		onSelect,
		onAfterDelete
	}: {
		selected: AccountNode;
		trail: AccountTrail;
		anchors: Anchors;
		onSelect: (id: string) => void;
		onAfterDelete: (nextId: string | null) => void;
	} = $props();

	const accountList = $derived(accounts.value ?? []);
	const entryList = $derived(entries.value ?? []);
	const recurringRules = $derived(recurring.value ?? []);

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
	const periodValue = $derived(periodValueOf(selected, entryList, bounds, displaySign));
	const recentEntries = $derived(
		recentEntriesOf(selected, entryList, displaySign, RECENT_LIMIT)
	);

	let adding = $state(false);
	let addBusy = $state(false);
	let addError = $state<string | null>(null);
	let confirmingDelete = $state(false);
	let deleteBusy = $state(false);
	let deleteError = $state<string | null>(null);
	let addingRule = $state(false);

	const accountNames = $derived.by(() => {
		const map: Record<string, string> = {};
		for (const account of accountList) map[account.id] = account.name;
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
		confirmingDelete = false;
	});

	async function commitDelete() {
		if (!canDelete) return;
		deleteBusy = true;
		deleteError = null;
		const result = await accounts.remove(selected.account.id);
		deleteBusy = false;
		if (result.status === 'success') {
			const nextId = parentNode?.account.id ?? null;
			onAfterDelete(nextId);
		} else {
			deleteError =
				result.status === 'fail'
					? String(Object.values(result.data)[0] ?? 'delete failed')
					: result.message;
		}
		confirmingDelete = false;
	}

	async function commitAdd(name: string) {
		const trimmed = name.trim();
		if (!trimmed) {
			adding = false;
			return;
		}
		addBusy = true;
		addError = null;
		const result = await accounts.create({
			name: trimmed,
			parentId: selected.account.id,
			currency: selected.account.currency
		});
		addBusy = false;
		if (result.status === 'success') {
			adding = false;
		} else {
			addError =
				result.status === 'fail'
					? String(Object.values(result.data)[0] ?? 'create failed')
					: result.message;
		}
	}
</script>

<div class="flex h-full flex-col gap-5 p-6">
	<nav class="flex flex-wrap items-center gap-1.5 font-mono text-[11px] text-ink-muted">
		<span class="{ROOT_ACCENT[trail.key]} font-semibold uppercase tracking-[0.08em]">
			{ROOT_LABEL[trail.key]}
		</span>
		{#each trail.nodes.slice(1, -1) as n (n.account.id)}
			<span class="opacity-50">/</span>
			<button
				type="button"
				class="hover:text-ink-body"
				onclick={() => onSelect(n.account.id)}
			>
				{n.account.name}
			</button>
		{/each}
	</nav>

	<header class="flex items-baseline justify-between gap-4">
		<div class="flex items-baseline gap-2">
			<h3 class="font-display text-2xl font-semibold tracking-tight text-ink">
				{selected.account.name}
			</h3>
			{#if badge}
				<span
					class="bg-surface-sunken px-1.5 py-0.5 font-ui text-[10px] font-medium uppercase tracking-[0.08em] text-ink-muted"
				>
					{badge}
				</span>
			{/if}
		</div>
		<span class="font-ui text-[10px] uppercase tracking-[0.1em] text-ink-muted">
			{selected.children.length}
			{selected.children.length === 1 ? 'child' : 'children'}
		</span>
	</header>

	<div class="flex items-baseline gap-3 border-y border-border-faint py-3">
		<span class="font-ui text-[10px] uppercase tracking-[0.1em] text-ink-muted">
			{valueLabel} value
		</span>
		<span
			class="font-mono text-2xl tabular-nums {periodValue < 0 ? 'text-negative' : 'text-ink'}"
		>
			<Num text={fmt(periodValue)} />
		</span>
	</div>

	{#if deleteError}
		<p
			in:fly={{ y: -4, duration: DURATION.fast }}
			class="border border-negative-border bg-negative-tint px-3 py-2 text-[11px] text-negative-strong"
		>
			{deleteError}
		</p>
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

		<div class="flex flex-col gap-6">
			<div>
				<div class="mb-2 flex items-baseline justify-between">
					<h4
						class="font-ui text-[10px] font-semibold uppercase tracking-[0.12em] text-ink-muted"
					>
						Recurring
					</h4>
					{#if !addingRule}
						<BorderButton onclick={() => (addingRule = true)}>+ new rule</BorderButton>
					{/if}
				</div>
				{#if addingRule}
					<div
						transition:slide={{ duration: DURATION.base, easing: expoOut }}
						class="mb-3 border border-border bg-surface-card p-4"
					>
						<RuleForm
							accounts={accountList}
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

	{#if canDelete}
		<footer class="mt-auto flex items-center justify-end gap-2 border-t border-border-faint pt-4">
			{#if confirmingDelete}
				<span class="text-[11px] text-ink-muted">Delete this account?</span>
				<BorderButton onclick={() => (confirmingDelete = false)} disabled={deleteBusy}>
					cancel
				</BorderButton>
				<BorderButton tone="negative" onclick={commitDelete} disabled={deleteBusy}>
					{deleteBusy ? 'deleting…' : 'confirm'}
				</BorderButton>
			{:else}
				<BorderButton tone="negative" onclick={() => (confirmingDelete = true)}>
					delete account
				</BorderButton>
			{/if}
		</footer>
	{/if}
</div>
