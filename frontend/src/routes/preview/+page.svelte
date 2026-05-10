<script lang="ts">
	const variants = [
		{ key: 'a', label: 'A — Default (current)' },
		{ key: 'b', label: 'B — Cool / Slate' },
		{ key: 'c', label: 'C — Warm / Earthy' }
	];

	const tabs = [
		{ id: 'overview', label: 'Overview' },
		{ id: 'entries', label: 'Entries' },
		{ id: 'income', label: 'Income' },
		{ id: 'balance-sheet', label: 'Balance Sheet' },
		{ id: 'cash-flow', label: 'Cash Flow' }
	];

	let activeTab = $state('income');
</script>

<svelte:head>
	<title>A/B Preview — Palette</title>
</svelte:head>

<div class="min-h-screen bg-neutral-100 p-6">
	<header class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-xl font-semibold">Palette A/B</h1>
			<p class="mt-1 text-sm text-neutral-600">
				Three palette variants rendered side-by-side. Pick a tab to preview that view across all
				three.
			</p>
		</div>
		<nav class="flex gap-1 rounded-lg border border-neutral-300 bg-white p-1">
			{#each tabs as tab}
				<button
					type="button"
					onclick={() => (activeTab = tab.id)}
					class="rounded px-3 py-1.5 text-sm transition-colors {activeTab === tab.id
						? 'bg-neutral-900 text-white'
						: 'text-neutral-700 hover:bg-neutral-100'}"
				>
					{tab.label}
				</button>
			{/each}
		</nav>
	</header>

	<div class="grid grid-cols-3 gap-4">
		{#each variants as v}
			<section class="flex flex-col">
				<header class="mb-2 flex items-baseline justify-between">
					<h2 class="font-semibold text-neutral-900">{v.label}</h2>
					<a
						href="/?theme={v.key}&tab={activeTab}"
						target="_blank"
						rel="noopener"
						class="text-xs text-neutral-500 hover:text-neutral-900"
					>
						open ↗
					</a>
				</header>
				<div
					class="overflow-hidden rounded-lg border border-neutral-300 bg-white shadow-sm"
					style:height="900px"
				>
					<iframe
						title={v.label}
						src="/?theme={v.key}&tab={activeTab}"
						style:width="1440px"
						style:height="1800px"
						style:border="0"
						style:transform="scale(0.5)"
						style:transform-origin="0 0"
					></iframe>
				</div>
			</section>
		{/each}
	</div>
</div>
