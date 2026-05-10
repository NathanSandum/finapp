<script lang="ts">
	import type { Entry } from './loader';
	import { formatAmount } from './loader';
	import EntryList from './EntryList.svelte';

	let {
		entries,
		accountNames
	}: { entries: Entry[]; accountNames: Record<string, string> } = $props();

	let selectedId = $state<string | null>(null);
	const selected = $derived(entries.find((e) => e.id === selectedId) ?? entries[0]);

	function entryAmount(entry: Entry): number {
		return entry.postings.filter((p) => p.amount > 0).reduce((s, p) => s + p.amount, 0);
	}

	function accountLabel(id: string): string {
		return accountNames[id] ?? id;
	}

	const postings = $derived(
		selected
			? [...selected.postings].sort((a, b) => (b.amount > 0 ? 1 : 0) - (a.amount > 0 ? 1 : 0))
			: []
	);
</script>

<div class="grid h-[calc(100vh-160px)] grid-cols-[320px_1fr] gap-0 overflow-hidden rounded-lg border border-border bg-surface-card">
	<aside class="overflow-hidden border-r border-ink-faint">
		<EntryList {entries} selectedId={selected?.id ?? null} onSelect={(id) => (selectedId = id)} />
	</aside>

	<section class="overflow-y-auto p-6">
		{#if selected}
			<header class="mb-4 flex items-baseline justify-between">
				<div>
					<h3 class="text-lg font-semibold text-ink">{selected.description}</h3>
					<p class="font-mono text-xs text-ink-muted">{selected.date} · {selected.id}</p>
				</div>
				<span class="font-mono text-xl tabular-nums text-ink">{formatAmount(entryAmount(selected))}</span>
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
							<span class="truncate text-sm text-ink">{accountLabel(posting.account_id)}</span>
							{#if posting.note}
								<span class="truncate text-[11px] text-ink-muted">{posting.note}</span>
							{/if}
						</div>
						<span class="font-mono text-base tabular-nums {isDebit ? 'text-ink' : 'text-ink-body'}">
							{formatAmount(Math.abs(posting.amount))}
						</span>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
</div>
