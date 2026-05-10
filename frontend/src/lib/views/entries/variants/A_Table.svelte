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
</script>

<div class="grid h-[calc(100vh-160px)] grid-cols-[320px_1fr] gap-4 overflow-hidden">
	<EntryList {entries} selectedId={selected?.id ?? null} onSelect={(id) => (selectedId = id)} />

	<section class="overflow-y-auto rounded-lg border border-border bg-surface-card p-6">
		{#if selected}
			<header class="mb-4 flex items-baseline justify-between">
				<div>
					<h3 class="text-lg font-semibold text-ink">{selected.description}</h3>
					<p class="font-mono text-xs text-ink-muted">{selected.date} · {selected.id}</p>
				</div>
				<span class="font-mono text-xl tabular-nums text-ink">{formatAmount(entryAmount(selected))}</span>
			</header>

			<table class="w-full text-sm">
				<thead class="text-left text-xs font-medium uppercase tracking-wide text-ink-muted">
					<tr>
						<th class="py-2 pr-4">Account</th>
						<th class="py-2 pr-4">Note</th>
						<th class="py-2 text-right">Amount</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-border-faint">
					{#each selected.postings as posting (posting.id)}
						<tr>
							<td class="py-2 pr-4 text-ink-body">{accountLabel(posting.account_id)}</td>
							<td class="py-2 pr-4 text-ink-muted">{posting.note}</td>
							<td
								class="py-2 text-right font-mono tabular-nums {posting.amount < 0
									? 'text-negative-strong'
									: 'text-positive'}"
							>
								{formatAmount(posting.amount)}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</section>
</div>
