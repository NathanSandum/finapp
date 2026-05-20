<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>FinApp</title>
</svelte:head>

<div class="flex h-screen w-screen flex-col bg-surface text-ink">
	{@render children()}
</div>
