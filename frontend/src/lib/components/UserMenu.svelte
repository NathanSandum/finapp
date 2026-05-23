<script lang="ts">
	import { logout } from '$lib/auth/session';
	import Popover from './Popover.svelte';

	let { onClose }: { onClose: () => void } = $props();

	async function handleSignOut() {
		onClose();
		await logout();
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
