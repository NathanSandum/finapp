<script lang="ts">
	import type { Workspace } from './api';

	let {
		workspace,
		isActive,
		onSelect,
		onStartEdit
	}: {
		workspace: Workspace;
		isActive: boolean;
		onSelect: () => void;
		onStartEdit: () => void;
	} = $props();
</script>

<div
	class="group flex w-full items-center gap-3 text-sm text-ink-inverse-body transition-colors hover:bg-surface-overlay-hover hover:text-ink-inverse {isActive
		? 'bg-surface-overlay-active text-ink-inverse'
		: ''}"
>
	<button type="button" onclick={onSelect} class="flex flex-1 items-center gap-3 px-4 py-2 text-left">
		<span
			class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-[10px] font-semibold {isActive
				? 'bg-brand text-ink'
				: 'bg-ink-inverse/10 text-ink-inverse-body'}"
		>
			{workspace.name.charAt(0).toUpperCase()}
		</span>
		<span class="flex-1 truncate">{workspace.name}</span>
	</button>
	<button
		type="button"
		onclick={onStartEdit}
		class="px-3 py-2 font-mono text-[10px] uppercase tracking-widest text-ink-inverse-muted opacity-0 transition-opacity hover:text-ink-inverse group-hover:opacity-100"
		aria-label="Rename {workspace.name}"
	>
		Edit
	</button>
</div>
