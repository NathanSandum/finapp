<script lang="ts">
	import { flip } from 'svelte/animate';
	import { fmt } from '$lib/format';
	import Num from '$lib/components/Num.svelte';
	import type { RecentEntry } from './aggregate';
	import { DURATION, rowEnter, rowExit } from '$lib/utils/motion';

	let { entries }: { entries: RecentEntry[] } = $props();
</script>

<div>
	<h4 class="mb-2 font-ui text-[10px] font-semibold uppercase tracking-[0.12em] text-ink-muted">
		Recent entries
	</h4>
	{#if entries.length > 0}
		<ul class="glass-faint flex flex-col gap-px rounded-xl p-1">
			{#each entries as entry (entry.id)}
				<li
					animate:flip={{ duration: DURATION.base }}
					in:rowEnter
					out:rowExit
					class="recent-row group relative flex items-baseline gap-3 rounded-lg px-3 py-2 text-[13px] transition-all duration-300"
				>
					<span class="recent-row__sheen pointer-events-none absolute inset-0 rounded-lg"></span>
					<span class="relative font-mono text-[11px] text-ink-muted">{entry.date}</span>
					<span class="relative flex-1 truncate text-ink">{entry.description}</span>
					<span
						class="relative font-mono tabular-nums {entry.amount < 0
							? 'text-negative'
							: 'text-ink'}"
					>
						<Num text={fmt(entry.amount)} />
					</span>
				</li>
			{/each}
		</ul>
	{:else}
		<p class="glass-faint rounded-xl px-4 py-5 text-center text-[12px] text-ink-muted">
			No entries on this account.
		</p>
	{/if}
</div>

<style>
	.recent-row__sheen {
		opacity: 0;
		background: linear-gradient(
			90deg,
			oklch(75% 0.14 268 / 0.08) 0%,
			transparent 80%
		);
		transition: opacity 240ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.recent-row:hover .recent-row__sheen {
		opacity: 1;
	}
</style>
