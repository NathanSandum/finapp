<script lang="ts">
	import { goto } from '$app/navigation';
	import { prefersReducedMotion } from '$lib/utils/motion';
	import SigninBackdrop from './SigninBackdrop.svelte';
	import SigninForm from './SigninForm.svelte';

	let exiting = $state(false);

	async function onSignedIn() {
		exiting = true;
		const outroMs = prefersReducedMotion() ? 0 : 700;
		await new Promise((r) => setTimeout(r, outroMs));
		goto('/');
	}
</script>

<main class="relative overflow-hidden bg-surface">
	<SigninBackdrop {exiting} />
</main>

<div class="pointer-events-none fixed inset-0 z-10 flex items-center justify-center p-6">
	<SigninForm {onSignedIn} />
</div>
