<script lang="ts">
	import { untrack } from 'svelte';

	let { text }: { text: string } = $props();

	let prev = untrack(() => text);
	let flashing = $state(false);
	let timer: ReturnType<typeof setTimeout> | undefined;

	$effect(() => {
		if (text === prev) return;
		prev = text;
		flashing = true;
		clearTimeout(timer);
		timer = setTimeout(() => (flashing = false), 240);
	});
</script>

<span class="num num-flash" style:opacity={flashing ? 0.6 : 1}>{text}</span>

<style>
	.num {
		display: inline-block;
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}
</style>
