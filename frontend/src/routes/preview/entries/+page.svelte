<script lang="ts">
	import { load, type EntriesData } from '$lib/views/entries/loader';
	import A from '$lib/views/entries/variants/A_Table.svelte';
	import B from '$lib/views/entries/variants/B_TAccount.svelte';
	import C from '$lib/views/entries/variants/C_FlowList.svelte';
	import D from '$lib/views/entries/variants/D_Stacked.svelte';

	let data = $state<EntriesData | null>(null);
	let error = $state<string | null>(null);

	$effect(() => {
		load().then((result) => {
			if (result.status === 'success') data = result.data;
			else if (result.status === 'fail') error = JSON.stringify(result.data);
			else error = result.message;
		});
	});

	const variants = [
		{ letter: 'A', name: 'Classic DR/CR table', Component: A },
		{ letter: 'B', name: 'T-account (centerline)', Component: B },
		{ letter: 'C', name: 'From → To flow', Component: C },
		{ letter: 'D', name: 'Stacked DR/CR badges', Component: D }
	];
</script>

<svelte:head>
	<title>Preview · Entries A/B/C/D</title>
</svelte:head>

<div class="min-h-screen bg-surface text-ink">
	<header class="sticky top-0 z-20 border-b border-border bg-surface-card px-8 py-4">
		<h1 class="font-display text-xl font-semibold">Entries — variants A/B/C/D</h1>
		<p class="text-sm text-ink-muted">All two-pane (list + detail). Differ in how the journal entry is rendered.</p>
	</header>

	{#if error}
		<p class="p-8 text-sm text-negative">Error: {error}</p>
	{:else if !data}
		<p class="p-8 text-sm text-ink-muted">Loading…</p>
	{:else}
		{#each variants as variant (variant.letter)}
			<section class="border-b border-border">
				<div class="flex items-baseline justify-between px-8 pt-8 pb-4">
					<h2 class="font-mono text-3xl font-bold tracking-tight text-brand">{variant.letter}</h2>
					<span class="font-mono text-[10px] uppercase tracking-[0.08em] text-ink-muted">{variant.name}</span>
				</div>
				<div class="px-8 pb-8">
					<variant.Component entries={data.entries} accountNames={data.accountNames} />
				</div>
			</section>
		{/each}
	{/if}
</div>
