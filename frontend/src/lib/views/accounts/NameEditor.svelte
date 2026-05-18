<script lang="ts">
	import { untrack } from 'svelte';

	let {
		initial,
		busy = false,
		onCommit,
		onCancel
	}: {
		initial: string;
		busy?: boolean;
		onCommit: (name: string) => void;
		onCancel: () => void;
	} = $props();

	let value = $state(untrack(() => initial));
	let el = $state<HTMLInputElement | null>(null);

	$effect(() => {
		if (el) {
			el.focus();
			el.select();
		}
	});

	function commit() {
		onCommit(value);
	}
</script>

<input
	type="text"
	bind:value
	bind:this={el}
	onblur={onCancel}
	onkeydown={(e) => {
		if (e.key === 'Enter') commit();
		if (e.key === 'Escape') onCancel();
	}}
	disabled={busy}
	class="flex-1 border border-border bg-surface-card px-1.5 py-0.5 text-sm"
/>
