<script lang="ts">
	import type { Workspace } from './api';
	import WorkspaceNameInput from './WorkspaceNameInput.svelte';

	let {
		workspace,
		busy,
		onCommit,
		onCancel
	}: {
		workspace: Workspace;
		busy: boolean;
		onCommit: (name: string) => void;
		onCancel: () => void;
	} = $props();
</script>

<div class="flex items-center gap-3 bg-surface-overlay-active px-4 py-2">
	<span class="flex h-6 w-6 shrink-0 items-center justify-center rounded bg-ink-inverse/10 text-[10px] font-semibold text-ink-inverse-body">
		{workspace.name.charAt(0).toUpperCase()}
	</span>
	<WorkspaceNameInput initial={workspace.name} {busy} {onCommit} {onCancel} />
</div>
