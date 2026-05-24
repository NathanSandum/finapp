<script lang="ts">
	import { fly } from 'svelte/transition';
	import { expoOut } from 'svelte/easing';
	import { fmt } from '$lib/format';
	import Num from '$lib/components/Num.svelte';
	import BorderButton from '$lib/components/BorderButton.svelte';
	import type { Entry } from '$lib/data';
	import { entryAmount } from '$lib/data';
	import { DURATION } from '$lib/utils/motion';

	let {
		entry,
		accountNames,
		onDelete
	}: {
		entry: Entry;
		accountNames: Record<string, string>;
		onDelete: () => void | Promise<void>;
	} = $props();

	const postings = $derived(
		[...entry.postings].sort((a, b) => (b.amount > 0 ? 1 : 0) - (a.amount > 0 ? 1 : 0))
	);
	const total = $derived(entryAmount(entry));

	let confirming = $state(false);
	let removing = $state(false);

	async function commitDelete() {
		removing = true;
		try {
			await onDelete();
		} finally {
			removing = false;
			confirming = false;
		}
	}
</script>

<header class="mb-5 flex items-baseline justify-between gap-4">
	<div class="min-w-0">
		<h3 class="truncate font-display text-lg font-semibold text-ink">{entry.description}</h3>
		<p class="mt-0.5 flex items-center gap-2 font-mono text-[11px] text-ink-muted">
			<span>{entry.date}</span>
			<span class="opacity-50">·</span>
			<span class="truncate">{entry.id}</span>
		</p>
	</div>
	<span class="font-mono text-2xl tabular-nums text-ink"><Num text={fmt(total)} /></span>
</header>

<ul class="divide-y divide-border-faint border-y border-border-faint">
	{#each postings as posting, i (posting.id)}
		{@const isDebit = posting.amount > 0}
		<li
			class="flex items-center gap-4 px-1 py-3"
			in:fly={{ y: 6, duration: DURATION.base, delay: i * 35, easing: expoOut }}
		>
			<span
				class="inline-flex h-5 w-10 items-center justify-center text-[10px] font-semibold uppercase tracking-[0.08em] {isDebit
					? 'bg-positive-tint-strong text-positive'
					: 'bg-negative-tint text-negative-strong'}"
			>
				{isDebit ? 'DR' : 'CR'}
			</span>
			<div class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="truncate text-[13px] text-ink">
					{accountNames[posting.account_id] ?? posting.account_id}
				</span>
				{#if posting.note}
					<span class="truncate text-[11px] text-ink-muted">{posting.note}</span>
				{/if}
			</div>
			<span
				class="font-mono text-[14px] tabular-nums {isDebit ? 'text-ink' : 'text-ink-body'}"
			>
				<Num text={fmt(Math.abs(posting.amount))} />
			</span>
		</li>
	{/each}
</ul>

<footer class="mt-6 flex items-center justify-end gap-2">
	{#if confirming}
		<span class="text-[11px] text-ink-muted">Delete this entry?</span>
		<BorderButton onclick={() => (confirming = false)} disabled={removing}>cancel</BorderButton>
		<BorderButton tone="negative" onclick={commitDelete} disabled={removing}>
			{removing ? 'deleting…' : 'confirm'}
		</BorderButton>
	{:else}
		<BorderButton tone="negative" onclick={() => (confirming = true)}>delete</BorderButton>
	{/if}
</footer>
