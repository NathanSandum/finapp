<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import Sidebar from '$lib/components/Sidebar.svelte';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>FinApp</title>
</svelte:head>

<div class="grid h-screen w-screen grid-cols-[240px_1fr] bg-surface-card text-ink">
	<aside class="flex flex-col bg-surface-inverse p-3 text-ink-inverse-body select-none">
		<Sidebar />
	</aside>
	{@render children()}
</div>
