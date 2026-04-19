<script lang="ts">
	import './layout.css';
	import { onMount } from 'svelte';
	import { loadToken } from '$lib/api.svelte';

	let { children } = $props();

	onMount(() => {
		loadToken();
	});
</script>

{@render children()}
