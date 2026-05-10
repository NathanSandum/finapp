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

	function flowLabel(entry: Entry): string {
		const debits = entry.postings.filter((p) => p.amount > 0);
		const credits = entry.postings.filter((p) => p.amount < 0);
		const dr = debits.length === 1 ? accountLabel(debits[0].account_id) : `${debits.length} debits`;
		const cr = credits.length === 1 ? accountLabel(credits[0].account_id) : `${credits.length} credits`;
		return `${cr} → ${dr}`;
	}

	const debits = $derived(selected ? selected.postings.filter((p) => p.amount > 0) : []);
	const credits = $derived(selected ? selected.postings.filter((p) => p.amount < 0) : []);
</script>

<div class="grid h-[calc(100vh-160px)] grid-cols-[440px_1fr] gap-0 overflow-hidden border border-border bg-surface-card">
	<aside class="flex flex-col overflow-hidden border-r border-border">
		<div class="grid grid-cols-[56px_1fr_auto] gap-3 border-b border-border bg-surface-sunken px-4 py-2 text-[10px] font-semibold uppercase tracking-[0.08em] text-ink-muted">
			<span>Date</span>
			<span>Flow</span>
			<span>Amount</span>
		</div>
		<ul class="flex-1 overflow-y-auto">
			{#each entries as entry (entry.id)}
				{@const isSel = selected?.id === entry.id}
				<li>
					<button
						type="button"
						onclick={() => (selectedId = entry.id)}
						class="grid w-full grid-cols-[56px_1fr_auto] items-baseline gap-3 border-b border-border-faint px-4 py-2 text-left transition-colors {isSel
							? 'border-l-2 border-l-brand bg-surface-sunken'
							: 'border-l-2 border-l-transparent hover:bg-surface-hover-faint'}"
					>
						<span class="font-mono text-xs tabular-nums text-ink-muted">{shortDate(entry.date)}</span>
						<span class="flex flex-col gap-0.5 truncate">
							<span class="truncate text-sm text-ink">{entry.description}</span>
							<span class="truncate font-mono text-[11px] text-ink-muted">{flowLabel(entry)}</span>
						</span>
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

			<div class="grid grid-cols-[1fr_24px_1fr] gap-0 px-6 py-6">
				<div class="flex flex-col gap-2">
					<span class="text-[10px] font-semibold uppercase tracking-[0.08em] text-ink-muted">From (credits)</span>
					{#each credits as posting (posting.id)}
						<div class="flex flex-col gap-0.5 border-l-2 border-negative-line bg-surface-sunken py-2 pl-3 pr-3">
							<span class="text-sm text-ink-body">{accountLabel(posting.account_id)}</span>
							{#if posting.note}
								<span class="text-[11px] text-ink-muted">{posting.note}</span>
							{/if}
							<span class="mt-1 font-mono text-sm tabular-nums text-ink">{formatAmount(Math.abs(posting.amount))}</span>
						</div>
					{/each}
				</div>

				<div class="flex flex-col items-center justify-center pt-6 text-ink-faint">
					<span class="font-mono text-xl">→</span>
				</div>

				<div class="flex flex-col gap-2">
					<span class="text-[10px] font-semibold uppercase tracking-[0.08em] text-ink-muted">To (debits)</span>
					{#each debits as posting (posting.id)}
						<div class="flex flex-col gap-0.5 border-l-2 border-positive-line bg-surface-sunken py-2 pl-3 pr-3">
							<span class="text-sm text-ink-body">{accountLabel(posting.account_id)}</span>
							{#if posting.note}
								<span class="text-[11px] text-ink-muted">{posting.note}</span>
							{/if}
							<span class="mt-1 font-mono text-sm tabular-nums text-ink">{formatAmount(posting.amount)}</span>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</section>
</div>
