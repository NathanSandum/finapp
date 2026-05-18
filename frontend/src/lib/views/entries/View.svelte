<script lang="ts">
	import { periodBounds } from '$lib/views/period';
	import { periodState } from '$lib/views/period-state.svelte';
	import { entryAmount, type EntriesData } from './loader';
	import { fmt } from '$lib/format';
	import Num from '$lib/components/Num.svelte';
	import EntryForm from './EntryForm.svelte';
	import { entryIntent } from './intent.svelte';
	import { tabState } from '$lib/tabs/state.svelte';
	import type { VerbId } from './verbs';

	let { data }: { data: EntriesData } = $props();

	let selectedId = $state<string | null>(null);
	let creating = $state(false);
	let formVerb = $state<VerbId>('spent');
	let formCounterpartyId = $state('');

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

	const bounds = $derived(periodBounds(periodState.year, periodState.month, periodState.granularity));
	const filteredEntries = $derived(
		data.entries.filter((e) => e.date >= bounds.start && e.date <= bounds.end)
	);
	const selected = $derived(
		filteredEntries.find((e) => e.id === selectedId) ?? filteredEntries[0]
	);
	const postings = $derived(
		selected
			? [...selected.postings].sort((a, b) => (b.amount > 0 ? 1 : 0) - (a.amount > 0 ? 1 : 0))
			: []
	);
</script>

<div class="flex h-full flex-col gap-4">
	<div class="grid min-h-0 flex-1 grid-cols-[320px_1fr] gap-0 overflow-hidden border border-border bg-surface-card">
		<ul class="h-full divide-y divide-border-faint overflow-y-auto border-r border-ink-faint">
			{#each filteredEntries as entry (entry.id)}
				{@const isSel = selected?.id === entry.id}
				<li>
					<button
						type="button"
						onclick={() => (selectedId = entry.id)}
						class="flex w-full flex-col items-start gap-0.5 px-4 py-3 text-left transition-colors {isSel
							? 'bg-surface-inverse text-ink-inverse'
							: 'hover:bg-surface-hover-faint'}"
					>
						<span class="flex w-full items-baseline justify-between">
							<span class="font-mono text-xs {isSel ? 'text-ink-inverse-muted' : 'text-ink-muted'}">
								{entry.date}
							</span>
							<span
								class="font-mono text-xs tabular-nums {isSel ? 'text-ink-inverse' : 'text-ink-body'}"
							>
								<Num text={fmt(entryAmount(entry))} />
							</span>
						</span>
						<span class="text-sm font-medium">{entry.description}</span>
					</button>
				</li>
			{/each}
		</ul>

		<section class="overflow-y-auto p-6">
			{#if creating}
				<header class="mb-4 flex items-baseline justify-between">
					<h3 class="text-lg font-semibold text-ink">New entry</h3>
				</header>
				{#key formVerb}
					<EntryForm
						accounts={data.accounts}
						anchors={data.anchors}
						initialVerb={formVerb}
						initialCounterpartyId={formCounterpartyId}
						onClose={closeForm}
					/>
				{/key}
			{:else if selected}
				<header class="mb-4 flex items-baseline justify-between">
					<div>
						<h3 class="text-lg font-semibold text-ink">{selected.description}</h3>
						<p class="font-mono text-xs text-ink-muted">{selected.date} · {selected.id}</p>
					</div>
					<span class="font-mono text-xl tabular-nums text-ink"><Num text={fmt(entryAmount(selected))} /></span>
				</header>

				<ul class="divide-y divide-border-faint">
					{#each postings as posting (posting.id)}
						{@const isDebit = posting.amount > 0}
						<li class="flex items-center gap-4 py-3">
							<span
								class="inline-flex h-5 w-10 items-center justify-center text-[10px] font-semibold uppercase tracking-[0.08em] {isDebit
									? 'bg-positive-tint-strong text-positive'
									: 'bg-negative-tint text-negative-strong'}"
							>
								{isDebit ? 'DR' : 'CR'}
							</span>
							<div class="flex flex-1 flex-col gap-0.5 min-w-0">
								<span class="truncate text-sm text-ink">
									{data.accountNames[posting.account_id] ?? posting.account_id}
								</span>
								{#if posting.note}
									<span class="truncate text-[11px] text-ink-muted">{posting.note}</span>
								{/if}
							</div>
							<span class="font-mono text-base tabular-nums {isDebit ? 'text-ink' : 'text-ink-body'}">
								<Num text={fmt(Math.abs(posting.amount))} />
							</span>
						</li>
					{/each}
				</ul>
			{/if}
		</section>
	</div>
</div>
