<script lang="ts">
	import { load, type EntriesData } from '$lib/views/entries/loader';
	import LedgerTwoCol from '$lib/views/entries/variants/LedgerTwoCol.svelte';
	import JournalFeed from '$lib/views/entries/variants/JournalFeed.svelte';

	let data = $state<EntriesData | null>(null);
	let error = $state<string | null>(null);

	$effect(() => {
		load().then((result) => {
			if (result.status === 'success') data = result.data;
			else if (result.status === 'fail') error = JSON.stringify(result.data);
			else error = result.message;
		});
	});
</script>

<svelte:head>
	<title>Preview · Entries A/B</title>
</svelte:head>

<div class="min-h-screen bg-surface text-ink">
	<header class="border-b border-border bg-surface-card px-8 py-4">
		<h1 class="font-display text-xl font-semibold">Entries — A/B preview</h1>
		<p class="text-sm text-ink-muted">A: two-column ledger · B: single-column journal feed</p>
	</header>

	{#if error}
		<p class="p-8 text-sm text-negative">Error: {error}</p>
	{:else if !data}
		<p class="p-8 text-sm text-ink-muted">Loading…</p>
	{:else}
		<section class="border-b border-border">
			<div class="flex items-baseline justify-between px-8 pt-8 pb-4">
				<h2 class="font-mono text-3xl font-bold tracking-tight text-brand">A</h2>
				<span class="font-mono text-[10px] uppercase tracking-[0.08em] text-ink-muted">Two-column ledger</span>
			</div>
			<div class="px-8 pb-8">
				<LedgerTwoCol entries={data.entries} accountNames={data.accountNames} />
			</div>
		</section>

		<section>
			<div class="flex items-baseline justify-between px-8 pt-8 pb-4">
				<h2 class="font-mono text-3xl font-bold tracking-tight text-brand">B</h2>
				<span class="font-mono text-[10px] uppercase tracking-[0.08em] text-ink-muted">Single-column journal feed</span>
			</div>
			<div class="px-8 pb-8">
				<JournalFeed entries={data.entries} accountNames={data.accountNames} />
			</div>
		</section>
	{/if}
</div>
