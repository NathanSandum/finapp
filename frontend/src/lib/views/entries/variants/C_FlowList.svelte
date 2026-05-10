<script lang="ts">
	import type { Entry } from '../loader';
	import { formatAmount } from '../loader';
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

	const debits = $derived(selected ? selected.postings.filter((p) => p.amount > 0) : []);
	const credits = $derived(selected ? selected.postings.filter((p) => p.amount < 0) : []);
</script>

<div class="grid h-[calc(100vh-160px)] grid-cols-[320px_1fr] gap-4 overflow-hidden">
	<aside class="overflow-hidden rounded-lg border border-border bg-surface-card">
		<EntryList {entries} selectedId={selected?.id ?? null} onSelect={(id) => (selectedId = id)} />
	</aside>

	<section class="overflow-y-auto rounded-lg border border-border bg-surface-card p-6">
		{#if selected}
			<header class="mb-6 flex items-baseline justify-between">
				<div>
					<h3 class="text-lg font-semibold text-ink">{selected.description}</h3>
					<p class="font-mono text-xs text-ink-muted">{selected.date} · {selected.id}</p>
				</div>
				<span class="font-mono text-xl tabular-nums text-ink">{formatAmount(entryAmount(selected))}</span>
			</header>

			<div class="grid grid-cols-[1fr_24px_1fr] gap-0">
				<div class="flex flex-col gap-2">
					<span class="text-[10px] font-semibold uppercase tracking-[0.08em] text-ink-muted">From (credits)</span>
					{#each credits as posting (posting.id)}
						<div class="flex items-baseline justify-between gap-3 border-l-2 border-negative-line bg-surface-sunken py-2 pl-3 pr-3">
							<div class="flex flex-col gap-0.5 min-w-0">
								<span class="truncate text-sm text-ink-body">{accountLabel(posting.account_id)}</span>
								{#if posting.note}
									<span class="truncate text-[11px] text-ink-muted">{posting.note}</span>
								{/if}
							</div>
							<span class="font-mono text-sm tabular-nums text-ink">{formatAmount(Math.abs(posting.amount))}</span>
						</div>
					{/each}
				</div>

				<div class="flex flex-col items-center justify-center pt-6 text-ink-faint">
					<span class="font-mono text-xl">→</span>
				</div>

				<div class="flex flex-col gap-2">
					<span class="text-[10px] font-semibold uppercase tracking-[0.08em] text-ink-muted">To (debits)</span>
					{#each debits as posting (posting.id)}
						<div class="flex items-baseline justify-between gap-3 border-l-2 border-positive-line bg-surface-sunken py-2 pl-3 pr-3">
							<div class="flex flex-col gap-0.5 min-w-0">
								<span class="truncate text-sm text-ink-body">{accountLabel(posting.account_id)}</span>
								{#if posting.note}
									<span class="truncate text-[11px] text-ink-muted">{posting.note}</span>
								{/if}
							</div>
							<span class="font-mono text-sm tabular-nums text-ink">{formatAmount(posting.amount)}</span>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</section>
</div>
