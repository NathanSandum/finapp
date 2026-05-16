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
	class="grid min-w-0 flex-1 grid-cols-[1fr_4rem] items-center gap-1"
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
		class="h-6 w-full min-w-0 border border-ink-inverse/20 bg-surface-inverse px-1.5 text-sm text-ink-inverse placeholder:text-ink-inverse-muted focus:border-brand focus:outline-none disabled:opacity-50"
	/>
	<button
		type="submit"
		disabled={busy}
		class="h-6 w-full rounded border border-ink-inverse/20 bg-surface-inverse text-xs leading-none text-ink-inverse hover:bg-surface-overlay-hover disabled:opacity-50"
	>
		Confirm
	</button>
</form>
