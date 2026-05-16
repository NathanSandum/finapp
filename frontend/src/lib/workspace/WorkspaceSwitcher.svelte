<script lang="ts">
	import { workspaceState } from './state.svelte';
	import { createWorkspace, listWorkspaces, renameWorkspace, type Workspace } from './api';
	import WorkspaceRow from './WorkspaceRow.svelte';
	import WorkspaceCreate from './WorkspaceCreate.svelte';

	let { onClose }: { onClose: () => void } = $props();

	let editingId = $state<string | null>(null);
	let creating = $state(false);
	let busy = $state(false);
	let error = $state<string | null>(null);

	async function refresh() {
		const result = await listWorkspaces();
		if (result.status === 'success') {
			workspaceState.setList(result.data.workspaces);
			error = null;
		} else {
			error = result.status === 'error' ? result.message : 'Failed to load workspaces';
		}
	}

	function switchTo(workspace: Workspace) {
		if (workspace.workspace === workspaceState.id()) {
			onClose();
			return;
		}
		workspaceState.setActive(workspace);
		onClose();
	}

	async function commitRename(id: string, name: string) {
		const trimmed = name.trim();
		if (!trimmed) {
			editingId = null;
			return;
		}
		busy = true;
		const result = await renameWorkspace(id, trimmed);
		busy = false;
		if (result.status === 'success') {
			editingId = null;
			await refresh();
		} else {
			error = result.status === 'error' ? result.message : 'Rename failed';
		}
	}

	async function commitCreate(name: string) {
		const trimmed = name.trim();
		if (!trimmed) {
			creating = false;
			return;
		}
		busy = true;
		const result = await createWorkspace(trimmed);
		busy = false;
		if (result.status === 'success') {
			creating = false;
			await refresh();
			workspaceState.setActive(result.data);
			onClose();
		} else {
			error = result.status === 'error' ? result.message : 'Create failed';
		}
	}
</script>

<div
	class="absolute -inset-x-2 top-full z-20 mt-1 bg-surface-overlay shadow-2xl shadow-black/60 ring-1 ring-inset ring-ink-inverse/25"
	style="box-shadow: inset 0 1px 0 oklch(60% 0.04 260), inset 0 -1px 0 oklch(14% 0.02 260), 0 18px 40px -8px rgba(0,0,0,0.6);"
	role="menu"
>
	<div class="border-y border-ink-inverse/10">
		{#each workspaceState.list as workspace (workspace.workspace)}
			<WorkspaceRow
				{workspace}
				isActive={workspace.workspace === workspaceState.id()}
				editing={editingId === workspace.workspace}
				{busy}
				onSelect={() => switchTo(workspace)}
				onStartEdit={() => (editingId = workspace.workspace)}
				onCommitRename={(name) => commitRename(workspace.workspace, name)}
				onCancelEdit={() => (editingId = null)}
			/>
		{/each}
	</div>

	<WorkspaceCreate
		{creating}
		{busy}
		onStart={() => (creating = true)}
		onCommit={commitCreate}
		onCancel={() => (creating = false)}
	/>

	{#if error}
		<p class="px-4 py-1.5 text-xs text-negative">{error}</p>
	{/if}
</div>
