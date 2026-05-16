<script lang="ts">
	import { workspaceState } from './state.svelte';
	import { createWorkspace, listWorkspaces, renameWorkspace, type Workspace } from './api';
	import WorkspaceNameInput from './WorkspaceNameInput.svelte';

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
	class="absolute left-2 right-2 top-full z-20 mt-1 rounded border border-border bg-surface-card text-ink shadow-lg"
	role="menu"
>
	<ul class="flex flex-col py-1">
		{#each workspaceState.list as workspace (workspace.workspace)}
			{@const isActive = workspace.workspace === workspaceState.id()}
			<li class="flex items-center gap-1 px-1.5">
				{#if editingId === workspace.workspace}
					<WorkspaceNameInput
						initial={workspace.name}
						{busy}
						onCommit={(name) => commitRename(workspace.workspace, name)}
						onCancel={() => (editingId = null)}
					/>
				{:else}
					<button
						type="button"
						onclick={() => switchTo(workspace)}
						class="flex flex-1 items-center gap-2 rounded px-1.5 py-1 text-left text-sm hover:bg-surface-hover"
					>
						<span
							class="flex h-4 w-4 items-center justify-center text-brand"
							aria-hidden="true"
						>
							{#if isActive}
								<svg viewBox="0 0 20 20" fill="currentColor" class="h-3.5 w-3.5">
									<path
										fill-rule="evenodd"
										d="M16.7 5.3a1 1 0 010 1.4l-7.5 7.5a1 1 0 01-1.4 0L3.3 9.7a1 1 0 011.4-1.4L8.5 12 15.3 5.3a1 1 0 011.4 0z"
										clip-rule="evenodd"
									/>
								</svg>
							{/if}
						</span>
						<span class="flex-1 truncate">{workspace.name}</span>
					</button>
					<button
						type="button"
						onclick={() => (editingId = workspace.workspace)}
						class="rounded px-1.5 py-1 text-xs text-ink-muted hover:bg-surface-hover hover:text-ink"
						aria-label="Rename {workspace.name}"
					>
						Rename
					</button>
				{/if}
			</li>
		{/each}
	</ul>

	<div class="border-t border-border px-1.5 py-1">
		{#if creating}
			<WorkspaceNameInput
				placeholder="new workspace name"
				{busy}
				onCommit={commitCreate}
				onCancel={() => (creating = false)}
			/>
		{:else}
			<button
				type="button"
				onclick={() => (creating = true)}
				class="flex w-full items-center gap-2 rounded px-1.5 py-1 text-left text-sm text-ink-muted hover:bg-surface-hover hover:text-ink"
			>
				<span class="flex h-4 w-4 items-center justify-center" aria-hidden="true">+</span>
				<span>New workspace</span>
			</button>
		{/if}
	</div>

	{#if error}
		<p class="px-3 py-1.5 text-xs text-negative">{error}</p>
	{/if}
</div>
