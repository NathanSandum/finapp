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
		return entry.postings
			.filter((p) => p.amount > 0)
			.reduce((sum, p) => sum + p.amount, 0);
	}

	function accountLabel(id: string): string {
		return accountNames[id] ?? id;
	}

	function shortDate(iso: string): string {
		return iso.slice(5);
	}

	const debits = $derived(selected ? selected.postings.filter((p) => p.amount > 0) : []);
	const credits = $derived(selected ? selected.postings.filter((p) => p.amount < 0) : []);
	const totalDebit = $derived(debits.reduce((s, p) => s + p.amount, 0));
	const totalCredit = $derived(credits.reduce((s, p) => s + Math.abs(p.amount), 0));
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

			<div class="px-6 py-4">
				<table class="w-full border-collapse text-sm">
					<thead>
						<tr class="border-b border-border text-[10px] font-semibold uppercase tracking-[0.08em] text-ink-muted">
							<th class="py-2 pr-4 text-left">Account</th>
							<th class="py-2 px-4 text-right">Debit</th>
							<th class="py-2 pl-4 text-right">Credit</th>
						</tr>
					</thead>
					<tbody>
						{#each debits as posting (posting.id)}
							<tr class="border-b border-border-faint">
								<td class="py-2 pr-4 text-ink-body">
									{accountLabel(posting.account_id)}
									{#if posting.note}
										<span class="ml-2 text-xs text-ink-muted">— {posting.note}</span>
									{/if}
								</td>
								<td class="py-2 px-4 text-right font-mono tabular-nums text-ink">{formatAmount(posting.amount)}</td>
								<td class="py-2 pl-4 text-right font-mono text-ink-faint">·</td>
							</tr>
						{/each}
						{#each credits as posting (posting.id)}
							<tr class="border-b border-border-faint">
								<td class="py-2 pr-4 pl-6 text-ink-body">
									{accountLabel(posting.account_id)}
									{#if posting.note}
										<span class="ml-2 text-xs text-ink-muted">— {posting.note}</span>
									{/if}
								</td>
								<td class="py-2 px-4 text-right font-mono text-ink-faint">·</td>
								<td class="py-2 pl-4 text-right font-mono tabular-nums text-ink">{formatAmount(Math.abs(posting.amount))}</td>
							</tr>
						{/each}
					</tbody>
					<tfoot>
						<tr class="text-[10px] font-semibold uppercase tracking-[0.08em] text-ink-muted">
							<td class="pt-3 pr-4 text-right">Total</td>
							<td class="pt-3 px-4 text-right font-mono tabular-nums text-ink">{formatAmount(totalDebit)}</td>
							<td class="pt-3 pl-4 text-right font-mono tabular-nums text-ink">{formatAmount(totalCredit)}</td>
						</tr>
					</tfoot>
				</table>
			</div>
		{/if}
	</section>
</div>
