<script lang="ts">
	import { onMount } from 'svelte';
	import { load, type EntriesData } from '$lib/views/entries/loader';
	import EntryForm from '$lib/views/entries/EntryForm.svelte';
	import EntryFormModal from '$lib/views/entries/EntryFormModal.svelte';

	let data = $state<EntriesData | null>(null);
	let loadError = $state<string | null>(null);
	let inlineOpen = $state(true);
	let modalOpen = $state(false);

	const accounts = $derived(
		data ? Object.entries(data.accountNames).map(([id, name]) => ({
			id,
			name,
			parent_id: null,
			account_type: 'asset',
			currency: 'CAD'
		})) : []
	);

	onMount(async () => {
		const result = await load();
		if (result.status === 'success') {
			data = result.data;
		} else if (result.status === 'fail') {
			loadError = `fail: ${JSON.stringify(result.data)}`;
		} else {
			loadError = result.message;
		}
	});
</script>

<div class="min-h-screen bg-surface p-8">
	<header class="mb-6">
		<h1 class="text-2xl font-semibold text-ink">Entry form A/B</h1>
		<p class="text-sm text-ink-muted">Pick A (inline) or B (modal). Submitting writes a real entry.</p>
	</header>

	{#if loadError}
		<p class="text-negative-strong">{loadError}</p>
	{:else if !data}
		<p class="text-ink-muted">Loading…</p>
	{:else}
		<div class="grid gap-6 lg:grid-cols-2">
			<section class="border border-border bg-surface-card">
				<header class="border-b border-border bg-surface-sunken px-4 py-2 text-xs font-semibold uppercase tracking-[0.08em] text-ink-muted">
					Variant A — Inline
				</header>
				<div class="grid grid-cols-[260px_1fr]">
					<div class="border-r border-border-faint bg-surface-sunken/40 p-4 text-xs text-ink-muted">
						(entries list — placeholder)
						<ul class="mt-2 space-y-1 text-ink-body">
							<li>2026-05-01 · Salary</li>
							<li>2026-05-03 · Rent</li>
							<li>2026-05-08 · Groceries</li>
						</ul>
					</div>
					<div class="p-5">
						{#if inlineOpen}
							<header class="mb-3 flex items-baseline justify-between">
								<h3 class="text-base font-semibold text-ink">New entry</h3>
							</header>
							<EntryForm {accounts} onClose={() => (inlineOpen = false)} />
						{:else}
							<button
								type="button"
								onclick={() => (inlineOpen = true)}
								class="bg-surface-inverse px-4 py-2 text-sm font-medium text-ink-inverse"
							>
								+ New entry
							</button>
						{/if}
					</div>
				</div>
			</section>

			<section class="border border-border bg-surface-card">
				<header class="border-b border-border bg-surface-sunken px-4 py-2 text-xs font-semibold uppercase tracking-[0.08em] text-ink-muted">
					Variant B — Modal
				</header>
				<div class="grid grid-cols-[260px_1fr]">
					<div class="border-r border-border-faint bg-surface-sunken/40 p-4 text-xs text-ink-muted">
						(entries list — placeholder)
						<ul class="mt-2 space-y-1 text-ink-body">
							<li>2026-05-01 · Salary</li>
							<li>2026-05-03 · Rent</li>
							<li>2026-05-08 · Groceries</li>
						</ul>
					</div>
					<div class="flex items-start p-5">
						<button
							type="button"
							onclick={() => (modalOpen = true)}
							class="bg-surface-inverse px-4 py-2 text-sm font-medium text-ink-inverse"
						>
							+ New entry
						</button>
					</div>
				</div>
			</section>
		</div>

		{#if modalOpen}
			<EntryFormModal {accounts} onClose={() => (modalOpen = false)} />
		{/if}
	{/if}
</div>
