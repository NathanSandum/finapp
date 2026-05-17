<script lang="ts">
	import { goto } from '$app/navigation';
	import { logout } from '$lib/api/auth';
	import { authState } from '$lib/api/state.svelte';
	import { cache } from '$lib/api/cache.svelte';
	import { workspaceState } from '$lib/workspace/state.svelte';
	import Popover from './Popover.svelte';

	let { onClose }: { onClose: () => void } = $props();

	async function handleSignOut() {
		onClose();
		await logout();
		authState.clear();
		workspaceState.clear();
		cache.invalidateAll();
		goto('/signin');
	}
</script>

<Popover direction="up" {onClose}>
	<button
		type="button"
		role="menuitem"
		onclick={handleSignOut}
		class="block w-full px-4 py-2 text-left text-sm text-ink-inverse-body hover:bg-surface-inverse-hover hover:text-ink-inverse"
	>
		Sign out
	</button>
</Popover>
