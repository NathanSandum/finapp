<script lang="ts">
	import SigninBackdrop from './SigninBackdrop.svelte';
	import SigninForm from './SigninForm.svelte';
</script>

<main class="relative overflow-hidden bg-surface">
	<SigninBackdrop />
</main>

<div class="pointer-events-none fixed inset-0 z-10 flex items-center justify-center p-6">
	<SigninForm />
</div>
