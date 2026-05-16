<script lang="ts">
	import WorkspaceNameInput from './WorkspaceNameInput.svelte';

	let {
		busy,
		onCommit,
		onCancel
	}: {
		busy: boolean;
		onCommit: (name: string) => void;
		onCancel: () => void;
	} = $props();
</script>

<div class="flex h-full items-center gap-1 px-4">
	<WorkspaceNameInput placeholder="new workspace name" {busy} {onCommit} {onCancel} />
</div>
