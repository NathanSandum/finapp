<script lang="ts">
	let {
		busy = false,
		onCommit,
		onCancel
	}: {
		busy?: boolean;
		onCommit: (name: string) => void;
		onCancel: () => void;
	} = $props();

	let value = $state('');
	let el = $state<HTMLInputElement | null>(null);

	$effect(() => {
		if (el) el.focus();
	});

	function commit() {
		onCommit(value);
	}
</script>

<input
	type="text"
	bind:value
	bind:this={el}
	placeholder="new account name"
	onblur={commit}
	onkeydown={(e) => {
		if (e.key === 'Enter') commit();
		if (e.key === 'Escape') onCancel();
	}}
	disabled={busy}
	class="flex-1 border border-border bg-surface-card px-1.5 py-0.5 text-sm"
/>
