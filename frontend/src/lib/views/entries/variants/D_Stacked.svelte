<script lang="ts">
	import type { Entry } from '../loader';
	import { formatAmount } from '../loader';

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

	function shortDate(iso: string): string {
		return iso.slice(5);
	}

	const postings = $derived(
		selected
			? [...selected.postings].sort((a, b) => (b.amount > 0 ? 1 : 0) - (a.amount > 0 ? 1 : 0))
			: []
	);
</script>

<div class="grid h-[calc(100vh-160px)] grid-cols-[360px_1fr] gap-0 overflow-hidden border border-border bg-surface-card">
	<aside class="flex flex-col overflow-hidden border-r border-border">
		<div class="grid grid-cols-[60px_1fr_auto] gap-3 border-b border-border bg-surface-sunken px-4 py-2 text-[10px] font-semibold uppercase tracking-[0.08em] text-ink-muted">
			<span>Date</span>
			<span>Description</span>
			<span>Amount</span>
		</div>
		<ul class="flex-1 overflow-y-auto">
			{#each entries as entry (entry.id)}
				{@const isSel = selected?.id === entry.id}
				<li>
					<button
						type="button"
						onclick={() => (selectedId = entry.id)}
						class="grid w-full grid-cols-[60px_1fr_auto] items-baseline gap-3 border-b border-border-faint px-4 py-2 text-left transition-colors {isSel
							? 'border-l-2 border-l-brand bg-surface-sunken'
							: 'border-l-2 border-l-transparent hover:bg-surface-hover-faint'}"
					>
						<span class="font-mono text-xs tabular-nums text-ink-muted">{shortDate(entry.date)}</span>
						<span class="truncate text-sm text-ink">{entry.description}</span>
						<span class="font-mono text-xs tabular-nums text-ink-body">{formatAmount(entryAmount(entry))}</span>
					</button>
				</li>
			{/each}
		</ul>
	</aside>

	<section class="overflow-y-auto">
		{#if selected}
			<header class="flex items-baseline justify-between border-b border-border px-6 py-4">
				<div class="flex flex-col gap-1">
					<h3 class="font-display text-lg font-semibold text-ink">{selected.description}</h3>
					<p class="font-mono text-[11px] uppercase tracking-[0.06em] text-ink-muted">
						{selected.date} · {selected.id}
					</p>
				</div>
				<span class="font-mono text-2xl tabular-nums text-ink">{formatAmount(entryAmount(selected))}</span>
			</header>

			<ul class="divide-y divide-border-faint px-6">
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
