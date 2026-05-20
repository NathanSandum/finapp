<script lang="ts">
	import { cache } from '$lib/api/cache.svelte';
	import { workspaceState } from './state.svelte';
	import { createWorkspace, listWorkspaces, renameWorkspace, type Workspace } from './api';
	import { createRecurringRun } from '$lib/views/recurring/api';
	import WorkspaceRow from './WorkspaceRow.svelte';
	import WorkspaceCreate from './WorkspaceCreate.svelte';
	import Popover from '$lib/components/Popover.svelte';

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

	async function syncRecurringAfterSwitch() {
		const result = await createRecurringRun();
		if (result.status === 'success' && result.data.posted > 0) {
			cache.invalidateAll();
		} else if (result.status !== 'success') {
			console.warn('recurring run failed', result);
		}
	}

	async function switchTo(workspace: Workspace) {
		if (workspace.workspace === workspaceState.id()) {
			onClose();
			return;
		}
		workspaceState.setActive(workspace);
		onClose();
		await syncRecurringAfterSwitch();
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
			await syncRecurringAfterSwitch();
		} else {
			error = result.status === 'error' ? result.message : 'Create failed';
		}
	}
</script>

<Popover direction="down" {onClose}>
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
</Popover>
