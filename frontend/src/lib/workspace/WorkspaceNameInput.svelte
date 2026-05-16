<script lang="ts">
	import { untrack } from 'svelte';

	let {
		initial = '',
		placeholder = '',
		busy = false,
		onCommit,
		onCancel
	}: {
		initial?: string;
		placeholder?: string;
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
</script>

<form
	class="flex flex-1 items-center gap-1"
	onsubmit={(e) => {
		e.preventDefault();
		onCommit(value);
	}}
>
	<input
		type="text"
		bind:value
		bind:this={el}
		{placeholder}
		onkeydown={(e) => {
			if (e.key === 'Escape') onCancel();
		}}
		disabled={busy}
		class="flex-1 border border-border bg-surface-card px-1.5 py-0.5 text-sm"
	/>
	<button
		type="submit"
		disabled={busy}
		class="rounded border border-border bg-surface-card px-1.5 py-0.5 text-xs text-ink hover:bg-surface-hover disabled:opacity-50"
	>
		Confirm
	</button>
</form>
