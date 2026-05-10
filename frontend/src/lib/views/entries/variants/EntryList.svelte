<script lang="ts">
	import type { Entry } from '../loader';
	import { formatAmount } from '../loader';

	let {
		entries,
		selectedId,
		onSelect
	}: {
		entries: Entry[];
		selectedId: string | null;
		onSelect: (id: string) => void;
	} = $props();

	function entryAmount(entry: Entry): number {
		return entry.postings.filter((p) => p.amount > 0).reduce((s, p) => s + p.amount, 0);
	}
</script>

<aside class="overflow-y-auto rounded-lg border border-border bg-surface-card">
	<ul class="divide-y divide-border-faint">
		{#each entries as entry (entry.id)}
			{@const isSel = selectedId === entry.id}
			<li>
				<button
					type="button"
					onclick={() => onSelect(entry.id)}
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
							{formatAmount(entryAmount(entry))}
						</span>
					</span>
					<span class="text-sm font-medium">{entry.description}</span>
				</button>
			</li>
		{/each}
	</ul>
</aside>
