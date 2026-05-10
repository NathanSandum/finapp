<script lang="ts">
	import type { Entry } from '../loader';
	import { formatAmount } from '../loader';

	let {
		entries,
		accountNames
	}: { entries: Entry[]; accountNames: Record<string, string> } = $props();

	let expandedId = $state<string | null>(null);

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

	function toggle(id: string) {
		expandedId = expandedId === id ? null : id;
	}
</script>

<div class="h-[calc(100vh-160px)] overflow-y-auto border border-border bg-surface-card">
	<div class="sticky top-0 z-10 grid grid-cols-[80px_1fr_140px] gap-4 border-b border-border bg-surface-sunken px-6 py-2 text-[10px] font-semibold uppercase tracking-[0.08em] text-ink-muted">
		<span>Date</span>
		<span>Description</span>
		<span class="text-right">Amount</span>
	</div>

	<ul>
		{#each entries as entry (entry.id)}
			{@const isOpen = expandedId === entry.id}
			{@const debits = entry.postings.filter((p) => p.amount > 0)}
			{@const credits = entry.postings.filter((p) => p.amount < 0)}
			{@const totalDebit = debits.reduce((s, p) => s + p.amount, 0)}
			{@const totalCredit = credits.reduce((s, p) => s + Math.abs(p.amount), 0)}
			<li class="border-b border-border-faint">
				<button
					type="button"
					onclick={() => toggle(entry.id)}
					class="grid w-full grid-cols-[80px_1fr_140px] items-baseline gap-4 px-6 py-2.5 text-left transition-colors {isOpen
						? 'bg-surface-sunken'
						: 'hover:bg-surface-hover-faint'}"
				>
					<span class="font-mono text-xs tabular-nums text-ink-muted">{shortDate(entry.date)}</span>
					<span class="truncate text-sm text-ink">{entry.description}</span>
					<span class="text-right font-mono text-xs tabular-nums text-ink-body">{formatAmount(entryAmount(entry))}</span>
				</button>

				{#if isOpen}
					<div class="border-t border-border-faint bg-surface-sunken px-6 pb-4 pt-2">
						<table class="ml-[80px] w-[calc(100%-80px-140px+24px)] max-w-[640px] border-collapse text-sm">
							<thead>
								<tr class="text-[10px] font-semibold uppercase tracking-[0.08em] text-ink-muted">
									<th class="py-1 pr-4 text-left">Account</th>
									<th class="py-1 px-3 text-right">Debit</th>
									<th class="py-1 pl-3 text-right">Credit</th>
								</tr>
							</thead>
							<tbody>
								{#each debits as posting (posting.id)}
									<tr class="border-t border-border-faint">
										<td class="py-1.5 pr-4 text-ink-body">
											{accountLabel(posting.account_id)}
											{#if posting.note}
												<span class="ml-2 text-xs text-ink-muted">— {posting.note}</span>
											{/if}
										</td>
										<td class="py-1.5 px-3 text-right font-mono tabular-nums text-ink">{formatAmount(posting.amount)}</td>
										<td class="py-1.5 pl-3 text-right font-mono text-ink-faint">·</td>
									</tr>
								{/each}
								{#each credits as posting (posting.id)}
									<tr class="border-t border-border-faint">
										<td class="py-1.5 pl-4 pr-4 text-ink-body">
											{accountLabel(posting.account_id)}
											{#if posting.note}
												<span class="ml-2 text-xs text-ink-muted">— {posting.note}</span>
											{/if}
										</td>
										<td class="py-1.5 px-3 text-right font-mono text-ink-faint">·</td>
										<td class="py-1.5 pl-3 text-right font-mono tabular-nums text-ink">{formatAmount(Math.abs(posting.amount))}</td>
									</tr>
								{/each}
								<tr class="border-t border-border text-[10px] font-semibold uppercase tracking-[0.08em] text-ink-muted">
									<td class="pt-2 pr-4 text-right">Total</td>
									<td class="pt-2 px-3 text-right font-mono tabular-nums text-ink">{formatAmount(totalDebit)}</td>
									<td class="pt-2 pl-3 text-right font-mono tabular-nums text-ink">{formatAmount(totalCredit)}</td>
								</tr>
							</tbody>
						</table>
					</div>
				{/if}
			</li>
		{/each}
	</ul>
</div>
