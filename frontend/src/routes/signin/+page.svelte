<script lang="ts">
	import { goto } from '$app/navigation';
	import { signIn, type UserPayload } from '$lib/api/auth';
	import { authState } from '$lib/api/state.svelte';
	import type { JSend } from '$lib/tabs/types';
	import FailView from '$lib/components/FailView.svelte';
	import ErrorView from '$lib/components/ErrorView.svelte';

	let email = $state('');
	let password = $state('');
	let result = $state<JSend<UserPayload> | null>(null);
	let submitting = $state(false);

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		submitting = true;
		result = await signIn(email, password);
		submitting = false;
		if (result.status === 'success') {
			authState.set(result.data.user);
			goto('/');
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center bg-surface p-6">
	<form
		onsubmit={handleSubmit}
		class="flex w-full max-w-sm flex-col gap-4 rounded border border-stroke bg-surface-raised p-6"
	>
		<h1 class="font-display text-xl text-ink">Sign in</h1>

		<label class="flex flex-col gap-1 text-sm">
			<span class="text-ink-muted">Email</span>
			<input
				type="email"
				bind:value={email}
				required
				autocomplete="email"
				class="rounded border border-stroke bg-surface px-2 py-1.5 text-sm text-ink focus:border-brand focus:outline-none"
			/>
		</label>

		<label class="flex flex-col gap-1 text-sm">
			<span class="text-ink-muted">Password</span>
			<input
				type="password"
				bind:value={password}
				required
				autocomplete="current-password"
				class="rounded border border-stroke bg-surface px-2 py-1.5 text-sm text-ink focus:border-brand focus:outline-none"
			/>
		</label>

		{#if result?.status === 'fail'}
			<FailView {result} />
		{:else if result?.status === 'error'}
			<ErrorView {result} />
		{/if}

		<button
			type="submit"
			disabled={submitting}
			class="rounded bg-brand px-3 py-1.5 text-sm font-medium text-ink hover:bg-brand-hover disabled:opacity-50"
		>
			{submitting ? 'Signing in…' : 'Sign in'}
		</button>
	</form>
</div>
