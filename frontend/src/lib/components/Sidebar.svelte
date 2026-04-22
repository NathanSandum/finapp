<script lang="ts">
	type View = 'Income Statement' | 'Balance Sheet' | 'Cash Flow' | 'Entries' | 'Settings';

	let { activeView = $bindable(), }: { activeView: View } = $props();

	const primaryViews: View[] = ['Income Statement', 'Balance Sheet', 'Cash Flow'];
	const secondaryViews: View[] = ['Entries'];
	const tertiaryViews: View[] = ['Settings'];
</script>

<aside class="flex h-screen w-48 shrink-0 flex-col border-r border-gray-800 bg-gray-900">
	<div class="border-b border-gray-800 px-4 py-3">
		<h1 class="text-sm font-semibold tracking-wide text-gray-100">FinApp</h1>
	</div>

	<nav class="flex flex-1 flex-col py-1.5">
		{#each primaryViews as view}
			<button
				class="px-4 py-2 text-left text-xs transition-colors {activeView === view
					? 'border-r-2 border-blue-500 bg-blue-950/30 font-medium text-white'
					: 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-300'}"
				onclick={() => (activeView = view)}
			>
				{view}
			</button>
		{/each}

		<div class="mx-4 my-1.5 border-t border-gray-800"></div>

		{#each secondaryViews as view}
			<button
				class="px-4 py-2 text-left text-xs transition-colors {activeView === view
					? 'border-r-2 border-blue-500 bg-blue-950/30 font-medium text-white'
					: 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-300'}"
				onclick={() => (activeView = view)}
			>
				{view}
			</button>
		{/each}

		<div class="mx-4 my-1.5 border-t border-gray-800"></div>

		{#each tertiaryViews as view}
			<button
				class="px-4 py-2 text-left text-xs transition-colors {activeView === view
					? 'border-r-2 border-blue-500 bg-blue-950/30 font-medium text-white'
					: 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-300'}"
				onclick={() => (activeView = view)}
			>
				{view}
			</button>
		{/each}
	</nav>
</aside>
