<script lang="ts">
	import { flip } from 'svelte/animate';
	import { entries, accounts, anchors, entryAmount } from '$lib/data';
	import { periodBounds } from '$lib/views/period';
	import { periodState } from '$lib/views/period-state.svelte';
	import { tabState } from '$lib/tabs/state.svelte';
	import { fmt } from '$lib/format';
	import Num from '$lib/components/Num.svelte';
	import EntryForm from './EntryForm.svelte';
	import EntryDetail from './EntryDetail.svelte';
	import BorderButton from '$lib/components/BorderButton.svelte';
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

<div class="flex h-full flex-col">
	<div class="glass grid min-h-0 flex-1 grid-cols-[360px_1fr] overflow-hidden rounded-2xl">
		<ul
			class="relative h-full overflow-y-auto border-r border-[var(--color-glass-divider)] p-2"
		>
			{#each filtered as entry (entry.id)}
				{@const isSel = visibleSelected?.id === entry.id && !creating}
				<li
					animate:flip={{ duration: DURATION.base }}
					in:rowEnter
					out:rowExit
					class="relative mb-1"
				>
					<button
						type="button"
						onclick={() => {
							selectedId = entry.id;
							creating = false;
						}}
						class="entry-row group relative flex w-full flex-col items-start gap-1 rounded-xl px-3.5 py-3 text-left transition-all duration-300 {isSel
							? 'is-selected'
							: ''}"
					>
						<span class="entry-row__sheen pointer-events-none absolute inset-0 rounded-xl"></span>
						<span class="relative flex w-full items-baseline justify-between">
							<span class="font-mono text-[11px] tracking-tight text-ink-muted">
								{entry.date}
							</span>
							<span class="font-mono text-[12px] tabular-nums text-ink">
								<Num text={fmt(entryAmount(entry))} />
							</span>
						</span>
						<span class="relative line-clamp-1 text-[13px] font-medium text-ink">
							{entry.description}
						</span>
					</button>
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
					<div in:panelEnter class="p-7">
						<header class="mb-6 flex items-baseline justify-between">
							<h3 class="font-display text-xl font-semibold tracking-tight text-ink">
								New entry
							</h3>
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
					<div in:panelEnter class="p-7">
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

<style>
	.entry-row__sheen {
		opacity: 0;
		background: linear-gradient(
			135deg,
			oklch(75% 0.14 268 / 0.12) 0%,
			oklch(82% 0.1 220 / 0.06) 60%,
			transparent 100%
		);
		transition: opacity 240ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.entry-row:hover .entry-row__sheen {
		opacity: 1;
	}
	.entry-row.is-selected {
		background: oklch(75% 0.14 268 / 0.14);
		box-shadow:
			inset 0 0 0 1px oklch(58% 0.2 268 / 0.4),
			0 8px 24px -10px oklch(58% 0.2 268 / 0.35);
	}
	.entry-row.is-selected .entry-row__sheen {
		opacity: 1;
	}
</style>
