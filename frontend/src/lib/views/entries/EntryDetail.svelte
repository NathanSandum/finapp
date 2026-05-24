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

<header class="mb-6 flex items-baseline justify-between gap-4">
	<div class="min-w-0">
		<h3 class="truncate font-display text-2xl font-semibold tracking-tight text-ink">
			{entry.description}
		</h3>
		<p class="mt-1 flex items-center gap-2 font-mono text-[11px] text-ink-muted">
			<span>{entry.date}</span>
			<span class="opacity-40">·</span>
			<span class="truncate">{entry.id}</span>
		</p>
	</div>
	<span class="font-mono text-3xl tabular-nums text-ink"><Num text={fmt(total)} /></span>
</header>

<ul class="flex flex-col gap-2">
	{#each postings as posting, i (posting.id)}
		{@const isDebit = posting.amount > 0}
		<li
			class="posting-row group relative flex items-center gap-4 rounded-xl px-4 py-3 transition-all duration-300"
			class:is-debit={isDebit}
			class:is-credit={!isDebit}
			in:fly={{ y: 8, duration: DURATION.base, delay: i * 50, easing: expoOut }}
		>
			<span class="posting-row__bg pointer-events-none absolute inset-0 rounded-xl"></span>
			<span
				class="relative inline-flex h-6 w-12 items-center justify-center rounded-md text-[10px] font-semibold uppercase tracking-[0.1em] {isDebit
					? 'bg-positive-tint-strong text-positive'
					: 'bg-negative-tint text-negative-strong'}"
			>
				{isDebit ? 'DR' : 'CR'}
			</span>
			<div class="relative flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="truncate text-[13px] font-medium text-ink">
					{accountNames[posting.account_id] ?? posting.account_id}
				</span>
				{#if posting.note}
					<span class="truncate text-[11px] text-ink-muted">{posting.note}</span>
				{/if}
			</div>
			<span class="relative font-mono text-[15px] tabular-nums text-ink">
				<Num text={fmt(Math.abs(posting.amount))} />
			</span>
		</li>
	{/each}
</ul>

<footer class="mt-8 flex items-center justify-end gap-2 border-t border-[var(--color-glass-divider)] pt-5">
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

<style>
	.posting-row__bg {
		background: oklch(100% 0 0 / 0.45);
		backdrop-filter: blur(8px);
		-webkit-backdrop-filter: blur(8px);
		box-shadow:
			inset 0 0 0 1px oklch(100% 0 0 / 0.55),
			0 1px 2px oklch(40% 0.05 260 / 0.05);
		transition: box-shadow 240ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.posting-row.is-debit .posting-row__bg {
		background: linear-gradient(
			90deg,
			oklch(95% 0.04 180 / 0.5) 0%,
			oklch(100% 0 0 / 0.45) 60%
		);
	}
	.posting-row.is-credit .posting-row__bg {
		background: linear-gradient(
			90deg,
			oklch(96.5% 0.018 12 / 0.5) 0%,
			oklch(100% 0 0 / 0.45) 60%
		);
	}
	.posting-row:hover .posting-row__bg {
		box-shadow:
			inset 0 0 0 1px oklch(100% 0 0 / 0.8),
			0 4px 12px -4px oklch(40% 0.05 260 / 0.12);
	}
</style>
