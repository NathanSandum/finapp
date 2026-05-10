<script lang="ts">
	import type { Entry } from './loader';
	import { entryAmount, formatAmount } from './loader';

	let {
		entries,
		selectedId,
		onSelect
	}: {
		entries: Entry[];
		selectedId: string | null;
		onSelect: (id: string) => void;
	} = $props();
</script>

<ul class="h-full divide-y divide-border-faint overflow-y-auto">
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
					<span class="font-mono text-xs tabular-nums {isSel ? 'text-ink-inverse' : 'text-ink-body'}">
						{formatAmount(entryAmount(entry))}
					</span>
				</span>
				<span class="text-sm font-medium">{entry.description}</span>
			</button>
		</li>
	{/each}
</ul>
