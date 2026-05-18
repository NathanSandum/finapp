<script lang="ts">
	import { fmt } from '$lib/format';
	import Num from '$lib/components/Num.svelte';
	import type { RecentEntry } from './aggregate';

	let { entries }: { entries: RecentEntry[] } = $props();
</script>

<div>
	<h4 class="mb-2 text-[11px] font-semibold uppercase tracking-[0.08em] text-ink-muted">Recent entries</h4>
	{#if entries.length > 0}
		<ul class="divide-y divide-border-faint border border-border-faint">
			{#each entries as entry (entry.id)}
				<li class="flex items-baseline gap-3 px-3 py-2 text-sm">
					<span class="font-mono text-[11px] text-ink-muted">{entry.date}</span>
					<span class="flex-1 truncate text-ink">{entry.description}</span>
					<span class="font-mono tabular-nums {entry.amount < 0 ? 'text-negative' : 'text-ink-body'}">
						<Num text={fmt(entry.amount)} />
					</span>
				</li>
			{/each}
		</ul>
	{:else}
		<p class="border border-dashed border-border-faint px-3 py-4 text-[12px] text-ink-muted">
			No entries on this account.
		</p>
	{/if}
</div>
