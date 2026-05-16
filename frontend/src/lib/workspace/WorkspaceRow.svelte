<script lang="ts">
	import type { Workspace } from './api';
	import WorkspaceRowDisplay from './WorkspaceRowDisplay.svelte';
	import WorkspaceRowEdit from './WorkspaceRowEdit.svelte';

	let {
		workspace,
		isActive,
		editing,
		busy,
		onSelect,
		onStartEdit,
		onCommitRename,
		onCancelEdit
	}: {
		workspace: Workspace;
		isActive: boolean;
		editing: boolean;
		busy: boolean;
		onSelect: () => void;
		onStartEdit: () => void;
		onCommitRename: (name: string) => void;
		onCancelEdit: () => void;
	} = $props();
</script>

{#if editing}
	<WorkspaceRowEdit {workspace} {busy} onCommit={onCommitRename} onCancel={onCancelEdit} />
{:else}
	<WorkspaceRowDisplay {workspace} {isActive} {onSelect} {onStartEdit} />
{/if}
