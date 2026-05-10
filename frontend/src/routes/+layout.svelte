<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { page } from '$app/state';

	let { children } = $props();

	let theme = $derived.by(() => {
		const t = page.url.searchParams.get('theme');
		return t === 'b' || t === 'c' ? t : null;
	});

	$effect(() => {
		if (theme) document.documentElement.setAttribute('data-theme', theme);
		else document.documentElement.removeAttribute('data-theme');
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>FinApp</title>
</svelte:head>

{@render children()}
