<script lang="ts">
	import { fade } from 'svelte/transition';

	let { text }: { text: string } = $props();
</script>

{#key text}
	<span class="num-cell" in:fade={{ duration: 220 }} out:fade={{ duration: 120 }}>{text}</span>
{/key}

<style>
	.num-cell {
		display: inline-block;
	}
</style>
