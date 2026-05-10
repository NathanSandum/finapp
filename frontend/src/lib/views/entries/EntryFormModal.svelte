<script lang="ts">
	import EntryForm from './EntryForm.svelte';
	import type { Account } from './loader';

	let {
		accounts,
		onClose,
		onCreated
	}: {
		accounts: Account[];
		onClose: () => void;
		onCreated?: () => void;
	} = $props();

	function backdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget) onClose();
	}

	function escClose(event: KeyboardEvent) {
		if (event.key === 'Escape') onClose();
	}
</script>

<svelte:window onkeydown={escClose} />

<div
	role="presentation"
	onclick={backdropClick}
	class="fixed inset-0 z-50 flex items-center justify-center bg-surface-inverse/40 backdrop-blur-sm"
>
	<div class="w-full max-w-md border border-border bg-surface-card p-6 shadow-2xl">
		<header class="mb-4 flex items-baseline justify-between">
			<h3 class="text-lg font-semibold text-ink">New entry</h3>
			<button
				type="button"
				onclick={onClose}
				class="text-xs uppercase tracking-wide text-ink-muted hover:text-ink"
			>
				Close
			</button>
		</header>
		<EntryForm {accounts} {onClose} {onCreated} />
	</div>
</div>
