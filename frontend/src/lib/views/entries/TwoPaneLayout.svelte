<script lang="ts">
	import type { Entry } from './loader';
	import { formatAmount } from './loader';

	let {
		entries,
		accountNames
	}: { entries: Entry[]; accountNames: Record<string, string> } = $props();

	let selectedId = $state<string | null>(null);
	const selected = $derived(entries.find((entry) => entry.id === selectedId) ?? entries[0]);

	function entryAmount(entry: Entry): number {
		return entry.postings
			.filter((posting) => posting.amount > 0)
			.reduce((sum, posting) => sum + posting.amount, 0);
	}

	function accountLabel(id: string): string {
		return accountNames[id] ?? id;
	}
</script>

<div class="grid h-[calc(100vh-160px)] grid-cols-[320px_1fr] gap-4 overflow-hidden">
	<aside class="overflow-y-auto rounded-lg border border-border bg-surface-card">
		<ul class="divide-y divide-border-faint">
			{#each entries as entry (entry.id)}
				<li>
					<button
						type="button"
						onclick={() => (selectedId = entry.id)}
						class="flex w-full flex-col items-start gap-0.5 px-4 py-3 text-left transition-colors {selected?.id === entry.id
							? 'bg-surface-inverse text-ink-inverse'
							: 'hover:bg-surface-hover-faint'}"
					>
						<span class="flex w-full items-baseline justify-between">
							<span class="font-mono text-xs {selected?.id === entry.id ? 'text-ink-inverse-muted' : 'text-ink-muted'}">{entry.date}</span>
							<span class="font-mono text-xs tabular-nums {selected?.id === entry.id ? 'text-ink-inverse' : 'text-ink-body'}">
								{formatAmount(entryAmount(entry))}
							</span>
						</span>
						<span class="text-sm font-medium">{entry.description}</span>
					</button>
				</li>
			{/each}
		</ul>
	</aside>

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
							<td class="py-2 text-right font-mono tabular-nums {posting.amount < 0 ? 'text-negative-strong' : 'text-positive'}">
								{formatAmount(posting.amount)}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</section>
</div>
