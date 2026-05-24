<script lang="ts">
	import { flip } from 'svelte/animate';
	import { entries, accounts, anchors, entryAmount } from '$lib/data';
	import { periodBounds, periodPrefix } from '$lib/views/period';
	import { periodState } from '$lib/views/period-state.svelte';
	import { tabState } from '$lib/tabs/state.svelte';
	import { fmt } from '$lib/format';
	import Num from '$lib/components/Num.svelte';
	import EntryForm from './EntryForm.svelte';
	import EntryDetail from './EntryDetail.svelte';
	import BorderButton from '$lib/components/BorderButton.svelte';
	import SolidButton from '$lib/components/SolidButton.svelte';
	import { entryIntent } from './intent.svelte';
	import { rowEnter, rowExit, panelEnter, DURATION } from '$lib/utils/motion';
	import type { VerbId } from './verbs';

	let selectedId = $state<string | null>(null);
	let creating = $state(false);
	let formVerb = $state<VerbId>('spent');
	let formCounterpartyId = $state('');

	const entryList = $derived(entries.value ?? []);
	const accountList = $derived(accounts.value ?? []);
	const anchorData = $derived(anchors.value);
	const accountNames = $derived.by(() => {
		const map: Record<string, string> = {};
		for (const account of accountList) map[account.id] = account.name;
		return map;
	});
	const bounds = $derived(
		periodBounds(periodState.year, periodState.month, periodState.granularity)
	);
	const periodLabel = $derived(
		periodPrefix(periodState.year, periodState.month, periodState.granularity)
	);
	const filtered = $derived(
		entryList.filter((e) => e.date >= bounds.start && e.date <= bounds.end)
	);
	const visibleSelected = $derived(filtered.find((e) => e.id === selectedId) ?? filtered[0]);

	const incomingIntent = $derived(entryIntent.get());
	$effect(() => {
		const intent = incomingIntent;
		if (intent) {
			creating = true;
			formVerb = intent.verb;
			formCounterpartyId = intent.counterpartyId ?? '';
		}
	});

	function returnToSourceTab() {
		const intent = entryIntent.get();
		if (intent) {
			tabState.set(intent.sourceTab);
			entryIntent.clear();
		}
	}

	function openCreate() {
		formVerb = 'spent';
		formCounterpartyId = '';
		creating = true;
	}

	function closeForm() {
		creating = false;
		returnToSourceTab();
	}

	function handleCreated(id: string) {
		selectedId = id;
	}

	async function deleteEntry(id: string) {
		const next = filtered.find((e) => e.id !== id);
		await entries.remove(id);
		selectedId = next?.id ?? null;
	}
</script>

<div class="flex h-full flex-col gap-3">
	<header class="flex items-baseline justify-between px-1">
		<div class="flex items-baseline gap-3">
			<h2 class="font-display text-xl font-semibold tracking-tight text-ink">Entries</h2>
			<span class="font-ui text-[11px] uppercase tracking-[0.1em] text-ink-muted">
				{filtered.length} · {periodLabel}
			</span>
		</div>
		{#if !creating}
			<div in:panelEnter={{ offset: 8 }}>
				<SolidButton size="sm" onclick={openCreate}>+ New entry</SolidButton>
			</div>
		{/if}
	</header>

	<div
		class="grid min-h-0 flex-1 grid-cols-[360px_1fr] overflow-hidden border border-border bg-surface-card shadow-[0_1px_0_rgba(0,0,0,0.02)]"
	>
		<ul class="relative h-full divide-y divide-border-faint overflow-y-auto border-r border-border-faint">
			{#each filtered as entry (entry.id)}
				{@const isSel = visibleSelected?.id === entry.id && !creating}
				<li
					animate:flip={{ duration: DURATION.base }}
					in:rowEnter
					out:rowExit
					class="relative"
				>
					<button
						type="button"
						onclick={() => {
							selectedId = entry.id;
							creating = false;
						}}
						class="flex w-full flex-col items-start gap-1 px-4 py-3 text-left transition-colors duration-200 {isSel
							? 'bg-surface-inverse text-ink-inverse'
							: 'hover:bg-surface-hover-faint'}"
					>
						<span class="flex w-full items-baseline justify-between">
							<span
								class="font-mono text-[11px] tracking-tight {isSel
									? 'text-ink-inverse-muted'
									: 'text-ink-muted'}"
							>
								{entry.date}
							</span>
							<span
								class="font-mono text-[12px] tabular-nums {isSel
									? 'text-ink-inverse'
									: 'text-ink-body'}"
							>
								<Num text={fmt(entryAmount(entry))} />
							</span>
						</span>
						<span class="line-clamp-1 text-[13px] font-medium">{entry.description}</span>
					</button>
					{#if isSel}
						<span
							in:panelEnter={{ offset: 4, duration: DURATION.fast }}
							class="pointer-events-none absolute inset-y-0 left-0 w-[2px] bg-brand"
						></span>
					{/if}
				</li>
			{:else}
				<li class="px-4 py-10 text-center text-[12px] text-ink-muted">
					No entries in this period.
				</li>
			{/each}
		</ul>

		<section class="relative overflow-y-auto">
			{#if creating && anchorData}
				{#key formVerb}
					<div in:panelEnter class="p-6">
						<header class="mb-5 flex items-baseline justify-between">
							<h3 class="font-display text-lg font-semibold text-ink">New entry</h3>
							<BorderButton onclick={closeForm}>cancel</BorderButton>
						</header>
						<EntryForm
							accounts={accountList}
							anchors={anchorData}
							initialVerb={formVerb}
							initialCounterpartyId={formCounterpartyId}
							onClose={closeForm}
							onCreated={handleCreated}
						/>
					</div>
				{/key}
			{:else if visibleSelected}
				{#key visibleSelected.id}
					<div in:panelEnter class="p-6">
						<EntryDetail
							entry={visibleSelected}
							{accountNames}
							onDelete={() => deleteEntry(visibleSelected.id)}
						/>
					</div>
				{/key}
			{:else}
				<div class="flex h-full items-center justify-center p-6 text-[12px] text-ink-muted">
					Select an entry to inspect — or create one.
				</div>
			{/if}
		</section>
	</div>
</div>
