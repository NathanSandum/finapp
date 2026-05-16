<script lang="ts">
	import WorkspaceCreateButton from './WorkspaceCreateButton.svelte';
	import WorkspaceCreateForm from './WorkspaceCreateForm.svelte';

	let {
		creating,
		busy,
		onStart,
		onCommit,
		onCancel
	}: {
		creating: boolean;
		busy: boolean;
		onStart: () => void;
		onCommit: (name: string) => void;
		onCancel: () => void;
	} = $props();
</script>

<div class="h-10">
	{#if creating}
		<WorkspaceCreateForm {busy} {onCommit} {onCancel} />
	{:else}
		<WorkspaceCreateButton {onStart} />
	{/if}
</div>
