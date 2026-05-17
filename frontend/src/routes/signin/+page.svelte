<script lang="ts">
	import SigninSidebar from './SigninSidebar.svelte';
	import SigninBackdrop from './SigninBackdrop.svelte';
	import SigninForm from './SigninForm.svelte';
</script>

<div class="grid h-screen w-screen grid-cols-[240px_1fr] bg-surface-card">
	<SigninSidebar />
	<main class="relative overflow-hidden bg-surface">
		<SigninBackdrop />
	</main>
</div>

<div class="pointer-events-none fixed inset-0 z-10 flex items-center justify-center p-6">
	<SigninForm />
</div>
