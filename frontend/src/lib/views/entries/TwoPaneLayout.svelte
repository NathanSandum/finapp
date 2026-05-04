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
	<aside class="overflow-y-auto rounded-lg border border-neutral-200 bg-white">
		<ul class="divide-y divide-neutral-100">
			{#each entries as entry (entry.id)}
				<li>
					<button
						type="button"
						onclick={() => (selectedId = entry.id)}
						class="flex w-full flex-col items-start gap-0.5 px-4 py-3 text-left transition-colors {selected?.id === entry.id
							? 'bg-neutral-900 text-white'
							: 'hover:bg-neutral-50'}"
					>
						<span class="flex w-full items-baseline justify-between">
							<span class="font-mono text-xs {selected?.id === entry.id ? 'text-neutral-300' : 'text-neutral-500'}">{entry.date}</span>
							<span class="font-mono text-xs tabular-nums {selected?.id === entry.id ? 'text-white' : 'text-neutral-700'}">
								{formatAmount(entryAmount(entry))}
							</span>
						</span>
						<span class="text-sm font-medium">{entry.description}</span>
					</button>
				</li>
			{/each}
		</ul>
	</aside>

	<section class="overflow-y-auto rounded-lg border border-neutral-200 bg-white p-6">
		{#if selected}
			<header class="mb-4 flex items-baseline justify-between">
				<div>
					<h3 class="text-lg font-semibold text-neutral-900">{selected.description}</h3>
					<p class="font-mono text-xs text-neutral-500">{selected.date} · {selected.id}</p>
				</div>
				<span class="font-mono text-xl tabular-nums text-neutral-900">{formatAmount(entryAmount(selected))}</span>
			</header>

			<table class="w-full text-sm">
				<thead class="text-left text-xs font-medium uppercase tracking-wide text-neutral-500">
					<tr>
						<th class="py-2 pr-4">Account</th>
						<th class="py-2 pr-4">Note</th>
						<th class="py-2 text-right">Amount</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-neutral-100">
					{#each selected.postings as posting (posting.id)}
						<tr>
							<td class="py-2 pr-4 text-neutral-700">{accountLabel(posting.account_id)}</td>
							<td class="py-2 pr-4 text-neutral-500">{posting.note}</td>
							<td class="py-2 text-right font-mono tabular-nums {posting.amount < 0 ? 'text-rose-600' : 'text-emerald-700'}">
								{formatAmount(posting.amount)}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</section>
</div>
